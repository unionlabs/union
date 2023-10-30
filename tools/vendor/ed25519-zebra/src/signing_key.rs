use core::convert::TryFrom;

use curve25519_dalek::{constants, scalar::Scalar};
use rand_core::{CryptoRng, RngCore};
use sha2::{Digest, Sha512};

use crate::{Error, Signature, VerificationKey, VerificationKeyBytes};

/// An Ed25519 signing key.
///
/// This is also called a secret key by other implementations.
#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "SerdeHelper"))]
#[cfg_attr(feature = "serde", serde(into = "SerdeHelper"))]
pub struct SigningKey {
    seed: [u8; 32],
    s: Scalar,
    prefix: [u8; 32],
    vk: VerificationKey,
}

impl core::fmt::Debug for SigningKey {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.debug_struct("SigningKey")
            .field("seed", &hex::encode(&self.seed))
            .field("s", &self.s)
            .field("prefix", &hex::encode(&self.prefix))
            .field("vk", &self.vk)
            .finish()
    }
}

impl<'a> From<&'a SigningKey> for VerificationKey {
    fn from(sk: &'a SigningKey) -> VerificationKey {
        sk.vk
    }
}

impl<'a> From<&'a SigningKey> for VerificationKeyBytes {
    fn from(sk: &'a SigningKey) -> VerificationKeyBytes {
        sk.vk.into()
    }
}

impl AsRef<[u8]> for SigningKey {
    fn as_ref(&self) -> &[u8] {
        &self.seed[..]
    }
}

impl From<SigningKey> for [u8; 32] {
    fn from(sk: SigningKey) -> [u8; 32] {
        sk.seed
    }
}

impl TryFrom<&[u8]> for SigningKey {
    type Error = Error;
    fn try_from(slice: &[u8]) -> Result<SigningKey, Error> {
        if slice.len() == 32 {
            let mut bytes = [0u8; 32];
            bytes[..].copy_from_slice(slice);
            Ok(bytes.into())
        } else {
            Err(Error::InvalidSliceLength)
        }
    }
}

impl From<[u8; 32]> for SigningKey {
    #[allow(non_snake_case)]
    fn from(seed: [u8; 32]) -> SigningKey {
        // Expand the seed to a 64-byte array with SHA512.
        let h = Sha512::digest(&seed[..]);

        // Convert the low half to a scalar with Ed25519 "clamping"
        let s = {
            let mut scalar_bytes = [0u8; 32];
            scalar_bytes[..].copy_from_slice(&h.as_slice()[0..32]);
            scalar_bytes[0] &= 248;
            scalar_bytes[31] &= 127;
            scalar_bytes[31] |= 64;
            Scalar::from_bits(scalar_bytes)
        };

        // Extract and cache the high half.
        let prefix = {
            let mut prefix = [0u8; 32];
            prefix[..].copy_from_slice(&h.as_slice()[32..64]);
            prefix
        };

        // Compute the public key as A = [s]B.
        let A = &s * &constants::ED25519_BASEPOINT_TABLE;

        SigningKey {
            seed,
            s,
            prefix,
            vk: VerificationKey {
                minus_A: -A,
                A_bytes: VerificationKeyBytes(A.compress().to_bytes()),
            },
        }
    }
}

impl zeroize::Zeroize for SigningKey {
    fn zeroize(&mut self) {
        self.seed.zeroize();
        self.s.zeroize()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct SerdeHelper([u8; 32]);

impl From<SerdeHelper> for SigningKey {
    fn from(helper: SerdeHelper) -> SigningKey {
        helper.0.into()
    }
}

impl From<SigningKey> for SerdeHelper {
    fn from(sk: SigningKey) -> Self {
        Self(sk.into())
    }
}

impl SigningKey {
    /// Generate a new signing key.
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> SigningKey {
        let mut bytes = [0u8; 32];
        rng.fill_bytes(&mut bytes[..]);
        bytes.into()
    }

    /// Create a signature on `msg` using this key.
    #[allow(non_snake_case)]
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let r = Scalar::from_hash(Sha512::default().chain(&self.prefix[..]).chain(msg));

        let R_bytes = (&r * &constants::ED25519_BASEPOINT_TABLE)
            .compress()
            .to_bytes();

        let k = Scalar::from_hash(
            Sha512::default()
                .chain(&R_bytes[..])
                .chain(&self.vk.A_bytes.0[..])
                .chain(msg),
        );

        let s_bytes = (r + k * self.s).to_bytes();

        Signature { R_bytes, s_bytes }
    }
}
