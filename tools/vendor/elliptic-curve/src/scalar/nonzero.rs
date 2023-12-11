//! Non-zero scalar type.

use crate::{
    ops::{Invert, Reduce, ReduceNonZero},
    scalar::IsHigh,
    CurveArithmetic, Error, FieldBytes, PrimeCurve, Scalar, ScalarPrimitive, SecretKey,
};
use base16ct::HexDisplay;
use core::{
    fmt,
    ops::{Deref, Mul, Neg},
    str,
};
use crypto_bigint::{ArrayEncoding, Integer};
use ff::{Field, PrimeField};
use generic_array::{typenum::Unsigned, GenericArray};
use rand_core::CryptoRngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};
use zeroize::Zeroize;

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

/// Non-zero scalar type.
///
/// This type ensures that its value is not zero, ala `core::num::NonZero*`.
/// To do this, the generic `S` type must impl both `Default` and
/// `ConstantTimeEq`, with the requirement that `S::default()` returns 0.
///
/// In the context of ECC, it's useful for ensuring that scalar multiplication
/// cannot result in the point at infinity.
#[derive(Clone)]
pub struct NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    scalar: Scalar<C>,
}

impl<C> NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    /// Generate a random `NonZeroScalar`.
    pub fn random(mut rng: &mut impl CryptoRngCore) -> Self {
        // Use rejection sampling to eliminate zero values.
        // While this method isn't constant-time, the attacker shouldn't learn
        // anything about unrelated outputs so long as `rng` is a secure `CryptoRng`.
        loop {
            if let Some(result) = Self::new(Field::random(&mut rng)).into() {
                break result;
            }
        }
    }

    /// Create a [`NonZeroScalar`] from a scalar.
    pub fn new(scalar: Scalar<C>) -> CtOption<Self> {
        CtOption::new(Self { scalar }, !scalar.is_zero())
    }

    /// Decode a [`NonZeroScalar`] from a big endian-serialized field element.
    pub fn from_repr(repr: FieldBytes<C>) -> CtOption<Self> {
        Scalar::<C>::from_repr(repr).and_then(Self::new)
    }

    /// Create a [`NonZeroScalar`] from a `C::Uint`.
    pub fn from_uint(uint: C::Uint) -> CtOption<Self> {
        ScalarPrimitive::new(uint).and_then(|scalar| Self::new(scalar.into()))
    }
}

impl<C> AsRef<Scalar<C>> for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn as_ref(&self) -> &Scalar<C> {
        &self.scalar
    }
}

impl<C> ConditionallySelectable for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self {
            scalar: Scalar::<C>::conditional_select(&a.scalar, &b.scalar, choice),
        }
    }
}

impl<C> ConstantTimeEq for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn ct_eq(&self, other: &Self) -> Choice {
        self.scalar.ct_eq(&other.scalar)
    }
}

impl<C> Copy for NonZeroScalar<C> where C: CurveArithmetic {}

impl<C> Deref for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    type Target = Scalar<C>;

    fn deref(&self) -> &Scalar<C> {
        &self.scalar
    }
}

impl<C> From<NonZeroScalar<C>> for FieldBytes<C>
where
    C: CurveArithmetic,
{
    fn from(scalar: NonZeroScalar<C>) -> FieldBytes<C> {
        Self::from(&scalar)
    }
}

impl<C> From<&NonZeroScalar<C>> for FieldBytes<C>
where
    C: CurveArithmetic,
{
    fn from(scalar: &NonZeroScalar<C>) -> FieldBytes<C> {
        scalar.to_repr()
    }
}

impl<C> From<NonZeroScalar<C>> for ScalarPrimitive<C>
where
    C: CurveArithmetic,
{
    #[inline]
    fn from(scalar: NonZeroScalar<C>) -> ScalarPrimitive<C> {
        Self::from(&scalar)
    }
}

impl<C> From<&NonZeroScalar<C>> for ScalarPrimitive<C>
where
    C: CurveArithmetic,
{
    fn from(scalar: &NonZeroScalar<C>) -> ScalarPrimitive<C> {
        ScalarPrimitive::from_bytes(&scalar.to_repr()).unwrap()
    }
}

impl<C> From<SecretKey<C>> for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn from(sk: SecretKey<C>) -> NonZeroScalar<C> {
        Self::from(&sk)
    }
}

impl<C> From<&SecretKey<C>> for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn from(sk: &SecretKey<C>) -> NonZeroScalar<C> {
        let scalar = sk.as_scalar_primitive().to_scalar();
        debug_assert!(!bool::from(scalar.is_zero()));
        Self { scalar }
    }
}

impl<C> Invert for NonZeroScalar<C>
where
    C: CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>>,
{
    type Output = Self;

    fn invert(&self) -> Self {
        Self {
            // This will always succeed since `scalar` will never be 0
            scalar: Invert::invert(&self.scalar).unwrap(),
        }
    }

    fn invert_vartime(&self) -> Self::Output {
        Self {
            // This will always succeed since `scalar` will never be 0
            scalar: Invert::invert_vartime(&self.scalar).unwrap(),
        }
    }
}

impl<C> IsHigh for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn is_high(&self) -> Choice {
        self.scalar.is_high()
    }
}

impl<C> Neg for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    type Output = NonZeroScalar<C>;

    fn neg(self) -> NonZeroScalar<C> {
        let scalar = -self.scalar;
        debug_assert!(!bool::from(scalar.is_zero()));
        NonZeroScalar { scalar }
    }
}

impl<C> Mul<NonZeroScalar<C>> for NonZeroScalar<C>
where
    C: PrimeCurve + CurveArithmetic,
{
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        Self::mul(self, &other)
    }
}

impl<C> Mul<&NonZeroScalar<C>> for NonZeroScalar<C>
where
    C: PrimeCurve + CurveArithmetic,
{
    type Output = Self;

    fn mul(self, other: &Self) -> Self {
        // Multiplication is modulo a prime, so the product of two non-zero
        // scalars is also non-zero.
        let scalar = self.scalar * other.scalar;
        debug_assert!(!bool::from(scalar.is_zero()));
        NonZeroScalar { scalar }
    }
}

/// Note: this is a non-zero reduction, as it's impl'd for [`NonZeroScalar`].
impl<C, I> Reduce<I> for NonZeroScalar<C>
where
    C: CurveArithmetic,
    I: Integer + ArrayEncoding,
    Scalar<C>: Reduce<I> + ReduceNonZero<I>,
{
    type Bytes = <Scalar<C> as Reduce<I>>::Bytes;

    fn reduce(n: I) -> Self {
        let scalar = Scalar::<C>::reduce_nonzero(n);
        debug_assert!(!bool::from(scalar.is_zero()));
        Self { scalar }
    }

    fn reduce_bytes(bytes: &Self::Bytes) -> Self {
        let scalar = Scalar::<C>::reduce_nonzero_bytes(bytes);
        debug_assert!(!bool::from(scalar.is_zero()));
        Self { scalar }
    }
}

/// Note: forwards to the [`Reduce`] impl.
impl<C, I> ReduceNonZero<I> for NonZeroScalar<C>
where
    Self: Reduce<I>,
    C: CurveArithmetic,
    I: Integer + ArrayEncoding,
    Scalar<C>: Reduce<I, Bytes = Self::Bytes> + ReduceNonZero<I>,
{
    fn reduce_nonzero(n: I) -> Self {
        Self::reduce(n)
    }

    fn reduce_nonzero_bytes(bytes: &Self::Bytes) -> Self {
        Self::reduce_bytes(bytes)
    }
}

impl<C> TryFrom<&[u8]> for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() == C::FieldBytesSize::USIZE {
            Option::from(NonZeroScalar::from_repr(GenericArray::clone_from_slice(
                bytes,
            )))
            .ok_or(Error)
        } else {
            Err(Error)
        }
    }
}

impl<C> Zeroize for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn zeroize(&mut self) {
        // Use zeroize's volatile writes to ensure value is cleared.
        self.scalar.zeroize();

        // Write a 1 instead of a 0 to ensure this type's non-zero invariant
        // is upheld.
        self.scalar = Scalar::<C>::ONE;
    }
}

impl<C> fmt::Display for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:X}")
    }
}

impl<C> fmt::LowerHex for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", HexDisplay(&self.to_repr()))
    }
}

impl<C> fmt::UpperHex for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:}", HexDisplay(&self.to_repr()))
    }
}

impl<C> str::FromStr for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    type Err = Error;

    fn from_str(hex: &str) -> Result<Self, Error> {
        let mut bytes = FieldBytes::<C>::default();

        if base16ct::mixed::decode(hex, &mut bytes)?.len() == bytes.len() {
            Option::from(Self::from_repr(bytes)).ok_or(Error)
        } else {
            Err(Error)
        }
    }
}

#[cfg(feature = "serde")]
impl<C> Serialize for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        ScalarPrimitive::from(self).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, C> Deserialize<'de> for NonZeroScalar<C>
where
    C: CurveArithmetic,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let scalar = ScalarPrimitive::deserialize(deserializer)?;
        Option::from(Self::new(scalar.into()))
            .ok_or_else(|| de::Error::custom("expected non-zero scalar"))
    }
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::dev::{NonZeroScalar, Scalar};
    use ff::{Field, PrimeField};
    use hex_literal::hex;
    use zeroize::Zeroize;

    #[test]
    fn round_trip() {
        let bytes = hex!("c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721");
        let scalar = NonZeroScalar::from_repr(bytes.into()).unwrap();
        assert_eq!(&bytes, scalar.to_repr().as_slice());
    }

    #[test]
    fn zeroize() {
        let mut scalar = NonZeroScalar::new(Scalar::from(42u64)).unwrap();
        scalar.zeroize();
        assert_eq!(*scalar, Scalar::ONE);
    }
}
