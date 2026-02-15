use clap::{Parser, Subcommand};

#[derive(Debug, Parser, Clone)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    Daemon,
}
