//! ECDSA signing support.

use super::{recoverable, Error, Signature, VerifyingKey};
use crate::{FieldBytes, NonZeroScalar, ProjectivePoint, PublicKey, Scalar, Secp256k1, SecretKey};
use core::{
    borrow::Borrow,
    fmt::{self, Debug},
};
use ecdsa_core::{
    hazmat::SignPrimitive,
    signature::{
        digest::{Digest, FixedOutput},
        hazmat::PrehashSigner,
        DigestSigner, Keypair, RandomizedDigestSigner,
    },
};
use elliptic_curve::{
    bigint::U256,
    consts::U32,
    ops::{Invert, Reduce},
    rand_core::{CryptoRng, RngCore},
    subtle::{Choice, ConstantTimeEq, CtOption},
    zeroize::{Zeroize, ZeroizeOnDrop},
    IsHigh,
};
use sha2::Sha256;

#[cfg(any(feature = "keccak256", feature = "sha256"))]
use ecdsa_core::signature::{self, PrehashSignature, RandomizedSigner};

#[cfg(feature = "pkcs8")]
use crate::pkcs8::{self, DecodePrivateKey};

#[cfg(feature = "pem")]
use core::str::FromStr;

/// ECDSA/secp256k1 signing key
#[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
#[derive(Clone)]
pub struct SigningKey {
    /// Secret scalar value (i.e. the private key)
    secret_scalar: NonZeroScalar,

    /// Verifying key which corresponds to this signing key.
    verifying_key: VerifyingKey,
}

impl SigningKey {
    /// Generate a cryptographically random [`SigningKey`].
    pub fn random(rng: impl CryptoRng + RngCore) -> Self {
        NonZeroScalar::random(rng).into()
    }

    /// Initialize [`SigningKey`] from a raw scalar value (big endian).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        SecretKey::from_be_bytes(bytes)
            .map(|sk| sk.to_nonzero_scalar().into())
            .map_err(|_| Error::new())
    }

    /// Get the [`VerifyingKey`] which corresponds to this [`SigningKey`].
    pub fn verifying_key(&self) -> VerifyingKey {
        self.verifying_key
    }

    /// Serialize this [`SigningKey`] as bytes
    pub fn to_bytes(&self) -> FieldBytes {
        self.secret_scalar.to_bytes()
    }
}

impl AsRef<VerifyingKey> for SigningKey {
    fn as_ref(&self) -> &VerifyingKey {
        &self.verifying_key
    }
}

#[cfg(any(feature = "keccak256", feature = "sha256"))]
impl<S> signature::Signer<S> for SigningKey
where
    S: PrehashSignature,
    Self: DigestSigner<S::Digest, S>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<S, Error> {
        self.try_sign_digest(S::Digest::new_with_prefix(msg))
    }
}

#[cfg(any(feature = "keccak256", feature = "sha256"))]
impl<S> RandomizedSigner<S> for SigningKey
where
    S: PrehashSignature,
    Self: RandomizedDigestSigner<S::Digest, S>,
{
    fn try_sign_with_rng(&self, rng: impl CryptoRng + RngCore, msg: &[u8]) -> signature::Result<S> {
        self.try_sign_digest_with_rng(rng, S::Digest::new_with_prefix(msg))
    }
}

impl<D> DigestSigner<D, Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest(&self, msg_digest: D) -> signature::Result<Signature> {
        self.sign_prehash(&msg_digest.finalize_fixed())
    }
}

impl<D> DigestSigner<D, recoverable::Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest(&self, msg_digest: D) -> signature::Result<recoverable::Signature> {
        self.sign_prehash(&msg_digest.finalize_fixed())
    }
}

#[cfg(feature = "sha256")]
#[cfg_attr(docsrs, doc(cfg(feature = "sha256")))]
impl Keypair<Signature> for SigningKey {
    type VerifyingKey = VerifyingKey;
}

#[cfg(feature = "keccak256")]
#[cfg_attr(docsrs, doc(cfg(feature = "keccak256")))]
impl Keypair<recoverable::Signature> for SigningKey {
    type VerifyingKey = VerifyingKey;
}

impl PrehashSigner<Signature> for SigningKey {
    fn sign_prehash(&self, prehash: &[u8]) -> signature::Result<Signature> {
        let prehash = <[u8; 32]>::try_from(prehash).map_err(|_| Error::new())?;

        Ok(self
            .secret_scalar
            .try_sign_prehashed_rfc6979::<Sha256>(prehash.into(), &[])?
            .0)
    }
}

impl PrehashSigner<recoverable::Signature> for SigningKey {
    fn sign_prehash(&self, prehash: &[u8]) -> signature::Result<recoverable::Signature> {
        let prehash = <[u8; 32]>::try_from(prehash).map_err(|_| Error::new())?;

        // Ethereum signatures use SHA-256 for RFC6979, even if the message
        // has been hashed with Keccak256
        let (signature, recid) = self
            .secret_scalar
            .try_sign_prehashed_rfc6979::<Sha256>(prehash.into(), &[])?;

        let recoverable_id = recid.ok_or_else(Error::new)?.try_into()?;
        recoverable::Signature::new(&signature, recoverable_id)
    }
}

impl<D> RandomizedDigestSigner<D, Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest_with_rng(
        &self,
        rng: impl CryptoRng + RngCore,
        digest: D,
    ) -> Result<Signature, Error> {
        RandomizedDigestSigner::<D, recoverable::Signature>::try_sign_digest_with_rng(
            self, rng, digest,
        )
        .map(Into::into)
    }
}

impl<D> RandomizedDigestSigner<D, recoverable::Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest_with_rng(
        &self,
        mut rng: impl CryptoRng + RngCore,
        msg_digest: D,
    ) -> Result<recoverable::Signature, Error> {
        let mut ad = FieldBytes::default();
        rng.fill_bytes(&mut ad);

        let digest = msg_digest.finalize_fixed();

        // Ethereum signatures use SHA-256 for RFC6979, even if the message
        // has been hashed with Keccak256
        let (signature, recid) = self
            .secret_scalar
            .try_sign_prehashed_rfc6979::<Sha256>(digest, &ad)?;

        let recoverable_id = recid.ok_or_else(Error::new)?.try_into()?;
        recoverable::Signature::new(&signature, recoverable_id)
    }
}

impl SignPrimitive<Secp256k1> for Scalar {
    #[allow(non_snake_case, clippy::many_single_char_names)]
    fn try_sign_prehashed<K>(
        &self,
        ephemeral_scalar: K,
        z: FieldBytes,
    ) -> Result<(Signature, Option<ecdsa_core::RecoveryId>), Error>
    where
        K: Borrow<Scalar> + Invert<Output = CtOption<Scalar>>,
    {
        let z = <Self as Reduce<U256>>::from_be_bytes_reduced(z);
        let k_inverse = ephemeral_scalar.invert();
        let k = ephemeral_scalar.borrow();

        if k_inverse.is_none().into() || k.is_zero().into() {
            return Err(Error::new());
        }

        let k_inverse = k_inverse.unwrap();

        // Compute 𝐑 = 𝑘×𝑮
        let R = (ProjectivePoint::GENERATOR * k).to_affine();

        // Lift x-coordinate of 𝐑 (element of base field) into a serialized big
        // integer, then reduce it into an element of the scalar field
        let r = <Scalar as Reduce<U256>>::from_be_bytes_reduced(R.x.to_bytes());

        // Compute `s` as a signature over `r` and `z`.
        let s = k_inverse * (z + (r * self));

        if s.is_zero().into() {
            return Err(Error::new());
        }

        let signature = Signature::from_scalars(r, s)?;
        let is_r_odd = R.y.normalize().is_odd();
        let is_s_high = signature.s().is_high();
        let is_y_odd = is_r_odd ^ is_s_high;
        let signature_low = signature.normalize_s().unwrap_or(signature);
        let recovery_id = ecdsa_core::RecoveryId::new(is_y_odd.into(), false);

        Ok((signature_low, Some(recovery_id)))
    }
}

impl ConstantTimeEq for SigningKey {
    fn ct_eq(&self, other: &Self) -> Choice {
        self.secret_scalar.ct_eq(&other.secret_scalar)
    }
}

impl Debug for SigningKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SigningKey").finish_non_exhaustive()
    }
}

impl Eq for SigningKey {}

impl PartialEq for SigningKey {
    fn eq(&self, other: &SigningKey) -> bool {
        self.ct_eq(other).into()
    }
}

impl From<SecretKey> for SigningKey {
    fn from(secret_key: SecretKey) -> SigningKey {
        Self::from(&secret_key)
    }
}

impl From<&SecretKey> for SigningKey {
    fn from(secret_key: &SecretKey) -> SigningKey {
        secret_key.to_nonzero_scalar().into()
    }
}

impl From<SigningKey> for SecretKey {
    fn from(signing_key: SigningKey) -> SecretKey {
        signing_key.secret_scalar.into()
    }
}

impl From<&SigningKey> for SecretKey {
    fn from(signing_key: &SigningKey) -> SecretKey {
        signing_key.secret_scalar.into()
    }
}

impl From<SigningKey> for VerifyingKey {
    fn from(signing_key: SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }
}

impl From<&SigningKey> for VerifyingKey {
    fn from(signing_key: &SigningKey) -> VerifyingKey {
        signing_key.verifying_key()
    }
}

impl From<NonZeroScalar> for SigningKey {
    fn from(secret_scalar: NonZeroScalar) -> Self {
        let public_key = PublicKey::from_secret_scalar(&secret_scalar);

        Self {
            secret_scalar,
            verifying_key: public_key.into(),
        }
    }
}

impl From<&NonZeroScalar> for SigningKey {
    fn from(secret_scalar: &NonZeroScalar) -> Self {
        Self::from(*secret_scalar)
    }
}

impl Drop for SigningKey {
    fn drop(&mut self) {
        self.secret_scalar.zeroize();
    }
}

impl ZeroizeOnDrop for SigningKey {}

#[cfg(feature = "pkcs8")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs8")))]
impl TryFrom<pkcs8::PrivateKeyInfo<'_>> for SigningKey {
    type Error = pkcs8::Error;

    fn try_from(private_key_info: pkcs8::PrivateKeyInfo<'_>) -> pkcs8::Result<Self> {
        SecretKey::try_from(private_key_info).map(Into::into)
    }
}

#[cfg(feature = "pkcs8")]
#[cfg_attr(docsrs, doc(cfg(feature = "pkcs8")))]
impl DecodePrivateKey for SigningKey {}

#[cfg(feature = "pem")]
#[cfg_attr(docsrs, doc(cfg(feature = "pem")))]
impl FromStr for SigningKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        Self::from_pkcs8_pem(s).map_err(|_| Error::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_vectors::ecdsa::ECDSA_TEST_VECTORS, Secp256k1};
    ecdsa_core::new_signing_test!(Secp256k1, ECDSA_TEST_VECTORS);
}
