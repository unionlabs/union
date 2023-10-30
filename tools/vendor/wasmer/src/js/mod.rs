#[cfg(all(feature = "std", feature = "core"))]
compile_error!(
    "The `std` and `core` features are both enabled, which is an error. Please enable only once."
);

#[cfg(all(not(feature = "std"), not(feature = "core")))]
compile_error!("Both the `std` and `core` features are disabled. Please enable one of them.");

#[cfg(feature = "core")]
extern crate alloc;

mod lib {
    #[cfg(feature = "core")]
    pub mod std {
        pub use alloc::{borrow, boxed, str, string, sync, vec};
        pub use core::fmt;
        pub use hashbrown as collections;
    }

    #[cfg(feature = "std")]
    pub mod std {
        pub use std::{borrow, boxed, collections, fmt, str, string, sync, vec};
    }
}

mod cell;
mod env;
mod error;
mod export;
mod exports;
mod externals;
mod import_object;
mod instance;
mod js_import_object;
mod module;
#[cfg(feature = "wasm-types-polyfill")]
mod module_info_polyfill;
mod native;
mod ptr;
mod resolver;
mod store;
mod trap;
mod types;
mod wasm_bindgen_polyfill;

/// Implement [`WasmerEnv`] for your type with `#[derive(WasmerEnv)]`.
///
/// See the [`WasmerEnv`] trait for more information.
pub use wasmer_derive::WasmerEnv;

pub use crate::js::cell::WasmCell;
pub use crate::js::env::{HostEnvInitError, LazyInit, WasmerEnv};
pub use crate::js::error::{DeserializeError, SerializeError};
pub use crate::js::export::Export;
pub use crate::js::exports::{ExportError, Exportable, Exports, ExportsIterator};
pub use crate::js::externals::{
    Extern, FromToNativeWasmType, Function, Global, HostFunction, Memory, MemoryError, Table,
    WasmTypeList,
};
pub use crate::js::import_object::{ImportObject, ImportObjectIterator, LikeNamespace};
pub use crate::js::instance::{Instance, InstantiationError};
pub use crate::js::js_import_object::JsImportObject;
pub use crate::js::module::{Module, ModuleTypeHints};
pub use crate::js::native::NativeFunc;
pub use crate::js::ptr::{Array, Item, WasmPtr};
pub use crate::js::resolver::{
    ChainableNamedResolver, NamedResolver, NamedResolverChain, Resolver,
};
pub use crate::js::trap::RuntimeError;

pub use crate::js::store::{Store, StoreObject};
pub use crate::js::types::{
    ExportType, ExternType, FunctionType, GlobalType, ImportType, MemoryType, Mutability,
    TableType, Val, ValType,
};
pub use crate::js::types::{Val as Value, ValType as Type};

pub use wasmer_types::is_wasm;
pub use wasmer_types::{
    Atomically, Bytes, ExportIndex, GlobalInit, LocalFunctionIndex, MemoryView, Pages, ValueType,
    WASM_MAX_PAGES, WASM_MIN_PAGES, WASM_PAGE_SIZE,
};

#[cfg(feature = "wat")]
pub use wat::parse_bytes as wat2wasm;

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
