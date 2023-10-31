//! # `object`
//!
//! The `object` crate provides a unified interface to working with object files
//! across platforms. It supports reading object files and executable files,
//! and writing object files and some executable files.
//!
//! ## Raw struct definitions
//!
//! Raw structs are defined for: [ELF](elf), [Mach-O](macho), [PE/COFF](pe), [archive].
//! Types and traits for zerocopy support are defined in [pod] and [endian].
//!
//! ## Unified read API
//!
//! The [read::Object] trait defines the unified interace. This trait is implemented
//! by [read::File], which allows reading any file format, as well as implementations
//! for each file format: [ELF](read::elf::ElfFile), [Mach-O](read::macho::MachOFile),
//! [COFF](read::coff::CoffFile), [PE](read::pe::PeFile), [Wasm](read::wasm::WasmFile).
//!
//! ## Low level read API
//!
//! In addition to the unified read API, the various `read` modules define helpers that
//! operate on the raw structs. These also provide traits that abstract over the differences
//! between 32-bit and 64-bit versions of the file format.
//!
//! ## Unified write API
//!
//! [write::Object] allows building a COFF/ELF/Mach-O object and then writing it out.
//!
//! ## Low level executable writers
//!
//! [write::elf::Writer] and [write::pe::Writer] allow writing executable files.
//!
//! ## Example for unified read API
//!  ```no_run
//! # #[cfg(feature = "read")]
//! use object::{Object, ObjectSection};
//! use std::error::Error;
//! use std::fs;
//!
//! /// Reads a file and displays the content of the ".boot" section.
//! fn main() -> Result<(), Box<dyn Error>> {
//! # #[cfg(all(feature = "read", feature = "std"))] {
//!   let bin_data = fs::read("./multiboot2-binary.elf")?;
//!   let obj_file = object::File::parse(&*bin_data)?;
//!   if let Some(section) = obj_file.section_by_name(".boot") {
//!     println!("{:#x?}", section.data()?);
//!   } else {
//!     eprintln!("section not available");
//!   }
//! # }
//!   Ok(())
//! }
//! ```

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![no_std]
// Style.
#![allow(clippy::collapsible_if)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::single_match)]
#![allow(clippy::type_complexity)]
// Occurs due to fallible iteration.
#![allow(clippy::should_implement_trait)]
// Unit errors are converted to other types by callers.
#![allow(clippy::result_unit_err)]
// Clippy is wrong.
#![allow(clippy::transmute_ptr_to_ptr)]
// Worse readability sometimes.
#![allow(clippy::collapsible_else_if)]

#[cfg(feature = "cargo-all")]
compile_error!("'--all-features' is not supported; use '--features all' instead");

#[cfg(all(feature = "xcoff", not(feature = "unstable")))]
compile_error!("'xcoff` is an unstable feature; enable 'unstable' as well");

#[cfg(any(feature = "read_core", feature = "write_core"))]
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
#[allow(unused_imports)]
#[macro_use]
extern crate std;

mod common;
pub use common::*;

#[macro_use]
pub mod endian;
pub use endian::*;

#[macro_use]
pub mod pod;
pub use pod::*;

#[cfg(feature = "read_core")]
pub mod read;
#[cfg(feature = "read_core")]
pub use read::*;

#[cfg(feature = "write_core")]
pub mod write;

#[cfg(feature = "archive")]
pub mod archive;
#[cfg(feature = "elf")]
pub mod elf;
#[cfg(feature = "macho")]
pub mod macho;
#[cfg(any(feature = "coff", feature = "pe"))]
pub mod pe;
#[cfg(feature = "xcoff")]
pub mod xcoff;
