use std::slice;
use std::ffi::CString;
use std::ptr;

// TODO: Figure out which library references this
#[no_mangle]
pub extern "C" fn getcwd(buf: *mut u8, size: libc::size_t) -> *mut u8 {
    let cwd = "/";

    unsafe {
        if size < cwd.len() + 1 {
            ptr::null_mut()
        } else {
            let cwd_c_bytes = CString::new(cwd).unwrap().as_bytes_with_nul();
        
            let buf_bytes = slice::from_raw_parts_mut(buf, size as usize);
            buf_bytes[..cwd_c_bytes.len()].copy_from_slice(cwd_c_bytes);

            buf
        }
    }
}

// TODO: Figure out which library references this
#[no_mangle]
pub extern "C" fn gai_strerror(ecode: libc::c_int) -> *const libc::c_char {
    "(detailed message not available)"
}

// TODO: Figure out which library references this
#[no_mangle]
pub extern fn timegm(_: libc::tm) -> libc::time_t {
    // Not supported but don't crash just in case
    0
}

// TODO: Figure out which library references this
#[no_mangle]
pub extern "C" fn pthread_atfork(
    _: *const libc::c_void,
    _: *const libc::c_void,
    _: *const libc::c_void) -> libc::c_int {
    panic!("Not supported")
}
