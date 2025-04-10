use crate::contract::{DataPacket, PluginCallback, SafePacket};
use crate::logger::Logger;
use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum PluginMode {
    Sync,
    Async,
}

struct PluginSymbols<'a> {
    init: Symbol<'a, unsafe extern "C" fn()>,
    run: Symbol<'a, unsafe extern "C" fn(*const DataPacket) -> *mut DataPacket>,
    run_async: Symbol<'a, unsafe extern "C" fn(*const DataPacket, PluginCallback, *mut c_void)>,
    cleanup: Symbol<'a, unsafe extern "C" fn(*mut DataPacket)>,
}

unsafe fn load_plugin_symbols<'a>(lib: &'a Library) -> PluginSymbols<'a> {
    PluginSymbols {
        init: lib.get(b"init").expect("❌ Failed to load init()"),
        run: lib.get(b"run").expect("❌ Failed to load run()"),
        run_async: lib.get(b"run_async").expect("❌ Failed to load run_async()"),
        cleanup: lib.get(b"cleanup").expect("❌ Failed to load cleanup()"),
    }
}

pub fn run_plugin(
    plugin_name: &str,
    input: *const DataPacket,
    mode: PluginMode,
) -> Option<Box<SafePacket>> {
    unsafe {
        Logger::info(&format!("[{} | {:?}] Loading plugin", plugin_name, mode));

        let lib = Library::new(plugin_name).expect("❌ Failed to load plugin library");
        let PluginSymbols { init, run, run_async, cleanup } = load_plugin_symbols(&lib);

        Logger::debug(&format!("[{} | {:?}] Plugin symbols loaded", plugin_name, mode));

        init();
        Logger::debug(&format!("[{} | {:?}] Plugin init() called", plugin_name, mode));

        match mode {
            PluginMode::Sync => {
                Logger::info(&format!("[{} | {:?}] Executing in Sync mode", plugin_name, mode));
                let output_ptr = run(input);
                let Some(pkt) = DataPacket::from_raw(output_ptr) else {
                    Logger::warn(&format!("[{} | {:?}] Plugin returned null pointer", plugin_name, mode));
                    return None;
                };
                cleanup(input as *mut _);
                cleanup(output_ptr);
                Logger::info(&format!("[{} | {:?}] Execution complete (Sync)", plugin_name, mode));
                Some(Box::new(pkt))
            }
            PluginMode::Async => {
                Logger::info(&format!("[{} | {:?}] Executing in Async mode", plugin_name, mode));
                let result_holder = Arc::new(Mutex::new(None));

                extern "C" fn handle_result(pkt: *mut DataPacket, ctx: *mut c_void) {
                    let arc = unsafe { Arc::from_raw(ctx as *const Mutex<Option<*mut DataPacket>>) };
                    {
                        let mut guard = arc.lock().unwrap();
                        *guard = Some(pkt);
                    }
                    let _ = Arc::into_raw(arc); // re-arm
                }

                let ctx_ptr = Arc::into_raw(result_holder.clone()) as *mut c_void;
                run_async(input, handle_result, ctx_ptr);
                cleanup(input as *mut _);

                for _ in 0..50 {
                    thread::sleep(Duration::from_millis(100));
                    let mut guard = result_holder.lock().unwrap();
                    if let Some(pkt_ptr) = guard.take() {
                        let Some(pkt) = DataPacket::from_raw(pkt_ptr) else {
                            Logger::warn(&format!("[{} | {:?}] Invalid data packet", plugin_name, mode));
                            return None;
                        };
                        cleanup(pkt_ptr as *mut DataPacket);
                        Logger::info(&format!("[{} | {:?}] Execution complete (Async)", plugin_name, mode));
                        return Some(Box::new(pkt));
                    }
                }

                Logger::warn(&format!("[{} | {:?}] Async response timed out", plugin_name, mode));
                None
            }
        }
    }
}

pub async fn run_plugin_async(
    plugin_name: &str,
    input: *const DataPacket,
    mode: PluginMode,
) -> Option<Box<SafePacket>> {
    unsafe {
        Logger::info(&format!("[{} | {:?}] [Async Entrypoint] Loading plugin", plugin_name, mode));

        let lib = Library::new(plugin_name).expect("❌ Failed to load plugin library");
        let PluginSymbols { init, run, run_async, cleanup } = load_plugin_symbols(&lib);

        Logger::debug(&format!("[{} | {:?}] Plugin symbols loaded", plugin_name, mode));

        init();
        Logger::debug(&format!("[{} | {:?}] Plugin init() called", plugin_name, mode));

        match mode {
            PluginMode::Sync => {
                Logger::info(&format!("[{} | {:?}] Executing Sync mode from async context", plugin_name, mode));
                let output_ptr = run(input);
                let Some(pkt) = DataPacket::from_raw(output_ptr) else {
                    Logger::warn(&format!("[{} | {:?}] Plugin returned null pointer", plugin_name, mode));
                    return None;
                };
                cleanup(input as *mut _);
                cleanup(output_ptr);
                Logger::info(&format!("[{} | {:?}] Execution complete (Sync)", plugin_name, mode));
                Some(Box::new(pkt))
            }
            PluginMode::Async => {
                Logger::info(&format!("[{} | {:?}] Executing Async plugin", plugin_name, mode));
                let (tx, rx) = oneshot::channel::<*mut DataPacket>();

                extern "C" fn handle_result(pkt: *mut DataPacket, ctx: *mut c_void) {
                    let sender = unsafe { Box::from_raw(ctx as *mut oneshot::Sender<*mut DataPacket>) };
                    let _ = sender.send(pkt);
                }

                let boxed_sender = Box::new(tx);
                let sender_ptr = Box::into_raw(boxed_sender) as *mut c_void;

                run_async(input, handle_result, sender_ptr);
                cleanup(input as *mut _);

                if let Ok(pkt_ptr) = rx.await {
                    let Some(pkt) = DataPacket::from_raw(pkt_ptr) else {
                        Logger::warn(&format!("[{} | {:?}] Invalid data packet", plugin_name, mode));
                        return None;
                    };
                    cleanup(pkt_ptr as *mut DataPacket);
                    Logger::info(&format!("[{} | {:?}] Execution complete (Async)", plugin_name, mode));
                    return Some(Box::new(pkt));
                }

                Logger::warn(&format!("[{} | {:?}] Async plugin timed out", plugin_name, mode));
                None
            }
        }
    }
}
