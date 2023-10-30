//! This module define the required structures to emit custom
//! Sections in a `Compilation`.
//!
//! The functions that access a custom [`CustomSection`] would need
//! to emit a custom relocation: `RelocationTarget::CustomSection`, so
//! it can be patched later by the engine (native or JIT).

use crate::lib::std::vec::Vec;
use crate::Relocation;
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use wasmer_types::entity::entity_impl;

/// Index type of a Section defined inside a WebAssembly `Compilation`.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(
    feature = "enable-rkyv",
    archive_attr(derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug))
)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, MemoryUsage)]
pub struct SectionIndex(u32);

entity_impl!(SectionIndex);

#[cfg(feature = "enable-rkyv")]
entity_impl!(ArchivedSectionIndex);

/// Custom section Protection.
///
/// Determines how a custom section may be used.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, MemoryUsage)]
pub enum CustomSectionProtection {
    /// A custom section with read permission.
    Read,

    /// A custom section with read and execute permissions.
    ReadExecute,
}

/// A Section for a `Compilation`.
///
/// This is used so compilers can store arbitrary information
/// in the emitted module.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, MemoryUsage)]
pub struct CustomSection {
    /// Memory protection that applies to this section.
    pub protection: CustomSectionProtection,

    /// The bytes corresponding to this section.
    ///
    /// > Note: These bytes have to be at-least 8-byte aligned
    /// > (the start of the memory pointer).
    /// > We might need to create another field for alignment in case it's
    /// > needed in the future.
    pub bytes: SectionBody,

    /// Relocations that apply to this custom section.
    pub relocations: Vec<Relocation>,
}

/// The bytes in the section.
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, Default, MemoryUsage)]
pub struct SectionBody(#[cfg_attr(feature = "enable-serde", serde(with = "serde_bytes"))] Vec<u8>);

impl SectionBody {
    /// Create a new section body with the given contents.
    pub fn new_with_vec(contents: Vec<u8>) -> Self {
        Self(contents)
    }

    /// Returns a raw pointer to the section's buffer.
    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Returns the length of this section in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Dereferences into the section's buffer.
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns whether or not the section body is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
