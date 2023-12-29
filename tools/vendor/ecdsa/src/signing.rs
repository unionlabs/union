//! ECDSA signing: producing signatures using a [`SigningKey`].

use crate::{
    ecdsa_oid_for_digest,
    hazmat::{bits2field, DigestPrimitive, SignPrimitive},
    Error, Result, Signature, SignatureSize, SignatureWithOid,
};
use core::fmt::{self, Debug};
use digest::{const_oid::AssociatedOid, Digest, FixedOutput};
use elliptic_curve::{
    generic_array::ArrayLength,
    group::ff::PrimeField,
    ops::Invert,
    subtle::{Choice, ConstantTimeEq, CtOption},
    zeroize::{Zeroize, ZeroizeOnDrop},
    CurveArithmetic, FieldBytes, FieldBytesSize, NonZeroScalar, PrimeCurve, Scalar, SecretKey,
};
use signature::{
    hazmat::{PrehashSigner, RandomizedPrehashSigner},
    rand_core::CryptoRngCore,
    DigestSigner, RandomizedDigestSigner, RandomizedSigner, Signer,
};

#[cfg(feature = "der")]
use {crate::der, core::ops::Add};

#[cfg(feature = "pem")]
use {
    crate::elliptic_curve::pkcs8::{DecodePrivateKey, EncodePrivateKey, SecretDocument},
    core::str::FromStr,
};

#[cfg(feature = "pkcs8")]
use crate::elliptic_curve::{
    pkcs8::{
        self,
        der::AnyRef,
        spki::{AlgorithmIdentifier, AssociatedAlgorithmIdentifier, SignatureAlgorithmIdentifier},
        ObjectIdentifier,
    },
    sec1::{self, FromEncodedPoint, ToEncodedPoint},
    AffinePoint,
};

#[cfg(feature = "verifying")]
use {crate::VerifyingKey, elliptic_curve::PublicKey, signature::KeypairRef};

/// ECDSA secret key used for signing. Generic over prime order elliptic curves
/// (e.g. NIST P-curves)
///
/// Requires an [`elliptic_curve::CurveArithmetic`] impl on the curve, and a
/// [`SignPrimitive`] impl on its associated `Scalar` type.
///
/// ## Usage
///
/// The [`signature`] crate defines the following traits which are the
/// primary API for signing:
///
/// - [`Signer`]: sign a message using this key
/// - [`DigestSigner`]: sign the output of a [`Digest`] using this key
/// - [`PrehashSigner`]: sign the low-level raw output bytes of a message digest
///
/// See the [`p256` crate](https://docs.rs/p256/latest/p256/ecdsa/index.html)
/// for examples of using this type with a concrete elliptic curve.
#[derive(Clone)]
pub struct SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    /// ECDSA signing keys are non-zero elements of a given curve's scalar field.
    secret_scalar: NonZeroScalar<C>,

    /// Verifying key which corresponds to this signing key.
    #[cfg(feature = "verifying")]
    verifying_key: VerifyingKey<C>,
}

impl<C> SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    /// Generate a cryptographically random [`SigningKey`].
    pub fn random(rng: &mut impl CryptoRngCore) -> Self {
        NonZeroScalar::<C>::random(rng).into()
    }

    /// Initialize signing key from a raw scalar serialized as a byte array.
    pub fn from_bytes(bytes: &FieldBytes<C>) -> Result<Self> {
        SecretKey::<C>::from_bytes(bytes)
            .map(Into::into)
            .map_err(|_| Error::new())
    }

    /// Initialize signing key from a raw scalar serialized as a byte slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        SecretKey::<C>::from_slice(bytes)
            .map(Into::into)
            .map_err(|_| Error::new())
    }

    /// Serialize this [`SigningKey`] as bytes
    pub fn to_bytes(&self) -> FieldBytes<C> {
        self.secret_scalar.to_repr()
    }

    /// Borrow the secret [`NonZeroScalar`] value for this key.
    ///
    /// # ⚠️ Warning
    ///
    /// This value is key material.
    ///
    /// Please treat it with the care it deserves!
    pub fn as_nonzero_scalar(&self) -> &NonZeroScalar<C> {
        &self.secret_scalar
    }

    /// Get the [`VerifyingKey`] which corresponds to this [`SigningKey`].
    #[cfg(feature = "verifying")]
    pub fn verifying_key(&self) -> &VerifyingKey<C> {
        &self.verifying_key
    }
}

//
// `*Signer` trait impls
//

/// Sign message digest using a deterministic ephemeral scalar (`k`)
/// computed using the algorithm described in [RFC6979 § 3.2].
///
/// [RFC6979 § 3.2]: https://tools.ietf.org/html/rfc6979#section-3
impl<C, D> DigestSigner<D, Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    D: Digest + FixedOutput<OutputSize = FieldBytesSize<C>>,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign_digest(&self, msg_digest: D) -> Result<Signature<C>> {
        self.sign_prehash(&msg_digest.finalize_fixed())
    }
}

/// Sign message prehash using a deterministic ephemeral scalar (`k`)
/// computed using the algorithm described in [RFC6979 § 3.2].
///
/// [RFC6979 § 3.2]: https://tools.ietf.org/html/rfc6979#section-3
impl<C> PrehashSigner<Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn sign_prehash(&self, prehash: &[u8]) -> Result<Signature<C>> {
        let z = bits2field::<C>(prehash)?;
        Ok(self
            .secret_scalar
            .try_sign_prehashed_rfc6979::<C::Digest>(&z, &[])?
            .0)
    }
}

/// Sign message using a deterministic ephemeral scalar (`k`)
/// computed using the algorithm described in [RFC6979 § 3.2].
///
/// [RFC6979 § 3.2]: https://tools.ietf.org/html/rfc6979#section-3
impl<C> Signer<Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<Signature<C>> {
        self.try_sign_digest(C::Digest::new_with_prefix(msg))
    }
}

impl<C, D> RandomizedDigestSigner<D, Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    D: Digest + FixedOutput<OutputSize = FieldBytesSize<C>>,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign_digest_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        msg_digest: D,
    ) -> Result<Signature<C>> {
        self.sign_prehash_with_rng(rng, &msg_digest.finalize_fixed())
    }
}

impl<C> RandomizedPrehashSigner<Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn sign_prehash_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        prehash: &[u8],
    ) -> Result<Signature<C>> {
        let z = bits2field::<C>(prehash)?;
        let mut ad = FieldBytes::<C>::default();
        rng.fill_bytes(&mut ad);
        Ok(self
            .secret_scalar
            .try_sign_prehashed_rfc6979::<C::Digest>(&z, &ad)?
            .0)
    }
}

impl<C> RandomizedSigner<Signature<C>> for SigningKey<C>
where
    Self: RandomizedDigestSigner<C::Digest, Signature<C>>,
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign_with_rng(&self, rng: &mut impl CryptoRngCore, msg: &[u8]) -> Result<Signature<C>> {
        self.try_sign_digest_with_rng(rng, C::Digest::new_with_prefix(msg))
    }
}

impl<C, D> DigestSigner<D, SignatureWithOid<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    D: AssociatedOid + Digest + FixedOutput<OutputSize = FieldBytesSize<C>>,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign_digest(&self, msg_digest: D) -> Result<SignatureWithOid<C>> {
        let signature: Signature<C> = self.try_sign_digest(msg_digest)?;
        let oid = ecdsa_oid_for_digest(D::OID).ok_or_else(Error::new)?;
        SignatureWithOid::new(signature, oid)
    }
}

impl<C> Signer<SignatureWithOid<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    C::Digest: AssociatedOid,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<SignatureWithOid<C>> {
        self.try_sign_digest(C::Digest::new_with_prefix(msg))
    }
}

#[cfg(feature = "der")]
impl<C> PrehashSigner<der::Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    der::MaxSize<C>: ArrayLength<u8>,
    <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
{
    fn sign_prehash(&self, prehash: &[u8]) -> Result<der::Signature<C>> {
        PrehashSigner::<Signature<C>>::sign_prehash(self, prehash).map(Into::into)
    }
}

#[cfg(feature = "der")]
impl<C> Signer<der::Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    der::MaxSize<C>: ArrayLength<u8>,
    <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<der::Signature<C>> {
        Signer::<Signature<C>>::try_sign(self, msg).map(Into::into)
    }
}

#[cfg(feature = "der")]
impl<C, D> RandomizedDigestSigner<D, der::Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    D: Digest + FixedOutput<OutputSize = FieldBytesSize<C>>,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    der::MaxSize<C>: ArrayLength<u8>,
    <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
{
    fn try_sign_digest_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        msg_digest: D,
    ) -> Result<der::Signature<C>> {
        RandomizedDigestSigner::<D, Signature<C>>::try_sign_digest_with_rng(self, rng, msg_digest)
            .map(Into::into)
    }
}

#[cfg(feature = "der")]
impl<C> RandomizedPrehashSigner<der::Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    der::MaxSize<C>: ArrayLength<u8>,
    <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
{
    fn sign_prehash_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        prehash: &[u8],
    ) -> Result<der::Signature<C>> {
        RandomizedPrehashSigner::<Signature<C>>::sign_prehash_with_rng(self, rng, prehash)
            .map(Into::into)
    }
}

#[cfg(feature = "der")]
impl<C> RandomizedSigner<der::Signature<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    der::MaxSize<C>: ArrayLength<u8>,
    <FieldBytesSize<C> as Add>::Output: Add<der::MaxOverhead> + ArrayLength<u8>,
{
    fn try_sign_with_rng(
        &self,
        rng: &mut impl CryptoRngCore,
        msg: &[u8],
    ) -> Result<der::Signature<C>> {
        RandomizedSigner::<Signature<C>>::try_sign_with_rng(self, rng, msg).map(Into::into)
    }
}

//
// Other trait impls
//

#[cfg(feature = "verifying")]
impl<C> AsRef<VerifyingKey<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn as_ref(&self) -> &VerifyingKey<C> {
        &self.verifying_key
    }
}

impl<C> ConstantTimeEq for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn ct_eq(&self, other: &Self) -> Choice {
        self.secret_scalar.ct_eq(&other.secret_scalar)
    }
}

impl<C> Debug for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SigningKey").finish_non_exhaustive()
    }
}

impl<C> Drop for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn drop(&mut self) {
        self.secret_scalar.zeroize();
    }
}

/// Constant-time comparison
impl<C> Eq for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
}
impl<C> PartialEq for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn eq(&self, other: &SigningKey<C>) -> bool {
        self.ct_eq(other).into()
    }
}

impl<C> From<NonZeroScalar<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(secret_scalar: NonZeroScalar<C>) -> Self {
        #[cfg(feature = "verifying")]
        let public_key = PublicKey::from_secret_scalar(&secret_scalar);

        Self {
            secret_scalar,
            #[cfg(feature = "verifying")]
            verifying_key: public_key.into(),
        }
    }
}

impl<C> From<SecretKey<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(secret_key: SecretKey<C>) -> Self {
        Self::from(&secret_key)
    }
}

impl<C> From<&SecretKey<C>> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(secret_key: &SecretKey<C>) -> Self {
        secret_key.to_nonzero_scalar().into()
    }
}

impl<C> From<SigningKey<C>> for SecretKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(key: SigningKey<C>) -> Self {
        key.secret_scalar.into()
    }
}

impl<C> From<&SigningKey<C>> for SecretKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(secret_key: &SigningKey<C>) -> Self {
        secret_key.secret_scalar.into()
    }
}

impl<C> TryFrom<&[u8]> for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        Self::from_slice(bytes)
    }
}

impl<C> ZeroizeOnDrop for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
}

#[cfg(feature = "verifying")]
impl<C> From<SigningKey<C>> for VerifyingKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(signing_key: SigningKey<C>) -> VerifyingKey<C> {
        signing_key.verifying_key
    }
}

#[cfg(feature = "verifying")]
impl<C> From<&SigningKey<C>> for VerifyingKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn from(signing_key: &SigningKey<C>) -> VerifyingKey<C> {
        signing_key.verifying_key
    }
}

#[cfg(feature = "verifying")]
impl<C> KeypairRef for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    type VerifyingKey = VerifyingKey<C>;
}

#[cfg(feature = "pkcs8")]
impl<C> AssociatedAlgorithmIdentifier for SigningKey<C>
where
    C: AssociatedOid + CurveArithmetic + PrimeCurve,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Params = ObjectIdentifier;

    const ALGORITHM_IDENTIFIER: AlgorithmIdentifier<ObjectIdentifier> =
        SecretKey::<C>::ALGORITHM_IDENTIFIER;
}

#[cfg(feature = "pkcs8")]
impl<C> SignatureAlgorithmIdentifier for SigningKey<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    Signature<C>: AssociatedAlgorithmIdentifier<Params = AnyRef<'static>>,
{
    type Params = AnyRef<'static>;

    const SIGNATURE_ALGORITHM_IDENTIFIER: AlgorithmIdentifier<Self::Params> =
        Signature::<C>::ALGORITHM_IDENTIFIER;
}

#[cfg(feature = "pkcs8")]
impl<C> TryFrom<pkcs8::PrivateKeyInfo<'_>> for SigningKey<C>
where
    C: PrimeCurve + AssociatedOid + CurveArithmetic,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: sec1::ModulusSize,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Error = pkcs8::Error;

    fn try_from(private_key_info: pkcs8::PrivateKeyInfo<'_>) -> pkcs8::Result<Self> {
        SecretKey::try_from(private_key_info).map(Into::into)
    }
}

#[cfg(feature = "pem")]
impl<C> EncodePrivateKey for SigningKey<C>
where
    C: AssociatedOid + PrimeCurve + CurveArithmetic,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: sec1::ModulusSize,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn to_pkcs8_der(&self) -> pkcs8::Result<SecretDocument> {
        SecretKey::from(self.secret_scalar).to_pkcs8_der()
    }
}

#[cfg(feature = "pem")]
impl<C> FromStr for SigningKey<C>
where
    C: PrimeCurve + AssociatedOid + CurveArithmetic,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: sec1::ModulusSize,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_pkcs8_pem(s).map_err(|_| Error::new())
    }
}
