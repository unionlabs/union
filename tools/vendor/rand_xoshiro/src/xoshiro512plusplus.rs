// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature="serde1")] use serde::{Serialize, Deserialize};
use rand_core::impls::fill_bytes_via_next;
use rand_core::le::read_u64_into;
use rand_core::{SeedableRng, RngCore, Error};

use crate::Seed512;

/// A xoshiro512++ random number generator.
///
/// The xoshiro512++ algorithm is not suitable for cryptographic purposes, but
/// is very fast and has excellent statistical properties.
///
/// The algorithm used here is translated from [the `xoshiro512plusplus.c`
/// reference source code](http://xoshiro.di.unimi.it/xoshiro512plusplus.c) by
/// David Blackman and Sebastiano Vigna.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde1", derive(Serialize, Deserialize))]
pub struct Xoshiro512PlusPlus {
    s: [u64; 8],
}

impl Xoshiro512PlusPlus {
    /// Jump forward, equivalently to 2^256 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^256 non-overlapping subsequences for
    /// parallel computations.
    ///
    /// ```
    /// use rand_xoshiro::rand_core::SeedableRng;
    /// use rand_xoshiro::Xoshiro512PlusPlus;
    ///
    /// let rng1 = Xoshiro512PlusPlus::seed_from_u64(0);
    /// let mut rng2 = rng1.clone();
    /// rng2.jump();
    /// let mut rng3 = rng2.clone();
    /// rng3.jump();
    /// ```
    pub fn jump(&mut self) {
        impl_jump!(u64, self, [
            0x33ed89b6e7a353f9, 0x760083d7955323be, 0x2837f2fbb5f22fae,
            0x4b8c5674d309511c, 0xb11ac47a7ba28c25, 0xf1be7667092bcc1c,
            0x53851efdb6df0aaf, 0x1ebbc8b23eaf25db
        ]);
    }

    /// Jump forward, equivalently to 2^384 calls to `next_u64()`.
    ///
    /// This can be used to generate 2^128 starting points, from each of which
    /// `jump()` will generate 2^128 non-overlapping subsequences for parallel
    /// distributed computations.
    pub fn long_jump(&mut self) {
        impl_jump!(u64, self, [
            0x11467fef8f921d28, 0xa2a819f2e79c8ea8, 0xa8299fc284b3959a,
            0xb4d347340ca63ee1, 0x1cb0940bedbff6ce, 0xd956c5c4fa1f8e17,
            0x915e38fd4eda93bc, 0x5b3ccdfa5d7daca5
        ]);
    }
}


impl SeedableRng for Xoshiro512PlusPlus {
    type Seed = Seed512;

    /// Create a new `Xoshiro512PlusPlus`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    #[inline]
    fn from_seed(seed: Seed512) -> Xoshiro512PlusPlus {
        deal_with_zero_seed!(seed, Self);
        let mut state = [0; 8];
        read_u64_into(&seed.0, &mut state);
        Xoshiro512PlusPlus { s: state }
    }

    /// Seed a `Xoshiro512PlusPlus` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoshiro512PlusPlus {
        from_splitmix!(seed)
    }
}

impl RngCore for Xoshiro512PlusPlus {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // The lowest bits have some linear dependencies, so we use the
        // upper bits instead.
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let result_plusplus = plusplus_u64!(self.s[2], self.s[0], 17);
        impl_xoshiro_large!(self);
        result_plusplus
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mut rng = Xoshiro512PlusPlus::from_seed(Seed512(
            [1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0,
             3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
             5, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0,
             7, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0]));
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro512plusplus.c
        let expected = [
            524291, 1048578, 539099140, 3299073855497, 6917532603230064654,
            7494048333530275843, 14418333309547923463, 10960079161595355914,
            18279570946505382726, 10209173166699159237,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u64(), e);
        }
    }
}
