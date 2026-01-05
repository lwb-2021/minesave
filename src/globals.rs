#[cfg(target_os = "linux")]
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

pub static MINESAVE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(data) = dirs::data_dir() {
        return data.join("minesave");
    }
    panic!("Could not determine the documents directory");
});
pub static CONFIG_FILE: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(config) = dirs::config_dir() {
        return config.join("minesave").join("config.json");
    }
    if let Some(home) = dirs::config_dir() {
        return home.join(".minesave").join("config.json");
    }
    panic!("Could not find path to store config file")
});

pub static MACHINE: LazyLock<String> = LazyLock::new(|| {
    #[cfg(target_os = "linux")]
    fs::read_to_string("/etc/machine-id")
        .expect("Unable to read /etc/machine-id")
        .replace("\n", "")
});
