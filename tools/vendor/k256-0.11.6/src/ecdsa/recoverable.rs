//! Ethereum-style "recoverable signatures".
//!
//! These signatures include an additional [`Id`] field which allows for
//! recovery of the [`VerifyingKey`] which can be used to verify them.
//!
//! This is helpful in cases where a hash/fingerprint of a [`VerifyingKey`]
//! for a given signature in known in advance.
//!
//! ## Signing/Recovery Example
//!
//! NOTE: make sure to enable both the `ecdsa` and `keccak256` features of
//! this crate for the example to work.
//!
//! ```
//! # #[cfg(all(feature = "ecdsa", feature = "keccak256"))]
//! # {
//! use k256::{
//!     ecdsa::{SigningKey, recoverable, signature::Signer},
//!     EncodedPoint
//! };
//! use rand_core::OsRng; // requires 'getrandom' feature
//!
//! // Signing
//! let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
//! let verifying_key = signing_key.verifying_key();
//! let message = b"ECDSA proves knowledge of a secret number in the context of a single message";
//!
//! // Note: the signature type must be annotated or otherwise inferrable as
//! // `Signer` has many impls of the `Signer` trait (for both regular and
//! // recoverable signature types).
//! let signature: recoverable::Signature = signing_key.sign(message);
//! let recovered_key = signature.recover_verifying_key(message).expect("couldn't recover pubkey");
//!
//! assert_eq!(&verifying_key, &recovered_key);
//! # }
//! ```

use core::fmt::{self, Debug};
use ecdsa_core::{signature::Signature as _, Error, Result};
use elliptic_curve::subtle::Choice;

#[cfg(feature = "ecdsa")]
use crate::{
    ecdsa::{
        signature::{
            digest::{Digest, FixedOutput},
            hazmat::PrehashVerifier,
        },
        VerifyingKey,
    },
    elliptic_curve::{
        bigint::U256,
        consts::U32,
        ops::{Invert, LinearCombination, Reduce},
        DecompressPoint,
    },
    AffinePoint, FieldBytes, NonZeroScalar, ProjectivePoint, Scalar,
};

#[cfg(feature = "keccak256")]
use sha3::Keccak256;

/// Size of an Ethereum-style recoverable signature in bytes
pub const SIZE: usize = 65;

/// Ethereum-style "recoverable signatures" which allow for the recovery of
/// the signer's [`VerifyingKey`] from the signature itself.
///
/// This format consists of [`Signature`] followed by a 1-byte recovery [`Id`]
/// (65-bytes total):
///
/// - `r`: 32-byte integer, big endian
/// - `s`: 32-byte integer, big endian
/// - `v`: 1-byte recovery [`Id`]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Signature {
    bytes: [u8; SIZE],
}

impl Signature {
    /// Create a new recoverable ECDSA/secp256k1 signature from a regular
    /// fixed-size signature and an associated recovery [`Id`].
    ///
    /// This is an "unchecked" conversion and assumes the provided [`Id`]
    /// is valid for this signature.
    pub fn new(signature: &super::Signature, recovery_id: Id) -> Result<Self> {
        let mut bytes = [0u8; SIZE];
        bytes[..64].copy_from_slice(signature.as_ref());
        bytes[64] = recovery_id.0;
        Ok(Self { bytes })
    }

    /// Get the recovery [`Id`] for this signature
    pub fn recovery_id(self) -> Id {
        self.bytes[64].try_into().expect("invalid recovery ID")
    }

    /// Given a public key, message, and signature, use trial recovery
    /// to determine if a suitable recovery ID exists, or return an error
    /// otherwise.
    ///
    /// Assumes Keccak256 as the message digest function. Use
    /// [`Signature::from_digest_trial_recovery`] to support other
    ///digest functions.
    #[cfg(all(feature = "ecdsa", feature = "keccak256"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "keccak256")))]
    pub fn from_trial_recovery(
        public_key: &VerifyingKey,
        msg: &[u8],
        signature: &super::Signature,
    ) -> Result<Self> {
        Self::from_digest_trial_recovery(public_key, Keccak256::new_with_prefix(msg), signature)
    }

    /// Given a public key, message digest, and signature, use trial recovery
    /// to determine if a suitable recovery ID exists, or return an error
    /// otherwise.
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    pub fn from_digest_trial_recovery<D>(
        public_key: &VerifyingKey,
        digest: D,
        signature: &super::Signature,
    ) -> Result<Self>
    where
        D: Clone + Digest + FixedOutput<OutputSize = U32>,
    {
        Self::from_digest_bytes_trial_recovery(public_key, &digest.finalize(), signature)
    }

    /// Given a public key, message digest, and signature, use trial recovery
    /// to determine if a suitable recovery ID exists, or return an error
    /// otherwise.
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    pub fn from_digest_bytes_trial_recovery(
        public_key: &VerifyingKey,
        digest_bytes: &FieldBytes,
        signature: &super::Signature,
    ) -> Result<Self> {
        let signature = signature.normalize_s().unwrap_or(*signature);

        for recovery_id in 0..=1 {
            if let Ok(recoverable_signature) = Signature::new(&signature, Id(recovery_id)) {
                if let Ok(recovered_key) =
                    recoverable_signature.recover_verifying_key_from_digest_bytes(digest_bytes)
                {
                    if public_key == &recovered_key
                        && public_key.verify_prehash(digest_bytes, &signature).is_ok()
                    {
                        return Ok(recoverable_signature);
                    }
                }
            }
        }

        Err(Error::new())
    }

    /// Recover the public key used to create the given signature as a
    /// [`VerifyingKey`].
    #[cfg(all(feature = "ecdsa", feature = "keccak256"))]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    #[cfg_attr(docsrs, doc(cfg(feature = "keccak256")))]
    pub fn recover_verifying_key(&self, msg: &[u8]) -> Result<VerifyingKey> {
        self.recover_verifying_key_from_digest(Keccak256::new_with_prefix(msg))
    }

    /// Recover the public key used to create the given signature as a
    /// [`VerifyingKey`] from the provided precomputed [`Digest`].
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    pub fn recover_verifying_key_from_digest<D>(&self, msg_digest: D) -> Result<VerifyingKey>
    where
        D: Digest<OutputSize = U32>,
    {
        self.recover_verifying_key_from_digest_bytes(&msg_digest.finalize())
    }

    /// Recover the public key used to create the given signature as a
    /// [`VerifyingKey`] from the raw bytes of a message digest.
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    #[allow(non_snake_case, clippy::many_single_char_names)]
    pub fn recover_verifying_key_from_digest_bytes(
        &self,
        digest_bytes: &FieldBytes,
    ) -> Result<VerifyingKey> {
        let r = self.r();
        let s = self.s();
        let z = <Scalar as Reduce<U256>>::from_be_bytes_reduced(*digest_bytes);
        let R = AffinePoint::decompress(&r.to_bytes(), self.recovery_id().is_y_odd());

        if R.is_none().into() {
            return Err(Error::new());
        }

        let R = ProjectivePoint::from(R.unwrap());
        let r_inv = *r.invert();
        let u1 = -(r_inv * z);
        let u2 = r_inv * *s;
        let pk = ProjectivePoint::lincomb(&ProjectivePoint::GENERATOR, &u1, &R, &u2);

        // TODO(tarcieri): ensure the signature verifies?
        VerifyingKey::try_from(pk)
    }

    /// Parse the `r` component of this signature to a [`NonZeroScalar`]
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    pub fn r(&self) -> NonZeroScalar {
        NonZeroScalar::try_from(&self.bytes[..32])
            .expect("r-component ensured valid in constructor")
    }

    /// Parse the `s` component of this signature to a [`NonZeroScalar`]
    #[cfg(feature = "ecdsa")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ecdsa")))]
    pub fn s(&self) -> NonZeroScalar {
        NonZeroScalar::try_from(&self.bytes[32..64])
            .expect("s-component ensured valid in constructor")
    }
}

impl ecdsa_core::signature::Signature for Signature {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bytes.try_into()
    }
}

impl AsRef<[u8]> for Signature {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..]
    }
}

impl Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RecoverableSignature {{ bytes: {:?}) }}", self.as_ref())
    }
}

impl TryFrom<&[u8]> for Signature {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SIZE {
            return Err(Error::new());
        }

        let signature = super::Signature::try_from(&bytes[..64])?;
        let recovery_id = Id::try_from(bytes[64])?;
        Self::new(&signature, recovery_id)
    }
}

impl From<Signature> for super::Signature {
    fn from(sig: Signature) -> Self {
        Self::from_bytes(&sig.bytes[..64]).unwrap()
    }
}

#[cfg(feature = "keccak256")]
impl ecdsa_core::signature::PrehashSignature for Signature {
    type Digest = Keccak256;
}

/// Identifier used to compute a [`VerifyingKey`] from a [`Signature`].
///
/// In practice these values are always either `0` or `1`, and indicate
/// whether or not the y-coordinate of the original [`VerifyingKey`] is odd.
///
/// While values `2` and `3` are also defined to capture whether `r`
/// overflowed the curve's order, this crate does *not* support them.
///
/// There is a vanishingly small chance of these values occurring outside
/// of contrived examples, so for simplicity's sake handling these values
/// is unsupported and will return an `Error` when parsing the `Id`.
#[derive(Copy, Clone, Debug)]
pub struct Id(pub(super) u8);

impl Id {
    /// Create a new [`Id`] from the given byte value
    pub fn new(byte: u8) -> Result<Self> {
        match byte {
            0 | 1 => Ok(Self(byte)),
            _ => Err(Error::new()),
        }
    }

    /// Is `y` odd?
    fn is_y_odd(self) -> Choice {
        self.0.into()
    }
}

impl TryFrom<u8> for Id {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self> {
        Self::new(byte)
    }
}

impl From<Id> for u8 {
    fn from(recovery_id: Id) -> u8 {
        recovery_id.0
    }
}

impl TryFrom<ecdsa_core::RecoveryId> for Id {
    type Error = Error;

    fn try_from(id: ecdsa_core::RecoveryId) -> Result<Id> {
        if id.is_x_reduced() {
            Err(Error::new())
        } else if id.is_y_odd() {
            Ok(Id(1))
        } else {
            Ok(Id(0))
        }
    }
}

impl From<Id> for ecdsa_core::RecoveryId {
    fn from(id: Id) -> ecdsa_core::RecoveryId {
        ecdsa_core::RecoveryId::new(id.is_y_odd().into(), false)
    }
}

#[cfg(all(test, feature = "ecdsa", feature = "keccak256", feature = "sha256"))]
mod tests {
    use super::Signature;
    use crate::{
        ecdsa::{signature::Signer, SigningKey},
        EncodedPoint,
    };
    use hex_literal::hex;
    use sha2::{Digest, Sha256};

    /// Signature recovery test vectors
    struct RecoveryTestVector {
        pk: [u8; 33],
        sig: [u8; 65],
        msg: &'static [u8],
    }

    const RECOVERY_TEST_VECTORS: &[RecoveryTestVector] = &[
        // Recovery ID 0
        RecoveryTestVector {
            pk: hex!("021a7a569e91dbf60581509c7fc946d1003b60c7dee85299538db6353538d59574"),
            sig: hex!(
                "ce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa03239
                 71edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d5200"
            ),
            msg: b"example message",
        },
        // Recovery ID 1
        RecoveryTestVector {
            pk: hex!("036d6caac248af96f6afa7f904f550253a0f3ef3f5aa2fe6838a95b216691468e2"),
            sig: hex!(
                "46c05b6368a44b8810d79859441d819b8e7cdc8bfd371e35c53196f4bcacdb5135
                 c7facce2a97b95eacba8a586d87b7958aaf8368ab29cee481f76e871dbd9cb01"
            ),
            msg: b"example message",
        },
    ];

    #[test]
    fn public_key_recovery() {
        for vector in RECOVERY_TEST_VECTORS {
            let sig = Signature::try_from(&vector.sig[..]).unwrap();
            let prehash = Sha256::new_with_prefix(vector.msg);
            let pk = sig.recover_verifying_key_from_digest(prehash).unwrap();
            assert_eq!(&vector.pk[..], EncodedPoint::from(&pk).as_bytes());
        }
    }

    /// Ensures RFC6979 is implemented in the same way as other Ethereum
    /// libraries, using HMAC-DRBG-SHA-256 for RFC6979, and Keccak256 for
    /// hashing the message.
    ///
    /// Test vectors adapted from:
    /// <https://github.com/gakonst/ethers-rs/blob/ba00f549/ethers-signers/src/wallet/private_key.rs#L197>
    #[test]
    fn signing_rfc6979() {
        let signing_key = SigningKey::from_bytes(&hex!(
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318"
        ))
        .unwrap();

        let msg = hex!(
            "e9808504e3b29200831e848094f0109fc8df283027b6285cc889f5aa624eac1f55843b9aca0080018080"
        );

        let sig: Signature = signing_key.sign(&msg);
        assert_eq!(sig.as_ref(), &hex!("c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa6800"));
    }
}
