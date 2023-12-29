use config::Config;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Database {
    pub name: String,
    pub url: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Logging {
    pub cli_log_level: String,
    pub journald_log_level: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub logging: Logging,
    pub databases: Vec<Database>,
}

impl From<Config> for AppConfig {
    fn from(config: Config) -> Self {
        AppConfig {
            logging: config
                .get::<Logging>("logging")
                .expect("No valid logging configuration found."),
            databases: config
                .get::<Vec<Database>>("database")
                .expect("No valid database configuration found."),
        }
    }
}
