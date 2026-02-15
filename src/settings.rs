use crate::utils::report_err;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::PathBuf,
    sync::{LazyLock, Mutex, MutexGuard},
};

static CONFIG_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::preference_dir().map_or_else(
        || {
            dirs::home_dir()
                .expect("Cannot locate config home")
                .join(".minesave")
                .join("config")
        },
        |x| x.join("minesave"),
    )
});

#[inline]
const fn default_compression_level() -> i32 {
    6
}

#[inline]
const fn daemon_backup_duration() -> u32 {
    3600
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]

pub struct Settings {
    pub language: String,
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,
    #[serde(default = "daemon_backup_duration")]
    pub daemon_backup_duration: u32,
    pub scan_root: Vec<PathBuf>,
    pub sync: bool,
    pub remote: Option<String>,
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
                    Settings {
                        compression_level: 6,
                        ..Default::default()
                    }
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
