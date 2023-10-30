//! This file contains the logic for propagating panics from a coroutine up to
//! its parent.
//!
//! If the `asm-unwind` feature is used then we can let exceptions unwind
//! through our inline assembly code directly by using the `asm-unwind` and
//! `c_unwind` nightly features.
//!
//! If nighlty Rust is not available then we catch panics with `catch_unwind`,
//! pass them through the assembly code as a `Result` and then call
//! `resume_unwind` to continue unwinding afterwards.
//!
//! If the standard library is not available then we have no way of catching
//! panics and must abort if one is about the reach the assembly code at the
//! root of a coroutine.

#![allow(unused_macros)]

use crate::stack::StackPointer;
use crate::util::EncodedValue;

cfg_if::cfg_if! {
    if #[cfg(feature = "unwind")] {
        extern crate std;
        use core::any::Any;
        use std::boxed::Box;
        use std::panic::{self, AssertUnwindSafe};
    }
}

// DW_CFA_GNU_args_size 0
//
// Indicates to the unwinder that this "call" does not take any arguments and no
// stack space needs to be popped before executing a landing pad. This is mainly
// used to undo the effect of any previous  DW_CFA_GNU_args_size that may have
// been set in the current function.
//
// This macro is a no-op on targets that don't use DWARF CFI for throwing
// exceptions. It should only be used in inline assembly with the may_unwind
// option to ensure that the surrounding function has CFI attributes.
cfg_if::cfg_if! {
    if #[cfg(any(not(windows), all(target_arch = "x86", target_env = "gnu")))] {
        macro_rules! cfi_reset_args_size {
            () => {
                ".cfi_escape 0x2e, 0x00"
            };
        }
    } else {
        macro_rules! cfi_reset_args_size {
            () => {
                ""
            };
        }
    }
}

// These definitions deal with propagating panics that occur inside a coroutine
// back out to its parent.
//
// On non-Windows platforms we can let the exception unwind straight through
// the stack switching trampoline when the "asm-unwind" feature is enabled.
//
// This is not supported on Windows because we need to update TEB fields when
// passing the exception across stack boundaries. This cannot be done with just
// unwind info, it requires a manual switch.
cfg_if::cfg_if! {
    if #[cfg(all(feature = "asm-unwind", not(windows)))] {
        macro_rules! asm_may_unwind_root {
            ($($tt:tt)*) => {
                // We assume $tt already includes a trailing comma.
                core::arch::asm!($($tt)* options(may_unwind))
            };
        }
        macro_rules! cfi_reset_args_size_root {
            () => {
                crate::unwind::cfi_reset_args_size!()
            }
        }

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                // Force the use of the SysV64 ABI on Windows, it makes the
                // assembly code much simpler since we don't need to worry about
                // the 32-byte shadow space and can use the same registers as
                // the generic x86_64 implementation.
                pub type InitialFunc<T> = unsafe extern "sysv64-unwind" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> = unsafe extern "sysv64-unwind" fn(
                    val: *mut T,
                    parent_link: *mut StackPointer,
                ) -> !;
                pub type StackCallFunc = unsafe extern "sysv64-unwind" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "sysv64-unwind" fn $($tt)*
                    }
                }
            } else if #[cfg(target_arch = "x86")] {
                // Use fastcall on x86 to pass arguments in registers instead of
                // using the stack.
                pub type InitialFunc<T> = unsafe extern "fastcall-unwind" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> = unsafe extern "fastcall-unwind" fn(
                    val: *mut T,
                    parent_link: *mut StackPointer,
                ) -> !;
                pub type StackCallFunc = unsafe extern "fastcall-unwind" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "fastcall-unwind" fn $($tt)*
                    }
                }
            } else {
                pub type InitialFunc<T> = unsafe extern "C-unwind" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> =
                    unsafe extern "C-unwind" fn(val: *mut T, parent_link: *mut StackPointer) -> !;
                pub type StackCallFunc = unsafe extern "C-unwind" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "C-unwind" fn $($tt)*
                    }
                }
            }
        }
    } else {
        macro_rules! asm_may_unwind_root {
            ($($tt:tt)*) => {
                core::arch::asm!($($tt)*)
            };
        }
        macro_rules! cfi_reset_args_size_root {
            () => {
                ""
            }
        }

        cfg_if::cfg_if! {
            // Force the use of the SysV64 ABI on Windows, it makes the
            // assembly code much simpler since we don't need to worry about
            // the 32-byte shadow space and can use the same registers as
            // the generic x86_64 implementation.
            if #[cfg(target_arch = "x86_64")] {
                pub type InitialFunc<T> = unsafe extern "sysv64" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> =
                    unsafe extern "sysv64" fn(val: *mut T, parent_link: *mut StackPointer) -> !;
                pub type StackCallFunc = unsafe extern "sysv64" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "sysv64" fn $($tt)*
                    }
                }
            } else if #[cfg(target_arch = "x86")] {
                // Use fastcall on x86 to pass arguments in registers instead of
                // using the stack.
                pub type InitialFunc<T> = unsafe extern "fastcall" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> =
                    unsafe extern "fastcall" fn(val: *mut T, parent_link: *mut StackPointer) -> !;
                pub type StackCallFunc = unsafe extern "fastcall" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "fastcall" fn $($tt)*
                    }
                }
            } else {
                pub type InitialFunc<T> = unsafe extern "C" fn(
                    arg: EncodedValue,
                    sp: &mut StackPointer,
                    obj: *mut T,
                ) -> !;
                pub type TrapHandler<T> =
                    unsafe extern "C" fn(val: *mut T, parent_link: *mut StackPointer) -> !;
                pub type StackCallFunc = unsafe extern "C" fn(ptr: *mut u8);
                macro_rules! initial_func_abi {
                    (unsafe fn $($tt:tt)*) => {
                        unsafe extern "C" fn $($tt)*
                    }
                }
            }
        }
    }
}
cfg_if::cfg_if! {
    if #[cfg(all(feature = "asm-unwind", not(windows)))] {
        pub type CaughtPanic = core::convert::Infallible;

        #[inline]
        pub fn catch_unwind_at_root<T, F: FnOnce() -> T>(f: F) -> Result<T, CaughtPanic> {
            Ok(f())
        }

        #[inline]
        pub fn catch_forced_unwind<T>(
            f: impl FnOnce() -> Result<T, CaughtPanic>
        )-> Result<T, Box<dyn Any + Send>> {
            panic::catch_unwind(AssertUnwindSafe(|| f().unwrap()))
        }

        #[inline]
        pub fn maybe_resume_unwind<T>(val: Result<T, CaughtPanic>) -> T {
            match val {
                Ok(val) => val,
                Err(_) => unreachable!(),
            }
        }
    } else if #[cfg(feature = "unwind")] {
        pub type CaughtPanic = Box<dyn Any + Send>;

        #[inline]
        pub fn catch_unwind_at_root<T, F: FnOnce() -> T>(f: F) -> Result<T, CaughtPanic> {
            panic::catch_unwind(AssertUnwindSafe(f))
        }

        #[inline]
        pub fn catch_forced_unwind<T>(f: impl FnOnce() -> Result<T, CaughtPanic>) -> Result<T, Box<dyn Any + Send>> {
            f()
        }

        #[inline]
        pub fn maybe_resume_unwind<T>(val: Result<T, CaughtPanic>) -> T {
            match val {
                Ok(val) => val,
                Err(e) => panic::resume_unwind(e),
            }
        }
    } else {
        pub type CaughtPanic = core::convert::Infallible;

        #[inline]
        pub fn catch_unwind_at_root<T, F: FnOnce() -> T>(f: F) -> Result<T, CaughtPanic> {
            let guard = scopeguard::guard((), |()| {
                // We can't catch panics in #![no_std], force an abort using
                // a double-panic. Add a recursive panic guard just in case
                // to ensure this function never returns with or without
                // unwinding.
                panic!("cannot propagte coroutine panic with #![no_std]");
            });
            let result = f();
            core::mem::forget(guard);
            Ok(result)
        }

        #[inline]
        pub fn maybe_resume_unwind<T>(val: Result<T, CaughtPanic>) -> T {
            match val {
                Ok(val) => val,
                Err(_) => unreachable!(),
            }
        }
    }
}

// These definitions deal with throwing a ForcedUnwind exception upon resuming a
// coroutine so that the coroutine state is safely unwound back to its root.
//
// This is only possible with standard library support since there is no way of
// triggering unwinding with #![no_std].
//
// Unlike the above, this does not require stack switch so it works just fine on
// Windows.
cfg_if::cfg_if! {
    if #[cfg(feature = "asm-unwind")] {
        macro_rules! asm_may_unwind_yield {
            ($($tt:tt)*) => {
                // We assume $tt already includes a trailing comma.
                core::arch::asm!($($tt)* options(may_unwind))
            };
        }
        macro_rules! cfi_reset_args_size_yield {
            () => {
                crate::unwind::cfi_reset_args_size!()
            }
        }
    } else {
        macro_rules! asm_may_unwind_yield {
            ($($tt:tt)*) => {
                core::arch::asm!($($tt)*)
            };
        }
        macro_rules! cfi_reset_args_size_yield {
            () => {
                ""
            }
        }
    }
}
cfg_if::cfg_if! {
    if #[cfg(feature = "asm-unwind")] {
        #[repr(transparent)]
        pub struct ForcedUnwind(pub StackPointer);
        pub type ForcedUnwindErr = core::convert::Infallible;

        #[inline]
        pub fn maybe_force_unwind<T>(val: Result<T, ForcedUnwindErr>) -> T {
            match val {
                Ok(val) => val,
                Err(_) => unreachable!(),
            }
        }
    } else if #[cfg(feature = "unwind")] {
        #[repr(transparent)]
        pub struct ForcedUnwind(pub StackPointer);
        pub type ForcedUnwindErr = ForcedUnwind;

        #[inline]
        pub fn maybe_force_unwind<T>(val: Result<T, ForcedUnwindErr>) -> T {
            match val {
                Ok(val) => val,
                Err(e) => panic::resume_unwind(Box::new(e)),
            }
        }
    } else {
        pub type ForcedUnwindErr = core::convert::Infallible;

        #[inline]
        pub fn maybe_force_unwind<T>(val: Result<T, ForcedUnwindErr>) -> T {
            match val {
                Ok(val) => val,
                Err(_) => unreachable!(),
            }
        }
    }
}

#[allow(unused_imports)]
pub(crate) use {
    asm_may_unwind_root, asm_may_unwind_yield, cfi_reset_args_size, cfi_reset_args_size_root,
    cfi_reset_args_size_yield, initial_func_abi,
};
