use configuration::{app_config::AppConfig, AppConfigManager};
use tokio::task::JoinSet;
use tracing::{debug, error, info, span, Instrument, Level};
use utils::{core_types::CoreResult, logging::LogSubscriberBuilder};

#[tokio::main]
async fn main() -> CoreResult<()> {
    let mut log_manager = LogSubscriberBuilder::new().with_fmt_logging(Level::INFO);

    #[cfg(feature = "journald")]
    let mut log_manager = log_manager.with_journald_logging(Level::INFO);

    log_manager.build()?;

    info!("Hello World");

    #[cfg(feature = "journald")]
    info!(
        "Journald logging enabled with syslog identifier \"{}\"",
        log_manager.get_syslog_identifier()
    );

    let app_config: AppConfig = AppConfigManager::clone_to_app_config().unwrap();

    let mut log_manager = log_manager.with_fmt_logging_str(&app_config.logging.cli_log_level);

    #[cfg(feature = "journald")]
    let mut log_manager =
        log_manager.with_journald_logging_str(&app_config.logging.journald_log_level);

    log_manager.refresh()?;

    const TOTAL_TASKS: usize = 14;

    let mut task_tracker: JoinSet<String> = JoinSet::new();

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
