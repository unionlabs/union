/*
Most of the code in this file is adapted from code from the Rust `rand` library, https://docs.rs/rand/latest/rand/, modified under the MIT license. The changes are released under either the MIT license or the Apache License 2.0, as described in the README. See LICENSE-MIT or LICENSE-APACHE at the project root.
The original license file for `rand` can be found in this project's root at licenses/LICENSE-rand.

The appropriate copyright notices for this are given below:
Copyright 2018 Developers of the Rand project.
Copyright 2013-2017 The Rust Project Developers.
Copyright 2018-2020 Developers of the Rand project.
Copyright 2017 The Rust Project Developers.
*/

//! Utilities for generating random bnum integers.
//!
//! The `rand` feature must be enabled to use items from this module.

use core::ops::{Deref, DerefMut};

/// Wrapper type designed to be filled with random big integers.
///
/// This type exists because [`rand::Fill`](https://docs.rs/rand/latest/rand/trait.Fill.html) can't be implemented for `[BUint<N>]` (or any other slice of bnum integers) due to Rust's orphan rules. Instead, it is implemented for `Slice<BUint<N>>`, etc. The underlying slice can then be accessed by calling [`AsRef::as_ref`](https://doc.rust-lang.org/core/convert/trait.AsRef.html#tymethod.as_ref) or [`AsMut::as_mut`](https://doc.rust-lang.org/core/convert/trait.AsMut.html#tymethod.as_mut) on the wrapper, or deferencing it. An alternative way of filling a slice with random big integers is using the [`try_fill_slice`] method in this crate's [`random`](crate::random) module.
pub struct Slice<T>(pub [T]);

impl<T> AsRef<[T]> for Slice<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T> AsMut<[T]> for Slice<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T> Deref for Slice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Slice<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// Fills a slice with random big integers.
///
/// An alternative way of filling a slice with random big integers is using the [`rand::Fill`](https://docs.rs/rand/latest/rand/trait.Fill.html) trait's method [`try_fill`](https://docs.rs/rand/latest/rand/trait.Fill.html#tymethod.try_fill) on the [`Slice`] struct in this crate's [`random`](crate::random) module.
///
/// # Examples
///
/// ```
/// // Fill a `Vec` of length 10 with random `I256`s
///
/// use bnum::types::I256;
/// use bnum::random;
/// use rand::rngs::StdRng;
/// use rand::SeedableRng;
///
/// let mut v = vec![I256::ZERO; 10];
/// let mut rng = StdRng::seed_from_u64(0);
///
/// random::try_fill_slice(&mut v, &mut rng).unwrap();
/// // each initial `I256::ZERO` is replaced with a random `I256`
///
/// println!("{:?}", v);
/// ```
pub fn try_fill_slice<T, R: Rng + ?Sized>(slice: &mut [T], rng: &mut R) -> Result<(), Error>
where
    Slice<T>: Fill,
{
    let slice = unsafe { &mut *(slice as *mut _ as *mut Slice<T>) };
    Fill::try_fill(slice, rng)
}

macro_rules! fill_impl {
    ($ty: ty) => {
        impl<const N: usize> Fill for crate::random::Slice<$ty> {
            #[inline(never)]
            fn try_fill<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), Error> {
                if self.0.len() > 0 {
                    rng.try_fill_bytes(unsafe {
                        core::slice::from_raw_parts_mut(
                            self.0.as_mut_ptr() as *mut u8,
                            self.0.len() * core::mem::size_of::<$ty>(),
                        )
                    })?;
                    for x in &mut self.0 {
                        *x = x.to_le();
                    }
                }
                Ok(())
            }
        }
    };
}

macro_rules! uniform_int_impl {
    ($ty: ty, $u_large: ty $(, $as_unsigned: ident, $as_signed: ident)?) => {
        impl<const N: usize> SampleUniform for $ty {
            type Sampler = UniformInt<$ty>;
        }

        impl<const N: usize> UniformSampler for UniformInt<$ty> {
            type X = $ty;

            #[inline]
            fn new<B1, B2>(low_b: B1, high_b: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                assert!(low < high, "Uniform::new called with `low >= high`");
                UniformSampler::new_inclusive(low, high - <$ty>::ONE)
            }

            #[inline]
            fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Self
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                assert!(
                    low <= high,
                    "Uniform::new_inclusive called with `low > high`"
                );

                let range = high.wrapping_sub(low).wrapping_add(<$ty>::ONE)$(.$as_unsigned())?;
                let ints_to_reject = if !range.is_zero() {
                    (<$u_large>::MAX - range + 1) % range
                } else {
                    <$u_large>::ZERO
                };

                UniformInt {
                    low,
                    range: $(<$ty>::$as_signed)?(range),
                    z: $(<$ty>::$as_signed)?(ints_to_reject),
                }
            }

            #[inline]
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                let range = self.range$(.$as_unsigned())?;
                if !range.is_zero() {
                    let zone = <$u_large>::MAX - (self.z)$(.$as_unsigned())?;
                    loop {
                        let v: $u_large = rng.gen();
                        let (lo, hi) = v.widening_mul(range);
                        if lo <= zone {
                            return self.low.wrapping_add($(<$ty>::$as_signed)?(hi));
                        }
                    }
                } else {
                    rng.gen()
                }
            }

            #[inline]
            fn sample_single<R: Rng + ?Sized, B1, B2>(low_b: B1, high_b: B2, rng: &mut R) -> Self::X
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                assert!(low < high, "UniformSampler::sample_single: low >= high");
                Self::sample_single_inclusive(low, high - <$ty>::ONE, rng)
            }

            #[inline]
            fn sample_single_inclusive<R: Rng + ?Sized, B1, B2>(low_b: B1, high_b: B2, rng: &mut R) -> Self::X
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                assert!(low <= high, "UniformSampler::sample_single_inclusive: low > high");
                let range = high.wrapping_sub(low).wrapping_add(<$ty>::ONE)$(.$as_unsigned())?;
                if range.is_zero() {
                    return rng.gen();
                }

                let zone = if <$u_large>::MAX.bits() <= 16 {
                    let ints_to_reject = (<$u_large>::MAX - range + 1) % range;
                    <$u_large>::MAX - ints_to_reject
                } else {
                    (range << range.leading_zeros()).wrapping_sub(<$u_large>::ONE)
                };

                loop {
                    let v: $u_large = rng.gen();
                    let (lo, hi) = v.widening_mul(range);
                    if lo <= zone {
                        return low.wrapping_add($(<$ty>::$as_signed)?(hi));
                    }
                }
            }
        }
    };
}

#[cfg(test)]
macro_rules! test_random {
    ($int: ty; $($Rng: ty), *) => {
        paste::paste! {
            $(
                quickcheck::quickcheck! {
                    #[allow(non_snake_case)]
                    fn [<quickcheck_ $Rng _gen_ $int>](seed: u64) -> bool {
                        use crate::test::TestConvert;
                        use crate::test::types::*;
                        use rand::Rng;
                        use rand::rngs::$Rng;

                        let (mut rng, mut rng2) = seeded_rngs::<$Rng>(seed);

                        let big = rng.gen::<[<$int:upper>]>();
                        let primitive = rng2.gen::<$int>();

                        TestConvert::into(big) == TestConvert::into(primitive)
                    }

                    #[allow(non_snake_case)]
                    fn [<quickcheck_ $Rng _fill_ $int _slice>](seed: u64) -> bool {
                        use crate::test::TestConvert;
                        use rand::Fill;
                        use rand::rngs::$Rng;

                        const SLICE_LENGTH: usize = 20;

                        let (mut rng, mut rng2) = seeded_rngs::<$Rng>(seed);

                        let mut big_array = [<[<$int:upper>]>::MIN; SLICE_LENGTH];
                        let mut primitive_array = [<$int>::MIN; SLICE_LENGTH];

                        crate::random::try_fill_slice(&mut big_array, &mut rng).unwrap();

                        primitive_array.try_fill(&mut rng2).unwrap();

                        big_array
                            .into_iter()
                            .zip(primitive_array.into_iter())
                            .all(|(big, primitive)| {
                                TestConvert::into(big) == TestConvert::into(primitive)
                            })
                    }

                    #[allow(non_snake_case)]
            		fn [<quickcheck_ $Rng _gen_range_ $int>](seed: u64, min: $int, max: $int) -> quickcheck::TestResult {
						if min >= max {
							return quickcheck::TestResult::discard();
						}
						use crate::test::TestConvert;
						use rand::Rng;
						use rand::rngs::$Rng;

						let (mut rng, mut rng2) = seeded_rngs::<$Rng>(seed);

						let min_big = [<$int:upper>]::from(min);
						let max_big = [<$int:upper>]::from(max);

						let big = rng.gen_range(min_big..max_big);
						let primitive = rng2.gen_range(min..max);

						quickcheck::TestResult::from_bool(TestConvert::into(big) == TestConvert::into(primitive))
					}
				}
			)*
		}
	};
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Used for generating random big integers in a given range.
///
/// Implements the [`UniformSampler`](https://docs.rs/rand/latest/rand/distributions/uniform/trait.UniformSampler.html) trait from the [`rand`](https://docs.rs/rand/latest/rand/) crate. This struct should not be used directly; instead use the [`Uniform`](https://docs.rs/rand/latest/rand/distributions/struct.Uniform.html) struct from the [`rand`](https://docs.rs/rand/latest/rand/) crate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UniformInt<X> {
    low: X,
    range: X,
    z: X,
}

use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use rand::distributions::{Distribution, Standard};
use rand::{Error, Fill, Rng};

macro_rules! random {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl<const N: usize> Distribution<$BUint<N>> for Standard {
            #[inline]
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $BUint<N> {
                let mut digits = [0; N];
                rng.fill(&mut digits);
                $BUint::from_digits(digits)
            }
        }

        impl<const N: usize> Distribution<$BInt<N>> for Standard {
            #[inline]
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $BInt<N> {
                $BInt::from_bits(rng.gen())
            }
        }

        fill_impl!($BUint<N>);
        fill_impl!($BInt<N>);

        uniform_int_impl!($BUint<N>, $BUint<N>);

        uniform_int_impl!($BInt<N>, $BUint<N>, to_bits, from_bits);

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::types::*;
                use rand::SeedableRng;

                fn seeded_rngs<R: SeedableRng + Clone>(seed: u64) -> (R, R) {
                    let rng = R::seed_from_u64(seed);
                    let rng2 = rng.clone(); // need to clone `rng` to produce the same results before it is used as `gen` mutates it
                    (rng, rng2)
                }

                test_random!(utest; StdRng, SmallRng);
                test_random!(itest; StdRng, SmallRng);
            }
        }
    };
}

crate::macro_impl!(random);
