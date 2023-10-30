//! Styling legacy Windows terminals
//!
//! See [`Console`]
//!
//! This fills a similar role as [`winapi-util`](https://crates.io/crates/winapi-util) does for
//! [`termcolor`](https://crates.io/crates/termcolor) with the differences
//! - Uses `windows-sys` rather than `winapi`
//! - Uses [`anstyle`](https://crates.io/crates/termcolor) rather than defining its own types
//! - More focused, smaller

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod console;
mod lockable;
mod stream;
#[cfg(windows)]
mod windows;

pub use console::Console;
pub use lockable::Lockable;
pub use stream::WinconStream;
