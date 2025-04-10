use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::Path;

/// Logger wrapper with log methods
pub struct Logger;

impl Logger {
    pub fn init_from_file<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
        log4rs::init_file(path, Default::default())?;
        Ok(())
    }

    pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} [{l}] {m}{n}")))
            .build();

        let file = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} [{l}] {m}{n}")))
            .build("logs/oobe.log")?;

        let config = Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .appender(Appender::builder().build("file", Box::new(file)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("file")
                    .build(LevelFilter::Info),
            )?;

        log4rs::init_config(config)?;
        Ok(())
    }

    pub fn trace(message: &str) {
        log::trace!("{}", message);
    }

    pub fn debug(message: &str) {
        log::debug!("{}", message);
    }

    pub fn info(message: &str) {
        log::info!("{}", message);
    }

    pub fn warn(message: &str) {
        log::warn!("{}", message);
    }

    pub fn error(message: &str) {
        log::error!("{}", message);
    }
}

/// Initializes logger from config file or falls back to default
pub fn init_logger() {
    if let Err(_) = Logger::init_from_file("config/logger.yaml") {
        Logger::init_default().expect("Logger fallback failed");
    }
}

pub fn init_default_logger() {
    Logger::init_default().expect("Logger fallback failed");
}
