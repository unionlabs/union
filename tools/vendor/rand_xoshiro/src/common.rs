// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Initialize a RNG from a `u64` seed using `SplitMix64`.
macro_rules! from_splitmix {
    ($seed:expr) => { {
        let mut rng = crate::SplitMix64::seed_from_u64($seed);
        Self::from_rng(&mut rng).unwrap()
    } }
}

/// Apply the ** scrambler used by some RNGs from the xoshiro family.
macro_rules! starstar_u64 {
    ($x:expr) => {
        $x.wrapping_mul(5).rotate_left(7).wrapping_mul(9)
    }
}

/// Apply the ** scrambler used by some RNGs from the xoshiro family.
macro_rules! starstar_u32 {
    ($x:expr) => {
        $x.wrapping_mul(0x9E3779BB).rotate_left(5).wrapping_mul(5)
    }
}

/// Apply the ++ scrambler used by some RNGs from the xoshiro family.
macro_rules! plusplus_u64 {
    ($x:expr, $y:expr, $rot:expr) => {
        $x.wrapping_add($y).rotate_left($rot).wrapping_add($x)
    }
}

/// Apply the ++ scrambler used by some RNGs from the xoshiro family.
macro_rules! plusplus_u32 {
    ($x:expr, $y:expr) => {
        $x.wrapping_add($y).rotate_left(7).wrapping_add($x)
    }
}

/// Implement a jump function for an RNG from the xoshiro family.
macro_rules! impl_jump {
    (u32, $self:expr, [$j0:expr, $j1:expr]) => {
        const JUMP: [u32; 2] = [$j0, $j1];
        let mut s0 = 0;
        let mut s1 = 0;
        for j in &JUMP {
            for b in 0..32 {
                if (j & 1 << b) != 0 {
                    s0 ^= $self.s0;
                    s1 ^= $self.s1;
                }
                $self.next_u32();
            }
        }
        $self.s0 = s0;
        $self.s1 = s1;
    };
    (u64, $self:expr, [$j0:expr, $j1:expr]) => {
        const JUMP: [u64; 2] = [$j0, $j1];
        let mut s0 = 0;
        let mut s1 = 0;
        for j in &JUMP {
            for b in 0..64 {
                if (j & 1 << b) != 0 {
                    s0 ^= $self.s0;
                    s1 ^= $self.s1;
                }
                $self.next_u64();
            }
        }
        $self.s0 = s0;
        $self.s1 = s1;
    };
    (u32, $self:expr, [$j0:expr, $j1:expr, $j2:expr, $j3:expr]) => {
        const JUMP: [u32; 4] = [$j0, $j1, $j2, $j3];
        let mut s0 = 0;
        let mut s1 = 0;
        let mut s2 = 0;
        let mut s3 = 0;
        for j in &JUMP {
            for b in 0..32 {
                if (j & 1 << b) != 0 {
                    s0 ^= $self.s[0];
                    s1 ^= $self.s[1];
                    s2 ^= $self.s[2];
                    s3 ^= $self.s[3];
                }
                $self.next_u32();
            }
        }
        $self.s[0] = s0;
        $self.s[1] = s1;
        $self.s[2] = s2;
        $self.s[3] = s3;
    };
    (u64, $self:expr, [$j0:expr, $j1:expr, $j2:expr, $j3:expr]) => {
        const JUMP: [u64; 4] = [$j0, $j1, $j2, $j3];
        let mut s0 = 0;
        let mut s1 = 0;
        let mut s2 = 0;
        let mut s3 = 0;
        for j in &JUMP {
            for b in 0..64 {
                if (j & 1 << b) != 0 {
                    s0 ^= $self.s[0];
                    s1 ^= $self.s[1];
                    s2 ^= $self.s[2];
                    s3 ^= $self.s[3];
                }
                $self.next_u64();
            }
        }
        $self.s[0] = s0;
        $self.s[1] = s1;
        $self.s[2] = s2;
        $self.s[3] = s3;
    };
    (u64, $self:expr, [$j0:expr, $j1:expr, $j2:expr, $j3:expr,
                       $j4:expr, $j5:expr, $j6:expr, $j7:expr]) => {
        const JUMP: [u64; 8] = [$j0, $j1, $j2, $j3, $j4, $j5, $j6, $j7];
        let mut s = [0; 8];
        for j in &JUMP {
            for b in 0..64 {
                if (j & 1 << b) != 0 {
                    s[0] ^= $self.s[0];
                    s[1] ^= $self.s[1];
                    s[2] ^= $self.s[2];
                    s[3] ^= $self.s[3];
                    s[4] ^= $self.s[4];
                    s[5] ^= $self.s[5];
                    s[6] ^= $self.s[6];
                    s[7] ^= $self.s[7];
                }
                $self.next_u64();
            }
        }
        $self.s = s;
    };
}

/// Implement the xoroshiro iteration.
macro_rules! impl_xoroshiro_u32 {
    ($self:expr) => {
        $self.s1 ^= $self.s0;
        $self.s0 = $self.s0.rotate_left(26) ^ $self.s1 ^ ($self.s1 << 9);
        $self.s1 = $self.s1.rotate_left(13);
    }
}

/// Implement the xoroshiro iteration.
macro_rules! impl_xoroshiro_u64 {
    ($self:expr) => {
        $self.s1 ^= $self.s0;
        $self.s0 = $self.s0.rotate_left(24) ^ $self.s1 ^ ($self.s1 << 16);
        $self.s1 = $self.s1.rotate_left(37);
    }
}

/// Implement the xoroshiro iteration for the ++ scrambler.
macro_rules! impl_xoroshiro_u64_plusplus {
    ($self:expr) => {
        $self.s1 ^= $self.s0;
        $self.s0 = $self.s0.rotate_left(49) ^ $self.s1 ^ ($self.s1 << 21);
        $self.s1 = $self.s1.rotate_left(28);
    }
}

/// Implement the xoshiro iteration for `u32` output.
macro_rules! impl_xoshiro_u32 {
    ($self:expr) => {
        let t = $self.s[1] << 9;

        $self.s[2] ^= $self.s[0];
        $self.s[3] ^= $self.s[1];
        $self.s[1] ^= $self.s[2];
        $self.s[0] ^= $self.s[3];

        $self.s[2] ^= t;

        $self.s[3] = $self.s[3].rotate_left(11);
    }
}

/// Implement the xoshiro iteration for `u64` output.
macro_rules! impl_xoshiro_u64 {
    ($self:expr) => {
        let t = $self.s[1] << 17;

        $self.s[2] ^= $self.s[0];
        $self.s[3] ^= $self.s[1];
        $self.s[1] ^= $self.s[2];
        $self.s[0] ^= $self.s[3];

        $self.s[2] ^= t;

        $self.s[3] = $self.s[3].rotate_left(45);
    }
}

/// Implement the large-state xoshiro iteration.
macro_rules! impl_xoshiro_large {
    ($self:expr) => {
        let t = $self.s[1] << 11;

        $self.s[2] ^= $self.s[0];
        $self.s[5] ^= $self.s[1];
        $self.s[1] ^= $self.s[2];
        $self.s[7] ^= $self.s[3];
        $self.s[3] ^= $self.s[4];
        $self.s[4] ^= $self.s[5];
        $self.s[0] ^= $self.s[6];
        $self.s[6] ^= $self.s[7];

        $self.s[6] ^= t;

        $self.s[7] = $self.s[7].rotate_left(21);
    }
}

/// Map an all-zero seed to a different one.
macro_rules! deal_with_zero_seed {
    ($seed:expr, $Self:ident) => {
        if $seed.iter().all(|&x| x == 0) {
            return $Self::seed_from_u64(0);
        }
    }
}

/// 512-bit seed for a generator.
///
/// This wrapper is necessary, because some traits required for a seed are not
/// implemented on large arrays.
#[derive(Clone)]
pub struct Seed512(pub [u8; 64]);

impl Seed512 {
    /// Return an iterator over the seed.
    pub fn iter(&self) -> core::slice::Iter<u8> {
        self.0.iter()
    }
}

impl core::fmt::Debug for Seed512 {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0[..].fmt(f)
    }
}

impl Default for Seed512 {
    fn default() -> Seed512 {
        Seed512([0; 64])
    }
}

impl AsMut<[u8]> for Seed512 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

