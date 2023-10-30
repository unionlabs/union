//! A common backend for bindgen crates.
//!
//! This (internal) crate provides functionality common to multiple bindgen
//! dependency crates. There are 4 main things exported from this crate:
//!
//! 1. [**`TryToTokens`**](./trait.TryToTokens.html)
//!
//!    Provides the ability to attempt conversion from an AST struct
//!    into a TokenStream
//!
//! 2. [**`Diagnostic`**](./struct.Diagnostic.html)
//!
//!    A struct used to provide diagnostic responses for failures of said
//!    tokenization
//!
//! 3. [**`ast`**](./ast/index.html)
//!
//!    Abstract Syntax Tree types used to represent a Rust program, with
//!    the necessary metadata to generate bindings for it
//!
//! 4. [**`util`**](./util/index.html)
//!
//!    Common utilities for manipulating parsed types from syn
//!

#![recursion_limit = "256"]
#![cfg_attr(feature = "extra-traits", deny(missing_debug_implementations))]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/wasm-bindgen-backend/0.2")]

pub use crate::codegen::TryToTokens;
pub use crate::error::Diagnostic;

#[macro_use]
mod error;

pub mod ast;
mod codegen;
mod encode;
pub mod util;
