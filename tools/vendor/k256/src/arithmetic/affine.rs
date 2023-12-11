//! Affine points

#![allow(clippy::op_ref)]

use super::{FieldElement, ProjectivePoint, CURVE_EQUATION_B};
use crate::{CompressedPoint, EncodedPoint, FieldBytes, PublicKey, Scalar, Secp256k1};
use core::ops::{Mul, Neg};
use elliptic_curve::{
    group::{prime::PrimeCurveAffine, GroupEncoding},
    point::{AffineCoordinates, DecompactPoint, DecompressPoint},
    sec1::{self, FromEncodedPoint, ToEncodedPoint},
    subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption},
    zeroize::DefaultIsZeroes,
    Error, Result,
};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

/// secp256k1 curve point expressed in affine coordinates.
///
/// # `serde` support
///
/// When the `serde` feature of this crate is enabled, the `Serialize` and
/// `Deserialize` traits are impl'd for this type.
///
/// The serialization uses the [SEC1] `Elliptic-Curve-Point-to-Octet-String`
/// encoding, serialized as binary.
///
/// When serialized with a text-based format, the SEC1 representation is
/// subsequently hex encoded.
///
/// [SEC1]: https://www.secg.org/sec1-v2.pdf
#[derive(Clone, Copy, Debug)]
pub struct AffinePoint {
    /// x-coordinate
    pub(crate) x: FieldElement,

    /// y-coordinate
    pub(crate) y: FieldElement,

    /// Is this point the point at infinity? 0 = no, 1 = yes
    ///
    /// This is a proxy for [`Choice`], but uses `u8` instead to permit `const`
    /// constructors for `IDENTITY` and `GENERATOR`.
    pub(super) infinity: u8,
}

impl AffinePoint {
    /// Additive identity of the group: the point at infinity.
    pub const IDENTITY: Self = Self {
        x: FieldElement::ZERO,
        y: FieldElement::ZERO,
        infinity: 1,
    };

    /// Base point of secp256k1.
    ///
    /// ```text
    /// Gₓ = 79be667e f9dcbbac 55a06295 ce870b07 029bfcdb 2dce28d9 59f2815b 16f81798
    /// Gᵧ = 483ada77 26a3c465 5da4fbfc 0e1108a8 fd17b448 a6855419 9c47d08f fb10d4b8
    /// ```
    pub const GENERATOR: Self = Self {
        x: FieldElement::from_bytes_unchecked(&[
            0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62, 0x95, 0xce, 0x87,
            0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce, 0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b,
            0x16, 0xf8, 0x17, 0x98,
        ]),
        y: FieldElement::from_bytes_unchecked(&[
            0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb, 0xfc, 0x0e, 0x11,
            0x08, 0xa8, 0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85, 0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f,
            0xfb, 0x10, 0xd4, 0xb8,
        ]),
        infinity: 0,
    };
}

impl AffinePoint {
    /// Create a new [`AffinePoint`] with the given coordinates.
    pub(crate) const fn new(x: FieldElement, y: FieldElement) -> Self {
        Self { x, y, infinity: 0 }
    }
}

impl PrimeCurveAffine for AffinePoint {
    type Scalar = Scalar;
    type Curve = ProjectivePoint;

    /// Returns the identity of the group: the point at infinity.
    fn identity() -> Self {
        Self::IDENTITY
    }

    /// Returns the base point of secp256k1.
    fn generator() -> Self {
        Self::GENERATOR
    }

    /// Is this point the identity point?
    fn is_identity(&self) -> Choice {
        Choice::from(self.infinity)
    }

    /// Convert to curve representation.
    fn to_curve(&self) -> ProjectivePoint {
        ProjectivePoint::from(*self)
    }
}

impl AffineCoordinates for AffinePoint {
    type FieldRepr = FieldBytes;

    fn x(&self) -> FieldBytes {
        self.x.to_bytes()
    }

    fn y_is_odd(&self) -> Choice {
        self.y.normalize().is_odd()
    }
}

impl ConditionallySelectable for AffinePoint {
    fn conditional_select(a: &AffinePoint, b: &AffinePoint, choice: Choice) -> AffinePoint {
        AffinePoint {
            x: FieldElement::conditional_select(&a.x, &b.x, choice),
            y: FieldElement::conditional_select(&a.y, &b.y, choice),
            infinity: u8::conditional_select(&a.infinity, &b.infinity, choice),
        }
    }
}

impl ConstantTimeEq for AffinePoint {
    fn ct_eq(&self, other: &AffinePoint) -> Choice {
        (self.x.negate(1) + &other.x).normalizes_to_zero()
            & (self.y.negate(1) + &other.y).normalizes_to_zero()
            & self.infinity.ct_eq(&other.infinity)
    }
}

impl Default for AffinePoint {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl DefaultIsZeroes for AffinePoint {}

impl PartialEq for AffinePoint {
    fn eq(&self, other: &AffinePoint) -> bool {
        self.ct_eq(other).into()
    }
}

impl Eq for AffinePoint {}

impl Mul<Scalar> for AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, scalar: Scalar) -> ProjectivePoint {
        ProjectivePoint::from(self) * scalar
    }
}

impl Mul<&Scalar> for AffinePoint {
    type Output = ProjectivePoint;

    fn mul(self, scalar: &Scalar) -> ProjectivePoint {
        ProjectivePoint::from(self) * scalar
    }
}

impl Neg for AffinePoint {
    type Output = AffinePoint;

    fn neg(self) -> Self::Output {
        AffinePoint {
            x: self.x,
            y: self.y.negate(1).normalize_weak(),
            infinity: self.infinity,
        }
    }
}

impl DecompressPoint<Secp256k1> for AffinePoint {
    fn decompress(x_bytes: &FieldBytes, y_is_odd: Choice) -> CtOption<Self> {
        FieldElement::from_bytes(x_bytes).and_then(|x| {
            let alpha = (x * &x * &x) + &CURVE_EQUATION_B;
            let beta = alpha.sqrt();

            beta.map(|beta| {
                let beta = beta.normalize(); // Need to normalize for is_odd() to be consistent
                let y = FieldElement::conditional_select(
                    &beta.negate(1),
                    &beta,
                    beta.is_odd().ct_eq(&y_is_odd),
                );

                Self::new(x, y.normalize())
            })
        })
    }
}

/// Decompaction using Taproot conventions as described in [BIP 340].
///
/// [BIP 340]: https://github.com/bitcoin/bips/blob/master/bip-0340.mediawiki
impl DecompactPoint<Secp256k1> for AffinePoint {
    fn decompact(x_bytes: &FieldBytes) -> CtOption<Self> {
        Self::decompress(x_bytes, Choice::from(0))
    }
}

impl GroupEncoding for AffinePoint {
    type Repr = CompressedPoint;

    fn from_bytes(bytes: &Self::Repr) -> CtOption<Self> {
        EncodedPoint::from_bytes(bytes)
            .map(|point| CtOption::new(point, Choice::from(1)))
            .unwrap_or_else(|_| {
                // SEC1 identity encoding is technically 1-byte 0x00, but the
                // `GroupEncoding` API requires a fixed-width `Repr`
                let is_identity = bytes.ct_eq(&Self::Repr::default());
                CtOption::new(EncodedPoint::identity(), is_identity)
            })
            .and_then(|point| Self::from_encoded_point(&point))
    }

    fn from_bytes_unchecked(bytes: &Self::Repr) -> CtOption<Self> {
        // No unchecked conversion possible for compressed points
        Self::from_bytes(bytes)
    }

    fn to_bytes(&self) -> Self::Repr {
        let encoded = self.to_encoded_point(true);
        let mut result = CompressedPoint::default();
        result[..encoded.len()].copy_from_slice(encoded.as_bytes());
        result
    }
}

impl FromEncodedPoint<Secp256k1> for AffinePoint {
    /// Attempts to parse the given [`EncodedPoint`] as an SEC1-encoded [`AffinePoint`].
    ///
    /// # Returns
    ///
    /// `None` value if `encoded_point` is not on the secp256k1 curve.
    fn from_encoded_point(encoded_point: &EncodedPoint) -> CtOption<Self> {
        match encoded_point.coordinates() {
            sec1::Coordinates::Identity => CtOption::new(Self::IDENTITY, 1.into()),
            sec1::Coordinates::Compact { x } => Self::decompact(x),
            sec1::Coordinates::Compressed { x, y_is_odd } => {
                AffinePoint::decompress(x, Choice::from(y_is_odd as u8))
            }
            sec1::Coordinates::Uncompressed { x, y } => {
                let x = FieldElement::from_bytes(x);
                let y = FieldElement::from_bytes(y);

                x.and_then(|x| {
                    y.and_then(|y| {
                        // Check that the point is on the curve
                        let lhs = (y * &y).negate(1);
                        let rhs = x * &x * &x + &CURVE_EQUATION_B;
                        let point = Self::new(x, y);
                        CtOption::new(point, (lhs + &rhs).normalizes_to_zero())
                    })
                })
            }
        }
    }
}

impl ToEncodedPoint<Secp256k1> for AffinePoint {
    fn to_encoded_point(&self, compress: bool) -> EncodedPoint {
        EncodedPoint::conditional_select(
            &EncodedPoint::from_affine_coordinates(
                &self.x.to_bytes(),
                &self.y.to_bytes(),
                compress,
            ),
            &EncodedPoint::identity(),
            self.is_identity(),
        )
    }
}

impl TryFrom<EncodedPoint> for AffinePoint {
    type Error = Error;

    fn try_from(point: EncodedPoint) -> Result<AffinePoint> {
        AffinePoint::try_from(&point)
    }
}

impl TryFrom<&EncodedPoint> for AffinePoint {
    type Error = Error;

    fn try_from(point: &EncodedPoint) -> Result<AffinePoint> {
        Option::from(AffinePoint::from_encoded_point(point)).ok_or(Error)
    }
}

impl From<AffinePoint> for EncodedPoint {
    fn from(affine_point: AffinePoint) -> EncodedPoint {
        EncodedPoint::from(&affine_point)
    }
}

impl From<&AffinePoint> for EncodedPoint {
    fn from(affine_point: &AffinePoint) -> EncodedPoint {
        affine_point.to_encoded_point(true)
    }
}

impl From<PublicKey> for AffinePoint {
    fn from(public_key: PublicKey) -> AffinePoint {
        *public_key.as_affine()
    }
}

impl From<&PublicKey> for AffinePoint {
    fn from(public_key: &PublicKey) -> AffinePoint {
        AffinePoint::from(*public_key)
    }
}

impl TryFrom<AffinePoint> for PublicKey {
    type Error = Error;

    fn try_from(affine_point: AffinePoint) -> Result<PublicKey> {
        PublicKey::from_affine(affine_point)
    }
}

impl TryFrom<&AffinePoint> for PublicKey {
    type Error = Error;

    fn try_from(affine_point: &AffinePoint) -> Result<PublicKey> {
        PublicKey::try_from(*affine_point)
    }
}

#[cfg(feature = "serde")]
impl Serialize for AffinePoint {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_encoded_point(true).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for AffinePoint {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        EncodedPoint::deserialize(deserializer)?
            .try_into()
            .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::AffinePoint;
    use crate::EncodedPoint;
    use elliptic_curve::{
        group::{prime::PrimeCurveAffine, GroupEncoding},
        sec1::{FromEncodedPoint, ToEncodedPoint},
    };
    use hex_literal::hex;

    const UNCOMPRESSED_BASEPOINT: &[u8] = &hex!(
        "0479BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798
         483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"
    );
    const COMPRESSED_BASEPOINT: &[u8] =
        &hex!("0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798");

    #[test]
    fn uncompressed_round_trip() {
        let pubkey = EncodedPoint::from_bytes(UNCOMPRESSED_BASEPOINT).unwrap();
        let res: EncodedPoint = AffinePoint::from_encoded_point(&pubkey)
            .unwrap()
            .to_encoded_point(false);

        assert_eq!(res, pubkey);
    }

    #[test]
    fn compressed_round_trip() {
        let pubkey = EncodedPoint::from_bytes(COMPRESSED_BASEPOINT).unwrap();
        let res: EncodedPoint = AffinePoint::from_encoded_point(&pubkey)
            .unwrap()
            .to_encoded_point(true);

        assert_eq!(res, pubkey);
    }

    #[test]
    fn uncompressed_to_compressed() {
        let encoded = EncodedPoint::from_bytes(UNCOMPRESSED_BASEPOINT).unwrap();

        let res = AffinePoint::from_encoded_point(&encoded)
            .unwrap()
            .to_encoded_point(true);

        assert_eq!(res.as_bytes(), COMPRESSED_BASEPOINT);
    }

    #[test]
    fn compressed_to_uncompressed() {
        let encoded = EncodedPoint::from_bytes(COMPRESSED_BASEPOINT).unwrap();

        let res = AffinePoint::from_encoded_point(&encoded)
            .unwrap()
            .to_encoded_point(false);

        assert_eq!(res.as_bytes(), UNCOMPRESSED_BASEPOINT);
    }

    #[test]
    fn affine_negation() {
        let basepoint = AffinePoint::GENERATOR;
        assert_eq!((-(-basepoint)), basepoint);
    }

    #[test]
    fn identity_encoding() {
        // This is technically an invalid SEC1 encoding, but is preferable to panicking.
        assert_eq!([0; 33], AffinePoint::IDENTITY.to_bytes().as_slice());
        assert!(bool::from(
            AffinePoint::from_bytes(&AffinePoint::IDENTITY.to_bytes())
                .unwrap()
                .is_identity()
        ))
    }
}
