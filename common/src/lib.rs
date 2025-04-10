mod logger;
pub mod contract;
pub mod plugin_runner;

pub use contract::{DataPacket, PluginCallback, SafePacket};
pub use plugin_runner::{run_plugin, run_plugin_async, PluginMode};
pub use logger::{init_default_logger, init_logger, Logger};

pub fn initialize_logging() {
    Logger::init_from_file("config/logger.yaml").unwrap_or_else(|_| {
        Logger::init_default().expect("Failed to init fallback logger");
    });

    Logger::info("✅ Logger initialized");
}
