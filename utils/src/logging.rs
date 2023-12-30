use std::collections::HashMap;

use tracing::{level_filters::LevelFilter, Level};
use tracing_error::ErrorLayer;
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
    pub params: HashMap<String, String>,
    pub reload_handle: Option<ReloadHandle>,
}

impl std::fmt::Debug for LogLayerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogLayerConfig")
            .field("log_level", &self.log_level)
            .field("params", &self.params)
            .finish_non_exhaustive()
    }
}

impl LogLayerConfig {
    fn new(level: Level) -> Self {
        Self {
            log_level: level,
            params: HashMap::new(),
            reload_handle: None,
        }
    }
}

#[cfg(feature = "logfile")]
#[derive(Debug)]
pub struct LogFileLogLayerConfig {
    pub layer_config: LogLayerConfig,
    pub base_dir: String,
    pub prefix: String,
    pub _guard: Option<tracing_appender::non_blocking::WorkerGuard>,
}

#[cfg(feature = "logfile")]
impl LogFileLogLayerConfig {
    fn new(level: Level) -> Self {
        Self {
            layer_config: LogLayerConfig::new(level),
            base_dir: "logs/".to_string(),
            prefix: "rust-starter-template".to_string(),
            _guard: None,
        }
    }
}

#[derive(Default, Debug)]
pub struct LogSubscriberBuilder {
    fmt: Option<LogLayerConfig>,
    #[cfg(feature = "journald")]
    journald: Option<LogLayerConfig>,
    #[cfg(feature = "logfile")]
    logfile: Option<LogFileLogLayerConfig>,
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

    #[cfg(feature = "logfile")]
    pub fn with_logfile_logging(mut self, level: Level) -> Self {
        if let Some(ref mut logfile) = &mut self.logfile {
            logfile.layer_config.log_level = level;
        } else {
            self.logfile = Some(LogFileLogLayerConfig::new(level));
        }

        self
    }

    #[cfg(feature = "logfile")]
    pub fn with_logfile_base_path(mut self, base_dir: String) -> Self {
        let logfile = self.logfile.as_mut().unwrap();
        logfile.base_dir = base_dir;

        self
    }

    #[cfg(feature = "logfile")]
    pub fn with_logfile_prefix(mut self, prefix: String) -> Self {
        let logfile = self.logfile.as_mut().unwrap();
        logfile.prefix = prefix;

        self
    }

    #[cfg(feature = "journald")]
    pub fn get_syslog_identifier(&self) -> String {
        if let Some(journald) = &self.journald {
            journald
                .params
                .get("syslog_identifier")
                .unwrap_or(&"".to_string())
                .clone()
        } else {
            "".to_string()
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
            let journald_layer = tracing_journald::layer()?;

            let syslog_identifier = journald_layer.syslog_identifier();

            journald_config.params.insert(
                "syslog_identifier".to_string(),
                syslog_identifier.to_string(),
            );

            let (layer, reload_handle): (ReloadLayer, ReloadHandle) = reload::Layer::new(
                journald_layer
                    .boxed()
                    .with_filter(filter::LevelFilter::from_level(journald_config.log_level)),
            );

            journald_config.reload_handle = Some(reload_handle);
            layers.push(layer);
        }

        #[cfg(feature = "logfile")]
        if let Some(ref mut logfile_config) = self.logfile {
            let file_appender = tracing_appender::rolling::RollingFileAppender::builder()
                .rotation(tracing_appender::rolling::Rotation::DAILY)
                .filename_prefix(&logfile_config.prefix)
                .filename_suffix("log")
                .max_log_files(5)
                .build(&logfile_config.base_dir)?;

            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let fmt_inner_layer = fmt::Layer::new()
                .with_ansi(false)
                .with_writer(non_blocking)
                .boxed()
                .with_filter(filter::LevelFilter::from_level(
                    logfile_config.layer_config.log_level,
                ));
            let (layer, reload_handle): (ReloadLayer, ReloadHandle) =
                reload::Layer::new(fmt_inner_layer);

            logfile_config._guard = Some(_guard);
            logfile_config.layer_config.reload_handle = Some(reload_handle);

            layers.push(layer);
        }

        let error_layer = ErrorLayer::default();

        registry.with(layers).with(error_layer).init();

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

        #[cfg(feature = "logfile")]
        if let Some(logfile_config) = &self.logfile {
            let reload_handle = logfile_config.layer_config.reload_handle.as_ref().unwrap();
            reload_handle.modify(|layer_box| {
                *layer_box.filter_mut() =
                    filter::LevelFilter::from_level(logfile_config.layer_config.log_level)
            })?;
        }

        Ok(())
    }
}
