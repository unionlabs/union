//! Helper functions and structures for the translation.
use crate::entity::entity_impl;
use core::u32;
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Index type of a function defined locally inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct LocalFunctionIndex(u32);
entity_impl!(LocalFunctionIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedLocalFunctionIndex);

/// Index type of a table defined locally inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct LocalTableIndex(u32);
entity_impl!(LocalTableIndex);

/// Index type of a memory defined locally inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct LocalMemoryIndex(u32);
entity_impl!(LocalMemoryIndex);

/// Index type of a global defined locally inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct LocalGlobalIndex(u32);
entity_impl!(LocalGlobalIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedLocalGlobalIndex);

/// Index type of a function (imported or local) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct FunctionIndex(u32);
entity_impl!(FunctionIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedFunctionIndex);

/// Index type of a table (imported or local) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct TableIndex(u32);
entity_impl!(TableIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedTableIndex);

/// Index type of a global variable (imported or local) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct GlobalIndex(u32);
entity_impl!(GlobalIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedGlobalIndex);

/// Index type of a linear memory (imported or local) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct MemoryIndex(u32);
entity_impl!(MemoryIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedMemoryIndex);

/// Index type of a signature (imported or local) inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct SignatureIndex(u32);
entity_impl!(SignatureIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedSignatureIndex);

/// Index type of a passive data segment inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct DataIndex(u32);
entity_impl!(DataIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedDataIndex);

/// Index type of a passive element segment inside the WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct ElemIndex(u32);
entity_impl!(ElemIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedElemIndex);

/// Index type of a custom section inside a WebAssembly module.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub struct CustomSectionIndex(u32);
entity_impl!(CustomSectionIndex);
#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedCustomSectionIndex);

/// An entity to export.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub enum ExportIndex {
    /// Function export.
    Function(FunctionIndex),
    /// Table export.
    Table(TableIndex),
    /// Memory export.
    Memory(MemoryIndex),
    /// Global export.
    Global(GlobalIndex),
}

/// An entity to import.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, MemoryUsage)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
pub enum ImportIndex {
    /// Function import.
    Function(FunctionIndex),
    /// Table import.
    Table(TableIndex),
    /// Memory import.
    Memory(MemoryIndex),
    /// Global import.
    Global(GlobalIndex),
}
