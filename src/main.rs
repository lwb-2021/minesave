mod api;
mod backup;
mod cmd;
mod error;
mod globals;
mod slint_api;

use crate::backup::MinecraftSaveCollection;
use anyhow::Result;
use clap::Parser;
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let parameters = cmd::Cli::parse();
    create_dirs()?;

    {
        let save_collection = MinecraftSaveCollection::global();
        let mut save_collection = save_collection.lock().unwrap();
        if let Err(err) = save_collection.load() {
            eprintln!(
                "Warning: Failed to load save information, using default one:\n{}",
                err
            );
            *save_collection = MinecraftSaveCollection::default();
        };
    }
    start_daemon_server().await?;
    match parameters.command {
        None | Some(cmd::Command::UI) => {
            slint_api::run_ui().unwrap();
        }
        Some(cmd::Command::Daemon) => {}
    }
    Ok(())
}

fn create_dirs() -> Result<()> {
    if !globals::MINESAVE_HOME.exists() {
        fs::create_dir_all(globals::MINESAVE_HOME.as_path())?;
    }
    if !globals::CONFIG_FILE.parent().as_ref().unwrap().exists() {
        fs::create_dir_all(globals::CONFIG_FILE.parent().as_ref().unwrap())?;
    }
    Ok(())
}

async fn start_daemon_server() -> Result<()> {
    tokio::spawn(async {});
    Ok(())
}
