use crate::{
    Result,
    backup::hash::{Sha256Sum, copy_to_storage},
    globals::{MACHINE, MINESAVE_HOME},
};
use anyhow::anyhow;
use async_compression::tokio::bufread::ZstdDecoder;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
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
    pub id: String,
    pub time: Duration,
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
            MinecraftSaveVersionType::Default => {
                Self::create_version_default(source, id, description, prev).await
            }
            MinecraftSaveVersionType::FollowSettings => {
                Self::create_version_default(source, id, description, prev).await // TODO: add a setting for this
            }
            #[allow(unreachable_patterns)]
            _ => todo!(),
        }
    }

    async fn create_version_default<P: AsRef<Path>>(
        source: P,
        id: String,
        description: String,
        prev: Option<Box<MinecraftSaveVersion>>,
    ) -> Result<Self> {
        let path = MINESAVE_HOME.join("versions").join(&id);
        let time = UNIX_EPOCH.elapsed().unwrap();

        debug!("backup: compress=[enabled zstd level=15]");
        fs::create_dir_all(MINESAVE_HOME.join("storage")).await?;
        fs::create_dir_all(&path).await?;
        let hash = copy_to_storage(&source, MINESAVE_HOME.join("storage")).await?;
        let packed_hash = rmp_serde::to_vec(&hash)?;

        let version_meta = path.join(time.as_millis().to_string());

        File::create(&version_meta)
            .await?
            .write_all(&packed_hash)
            .await?;
        debug!("backup: hash created");

        Ok(Self {
            id,
            time,
            description,
            prev,
            version_type: MinecraftSaveVersionType::Default,
        })
    }
    pub async fn recover<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        if self.version_type == MinecraftSaveVersionType::Default {
            return self.recover_self(target).await;
        }
        self.prev
            .as_ref()
            .ok_or_else(|| {
                anyhow!(
                    "Previous version of backup {}, id: {:?} not found",
                    self.description,
                    self.id
                )
            })?
            .recover_self(&target)
            .await?;
        self.recover_self(&target).await
    }
    async fn recover_self<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        let mut packed_hash = vec![];
        File::open(
            MINESAVE_HOME
                .join("versions")
                .join(&self.id)
                .join(self.time.as_millis().to_string()),
        )
        .await?
        .read_to_end(&mut packed_hash)
        .await?;
        let hash: HashMap<PathBuf, Sha256Sum> = rmp_serde::from_slice(&packed_hash)?;
        for (relative, hash) in hash {
            fs::create_dir_all(target.as_ref().join(&relative).parent().unwrap()).await?;
            let mut decoder = ZstdDecoder::new(BufReader::new(
                File::open(
                    MINESAVE_HOME
                        .join("storage")
                        .join(hash.to_string())
                        .with_extension("zst"),
                )
                .await?,
            ));
            let mut target_file = File::create(target.as_ref().join(&relative)).await?;
            io::copy(&mut decoder, &mut target_file).await?;
            target_file.flush().await?;
            target_file.shutdown().await?;
            debug!("recover: copied {:?}", relative);
        }
        Ok(())
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinecraftSaveVersionType {
    Default = 0,
    IncreasementData = 1,
    Snapshot = 2,
    FollowSettings = 255,
}
