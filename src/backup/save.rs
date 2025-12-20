use crate::{Result, backup::hash::create_full_copy_with_hash, globals::MINESAVE_HOME};
use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinecraftSave {
    // pub path: PathBuf, // Where the backup is stored
    pub id: String,
    pub name: String,
    pub description: String,
    pub target: PathBuf,
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
            target: source.as_ref().to_path_buf(),
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
    ) -> Result<Self> {
        match version_type {
            MinecraftSaveVersionType::Full => {
                Self::create_version_full(source, id, description).await
            }
            MinecraftSaveVersionType::Default => {
                Self::create_version_full(source, id, description).await // TODO: Considering add a config for this
            }
            #[allow(unreachable_patterns)]
            _ => todo!(),
        }
    }

    async fn create_version_full<P: AsRef<Path>>(
        source: P,
        id: String,
        description: String,
    ) -> Result<Self> {
        let path = MINESAVE_HOME.join(id).join(
            std::time::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_millis()
                .to_string(),
        );
        fs::create_dir_all(&path).await?;
        let hash = create_full_copy_with_hash(&source, &path).await?;
        let packed_hash = rmp_serde::to_vec(&hash)?;
        let mut hash_file = File::create(&path.with_file_name("hash.dat")).await?;
        hash_file.write_all(&packed_hash).await?;
        // TODO: compress
        Ok(Self {
            path,
            description,
            prev: None,
            version_type: MinecraftSaveVersionType::Full,
        })
    }
    pub async fn merge(self, save_name: String) -> Result<Self> {
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
        )
        .await
    }
    pub async fn recover<P: AsRef<Path>>(&self, target: P) -> Result<Self> {
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
    async fn recover_self<P: AsRef<Path>>(&self, target: P) -> Result<Self> {
        bail!("Not implemented, {:?}", target.as_ref());
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinecraftSaveVersionType {
    Full = 0,
    Increasement = 1,
    Snapshot = 2,
    Default = 255,
}
