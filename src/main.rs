use configuration::{app_config::AppConfig, AppConfigManager};
use tokio::{fs::File, io::AsyncReadExt, task::JoinSet};
use tracing::{debug, error, info, instrument, span, Instrument, Level};
use utils::{
    core_types::CoreResult, logging::LogSubscriberBuilder, panic::initialize_panic_handler,
};

#[instrument]
async fn test_errors() -> CoreResult<()> {
    debug!("Opening file.");

    let mut found_file = File::open("non-existent-file").await?;
    let mut buffer = vec![];

    debug!("Reading file contents.");

    let byte_count = found_file.read_to_end(&mut buffer).await?;

    info!("Read {byte_count} bytes.");

    Ok(())
}

#[instrument]
async fn test_tasks() -> CoreResult<()> {
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

#[instrument]
async fn entrypoint() -> CoreResult<()> {
    test_errors().await?;

    test_tasks().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> CoreResult<()> {
    initialize_panic_handler()?;

    let mut log_manager = LogSubscriberBuilder::new().with_fmt_logging(Level::INFO);

    #[cfg(feature = "journald")]
    let mut log_manager = log_manager.with_journald_logging(Level::INFO);

    #[cfg(feature = "logfile")]
    let mut log_manager = log_manager.with_logfile_logging(Level::TRACE);

    log_manager.build()?;

    info!("Application started");

    #[cfg(feature = "journald")]
    info!(
        "Journald logging enabled with syslog identifier \"{}\"",
        log_manager.get_syslog_identifier()
    );

    let app_config: AppConfig = AppConfigManager::clone_to_app_config().unwrap();

    debug!("Configuration loaded.");

    let mut log_manager =
        log_manager.with_fmt_logging(app_config.logging.cli_log_level.parse::<Level>()?);

    #[cfg(feature = "journald")]
    let mut log_manager =
        log_manager.with_journald_logging(app_config.logging.journald_log_level.parse::<Level>()?);

    #[cfg(feature = "logfile")]
    let mut log_manager = log_manager
        .with_logfile_logging(app_config.logging.rolling_log_level.parse::<Level>()?)
        .with_logfile_prefix(app_config.logging.rolling_log_prefix)
        .with_logfile_base_path(app_config.logging.rolling_log_path);

    log_manager.refresh()?;

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Interrupted. Shutting down.");
            Ok(())
        },
        res = entrypoint() => {
            match res {
                Ok(_) => {
                    info!("Completed. Exiting.");
                    Ok(())
                },
                Err(e) => {
                    Err(e)
                }
            }
        }
    }
}
