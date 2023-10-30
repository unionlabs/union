//! Support for reading AIX XCOFF files.
//!
//! Provides `XcoffFile` and related types which implement the `Object` trait.

mod file;
pub use file::*;

mod section;
pub use section::*;

mod symbol;
pub use symbol::*;

mod relocation;
pub use relocation::*;

mod comdat;
pub use comdat::*;

mod segment;
pub use segment::*;
