// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! This section defines the `PROBESTACK` intrinsic which is used in the
//! implementation of "stack probes" on certain platforms.
//!
//! The purpose of a stack probe is to provide a static guarantee that if a
//! thread has a guard page then a stack overflow is guaranteed to hit that
//! guard page. If a function did not have a stack probe then there's a risk of
//! having a stack frame *larger* than the guard page, so a function call could
//! skip over the guard page entirely and then later hit maybe the heap or
//! another thread, possibly leading to security vulnerabilities such as [The
//! Stack Clash], for example.
//!
//! [The Stack Clash]: https://blog.qualys.com/securitylabs/2017/06/19/the-stack-clash

// A declaration for the stack probe function in Rust's standard library, for
// catching callstack overflow.
cfg_if::cfg_if! {
    if #[cfg(all(
            target_os = "windows",
            target_env = "msvc",
            target_pointer_width = "64"
            ))] {
        extern "C" {
            pub fn __chkstk();
        }
        /// The probestack for 64bit Windows when compiled with MSVC (note the double underscore)
        pub const PROBESTACK: unsafe extern "C" fn() = __chkstk;
    } else if #[cfg(all(
            target_os = "windows",
            target_env = "msvc",
            target_pointer_width = "32"
            ))] {
        extern "C" {
            pub fn _chkstk();
        }
        /// The probestack for 32bit Windows when compiled with MSVC (note the singular underscore)
        pub const PROBESTACK: unsafe extern "C" fn() = _chkstk;
    } else if #[cfg(all(target_os = "windows", target_env = "gnu"))] {
        extern "C" {
            // ___chkstk (note the triple underscore) is implemented in compiler-builtins/src/x86_64.rs
            // by the Rust compiler for the MinGW target
            #[cfg(all(target_os = "windows", target_env = "gnu"))]
            pub fn ___chkstk();
        }
        /// The probestack for Windows when compiled with GNU
        pub const PROBESTACK: unsafe extern "C" fn() = ___chkstk;
    } else if #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))] {
        // As per
        // https://github.com/rust-lang/compiler-builtins/blob/cae3e6ea23739166504f9f9fb50ec070097979d4/src/probestack.rs#L39,
        // LLVM only has stack-probe support on x86-64 and x86. Thus, on any other CPU
        // architecture, we simply use an empty stack-probe function.
        extern "C" fn empty_probestack() {}
        /// A default probestack for other architectures
        pub const PROBESTACK: unsafe extern "C" fn() = empty_probestack;
    } else {
        extern "C" {
            pub fn __rust_probestack();
        }
        /// The probestack based on the Rust probestack
        pub static PROBESTACK: unsafe extern "C" fn() = __rust_probestack;
    }
}
