use std::rc::{Rc, Weak};

use cli::cli_match;
use configuration::{app_config::AppConfig, AppConfigManager};
use rt::AppRuntime;
use tracing::{debug, info, instrument, Level};
use utils::{core_types::CoreResult, logging::LoggingManager, panic::initialize_panic_handler};

async fn initialize_log_manager() -> LoggingManager {
    let log_manager = LoggingManager::new();

    #[cfg(feature = "journald")]
    let log_manager = log_manager.with_journald_logging(Level::TRACE);

    #[cfg(feature = "logfile")]
    let log_manager = log_manager.with_logfile_logging(Level::TRACE);

    log_manager.with_fmt_logging(Level::INFO)
}

async fn refresh_logging_with_config(
    log_manager: LoggingManager, config: &AppConfig,
) -> CoreResult<LoggingManager> {
    let log_manager = log_manager.with_fmt_logging(config.logging.cli_log_level.parse::<Level>()?);

    #[cfg(feature = "journald")]
    let log_manager =
        log_manager.with_journald_logging(config.logging.journald_log_level.parse::<Level>()?);

    #[cfg(feature = "logfile")]
    let log_manager = log_manager
        .with_logfile_logging(config.logging.rolling_log_level.parse::<Level>()?)
        .with_logfile_prefix(config.logging.rolling_log_prefix.clone())
        .with_logfile_base_path(config.logging.rolling_log_path.clone());

    log_manager.refresh()?;

    Ok(log_manager)
}

#[instrument(skip(log_manager, app_config))]
async fn entrypoint(log_manager: Weak<LoggingManager>, app_config: AppConfig) -> CoreResult<()> {
    let command = cli_match()?;

    //TODO: Fix log_manager not logging all logs before program closing. Maybe only pass in a reference to the log_manager instead.
    let app_state: AppRuntime = AppRuntime::new(log_manager, app_config);

    app_state.enter(command).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> CoreResult<()> {
    initialize_panic_handler()?;

    let mut log_manager = initialize_log_manager().await;

    log_manager.build()?;

    debug!("Application started");

    #[cfg(feature = "journald")]
    tracing::trace!(
        "Journald logging enabled with syslog identifier \"{}\"",
        log_manager.get_syslog_identifier()
    );

    let app_config: AppConfig = AppConfigManager::clone_to_app_config().unwrap();

    debug!("Configuration loaded.");

    let log_manager = refresh_logging_with_config(log_manager, &app_config).await?;

    // Initialise a shareable pointer to the log manager to maintain top-level ownership.
    // This prevents the log manager being dropped before the program has finished all logging.
    let log_manager_pointer = Rc::new(log_manager);

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Interrupted. Shutting down.");
            Ok(())
        },
        res = entrypoint(Rc::downgrade(&log_manager_pointer), app_config) => {
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
