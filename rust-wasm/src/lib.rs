use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn hello_world() -> *mut c_char {
    let message = CString::new("hello, world").unwrap();
    message.into_raw()
}

/// Free a string that was allocated by Rust and passed to the host.
///
/// # Safety
///
/// This function is unsafe because it takes a raw pointer. The caller must ensure:
/// - The pointer was originally obtained from a CString created by this module
/// - The pointer has not been freed already
/// - The pointer is not being used elsewhere after this call
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    let _ = CString::from_raw(s);
}
