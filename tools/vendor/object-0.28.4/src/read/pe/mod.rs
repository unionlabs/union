//! Support for reading PE files.
//!
//! Defines traits to abstract over the difference between PE32/PE32+,
//! and implements read functionality in terms of these traits.
//!
//! This module reuses some of the COFF functionality.
//!
//! Also provides `PeFile` and related types which implement the `Object` trait.

mod file;
pub use file::*;

mod section;
pub use section::*;

mod data_directory;
pub use data_directory::*;

mod export;
pub use export::*;

mod import;
pub use import::*;

mod relocation;
pub use relocation::*;

mod resource;
pub use resource::*;

mod rich;
pub use rich::*;

pub use super::coff::{SectionTable, SymbolTable};
