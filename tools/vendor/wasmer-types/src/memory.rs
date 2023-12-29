use crate::{Pages, ValueType};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::iter::Sum;
use std::ops::{Add, AddAssign};

/// Implementation styles for WebAssembly linear memory.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    RkyvSerialize,
    RkyvDeserialize,
    Archive,
    rkyv::CheckBytes,
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[archive(as = "Self")]
#[repr(u8)]
pub enum MemoryStyle {
    /// The actual memory can be resized and moved.
    Dynamic {
        /// Our chosen offset-guard size.
        ///
        /// It represents the size in bytes of extra guard pages after the end
        /// to optimize loads and stores with constant offsets.
        offset_guard_size: u64,
    },
    /// Address space is allocated up front.
    Static {
        /// The number of mapped and unmapped pages.
        bound: Pages,
        /// Our chosen offset-guard size.
        ///
        /// It represents the size in bytes of extra guard pages after the end
        /// to optimize loads and stores with constant offsets.
        offset_guard_size: u64,
    },
}

impl MemoryStyle {
    /// Returns the offset-guard size
    pub fn offset_guard_size(&self) -> u64 {
        match self {
            Self::Dynamic { offset_guard_size } => *offset_guard_size,
            Self::Static {
                offset_guard_size, ..
            } => *offset_guard_size,
        }
    }
}

/// Trait for the `Memory32` and `Memory64` marker types.
///
/// This allows code to be generic over 32-bit and 64-bit memories.
/// # Safety
/// Direct memory access is unsafe
pub unsafe trait MemorySize: Copy {
    /// Type used to represent an offset into a memory. This is `u32` or `u64`.
    type Offset: Default
        + std::fmt::Debug
        + std::fmt::Display
        + Eq
        + Ord
        + PartialEq<Self::Offset>
        + PartialOrd<Self::Offset>
        + Clone
        + Copy
        + Sync
        + Send
        + ValueType
        + Into<u64>
        + From<u32>
        + From<u16>
        + From<u8>
        + TryFrom<u64>
        + TryFrom<u32>
        + TryFrom<u16>
        + TryFrom<u8>
        + TryFrom<i32>
        + TryInto<usize>
        + TryInto<u64>
        + TryInto<u32>
        + TryInto<u16>
        + TryInto<u8>
        + TryInto<i32>
        + TryFrom<usize>
        + Add<Self::Offset>
        + Sum<Self::Offset>
        + AddAssign<Self::Offset>
        + 'static;

    /// Type used to pass this value as an argument or return value for a Wasm function.
    type Native: super::NativeWasmType;

    /// Zero value used for `WasmPtr::is_null`.
    const ZERO: Self::Offset;

    /// One value used for counting.
    const ONE: Self::Offset;

    /// Convert an `Offset` to a `Native`.
    fn offset_to_native(offset: Self::Offset) -> Self::Native;

    /// Convert a `Native` to an `Offset`.
    fn native_to_offset(native: Self::Native) -> Self::Offset;

    /// True if the memory is 64-bit
    fn is_64bit() -> bool;
}

/// Marker trait for 32-bit memories.
#[derive(Clone, Copy)]
pub struct Memory32;
unsafe impl MemorySize for Memory32 {
    type Offset = u32;
    type Native = i32;
    const ZERO: Self::Offset = 0;
    const ONE: Self::Offset = 1;
    fn offset_to_native(offset: Self::Offset) -> Self::Native {
        offset as Self::Native
    }
    fn native_to_offset(native: Self::Native) -> Self::Offset {
        native as Self::Offset
    }
    fn is_64bit() -> bool {
        false
    }
}

/// Marker trait for 64-bit memories.
#[derive(Clone, Copy)]
pub struct Memory64;
unsafe impl MemorySize for Memory64 {
    type Offset = u64;
    type Native = i64;
    const ZERO: Self::Offset = 0;
    const ONE: Self::Offset = 1;
    fn offset_to_native(offset: Self::Offset) -> Self::Native {
        offset as Self::Native
    }
    fn native_to_offset(native: Self::Native) -> Self::Offset {
        native as Self::Offset
    }
    fn is_64bit() -> bool {
        true
    }
}
