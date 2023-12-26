use std::error::Error;

use configuration::{app_config::AppConfig, AppConfigManager};
use tokio::task::JoinSet;
use tracing::{debug, error, info, span, Instrument, Level};
use utils::logging::LogSubscriberBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut log_manager = LogSubscriberBuilder::new().with_fmt_logging(Level::INFO);

    log_manager.build();

    info!("Hello World");

    let app_config: AppConfig = AppConfigManager::clone_to_app_config().unwrap();

    log_manager
        .with_fmt_logging_str(&app_config.logging.cli_log_level)
        .refresh()?;

    const TOTAL_TASKS: usize = 14;

    let mut task_tracker = JoinSet::new();

    for i in 0..TOTAL_TASKS {
        task_tracker.spawn(
            async move {
                debug!("Entered task {i}");

                format!("Hello from task {i}")
            }
            .instrument(span!(Level::DEBUG, "Hello-Tasks")),
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
