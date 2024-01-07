use color_eyre::Report;
use thiserror::Error;

pub type CoreResult<T> = Result<T, Report>;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("General Task Error: {0}")]
    GeneralTaskError(String),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "logfile")]
    #[error("Log file initialisation Error")]
    LogFileInitialisationError(#[from] tracing_appender::rolling::InitError),
    #[error("Logger reload Error")]
    LoggerReloadError(#[from] tracing_subscriber::reload::Error),
    #[error("Error handler initialisation Error")]
    ErrorHandlerInitialisationError(#[from] color_eyre::eyre::InstallError),
    #[error("App configuration Error")]
    AppConfigError(#[from] config::ConfigError),
    #[error("Tokio Error: {0}")]
    GeneralTokioError(String),
}

impl CoreError {
    pub fn to_report(self) -> Report {
        Report::new(self)
    }
}
