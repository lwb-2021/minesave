use crate::{settings::Settings, utils::report_err};
use anyhow::{Result, anyhow, bail};
use rustic_backend::BackendOptions;
use rustic_core::{
    BackupOptions, CommandInput, ConfigOptions, KeyOptions, PathList, Repository,
    RepositoryOptions, SnapshotOptions, repofile::SnapshotFile,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    sync::{LazyLock, Mutex, MutexGuard},
};

static MINESAVE_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            dirs::document_dir().map_or_else(
                || {
                    dirs::home_dir()
                        .expect("Cannot locate data home")
                        .join(".minesave")
                        .join("data")
                },
                |x| x.join("minesave"),
            )
        })
        .to_path_buf()
});

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppState {
    saves: HashMap<String, SaveBackupConfiguration>,
}

impl AppState {
    pub fn instance() -> MutexGuard<'static, AppState> {
        static INSTANCE: LazyLock<Mutex<AppState>> =
            LazyLock::new(|| Mutex::new(AppState::default()));
        INSTANCE.lock().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveBackupConfiguration {
    id: String,
    init: bool,
    source: PathBuf,
    #[serde(skip, default)]
    last_snapshot: Option<SnapshotFile>,
}
impl SaveBackupConfiguration {
    pub fn new(source: PathBuf) -> Self {
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        Self {
            id: format!("{:x}", hasher.finish()),
            init: false,
            source,
            last_snapshot: None,
        }
    }
    pub fn run_backup(&mut self, snapshot_options: SnapshotOptions) -> Result<()> {
        let mut backend_options = BackendOptions::default();
        backend_options.repo_hot = Some(
            MINESAVE_DATA_HOME
                .join(&self.id)
                .to_string_lossy()
                .to_string(),
        );
        backend_options.repository = Some(
            Settings::instance()
                .remote
                .clone()
                .unwrap_or(backend_options.repository.clone().unwrap()),
        );
        let backends = backend_options
            .to_backends()
            .inspect_err(report_err("Failed to init backend"))?;
        let mut repo_options = RepositoryOptions::default();
        if repo_options.password.is_none() && repo_options.password_command.is_none() {
            warn!("Neither password nor password command is configured");
            native_dialog::MessageDialogBuilder::default()
                .set_title(t!("set-password"))
                .set_text(t!("set-password"))
                .alert();
            bail!("no password");
        }
        repo_options.password = Settings::instance().password.clone();
        repo_options.password_command = Settings::instance().password_cmd.as_ref().map(|s| {
            CommandInput::from(
                s.clone()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            )
        });
        let key_options = KeyOptions::default();
        let config_options = ConfigOptions::default();
        let repo = Repository::new(&repo_options, &backends)
            .inspect_err(report_err("Failed to create backup storage instance"))?;
        let repo = if !self.init {
            info!("storage_init(id={})", self.id);
            repo.init(&key_options, &config_options)
                .inspect_err(report_err("Failed to init backup storage"))?
        } else {
            repo.open()
                .inspect_err(report_err("Failed to open backup storage"))?
        };
        let repo = repo
            .to_indexed_ids()
            .inspect_err(report_err("Failed to index repo"))?;
        let mut backup_options = BackupOptions::default();
        let source = PathList::from_string(
            self.source
                .to_str()
                .expect("Character in path is not UTF-8"),
        )
        .inspect_err(report_err("Failed to parse source path"))?;
        self.last_snapshot = Some(
            repo.backup(
                &backup_options,
                &source,
                snapshot_options
                    .to_snapshot()
                    .inspect_err(report_err("Bad snapshot options"))?,
            )
            .inspect_err(report_err("Failed to create backup"))?,
        );
        info!(
            "backup(id={}, snapshot_id={:x})",
            self.id,
            self.last_snapshot
                .as_ref()
                .expect("This should never happen")
                .uid
        );
        Ok(())
    }
}
