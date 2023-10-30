//! WebAssembly format library
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub mod builder;
pub mod elements;
mod io;

pub use elements::{deserialize_buffer, peek_size, serialize, Error as SerializationError};

#[cfg(feature = "std")]
pub use elements::{deserialize_file, serialize_to_file};
