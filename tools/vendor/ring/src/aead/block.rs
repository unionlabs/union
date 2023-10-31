// Copyright 2018 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use crate::{endian::*, polyfill};

/// An array of 16 bytes that can (in the x86_64 and AAarch64 ABIs, at least)
/// be efficiently passed by value and returned by value (i.e. in registers),
/// and which meets the alignment requirements of `u32` and `u64` (at least)
/// for the target.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Block {
    subblocks: [u64; 2],
}

pub const BLOCK_LEN: usize = 16;

impl Block {
    #[inline]
    pub fn zero() -> Self {
        Self { subblocks: [0, 0] }
    }

    // TODO: Remove this.
    #[inline]
    pub fn from_u64_le(first: LittleEndian<u64>, second: LittleEndian<u64>) -> Self {
        #[allow(deprecated)]
        Self {
            subblocks: [first.into_raw_value(), second.into_raw_value()],
        }
    }

    // TODO: Remove this.
    #[inline]
    pub fn from_u64_be(first: BigEndian<u64>, second: BigEndian<u64>) -> Self {
        #[allow(deprecated)]
        Self {
            subblocks: [first.into_raw_value(), second.into_raw_value()],
        }
    }

    pub fn u64s_be_to_native(&self) -> [u64; 2] {
        [
            u64::from_be(self.subblocks[0]),
            u64::from_be(self.subblocks[1]),
        ]
    }

    #[inline]
    pub fn overwrite_part_at(&mut self, index: usize, a: &[u8]) {
        let mut tmp: [u8; BLOCK_LEN] = *self.as_ref();
        tmp[index..][..a.len()].copy_from_slice(a);
        *self = Self::from(&tmp)
    }

    #[inline]
    pub fn zero_from(&mut self, index: usize) {
        let mut tmp: [u8; BLOCK_LEN] = *self.as_ref();
        polyfill::slice::fill(&mut tmp[index..], 0);
        *self = Self::from(&tmp)
    }

    #[inline]
    pub fn bitxor_assign(&mut self, a: Block) {
        for (r, a) in self.subblocks.iter_mut().zip(a.subblocks.iter()) {
            *r ^= *a;
        }
    }
}

impl From<&'_ [u8; BLOCK_LEN]> for Block {
    #[inline]
    fn from(bytes: &[u8; BLOCK_LEN]) -> Self {
        unsafe { core::mem::transmute_copy(bytes) }
    }
}

impl AsRef<[u8; BLOCK_LEN]> for Block {
    #[allow(clippy::transmute_ptr_to_ptr)]
    #[inline]
    fn as_ref(&self) -> &[u8; BLOCK_LEN] {
        unsafe { core::mem::transmute(self) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitxor_assign() {
        const ONES: u64 = -1i64 as u64;
        const TEST_CASES: &[([u64; 2], [u64; 2], [u64; 2])] = &[
            ([0, 0], [0, 0], [0, 0]),
            ([0, 0], [ONES, ONES], [ONES, ONES]),
            ([0, ONES], [ONES, 0], [ONES, ONES]),
            ([ONES, 0], [0, ONES], [ONES, ONES]),
            ([ONES, ONES], [ONES, ONES], [0, 0]),
        ];
        for (expected_result, a, b) in TEST_CASES {
            let mut r = Block::from_u64_le(a[0].into(), a[1].into());
            r.bitxor_assign(Block::from_u64_le(b[0].into(), b[1].into()));
            assert_eq!(*expected_result, r.subblocks);

            // XOR is symmetric.
            let mut r = Block::from_u64_le(b[0].into(), b[1].into());
            r.bitxor_assign(Block::from_u64_le(a[0].into(), a[1].into()));
            assert_eq!(*expected_result, r.subblocks);
        }
    }
}
