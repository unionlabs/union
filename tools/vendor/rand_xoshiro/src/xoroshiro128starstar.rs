// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature="serde1")] use serde::{Serialize, Deserialize};
use rand_core::le::read_u64_into;
use rand_core::impls::fill_bytes_via_next;
use rand_core::{RngCore, SeedableRng};

/// A xoroshiro128** random number generator.
///
/// The xoroshiro128** algorithm is not suitable for cryptographic purposes, but
/// is very fast and has excellent statistical properties.
///
/// The algorithm used here is translated from [the `xoroshiro128starstar.c`
/// reference source code](http://xoshiro.di.unimi.it/xoroshiro128starstar.c) by
/// David Blackman and Sebastiano Vigna.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde1", derive(Serialize, Deserialize))]
pub struct Xoroshiro128StarStar {
    s0: u64,
    s1: u64,
}

impl Xoroshiro128StarStar {
    /// Jump forward, equivalently to 2^64 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^64 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoroshiro128StarStar;
    ///
    /// let rng1 = Xoroshiro128StarStar::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(u64, self, [0xdf900294d8f554a5, 0x170865df4b3201fc]);
    }

    /// Jump forward, equivalently to 2^96 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^32 starting points, from each of which
    /// `jump()` will generate 2^32 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(u64, self, [0xd2a98b26625eee7b, 0xdddf9b1090aa7ac1]);
    }
}

impl RngCore for Xoroshiro128StarStar {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let r = starstar_u64!(self.s0);
        impl_xoroshiro_u64!(self);
        r
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl SeedableRng for Xoroshiro128StarStar {
    type Seed = [u8; 16];

    /// Create a new `Xoroshiro128StarStar`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    fn from_seed(seed: [u8; 16]) -> Xoroshiro128StarStar {
        deal_with_zero_seed!(seed, Self);
        let mut s = [0; 2];
        read_u64_into(&seed, &mut s);

        Xoroshiro128StarStar {
            s0: s[0],
            s1: s[1],
        }
    }

    /// Seed a `Xoroshiro128StarStar` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoroshiro128StarStar {
        from_splitmix!(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mut rng = Xoroshiro128StarStar::from_seed(
            [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro128starstar.c
        let expected = [
            5760, 97769243520, 9706862127477703552, 9223447511460779954,
            8358291023205304566, 15695619998649302768, 8517900938696309774,
            16586480348202605369, 6959129367028440372, 16822147227405758281,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }
}
