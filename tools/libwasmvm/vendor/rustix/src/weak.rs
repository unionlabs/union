// Implementation derived from `weak` in Rust's
// library/std/src/sys/unix/weak.rs at revision
// fd0cb0cdc21dd9c06025277d772108f8d42cb25f.
//
// Ideally we should update to a newer version which doesn't need `dlsym`,
// however that depends on the `extern_weak` feature which is currrently
// unstable.

#![cfg_attr(linux_raw, allow(unsafe_code))]

//! Support for "weak linkage" to symbols on Unix
//!
//! Some I/O operations we do in libstd require newer versions of OSes but we
//! need to maintain binary compatibility with older releases for now. In order
//! to use the new functionality when available we use this module for
//! detection.
//!
//! One option to use here is weak linkage, but that is unfortunately only
//! really workable on Linux. Hence, use dlsym to get the symbol value at
//! runtime. This is also done for compatibility with older versions of glibc,
//! and to avoid creating dependencies on `GLIBC_PRIVATE` symbols. It assumes
//! that we've been dynamically linked to the library the symbol comes from,
//! but that is currently always the case for things like libpthread/libc.
//!
//! A long time ago this used weak linkage for the `__pthread_get_minstack`
//! symbol, but that caused Debian to detect an unnecessarily strict versioned
//! dependency on libc6 (#23628).

// There are a variety of `#[cfg]`s controlling which targets are involved in
// each instance of `weak!` and `syscall!`. Rather than trying to unify all of
// that, we'll just allow that some unix targets don't use this module at all.
#![allow(dead_code, unused_macros)]
#![allow(clippy::doc_markdown)]

use crate::ffi::CStr;
use core::ffi::c_void;
use core::ptr::null_mut;
use core::sync::atomic::{self, AtomicPtr, Ordering};
use core::{marker, mem};

const NULL: *mut c_void = null_mut();
const INVALID: *mut c_void = 1 as *mut c_void;

macro_rules! weak {
    ($vis:vis fn $name:ident($($t:ty),*) -> $ret:ty) => (
        #[allow(non_upper_case_globals)]
        $vis static $name: $crate::weak::Weak<unsafe extern fn($($t),*) -> $ret> =
            $crate::weak::Weak::new(concat!(stringify!($name), '\0'));
    )
}

pub(crate) struct Weak<F> {
    name: &'static str,
    addr: AtomicPtr<c_void>,
    _marker: marker::PhantomData<F>,
}

impl<F> Weak<F> {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            name,
            addr: AtomicPtr::new(INVALID),
            _marker: marker::PhantomData,
        }
    }

    pub(crate) fn get(&self) -> Option<F> {
        assert_eq!(mem::size_of::<F>(), mem::size_of::<usize>());
        unsafe {
            // Relaxed is fine here because we fence before reading through the
            // pointer (see the comment below).
            match self.addr.load(Ordering::Relaxed) {
                INVALID => self.initialize(),
                NULL => None,
                addr => {
                    let func = mem::transmute_copy::<*mut c_void, F>(&addr);
                    // The caller is presumably going to read through this value
                    // (by calling the function we've dlsymed). This means we'd
                    // need to have loaded it with at least C11's consume
                    // ordering in order to be guaranteed that the data we read
                    // from the pointer isn't from before the pointer was
                    // stored. Rust has no equivalent to memory_order_consume,
                    // so we use an acquire fence (sorry, ARM).
                    //
                    // Now, in practice this likely isn't needed even on CPUs
                    // where relaxed and consume mean different things. The
                    // symbols we're loading are probably present (or not) at
                    // init, and even if they aren't the runtime dynamic loader
                    // is extremely likely have sufficient barriers internally
                    // (possibly implicitly, for example the ones provided by
                    // invoking `mprotect`).
                    //
                    // That said, none of that's *guaranteed*, and so we fence.
                    atomic::fence(Ordering::Acquire);
                    Some(func)
                }
            }
        }
    }

    // Cold because it should only happen during first-time initialization.
    #[cold]
    unsafe fn initialize(&self) -> Option<F> {
        let val = fetch(self.name);
        // This synchronizes with the acquire fence in `get`.
        self.addr.store(val, Ordering::Release);

        match val {
            NULL => None,
            addr => Some(mem::transmute_copy::<*mut c_void, F>(&addr)),
        }
    }
}

unsafe fn fetch(name: &str) -> *mut c_void {
    let name = match CStr::from_bytes_with_nul(name.as_bytes()) {
        Ok(c_str) => c_str,
        Err(..) => return null_mut(),
    };
    libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr())
}

#[cfg(not(linux_kernel))]
macro_rules! syscall {
    (fn $name:ident($($arg_name:ident: $t:ty),*) via $_sys_name:ident -> $ret:ty) => (
        unsafe fn $name($($arg_name: $t),*) -> $ret {
            weak! { fn $name($($t),*) -> $ret }

            if let Some(fun) = $name.get() {
                fun($($arg_name),*)
            } else {
                libc_errno::set_errno(libc_errno::Errno(libc::ENOSYS));
                -1
            }
        }
    )
}

#[cfg(linux_kernel)]
macro_rules! syscall {
    (fn $name:ident($($arg_name:ident: $t:ty),*) via $sys_name:ident -> $ret:ty) => (
        unsafe fn $name($($arg_name:$t),*) -> $ret {
            // This looks like a hack, but `concat_idents` only accepts idents
            // (not paths).
            use libc::*;

            trait AsSyscallArg {
                type SyscallArgType;
                fn into_syscall_arg(self) -> Self::SyscallArgType;
            }

            // Pass pointer types as pointers, to preserve provenance.
            impl<T> AsSyscallArg for *mut T {
                type SyscallArgType = *mut T;
                fn into_syscall_arg(self) -> Self::SyscallArgType { self }
            }
            impl<T> AsSyscallArg for *const T {
                type SyscallArgType = *const T;
                fn into_syscall_arg(self) -> Self::SyscallArgType { self }
            }

            // Pass `BorrowedFd` values as the integer value.
            impl AsSyscallArg for $crate::fd::BorrowedFd<'_> {
                type SyscallArgType = c::c_long;
                fn into_syscall_arg(self) -> Self::SyscallArgType {
                    $crate::fd::AsRawFd::as_raw_fd(&self) as _
                }
            }

            // Coerce integer values into `c_long`.
            impl AsSyscallArg for i32 {
                type SyscallArgType = c::c_long;
                fn into_syscall_arg(self) -> Self::SyscallArgType { self as _ }
            }
            impl AsSyscallArg for u32 {
                type SyscallArgType = c::c_long;
                fn into_syscall_arg(self) -> Self::SyscallArgType { self as _ }
            }
            impl AsSyscallArg for usize {
                type SyscallArgType = c::c_long;
                fn into_syscall_arg(self) -> Self::SyscallArgType { self as _ }
            }

            // `concat_idents` is [unstable], so we take an extra `sys_name`
            // parameter and have our users do the concat for us for now.
            //
            // [unstable]: https://github.com/rust-lang/rust/issues/29599
            /*
            syscall(
                concat_idents!(SYS_, $name),
                $($arg_name.into_syscall_arg()),*
            ) as $ret
            */

            syscall($sys_name, $($arg_name.into_syscall_arg()),*) as $ret
        }
    )
}

macro_rules! weakcall {
    ($vis:vis fn $name:ident($($arg_name:ident: $t:ty),*) -> $ret:ty) => (
        $vis unsafe fn $name($($arg_name: $t),*) -> $ret {
            weak! { fn $name($($t),*) -> $ret }

            // Use a weak symbol from libc when possible, allowing `LD_PRELOAD`
            // interposition, but if it's not found just fail.
            if let Some(fun) = $name.get() {
                fun($($arg_name),*)
            } else {
                libc_errno::set_errno(libc_errno::Errno(libc::ENOSYS));
                -1
            }
        }
    )
}

/// A combination of `weakcall` and `syscall`. Use the libc function if it's
/// available, and fall back to `libc::syscall` otherwise.
macro_rules! weak_or_syscall {
    ($vis:vis fn $name:ident($($arg_name:ident: $t:ty),*) via $sys_name:ident -> $ret:ty) => (
        $vis unsafe fn $name($($arg_name: $t),*) -> $ret {
            weak! { fn $name($($t),*) -> $ret }

            // Use a weak symbol from libc when possible, allowing `LD_PRELOAD`
            // interposition, but if it's not found just fail.
            if let Some(fun) = $name.get() {
                fun($($arg_name),*)
            } else {
                syscall! { fn $name($($arg_name: $t),*) via $sys_name -> $ret }
                $name($($arg_name),*)
            }
        }
    )
}
