// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Nibble-orientated view onto byte-slice, allowing nibble-precision offsets.

use std::{
    cmp::{min, Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt,
};

/// Nibble-orientated view onto byte-slice, allowing nibble-precision offsets.
///
/// This is an immutable struct. No operations actually change it.
///
/// # Example
/// ```snippet
/// use patricia_trie::nibbleslice::NibbleSlice;
/// {
///   let d1 = &[0x01u8, 0x23, 0x45];
///   let d2 = &[0x34u8, 0x50, 0x12];
///   let d3 = &[0x00u8, 0x12];
///   let n1 = NibbleSlice::new(d1);            // 0,1,2,3,4,5
///   let n2 = NibbleSlice::new(d2);            // 3,4,5,0,1,2
///   let n3 = NibbleSlice::new_offset(d3, 1);  // 0,1,2
///   assert!(n1 > n3);                         // 0,1,2,... > 0,1,2
///   assert!(n1 < n2);                         // 0,... < 3,...
///   assert!(n2.mid(3) == n3);                 // 0,1,2 == 0,1,2
///   assert!(n1.starts_with(&n3));
///   assert_eq!(n1.common_prefix(&n3), 3);
///   assert_eq!(n2.mid(3).common_prefix(&n1), 3);
/// }
/// ```
#[derive(Copy, Clone, Eq)]
pub struct NibbleSlice<'a> {
    data: &'a [u8],
    offset: usize,
}

/// Iterator type for a nibble slice.
pub struct NibbleSliceIterator<'a> {
    p: &'a NibbleSlice<'a>,
    i: usize,
}

impl Iterator for NibbleSliceIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        self.i += 1;
        if self.i <= self.p.len() {
            Some(self.p.at(self.i - 1))
        } else {
            None
        }
    }
}

impl<'a> NibbleSlice<'a> {
    /// Create a new nibble slice with the given byte-slice.
    pub fn new(data: &'a [u8]) -> Self {
        NibbleSlice::new_offset(data, 0)
    }

    /// Create a new nibble slice with the given byte-slice with a nibble offset.
    pub fn new_offset(data: &'a [u8], offset: usize) -> Self {
        NibbleSlice { data, offset }
    }

    /// Create a new nibble slice from the given HPE encoded data (e.g. output of `encoded()`).
    pub fn from_encoded(data: &'a [u8]) -> (Self, bool) {
        (
            Self::new_offset(data, if data[0] & 16 == 16 { 1 } else { 2 }),
            data[0] & 32 == 32,
        )
    }

    /// Is this an empty slice?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the length (in nibbles, naturally) of this slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len() * 2 - self.offset
    }

    /// Get the nibble at position `i`.
    #[inline]
    pub fn at(&self, i: usize) -> u8 {
        if (self.offset + i) & 1 == 1 {
            self.data[(self.offset + i) / 2] & 15u8
        } else {
            self.data[(self.offset + i) / 2] >> 4
        }
    }

    /// Return object which represents a view on to this slice (further) offset by `i` nibbles.
    pub fn mid(&self, i: usize) -> Self {
        NibbleSlice {
            data: self.data,
            offset: self.offset + i,
        }
    }

    /// Do we start with the same nibbles as the whole of `them`?
    pub fn starts_with(&self, them: &Self) -> bool {
        self.common_prefix(them) == them.len()
    }

    /// How many of the same nibbles at the beginning do we match with `them`?
    pub fn common_prefix(&self, them: &Self) -> usize {
        let s = min(self.len(), them.len());
        for i in 0..s {
            if self.at(i) != them.at(i) {
                return i;
            }
        }
        s
    }
}

impl PartialEq for NibbleSlice<'_> {
    fn eq(&self, them: &Self) -> bool {
        self.len() == them.len() && self.starts_with(them)
    }
}

impl Ord for NibbleSlice<'_> {
    fn cmp(&self, them: &Self) -> Ordering {
        let s = min(self.len(), them.len());
        for i in 0..s {
            match self.at(i).cmp(&them.at(i)) {
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
                Ordering::Equal => {}
            }
        }
        self.len().cmp(&them.len())
    }
}

impl PartialOrd for NibbleSlice<'_> {
    fn partial_cmp(&self, them: &Self) -> Option<Ordering> {
        Some(self.cmp(them))
    }
}

impl fmt::Debug for NibbleSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return Ok(());
        }
        write!(f, "{:01x}", self.at(0))?;
        for i in 1..self.len() {
            write!(f, "'{:01x}", self.at(i))?;
        }
        Ok(())
    }
}
