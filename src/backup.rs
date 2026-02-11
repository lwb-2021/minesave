use crate::{MINESAVE_DATA_HOME, settings::Settings, utils::report_err};
use anyhow::{Result, bail};
use rustic_backend::BackendOptions;
use rustic_core::{
    BackupOptions, CommandInput, ConfigOptions, KeyOptions, PathList, Repository,
    RepositoryOptions, SnapshotOptions, repofile::SnapshotFile,
};
use serde::{Deserialize, Serialize};
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex, MutexGuard},
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
    pub save_dirs: HashSet<PathBuf>,
    pub saves: HashMap<String, SaveBackupConfiguration>,
}

impl AppState {
    pub fn instance() -> MutexGuard<'static, AppState> {
        static INSTANCE: LazyLock<Mutex<AppState>> =
            LazyLock::new(|| Mutex::new(AppState::default()));
        INSTANCE.lock().unwrap()
    }

    pub fn reload(&mut self) {
        debug!("load_state");
        if let Ok(()) = fs::create_dir_all(MINESAVE_DATA_HOME.to_path_buf())
            .inspect_err(report_err("Failed to create data dir"))
            && let Ok(file) = fs::File::open(MINESAVE_DATA_HOME.join("state.json"))
                .inspect_err(report_err("Failed to open state file"))
        {
            *self = serde_json::from_reader(file)
                .inspect_err(report_err("Failed to read state file"))
                .unwrap_or_default();
        }

        debug!("rescan_saves");
        for item in Settings::instance().scan_root.iter() {
            let save_dirs: HashSet<PathBuf> = walkdir::WalkDir::new(item)
                .into_iter()
                .filter_map(|x| x.inspect_err(report_err("Error when visiting dir")).ok())
                .filter(|x| x.file_type().is_dir())
                .filter(|x| {
                    fs::exists(x.path().join("level.dat"))
                        .inspect_err(report_err("Error when visiting dir "))
                        .is_ok_and(|x| x)
                })
                .map(|x| x.into_path())
                .collect();
            let add: Vec<PathBuf> = save_dirs.difference(&self.save_dirs).cloned().collect();
            let delete: HashSet<PathBuf> = self.save_dirs.difference(&save_dirs).cloned().collect();
            let mut delete_keys = vec![]; // TODO: for GC
            for (k, v) in self.saves.iter() {
                if delete.contains(&v.source) {
                    delete_keys.push(k.clone());
                }
            }
            for item in add {
                let config = SaveBackupConfiguration::new(&item);
                self.saves.insert(config.id.clone(), config);
                self.save_dirs.insert(item);
            }
        }
        self.save().unwrap_or_default()
    }
    pub fn save(&self) -> Result<()> {
        debug!("save_state");
        serde_json::to_writer(
            File::create(MINESAVE_DATA_HOME.join("state.json"))
                .inspect_err(report_err("Failed to save state file"))?,
            self,
        )
        .inspect_err(report_err("Failed to save state file"))?;
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveBackupConfiguration {
    id: String,
    init: bool,
    source: PathBuf,
    #[serde(default)]
    last_snapshot: Option<SnapshotFile>,
}
impl SaveBackupConfiguration {
    pub fn new<P: AsRef<Path>>(source: P) -> Self {
        let mut hasher = DefaultHasher::new();
        source.as_ref().hash(&mut hasher);
        Self {
            id: format!("{:x}", hasher.finish()),
            init: false,
            source: source.as_ref().to_path_buf(),
            last_snapshot: None,
        }
    }
    pub fn run_backup(&mut self, snapshot_options: SnapshotOptions) -> Result<()> {
        debug!(
            "backup_start(id={}, options={:?})",
            self.id, snapshot_options
        );
        let settings = Settings::instance();
        debug!("settings_lock");
        let backends = BackendOptions::default()
            .repo_hot(
                MINESAVE_DATA_HOME
                    .join(&self.id)
                    .to_string_lossy()
                    .to_string(),
            )
            .repository(
                settings.remote.clone().unwrap_or(
                    MINESAVE_DATA_HOME
                        .join(&self.id)
                        .to_string_lossy()
                        .to_string(),
                ),
            )
            .to_backends()
            .inspect_err(report_err("Failed to init backend"))?;
        let mut repo_options = RepositoryOptions::default();
        if settings.password.is_none() && settings.password_cmd.is_none() {
            warn!("Neither password nor password command is configured");
            native_dialog::MessageDialogBuilder::default()
                .set_title(t!("set-password"))
                .set_text(t!("set-password"))
                .alert();
            bail!("no password");
        }
        repo_options.password = settings.password.clone();
        repo_options.password_command = settings.password_cmd.as_ref().map(|s| {
            CommandInput::from(
                s.clone()
                    .split(" ")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>(),
            )
        });
        let key_options = KeyOptions::default();
        let config_options = ConfigOptions::default().set_compression(min(
            rustic_core::max_compression_level(),
            settings.compression_level,
        ));
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
        let backup_options = BackupOptions::default();
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
        debug!(
            "backup_finish(id={}, snapshot_id={:x}, option={:?})",
            self.id,
            self.last_snapshot
                .as_ref()
                .expect("This should never happen")
                .uid,
            snapshot_options
        );
        Ok(()) // Settings released
    }
}
