//! Development-related functionality.
//!
//! Helpers and types for writing tests against concrete implementations of
//! the traits in this crate.

use crate::{
    bigint::{Limb, U256},
    error::{Error, Result},
    generic_array::typenum::U32,
    ops::{Invert, LinearCombination, MulByGenerator, Reduce, ShrAssign},
    pkcs8,
    point::AffineCoordinates,
    rand_core::RngCore,
    scalar::{FromUintUnchecked, IsHigh},
    sec1::{CompressedPoint, FromEncodedPoint, ToEncodedPoint},
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption},
    zeroize::DefaultIsZeroes,
    Curve, CurveArithmetic, FieldBytesEncoding, PrimeCurve,
};
use core::{
    iter::{Product, Sum},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};
use ff::{Field, PrimeField};
use hex_literal::hex;
use pkcs8::AssociatedOid;

#[cfg(feature = "bits")]
use ff::PrimeFieldBits;

#[cfg(feature = "jwk")]
use crate::JwkParameters;

/// Pseudo-coordinate for fixed-based scalar mult output
pub const PSEUDO_COORDINATE_FIXED_BASE_MUL: [u8; 32] =
    hex!("deadbeef00000000000000000000000000000000000000000000000000000001");

/// SEC1 encoded point.
pub type EncodedPoint = crate::sec1::EncodedPoint<MockCurve>;

/// Field element bytes.
pub type FieldBytes = crate::FieldBytes<MockCurve>;

/// Non-zero scalar value.
pub type NonZeroScalar = crate::NonZeroScalar<MockCurve>;

/// Public key.
pub type PublicKey = crate::PublicKey<MockCurve>;

/// Secret key.
pub type SecretKey = crate::SecretKey<MockCurve>;

/// Scalar primitive type.
// TODO(tarcieri): make this the scalar type when it's more capable
pub type ScalarPrimitive = crate::ScalarPrimitive<MockCurve>;

/// Scalar bits.
#[cfg(feature = "bits")]
pub type ScalarBits = crate::scalar::ScalarBits<MockCurve>;

/// Mock elliptic curve type useful for writing tests which require a concrete
/// curve type.
///
/// Note: this type is roughly modeled off of NIST P-256, but does not provide
/// an actual cure arithmetic implementation.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct MockCurve;

impl Curve for MockCurve {
    type FieldBytesSize = U32;
    type Uint = U256;

    const ORDER: U256 =
        U256::from_be_hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551");
}

impl PrimeCurve for MockCurve {}

impl CurveArithmetic for MockCurve {
    type AffinePoint = AffinePoint;
    type ProjectivePoint = ProjectivePoint;
    type Scalar = Scalar;
}

impl AssociatedOid for MockCurve {
    /// OID for NIST P-256
    const OID: pkcs8::ObjectIdentifier = pkcs8::ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");
}

#[cfg(feature = "jwk")]
impl JwkParameters for MockCurve {
    const CRV: &'static str = "P-256";
}

/// Example scalar type
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Scalar(ScalarPrimitive);

impl Field for Scalar {
    const ZERO: Self = Self(ScalarPrimitive::ZERO);
    const ONE: Self = Self(ScalarPrimitive::ONE);

    fn random(mut rng: impl RngCore) -> Self {
        let mut bytes = FieldBytes::default();

        loop {
            rng.fill_bytes(&mut bytes);
            if let Some(scalar) = Self::from_repr(bytes).into() {
                return scalar;
            }
        }
    }

    fn is_zero(&self) -> Choice {
        self.0.is_zero()
    }

    #[must_use]
    fn square(&self) -> Self {
        unimplemented!();
    }

    #[must_use]
    fn double(&self) -> Self {
        self.add(self)
    }

    fn invert(&self) -> CtOption<Self> {
        unimplemented!();
    }

    fn sqrt(&self) -> CtOption<Self> {
        unimplemented!();
    }

    fn sqrt_ratio(_num: &Self, _div: &Self) -> (Choice, Self) {
        unimplemented!();
    }
}

impl PrimeField for Scalar {
    type Repr = FieldBytes;

    const MODULUS: &'static str =
        "0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff";
    const NUM_BITS: u32 = 256;
    const CAPACITY: u32 = 255;
    const TWO_INV: Self = Self::ZERO; // BOGUS!
    const MULTIPLICATIVE_GENERATOR: Self = Self::ZERO; // BOGUS! Should be 7
    const S: u32 = 4;
    const ROOT_OF_UNITY: Self = Self::ZERO; // BOGUS! Should be 0xffc97f062a770992ba807ace842a3dfc1546cad004378daf0592d7fbb41e6602
    const ROOT_OF_UNITY_INV: Self = Self::ZERO; // BOGUS!
    const DELTA: Self = Self::ZERO; // BOGUS!

    fn from_repr(bytes: FieldBytes) -> CtOption<Self> {
        ScalarPrimitive::from_bytes(&bytes).map(Self)
    }

    fn to_repr(&self) -> FieldBytes {
        self.0.to_bytes()
    }

    fn is_odd(&self) -> Choice {
        self.0.is_odd()
    }
}

#[cfg(feature = "bits")]
impl PrimeFieldBits for Scalar {
    #[cfg(target_pointer_width = "32")]
    type ReprBits = [u32; 8];

    #[cfg(target_pointer_width = "64")]
    type ReprBits = [u64; 4];

    fn to_le_bits(&self) -> ScalarBits {
        self.0.as_uint().to_words().into()
    }

    fn char_le_bits() -> ScalarBits {
        MockCurve::ORDER.to_words().into()
    }
}

impl AsRef<Scalar> for Scalar {
    fn as_ref(&self) -> &Scalar {
        self
    }
}

impl ConditionallySelectable for Scalar {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self(ScalarPrimitive::conditional_select(&a.0, &b.0, choice))
    }
}

impl ConstantTimeEq for Scalar {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.0.ct_eq(&other.0)
    }
}

impl DefaultIsZeroes for Scalar {}

impl Add<Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        self.add(&other)
    }
}

impl Add<&Scalar> for Scalar {
    type Output = Scalar;

    fn add(self, other: &Scalar) -> Scalar {
        Self(self.0.add(&other.0))
    }
}

impl AddAssign<Scalar> for Scalar {
    fn add_assign(&mut self, other: Scalar) {
        *self = *self + other;
    }
}

impl AddAssign<&Scalar> for Scalar {
    fn add_assign(&mut self, other: &Scalar) {
        *self = *self + other;
    }
}

impl Sub<Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: Scalar) -> Scalar {
        self.sub(&other)
    }
}

impl Sub<&Scalar> for Scalar {
    type Output = Scalar;

    fn sub(self, other: &Scalar) -> Scalar {
        Self(self.0.sub(&other.0))
    }
}

impl SubAssign<Scalar> for Scalar {
    fn sub_assign(&mut self, other: Scalar) {
        *self = *self - other;
    }
}

impl SubAssign<&Scalar> for Scalar {
    fn sub_assign(&mut self, other: &Scalar) {
        *self = *self - other;
    }
}

impl Mul<Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, _other: Scalar) -> Scalar {
        unimplemented!();
    }
}

impl Mul<&Scalar> for Scalar {
    type Output = Scalar;

    fn mul(self, _other: &Scalar) -> Scalar {
        unimplemented!();
    }
}

impl MulAssign<Scalar> for Scalar {
    fn mul_assign(&mut self, _rhs: Scalar) {
        unimplemented!();
    }
}

impl MulAssign<&Scalar> for Scalar {
    fn mul_assign(&mut self, _rhs: &Scalar) {
        unimplemented!();
    }
}

impl Neg for Scalar {
    type Output = Scalar;

    fn neg(self) -> Scalar {
        Self(self.0.neg())
    }
}

impl ShrAssign<usize> for Scalar {
    fn shr_assign(&mut self, rhs: usize) {
        self.0 >>= rhs;
    }
}

impl Sum for Scalar {
    fn sum<I: Iterator<Item = Self>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl<'a> Sum<&'a Scalar> for Scalar {
    fn sum<I: Iterator<Item = &'a Scalar>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl Product for Scalar {
    fn product<I: Iterator<Item = Self>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl<'a> Product<&'a Scalar> for Scalar {
    fn product<I: Iterator<Item = &'a Scalar>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl Invert for Scalar {
    type Output = CtOption<Scalar>;

    fn invert(&self) -> CtOption<Scalar> {
        unimplemented!();
    }
}

impl Reduce<U256> for Scalar {
    type Bytes = FieldBytes;

    #[allow(clippy::integer_arithmetic)]
    fn reduce(w: U256) -> Self {
        let (r, underflow) = w.sbb(&MockCurve::ORDER, Limb::ZERO);
        let underflow = Choice::from((underflow.0 >> (Limb::BITS - 1)) as u8);
        let reduced = U256::conditional_select(&w, &r, !underflow);
        Self(ScalarPrimitive::new(reduced).unwrap())
    }

    fn reduce_bytes(_: &FieldBytes) -> Self {
        todo!()
    }
}

impl FieldBytesEncoding<MockCurve> for U256 {}

impl From<u64> for Scalar {
    fn from(n: u64) -> Scalar {
        Self(n.into())
    }
}

impl From<ScalarPrimitive> for Scalar {
    fn from(scalar: ScalarPrimitive) -> Scalar {
        Self(scalar)
    }
}

impl From<Scalar> for ScalarPrimitive {
    fn from(scalar: Scalar) -> ScalarPrimitive {
        scalar.0
    }
}

impl From<Scalar> for U256 {
    fn from(scalar: Scalar) -> U256 {
        scalar.0.to_uint()
    }
}

impl TryFrom<U256> for Scalar {
    type Error = Error;

    fn try_from(w: U256) -> Result<Self> {
        Option::from(ScalarPrimitive::new(w)).map(Self).ok_or(Error)
    }
}

impl FromUintUnchecked for Scalar {
    type Uint = U256;

    fn from_uint_unchecked(uint: U256) -> Self {
        Self(ScalarPrimitive::from_uint_unchecked(uint))
    }
}

impl From<Scalar> for FieldBytes {
    fn from(scalar: Scalar) -> Self {
        Self::from(&scalar)
    }
}

impl From<&Scalar> for FieldBytes {
    fn from(scalar: &Scalar) -> Self {
        scalar.to_repr()
    }
}

impl IsHigh for Scalar {
    fn is_high(&self) -> Choice {
        self.0.is_high()
    }
}

/// Example affine point type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AffinePoint {
    /// Result of fixed-based scalar multiplication.
    FixedBaseOutput(Scalar),

    /// Identity.
    Identity,

    /// Base point.
    Generator,

    /// Point corresponding to a given [`EncodedPoint`].
    Other(EncodedPoint),
}

impl AffineCoordinates for AffinePoint {
    type FieldRepr = FieldBytes;

    fn x(&self) -> FieldBytes {
        unimplemented!();
    }

    fn y_is_odd(&self) -> Choice {
        unimplemented!();
    }
}

impl ConstantTimeEq for AffinePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        match (self, other) {
            (Self::FixedBaseOutput(scalar), Self::FixedBaseOutput(other_scalar)) => {
                scalar.ct_eq(other_scalar)
            }
            (Self::Identity, Self::Identity) | (Self::Generator, Self::Generator) => 1.into(),
            (Self::Other(point), Self::Other(other_point)) => u8::from(point == other_point).into(),
            _ => 0.into(),
        }
    }
}

impl ConditionallySelectable for AffinePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        // Not really constant time, but this is dev code
        if choice.into() {
            *b
        } else {
            *a
        }
    }
}

impl Default for AffinePoint {
    fn default() -> Self {
        Self::Identity
    }
}

impl DefaultIsZeroes for AffinePoint {}

impl FromEncodedPoint<MockCurve> for AffinePoint {
    fn from_encoded_point(encoded_point: &EncodedPoint) -> CtOption<Self> {
        let point = if encoded_point.is_identity() {
            Self::Identity
        } else {
            Self::Other(*encoded_point)
        };

        CtOption::new(point, Choice::from(1))
    }
}

impl ToEncodedPoint<MockCurve> for AffinePoint {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint {
        match self {
            Self::FixedBaseOutput(scalar) => EncodedPoint::from_affine_coordinates(
                &scalar.to_repr(),
                &PSEUDO_COORDINATE_FIXED_BASE_MUL.into(),
                false,
            ),
            Self::Other(point) => {
                if compress == point.is_compressed() {
                    *point
                } else {
                    unimplemented!();
                }
            }
            _ => unimplemented!(),
        }
    }
}

impl Mul<NonZeroScalar> for AffinePoint {
    type Output = AffinePoint;

    fn mul(self, _scalar: NonZeroScalar) -> Self {
        unimplemented!();
    }
}

/// Example projective point type
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectivePoint {
    /// Result of fixed-based scalar multiplication
    FixedBaseOutput(Scalar),

    /// Is this point the identity point?
    Identity,

    /// Is this point the generator point?
    Generator,

    /// Is this point a different point corresponding to a given [`AffinePoint`]
    Other(AffinePoint),
}

impl ConstantTimeEq for ProjectivePoint {
    fn ct_eq(&self, other: &Self) -> Choice {
        match (self, other) {
            (Self::FixedBaseOutput(scalar), Self::FixedBaseOutput(other_scalar)) => {
                scalar.ct_eq(other_scalar)
            }
            (Self::Identity, Self::Identity) | (Self::Generator, Self::Generator) => 1.into(),
            (Self::Other(point), Self::Other(other_point)) => point.ct_eq(other_point),
            _ => 0.into(),
        }
    }
}

impl ConditionallySelectable for ProjectivePoint {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        if choice.into() {
            *b
        } else {
            *a
        }
    }
}

impl Default for ProjectivePoint {
    fn default() -> Self {
        Self::Identity
    }
}

impl DefaultIsZeroes for ProjectivePoint {}

impl From<AffinePoint> for ProjectivePoint {
    fn from(point: AffinePoint) -> ProjectivePoint {
        match point {
            AffinePoint::FixedBaseOutput(scalar) => ProjectivePoint::FixedBaseOutput(scalar),
            AffinePoint::Identity => ProjectivePoint::Identity,
            AffinePoint::Generator => ProjectivePoint::Generator,
            other => ProjectivePoint::Other(other),
        }
    }
}

impl From<ProjectivePoint> for AffinePoint {
    fn from(point: ProjectivePoint) -> AffinePoint {
        group::Curve::to_affine(&point)
    }
}

impl FromEncodedPoint<MockCurve> for ProjectivePoint {
    fn from_encoded_point(_point: &EncodedPoint) -> CtOption<Self> {
        unimplemented!();
    }
}

impl ToEncodedPoint<MockCurve> for ProjectivePoint {
    fn to_encoded_point(&self, _compress: bool) -> EncodedPoint {
        unimplemented!();
    }
}

impl group::Group for ProjectivePoint {
    type Scalar = Scalar;

    fn random(_rng: impl RngCore) -> Self {
        unimplemented!();
    }

    fn identity() -> Self {
        Self::Identity
    }

    fn generator() -> Self {
        Self::Generator
    }

    fn is_identity(&self) -> Choice {
        Choice::from(u8::from(self == &Self::Identity))
    }

    #[must_use]
    fn double(&self) -> Self {
        unimplemented!();
    }
}

impl group::GroupEncoding for AffinePoint {
    type Repr = CompressedPoint<MockCurve>;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        EncodedPoint::from_bytes(bytes)
            .map(|point| CtOption::new(point, Choice::from(1)))
            .unwrap_or_else(|_| {
                let is_identity = bytes.ct_eq(&Self::Repr::default());
                CtOption::new(EncodedPoint::identity(), is_identity)
            })
            .and_then(|point| Self::from_encoded_point(&point))
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        Self::from_bytes(bytes)
    }

    fn to_bytes(&self) -> Self::Repr {
        let encoded = self.to_encoded_point(true);
        let mut result = CompressedPoint::<MockCurve>::default();
        result[..encoded.len()].copy_from_slice(encoded.as_bytes());
        result
    }
}

impl group::GroupEncoding for ProjectivePoint {
    type Repr = CompressedPoint<MockCurve>;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        <AffinePoint as group::GroupEncoding>::from_bytes(bytes).map(Into::into)
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        Self::from_bytes(bytes)
    }

    fn to_bytes(&self) -> Self::Repr {
        group::Curve::to_affine(self).to_bytes()
    }
}

impl group::Curve for ProjectivePoint {
    type AffineRepr = AffinePoint;

    fn to_affine(&self) -> AffinePoint {
        match self {
            Self::FixedBaseOutput(scalar) => AffinePoint::FixedBaseOutput(*scalar),
            Self::Other(affine) => *affine,
            _ => unimplemented!(),
        }
    }
}

impl LinearCombination for ProjectivePoint {}

impl Add<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, _other: ProjectivePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl Add<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, _other: &ProjectivePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl AddAssign<ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, _rhs: ProjectivePoint) {
        unimplemented!();
    }
}

impl AddAssign<&ProjectivePoint> for ProjectivePoint {
    fn add_assign(&mut self, _rhs: &ProjectivePoint) {
        unimplemented!();
    }
}

impl Sub<ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, _other: ProjectivePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl Sub<&ProjectivePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, _other: &ProjectivePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl SubAssign<ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, _rhs: ProjectivePoint) {
        unimplemented!();
    }
}

impl SubAssign<&ProjectivePoint> for ProjectivePoint {
    fn sub_assign(&mut self, _rhs: &ProjectivePoint) {
        unimplemented!();
    }
}

impl Add<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, _other: AffinePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl Add<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn add(self, _other: &AffinePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl AddAssign<AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, _rhs: AffinePoint) {
        unimplemented!();
    }
}

impl AddAssign<&AffinePoint> for ProjectivePoint {
    fn add_assign(&mut self, _rhs: &AffinePoint) {
        unimplemented!();
    }
}

impl Sum for ProjectivePoint {
    fn sum<I: Iterator<Item = Self>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl<'a> Sum<&'a ProjectivePoint> for ProjectivePoint {
    fn sum<I: Iterator<Item = &'a ProjectivePoint>>(_iter: I) -> Self {
        unimplemented!();
    }
}

impl Sub<AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, _other: AffinePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl Sub<&AffinePoint> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn sub(self, _other: &AffinePoint) -> ProjectivePoint {
        unimplemented!();
    }
}

impl SubAssign<AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, _rhs: AffinePoint) {
        unimplemented!();
    }
}

impl SubAssign<&AffinePoint> for ProjectivePoint {
    fn sub_assign(&mut self, _rhs: &AffinePoint) {
        unimplemented!();
    }
}

impl Mul<Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, scalar: Scalar) -> ProjectivePoint {
        match self {
            Self::Generator => Self::FixedBaseOutput(scalar),
            _ => unimplemented!(),
        }
    }
}

impl Mul<&Scalar> for ProjectivePoint {
    type Output = ProjectivePoint;

    fn mul(self, scalar: &Scalar) -> ProjectivePoint {
        self * *scalar
    }
}

impl MulAssign<Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, _rhs: Scalar) {
        unimplemented!();
    }
}

impl MulAssign<&Scalar> for ProjectivePoint {
    fn mul_assign(&mut self, _rhs: &Scalar) {
        unimplemented!();
    }
}

impl MulByGenerator for ProjectivePoint {}

impl Neg for ProjectivePoint {
    type Output = ProjectivePoint;

    fn neg(self) -> ProjectivePoint {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::Scalar;
    use ff::PrimeField;
    use hex_literal::hex;

    #[test]
    fn round_trip() {
        let bytes = hex!("c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721");
        let scalar = Scalar::from_repr(bytes.into()).unwrap();
        assert_eq!(&bytes, scalar.to_repr().as_slice());
    }
}
