use std::path::PathBuf;

use app_config::AppConfig;
use config::{Config, ConfigBuilder};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use utils::core_types::{CoreError, CoreResult};

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
            *builder = builder
                .clone()
                .set_override(key, value)
                .map_err(CoreError::from)?;
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
        let c = CONFIG_BUILDER
            .read()
            .build_cloned()
            .map_err(CoreError::from)?;

        let app_config: AppConfig = c.into();

        Ok(app_config)
    }

    pub fn add_file_source(file: PathBuf) {
        let mut builder = CONFIG_BUILDER.write();
        *builder = builder
            .clone()
            .add_source(config::File::with_name(file.to_str().unwrap()));
    }
}

#[cfg(test)]
mod app_config_manager_tests {
    use std::path::PathBuf;

    #[test]
    fn add_file_source_works() {
        use crate::AppConfigManager;
        use crate::CONFIG_BUILDER;

        let source = PathBuf::from("../configs/test_config.toml");

        AppConfigManager::add_file_source(source);

        let found_name: String = CONFIG_BUILDER
            .read()
            .build_cloned()
            .unwrap()
            .get_string("program.name")
            .unwrap();

        assert_eq!("test-source".to_string(), found_name);
    }

    #[test]
    fn get_works() {
        use crate::AppConfigManager;
        use crate::CONFIG_BUILDER;

        {
            let mut builder = CONFIG_BUILDER.write();
            *builder = builder
                .clone()
                .set_override("test-get-method", "Foo")
                .unwrap();
        }

        let found_name: String = AppConfigManager::get("test-get-method").unwrap();

        assert_eq!("Foo".to_string(), found_name);
    }

    #[test]
    fn set_works() {
        use crate::AppConfigManager;
        use crate::CONFIG_BUILDER;

        AppConfigManager::set("test-set-method", "Foo").unwrap();

        let found_name = CONFIG_BUILDER
            .read()
            .build_cloned()
            .unwrap()
            .get_string("test-set-method")
            .unwrap();

        assert_eq!("Foo".to_string(), found_name);
    }
}
