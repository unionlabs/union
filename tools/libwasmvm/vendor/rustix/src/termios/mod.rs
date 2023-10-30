//! Terminal I/O stream operations.

#[cfg(not(target_os = "wasi"))]
mod cf;
#[cfg(not(target_os = "wasi"))]
mod constants;
#[cfg(not(target_os = "wasi"))]
mod tc;
#[cfg(not(windows))]
mod tty;

#[cfg(not(target_os = "wasi"))]
pub use cf::*;
#[cfg(not(target_os = "wasi"))]
pub use constants::*;
#[cfg(not(target_os = "wasi"))]
pub use tc::*;
#[cfg(not(windows))]
pub use tty::*;
