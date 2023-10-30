//! libc - Raw FFI bindings to platforms' system libraries
//!
//! [Documentation for other platforms][pd].
//!
//! [pd]: https://rust-lang.github.io/libc/#platform-specific-documentation
#![crate_name = "libc"]
#![crate_type = "rlib"]
#![allow(
    renamed_and_removed_lints, // Keep this order.
    unknown_lints, // Keep this order.
    bad_style,
    overflowing_literals,
    improper_ctypes,
    // This lint is renamed but we run CI for old stable rustc so should be here.
    redundant_semicolon,
    redundant_semicolons,
    unused_macros,
    unused_macro_rules,
)]
#![cfg_attr(libc_deny_warnings, deny(warnings))]
// Attributes needed when building as part of the standard library
#![cfg_attr(feature = "rustc-dep-of-std", feature(link_cfg, no_core))]
#![cfg_attr(libc_thread_local, feature(thread_local))]
// Enable extra lints:
#![cfg_attr(feature = "extra_traits", deny(missing_debug_implementations))]
#![deny(missing_copy_implementations, safe_packed_borrows)]
#![cfg_attr(not(feature = "rustc-dep-of-std"), no_std)]
#![cfg_attr(feature = "rustc-dep-of-std", no_core)]
#![cfg_attr(libc_const_extern_fn_unstable, feature(const_extern_fn))]

#[macro_use]
mod macros;

cfg_if! {
    if #[cfg(feature = "rustc-dep-of-std")] {
        extern crate rustc_std_workspace_core as core;
        #[allow(unused_imports)]
        use core::iter;
        #[allow(unused_imports)]
        use core::ops;
        #[allow(unused_imports)]
        use core::option;
    }
}

cfg_if! {
    if #[cfg(libc_priv_mod_use)] {
        #[cfg(libc_core_cvoid)]
        #[allow(unused_imports)]
        use core::ffi;
        #[allow(unused_imports)]
        use core::fmt;
        #[allow(unused_imports)]
        use core::hash;
        #[allow(unused_imports)]
        use core::num;
        #[allow(unused_imports)]
        use core::mem;
        #[doc(hidden)]
        #[allow(unused_imports)]
        use core::clone::Clone;
        #[doc(hidden)]
        #[allow(unused_imports)]
        use core::marker::{Copy, Send, Sync};
        #[doc(hidden)]
        #[allow(unused_imports)]
        use core::option::Option;
    } else {
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::fmt;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::hash;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::num;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::mem;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::clone::Clone;
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::marker::{Copy, Send, Sync};
        #[doc(hidden)]
        #[allow(unused_imports)]
        pub use core::option::Option;
    }
}

cfg_if! {
    if #[cfg(windows)] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod windows;
        pub use windows::*;
    } else if #[cfg(target_os = "fuchsia")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod fuchsia;
        pub use fuchsia::*;
    } else if #[cfg(target_os = "switch")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod switch;
        pub use switch::*;
    } else if #[cfg(target_os = "psp")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod psp;
        pub use psp::*;
    } else if #[cfg(target_os = "vxworks")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod vxworks;
        pub use vxworks::*;
    } else if #[cfg(target_os = "solid_asp3")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod solid;
        pub use solid::*;
    } else if #[cfg(unix)] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod unix;
        pub use unix::*;
    } else if #[cfg(target_os = "hermit")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod hermit;
        pub use hermit::*;
    } else if #[cfg(all(target_env = "sgx", target_vendor = "fortanix"))] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod sgx;
        pub use sgx::*;
    } else if #[cfg(any(target_env = "wasi", target_os = "wasi"))] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod wasi;
        pub use wasi::*;
    } else if #[cfg(target_os = "xous")] {
        mod fixed_width_ints;
        pub use fixed_width_ints::*;

        mod xous;
        pub use xous::*;
    } else {
        // non-supported targets: empty...
    }
}
