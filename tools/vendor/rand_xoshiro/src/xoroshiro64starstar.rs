// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature="serde1")] use serde::{Serialize, Deserialize};
use rand_core::le::read_u32_into;
use rand_core::impls::{fill_bytes_via_next, next_u64_via_u32};
use rand_core::{RngCore, SeedableRng};

/// A xoroshiro64** random number generator.
///
/// The xoshiro64** algorithm is not suitable for cryptographic purposes, but
/// is very fast and has excellent statistical properties.
///
/// The algorithm used here is translated from [the `xoroshiro64starstar.c`
/// reference source code](http://xoshiro.di.unimi.it/xoroshiro64starstar.c) by
/// David Blackman and Sebastiano Vigna.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde1", derive(Serialize, Deserialize))]
pub struct Xoroshiro64StarStar {
    s0: u32,
    s1: u32,
}

impl RngCore for Xoroshiro64StarStar {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let r = starstar_u32!(self.s0);
        impl_xoroshiro_u32!(self);
        r
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        next_u64_via_u32(self)
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

impl SeedableRng for Xoroshiro64StarStar {
    type Seed = [u8; 8];

    /// Create a new `Xoroshiro64StarStar`.  If `seed` is entirely 0, it will be
    /// mapped to a different seed.
    fn from_seed(seed: [u8; 8]) -> Xoroshiro64StarStar {
        deal_with_zero_seed!(seed, Self);
        let mut s = [0; 2];
        read_u32_into(&seed, &mut s);

        Xoroshiro64StarStar {
            s0: s[0],
            s1: s[1],
        }
    }

    /// Seed a `Xoroshiro64StarStar` from a `u64` using `SplitMix64`.
    fn seed_from_u64(seed: u64) -> Xoroshiro64StarStar {
        from_splitmix!(seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mut rng = Xoroshiro64StarStar::from_seed([1, 0, 0, 0, 2, 0, 0, 0]);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/xoshiro64starstar.c
        let expected = [
            3802928447, 813792938, 1618621494, 2955957307, 3252880261,
            1129983909, 2539651700, 1327610908, 1757650787, 2763843748,
        ];
        for &e in &expected {
            assert_eq!(rng.next_u32(), e);
        }
    }

    #[test]
    fn zero_seed() {
        let mut rng = Xoroshiro64StarStar::seed_from_u64(0);
        assert_ne!(rng.next_u64(), 0);
    }
}
