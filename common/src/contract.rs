use std::os::raw::{c_char, c_void};

#[repr(C)]
pub struct DataPacket {
    pub id: *const c_char,
    pub data: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct SafePacket {
    pub id: String,
    pub data: String,
}

impl DataPacket {
    /// Converts a raw pointer to a `SafePacket`, returning None if the pointer or its contents are invalid.
    pub fn from_raw(ptr: *const DataPacket) -> Option<SafePacket> {
        if ptr.is_null() {
            return None;
        }

        unsafe {
            let id_ptr = (*ptr).id;
            let data_ptr = (*ptr).data;

            if id_ptr.is_null() || data_ptr.is_null() {
                return None;
            }

            Some(SafePacket {
                id: std::ffi::CStr::from_ptr(id_ptr).to_string_lossy().into_owned(),
                data: std::ffi::CStr::from_ptr(data_ptr).to_string_lossy().into_owned(),
            })
        }
    }

    /// Converts a Rust `&str` id and data into a heap-allocated `DataPacket` pointer.
    pub fn to_raw(id: &str, data: &str) -> *mut DataPacket {
        let id_cstr = std::ffi::CString::new(id).unwrap();
        let data_cstr = std::ffi::CString::new(data).unwrap();

        let id_ptr = id_cstr.into_raw();
        let data_ptr = data_cstr.into_raw();

        Box::into_raw(Box::new(DataPacket { id: id_ptr, data: data_ptr }))
    }

    /// Frees a previously created `DataPacket` and its associated memory.
    pub unsafe fn free(ptr: *mut DataPacket) {
        if ptr.is_null() {
            return;
        }

        let pkt = Box::from_raw(ptr);

        if !pkt.id.is_null() {
            let _ = std::ffi::CString::from_raw(pkt.id as *mut c_char);
        }

        if !pkt.data.is_null() {
            let _ = std::ffi::CString::from_raw(pkt.data as *mut c_char);
        }
    }
}

/// Callback function type for asynchronous plugin completion.
pub type PluginCallback = extern "C" fn(*mut DataPacket, *mut c_void);
