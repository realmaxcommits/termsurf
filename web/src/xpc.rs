//! Minimal XPC client for sending overlay coordinates to the TermSurf compositor.
//!
//! This is a stripped-down XPC client that only supports connecting to a Mach
//! service and sending dictionary messages. It does not need the full
//! termsurf-xpc crate (which lives in the ts4 workspace).

use std::ffi::{c_char, c_void, CString};

// --- FFI bindings ---

type XpcConnectionT = *mut c_void;
type XpcObjectT = *mut c_void;

extern "C" {
    fn xpc_connection_create_mach_service(
        name: *const c_char,
        targetq: *mut c_void,
        flags: u64,
    ) -> XpcConnectionT;
    fn xpc_connection_set_event_handler(conn: XpcConnectionT, handler: *mut c_void);
    fn xpc_connection_resume(conn: XpcConnectionT);
    fn xpc_connection_send_message(conn: XpcConnectionT, message: XpcObjectT);
    fn xpc_connection_cancel(conn: XpcConnectionT);
    fn xpc_release(object: XpcObjectT);
    fn xpc_dictionary_create(
        keys: *const *const c_char,
        values: *const XpcObjectT,
        count: usize,
    ) -> XpcObjectT;
    fn xpc_dictionary_set_string(dict: XpcObjectT, key: *const c_char, value: *const c_char);
    fn xpc_dictionary_set_uint64(dict: XpcObjectT, key: *const c_char, value: u64);
}

// --- Public API ---

/// A connection to the TermSurf compositor XPC Mach service.
pub struct CompositorConnection {
    raw: XpcConnectionT,
}

unsafe impl Send for CompositorConnection {}

impl CompositorConnection {
    /// Connect to `com.termsurf.compositor`.
    ///
    /// Returns `None` if the Mach service is not registered (not running inside TermSurf).
    pub fn connect() -> Option<Self> {
        let name = CString::new("com.termsurf.compositor").unwrap();
        let raw = unsafe {
            xpc_connection_create_mach_service(name.as_ptr(), std::ptr::null_mut(), 0)
        };
        if raw.is_null() {
            return None;
        }

        // Set a minimal event handler (required before resume).
        // We use block2 to create an Obj-C block from a Rust closure.
        let block = block2::RcBlock::new(|_event: XpcObjectT| {
            // We don't process replies for now.
        });
        unsafe {
            xpc_connection_set_event_handler(
                raw,
                &*block as *const _ as *mut c_void,
            );
        }

        unsafe { xpc_connection_resume(raw) };
        Some(Self { raw })
    }

    /// Send a `set_overlay` message to the compositor.
    pub fn send_set_overlay(&self, pane_id: &str, col: u16, row: u16, width: u16, height: u16) {
        let dict = unsafe {
            xpc_dictionary_create(std::ptr::null(), std::ptr::null(), 0)
        };
        if dict.is_null() {
            return;
        }

        let action = CString::new("set_overlay").unwrap();
        let action_key = CString::new("action").unwrap();
        let pane_id_c = CString::new(pane_id).unwrap();
        let pane_id_key = CString::new("pane_id").unwrap();
        let col_key = CString::new("col").unwrap();
        let row_key = CString::new("row").unwrap();
        let width_key = CString::new("width").unwrap();
        let height_key = CString::new("height").unwrap();

        unsafe {
            xpc_dictionary_set_string(dict, action_key.as_ptr(), action.as_ptr());
            xpc_dictionary_set_string(dict, pane_id_key.as_ptr(), pane_id_c.as_ptr());
            xpc_dictionary_set_uint64(dict, col_key.as_ptr(), col as u64);
            xpc_dictionary_set_uint64(dict, row_key.as_ptr(), row as u64);
            xpc_dictionary_set_uint64(dict, width_key.as_ptr(), width as u64);
            xpc_dictionary_set_uint64(dict, height_key.as_ptr(), height as u64);

            xpc_connection_send_message(self.raw, dict);
            xpc_release(dict);
        }
    }
}

impl Drop for CompositorConnection {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe {
                xpc_connection_cancel(self.raw);
                xpc_release(self.raw);
            }
        }
    }
}
