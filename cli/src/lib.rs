use std::{fmt::Display, path::PathBuf};

use clap::{CommandFactory, Parser, Subcommand};
use configuration::AppConfigManager;
use utils::{core_types::CoreResult, project_name_str};

#[derive(Parser, Debug)]
#[command(
    name = project_name_str!(),
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
    pub command: AppCommand,
}

#[derive(Subcommand, Debug)]
pub enum AppCommand {
    #[clap(name = "file-error", about = "Generate a file not found error.", long_about = None)]
    FileError,
    #[clap(name = "demo-tasks", about = "Test generating of tasks.", long_about = None)]
    TasksDemo {
        #[arg(value_name = "NUM_TASKS", default_value = "64")]
        num_tasks: usize,
    },
    #[clap(name = "completion", about = "Generate shell completion scripts.", long_about = None)]
    Completion {
        #[clap(subcommand)]
        subcommand: CompletionSubCommand,
    },
}

#[derive(Subcommand, PartialEq, Debug)]
pub enum CompletionSubCommand {
    #[clap(about = "Generate the autocompletion script for Bash.")]
    Bash,
    #[clap(about = "Generate the autocompletion script for Zsh.")]
    Zsh,
    #[clap(about = "Generate the autocompletion script for Fish.")]
    Fish,
}

impl Display for AppCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileError => write!(f, "FileError"),
            Self::TasksDemo { num_tasks } => write!(f, "TasksDemo({num_tasks})"),
            Self::Completion { subcommand } => write!(
                f,
                "GenerateCompletions({})",
                match subcommand {
                    CompletionSubCommand::Bash => "Bash",
                    CompletionSubCommand::Zsh => "Zsh",
                    CompletionSubCommand::Fish => "Fish",
                }
            ),
        }
    }
}

pub fn cli_match() -> CoreResult<AppCommand> {
    let cli = Cli::parse();

    if let Some(config_path) = cli.config_path {
        AppConfigManager::add_file_source(config_path)?;
    }

    Ok(cli.command)
}

pub fn get_command() -> clap::Command {
    Cli::command()
}
