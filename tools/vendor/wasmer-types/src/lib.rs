//! This are the common types and utility tools for using WebAssembly
//! in a Rust environment.
//!
//! This crate provides common structures such as `Type` or `Value`, type indexes
//! and native function wrappers with `Func`.

#![deny(missing_docs, unused_extern_crates)]
#![warn(unused_import_braces)]
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
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

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

pub mod compilation;
pub mod error;
mod features;
mod indexes;
mod initializers;
mod libcalls;
mod memory;
mod module;
mod serialize;
mod stack;
mod store_id;
mod table;
mod trapcode;
mod types;
mod units;
mod utils;
mod value;
mod vmoffsets;

pub use crate::compilation::target::{
    Aarch64Architecture, Architecture, BinaryFormat, CallingConvention, CpuFeature, Endianness,
    Environment, OperatingSystem, PointerWidth, Target, Triple, Vendor,
};
pub use crate::serialize::{
    ArchivedSerializableCompilation, ArchivedSerializableModule, MetadataHeader,
    SerializableCompilation, SerializableModule,
};
pub use error::{
    CompileError, DeserializeError, ImportError, MemoryError, MiddlewareError,
    ParseCpuFeatureError, PreInstantiationError, SerializeError, WasmError, WasmResult,
};

/// The entity module, with common helpers for Rust structures
pub mod entity;
pub use crate::features::Features;
pub use crate::indexes::{
    CustomSectionIndex, DataIndex, ElemIndex, ExportIndex, FunctionIndex, GlobalIndex, ImportIndex,
    LocalFunctionIndex, LocalGlobalIndex, LocalMemoryIndex, LocalTableIndex, MemoryIndex,
    SignatureIndex, TableIndex,
};
pub use crate::initializers::{
    ArchivedDataInitializerLocation, ArchivedOwnedDataInitializer, DataInitializer,
    DataInitializerLike, DataInitializerLocation, DataInitializerLocationLike,
    OwnedDataInitializer, TableInitializer,
};
pub use crate::memory::{Memory32, Memory64, MemorySize};
pub use crate::module::{ExportsIterator, ImportKey, ImportsIterator, ModuleInfo};
pub use crate::units::{
    Bytes, PageCountOutOfRange, Pages, WASM_MAX_PAGES, WASM_MIN_PAGES, WASM_PAGE_SIZE,
};
pub use types::{
    ExportType, ExternType, FunctionType, GlobalInit, GlobalType, ImportType, MemoryType,
    Mutability, TableType, Type, V128,
};
pub use value::{RawValue, ValueType};

pub use crate::libcalls::LibCall;
pub use crate::memory::MemoryStyle;
pub use crate::table::TableStyle;
// TODO: OnCalledAction is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
pub use crate::trapcode::{OnCalledAction, TrapCode};
pub use crate::vmoffsets::{TargetSharedSignatureIndex, VMBuiltinFunctionIndex, VMOffsets};

pub use crate::utils::is_wasm;

pub use crate::compilation::relocation::{
    ArchivedRelocation, Relocation, RelocationKind, RelocationLike, RelocationTarget, Relocations,
};
pub use crate::compilation::section::{
    ArchivedCustomSection, CustomSection, CustomSectionLike, CustomSectionProtection, SectionBody,
    SectionIndex,
};

pub use crate::compilation::address_map::{FunctionAddressMap, InstructionAddressMap};
pub use crate::compilation::function::{
    ArchivedFunctionBody, Compilation, CompiledFunction, CompiledFunctionFrameInfo, CustomSections,
    Dwarf, FunctionBody, FunctionBodyLike, Functions,
};
pub use crate::compilation::module::CompileModuleInfo;
pub use crate::compilation::symbols::{Symbol, SymbolRegistry};
pub use crate::compilation::unwind::{
    ArchivedCompiledFunctionUnwindInfo, CompiledFunctionUnwindInfo, CompiledFunctionUnwindInfoLike,
    CompiledFunctionUnwindInfoReference,
};

pub use crate::stack::{FrameInfo, SourceLoc, TrapInformation};
pub use crate::store_id::StoreId;

/// Offset in bytes from the beginning of the function.
pub type CodeOffset = u32;

/// Addend to add to the symbol value.
pub type Addend = i64;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod native {
    use super::Type;
    use crate::memory::{Memory32, Memory64, MemorySize};
    use std::fmt;

    /// `NativeWasmType` represents a Wasm type that has a direct
    /// representation on the host (hence the “native” term).
    ///
    /// It uses the Rust Type system to automatically detect the
    /// Wasm type associated with a native Rust type.
    ///
    /// ```
    /// use wasmer_types::{NativeWasmType, Type};
    ///
    /// let wasm_type = i32::WASM_TYPE;
    /// assert_eq!(wasm_type, Type::I32);
    /// ```
    ///
    /// > Note: This strategy will be needed later to
    /// > automatically detect the signature of a Rust function.
    pub trait NativeWasmType: Sized {
        /// The ABI for this type (i32, i64, f32, f64)
        type Abi: Copy + fmt::Debug;

        /// Type for this `NativeWasmType`.
        const WASM_TYPE: Type;
    }

    impl NativeWasmType for u32 {
        const WASM_TYPE: Type = Type::I32;
        type Abi = Self;
    }

    impl NativeWasmType for i32 {
        const WASM_TYPE: Type = Type::I32;
        type Abi = Self;
    }

    impl NativeWasmType for i64 {
        const WASM_TYPE: Type = Type::I64;
        type Abi = Self;
    }

    impl NativeWasmType for u64 {
        const WASM_TYPE: Type = Type::I64;
        type Abi = Self;
    }

    impl NativeWasmType for f32 {
        const WASM_TYPE: Type = Type::F32;
        type Abi = Self;
    }

    impl NativeWasmType for f64 {
        const WASM_TYPE: Type = Type::F64;
        type Abi = Self;
    }

    impl NativeWasmType for u128 {
        const WASM_TYPE: Type = Type::V128;
        type Abi = Self;
    }

    impl NativeWasmType for Memory32 {
        const WASM_TYPE: Type = <<Self as MemorySize>::Native as NativeWasmType>::WASM_TYPE;
        type Abi = <<Self as MemorySize>::Native as NativeWasmType>::Abi;
    }

    impl NativeWasmType for Memory64 {
        const WASM_TYPE: Type = <<Self as MemorySize>::Native as NativeWasmType>::WASM_TYPE;
        type Abi = <<Self as MemorySize>::Native as NativeWasmType>::Abi;
    }

    impl<T: NativeWasmType> NativeWasmType for Option<T> {
        const WASM_TYPE: Type = T::WASM_TYPE;
        type Abi = T::Abi;
    }
}

pub use crate::native::*;
