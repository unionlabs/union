//! Random-related operations.

#[cfg(any(linux_raw, all(libc, target_os = "linux")))]
mod getrandom;

#[cfg(any(linux_raw, all(libc, target_os = "linux")))]
pub use getrandom::{getrandom, GetRandomFlags};
