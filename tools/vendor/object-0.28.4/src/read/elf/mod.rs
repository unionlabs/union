//! Support for reading ELF files.
//!
//! Defines traits to abstract over the difference between ELF32/ELF64,
//! and implements read functionality in terms of these traits.
//!
//! Also provides `ElfFile` and related types which implement the `Object` trait.

mod file;
pub use file::*;

mod segment;
pub use segment::*;

mod section;
pub use section::*;

mod symbol;
pub use symbol::*;

mod relocation;
pub use relocation::*;

mod comdat;
pub use comdat::*;

mod dynamic;
pub use dynamic::*;

mod compression;
pub use compression::*;

mod note;
pub use note::*;

mod hash;
pub use hash::*;

mod version;
pub use version::*;
