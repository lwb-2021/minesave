use crate::utils::report_err;
use rustic_backend::BackendOptions;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::{LazyLock, Mutex, MutexGuard, OnceLock},
};

static CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::config_dir()
        .unwrap_or_else(|| dirs::home_dir().expect("Cannot locate config home"))
        .join("minesave")
});

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub language: String,
    pub scan_root: Vec<PathBuf>,
    pub sync: bool,
    pub password: Option<String>,
    pub password_cmd: Option<String>,
}
impl Settings {
    pub fn instance() -> MutexGuard<'static, Self> {
        static INSTANCE: LazyLock<Mutex<Settings>> = LazyLock::new(|| {
            Mutex::new({
                if let Ok(()) = fs::create_dir_all(CONFIG_HOME.as_path())
                    .inspect_err(report_err("Failed to create config home"))
                    && let Ok(file) = File::open(CONFIG_HOME.join("config.json"))
                        .inspect_err(report_err("Failed to read config file"))
                    && let Ok(settings) = serde_json::from_reader(file)
                        .inspect_err(report_err("Failed to read config file"))
                {
                    settings
                } else {
                    warn!("Using default settings");
                    Settings::default()
                }
            })
        });
        return INSTANCE
            .lock()
            .expect("Failed to lock Settings (This shouldn't happen)");
    }
    pub fn save(&self) {
        if let Ok(file) = File::create(CONFIG_HOME.join("config.json"))
            .inspect_err(report_err("Failed to write config file"))
            && let Ok(()) = serde_json::to_writer_pretty(file, self)
                .inspect_err(report_err("Failed to write config file"))
        {
            info!("save_config(target='{:?}'/config.json')", CONFIG_HOME)
        }
    }
}
