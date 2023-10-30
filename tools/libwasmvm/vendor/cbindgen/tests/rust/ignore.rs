/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn root() {}

/// cbindgen:ignore
///
/// Something else.
#[no_mangle]
pub extern "C" fn another_root() {}

#[no_mangle]
pub extern "C" fn no_ignore_root() {}
