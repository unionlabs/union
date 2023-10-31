//! The Rust core string library
//!
//! This library provides a UTF-8 encoded, growable string.

mod extend;
mod from_stream;

#[doc(inline)]
pub use std::string::String;
