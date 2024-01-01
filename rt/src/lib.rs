use std::rc::Weak;

use cli::Command;
use configuration::app_config::AppConfig;
use parking_lot::RwLock;
use tokio::{fs::File, io::AsyncReadExt, task::JoinSet};
use tracing::{debug, error, info, instrument, Instrument};
use utils::{core_types::CoreResult, logging::LoggingManager};

#[derive(Debug)]
pub struct AppState {
    pub log_manager: Weak<LoggingManager>,
    pub app_config: RwLock<AppConfig>,
}

impl AppState {
    pub fn new(log_manager: Weak<LoggingManager>, app_config: AppConfig) -> Self {
        Self {
            log_manager,
            app_config: RwLock::new(app_config),
        }
    }

    #[instrument(skip(self), fields(command))]
    pub async fn enter(&self, command: Command) -> CoreResult<()> {
        info!("Executing command \"{command}\".");
        match command {
            Command::TasksDemo { num_tasks } => {
                self.test_tasks(num_tasks).await?;
            }
            Command::FileError => {
                self.test_errors().await?;
            }
        }

        Ok(())
    }

    #[instrument]
    pub async fn test_errors(&self) -> CoreResult<()> {
        debug!("Opening file.");

        let mut found_file = File::open("non-existent-file").await?;
        let mut buffer = vec![];

        debug!("Reading file contents.");

        let byte_count = found_file.read_to_end(&mut buffer).await?;

        info!("Read {byte_count} bytes.");

        Ok(())
    }

    #[instrument(skip(self), fields(task_count))]
    pub async fn test_tasks(&self, task_count: usize) -> CoreResult<()> {
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
