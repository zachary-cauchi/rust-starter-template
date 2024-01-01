use std::{fmt::Display, path::PathBuf};

use clap::{Parser, Subcommand};
use configuration::AppConfigManager;
use utils::core_types::CoreResult;

#[derive(Parser, Debug)]
#[command(
    name = "rust-starter-template",
    author,
    about = "A sample repository to build upon existing datasets.",
    long_about = "Rust Starter Template",
    version
)]
pub struct Cli {
    /// Load a new config file.
    #[arg(short = 'c', long = "config", value_name = "FILE")]
    pub config_path: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[clap(name = "file-error", about = "Generate a file not found error.", long_about = None)]
    FileError,
    #[clap(name = "demo-tasks", about = "Test generating of tasks.", long_about = None)]
    TasksDemo {
        #[arg(value_name = "NUM_TASKS", default_value = "64")]
        num_tasks: usize,
    },
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileError => write!(f, "FileError"),
            Self::TasksDemo { num_tasks } => write!(f, "TasksDemo({num_tasks})"),
        }
    }
}

pub fn cli_match() -> CoreResult<Command> {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config_path {
        AppConfigManager::add_file_source(config_path)?;
    }

    Ok(cli.command)
}
