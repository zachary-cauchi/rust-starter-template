use std::rc::Weak;

use clap_complete::generate;
use cli::{get_command, AppCommand};
use configuration::app_config::AppConfig;
use parking_lot::RwLock;
use tokio::{fs::File, io::AsyncReadExt, task::JoinSet};
use tracing::{debug, error, info, instrument, Instrument};
use utils::{core_types::CoreResult, logging::LoggingManager, project_name_str};

#[derive(Debug)]
pub struct AppRuntime {
    pub log_manager: Weak<LoggingManager>,
    pub app_config: RwLock<AppConfig>,
}

impl AppRuntime {
    pub fn new(log_manager: Weak<LoggingManager>, app_config: AppConfig) -> Self {
        Self {
            log_manager,
            app_config: RwLock::new(app_config),
        }
    }

    #[instrument(skip(self), fields(command))]
    pub async fn enter(&self, command: AppCommand) -> CoreResult<()> {
        info!("Executing command \"{command}\".");
        match command {
            AppCommand::TasksDemo { num_tasks } => {
                self.test_tasks(num_tasks).await?;
            }
            AppCommand::FileError => {
                self.test_errors().await?;
            }
            AppCommand::Completion { subcommand } => {
                let mut app = get_command();

                match subcommand {
                    cli::CompletionSubCommand::Bash => generate(
                        clap_complete::shells::Bash,
                        &mut app,
                        project_name_str!(),
                        &mut std::io::stdout(),
                    ),
                    cli::CompletionSubCommand::Zsh => generate(
                        clap_complete::shells::Zsh,
                        &mut app,
                        project_name_str!(),
                        &mut std::io::stdout(),
                    ),
                    cli::CompletionSubCommand::Fish => generate(
                        clap_complete::shells::Fish,
                        &mut app,
                        project_name_str!(),
                        &mut std::io::stdout(),
                    ),
                }
            }
        }

        Ok(())
    }

    #[instrument]
    async fn test_errors(&self) -> CoreResult<()> {
        debug!("Opening file.");

        let mut found_file = File::open("non-existent-file").await?;
        let mut buffer = vec![];

        debug!("Reading file contents.");

        let byte_count = found_file.read_to_end(&mut buffer).await?;

        info!("Read {byte_count} bytes.");

        Ok(())
    }

    #[instrument(skip(self), fields(task_count))]
    async fn test_tasks(&self, task_count: usize) -> CoreResult<()> {
        let mut task_tracker: JoinSet<String> = JoinSet::new();

        for i in 0..task_count {
            task_tracker.spawn(
                async move {
                    debug!("Entered task {i}");

                    format!("Hello from task {i}")
                }
                .instrument(tracing::trace_span!("test_tasks_task")),
            );
        }

        while let Some(task_result) = task_tracker.join_next().await {
            match task_result {
                Ok(msg) => info!("Received msg \"{}\"", msg),
                Err(e) => error!("Failed to process task: {}", e),
            }
        }

        Ok(())
    }
}
