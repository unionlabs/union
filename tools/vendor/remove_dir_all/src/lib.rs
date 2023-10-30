//! Reliably remove a directory and all of its children.
//!
//! This library provides a reliable implementation of `remove_dir_all` for Windows.
//! For Unix systems, it re-exports `std::fs::remove_dir_all`.
//!
//! It also provides `remove_dir_contents` and `ensure_empty_dir`
//! for both Unix and Windows.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
// See under "known problems" https://rust-lang.github.io/rust-clippy/master/index.html#mutex_atomic
#![allow(clippy::mutex_atomic)]

#[cfg(doctest)]
#[macro_use]
extern crate doc_comment;

#[cfg(doctest)]
doctest!("../README.md");

#[cfg(windows)]
mod fs;

#[cfg(not(windows))]
mod unix;

mod portable;

#[cfg(windows)]
pub use self::fs::remove_dir_all;

#[cfg(not(windows))]
pub use std::fs::remove_dir_all;

pub use portable::ensure_empty_dir;
pub use portable::remove_dir_contents;
