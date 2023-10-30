// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(dead_code)]
use crate::Error;
use core::{
    cmp::min,
    mem::MaybeUninit,
    num::NonZeroU32,
    ptr::NonNull,
    sync::atomic::{fence, AtomicPtr, Ordering},
};
use libc::c_void;

cfg_if! {
    if #[cfg(any(target_os = "netbsd", target_os = "openbsd", target_os = "android"))] {
        use libc::__errno as errno_location;
    } else if #[cfg(any(target_os = "linux", target_os = "emscripten", target_os = "redox"))] {
        use libc::__errno_location as errno_location;
    } else if #[cfg(any(target_os = "solaris", target_os = "illumos"))] {
        use libc::___errno as errno_location;
    } else if #[cfg(any(target_os = "macos", target_os = "freebsd"))] {
        use libc::__error as errno_location;
    } else if #[cfg(target_os = "haiku")] {
        use libc::_errnop as errno_location;
    } else if #[cfg(target_os = "nto")] {
        use libc::__get_errno_ptr as errno_location;
    } else if #[cfg(any(all(target_os = "horizon", target_arch = "arm"), target_os = "vita"))] {
        extern "C" {
            // Not provided by libc: https://github.com/rust-lang/libc/issues/1995
            fn __errno() -> *mut libc::c_int;
        }
        use __errno as errno_location;
    } else if #[cfg(target_os = "aix")] {
        use libc::_Errno as errno_location;
    }
}

cfg_if! {
    if #[cfg(target_os = "vxworks")] {
        use libc::errnoGet as get_errno;
    } else if #[cfg(target_os = "dragonfly")] {
        // Until rust-lang/rust#29594 is stable, we cannot get the errno value
        // on DragonFlyBSD. So we just return an out-of-range errno.
        unsafe fn get_errno() -> libc::c_int { -1 }
    } else {
        unsafe fn get_errno() -> libc::c_int { *errno_location() }
    }
}

pub fn last_os_error() -> Error {
    let errno = unsafe { get_errno() };
    if errno > 0 {
        Error::from(NonZeroU32::new(errno as u32).unwrap())
    } else {
        Error::ERRNO_NOT_POSITIVE
    }
}

// Fill a buffer by repeatedly invoking a system call. The `sys_fill` function:
//   - should return -1 and set errno on failure
//   - should return the number of bytes written on success
pub fn sys_fill_exact(
    mut buf: &mut [MaybeUninit<u8>],
    sys_fill: impl Fn(&mut [MaybeUninit<u8>]) -> libc::ssize_t,
) -> Result<(), Error> {
    while !buf.is_empty() {
        let res = sys_fill(buf);
        if res < 0 {
            let err = last_os_error();
            // We should try again if the call was interrupted.
            if err.raw_os_error() != Some(libc::EINTR) {
                return Err(err);
            }
        } else {
            // We don't check for EOF (ret = 0) as the data we are reading
            // should be an infinite stream of random bytes.
            let len = min(res as usize, buf.len());
            buf = &mut buf[len..];
        }
    }
    Ok(())
}

// A "weak" binding to a C function that may or may not be present at runtime.
// Used for supporting newer OS features while still building on older systems.
// Based off of the DlsymWeak struct in libstd:
// https://github.com/rust-lang/rust/blob/1.61.0/library/std/src/sys/unix/weak.rs#L84
// except that the caller must manually cast self.ptr() to a function pointer.
pub struct Weak {
    name: &'static str,
    addr: AtomicPtr<c_void>,
}

impl Weak {
    // A non-null pointer value which indicates we are uninitialized. This
    // constant should ideally not be a valid address of a function pointer.
    // However, if by chance libc::dlsym does return UNINIT, there will not
    // be undefined behavior. libc::dlsym will just be called each time ptr()
    // is called. This would be inefficient, but correct.
    // TODO: Replace with core::ptr::invalid_mut(1) when that is stable.
    const UNINIT: *mut c_void = 1 as *mut c_void;

    // Construct a binding to a C function with a given name. This function is
    // unsafe because `name` _must_ be null terminated.
    pub const unsafe fn new(name: &'static str) -> Self {
        Self {
            name,
            addr: AtomicPtr::new(Self::UNINIT),
        }
    }

    // Return the address of a function if present at runtime. Otherwise,
    // return None. Multiple callers can call ptr() concurrently. It will
    // always return _some_ value returned by libc::dlsym. However, the
    // dlsym function may be called multiple times.
    pub fn ptr(&self) -> Option<NonNull<c_void>> {
        // Despite having only a single atomic variable (self.addr), we still
        // cannot always use Ordering::Relaxed, as we need to make sure a
        // successful call to dlsym() is "ordered before" any data read through
        // the returned pointer (which occurs when the function is called).
        // Our implementation mirrors that of the one in libstd, meaning that
        // the use of non-Relaxed operations is probably unnecessary.
        match self.addr.load(Ordering::Relaxed) {
            Self::UNINIT => {
                let symbol = self.name.as_ptr() as *const _;
                let addr = unsafe { libc::dlsym(libc::RTLD_DEFAULT, symbol) };
                // Synchronizes with the Acquire fence below
                self.addr.store(addr, Ordering::Release);
                NonNull::new(addr)
            }
            addr => {
                let func = NonNull::new(addr)?;
                fence(Ordering::Acquire);
                Some(func)
            }
        }
    }
}

// SAFETY: path must be null terminated, FD must be manually closed.
pub unsafe fn open_readonly(path: &str) -> Result<libc::c_int, Error> {
    debug_assert_eq!(path.as_bytes().last(), Some(&0));
    loop {
        let fd = libc::open(path.as_ptr() as *const _, libc::O_RDONLY | libc::O_CLOEXEC);
        if fd >= 0 {
            return Ok(fd);
        }
        let err = last_os_error();
        // We should try again if open() was interrupted.
        if err.raw_os_error() != Some(libc::EINTR) {
            return Err(err);
        }
    }
}
