//! A collection of errors specific to this crate's types that can occur when using them.

mod macros;

#[allow(unused_imports)]
pub use macros::*;

mod parseint;
pub use parseint::*;

mod tryfrom;
pub use tryfrom::*;
