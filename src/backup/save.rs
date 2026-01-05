use crate::{
    Result,
    backup::hash::{Sha256Sum, copy_file_to_storage, copy_to_storage, hash_diff},
    globals::{MACHINE, MINESAVE_HOME},
};
use anyhow::anyhow;
use async_compression::tokio::bufread::ZstdDecoder;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftSave {
    // pub path: PathBuf, // Where the backup is stored
    pub id: String,
    pub name: String,
    pub description: String,
    pub target: HashMap<String, PathBuf>,
    pub latest_version: Option<MinecraftSaveVersion>,
}
impl MinecraftSave {
    pub fn create<P: AsRef<Path>>(id: String, source: P) -> Self {
        Self {
            id,
            name: source
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            description: String::new(),
            target: HashMap::from_iter([(MACHINE.clone(), source.as_ref().to_path_buf())]),
            latest_version: None,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftSaveVersion {
    pub path: PathBuf, // Where the backup is stored
    pub version_type: MinecraftSaveVersionType,
    pub description: String,
    pub prev: Option<Box<Self>>,
}

impl MinecraftSaveVersion {
    pub async fn create<'a, P: AsRef<Path> + 'a>(
        version_type: MinecraftSaveVersionType,
        source: P,
        id: String,
        description: String,
        prev: Option<Box<MinecraftSaveVersion>>,
    ) -> Result<Self> {
        match version_type {
            MinecraftSaveVersionType::Full => {
                Self::create_version_full(source, id, description, prev).await
            }
            MinecraftSaveVersionType::IncreasementFile => {
                Self::create_version_increasement_file(source, id, description, prev).await
            }
            MinecraftSaveVersionType::Default => {
                Self::create_version_increasement_file(source, id, description, prev).await // TODO: Considering add a config for this
            }
            #[allow(unreachable_patterns)]
            _ => todo!(),
        }
    }

    async fn create_version_full<P: AsRef<Path>>(
        source: P,
        id: String,
        description: String,
        prev: Option<Box<MinecraftSaveVersion>>,
    ) -> Result<Self> {
        let path = MINESAVE_HOME.join(id);
        debug!("backup started");
        fs::create_dir_all(path.join("storage")).await?;
        fs::create_dir_all(path.join("versions")).await?;
        let hash = copy_to_storage(&source, path.join("storage")).await?;
        let packed_hash = rmp_serde::to_vec(&hash)?;

        let version_meta = path
            .join("versions")
            .join(UNIX_EPOCH.elapsed().unwrap().as_millis().to_string());

        File::create(&version_meta)
            .await?
            .write_all(&packed_hash)
            .await?;
        debug!("hash created");
        debug!("compressing: zstd level=15");

        Ok(Self {
            path: version_meta,
            description,
            prev,
            version_type: MinecraftSaveVersionType::Full,
        })
    }
    async fn create_version_increasement_file<P: AsRef<Path>>(
        source: P,
        id: String,
        description: String,
        prev: Option<Box<MinecraftSaveVersion>>,
    ) -> Result<Self> {
        if prev.is_none() {
            return Self::create_version_full(source, id, description, prev).await;
        }

        let path = MINESAVE_HOME.join(id);

        let mut packed_hash = vec![];

        File::open(&prev.as_ref().unwrap().path)
            .await?
            .read_to_end(&mut packed_hash)
            .await?;

        let mut hash: HashMap<PathBuf, Sha256Sum> = rmp_serde::from_slice(&packed_hash)?;
        let new_hash = hash_diff(&source, &hash).await?;
        for (relative, item) in new_hash {
            debug!("change detected: {:?}", relative);
            copy_file_to_storage(
                source.as_ref().join(&relative),
                &path,
                relative.to_string_lossy().to_string(),
            )
            .await?;

            debug!("copied: {:?}", relative);
            hash.insert(relative, item);
        }
        let version_meta = path
            .join("versions")
            .join(UNIX_EPOCH.elapsed().unwrap().as_millis().to_string());
        let packed_hash = rmp_serde::to_vec(&hash)?;
        File::create(&version_meta)
            .await?
            .write_all(&packed_hash)
            .await?;

        Ok(Self {
            path: version_meta,
            description,
            prev,
            version_type: MinecraftSaveVersionType::IncreasementFile,
        })
    }
    pub async fn merge(self, save_name: String) -> Result<Self> {
        // TODO More effecient version
        let temp = env::temp_dir().join(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .to_string(),
        );
        self.recover(&temp).await?;
        Self::create(
            MinecraftSaveVersionType::Full,
            &temp,
            save_name,
            format!("Merged from {} and previous versions", self.description),
            None,
        )
        .await
    }
    pub async fn recover<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        if self.version_type == MinecraftSaveVersionType::Full {
            return self.recover_self(target).await;
        }
        self.prev
            .as_ref()
            .ok_or_else(|| {
                anyhow!(
                    "Previous version of backup {} in {:?} not found",
                    self.description,
                    self.path
                )
            })?
            .recover_self(&target)
            .await?;
        self.recover_self(&target).await
    }
    async fn recover_self<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        let mut packed_hash = vec![];
        File::open(&self.path)
            .await?
            .read_to_end(&mut packed_hash)
            .await?;

        let hash: HashMap<PathBuf, Sha256Sum> = rmp_serde::from_slice(&packed_hash)?;
        for (relative, hash) in hash {
            io::copy(
                &mut ZstdDecoder::new(BufReader::new(
                    File::open(
                        &self
                            .path
                            .parent()
                            .unwrap()
                            .join("storage")
                            .join(hash.to_string()),
                    )
                    .await?,
                )),
                &mut File::open(target.as_ref().join(&relative)).await?,
            )
            .await?;
            debug!("recovered: {:?}", relative);
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinecraftSaveVersionType {
    Full = 0,
    IncreasementFile = 1,
    IncreasementData = 2,
    Snapshot = 3,
    Default = 255,
}
