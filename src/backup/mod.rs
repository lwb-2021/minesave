mod data;
mod save;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

pub use crate::backup::save::MinecraftSave;
pub use crate::backup::save::MinecraftSaveVersion;
pub use crate::backup::save::MinecraftSaveVersionType;

use crate::{error::Result, globals::MINESAVE_HOME};
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MinecraftSaveCollection {
    pub scan_root: Vec<PathBuf>,
    pub saves: HashMap<String, MinecraftSave>,
}

impl MinecraftSaveCollection {
    pub fn global() -> Arc<Mutex<Self>> {
        static INSTANCE: LazyLock<Arc<Mutex<MinecraftSaveCollection>>> =
            LazyLock::new(|| Arc::new(Mutex::new(MinecraftSaveCollection::default())));
        INSTANCE.clone()
    }
    pub fn load(&mut self) -> Result<()> {
        *self = serde_json::from_reader(File::open(MINESAVE_HOME.join("saves.json"))?)?;
        Ok(())
    }
    pub fn refresh(&mut self) {
        for root in &self.scan_root {
            if !fs::exists(root).unwrap_or_default() {
                eprintln!("Warning: Cannot find or read {:?}", root);
                continue;
            }
            self.saves.extend(
                walkdir::WalkDir::new(root)
                    .into_iter()
                    .filter_map(|x| x.ok())
                    .filter(|x| x.path().is_dir() && x.path().join("level.dat").is_file())
                    .map(|x| x.path().to_path_buf())
                    .map(|path| {
                        (
                            hash_string(&path),
                            MinecraftSave::create(hash_string(&path), path),
                        )
                    }),
            )
        }
        if let Err(err) = self.save() {
            eprintln!("Warning: Failed to save saves.json: {}", err);
        }
    }
    pub fn save(&self) -> Result<()> {
        serde_json::to_writer_pretty(File::create(MINESAVE_HOME.join("saves.json"))?, &self)?;
        Ok(())
    }
}
fn hash_string(input: &PathBuf) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    hasher.finish().to_string()
}
