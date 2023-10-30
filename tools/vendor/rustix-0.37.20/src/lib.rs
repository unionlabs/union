//! `rustix` provides efficient memory-safe and [I/O-safe] wrappers to
//! POSIX-like, Unix-like, Linux, and Winsock2 syscall-like APIs, with
//! configurable backends.
//!
//! With rustix, you can write code like this:
//!
//! ```
//! # #[cfg(feature = "net")]
//! # fn read(sock: std::net::TcpStream, buf: &mut [u8]) -> std::io::Result<()> {
//! # use rustix::net::RecvFlags;
//! let nread: usize = rustix::net::recv(&sock, buf, RecvFlags::PEEK)?;
//! # let _ = nread;
//! # Ok(())
//! # }
//! ```
//!
//! instead of like this:
//!
//! ```
//! # #[cfg(feature = "net")]
//! # fn read(sock: std::net::TcpStream, buf: &mut [u8]) -> std::io::Result<()> {
//! # use std::convert::TryInto;
//! # #[cfg(unix)]
//! # use std::os::unix::io::AsRawFd;
//! # #[cfg(target_os = "wasi")]
//! # use std::os::wasi::io::AsRawFd;
//! # #[cfg(windows)]
//! # use windows_sys::Win32::Networking::WinSock as libc;
//! # #[cfg(windows)]
//! # use std::os::windows::io::AsRawSocket;
//! # const MSG_PEEK: i32 = libc::MSG_PEEK;
//! let nread: usize = unsafe {
//!     #[cfg(any(unix, target_os = "wasi"))]
//!     let raw = sock.as_raw_fd();
//!     #[cfg(windows)]
//!     let raw = sock.as_raw_socket();
//!     match libc::recv(
//!         raw as _,
//!         buf.as_mut_ptr().cast(),
//!         buf.len().try_into().unwrap_or(i32::MAX as _),
//!         MSG_PEEK,
//!     ) {
//!         -1 => return Err(std::io::Error::last_os_error()),
//!         nread => nread as usize,
//!     }
//! };
//! # let _ = nread;
//! # Ok(())
//! # }
//! ```
//!
//! rustix's APIs perform the following tasks:
//!  - Error values are translated to [`Result`]s.
//!  - Buffers are passed as Rust slices.
//!  - Out-parameters are presented as return values.
//!  - Path arguments use [`Arg`], so they accept any string type.
//!  - File descriptors are passed and returned via [`AsFd`] and [`OwnedFd`]
//!    instead of bare integers, ensuring I/O safety.
//!  - Constants use `enum`s and [`bitflags`] types.
//!  - Multiplexed functions (eg. `fcntl`, `ioctl`, etc.) are de-multiplexed.
//!  - Variadic functions (eg. `openat`, etc.) are presented as non-variadic.
//!  - Functions that return strings automatically allocate sufficient memory
//!    and retry the syscall as needed to determine the needed length.
//!  - Functions and types which need `l` prefixes or `64` suffixes to enable
//!    large-file support (LFS) are used automatically. File sizes and offsets
//!    are always presented as `u64` and `i64`.
//!  - Behaviors that depend on the sizes of C types like `long` are hidden.
//!  - In some places, more human-friendly and less historical-accident names
//!    are used (and documentation aliases are used so that the original names
//!    can still be searched for).
//!  - Provide y2038 compatibility, on platforms which support this.
//!  - Correct selected platform bugs, such as behavioral differences when
//!    running under seccomp.
//!
//! Things they don't do include:
//!  - Detecting whether functions are supported at runtime, except in specific
//!    cases where new interfaces need to be detected to support y2038 and LFS.
//!  - Hiding significant differences between platforms.
//!  - Restricting ambient authorities.
//!  - Imposing sandboxing features such as filesystem path or network address
//!    sandboxing.
//!
//! See [`cap-std`], [`system-interface`], and [`io-streams`] for libraries
//! which do hide significant differences between platforms, and [`cap-std`]
//! which does perform sandboxing and restricts ambient authorities.
//!
//! [`cap-std`]: https://crates.io/crates/cap-std
//! [`system-interface`]: https://crates.io/crates/system-interface
//! [`io-streams`]: https://crates.io/crates/io-streams
//! [`getrandom`]: https://crates.io/crates/getrandom
//! [`bitflags`]: https://crates.io/crates/bitflags
//! [`AsFd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.AsFd.html
//! [`OwnedFd`]: https://doc.rust-lang.org/stable/std/os/fd/struct.OwnedFd.html
//! [I/O-safe]: https://github.com/rust-lang/rfcs/blob/master/text/3128-io-safety.md
//! [`Result`]: https://doc.rust-lang.org/stable/std/result/enum.Result.html
//! [`Arg`]: https://docs.rs/rustix/*/rustix/path/trait.Arg.html

#![deny(missing_docs)]
#![allow(stable_features)]
#![cfg_attr(linux_raw, deny(unsafe_code))]
#![cfg_attr(rustc_attrs, feature(rustc_attrs))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(all(wasi_ext, target_os = "wasi", feature = "std"), feature(wasi_ext))]
#![cfg_attr(
    all(linux_raw, naked_functions, target_arch = "x86"),
    feature(naked_functions)
)]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg_attr(core_ffi_c, feature(core_ffi_c))]
#![cfg_attr(core_c_str, feature(core_c_str))]
#![cfg_attr(alloc_c_string, feature(alloc_ffi))]
#![cfg_attr(alloc_c_string, feature(alloc_c_string))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "rustc-dep-of-std", feature(ip))]
#![cfg_attr(
    any(feature = "rustc-dep-of-std", core_intrinsics),
    feature(core_intrinsics)
)]
#![cfg_attr(asm_experimental_arch, feature(asm_experimental_arch))]
#![cfg_attr(not(feature = "all-apis"), allow(dead_code))]
// Clamp depends on Rust 1.50 which is newer than our MSRV.
#![allow(clippy::manual_clamp)]
// It is common in linux and libc APIs for types to vary between platforms.
#![allow(clippy::unnecessary_cast)]
// It is common in linux and libc APIs for types to vary between platforms.
#![allow(clippy::useless_conversion)]
// Redox and WASI have enough differences that it isn't worth
// precisely conditionallizing all the `use`s for them.
#![cfg_attr(any(target_os = "redox", target_os = "wasi"), allow(unused_imports))]

#[cfg(not(feature = "rustc-dep-of-std"))]
extern crate alloc;

// Internal utilities.
#[cfg(not(windows))]
#[macro_use]
pub(crate) mod cstr;
#[macro_use]
pub(crate) mod const_assert;
pub(crate) mod utils;

// linux_raw: Weak symbols are used by the use-libc-auxv feature for
// glibc 2.15 support.
//
// libc: Weak symbols are used to call various functions available in some
// versions of libc and not others.
#[cfg(any(
    all(linux_raw, feature = "use-libc-auxv"),
    all(libc, not(any(windows, target_os = "wasi")))
))]
#[macro_use]
mod weak;

// Pick the backend implementation to use.
#[cfg_attr(libc, path = "backend/libc/mod.rs")]
#[cfg_attr(linux_raw, path = "backend/linux_raw/mod.rs")]
#[cfg_attr(wasi, path = "backend/wasi/mod.rs")]
mod backend;

/// Export the `*Fd` types and traits that are used in rustix's public API.
///
/// Users can use this to avoid needing to import anything else to use the same
/// versions of these types and traits.
pub mod fd {
    use super::backend;
    #[cfg(windows)]
    pub use backend::fd::AsSocket;
    pub use backend::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
}

// The public API modules.
#[cfg(not(windows))]
pub mod ffi;
#[cfg(not(windows))]
#[cfg(feature = "fs")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "fs")))]
pub mod fs;
pub mod io;
#[cfg(linux_kernel)]
#[cfg(feature = "io_uring")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "io_uring")))]
pub mod io_uring;
#[cfg(not(any(windows, target_os = "wasi")))]
#[cfg(feature = "mm")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "mm")))]
pub mod mm;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[cfg(feature = "net")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "net")))]
pub mod net;
#[cfg(not(windows))]
#[cfg(feature = "param")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "param")))]
pub mod param;
#[cfg(not(windows))]
#[cfg(any(feature = "fs", feature = "net"))]
#[cfg_attr(doc_cfg, doc(cfg(any(feature = "fs", feature = "net"))))]
pub mod path;
#[cfg(not(windows))]
#[cfg(feature = "process")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "process")))]
pub mod process;
#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
#[cfg(feature = "pty")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "pty")))]
pub mod pty;
#[cfg(not(windows))]
#[cfg(feature = "rand")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "rand")))]
pub mod rand;
#[cfg(not(windows))]
#[cfg(feature = "termios")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "termios")))]
pub mod termios;
#[cfg(not(windows))]
#[cfg(feature = "thread")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "thread")))]
pub mod thread;
#[cfg(not(windows))]
#[cfg(feature = "time")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "time")))]
pub mod time;

// "runtime" is also a public API module, but it's only for libc-like users.
#[cfg(not(windows))]
#[cfg(feature = "runtime")]
#[doc(hidden)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "runtime")))]
pub mod runtime;

// We have some internal interdependencies in the API features, so for now,
// for API features that aren't enabled, declare them as `pub(crate)` so
// that they're not public, but still available for internal use.

#[cfg(not(windows))]
#[cfg(all(
    not(feature = "param"),
    any(feature = "runtime", feature = "time", target_arch = "x86"),
))]
pub(crate) mod param;
#[cfg(not(windows))]
#[cfg(not(any(feature = "fs", feature = "net")))]
pub(crate) mod path;
#[cfg(not(windows))]
#[cfg(not(feature = "process"))]
pub(crate) mod process;
