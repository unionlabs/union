#![allow(missing_docs)]

mod array;
mod primitive;

use core::fmt::Debug;
use core::hash::Hash;
use core::ops::*;

/// A trait marking valid underlying bitset storage types and providing the
/// operations `EnumSet` and related types use.
///
/// # Safety
///
/// Note that `iter` *MUST* be implemented correctly and only return bits that
/// are actually set in the representation, or else it will cause undefined
/// behavior upstream in `EnumSet`.
pub trait EnumSetTypeRepr :
    // Basic traits used to derive traits
    Copy +
    Ord +
    Eq +
    Debug +
    Hash +
    // Operations used by enumset
    BitAnd<Output = Self> +
    BitOr<Output = Self> +
    BitXor<Output = Self> +
    Not<Output = Self> +
{
    const PREFERRED_ARRAY_LEN: usize;
    const WIDTH: u32;
    const EMPTY: Self;

    fn is_empty(&self) -> bool;

    fn add_bit(&mut self, bit: u32);
    fn remove_bit(&mut self, bit: u32);
    fn has_bit(&self, bit: u32) -> bool;

    fn count_ones(&self) -> u32;
    fn leading_zeros(&self) -> u32;
    fn trailing_zeros(&self) -> u32;

    fn and_not(&self, other: Self) -> Self;

    type Iter: Iterator<Item = u32> + DoubleEndedIterator + Clone + Debug;
    fn iter(self) -> Self::Iter;

    fn from_u8(v: u8) -> Self;
    fn from_u16(v: u16) -> Self;
    fn from_u32(v: u32) -> Self;
    fn from_u64(v: u64) -> Self;
    fn from_u128(v: u128) -> Self;
    fn from_usize(v: usize) -> Self;

    fn to_u8(&self) -> u8;
    fn to_u16(&self) -> u16;
    fn to_u32(&self) -> u32;
    fn to_u64(&self) -> u64;
    fn to_u128(&self) -> u128;
    fn to_usize(&self) -> usize;

    fn from_u8_opt(v: u8) -> Option<Self>;
    fn from_u16_opt(v: u16) -> Option<Self>;
    fn from_u32_opt(v: u32) -> Option<Self>;
    fn from_u64_opt(v: u64) -> Option<Self>;
    fn from_u128_opt(v: u128) -> Option<Self>;
    fn from_usize_opt(v: usize) -> Option<Self>;

    fn to_u8_opt(&self) -> Option<u8>;
    fn to_u16_opt(&self) -> Option<u16>;
    fn to_u32_opt(&self) -> Option<u32>;
    fn to_u64_opt(&self) -> Option<u64>;
    fn to_u128_opt(&self) -> Option<u128>;
    fn to_usize_opt(&self) -> Option<usize>;

    fn to_u64_array<const O: usize>(&self) -> [u64; O];
    fn to_u64_array_opt<const O: usize>(&self) -> Option<[u64; O]>;

    fn from_u64_array<const O: usize>(v: [u64; O]) -> Self;
    fn from_u64_array_opt<const O: usize>(v: [u64; O]) -> Option<Self>;

    fn to_u64_slice(&self, out: &mut [u64]);
    #[must_use]
    fn to_u64_slice_opt(&self, out: &mut [u64]) -> Option<()>;

    fn from_u64_slice(v: &[u64]) -> Self;
    fn from_u64_slice_opt(v: &[u64]) -> Option<Self>;
}

pub use array::ArrayRepr;
