//! This are the common types and utility tools for using WebAssembly
//! in a Rust environment.
//!
//! This crate provides common structures such as `Type` or `Value`, type indexes
//! and native function wrappers with `Func`.

#![deny(missing_docs, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(feature = "std", deny(unstable_features))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        clippy::float_arithmetic,
        clippy::mut_mut,
        clippy::nonminimal_bool,
        clippy::map_unwrap_or,
        clippy::print_stdout,
        clippy::unicode_not_nfc,
        clippy::use_self
    )
)]

#[cfg(all(feature = "std", feature = "core"))]
compile_error!(
    "The `std` and `core` features are both enabled, which is an error. Please enable only once."
);

#[cfg(all(not(feature = "std"), not(feature = "core")))]
compile_error!("Both the `std` and `core` features are disabled. Please enable one of them.");

#[cfg(feature = "core")]
extern crate alloc;

/// The `lib` module defines a `std` module that is identical whether
/// the `core` or the `std` feature is enabled.
pub mod lib {
    /// Custom `std` module.
    #[cfg(feature = "core")]
    pub mod std {
        pub use alloc::{borrow, boxed, format, iter, rc, slice, string, vec};
        pub use core::{any, cell, cmp, convert, fmt, hash, marker, mem, ops, ptr, sync, u32};
    }

    /// Custom `std` module.
    #[cfg(feature = "std")]
    pub mod std {
        pub use std::{
            any, borrow, boxed, cell, cmp, convert, fmt, format, hash, iter, marker, mem, ops, ptr,
            rc, slice, string, sync, u32, vec,
        };
    }
}

#[cfg(feature = "enable-rkyv")]
mod archives;
mod extern_ref;
mod features;
mod indexes;
mod initializers;
mod libcalls;
mod memory;
mod memory_view;
mod module;
mod native;
mod table;
mod trapcode;
mod types;
mod units;
mod utils;
mod values;
mod vmoffsets;

/// The entity module, with common helpers for Rust structures
pub mod entity;
pub use crate::extern_ref::{ExternRef, VMExternRef};
pub use crate::features::Features;
pub use crate::indexes::{
    CustomSectionIndex, DataIndex, ElemIndex, ExportIndex, FunctionIndex, GlobalIndex, ImportIndex,
    LocalFunctionIndex, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex,
    SignatureIndex, TableIndex,
};
pub use crate::initializers::{
    DataInitializer, DataInitializerLocation, OwnedDataInitializer, TableInitializer,
};
pub use crate::memory_view::{Atomically, MemoryView};
pub use crate::module::{ExportsIterator, ImportsIterator, ModuleInfo};
pub use crate::native::{NativeWasmType, ValueType};
pub use crate::units::{
    Bytes, PageCountOutOfRange, Pages, WASM_MAX_PAGES, WASM_MIN_PAGES, WASM_PAGE_SIZE,
};
pub use crate::values::{Value, WasmValueType};
pub use types::{
    ExportType, ExternType, FunctionType, GlobalInit, GlobalType, ImportType, MemoryType,
    Mutability, TableType, Type, V128,
};

#[cfg(feature = "enable-rkyv")]
pub use archives::ArchivableIndexMap;

pub use crate::libcalls::LibCall;
pub use crate::memory::MemoryStyle;
pub use crate::table::TableStyle;
pub use crate::trapcode::TrapCode;
pub use crate::vmoffsets::{TargetSharedSignatureIndex, VMBuiltinFunctionIndex, VMOffsets};

pub use crate::utils::is_wasm;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
