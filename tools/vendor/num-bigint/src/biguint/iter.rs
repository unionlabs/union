use core::iter::FusedIterator;

#[cfg(not(u64_digit))]
use super::u32_chunk_to_u64;

/// An iterator of `u32` digits representation of a `BigUint` or `BigInt`,
/// ordered least significant digit first.
pub struct U32Digits<'a> {
    #[cfg(u64_digit)]
    data: &'a [u64],
    #[cfg(u64_digit)]
    next_is_lo: bool,
    #[cfg(u64_digit)]
    last_hi_is_zero: bool,

    #[cfg(not(u64_digit))]
    it: core::slice::Iter<'a, u32>,
}

#[cfg(u64_digit)]
impl<'a> U32Digits<'a> {
    #[inline]
    pub(super) fn new(data: &'a [u64]) -> Self {
        let last_hi_is_zero = data
            .last()
            .map(|&last| {
                let last_hi = (last >> 32) as u32;
                last_hi == 0
            })
            .unwrap_or(false);
        U32Digits {
            data,
            next_is_lo: true,
            last_hi_is_zero,
        }
    }
}

#[cfg(u64_digit)]
impl Iterator for U32Digits<'_> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        match self.data.split_first() {
            Some((&first, data)) => {
                let next_is_lo = self.next_is_lo;
                self.next_is_lo = !next_is_lo;
                if next_is_lo {
                    Some(first as u32)
                } else {
                    self.data = data;
                    if data.is_empty() && self.last_hi_is_zero {
                        self.last_hi_is_zero = false;
                        None
                    } else {
                        Some((first >> 32) as u32)
                    }
                }
            }
            None => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn last(self) -> Option<u32> {
        self.data.last().map(|&last| {
            if self.last_hi_is_zero {
                last as u32
            } else {
                (last >> 32) as u32
            }
        })
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

#[cfg(u64_digit)]
impl DoubleEndedIterator for U32Digits<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.data.split_last() {
            Some((&last, data)) => {
                let last_is_lo = self.last_hi_is_zero;
                self.last_hi_is_zero = !last_is_lo;
                if last_is_lo {
                    self.data = data;
                    if data.is_empty() && !self.next_is_lo {
                        self.next_is_lo = true;
                        None
                    } else {
                        Some(last as u32)
                    }
                } else {
                    Some((last >> 32) as u32)
                }
            }
            None => None,
        }
    }
}

#[cfg(u64_digit)]
impl ExactSizeIterator for U32Digits<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.data.len() * 2 - usize::from(self.last_hi_is_zero) - usize::from(!self.next_is_lo)
    }
}

#[cfg(not(u64_digit))]
impl<'a> U32Digits<'a> {
    #[inline]
    pub(super) fn new(data: &'a [u32]) -> Self {
        Self { it: data.iter() }
    }
}

#[cfg(not(u64_digit))]
impl Iterator for U32Digits<'_> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<u32> {
        self.it.next().cloned()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<u32> {
        self.it.nth(n).cloned()
    }

    #[inline]
    fn last(self) -> Option<u32> {
        self.it.last().cloned()
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count()
    }
}

#[cfg(not(u64_digit))]
impl DoubleEndedIterator for U32Digits<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.it.next_back().copied()
    }
}

#[cfg(not(u64_digit))]
impl ExactSizeIterator for U32Digits<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.it.len()
    }
}

impl FusedIterator for U32Digits<'_> {}

/// An iterator of `u64` digits representation of a `BigUint` or `BigInt`,
/// ordered least significant digit first.
pub struct U64Digits<'a> {
    #[cfg(not(u64_digit))]
    it: core::slice::Chunks<'a, u32>,

    #[cfg(u64_digit)]
    it: core::slice::Iter<'a, u64>,
}

#[cfg(not(u64_digit))]
impl<'a> U64Digits<'a> {
    #[inline]
    pub(super) fn new(data: &'a [u32]) -> Self {
        U64Digits { it: data.chunks(2) }
    }
}

#[cfg(not(u64_digit))]
impl Iterator for U64Digits<'_> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        self.it.next().map(u32_chunk_to_u64)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn last(self) -> Option<u64> {
        self.it.last().map(u32_chunk_to_u64)
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

#[cfg(not(u64_digit))]
impl DoubleEndedIterator for U64Digits<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.it.next_back().map(u32_chunk_to_u64)
    }
}

#[cfg(u64_digit)]
impl<'a> U64Digits<'a> {
    #[inline]
    pub(super) fn new(data: &'a [u64]) -> Self {
        Self { it: data.iter() }
    }
}

#[cfg(u64_digit)]
impl Iterator for U64Digits<'_> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        self.it.next().cloned()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.it.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<u64> {
        self.it.nth(n).cloned()
    }

    #[inline]
    fn last(self) -> Option<u64> {
        self.it.last().cloned()
    }

    #[inline]
    fn count(self) -> usize {
        self.it.count()
    }
}

#[cfg(u64_digit)]
impl DoubleEndedIterator for U64Digits<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.it.next_back().cloned()
    }
}

impl ExactSizeIterator for U64Digits<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.it.len()
    }
}

impl FusedIterator for U64Digits<'_> {}

#[test]
fn test_iter_u32_digits() {
    let n = super::BigUint::from(5u8);
    let mut it = n.iter_u32_digits();
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);

    let n = super::BigUint::from(112500000000u64);
    let mut it = n.iter_u32_digits();
    assert_eq!(it.len(), 2);
    assert_eq!(it.next(), Some(830850304));
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(26));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
}

#[test]
fn test_iter_u64_digits() {
    let n = super::BigUint::from(5u8);
    let mut it = n.iter_u64_digits();
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);

    let n = super::BigUint::from(18_446_744_073_709_551_616u128);
    let mut it = n.iter_u64_digits();
    assert_eq!(it.len(), 2);
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
}

#[test]
fn test_iter_u32_digits_be() {
    let n = super::BigUint::from(5u8);
    let mut it = n.iter_u32_digits();
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(5));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);

    let n = super::BigUint::from(112500000000u64);
    let mut it = n.iter_u32_digits();
    assert_eq!(it.len(), 2);
    assert_eq!(it.next(), Some(830850304));
    assert_eq!(it.len(), 1);
    assert_eq!(it.next(), Some(26));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
}

#[test]
fn test_iter_u64_digits_be() {
    let n = super::BigUint::from(5u8);
    let mut it = n.iter_u64_digits();
    assert_eq!(it.len(), 1);
    assert_eq!(it.next_back(), Some(5));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);

    let n = super::BigUint::from(18_446_744_073_709_551_616u128);
    let mut it = n.iter_u64_digits();
    assert_eq!(it.len(), 2);
    assert_eq!(it.next_back(), Some(1));
    assert_eq!(it.len(), 1);
    assert_eq!(it.next_back(), Some(0));
    assert_eq!(it.len(), 0);
    assert_eq!(it.next(), None);
}
