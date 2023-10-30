//! Argon2 memory block functions

use core::{
    num::Wrapping,
    ops::{BitXor, BitXorAssign, Index, IndexMut},
    slice,
};

#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// Structure for the (1KB) memory block implemented as 128 64-bit words.
#[derive(Copy, Clone, Debug)]
pub struct Block([u64; Self::SIZE / 8]);

impl Default for Block {
    fn default() -> Self {
        Self([0u64; Self::SIZE / 8])
    }
}

impl Block {
    /// Memory block size in bytes
    pub const SIZE: usize = 1024;

    /// Load a block from a block-sized byte slice
    pub(crate) fn load(&mut self, input: &[u8]) {
        debug_assert_eq!(input.len(), Block::SIZE);

        for (i, chunk) in input.chunks(8).enumerate() {
            self[i] = u64::from_le_bytes(chunk.try_into().unwrap());
        }
    }

    /// Iterate over the `u64` values contained in this block
    pub(crate) fn iter(&self) -> slice::Iter<'_, u64> {
        self.0.iter()
    }

    /// Iterate mutably over the `u64` values contained in this block
    pub(crate) fn iter_mut(&mut self) -> slice::IterMut<'_, u64> {
        self.0.iter_mut()
    }

    /// Function fills a new memory block and optionally XORs the old block over the new one.
    // TODO(tarcieri): optimized implementation (i.e. from opt.c instead of ref.c)
    pub(crate) fn fill_block(&mut self, prev_block: Block, ref_block: Block, with_xor: bool) {
        let mut block_r = ref_block ^ prev_block;
        let mut block_tmp = block_r;

        // Now block_r = ref_block + prev_block and block_tmp = ref_block + prev_block
        if with_xor {
            // Saving the next block contents for XOR over
            block_tmp ^= *self;
            // Now block_r = ref_block + prev_block and
            // block_tmp = ref_block + prev_block + next_block
        }

        /// Note: designed by the Lyra PHC team
        fn blake2_mult(x: u64, y: u64) -> u64 {
            let m = 0xFFFFFFFF;
            let xy = Wrapping((x & m) * (y & m)) * Wrapping(2);
            (Wrapping(x) + Wrapping(y) + xy).0
        }

        /// Blake2 round function
        macro_rules! blake2_round {
            (
                $v0:expr, $v1:expr, $v2:expr, $v3:expr, $v4:expr, $v5:expr, $v6:expr, $v7:expr,
                $v8:expr, $v9:expr, $v10:expr, $v11:expr, $v12:expr, $v13:expr, $v14:expr, $v15:expr
            ) => {
                blake2_inner!($v0, $v4, $v8, $v12);
                blake2_inner!($v1, $v5, $v9, $v13);
                blake2_inner!($v2, $v6, $v10, $v14);
                blake2_inner!($v3, $v7, $v11, $v15);
                blake2_inner!($v0, $v5, $v10, $v15);
                blake2_inner!($v1, $v6, $v11, $v12);
                blake2_inner!($v2, $v7, $v8, $v13);
                blake2_inner!($v3, $v4, $v9, $v14);
            };
        }

        macro_rules! blake2_inner {
            ($a:expr, $b:expr, $c:expr, $d:expr) => {
                $a = blake2_mult($a, $b);
                $d = ($d ^ $a).rotate_right(32);
                $c = blake2_mult($c, $d);
                $b = ($b ^ $c).rotate_right(24);
                $a = blake2_mult($a, $b);
                $d = ($d ^ $a).rotate_right(16);
                $c = blake2_mult($c, $d);
                $b = ($b ^ $c).rotate_right(63);
            };
        }

        // Apply Blake2 on columns of 64-bit words: (0, 1, ..., 15), then
        // (16, 17, ..31)... finally (112, 113, ...127)
        for i in 0..8 {
            blake2_round!(
                block_r[16 * i],
                block_r[16 * i + 1],
                block_r[16 * i + 2],
                block_r[16 * i + 3],
                block_r[16 * i + 4],
                block_r[16 * i + 5],
                block_r[16 * i + 6],
                block_r[16 * i + 7],
                block_r[16 * i + 8],
                block_r[16 * i + 9],
                block_r[16 * i + 10],
                block_r[16 * i + 11],
                block_r[16 * i + 12],
                block_r[16 * i + 13],
                block_r[16 * i + 14],
                block_r[16 * i + 15]
            );
        }

        // Apply Blake2 on rows of 64-bit words: (0, 1, 16, 17, ...112, 113), then
        // (2, 3, 18, 19, ..., 114, 115).. finally (14, 15, 30, 31, ..., 126, 127)
        for i in 0..8 {
            blake2_round!(
                block_r[2 * i],
                block_r[2 * i + 1],
                block_r[2 * i + 16],
                block_r[2 * i + 17],
                block_r[2 * i + 32],
                block_r[2 * i + 33],
                block_r[2 * i + 48],
                block_r[2 * i + 49],
                block_r[2 * i + 64],
                block_r[2 * i + 65],
                block_r[2 * i + 80],
                block_r[2 * i + 81],
                block_r[2 * i + 96],
                block_r[2 * i + 97],
                block_r[2 * i + 112],
                block_r[2 * i + 113]
            );
        }

        *self = block_tmp ^ block_r;
    }
}

impl BitXor for Block {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut res = self;
        res ^= rhs;
        res
    }
}

impl BitXorAssign for Block {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (a, b) in self.iter_mut().zip(rhs.iter()) {
            *a ^= *b;
        }
    }
}

impl Index<usize> for Block {
    type Output = u64;

    fn index(&self, index: usize) -> &u64 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Block {
    fn index_mut(&mut self, index: usize) -> &mut u64 {
        &mut self.0[index]
    }
}

#[cfg(feature = "zeroize")]
impl Zeroize for Block {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}
