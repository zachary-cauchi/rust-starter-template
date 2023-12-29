use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;
pub type CoreErrorSource = Box<dyn std::error::Error + Send + Sync>;

#[derive(Error, Debug)]
pub struct CoreError {
    pub msg: String,
    #[cfg(feature = "nightly")]
    backtrace: std::backtrace::Backtrace,
    source: Option<CoreErrorSource>,
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Default for CoreError {
    fn default() -> Self {
        CoreError {
            msg: "".to_string(),
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
