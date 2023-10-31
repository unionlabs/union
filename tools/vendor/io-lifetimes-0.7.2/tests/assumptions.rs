#![cfg_attr(target_os = "wasi", feature(wasi_ext))]

#[cfg(any(not(windows), feature = "close"))]
use std::mem::size_of;

#[cfg(unix)]
#[test]
fn test_assumptions() {
    assert_eq!(size_of::<std::os::unix::io::RawFd>(), size_of::<i32>());
    assert_eq!(
        size_of::<std::os::unix::io::RawFd>(),
        size_of::<std::os::raw::c_int>()
    );
}

#[cfg(target_os = "wasi")]
#[test]
fn test_assumptions() {
    assert_eq!(size_of::<std::os::wasi::io::RawFd>(), size_of::<i32>());
    assert_eq!(
        size_of::<std::os::wasi::io::RawFd>(),
        size_of::<std::os::raw::c_int>()
    );
}

#[cfg(all(windows, feature = "close"))]
#[test]
fn test_assumptions() {
    assert_eq!(
        size_of::<std::os::windows::io::RawSocket>(),
        size_of::<windows_sys::Win32::Networking::WinSock::SOCKET>()
    );
    assert_eq!(
        size_of::<std::os::windows::io::RawHandle>(),
        size_of::<windows_sys::Win32::Foundation::HANDLE>()
    );
    assert_eq!(
        windows_sys::Win32::Networking::WinSock::INVALID_SOCKET,
        usize::MAX
    );
    assert_ne!(
        windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE,
        std::ptr::null_mut() as std::os::windows::io::RawHandle as _
    );
}
