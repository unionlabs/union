use core::fmt::Display;

use bip32::{
    secp256k1::{ecdsa, ecdsa::Signature, schnorr::signature::Signer},
    PrivateKey, PublicKey,
};
use ripemd::Digest;
use unionlabs_primitives::{FixedBytes, H160};

use crate::{bech32::Bech32, primitives::H256};

/// A simple wrapper around a cosmos signer (ECDSA), easily representable as a bech32 string.
#[derive(Debug, Clone)]
pub struct CosmosSigner {
    pub(crate) signing_key: k256::ecdsa::SigningKey,
    pub(crate) prefix: String,
}

impl CosmosSigner {
    #[must_use]
    pub fn new(signing_key: k256::ecdsa::SigningKey, prefix: String) -> Self {
        Self {
            signing_key,
            prefix,
        }
    }

    pub fn from_raw(signing_key: [u8; 32], prefix: String) -> Result<Self, k256::ecdsa::Error> {
        Ok(Self {
            signing_key: k256::ecdsa::SigningKey::from_bytes(&signing_key.into())?,
            prefix,
        })
    }

    pub fn new_from_bytes(
        signing_key_bytes: H256,
        prefix: String,
    ) -> Result<Self, k256::ecdsa::Error> {
        Ok(Self {
            signing_key: k256::ecdsa::SigningKey::from_bytes(&signing_key_bytes.into())?,
            prefix,
        })
    }

    #[must_use]
    pub fn public_key(&self) -> FixedBytes<33> {
        self.signing_key.public_key().to_bytes().into()
    }

    /// Attempt to sign the given bytes.
    ///
    /// # Errors
    ///
    /// See [`Signer::try_sign`].
    pub fn try_sign(&self, bytes: &[u8]) -> Result<Signature, ecdsa::Error> {
        Signer::<Signature>::try_sign(&self.signing_key, bytes)
    }

    #[must_use]
    pub fn address(&self) -> Bech32<H160> {
        // TODO: benchmark this, and consider caching it in the struct
        // bech32(prefix, ripemd(sha256(pubkey)))
        Bech32::new(
            self.prefix.clone(),
            ripemd::Ripemd160::new()
                .chain_update(
                    sha2::Sha256::new()
                        .chain_update(self.signing_key.public_key().to_bytes())
                        .finalize(),
                )
                .finalize()
                .into(),
        )
    }

    #[must_use]
    pub fn private_key(&self) -> H256 {
        self.signing_key.to_bytes().into()
    }
}

impl Display for CosmosSigner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address())
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    fn signing_key() -> k256::ecdsa::SigningKey {
        <k256::ecdsa::SigningKey as PrivateKey>::from_bytes(&hex!(
            // cspell:disable-next-line
            "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
        ))
        .unwrap()
    }

    #[test]
    fn display() {
        assert_eq!(
            CosmosSigner::new(signing_key(), "prefix".to_string()).to_string(),
            // cspell:disable-next-line
            "prefix14sarpj4p7l68eze5shfx4xtxr7vl92ge20mdc5"
        );
    }
}
