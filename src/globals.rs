use std::path::PathBuf;
use std::sync::LazyLock;

pub static MINESAVE_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    if let Some(document) = dirs::document_dir() {
        return document.join("MineSave");
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
