use unionlabs::{
    bech32::Bech32,
    primitives::{FixedBytes, H160, H256, H512},
    signer::CosmosSigner,
};

pub trait WalletT {
    fn address(&self) -> Bech32<H160>;

    fn public_key(&self) -> FixedBytes<33>;

    fn sign(&self, bz: &[u8]) -> H512;
}

#[derive(Debug)]
pub struct LocalSigner {
    signer: CosmosSigner,
}

impl LocalSigner {
    pub fn new(private_key: H256, prefix: impl Into<String>) -> Self {
        Self {
            signer: CosmosSigner::new(
                bip32::secp256k1::ecdsa::SigningKey::from_bytes(&(*private_key.get()).into())
                    .expect("invalid private key"),
                prefix.into(),
            ),
        }
    }
}

impl WalletT for LocalSigner {
    fn address(&self) -> Bech32<H160> {
        self.signer.address()
    }

    fn public_key(&self) -> FixedBytes<33> {
        self.signer.public_key()
    }

    fn sign(&self, bz: &[u8]) -> H512 {
        self.signer
            .try_sign(bz)
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
