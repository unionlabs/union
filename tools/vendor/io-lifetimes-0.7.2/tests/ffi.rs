#![cfg_attr(not(rustc_attrs), allow(unused_imports))]
#![cfg(feature = "close")]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

#[cfg(any(unix, windows))]
use io_lifetimes::example_ffi::*;
#[cfg(windows)]
use io_lifetimes::{InvalidHandleError, OwnedHandle};
#[cfg(windows)]
use std::{convert::TryInto, os::windows::io::RawHandle, ptr::null_mut};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, OPEN_EXISTING,
};

#[cfg(all(rustc_attrs, unix))]
#[test]
fn test_file_not_found() {
    assert!(unsafe {
        open(
            "/dev/no/such/file\0".as_ptr() as *const _,
            O_RDONLY | O_CLOEXEC,
        )
    }
    .is_none());
}

#[cfg(windows)]
#[test]
fn test_file_not_found() {
    let handle: Result<OwnedHandle, InvalidHandleError> = unsafe {
        CreateFileW(
            [
                'C' as u16, ':' as _, '/' as _, 'n' as _, 'o' as _, '/' as _, 's' as _, 'u' as _,
                'c' as _, 'h' as _, '/' as _, 'f' as _, 'i' as _, 'l' as _, 'e' as _, 0,
            ]
            .as_ptr(),
            FILE_GENERIC_READ,
            0,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut() as RawHandle as HANDLE,
        )
    }
    .try_into();
    assert!(handle.is_err());
    assert_eq!(
        std::io::Error::last_os_error().kind(),
        std::io::ErrorKind::NotFound
    );
}

#[cfg(all(rustc_attrs, unix))]
#[test]
fn test_file_found() {
    assert!(unsafe { open("Cargo.toml\0".as_ptr() as *const _, O_RDONLY | O_CLOEXEC) }.is_some());
}

#[cfg(windows)]
#[test]
fn test_file_found() {
    let handle: Result<OwnedHandle, InvalidHandleError> = unsafe {
        CreateFileW(
            [
                'C' as u16, 'a' as _, 'r' as _, 'g' as _, 'o' as _, '.' as _, 't' as _, 'o' as _,
                'm' as _, 'l' as _, 0,
            ]
            .as_ptr(),
            FILE_GENERIC_READ,
            0,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut() as RawHandle as HANDLE,
        )
    }
    .try_into();
    assert!(handle.is_ok());
}
