use cli::cli_match;
use configuration::{app_config::AppConfig, AppConfigManager};
use rt::{test_errors, test_tasks};
use tracing::{debug, info, instrument, Level};
use utils::{core_types::CoreResult, logging::LoggingManager, panic::initialize_panic_handler};

#[instrument]
async fn entrypoint() -> CoreResult<()> {
    let command = cli_match()?;

    match command {
        cli::Command::TasksDemo { num_tasks } => {
            test_tasks(num_tasks).await?;
        }
        cli::Command::FileError => {
            test_errors().await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> CoreResult<()> {
    initialize_panic_handler()?;

    let log_manager = LoggingManager::new();

    #[cfg(feature = "journald")]
    let log_manager = log_manager.with_journald_logging(Level::TRACE);

    #[cfg(feature = "logfile")]
    let log_manager = log_manager.with_logfile_logging(Level::TRACE);

    let mut log_manager = log_manager.with_fmt_logging(Level::INFO);

    log_manager.build()?;

    debug!("Application started");

    #[cfg(feature = "journald")]
    tracing::trace!(
        "Journald logging enabled with syslog identifier \"{}\"",
        log_manager.get_syslog_identifier()
    );

    let app_config: AppConfig = AppConfigManager::clone_to_app_config().unwrap();

    debug!("Configuration loaded.");

    let log_manager =
        log_manager.with_fmt_logging(app_config.logging.cli_log_level.parse::<Level>()?);

    #[cfg(feature = "journald")]
    let log_manager =
        log_manager.with_journald_logging(app_config.logging.journald_log_level.parse::<Level>()?);

    #[cfg(feature = "logfile")]
    let log_manager = log_manager
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
