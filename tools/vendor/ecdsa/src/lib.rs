#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/RustCrypto/media/8f1a9894/logo.svg"
)]
#![forbid(unsafe_code)]
#![warn(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::checked_conversions,
    clippy::implicit_saturating_sub,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

//! ## `serde` support
//!
//! When the `serde` feature of this crate is enabled, `Serialize` and
//! `Deserialize` impls are provided for the [`Signature`] and [`VerifyingKey`]
//! types.
//!
//! Please see type-specific documentation for more information.
//!
//! ## Interop
//!
//! Any crates which provide an implementation of ECDSA for a particular
//! elliptic curve can leverage the types from this crate, along with the
//! [`k256`], [`p256`], and/or [`p384`] crates to expose ECDSA functionality in
//! a generic, interoperable way by leveraging the [`Signature`] type with in
//! conjunction with the [`signature::Signer`] and [`signature::Verifier`]
//! traits.
//!
//! For example, the [`ring-compat`] crate implements the [`signature::Signer`]
//! and [`signature::Verifier`] traits in conjunction with the
//! [`p256::ecdsa::Signature`] and [`p384::ecdsa::Signature`] types to
//! wrap the ECDSA implementations from [*ring*] in a generic, interoperable
//! API.
//!
//! [`k256`]: https://docs.rs/k256
//! [`p256`]: https://docs.rs/p256
//! [`p256::ecdsa::Signature`]: https://docs.rs/p256/latest/p256/ecdsa/type.Signature.html
//! [`p384`]: https://docs.rs/p384
//! [`p384::ecdsa::Signature`]: https://docs.rs/p384/latest/p384/ecdsa/type.Signature.html
//! [`ring-compat`]: https://docs.rs/ring-compat
//! [*ring*]: https://docs.rs/ring

#[cfg(feature = "alloc")]
extern crate alloc;

mod recovery;

#[cfg(feature = "der")]
pub mod der;
#[cfg(feature = "dev")]
pub mod dev;
#[cfg(feature = "hazmat")]
pub mod hazmat;
#[cfg(feature = "signing")]
mod signing;
#[cfg(feature = "verifying")]
mod verifying;

pub use crate::recovery::RecoveryId;

// Re-export the `elliptic-curve` crate (and select types)
pub use elliptic_curve::{self, sec1::EncodedPoint, PrimeCurve};

// Re-export the `signature` crate (and select types)
pub use signature::{self, Error, Result, SignatureEncoding};

#[cfg(feature = "signing")]
pub use crate::signing::SigningKey;
#[cfg(feature = "verifying")]
pub use crate::verifying::VerifyingKey;

use core::{fmt, ops::Add};
use elliptic_curve::{
    generic_array::{typenum::Unsigned, ArrayLength, GenericArray},
    FieldBytes, FieldBytesSize, ScalarPrimitive,
};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "arithmetic")]
use {
    core::str,
    elliptic_curve::{scalar::IsHigh, CurveArithmetic, NonZeroScalar},
};

#[cfg(feature = "digest")]
use digest::{
    const_oid::{AssociatedOid, ObjectIdentifier},
    Digest,
};

#[cfg(feature = "pkcs8")]
use elliptic_curve::pkcs8::spki::{
    der::AnyRef, AlgorithmIdentifierRef, AssociatedAlgorithmIdentifier,
};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

#[cfg(all(feature = "alloc", feature = "pkcs8"))]
use elliptic_curve::pkcs8::spki::{
    self, AlgorithmIdentifierOwned, DynAssociatedAlgorithmIdentifier,
};

/// OID for ECDSA with SHA-224 digests.
///
/// ```text
/// ecdsa-with-SHA224 OBJECT IDENTIFIER ::= { iso(1) member-body(2)
///      us(840) ansi-X9-62(10045) signatures(4) ecdsa-with-SHA2(3) 1 }
/// ```
// TODO(tarcieri): use `ObjectIdentifier::push_arc` when const unwrap is stable
#[cfg(feature = "digest")]
pub const ECDSA_SHA224_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.1");

/// OID for ECDSA with SHA-256 digests.
///
/// ```text
/// ecdsa-with-SHA256 OBJECT IDENTIFIER ::= { iso(1) member-body(2)
///      us(840) ansi-X9-62(10045) signatures(4) ecdsa-with-SHA2(3) 2 }
/// ```
#[cfg(feature = "digest")]
pub const ECDSA_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");

/// OID for ECDSA with SHA-384 digests.
///
/// ```text
/// ecdsa-with-SHA384 OBJECT IDENTIFIER ::= { iso(1) member-body(2)
///      us(840) ansi-X9-62(10045) signatures(4) ecdsa-with-SHA2(3) 3 }
/// ```
#[cfg(feature = "digest")]
pub const ECDSA_SHA384_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.3");

/// OID for ECDSA with SHA-512 digests.
///
/// ```text
/// ecdsa-with-SHA512 OBJECT IDENTIFIER ::= { iso(1) member-body(2)
///      us(840) ansi-X9-62(10045) signatures(4) ecdsa-with-SHA2(3) 4 }
/// ```
#[cfg(feature = "digest")]
pub const ECDSA_SHA512_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.4");

#[cfg(feature = "digest")]
const SHA224_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.4");
#[cfg(feature = "digest")]
const SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
#[cfg(feature = "digest")]
const SHA384_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
#[cfg(feature = "digest")]
const SHA512_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");

/// Size of a fixed sized signature for the given elliptic curve.
pub type SignatureSize<C> = <FieldBytesSize<C> as Add>::Output;

/// Fixed-size byte array containing an ECDSA signature
pub type SignatureBytes<C> = GenericArray<u8, SignatureSize<C>>;

/// ECDSA signature (fixed-size). Generic over elliptic curve types.
///
/// Serialized as fixed-sized big endian scalar values with no added framing:
///
/// - `r`: field element size for the given curve, big-endian
/// - `s`: field element size for the given curve, big-endian
///
/// Both `r` and `s` MUST be non-zero.
///
/// For example, in a curve with a 256-bit modulus like NIST P-256 or
/// secp256k1, `r` and `s` will both be 32-bytes and serialized as big endian,
/// resulting in a signature with a total of 64-bytes.
///
/// ASN.1 DER-encoded signatures also supported via the
/// [`Signature::from_der`] and [`Signature::to_der`] methods.
///
/// # `serde` support
///
/// When the `serde` feature of this crate is enabled, it provides support for
/// serializing and deserializing ECDSA signatures using the `Serialize` and
/// `Deserialize` traits.
///
/// The serialization uses a hexadecimal encoding when used with
/// "human readable" text formats, and a binary encoding otherwise.
#[derive(Clone, Eq, PartialEq)]
pub struct Signature<C: PrimeCurve> {
    r: ScalarPrimitive<C>,
    s: ScalarPrimitive<C>,
}

impl<C> Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    /// Parse a signature from fixed-width bytes, i.e. 2 * the size of
    /// [`FieldBytes`] for a particular curve.
    ///
    /// # Returns
    /// - `Ok(signature)` if the `r` and `s` components are both in the valid
    ///   range `1..n` when serialized as concatenated big endian integers.
    /// - `Err(err)` if the `r` and/or `s` component of the signature is
    ///   out-of-range when interpreted as a big endian integer.
    pub fn from_bytes(bytes: &SignatureBytes<C>) -> Result<Self> {
        let (r_bytes, s_bytes) = bytes.split_at(C::FieldBytesSize::USIZE);
        let r = FieldBytes::<C>::clone_from_slice(r_bytes);
        let s = FieldBytes::<C>::clone_from_slice(s_bytes);
        Self::from_scalars(r, s)
    }

    /// Parse a signature from a byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() == SignatureSize::<C>::USIZE {
            Self::from_bytes(SignatureBytes::<C>::from_slice(slice))
        } else {
            Err(Error::new())
        }
    }

    /// Parse a signature from ASN.1 DER.
    #[cfg(feature = "der")]
    pub fn from_der(bytes: &[u8]) -> Result<Self>
    where
        der::MaxSize<C>: ArrayLength<u8>,
        <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
    {
        der::Signature::<C>::try_from(bytes).and_then(Self::try_from)
    }

    /// Create a [`Signature`] from the serialized `r` and `s` scalar values
    /// which comprise the signature.
    ///
    /// # Returns
    /// - `Ok(signature)` if the `r` and `s` components are both in the valid
    ///   range `1..n` when serialized as concatenated big endian integers.
    /// - `Err(err)` if the `r` and/or `s` component of the signature is
    ///   out-of-range when interpreted as a big endian integer.
    pub fn from_scalars(r: impl Into<FieldBytes<C>>, s: impl Into<FieldBytes<C>>) -> Result<Self> {
        let r = ScalarPrimitive::from_slice(&r.into()).map_err(|_| Error::new())?;
        let s = ScalarPrimitive::from_slice(&s.into()).map_err(|_| Error::new())?;

        if r.is_zero().into() || s.is_zero().into() {
            return Err(Error::new());
        }

        Ok(Self { r, s })
    }

    /// Split the signature into its `r` and `s` components, represented as bytes.
    pub fn split_bytes(&self) -> (FieldBytes<C>, FieldBytes<C>) {
        (self.r.to_bytes(), self.s.to_bytes())
    }

    /// Serialize this signature as bytes.
    pub fn to_bytes(&self) -> SignatureBytes<C> {
        let mut bytes = SignatureBytes::<C>::default();
        let (r_bytes, s_bytes) = bytes.split_at_mut(C::FieldBytesSize::USIZE);
        r_bytes.copy_from_slice(&self.r.to_bytes());
        s_bytes.copy_from_slice(&self.s.to_bytes());
        bytes
    }

    /// Serialize this signature as ASN.1 DER.
    #[cfg(feature = "der")]
    pub fn to_der(&self) -> der::Signature<C>
    where
        der::MaxSize<C>: ArrayLength<u8>,
        <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
    {
        let (r, s) = self.split_bytes();
        der::Signature::from_components(&r, &s).expect("DER encoding error")
    }

    /// Convert this signature into a byte vector.
    #[cfg(feature = "alloc")]
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }
}

#[cfg(feature = "arithmetic")]
impl<C> Signature<C>
where
    C: PrimeCurve + CurveArithmetic,
    SignatureSize<C>: ArrayLength<u8>,
{
    /// Get the `r` component of this signature
    pub fn r(&self) -> NonZeroScalar<C> {
        NonZeroScalar::new(self.r.into()).unwrap()
    }

    /// Get the `s` component of this signature
    pub fn s(&self) -> NonZeroScalar<C> {
        NonZeroScalar::new(self.s.into()).unwrap()
    }

    /// Split the signature into its `r` and `s` scalars.
    pub fn split_scalars(&self) -> (NonZeroScalar<C>, NonZeroScalar<C>) {
        (self.r(), self.s())
    }

    /// Normalize signature into "low S" form as described in
    /// [BIP 0062: Dealing with Malleability][1].
    ///
    /// [1]: https://github.com/bitcoin/bips/blob/master/bip-0062.mediawiki
    pub fn normalize_s(&self) -> Option<Self> {
        let s = self.s();

        if s.is_high().into() {
            let mut result = self.clone();
            result.s = ScalarPrimitive::from(-s);
            Some(result)
        } else {
            None
        }
    }
}

impl<C> Copy for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
    <SignatureSize<C> as ArrayLength<u8>>::ArrayType: Copy,
{
}

impl<C> From<Signature<C>> for SignatureBytes<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(signature: Signature<C>) -> SignatureBytes<C> {
        signature.to_bytes()
    }
}

impl<C> SignatureEncoding for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Repr = SignatureBytes<C>;
}

impl<C> TryFrom<&[u8]> for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Self> {
        Self::from_slice(slice)
    }
}

impl<C> fmt::Debug for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ecdsa::Signature<{:?}>(", C::default())?;

        for byte in self.to_bytes() {
            write!(f, "{:02X}", byte)?;
        }

        write!(f, ")")
    }
}

impl<C> fmt::Display for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self)
    }
}

impl<C> fmt::LowerHex for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.to_bytes() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<C> fmt::UpperHex for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.to_bytes() {
            write!(f, "{:02X}", byte)?;
        }
        Ok(())
    }
}

#[cfg(feature = "arithmetic")]
impl<C> str::FromStr for Signature<C>
where
    C: PrimeCurve + CurveArithmetic,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Err = Error;

    fn from_str(hex: &str) -> Result<Self> {
        if hex.as_bytes().len() != C::FieldBytesSize::USIZE * 4 {
            return Err(Error::new());
        }

        // This check is mainly to ensure `hex.split_at` below won't panic
        if !hex
            .as_bytes()
            .iter()
            .all(|&byte| matches!(byte, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z'))
        {
            return Err(Error::new());
        }

        let (r_hex, s_hex) = hex.split_at(C::FieldBytesSize::USIZE * 2);

        let r = r_hex
            .parse::<NonZeroScalar<C>>()
            .map_err(|_| Error::new())?;

        let s = s_hex
            .parse::<NonZeroScalar<C>>()
            .map_err(|_| Error::new())?;

        Self::from_scalars(r, s)
    }
}

/// ECDSA [`ObjectIdentifier`] which identifies the digest used by default
/// with the `Signer` and `Verifier` traits.
///
/// To support non-default digest algorithms, use the [`SignatureWithOid`]
/// type instead.
#[cfg(all(feature = "digest", feature = "hazmat"))]
impl<C> AssociatedOid for Signature<C>
where
    C: hazmat::DigestPrimitive,
    C::Digest: AssociatedOid,
{
    const OID: ObjectIdentifier = match ecdsa_oid_for_digest(C::Digest::OID) {
        Some(oid) => oid,
        None => panic!("no RFC5758 ECDSA OID defined for DigestPrimitive::Digest"),
    };
}

/// ECDSA `AlgorithmIdentifier` which identifies the digest used by default
/// with the `Signer` and `Verifier` traits.
#[cfg(feature = "pkcs8")]
impl<C> AssociatedAlgorithmIdentifier for Signature<C>
where
    C: PrimeCurve,
    Self: AssociatedOid,
{
    type Params = AnyRef<'static>;

    const ALGORITHM_IDENTIFIER: AlgorithmIdentifierRef<'static> = AlgorithmIdentifierRef {
        oid: Self::OID,
        parameters: None,
    };
}

#[cfg(feature = "serde")]
impl<C> Serialize for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(&self.to_bytes(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, C> Deserialize<'de> for Signature<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = SignatureBytes::<C>::default();
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Self::try_from(bytes.as_slice()).map_err(de::Error::custom)
    }
}

/// An extended [`Signature`] type which is parameterized by an
/// `ObjectIdentifier` which identifies the ECDSA variant used by a
/// particular signature.
///
/// Valid `ObjectIdentifiers` are defined in [RFC5758 § 3.2]:
///
/// - SHA-224: [`ECDSA_SHA224_OID`] (1.2.840.10045.4.3.1)
/// - SHA-256: [`ECDSA_SHA256_OID`] (1.2.840.10045.4.3.2)
/// - SHA-384: [`ECDSA_SHA384_OID`] (1.2.840.10045.4.3.3)
/// - SHA-512: [`ECDSA_SHA512_OID`] (1.2.840.10045.4.3.4)
///
/// [RFC5758 § 3.2]: https://www.rfc-editor.org/rfc/rfc5758#section-3.2
#[cfg(feature = "digest")]
#[derive(Clone, Eq, PartialEq)]
pub struct SignatureWithOid<C: PrimeCurve> {
    /// Inner signature type.
    signature: Signature<C>,

    /// OID which identifies the ECDSA variant used.
    ///
    /// MUST be one of the ECDSA algorithm variants as defined in RFC5758.
    ///
    /// These OIDs begin with `1.2.840.10045.4`.
    oid: ObjectIdentifier,
}

#[cfg(feature = "digest")]
impl<C> SignatureWithOid<C>
where
    C: PrimeCurve,
{
    /// Create a new signature with an explicitly provided OID.
    ///
    /// OID must begin with `1.2.840.10045.4`, the [RFC5758] OID prefix for
    /// ECDSA variants.
    ///
    /// [RFC5758]: https://www.rfc-editor.org/rfc/rfc5758#section-3.2
    pub fn new(signature: Signature<C>, oid: ObjectIdentifier) -> Result<Self> {
        // TODO(tarcieri): use `ObjectIdentifier::starts_with`
        for (arc1, arc2) in ObjectIdentifier::new_unwrap("1.2.840.10045.4.3")
            .arcs()
            .zip(oid.arcs())
        {
            if arc1 != arc2 {
                return Err(Error::new());
            }
        }

        Ok(Self { signature, oid })
    }

    /// Create a new signature, determining the OID from the given digest.
    ///
    /// Supports SHA-2 family digests as enumerated in [RFC5758 § 3.2], i.e.
    /// SHA-224, SHA-256, SHA-384, or SHA-512.
    ///
    /// [RFC5758 § 3.2]: https://www.rfc-editor.org/rfc/rfc5758#section-3.2
    pub fn new_with_digest<D>(signature: Signature<C>) -> Result<Self>
    where
        D: AssociatedOid + Digest,
    {
        let oid = ecdsa_oid_for_digest(D::OID).ok_or_else(Error::new)?;
        Ok(Self { signature, oid })
    }

    /// Parse a signature from fixed-with bytes.
    pub fn from_bytes_with_digest<D>(bytes: &SignatureBytes<C>) -> Result<Self>
    where
        D: AssociatedOid + Digest,
        SignatureSize<C>: ArrayLength<u8>,
    {
        Self::new_with_digest::<D>(Signature::<C>::from_bytes(bytes)?)
    }

    /// Parse a signature from a byte slice.
    pub fn from_slice_with_digest<D>(slice: &[u8]) -> Result<Self>
    where
        D: AssociatedOid + Digest,
        SignatureSize<C>: ArrayLength<u8>,
    {
        Self::new_with_digest::<D>(Signature::<C>::from_slice(slice)?)
    }

    /// Get the fixed-width ECDSA signature.
    pub fn signature(&self) -> &Signature<C> {
        &self.signature
    }

    /// Get the ECDSA OID for this signature.
    pub fn oid(&self) -> ObjectIdentifier {
        self.oid
    }

    /// Serialize this signature as bytes.
    pub fn to_bytes(&self) -> SignatureBytes<C>
    where
        SignatureSize<C>: ArrayLength<u8>,
    {
        self.signature.to_bytes()
    }
}

#[cfg(feature = "digest")]
impl<C> Copy for SignatureWithOid<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
    <SignatureSize<C> as ArrayLength<u8>>::ArrayType: Copy,
{
}

#[cfg(feature = "digest")]
impl<C> From<SignatureWithOid<C>> for Signature<C>
where
    C: PrimeCurve,
{
    fn from(sig: SignatureWithOid<C>) -> Signature<C> {
        sig.signature
    }
}

#[cfg(feature = "digest")]
impl<C> From<SignatureWithOid<C>> for SignatureBytes<C>
where
    C: PrimeCurve,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(signature: SignatureWithOid<C>) -> SignatureBytes<C> {
        signature.to_bytes()
    }
}

/// NOTE: this implementation assumes the default digest for the given elliptic
/// curve as defined by [`hazmat::DigestPrimitive`].
///
/// When working with alternative digests, you will need to use e.g.
/// [`SignatureWithOid::new_with_digest`].
#[cfg(all(feature = "digest", feature = "hazmat"))]
impl<C> SignatureEncoding for SignatureWithOid<C>
where
    C: hazmat::DigestPrimitive,
    C::Digest: AssociatedOid,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Repr = SignatureBytes<C>;
}

/// NOTE: this implementation assumes the default digest for the given elliptic
/// curve as defined by [`hazmat::DigestPrimitive`].
///
/// When working with alternative digests, you will need to use e.g.
/// [`SignatureWithOid::new_with_digest`].
#[cfg(all(feature = "digest", feature = "hazmat"))]
impl<C> TryFrom<&[u8]> for SignatureWithOid<C>
where
    C: hazmat::DigestPrimitive,
    C::Digest: AssociatedOid,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Self> {
        Self::new(Signature::<C>::from_slice(slice)?, C::Digest::OID)
    }
}

#[cfg(all(feature = "alloc", feature = "pkcs8"))]
impl<C> DynAssociatedAlgorithmIdentifier for SignatureWithOid<C>
where
    C: PrimeCurve,
{
    fn algorithm_identifier(&self) -> spki::Result<AlgorithmIdentifierOwned> {
        Ok(AlgorithmIdentifierOwned {
            oid: self.oid,
            parameters: None,
        })
    }
}

/// Get the ECDSA OID for a given digest OID.
#[cfg(feature = "digest")]
const fn ecdsa_oid_for_digest(digest_oid: ObjectIdentifier) -> Option<ObjectIdentifier> {
    match digest_oid {
        SHA224_OID => Some(ECDSA_SHA224_OID),
        SHA256_OID => Some(ECDSA_SHA256_OID),
        SHA384_OID => Some(ECDSA_SHA384_OID),
        SHA512_OID => Some(ECDSA_SHA512_OID),
        _ => None,
    }
}
