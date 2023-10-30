//! Heaps.

use crate::ir::immediates::Uimm64;
use crate::ir::{GlobalValue, Type};
use core::fmt;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Information about a heap declaration.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct HeapData {
    /// The address of the start of the heap's storage.
    pub base: GlobalValue,

    /// Guaranteed minimum heap size in bytes. Heap accesses before `min_size` don't need bounds
    /// checking.
    pub min_size: Uimm64,

    /// Size in bytes of the offset-guard pages following the heap.
    pub offset_guard_size: Uimm64,

    /// Heap style, with additional style-specific info.
    pub style: HeapStyle,

    /// The index type for the heap.
    pub index_type: Type,
}

/// Style of heap including style-specific information.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum HeapStyle {
    /// A dynamic heap can be relocated to a different base address when it is grown.
    Dynamic {
        /// Global value providing the current bound of the heap in bytes.
        bound_gv: GlobalValue,
    },

    /// A static heap has a fixed base address and a number of not-yet-allocated pages before the
    /// offset-guard pages.
    Static {
        /// Heap bound in bytes. The offset-guard pages are allocated after the bound.
        bound: Uimm64,
    },
}

impl fmt::Display for HeapData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self.style {
            HeapStyle::Dynamic { .. } => "dynamic",
            HeapStyle::Static { .. } => "static",
        })?;

        write!(f, " {}, min {}", self.base, self.min_size)?;
        match self.style {
            HeapStyle::Dynamic { bound_gv } => write!(f, ", bound {}", bound_gv)?,
            HeapStyle::Static { bound } => write!(f, ", bound {}", bound)?,
        }
        write!(
            f,
            ", offset_guard {}, index_type {}",
            self.offset_guard_size, self.index_type
        )
    }
}
