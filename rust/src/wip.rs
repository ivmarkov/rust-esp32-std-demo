// TODO: Figure out which library references this
#[no_mangle]
pub extern fn timegm(_: libc::tm) -> libc::time_t {
    // Not supported but don't crash just in case
    0
}

// Called by the rand crate
#[no_mangle]
pub extern "C" fn pthread_atfork(
    _: *const libc::c_void,
    _: *const libc::c_void,
    _: *const libc::c_void) -> libc::c_int {
    0
}
