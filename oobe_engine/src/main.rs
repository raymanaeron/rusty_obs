use common::{init_logger, Logger, DataPacket, PluginMode, run_plugin};
use clap::Parser;

/// Command-line options
#[derive(Parser, Debug)]
#[command(name = "OOBE Engine")]
#[command(author = "DS2")]
#[command(version = "1.0")]
#[command(about = "Out-of-box experience engine", long_about = None)]
struct Args {
    /// Run the engine in UI mode
    #[arg(long)]
    ui: bool,
}

fn main() {
    init_logger();
    let args = Args::parse();

    Logger::info("🚀 OOBE Engine started");

    if args.ui {
        Logger::info("🖥️ Running in UI mode");
        // show_ui_window(); // ← Launch the UI window
    } else {
        Logger::info("🧱 Running in headless (console) mode");
    }

    let input_sync = DataPacket::to_raw("engine.sync", r#"{"task":"sync"}"#);
    let input_async = DataPacket::to_raw("engine.async", r#"{"task":"async"}"#);

    // Run sync
    if let Some(pkt) = run_plugin("plugin_example_1.dll", input_sync, PluginMode::Sync) {
        println!("✅ Sync Result: ID = {}, Data = {}", pkt.id, pkt.data);
        Logger::info("Ran Sync");
    }

    // Run async
    if let Some(pkt) = run_plugin("plugin_example_1.dll", input_async, PluginMode::Async) {
        println!("✅ Sync Result: ID = {}, Data = {}", pkt.id, pkt.data);
        Logger::info("Ran Async");
    }

    Logger::info("🛑 OOBE Engine exiting");
}
