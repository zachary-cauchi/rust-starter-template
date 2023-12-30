use app_config::AppConfig;
use config::Config;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use utils::core_types::CoreResult;

pub mod app_config;

lazy_static! {
    /// The Global default config. Any default start-time configuration should be set here.
    /// A default configuration is hard-coded at compile-time using `include_str!`, then environment overrides are loaded.
    static ref CONFIG: RwLock<Config> = RwLock::new(
        Config::builder()
            .add_source(config::File::from_str(
                include_str!("../../configs/default_config.toml"),
                config::FileFormat::Toml
            ))
            .add_source(config::Environment::with_prefix("RUST_STARTER_TEMPLATE"))
            .build()
            .unwrap()
    );
}

/// The main configuration manager for the application. All config changes should go through here.
pub struct AppConfigManager {}

impl AppConfigManager {
    pub fn set(key: &str, value: &str) -> CoreResult<()> {
        //TODO: Replace deprecated Config::set method.
        #[allow(deprecated)]
        CONFIG.write().set(key, value)?;

        Ok(())
    }

    pub fn get<'de, T>(key: &'de str) -> CoreResult<T>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(CONFIG.read().get::<T>(key)?)
    }

    pub fn clone_to_app_config() -> CoreResult<AppConfig> {
        let c = CONFIG.read().clone();

        let app_config: AppConfig = c.into();

        Ok(app_config)
    }
}
