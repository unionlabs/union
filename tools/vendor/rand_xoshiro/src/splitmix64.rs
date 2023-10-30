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
use rand_core::{RngCore, SeedableRng, Error};

/// A splitmix64 random number generator.
///
/// The splitmix algorithm is not suitable for cryptographic purposes, but is
/// very fast and has a 64 bit state.
///
/// The algorithm used here is translated from [the `splitmix64.c`
/// reference source code](http://xoshiro.di.unimi.it/splitmix64.c) by
/// Sebastiano Vigna. For `next_u32`, a more efficient mixing function taken
/// from [`dsiutils`](http://dsiutils.di.unimi.it/) is used.
#[allow(missing_copy_implementations)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature="serde1", derive(Serialize, Deserialize))]
pub struct SplitMix64 {
    x: u64,
}

const PHI: u64 = 0x9e3779b97f4a7c15;

impl RngCore for SplitMix64 {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.x = self.x.wrapping_add(PHI);
        let mut z = self.x;
        // David Stafford's
        // (http://zimbry.blogspot.com/2011/09/better-bit-mixing-improving-on.html)
        // "Mix4" variant of the 64-bit finalizer in Austin Appleby's
        // MurmurHash3 algorithm.
        z = (z ^ (z >> 33)).wrapping_mul(0x62A9D9ED799705F5);
        z = (z ^ (z >> 28)).wrapping_mul(0xCB24D0A5C88C35B3);
        (z >> 32) as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.x = self.x.wrapping_add(PHI);
        let mut z = self.x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
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

impl SeedableRng for SplitMix64 {
    type Seed = [u8; 8];

    /// Create a new `SplitMix64`.
    fn from_seed(seed: [u8; 8]) -> SplitMix64 {
        let mut state = [0; 1];
        read_u64_into(&seed, &mut state);
        SplitMix64 {
            x: state[0],
        }
    }

    /// Seed a `SplitMix64` from a `u64`.
    fn seed_from_u64(seed: u64) -> SplitMix64 {
        SplitMix64::from_seed(seed.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference() {
        let mut rng = SplitMix64::seed_from_u64(1477776061723855037);
        // These values were produced with the reference implementation:
        // http://xoshiro.di.unimi.it/splitmix64.c
        let expected : [u64 ; 50]= [
            1985237415132408290, 2979275885539914483, 13511426838097143398,
            8488337342461049707, 15141737807933549159, 17093170987380407015,
            16389528042912955399, 13177319091862933652, 10841969400225389492,
            17094824097954834098, 3336622647361835228, 9678412372263018368,
            11111587619974030187, 7882215801036322410, 5709234165213761869,
            7799681907651786826, 4616320717312661886, 4251077652075509767,
            7836757050122171900, 5054003328188417616, 12919285918354108358,
            16477564761813870717, 5124667218451240549, 18099554314556827626,
            7603784838804469118, 6358551455431362471, 3037176434532249502,
            3217550417701719149, 9958699920490216947, 5965803675992506258,
            12000828378049868312, 12720568162811471118, 245696019213873792,
            8351371993958923852, 14378754021282935786, 5655432093647472106,
            5508031680350692005, 8515198786865082103, 6287793597487164412,
            14963046237722101617, 3630795823534910476, 8422285279403485710,
            10554287778700714153, 10871906555720704584, 8659066966120258468,
            9420238805069527062, 10338115333623340156, 13514802760105037173,
            14635952304031724449, 15419692541594102413,
        ];
        for &e in expected.iter() {
            assert_eq!(rng.next_u64(), e);
        }
    }

    #[test]
    fn next_u32() {
        let mut rng = SplitMix64::seed_from_u64(10);
        // These values were produced with the reference implementation:
        // http://dsiutils.di.unimi.it/dsiutils-2.5.1-src.tar.gz
        let expected : [u32 ; 100]= [
            3930361779, 4016923089, 4113052479, 925926767, 1755287528,
            802865554, 954171070, 3724185978, 173676273, 1414488795, 12664133,
            1784889697, 1303817078, 261610523, 941280008, 2571813643,
            2954453492, 378291111, 2546873158, 3923319175, 645257028,
            3881821278, 2681538690, 3037029984, 1999958137, 1853970361,
            2989951788, 2126166628, 839962987, 3989679659, 3656977858,
            684284364, 1673258011, 170979192, 3037622326, 1600748179,
            1780764218, 1141430714, 4139736875, 3336905707, 2262051600,
            3830850262, 2430765325, 1073032139, 1668888979, 2716938970,
            4102420032, 40305196, 386350562, 2754480591, 622869439, 2129598760,
            2306038241, 4218338739, 412298926, 3453855056, 3061469690,
            4284292697, 994843708, 1591016681, 414726151, 1238182607, 18073498,
            1237631493, 351884714, 2347486264, 2488990876, 802846256, 645670443,
            957607012, 3126589776, 1966356370, 3036485766, 868696717,
            2808613630, 2070968151, 1025536863, 1743949425, 466212687,
            2994327271, 209776458, 1246125124, 3344380309, 2203947859,
            968313105, 2805485302, 197484837, 3472483632, 3931823935,
            3288490351, 4165666529, 3671080416, 689542830, 1272555356,
            1039141475, 3984640460, 4142959054, 2252788890, 2459379590,
            991872507,
        ];
        for &e in expected.iter() {
            assert_eq!(rng.next_u32(), e);
        }
    }
}
