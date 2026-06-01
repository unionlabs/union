use core::fmt::Display;

use bip32::{
    PrivateKey, PublicKey,
    secp256k1::{ecdsa::Signature, schnorr::signature::Signer},
};
use ripemd::Digest;
use unionlabs::primitives::{Bech32, FixedBytes, H160, H256, H512};

pub trait WalletT {
    fn address(&self) -> Bech32<H160>;

    fn public_key(&self) -> FixedBytes<33>;

    fn sign(&self, bz: &[u8]) -> H512;
}

/// A simple wrapper around a cosmos signer (ECDSA), easily representable as a bech32 string.
#[derive(Debug)]
pub struct LocalSigner {
    pub(crate) signing_key: k256::ecdsa::SigningKey,
    pub(crate) prefix: String,
}

impl LocalSigner {
    #[must_use]
    pub fn new_from_signing_key(signing_key: k256::ecdsa::SigningKey, prefix: String) -> Self {
        Self {
            signing_key,
            prefix,
        }
    }

    pub fn new_from_private_key(private_key: H256, prefix: impl Into<String>) -> Self {
        Self::new_from_signing_key(
            bip32::secp256k1::ecdsa::SigningKey::from_bytes(&(*private_key.get()).into())
                .expect("invalid private key"),
            prefix.into(),
        )
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
    pub fn private_key(&self) -> H256 {
        self.signing_key.to_bytes().into()
    }
}

impl Display for LocalSigner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.address())
    }
}

impl WalletT for LocalSigner {
    fn address(&self) -> Bech32<H160> {
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

    fn public_key(&self) -> FixedBytes<33> {
        self.signing_key.public_key().to_bytes().into()
    }

    fn sign(&self, bz: &[u8]) -> H512 {
        Signer::<Signature>::try_sign(&self.signing_key, bz)
            .expect("infallible")
            .to_bytes()
            .into()
    }
}

impl<T: WalletT> WalletT for &T {
    fn address(&self) -> Bech32<H160> {
        (*self).address()
    }

    fn public_key(&self) -> FixedBytes<33> {
        (*self).public_key()
    }

    fn sign(&self, bz: &[u8]) -> H512 {
        (*self).sign(bz)
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
            LocalSigner::new_from_signing_key(signing_key(), "prefix".to_string()).to_string(),
            // cspell:disable-next-line
            "prefix14sarpj4p7l68eze5shfx4xtxr7vl92ge20mdc5"
        );
    }
}
