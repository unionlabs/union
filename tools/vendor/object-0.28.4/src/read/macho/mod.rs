//! Support for reading Mach-O files.
//!
//! Defines traits to abstract over the difference between 32-bit and 64-bit
//! Mach-O files, and implements read functionality in terms of these traits.
//!
//! Also provides `MachOFile` and related types which implement the `Object` trait.

mod dyld_cache;
pub use dyld_cache::*;

mod fat;
pub use fat::*;

mod file;
pub use file::*;

mod load_command;
pub use load_command::*;

mod segment;
pub use segment::*;

mod section;
pub use section::*;

mod symbol;
pub use symbol::*;

mod relocation;
pub use relocation::*;
