use std::path::PathBuf;

use rustic_backend::BackendOptions;
use rustic_core::RepositoryOptions;
use serde::{Deserialize, Serialize};

use crate::settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveBackupConfiguration {
    source: PathBuf,
}
impl SaveBackupConfiguration {
    pub fn new(source: PathBuf) -> Self {
        Self { source }
    }
}
