//! Small Bitset
//!
//! This module defines a struct `BitSet<T>` encapsulating a bitset built over the type T.
//! T is intended to be a primitive unsigned type. Currently it can be any type between u8 and u32
//!
//! If you would like to add support for larger bitsets in the future, you need to change the trait
//! bound Into<u32> and the u32 in the implementation of `max_bits()`.

use core::convert::{From, Into};
use core::mem::size_of;
use core::ops::{Add, BitOr, Shl, Sub};

/// A small bitset built on a single primitive integer type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BitSet<T>(pub T);

impl<T> BitSet<T>
where
    T: Into<u32>
        + From<u8>
        + BitOr<T, Output = T>
        + Shl<u8, Output = T>
        + Sub<T, Output = T>
        + Add<T, Output = T>
        + PartialEq
        + Copy,
{
    /// Maximum number of bits supported by this BitSet instance
    pub fn bits() -> usize {
        size_of::<T>() * 8
    }

    /// Maximum number of bits supported by any bitset instance atm.
    pub fn max_bits() -> usize {
        size_of::<u32>() * 8
    }

    /// Check if this BitSet contains the number num
    pub fn contains(&self, num: u8) -> bool {
        debug_assert!((num as usize) < Self::bits());
        debug_assert!((num as usize) < Self::max_bits());
        self.0.into() & (1 << num) != 0
    }

    /// Return the smallest number contained in the bitset or None if empty
    pub fn min(&self) -> Option<u8> {
        if self.0.into() == 0 {
            None
        } else {
            Some(self.0.into().trailing_zeros() as u8)
        }
    }

    /// Return the largest number contained in the bitset or None if empty
    pub fn max(&self) -> Option<u8> {
        if self.0.into() == 0 {
            None
        } else {
            let leading_zeroes = self.0.into().leading_zeros() as usize;
            Some((Self::max_bits() - leading_zeroes - 1) as u8)
        }
    }

    /// Construct a BitSet with the half-open range [lo,hi) filled in
    pub fn from_range(lo: u8, hi: u8) -> Self {
        debug_assert!(lo <= hi);
        debug_assert!((hi as usize) <= Self::bits());
        let one: T = T::from(1);
        // I can't just do (one << hi) - one here as the shift may overflow
        let hi_rng = if hi >= 1 {
            (one << (hi - 1)) + ((one << (hi - 1)) - one)
        } else {
            T::from(0)
        };

        let lo_rng = (one << lo) - one;

        Self(hi_rng - lo_rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains() {
        let s = BitSet::<u8>(255);
        for i in 0..7 {
            assert!(s.contains(i));
        }

        let s1 = BitSet::<u8>(0);
        for i in 0..7 {
            assert!(!s1.contains(i));
        }

        let s2 = BitSet::<u8>(127);
        for i in 0..6 {
            assert!(s2.contains(i));
        }
        assert!(!s2.contains(7));

        let s3 = BitSet::<u8>(2 | 4 | 64);
        assert!(!s3.contains(0) && !s3.contains(3) && !s3.contains(4));
        assert!(!s3.contains(5) && !s3.contains(7));
        assert!(s3.contains(1) && s3.contains(2) && s3.contains(6));

        let s4 = BitSet::<u16>(4 | 8 | 256 | 1024);
        assert!(
            !s4.contains(0)
                && !s4.contains(1)
                && !s4.contains(4)
                && !s4.contains(5)
                && !s4.contains(6)
                && !s4.contains(7)
                && !s4.contains(9)
                && !s4.contains(11)
        );
        assert!(s4.contains(2) && s4.contains(3) && s4.contains(8) && s4.contains(10));
    }

    #[test]
    fn minmax() {
        let s = BitSet::<u8>(255);
        assert_eq!(s.min(), Some(0));
        assert_eq!(s.max(), Some(7));
        assert!(s.min() == Some(0) && s.max() == Some(7));
        let s1 = BitSet::<u8>(0);
        assert!(s1.min() == None && s1.max() == None);
        let s2 = BitSet::<u8>(127);
        assert!(s2.min() == Some(0) && s2.max() == Some(6));
        let s3 = BitSet::<u8>(2 | 4 | 64);
        assert!(s3.min() == Some(1) && s3.max() == Some(6));
        let s4 = BitSet::<u16>(4 | 8 | 256 | 1024);
        assert!(s4.min() == Some(2) && s4.max() == Some(10));
    }

    #[test]
    fn from_range() {
        let s = BitSet::<u8>::from_range(5, 5);
        assert!(s.0 == 0);

        let s = BitSet::<u8>::from_range(0, 8);
        assert!(s.0 == 255);

        let s = BitSet::<u16>::from_range(0, 8);
        assert!(s.0 == 255u16);

        let s = BitSet::<u16>::from_range(0, 16);
        assert!(s.0 == 65535u16);

        let s = BitSet::<u8>::from_range(5, 6);
        assert!(s.0 == 32u8);

        let s = BitSet::<u8>::from_range(3, 7);
        assert!(s.0 == 8 | 16 | 32 | 64);

        let s = BitSet::<u16>::from_range(5, 11);
        assert!(s.0 == 32 | 64 | 128 | 256 | 512 | 1024);
    }
}
