use tracing::Level;

#[derive(Debug)]
pub struct LogSubscriberBuilder {
    log_level: Level,
}

impl Default for LogSubscriberBuilder {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
        }
    }
}

impl LogSubscriberBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    pub fn build_global(self) {
        tracing_subscriber::fmt()
            .with_max_level(self.log_level)
            .init()
    }
}

pub fn init_log_subscriber() {}
