use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{
    filter::{self, Filtered},
    fmt::{self},
    prelude::*,
    reload::{self, Handle, Layer},
    Registry,
};

use crate::core_types::CoreResult;

type BaseLayer = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync>;
type ReloadLayer = Layer<Filtered<BaseLayer, LevelFilter, Registry>, Registry>;
type ReloadHandle = Handle<Filtered<BaseLayer, LevelFilter, Registry>, Registry>;

pub struct LogLayerConfig {
    pub log_level: Level,
    pub reload_handle: Option<ReloadHandle>,
}

impl Default for LogLayerConfig {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
            reload_handle: None,
        }
    }
}

impl LogLayerConfig {
    fn new(level: Level) -> Self {
        Self {
            log_level: level,
            reload_handle: None,
        }
    }
}

#[derive(Default)]
pub struct LogSubscriberBuilder {
    fmt: Option<LogLayerConfig>,
    #[cfg(feature = "journald")]
    journald: Option<LogLayerConfig>,
}

impl LogSubscriberBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fmt_logging(mut self, level: Level) -> Self {
        if let Some(ref mut fmt) = &mut self.fmt {
            fmt.log_level = level;
        } else {
            self.fmt = Some(LogLayerConfig::new(level));
        }

        self
    }

    #[cfg(feature = "journald")]
    pub fn with_journald_logging(mut self, level: Level) -> Self {
        if let Some(ref mut journald) = &mut self.journald {
            journald.log_level = level;
        } else {
            self.journald = Some(LogLayerConfig::new(level));
        }

        self
    }

    pub fn with_fmt_logging_str(self, level_str: &str) -> Self {
        match Self::level_from_str(level_str) {
            Some(level) => self.with_fmt_logging(level),
            None => self,
        }
    }

    #[cfg(feature = "journald")]
    pub fn with_journald_logging_str(self, level_str: &str) -> Self {
        match Self::level_from_str(level_str) {
            Some(level) => self.with_journald_logging(level),
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
    pub fn build(&mut self) -> CoreResult<()> {
        let registry = tracing_subscriber::registry();

        let mut layers = Vec::new();
        if let Some(ref mut fmt_config) = self.fmt {
            let (layer, reload_handle): (ReloadLayer, ReloadHandle) = reload::Layer::new(
                fmt::Layer::new()
                    .boxed()
                    .with_filter(filter::LevelFilter::from_level(fmt_config.log_level)),
            );
            fmt_config.reload_handle = Some(reload_handle);
            layers.push(layer);
        } else {
            panic!("Cannot initialise logging without fmt_params.");
        }

        #[cfg(feature = "journald")]
        if let Some(ref mut journald_config) = self.journald {
            let journald_layer: Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync> =
                tracing_journald::layer()?.boxed();

            let (layer, reload_handle): (ReloadLayer, ReloadHandle) = reload::Layer::new(
                journald_layer
                    .with_filter(filter::LevelFilter::from_level(journald_config.log_level)),
            );

            journald_config.reload_handle = Some(reload_handle);
            layers.push(layer);
        }

        registry.with(layers).init();

        Ok(())
    }

    /// Refresh the global subscribers with any updated filters.
    pub fn refresh(&mut self) -> CoreResult<()> {
        if let Some(fmt_config) = &self.fmt {
            let reload_handle = fmt_config.reload_handle.as_ref().unwrap();
            reload_handle.modify(|layer_box| {
                *layer_box.filter_mut() = filter::LevelFilter::from_level(fmt_config.log_level)
            })?;
        }

        #[cfg(feature = "journald")]
        if let Some(journald_config) = &self.journald {
            let reload_handle = journald_config.reload_handle.as_ref().unwrap();
            reload_handle.modify(|layer_box| {
                *layer_box.filter_mut() = filter::LevelFilter::from_level(journald_config.log_level)
            })?;
        }

        Ok(())
    }
}
