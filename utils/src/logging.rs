use std::error::Error;

use tracing::Level;
use tracing_subscriber::{
    filter,
    fmt::{self},
    prelude::*,
    reload, Registry,
};

#[derive(Debug)]
pub struct LogSubscriberBuilder {
    fmt_log_level: Option<Level>,
    fmt_layer_reload_handle: Option<reload::Handle<filter::LevelFilter, Registry>>,
}

impl Default for LogSubscriberBuilder {
    fn default() -> Self {
        Self {
            fmt_log_level: Some(Level::INFO),
            fmt_layer_reload_handle: None,
        }
    }
}

impl LogSubscriberBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fmt_logging(mut self, level: Level) -> Self {
        self.fmt_log_level = Some(level);

        self
    }

    pub fn with_fmt_logging_str(self, level_str: &str) -> Self {
        match Self::level_from_str(level_str) {
            Some(level) => self.with_fmt_logging(level),
            None => self,
        }
    }

    fn level_from_str(level_str: &str) -> Option<Level> {
        match level_str.to_uppercase().as_str() {
            "INFO" => Some(Level::INFO),
            "DEBUG" => Some(Level::DEBUG),
            "WARN" => Some(Level::WARN),
            "ERROR" => Some(Level::ERROR),
            "TRACE" => Some(Level::TRACE),
            _ => None,
        }
    }

    /// Set the global logging filters.
    /// Can only be called once.
    pub fn build(&mut self) {
        let registry = tracing_subscriber::registry();

        if let Some(log_level) = self.fmt_log_level {
            let (filter, reload_handle) =
                reload::Layer::new(filter::LevelFilter::from_level(log_level));
            self.fmt_layer_reload_handle = Some(reload_handle);

            let registry = registry.with(filter).with(fmt::Layer::default());

            registry.init();
        } else {
            panic!("Cannot initialise logging without fmt_params.")
        }
    }

    /// Refresh the global subscribers with any updated filters.
    pub fn refresh(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(reload_handle) = &self.fmt_layer_reload_handle {
            reload_handle.modify(|filter| {
                *filter = filter::LevelFilter::from_level(self.fmt_log_level.unwrap())
            })?;
        }

        Ok(())
    }
}
