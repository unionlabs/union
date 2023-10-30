//! A simple testcase that prints a few messages to the console, demonstrating
//! the io-lifetimes API.

#![cfg_attr(not(rustc_attrs), allow(unused_imports))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

#[cfg(feature = "close")]
use io_lifetimes::example_ffi::*;
#[cfg(feature = "close")]
use std::{
    fs::File,
    io::{self, Write},
};

#[cfg(all(unix, feature = "close"))]
use io_lifetimes::{AsFd, FromFd, OwnedFd};

#[cfg(windows)]
use io_lifetimes::{AsHandle, FromHandle, OwnedHandle};
#[cfg(windows)]
use std::{convert::TryInto, os::windows::io::RawHandle, ptr::null_mut};

#[cfg(all(rustc_attrs, unix, feature = "close"))]
fn main() -> io::Result<()> {
    let fd = unsafe {
        // Open a file, which returns an `Option<OwnedFd>`, which we can
        // maybe convert into an `OwnedFile`.
        let fd: OwnedFd = open("/dev/stdout\0".as_ptr() as *const _, O_WRONLY | O_CLOEXEC)
            .ok_or_else(io::Error::last_os_error)?;

        // Borrow the fd to write to it.
        let result = write(fd.as_fd(), "hello, world\n".as_ptr() as *const _, 13);
        match result {
            -1 => return Err(io::Error::last_os_error()),
            13 => (),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "short write")),
        }

        fd
    };

    // Convert into a `File`. No `unsafe` here!
    let mut file = File::from_fd(fd);
    writeln!(&mut file, "greetings, y'all")?;

    // We can borrow a `BorrowedFd` from a `File`.
    unsafe {
        let result = write(file.as_fd(), "sup?\n".as_ptr() as *const _, 5);
        match result {
            -1 => return Err(io::Error::last_os_error()),
            5 => (),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "short write")),
        }
    }

    // `OwnedFd` closes the fd in its `Drop` implementation.

    Ok(())
}

/// The Windows analog of the above.
#[cfg(all(windows, feature = "close"))]
fn main() -> io::Result<()> {
    let handle = unsafe {
        // Open a file, which returns an `HandleOrInvalid`, which we can fallibly
        // convert into an `OwnedFile`.
        let handle: OwnedHandle = CreateFileW(
            ['C' as u16, 'O' as _, 'N' as _, 0].as_ptr(),
            FILE_GENERIC_WRITE,
            0,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut() as RawHandle as HANDLE,
        )
        .try_into()
        .map_err(|_err| io::Error::last_os_error())?;

        // Borrow the handle to write to it.
        let mut number_of_bytes_written = 0;
        let result = WriteFile(
            handle.as_handle(),
            "hello, world\n".as_ptr() as *const _,
            13,
            &mut number_of_bytes_written,
            null_mut(),
        );
        match (result, number_of_bytes_written) {
            (0, _) => return Err(io::Error::last_os_error()),
            (_, 13) => (),
            (_, _) => return Err(io::Error::new(io::ErrorKind::Other, "short write")),
        }

        handle
    };

    // Convert into a `File`. No `unsafe` here!
    let mut file = File::from_handle(handle);
    writeln!(&mut file, "greetings, y'all")?;

    // We can borrow a `BorrowedHandle` from a `File`.
    unsafe {
        let mut number_of_bytes_written = 0;
        let result = WriteFile(
            file.as_handle(),
            "sup?\n".as_ptr() as *const _,
            5,
            &mut number_of_bytes_written,
            null_mut(),
        );
        match (result, number_of_bytes_written) {
            (0, _) => return Err(io::Error::last_os_error()),
            (_, 5) => (),
            (_, _) => return Err(io::Error::new(io::ErrorKind::Other, "short write")),
        }
    }

    // `OwnedHandle` closes the handle in its `Drop` implementation.

    Ok(())
}

#[cfg(all(
    not(all(rustc_attrs, unix, feature = "close")),
    not(all(windows, feature = "close"))
))]
fn main() {
    println!("On Unix, this example requires Rust nightly (for `rustc_attrs`) and the \"close\" feature.");
}
