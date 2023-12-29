#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo.svg"
)]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unused_qualifications)]

//! ## `serde` support
//!
//! When the `serde` feature of this crate is enabled, `Serialize` and
//! `Deserialize` are impl'd for the following types:
//!
//! - [`AffinePoint`]
//! - [`Scalar`]
//! - [`ecdsa::VerifyingKey`]
//!
//! Please see type-specific documentation for more information.

#[cfg(feature = "arithmetic")]
mod arithmetic;

#[cfg(feature = "ecdh")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecdh")))]
pub mod ecdh;

#[cfg(feature = "ecdsa-core")]
#[cfg_attr(docsrs, doc(cfg(feature = "ecdsa-core")))]
pub mod ecdsa;

#[cfg(feature = "schnorr")]
#[cfg_attr(docsrs, doc(cfg(feature = "schnorr")))]
pub mod schnorr;

#[cfg(any(feature = "test-vectors", test))]
#[cfg_attr(docsrs, doc(cfg(feature = "test-vectors")))]
pub mod test_vectors;

pub use elliptic_curve::{self, bigint::U256};

#[cfg(feature = "arithmetic")]
pub use arithmetic::{affine::AffinePoint, projective::ProjectivePoint, scalar::Scalar};

#[cfg(feature = "expose-field")]
pub use arithmetic::FieldElement;

#[cfg(feature = "pkcs8")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs8")))]
pub use elliptic_curve::pkcs8;

#[cfg(feature = "sha2")]
#[cfg_attr(docsrs, doc(cfg(feature = "sha2")))]
pub use sha2;

use elliptic_curve::{consts::U33, generic_array::GenericArray};

/// Order of the secp256k1 elliptic curve
const ORDER: U256 =
    U256::from_be_hex("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141");

/// secp256k1 (K-256) elliptic curve.
///
/// Specified in Certicom's SECG in "SEC 2: Recommended Elliptic Curve Domain Parameters":
///
/// <https://www.secg.org/sec2-v2.pdf>
///
/// The curve's equation is `y² = x³ + 7` over a ~256-bit prime field.
///
/// It's primarily notable for usage in Bitcoin and other cryptocurrencies,
/// particularly in conjunction with the Elliptic Curve Digital Signature
/// Algorithm (ECDSA).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Secp256k1;

impl elliptic_curve::Curve for Secp256k1 {
    /// 256-bit field modulus
    type UInt = U256;

    /// Curve order
    const ORDER: U256 = ORDER;
}

impl elliptic_curve::PrimeCurve for Secp256k1 {}

impl elliptic_curve::PointCompression for Secp256k1 {
    /// secp256k1 points are typically compressed.
    const COMPRESS_POINTS: bool = true;
}

#[cfg(feature = "jwk")]
#[cfg_attr(docsrs, doc(cfg(feature = "jwk")))]
impl elliptic_curve::JwkParameters for Secp256k1 {
    const CRV: &'static str = "secp256k1";
}

#[cfg(feature = "pkcs8")]
impl pkcs8::AssociatedOid for Secp256k1 {
    const OID: pkcs8::ObjectIdentifier = pkcs8::ObjectIdentifier::new_unwrap("1.3.132.0.10");
}

/// Compressed SEC1-encoded secp256k1 (K-256) curve point.
pub type CompressedPoint = GenericArray<u8, U33>;

/// secp256k1 (K-256) field element serialized as bytes.
///
/// Byte array containing a serialized field element value (base field or scalar).
pub type FieldBytes = elliptic_curve::FieldBytes<Secp256k1>;

/// SEC1-encoded secp256k1 (K-256) curve point.
pub type EncodedPoint = elliptic_curve::sec1::EncodedPoint<Secp256k1>;

/// Non-zero secp256k1 (K-256) scalar field element.
#[cfg(feature = "arithmetic")]
pub type NonZeroScalar = elliptic_curve::NonZeroScalar<Secp256k1>;

/// secp256k1 (K-256) public key.
#[cfg(feature = "arithmetic")]
pub type PublicKey = elliptic_curve::PublicKey<Secp256k1>;

/// secp256k1 (K-256) secret key.
pub type SecretKey = elliptic_curve::SecretKey<Secp256k1>;

#[cfg(not(feature = "arithmetic"))]
impl elliptic_curve::sec1::ValidatePublicKey for Secp256k1 {}

/// Bit representation of a secp256k1 (K-256) scalar field element.
#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
pub type ScalarBits = elliptic_curve::ScalarBits<Secp256k1>;
