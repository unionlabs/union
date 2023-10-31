#![cfg_attr(not(rustc_attrs), allow(unused_imports))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

use std::mem::size_of;

#[cfg(any(unix, target_os = "wasi"))]
use io_lifetimes::{BorrowedFd, OwnedFd};
#[cfg(windows)]
use io_lifetimes::{BorrowedSocket, OwnedSocket};

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{FromRawSocket, IntoRawSocket, RawFd};
#[cfg(windows)]
use std::os::windows::io::{FromRawSocket, IntoRawSocket, RawSocket};

#[cfg(all(rustc_attrs, any(unix, target_os = "wasi")))]
#[test]
fn test_niche_optimizations() {
    assert_eq!(size_of::<Option<OwnedFd>>(), size_of::<RawFd>());
    assert_eq!(size_of::<Option<BorrowedFd<'static>>>(), size_of::<RawFd>());
    unsafe {
        assert_eq!(OwnedFd::from_raw_fd(RawFd::MIN).into_raw_fd(), RawFd::MIN);
        assert_eq!(OwnedFd::from_raw_fd(RawFd::MAX).into_raw_fd(), RawFd::MAX);
        assert_eq!(
            Some(OwnedFd::from_raw_fd(RawFd::MIN))
                .unwrap()
                .into_raw_fd(),
            RawFd::MIN
        );
        assert_eq!(
            Some(OwnedFd::from_raw_fd(RawFd::MAX))
                .unwrap()
                .into_raw_fd(),
            RawFd::MAX
        );
    }
}

#[cfg(all(rustc_attrs, windows))]
#[test]
fn test_niche_optimizations_socket() {
    assert_eq!(size_of::<Option<OwnedSocket>>(), size_of::<RawSocket>());
    assert_eq!(
        size_of::<Option<BorrowedSocket<'static>>>(),
        size_of::<RawSocket>(),
    );
    unsafe {
        #[cfg(target_pointer_width = "32")]
        let (min, max) = (i32::MIN as u32, i32::MAX as u32);
        #[cfg(target_pointer_width = "64")]
        let (min, max) = (i64::MIN as u64, i64::MAX as u64);

        assert_eq!(OwnedSocket::from_raw_socket(min).into_raw_socket(), min);
        assert_eq!(OwnedSocket::from_raw_socket(max).into_raw_socket(), max);
        assert_eq!(
            Some(OwnedSocket::from_raw_socket(min))
                .unwrap()
                .into_raw_socket(),
            min
        );
        assert_eq!(
            Some(OwnedSocket::from_raw_socket(max))
                .unwrap()
                .into_raw_socket(),
            max
        );
    }
}
