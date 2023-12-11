use enum_iterator::IntoEnumIterator;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// The name of a runtime library routine.
///
/// This list is likely to grow over time.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    IntoEnumIterator,
    RkyvSerialize,
    RkyvDeserialize,
    Archive,
    rkyv::CheckBytes,
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[archive(as = "Self")]
#[repr(u16)]
pub enum LibCall {
    /// ceil.f32
    CeilF32,

    /// ceil.f64
    CeilF64,

    /// floor.f32
    FloorF32,

    /// floor.f64
    FloorF64,

    /// nearest.f32
    NearestF32,

    /// nearest.f64
    NearestF64,

    /// trunc.f32
    TruncF32,

    /// trunc.f64
    TruncF64,

    /// memory.size for local functions
    Memory32Size,

    /// memory.size for imported functions
    ImportedMemory32Size,

    /// table.copy
    TableCopy,

    /// table.init
    TableInit,

    /// table.fill
    TableFill,

    /// table.size for local tables
    TableSize,

    /// table.size for imported tables
    ImportedTableSize,

    /// table.get for local tables
    TableGet,

    /// table.get for imported tables
    ImportedTableGet,

    /// table.set for local tables
    TableSet,

    /// table.set for imported tables
    ImportedTableSet,

    /// table.grow for local tables
    TableGrow,

    /// table.grow for imported tables
    ImportedTableGrow,

    /// ref.func
    FuncRef,

    /// elem.drop
    ElemDrop,

    /// memory.copy for local memories
    Memory32Copy,

    /// memory.copy for imported memories
    ImportedMemory32Copy,

    /// memory.fill for local memories
    Memory32Fill,

    /// memory.fill for imported memories
    ImportedMemory32Fill,

    /// memory.init
    Memory32Init,

    /// data.drop
    DataDrop,

    /// A custom trap
    RaiseTrap,

    /// probe for stack overflow. These are emitted for functions which need
    /// when the `enable_probestack` setting is true.
    Probestack,

    /// memory.atomic.wait32 for local memories
    Memory32AtomicWait32,

    /// memory.atomic.wait32 for imported memories
    ImportedMemory32AtomicWait32,

    /// memory.atomic.wait64 for local memories
    Memory32AtomicWait64,

    /// memory.atomic.wait64 for imported memories
    ImportedMemory32AtomicWait64,

    /// memory.atomic.notify for local memories
    Memory32AtomicNotify,

    /// memory.atomic.botify for imported memories
    ImportedMemory32AtomicNotify,
}

impl LibCall {
    /// Return the function name associated to the libcall.
    pub fn to_function_name(&self) -> &str {
        match self {
            Self::CeilF32 => "wasmer_vm_f32_ceil",
            Self::CeilF64 => "wasmer_vm_f64_ceil",
            Self::FloorF32 => "wasmer_vm_f32_floor",
            Self::FloorF64 => "wasmer_vm_f64_floor",
            Self::NearestF32 => "wasmer_vm_f32_nearest",
            Self::NearestF64 => "wasmer_vm_f64_nearest",
            Self::TruncF32 => "wasmer_vm_f32_trunc",
            Self::TruncF64 => "wasmer_vm_f64_trunc",
            Self::Memory32Size => "wasmer_vm_memory32_size",
            Self::ImportedMemory32Size => "wasmer_vm_imported_memory32_size",
            Self::TableCopy => "wasmer_vm_table_copy",
            Self::TableInit => "wasmer_vm_table_init",
            Self::TableFill => "wasmer_vm_table_fill",
            Self::TableSize => "wasmer_vm_table_size",
            Self::ImportedTableSize => "wasmer_vm_imported_table_size",
            Self::TableGet => "wasmer_vm_table_get",
            Self::ImportedTableGet => "wasmer_vm_imported_table_get",
            Self::TableSet => "wasmer_vm_table_set",
            Self::ImportedTableSet => "wasmer_vm_imported_table_set",
            Self::TableGrow => "wasmer_vm_table_grow",
            Self::ImportedTableGrow => "wasmer_vm_imported_table_grow",
            Self::FuncRef => "wasmer_vm_func_ref",
            Self::ElemDrop => "wasmer_vm_elem_drop",
            Self::Memory32Copy => "wasmer_vm_memory32_copy",
            Self::ImportedMemory32Copy => "wasmer_vm_imported_memory32_copy",
            Self::Memory32Fill => "wasmer_vm_memory32_fill",
            Self::ImportedMemory32Fill => "wasmer_vm_imported_memory32_fill",
            Self::Memory32Init => "wasmer_vm_memory32_init",
            Self::DataDrop => "wasmer_vm_data_drop",
            Self::RaiseTrap => "wasmer_vm_raise_trap",
            // We have to do this because macOS requires a leading `_` and it's not
            // a normal function, it's a static variable, so we have to do it manually.
            #[cfg(target_vendor = "apple")]
            Self::Probestack => "_wasmer_vm_probestack",
            #[cfg(not(target_vendor = "apple"))]
            Self::Probestack => "wasmer_vm_probestack",
            Self::Memory32AtomicWait32 => "wasmer_vm_memory32_atomic_wait32",
            Self::ImportedMemory32AtomicWait32 => "wasmer_vm_imported_memory32_atomic_wait32",
            Self::Memory32AtomicWait64 => "wasmer_vm_memory32_atomic_wait64",
            Self::ImportedMemory32AtomicWait64 => "wasmer_vm_imported_memory32_atomic_wait64",
            Self::Memory32AtomicNotify => "wasmer_vm_memory32_atomic_notify",
            Self::ImportedMemory32AtomicNotify => "wasmer_vm_imported_memory32_atomic_notify",
        }
    }
}

impl fmt::Display for LibCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
