#[macro_use]
extern crate rust_i18n;
#[macro_use]
extern crate log;

use std::{
    fs::OpenOptions,
    io::Sink,
    os::unix::thread,
    panic,
    path::PathBuf,
    sync::LazyLock,
    thread::{self, sleep},
    time::Duration,
};

use clap::Parser;
use env_logger::Target;
use rustic_core::SnapshotOptions;

use crate::{backup::AppState, settings::Settings, utils::report_err};

mod backup;
mod cli;
mod settings;
mod tasks;
mod ui;
mod utils;

i18n!();

pub static MINESAVE_DATA_HOME: LazyLock<PathBuf> = LazyLock::new(|| {
    dirs::data_local_dir()
        .map_or_else(
            || {
                dirs::document_dir().unwrap_or_else(|| {
                    dirs::home_dir()
                        .expect("Cannot locate data home")
                        .join(".minesave")
                        .join("data")
                })
            },
            |x| x.join("minesave"),
        )
        .to_path_buf()
});

fn main() {
    setup_logger();
    rust_i18n::set_locale(&sys_locale::get_locale().unwrap_or_else(|| String::from("en-US")));

    let res = panic::catch_unwind(|| {
        AppState::instance().reload();
        let cli = cli::Cli::parse();
        if let None = cli.command {
            let daemon = std::thread::spawn(daemon);
            ui::run_app();
            daemon.join().unwrap();
        } else {
            match cli.command.unwrap() {
                cli::Command::Daemon => daemon(),
            }
        }
    });

    AppState::instance().save().unwrap_or_default();
    tasks::wait_all();

    res.unwrap();
}

#[inline]
fn setup_logger() {
    env_logger::builder()
        .filter_level(if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
        .target(Target::Stdout)
        .target({
            if let Ok(log) = OpenOptions::new()
                .append(true)
                .create(true)
                .open(MINESAVE_DATA_HOME.join("log"))
                .inspect_err(report_err("Failed to open log file"))
            {
                Target::Pipe(Box::new(log))
            } else {
                Target::Pipe(Box::new(Sink::default()))
            }
        })
        .init();
}

fn daemon() {
    loop {
        tasks::spawn(
            t!("auto-backup").to_string(),
            tasks::TaskInfo::Backup {
                for_id: None,
                options: SnapshotOptions::default().label(t!("auto-backup").to_string()),
            },
        );
        tasks::wait_all();
        sleep(Duration::from_secs(
            Settings::instance().daemon_backup_duration as u64,
        ));
    }
}
