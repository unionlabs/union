//! Taproot Schnorr signing key.

use super::{tagged_hash, Signature, VerifyingKey, AUX_TAG, CHALLENGE_TAG, NONCE_TAG};
use crate::{AffinePoint, FieldBytes, NonZeroScalar, ProjectivePoint, PublicKey, Scalar};
use ecdsa_core::signature::{
    digest::{consts::U32, FixedOutput},
    DigestSigner, Error, RandomizedDigestSigner, RandomizedSigner, Result, Signer,
};
use elliptic_curve::{
    bigint::U256,
    ops::Reduce,
    rand_core::{CryptoRng, RngCore},
    subtle::ConditionallySelectable,
    zeroize::{Zeroize, ZeroizeOnDrop},
};
use sha2::{Digest, Sha256};

/// Taproot Schnorr signing key.
#[derive(Clone)]
pub struct SigningKey {
    /// Secret key material
    secret_key: NonZeroScalar,

    /// Verifying key
    verifying_key: VerifyingKey,
}

impl SigningKey {
    /// Generate a cryptographically random [`SigningKey`].
    pub fn random(rng: impl CryptoRng + RngCore) -> Self {
        let bytes = NonZeroScalar::random(rng).to_bytes();
        Self::from_bytes(&bytes).unwrap()
    }

    /// Parse signing key from big endian-encoded bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (secret_key, verifying_point) = Self::raw_from_bytes(bytes)?;
        let verifying_key = PublicKey::from_affine(verifying_point).map_err(|_| Error::new())?;

        Ok(Self {
            secret_key,
            verifying_key: verifying_key.try_into()?,
        })
    }

    // a little type dance for use in SigningKey's `from_bytes` and `try_sign`.
    fn raw_from_bytes(bytes: &[u8]) -> Result<(NonZeroScalar, AffinePoint)> {
        let trial_secret_key = NonZeroScalar::try_from(bytes).map_err(|_| Error::new())?;

        let even = (ProjectivePoint::GENERATOR * *trial_secret_key)
            .to_affine()
            .y
            .normalize()
            .is_even();

        let secret_key =
            NonZeroScalar::conditional_select(&-trial_secret_key, &trial_secret_key, even);

        let verifying_point = (ProjectivePoint::GENERATOR * *secret_key).to_affine();

        Ok((secret_key, verifying_point))
    }

    /// Serialize as bytes.
    pub fn to_bytes(&self) -> FieldBytes {
        self.secret_key.to_bytes()
    }

    /// Get the [`VerifyingKey`] that corresponds to this signing key.
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Compute Schnorr signature.
    ///
    /// # ⚠️ Warning
    ///
    /// This is a low-level interface intended only for unusual use cases
    /// involving signing pre-hashed messages.
    ///
    /// The preferred interfaces are the [`Signer`] or [`RandomizedSigner`] traits.
    pub fn try_sign_prehashed(
        &self,
        msg_digest: &[u8; 32],
        aux_rand: &[u8; 32],
    ) -> Result<Signature> {
        let mut t = tagged_hash(AUX_TAG).chain_update(aux_rand).finalize();

        for (a, b) in t.iter_mut().zip(self.secret_key.to_bytes().iter()) {
            *a ^= b
        }

        let rand = tagged_hash(NONCE_TAG)
            .chain_update(&t)
            .chain_update(&self.verifying_key.as_affine().x.to_bytes())
            .chain_update(msg_digest)
            .finalize();

        // the ephemeral key "k"
        let (secret_key, verifying_point) = SigningKey::raw_from_bytes(&rand)?;

        let r = verifying_point.x.normalize();

        let e = <Scalar as Reduce<U256>>::from_be_bytes_reduced(
            tagged_hash(CHALLENGE_TAG)
                .chain_update(&r.to_bytes())
                .chain_update(&self.verifying_key.to_bytes())
                .chain_update(msg_digest)
                .finalize(),
        );

        let s = *secret_key + e * *self.secret_key;
        let s: NonZeroScalar = Option::from(NonZeroScalar::new(s)).ok_or_else(Error::new)?;

        let mut bytes = [0u8; Signature::BYTE_SIZE];
        let (r_bytes, s_bytes) = bytes.split_at_mut(Signature::BYTE_SIZE / 2);
        r_bytes.copy_from_slice(&r.to_bytes());
        s_bytes.copy_from_slice(&s.to_bytes());

        let sig = Signature { bytes, r, s };

        #[cfg(debug_assertions)]
        self.verifying_key.verify_prehashed(msg_digest, &sig)?;

        Ok(sig)
    }
}

impl<D> DigestSigner<D, Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest(&self, digest: D) -> Result<Signature> {
        self.try_sign_prehashed(&digest.finalize_fixed().into(), &Default::default())
    }
}

impl<D> RandomizedDigestSigner<D, Signature> for SigningKey
where
    D: Digest + FixedOutput<OutputSize = U32>,
{
    fn try_sign_digest_with_rng(
        &self,
        mut rng: impl CryptoRng + RngCore,
        digest: D,
    ) -> Result<Signature> {
        let mut aux_rand = [0u8; 32];
        rng.fill_bytes(&mut aux_rand);
        self.try_sign_prehashed(&digest.finalize_fixed().into(), &aux_rand)
    }
}

impl RandomizedSigner<Signature> for SigningKey {
    fn try_sign_with_rng(&self, rng: impl CryptoRng + RngCore, msg: &[u8]) -> Result<Signature> {
        self.try_sign_digest_with_rng(rng, Sha256::new_with_prefix(msg))
    }
}

impl Signer<Signature> for SigningKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature> {
        self.try_sign_digest(Sha256::new_with_prefix(msg))
    }
}

impl Drop for SigningKey {
    fn drop(&mut self) {
        self.secret_key.zeroize();
    }
}

impl ZeroizeOnDrop for SigningKey {}
