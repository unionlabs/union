use core::fmt::Display;

use bip32::{
    secp256k1::{ecdsa, ecdsa::Signature, schnorr::signature::Signer},
    PrivateKey, PublicKey,
};
use ripemd::Digest;

use crate::hash::H256;

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
    pub fn public_key(&self) -> [u8; 33] {
        self.signing_key.public_key().to_bytes()
    }

    /// Attempt to sign the given bytes.
    ///
    /// # Errors
    ///
    /// See [`Signer::try_sign`].
    pub fn try_sign(&self, bytes: &[u8]) -> Result<Signature, ecdsa::Error> {
        Signer::<Signature>::try_sign(&self.signing_key, bytes)
    }
}

impl Display for CosmosSigner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO: benchmark this, and consider caching it in the struct
        // bech32(prefix, ripemd(sha256(pubkey)))
        let encoded = subtle_encoding::bech32::encode(
            &self.prefix,
            ripemd::Ripemd160::new()
                .chain_update(
                    sha2::Sha256::new()
                        .chain_update(self.signing_key.public_key().to_bytes())
                        .finalize(),
                )
                .finalize(),
        );

        f.write_str(&encoded)
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
