use std::path::PathBuf;

use app_config::AppConfig;
use config::{Config, ConfigBuilder};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use utils::core_types::CoreResult;

pub mod app_config;

lazy_static! {
    /// The Global default config builder. Any default start-time configuration should be set here.
    /// A default configuration is hard-coded at compile-time using `include_str!`, then environment overrides are loaded.
    static ref CONFIG_BUILDER: RwLock<ConfigBuilder<config::builder::DefaultState>> = RwLock::new(
        Config::builder()
        .add_source(config::File::from_str(
            include_str!("../../configs/default_config.toml"),
            config::FileFormat::Toml
        ))
        .add_source(config::Environment::with_prefix("RUST_STARTER_TEMPLATE"))
    );
}

/// The main configuration manager for the application. All config changes should go through here.
pub struct AppConfigManager {}

impl AppConfigManager {
    pub fn set(key: &str, value: &str) -> CoreResult<()> {
        {
            let mut builder = CONFIG_BUILDER.write();
            *builder = builder.clone().set_override(key, value)?;
        }

        Ok(())
    }

    pub fn get<'de, T>(key: &'de str) -> CoreResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(CONFIG_BUILDER.read().build_cloned()?.get::<T>(key)?)
    }

    pub fn clone_to_app_config() -> CoreResult<AppConfig> {
        let c = CONFIG_BUILDER.read().build_cloned()?;

        let app_config: AppConfig = c.into();

        Ok(app_config)
    }

    pub fn add_file_source(file: PathBuf) -> CoreResult<AppConfig> {
        {
            let mut builder = CONFIG_BUILDER.write();
            *builder = builder
                .clone()
                .add_source(config::File::with_name(file.to_str().unwrap()));
        }

        Self::clone_to_app_config()
    }
}
