use color_eyre::Result;
use thiserror::Error;
use tracing_error::SpanTrace;

pub type CoreResult<T> = Result<T>;

pub type CoreErrorSource = Box<dyn std::error::Error + Send + Sync>;

#[derive(Error, Debug)]
pub struct CoreError {
    pub msg: String,
    pub context: SpanTrace,
    #[cfg(feature = "nightly")]
    pub backtrace: std::backtrace::Backtrace,
    pub source: Option<CoreErrorSource>,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)?;

        self.context.fmt(f)?;

        Ok(())
    }
}

impl Default for CoreError {
    fn default() -> Self {
        CoreError {
            msg: "".to_string(),
            context: SpanTrace::capture(),
            #[cfg(feature = "nightly")]
            backtrace: std::backtrace::Backtrace::capture(),
            source: None,
        }
    }
}

impl CoreError {
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
            ..Default::default()
        }
    }

    pub fn with_source(msg: &str, source: CoreErrorSource) -> Self {
        Self {
            msg: msg.to_string(),
            source: Some(source),
            ..Default::default()
        }
    }
}

impl From<tracing_subscriber::reload::Error> for CoreError {
    fn from(_err: tracing_subscriber::reload::Error) -> Self {
        Self::with_source("Log configuration reload failed.", Box::new(_err))
    }
}

impl From<config::ConfigError> for CoreError {
    fn from(_err: config::ConfigError) -> Self {
        Self::with_source("Configuration error", Box::new(_err))
    }
}

impl From<std::io::Error> for CoreError {
    fn from(_err: std::io::Error) -> Self {
        Self::with_source("IO Error", Box::new(_err))
    }
}
