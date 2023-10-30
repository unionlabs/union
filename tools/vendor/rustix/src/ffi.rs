//! Utilities related to FFI bindings.

// If we have std, use it.
#[cfg(feature = "std")]
pub use {
    std::ffi::{CStr, CString, FromBytesWithNulError, NulError},
    std::os::raw::c_char,
};

// If we don't have std, we can depend on core and alloc having these features
// in Rust 1.64+.
#[cfg(not(feature = "std"))]
pub use {
    alloc::ffi::{CString, NulError},
    core::ffi::{c_char, CStr, FromBytesWithNulError},
};
