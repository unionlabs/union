//! Runtime library support for Wasmer.

#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates)]
#![deny(trivial_numeric_casts, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(
    feature = "cargo-clippy",
    allow(clippy::new_without_default, vtable_address_comparisons)
)]
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

mod export;
mod func_data_registry;
mod global;
mod imports;
mod instance;
mod memory;
mod mmap;
mod probestack;
mod sig_registry;
mod table;
mod trap;
mod vmcontext;

pub mod libcalls;

pub use crate::export::*;
pub use crate::func_data_registry::{FuncDataRegistry, VMFuncRef};
pub use crate::global::*;
pub use crate::imports::Imports;
pub use crate::instance::{
    ImportFunctionEnv, ImportInitializerFuncPtr, InstanceAllocator, InstanceHandle,
    WeakOrStrongInstanceRef,
};
pub use crate::memory::{LinearMemory, Memory, MemoryError};
pub use crate::mmap::Mmap;
pub use crate::probestack::PROBESTACK;
pub use crate::sig_registry::SignatureRegistry;
pub use crate::table::{LinearTable, Table, TableElement};
pub use crate::trap::*;
pub use crate::vmcontext::{
    VMCallerCheckedAnyfunc, VMContext, VMDynamicFunctionContext, VMFunctionEnvironment,
    VMFunctionImport, VMFunctionKind, VMGlobalDefinition, VMGlobalImport, VMMemoryDefinition,
    VMMemoryImport, VMSharedSignatureIndex, VMTableDefinition, VMTableImport, VMTrampoline,
};
pub use wasmer_artifact::{FunctionBodyPtr, VMFunctionBody};
pub use wasmer_types::LibCall;
pub use wasmer_types::MemoryStyle;
pub use wasmer_types::TableStyle;
#[deprecated(
    since = "2.1.0",
    note = "ModuleInfo, ExportsIterator, ImportsIterator should be imported from wasmer_types."
)]
pub use wasmer_types::{ExportsIterator, ImportsIterator, ModuleInfo};
pub use wasmer_types::{
    TargetSharedSignatureIndex, VMBuiltinFunctionIndex, VMExternRef, VMOffsets,
};

/// Version number of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Pointers to section data.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct SectionBodyPtr(pub *const u8);

impl std::ops::Deref for SectionBodyPtr {
    type Target = *const u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
