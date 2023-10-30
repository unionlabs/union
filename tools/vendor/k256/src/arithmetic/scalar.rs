//! Scalar field arithmetic.

#[cfg_attr(not(target_pointer_width = "64"), path = "scalar/wide32.rs")]
#[cfg_attr(target_pointer_width = "64", path = "scalar/wide64.rs")]
mod wide;

pub(crate) use self::wide::WideScalar;

use crate::{FieldBytes, Secp256k1, ORDER};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Shr, Sub, SubAssign};
use elliptic_curve::{
    bigint::{self, prelude::*, Limb, Word, U256, U512},
    generic_array::arr,
    group::ff::{Field, PrimeField},
    ops::{Reduce, ReduceNonZero},
    rand_core::{CryptoRng, RngCore},
    subtle::{
        Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
        CtOption,
    },
    zeroize::DefaultIsZeroes,
    Curve, IsHigh, ScalarArithmetic, ScalarCore,
};

#[cfg(feature = "bits")]
use {crate::ScalarBits, elliptic_curve::group::ff::PrimeFieldBits};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

#[cfg(test)]
use num_bigint::{BigUint, ToBigUint};

impl ScalarArithmetic for Secp256k1 {
    type Scalar = Scalar;
}

/// Constant representing the modulus
/// n = FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141
const MODULUS: [Word; bigint::nlimbs!(256)] = ORDER.to_words();

/// Constant representing the modulus / 2
const FRAC_MODULUS_2: U256 = ORDER.shr_vartime(1);

/// Scalars are elements in the finite field modulo n.
///
/// # Trait impls
///
/// Much of the important functionality of scalars is provided by traits from
/// the [`ff`](https://docs.rs/ff/) crate, which is re-exported as
/// `k256::elliptic_curve::ff`:
///
/// - [`Field`](https://docs.rs/ff/latest/ff/trait.Field.html) -
///   represents elements of finite fields and provides:
///   - [`Field::random`](https://docs.rs/ff/latest/ff/trait.Field.html#tymethod.random) -
///     generate a random scalar
///   - `double`, `square`, and `invert` operations
///   - Bounds for [`Add`], [`Sub`], [`Mul`], and [`Neg`] (as well as `*Assign` equivalents)
///   - Bounds for [`ConditionallySelectable`] from the `subtle` crate
/// - [`PrimeField`](https://docs.rs/ff/latest/ff/trait.PrimeField.html) -
///   represents elements of prime fields and provides:
///   - `from_repr`/`to_repr` for converting field elements from/to big integers.
///   - `multiplicative_generator` and `root_of_unity` constants.
/// - [`PrimeFieldBits`](https://docs.rs/ff/latest/ff/trait.PrimeFieldBits.html) -
///   operations over field elements represented as bits (requires `bits` feature)
///
/// Please see the documentation for the relevant traits for more information.
///
/// # `serde` support
///
/// When the `serde` feature of this crate is enabled, the `Serialize` and
/// `Deserialize` traits are impl'd for this type.
///
/// The serialization is a fixed-width big endian encoding. When used with
/// textual formats, the binary data is encoded as hexadecimal.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(docsrs, doc(cfg(feature = "arithmetic")))]
pub struct Scalar(U256);

impl Scalar {
    /// Zero scalar.
    pub const ZERO: Self = Self(U256::ZERO);

    /// Multiplicative identity.
    pub const ONE: Self = Self(U256::ONE);

    /// Checks if the scalar is zero.
    pub fn is_zero(&self) -> Choice {
        self.0.is_zero()
    }

    /// Returns the SEC1 encoding of this scalar.
    pub fn to_bytes(&self) -> FieldBytes {
        self.0.to_be_byte_array()
    }

    /// Negates the scalar.
    pub const fn negate(&self) -> Self {
        Self(self.0.neg_mod(&ORDER))
    }

    /// Returns self + rhs mod n.
    pub const fn add(&self, rhs: &Self) -> Self {
        Self(self.0.add_mod(&rhs.0, &ORDER))
    }

    /// Returns self - rhs mod n.
    pub const fn sub(&self, rhs: &Self) -> Self {
        Self(self.0.sub_mod(&rhs.0, &ORDER))
    }

    /// Modulo multiplies two scalars.
    pub fn mul(&self, rhs: &Scalar) -> Scalar {
        WideScalar::mul_wide(self, rhs).reduce()
    }

    /// Modulo squares the scalar.
    pub fn square(&self) -> Self {
        self.mul(self)
    }

    /// Right shifts the scalar.
    ///
    /// Note: not constant-time with respect to the `shift` parameter.
    pub fn shr_vartime(&self, shift: usize) -> Scalar {
        Self(self.0.shr_vartime(shift))
    }

    /// Inverts the scalar.
    pub fn invert(&self) -> CtOption<Self> {
        // Using an addition chain from
        // https://briansmith.org/ecc-inversion-addition-chains-01#secp256k1_scalar_inversion
        let x_1 = *self;
        let x_10 = self.pow2k(1);
        let x_11 = x_10.mul(&x_1);
        let x_101 = x_10.mul(&x_11);
        let x_111 = x_10.mul(&x_101);
        let x_1001 = x_10.mul(&x_111);
        let x_1011 = x_10.mul(&x_1001);
        let x_1101 = x_10.mul(&x_1011);

        let x6 = x_1101.pow2k(2).mul(&x_1011);
        let x8 = x6.pow2k(2).mul(&x_11);
        let x14 = x8.pow2k(6).mul(&x6);
        let x28 = x14.pow2k(14).mul(&x14);
        let x56 = x28.pow2k(28).mul(&x28);

        #[rustfmt::skip]
            let res = x56
            .pow2k(56).mul(&x56)
            .pow2k(14).mul(&x14)
            .pow2k(3).mul(&x_101)
            .pow2k(4).mul(&x_111)
            .pow2k(4).mul(&x_101)
            .pow2k(5).mul(&x_1011)
            .pow2k(4).mul(&x_1011)
            .pow2k(4).mul(&x_111)
            .pow2k(5).mul(&x_111)
            .pow2k(6).mul(&x_1101)
            .pow2k(4).mul(&x_101)
            .pow2k(3).mul(&x_111)
            .pow2k(5).mul(&x_1001)
            .pow2k(6).mul(&x_101)
            .pow2k(10).mul(&x_111)
            .pow2k(4).mul(&x_111)
            .pow2k(9).mul(&x8)
            .pow2k(5).mul(&x_1001)
            .pow2k(6).mul(&x_1011)
            .pow2k(4).mul(&x_1101)
            .pow2k(5).mul(&x_11)
            .pow2k(6).mul(&x_1101)
            .pow2k(10).mul(&x_1101)
            .pow2k(4).mul(&x_1001)
            .pow2k(6).mul(&x_1)
            .pow2k(8).mul(&x6);

        CtOption::new(res, !self.is_zero())
    }

    /// Returns the scalar modulus as a `BigUint` object.
    #[cfg(test)]
    pub fn modulus_as_biguint() -> BigUint {
        Self::one().negate().to_biguint().unwrap() + 1.to_biguint().unwrap()
    }

    /// Returns a (nearly) uniformly-random scalar, generated in constant time.
    pub fn generate_biased(mut rng: impl CryptoRng + RngCore) -> Self {
        // We reduce a random 512-bit value into a 256-bit field, which results in a
        // negligible bias from the uniform distribution, but the process is constant-time.
        let mut buf = [0u8; 64];
        rng.fill_bytes(&mut buf);
        WideScalar::from_bytes(&buf).reduce()
    }

    /// Returns a uniformly-random scalar, generated using rejection sampling.
    // TODO(tarcieri): make this a `CryptoRng` when `ff` allows it
    pub fn generate_vartime(mut rng: impl RngCore) -> Self {
        let mut bytes = FieldBytes::default();

        // TODO: pre-generate several scalars to bring the probability of non-constant-timeness down?
        loop {
            rng.fill_bytes(&mut bytes);
            if let Some(scalar) = Scalar::from_repr(bytes).into() {
                return scalar;
            }
        }
    }

    /// Attempts to parse the given byte array as a scalar.
    /// Does not check the result for being in the correct range.
    pub(crate) const fn from_bytes_unchecked(bytes: &[u8; 32]) -> Self {
        Self(U256::from_be_slice(bytes))
    }

    /// Raises the scalar to the power `2^k`.
    fn pow2k(&self, k: usize) -> Self {
        let mut x = *self;
        for _j in 0..k {
            x = x.square();
        }
        x
    }
}

impl Field for Scalar {
    fn random(rng: impl RngCore) -> Self {
        // Uses rejection sampling as the default random generation method,
        // which produces a uniformly random distribution of scalars.
        //
        // This method is not constant time, but should be secure so long as
        // rejected RNG outputs are unrelated to future ones (which is a
        // necessary property of a `CryptoRng`).
        //
        // With an unbiased RNG, the probability of failing to complete after 4
        // iterations is vanishingly small.
        Self::generate_vartime(rng)
    }

    fn zero() -> Self {
        Self::ZERO
    }

    fn one() -> Self {
        Self::ONE
    }

    #[must_use]
    fn square(&self) -> Self {
        Scalar::square(self)
    }

    #[must_use]
    fn double(&self) -> Self {
        self.add(self)
    }

    fn invert(&self) -> CtOption<Self> {
        Scalar::invert(self)
    }

    /// Tonelli-Shank's algorithm for q mod 16 = 1
    /// <https://eprint.iacr.org/2012/685.pdf> (page 12, algorithm 5)
    #[allow(clippy::many_single_char_names)]
    fn sqrt(&self) -> CtOption<Self> {
        // Note: `pow_vartime` is constant-time with respect to `self`
        let w = self.pow_vartime(&[
            0x777fa4bd19a06c82,
            0xfd755db9cd5e9140,
            0xffffffffffffffff,
            0x1ffffffffffffff,
        ]);

        let mut v = Self::S;
        let mut x = *self * w;
        let mut b = x * w;
        let mut z = Self::root_of_unity();

        for max_v in (1..=Self::S).rev() {
            let mut k = 1;
            let mut tmp = b.square();
            let mut j_less_than_v = Choice::from(1);

            for j in 2..max_v {
                let tmp_is_one = tmp.ct_eq(&Self::one());
                let squared = Self::conditional_select(&tmp, &z, tmp_is_one).square();
                tmp = Self::conditional_select(&squared, &tmp, tmp_is_one);
                let new_z = Self::conditional_select(&z, &squared, tmp_is_one);
                j_less_than_v &= !j.ct_eq(&v);
                k = u32::conditional_select(&j, &k, tmp_is_one);
                z = Self::conditional_select(&z, &new_z, j_less_than_v);
            }

            let result = x * z;
            x = Self::conditional_select(&result, &x, b.ct_eq(&Self::one()));
            z = z.square();
            b *= z;
            v = k;
        }

        CtOption::new(x, x.square().ct_eq(self))
    }
}

impl PrimeField for Scalar {
    type Repr = FieldBytes;

    const NUM_BITS: u32 = 256;
    const CAPACITY: u32 = 255;
    const S: u32 = 6;

    /// Attempts to parse the given byte array as an SEC1-encoded scalar.
    ///
    /// Returns None if the byte array does not contain a big-endian integer in the range
    /// [0, p).
    fn from_repr(bytes: FieldBytes) -> CtOption<Self> {
        let inner = U256::from_be_byte_array(bytes);
        CtOption::new(Self(inner), inner.ct_lt(&Secp256k1::ORDER))
    }

    fn to_repr(&self) -> FieldBytes {
        self.to_bytes()
    }

    fn is_odd(&self) -> Choice {
        self.0.is_odd()
    }

    fn multiplicative_generator() -> Self {
        7u64.into()
    }

    fn root_of_unity() -> Self {
        Scalar::from_repr(arr![u8;
            0x0c, 0x1d, 0xc0, 0x60, 0xe7, 0xa9, 0x19, 0x86, 0xdf, 0x98, 0x79, 0xa3, 0xfb, 0xc4,
            0x83, 0xa8, 0x98, 0xbd, 0xea, 0xb6, 0x80, 0x75, 0x60, 0x45, 0x99, 0x2f, 0x4b, 0x54,
            0x02, 0xb0, 0x52, 0xf2
        ])
        .unwrap()
    }
}

#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
impl PrimeFieldBits for Scalar {
    #[cfg(target_pointer_width = "32")]
    type ReprBits = [u32; 8];

    #[cfg(target_pointer_width = "64")]
    type ReprBits = [u64; 4];

    fn to_le_bits(&self) -> ScalarBits {
        self.into()
    }

    fn char_le_bits() -> ScalarBits {
        ORDER.to_words().into()
    }
}

impl DefaultIsZeroes for Scalar {}

impl From<u32> for Scalar {
    fn from(k: u32) -> Self {
        Self::from(k as u64)
    }
}

impl From<u64> for Scalar {
    fn from(k: u64) -> Self {
        Self(k.into())
    }
}

impl From<ScalarCore<Secp256k1>> for Scalar {
    fn from(scalar: ScalarCore<Secp256k1>) -> Scalar {
        Scalar(*scalar.as_uint())
    }
}

impl From<&ScalarCore<Secp256k1>> for Scalar {
    fn from(scalar: &ScalarCore<Secp256k1>) -> Scalar {
        Scalar(*scalar.as_uint())
    }
}

impl From<Scalar> for ScalarCore<Secp256k1> {
    fn from(scalar: Scalar) -> ScalarCore<Secp256k1> {
        ScalarCore::from(&scalar)
    }
}

impl From<&Scalar> for ScalarCore<Secp256k1> {
    fn from(scalar: &Scalar) -> ScalarCore<Secp256k1> {
        ScalarCore::new(scalar.0).unwrap()
    }
}

impl IsHigh for Scalar {
    fn is_high(&self) -> Choice {
        self.0.ct_gt(&FRAC_MODULUS_2)
    }
}

impl Shr<usize> for Scalar {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shr_vartime(rhs)
    }
}

impl Shr<usize> for &Scalar {
    type Output = Scalar;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shr_vartime(rhs)
    }
}

impl ConditionallySelectable for Scalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(U256::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&(other.0))
    }
}

impl PartialEq for Scalar {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for Scalar {}

impl Neg for Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        self.negate()
    }
}

impl Neg for &Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        self.negate()
    }
}

impl Add<Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        Scalar::add(&self, &other)
    }
}

impl Add<&Scalar> for &Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Scalar::add(self, other)
    }
}

impl Add<Scalar> for &Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        Scalar::add(self, &other)
    }
}

impl Add<&Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Scalar::add(&self, other)
    }
}

impl AddAssign<Scalar> for Scalar {
    fn add_assign(&mut self, rhs: Scalar) {
        *self = Scalar::add(self, &rhs);
    }
}

impl AddAssign<&Scalar> for Scalar {
    fn add_assign(&mut self, rhs: &Scalar) {
        *self = Scalar::add(self, rhs);
    }
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: Scalar) -> Scalar {
        Scalar::sub(&self, &other)
    }
}

impl Sub<&Scalar> for &Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Scalar::sub(self, other)
    }
}

impl Sub<&Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Scalar::sub(&self, other)
    }
}

impl SubAssign<Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: Scalar) {
        *self = Scalar::sub(self, &rhs);
    }
}

impl SubAssign<&Scalar> for Scalar {
    fn sub_assign(&mut self, rhs: &Scalar) {
        *self = Scalar::sub(self, rhs);
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, other: Scalar) -> Scalar {
        Scalar::mul(&self, &other)
    }
}

impl Mul<&Scalar> for &Scalar {
    type Output = Scalar;

    fn mul(self, other: &Scalar) -> Scalar {
        Scalar::mul(self, other)
    }
}

impl Mul<&Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, other: &Scalar) -> Scalar {
        Scalar::mul(&self, other)
    }
}

impl MulAssign<Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: Scalar) {
        *self = Scalar::mul(self, &rhs);
    }
}

impl MulAssign<&Scalar> for Scalar {
    fn mul_assign(&mut self, rhs: &Scalar) {
        *self = Scalar::mul(self, rhs);
    }
}

impl Reduce<U256> for Scalar {
    fn from_uint_reduced(w: U256) -> Self {
        let (r, underflow) = w.sbb(&ORDER, Limb::ZERO);
        let underflow = Choice::from((underflow.0 >> (Limb::BIT_SIZE - 1)) as u8);
        Self(U256::conditional_select(&w, &r, !underflow))
    }
}

impl Reduce<U512> for Scalar {
    fn from_uint_reduced(w: U512) -> Self {
        WideScalar(w).reduce()
    }
}

impl ReduceNonZero<U256> for Scalar {
    fn from_uint_reduced_nonzero(w: U256) -> Self {
        const ORDER_MINUS_ONE: U256 = ORDER.wrapping_sub(&U256::ONE);
        let (r, underflow) = w.sbb(&ORDER_MINUS_ONE, Limb::ZERO);
        let underflow = Choice::from((underflow.0 >> (Limb::BIT_SIZE - 1)) as u8);
        Self(U256::conditional_select(&w, &r, !underflow).wrapping_add(&U256::ONE))
    }
}

impl ReduceNonZero<U512> for Scalar {
    fn from_uint_reduced_nonzero(w: U512) -> Self {
        WideScalar(w).reduce_nonzero()
    }
}

#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
impl From<&Scalar> for ScalarBits {
    fn from(scalar: &Scalar) -> ScalarBits {
        scalar.0.to_words().into()
    }
}

impl From<Scalar> for FieldBytes {
    fn from(scalar: Scalar) -> Self {
        scalar.to_bytes()
    }
}

impl From<&Scalar> for FieldBytes {
    fn from(scalar: &Scalar) -> Self {
        scalar.to_bytes()
    }
}

impl From<Scalar> for U256 {
    fn from(scalar: Scalar) -> Self {
        scalar.0
    }
}

impl From<&Scalar> for U256 {
    fn from(scalar: &Scalar) -> Self {
        scalar.0
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        ScalarCore::from(self).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(ScalarCore::deserialize(deserializer)?.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Scalar;
    use crate::arithmetic::dev::{biguint_to_bytes, bytes_to_biguint};
    use crate::{NonZeroScalar, ORDER};
    use elliptic_curve::{
        bigint::{ArrayEncoding, U256, U512},
        ff::{Field, PrimeField},
        generic_array::GenericArray,
        ops::Reduce,
        IsHigh,
    };
    use num_bigint::{BigUint, ToBigUint};
    use num_traits::Zero;
    use proptest::prelude::*;

    impl From<&BigUint> for Scalar {
        fn from(x: &BigUint) -> Self {
            debug_assert!(x < &Scalar::modulus_as_biguint());
            let bytes = biguint_to_bytes(x);
            Self::from_repr(bytes.into()).unwrap()
        }
    }

    impl From<BigUint> for Scalar {
        fn from(x: BigUint) -> Self {
            Self::from(&x)
        }
    }

    impl ToBigUint for Scalar {
        fn to_biguint(&self) -> Option<BigUint> {
            Some(bytes_to_biguint(self.to_bytes().as_ref()))
        }
    }

    #[test]
    fn is_high() {
        // 0 is not high
        let high: bool = Scalar::zero().is_high().into();
        assert!(!high);

        // 1 is not high
        let one = 1.to_biguint().unwrap();
        let high: bool = Scalar::from(&one).is_high().into();
        assert!(!high);

        let m = Scalar::modulus_as_biguint();
        let m_by_2 = &m >> 1;

        // M / 2 is not high
        let high: bool = Scalar::from(&m_by_2).is_high().into();
        assert!(!high);

        // M / 2 + 1 is high
        let high: bool = Scalar::from(&m_by_2 + &one).is_high().into();
        assert!(high);

        // MODULUS - 1 is high
        let high: bool = Scalar::from(&m - &one).is_high().into();
        assert!(high);
    }

    /// Basic tests that sqrt works.
    #[test]
    fn sqrt() {
        for &n in &[1u64, 4, 9, 16, 25, 36, 49, 64] {
            let scalar = Scalar::from(n);
            let sqrt = scalar.sqrt().unwrap();
            assert_eq!(sqrt.square(), scalar);
        }
    }

    #[test]
    fn negate() {
        let zero_neg = -Scalar::zero();
        assert_eq!(zero_neg, Scalar::zero());

        let m = Scalar::modulus_as_biguint();
        let one = 1.to_biguint().unwrap();
        let m_minus_one = &m - &one;
        let m_by_2 = &m >> 1;

        let one_neg = -Scalar::one();
        assert_eq!(one_neg, Scalar::from(&m_minus_one));

        let frac_modulus_2_neg = -Scalar::from(&m_by_2);
        let frac_modulus_2_plus_one = Scalar::from(&m_by_2 + &one);
        assert_eq!(frac_modulus_2_neg, frac_modulus_2_plus_one);

        let modulus_minus_one_neg = -Scalar::from(&m - &one);
        assert_eq!(modulus_minus_one_neg, Scalar::one());
    }

    #[test]
    fn add_result_within_256_bits() {
        // A regression for a bug where reduction was not applied
        // when the unreduced result of addition was in the range `[modulus, 2^256)`.
        let t = 1.to_biguint().unwrap() << 255;
        let one = 1.to_biguint().unwrap();

        let a = Scalar::from(&t - &one);
        let b = Scalar::from(&t);
        let res = &a + &b;

        let m = Scalar::modulus_as_biguint();
        let res_ref = Scalar::from((&t + &t - &one) % &m);

        assert_eq!(res, res_ref);
    }

    #[test]
    fn generate_biased() {
        use elliptic_curve::rand_core::OsRng;
        let a = Scalar::generate_biased(&mut OsRng);
        // just to make sure `a` is not optimized out by the compiler
        assert_eq!((a - &a).is_zero().unwrap_u8(), 1);
    }

    #[test]
    fn generate_vartime() {
        use elliptic_curve::rand_core::OsRng;
        let a = Scalar::generate_vartime(&mut OsRng);
        // just to make sure `a` is not optimized out by the compiler
        assert_eq!((a - &a).is_zero().unwrap_u8(), 1);
    }

    #[test]
    fn from_bytes_reduced() {
        let m = Scalar::modulus_as_biguint();

        fn reduce<T: Reduce<U256>>(arr: &[u8]) -> T {
            T::from_be_bytes_reduced(GenericArray::clone_from_slice(arr))
        }

        // Regular reduction

        let s = reduce::<Scalar>(&[0xffu8; 32]).to_biguint().unwrap();
        assert!(s < m);

        let s = reduce::<Scalar>(&[0u8; 32]).to_biguint().unwrap();
        assert!(s.is_zero());

        let s = reduce::<Scalar>(&ORDER.to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s.is_zero());

        // Reduction to a non-zero scalar

        let s = reduce::<NonZeroScalar>(&[0xffu8; 32]).to_biguint().unwrap();
        assert!(s < m);

        let s = reduce::<NonZeroScalar>(&[0u8; 32]).to_biguint().unwrap();
        assert!(s < m);
        assert!(!s.is_zero());

        let s = reduce::<NonZeroScalar>(&ORDER.to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s < m);
        assert!(!s.is_zero());

        let s = reduce::<NonZeroScalar>(&(ORDER.wrapping_sub(&U256::ONE)).to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s < m);
        assert!(!s.is_zero());
    }

    #[test]
    fn from_wide_bytes_reduced() {
        let m = Scalar::modulus_as_biguint();

        fn reduce<T: Reduce<U512>>(arr: &[u8]) -> T {
            let mut full_arr = [0u8; 64];
            full_arr[(64 - arr.len())..64].copy_from_slice(arr);
            T::from_be_bytes_reduced(GenericArray::clone_from_slice(&full_arr))
        }

        // Regular reduction

        let s = reduce::<Scalar>(&[0xffu8; 64]).to_biguint().unwrap();
        assert!(s < m);

        let s = reduce::<Scalar>(&[0u8; 64]).to_biguint().unwrap();
        assert!(s.is_zero());

        let s = reduce::<Scalar>(&ORDER.to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s.is_zero());

        // Reduction to a non-zero scalar

        let s = reduce::<NonZeroScalar>(&[0xffu8; 64]).to_biguint().unwrap();
        assert!(s < m);

        let s = reduce::<NonZeroScalar>(&[0u8; 64]).to_biguint().unwrap();
        assert!(s < m);
        assert!(!s.is_zero());

        let s = reduce::<NonZeroScalar>(&ORDER.to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s < m);
        assert!(!s.is_zero());

        let s = reduce::<NonZeroScalar>(&(ORDER.wrapping_sub(&U256::ONE)).to_be_byte_array())
            .to_biguint()
            .unwrap();
        assert!(s < m);
        assert!(!s.is_zero());
    }

    prop_compose! {
        fn scalar()(bytes in any::<[u8; 32]>()) -> Scalar {
            let mut res = bytes_to_biguint(&bytes);
            let m = Scalar::modulus_as_biguint();
            // Modulus is 256 bit long, same as the maximum `res`,
            // so this is guaranteed to land us in the correct range.
            if res >= m {
                res -= m;
            }
            Scalar::from(&res)
        }
    }

    proptest! {
        #[test]
        fn fuzzy_roundtrip_to_bytes(a in scalar()) {
            let a_back = Scalar::from_repr(a.to_bytes()).unwrap();
            assert_eq!(a, a_back);
        }

        #[test]
        fn fuzzy_roundtrip_to_bytes_unchecked(a in scalar()) {
            let bytes = a.to_bytes();
            let a_back = Scalar::from_bytes_unchecked(bytes.as_ref());
            assert_eq!(a, a_back);
        }

        #[test]
        fn fuzzy_add(a in scalar(), b in scalar()) {
            let a_bi = a.to_biguint().unwrap();
            let b_bi = b.to_biguint().unwrap();

            let res_bi = (&a_bi + &b_bi) % &Scalar::modulus_as_biguint();
            let res_ref = Scalar::from(&res_bi);
            let res_test = a.add(&b);

            assert_eq!(res_ref, res_test);
        }

        #[test]
        fn fuzzy_sub(a in scalar(), b in scalar()) {
            let a_bi = a.to_biguint().unwrap();
            let b_bi = b.to_biguint().unwrap();

            let m = Scalar::modulus_as_biguint();
            let res_bi = (&m + &a_bi - &b_bi) % &m;
            let res_ref = Scalar::from(&res_bi);
            let res_test = a.sub(&b);

            assert_eq!(res_ref, res_test);
        }

        #[test]
        fn fuzzy_neg(a in scalar()) {
            let a_bi = a.to_biguint().unwrap();

            let m = Scalar::modulus_as_biguint();
            let res_bi = (&m - &a_bi) % &m;
            let res_ref = Scalar::from(&res_bi);
            let res_test = -a;

            assert_eq!(res_ref, res_test);
        }

        #[test]
        fn fuzzy_mul(a in scalar(), b in scalar()) {
            let a_bi = a.to_biguint().unwrap();
            let b_bi = b.to_biguint().unwrap();

            let res_bi = (&a_bi * &b_bi) % &Scalar::modulus_as_biguint();
            let res_ref = Scalar::from(&res_bi);
            let res_test = a.mul(&b);

            assert_eq!(res_ref, res_test);
        }

        #[test]
        fn fuzzy_rshift(a in scalar(), b in 0usize..512) {
            let a_bi = a.to_biguint().unwrap();

            let res_bi = &a_bi >> b;
            let res_ref = Scalar::from(&res_bi);
            let res_test = a >> b;

            assert_eq!(res_ref, res_test);
        }

        #[test]
        fn fuzzy_invert(
            a in scalar()
        ) {
            let a = if bool::from(a.is_zero()) { Scalar::one() } else { a };
            let a_bi = a.to_biguint().unwrap();
            let inv = a.invert().unwrap();
            let inv_bi = inv.to_biguint().unwrap();
            let m = Scalar::modulus_as_biguint();
            assert_eq!((&inv_bi * &a_bi) % &m, 1.to_biguint().unwrap());
        }

        #[test]
        fn fuzzy_from_wide_bytes_reduced(bytes_hi in any::<[u8; 32]>(), bytes_lo in any::<[u8; 32]>()) {
            let m = Scalar::modulus_as_biguint();
            let mut bytes = [0u8; 64];
            bytes[0..32].clone_from_slice(&bytes_hi);
            bytes[32..64].clone_from_slice(&bytes_lo);
            let s = <Scalar as Reduce<U512>>::from_be_bytes_reduced(GenericArray::clone_from_slice(&bytes));
            let s_bu = s.to_biguint().unwrap();
            assert!(s_bu < m);
        }
    }
}
