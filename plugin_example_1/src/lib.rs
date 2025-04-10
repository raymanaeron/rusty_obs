use common::{init_logger, Logger, DataPacket, PluginCallback};
use std::os::raw::c_void;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().unwrap())
}

#[no_mangle]
pub extern "C" fn init() {
    println!("🔧 Plugin initialized.");
    init_logger();
}

#[no_mangle]
pub extern "C" fn run(input: *const DataPacket) -> *mut DataPacket {
    Logger::info("In run");
    let Some(pkt) = DataPacket::from_raw(input) else {
        return std::ptr::null_mut();
    };

    println!("Received ID: {}, Data: {}", pkt.id, pkt.data);
    Logger::info("In run - returning");
    DataPacket::to_raw(&pkt.id, &pkt.data)
}

#[no_mangle]
pub extern "C" fn run_async(
    input: *const DataPacket,
    callback: PluginCallback,
    user_data: *mut c_void,
) {
    Logger::info("In run async");
    let Some(pkt) = DataPacket::from_raw(input) else {
        return;
    };

    let id = pkt.id;
    let data = pkt.data;

    // ✅ Cast raw pointer to usize (Send-safe)
    let user_data_as_usize = user_data as usize;

    get_runtime().spawn(async move {
        println!("⏳ Async task running...");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let result = format!("Processed: {}", data);
        let output = DataPacket::to_raw(&id, &result);

        // ✅ Recast back to original pointer type before calling
        let user_data_ptr = user_data_as_usize as *mut c_void;
        println!("⏳ Async task invoking callback...");
        Logger::info("In run async - invoking callback");
        callback(output, user_data_ptr);
    });
}

#[no_mangle]
pub extern "C" fn cleanup(ptr: *mut DataPacket) {
    unsafe {
        DataPacket::free(ptr);
    }
}