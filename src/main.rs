mod api;
mod backup;
mod cmd;
mod error;
mod globals;
mod slint_api;

use crate::{backup::MinecraftSaveCollection, error::Result};
use clap::Parser;
use env_logger::Env;
use log::warn;
use std::fs;
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::new().filter_or(
        "RUST_LOG",
        if cfg!(debug_assertions) {
            "minesave=debug"
        } else {
            "minesave=info"
        },
    );
    env_logger::init_from_env(env);
    let parameters = cmd::Cli::parse();
    create_dirs()?;
    {
        let save_collection = MinecraftSaveCollection::global();
        let mut save_collection = save_collection.lock().unwrap();
        if let Err(err) = save_collection.load() {
            warn!(
                "Failed to load save information, using default one:\n{}",
                err
            );
            *save_collection = MinecraftSaveCollection::default();
        };
    }
    match parameters.command {
        Some(cmd::Command::UI) | None => {
            let handle = start_daemon_server().await?;
            slint_api::run_ui().unwrap();
            handle.abort();
        }
        Some(cmd::Command::Daemon) => {
            daemon().await?;
        }
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

async fn start_daemon_server() -> Result<JoinHandle<Result<()>>> {
    Ok(tokio::spawn(daemon()))
}
async fn daemon() -> Result<()> {
    let server = TcpListener::bind("127.0.0.1:7908").await?;

    log::debug!("bind: 127.0.0.1:7908");

    log::info!("daemon started");
    loop {
        if let Ok((mut conn, addr)) = server.accept().await {
            match daemon_inner(&mut conn).await {
                Ok(()) => log::debug!("request handled: addr={:?}", addr),
                Err(err) => log::error!("Failed to handle request: {:?}", err),
            };
        } else {
            log::error!("Failed to accept connection");
        }
    }
}
#[inline]
async fn daemon_inner(_conn: &mut TcpStream) -> Result<()> {
    Ok(())
}
