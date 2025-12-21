#![feature(cfg_select)]
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
use std::{fs, io::Bytes};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let env = Env::new().filter_or(
        "RUST_LOG",
        cfg_select! {
            debug_assertions => {"minesave=debug"}
            _ => {"minesave=info"}
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
        None | Some(cmd::Command::UI) => {
            slint_api::run_ui().unwrap();
            start_daemon_server().await?;
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

async fn start_daemon_server() -> Result<()> {
    tokio::spawn(daemon()).await?;
    log::info!("daemon started");
    Ok(())
}
async fn daemon() -> Result<()> {
    let server = TcpListener::bind("127.0.0.1:7908").await?;

    loop {
        if let Ok((mut conn, _addr)) = server.accept().await {
            daemon_inner(&mut conn).await;
        } else {
            log::error!("Failed to accept connection");
        }
    }
}
#[inline]
async fn daemon_inner(conn: &mut TcpStream) -> Result<()> {
    Ok(())
}
