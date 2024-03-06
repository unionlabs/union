use milagro_bls::{AmclError, G2_BYTES};

use crate::macros::hex_string_array_wrapper;

const BLS_PUBLIC_KEY_BYTES_LEN: usize = 48;
const BLS_SIGNATURE_BYTES_LEN: usize = G2_BYTES;

#[derive(Clone)]
pub struct BlsSecretKey(milagro_bls::SecretKey);

impl TryFrom<Vec<u8>> for BlsSecretKey {
    type Error = AmclError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        milagro_bls::SecretKey::from_bytes(&data).map(Self)
    }
}

hex_string_array_wrapper! {
    pub struct BlsPublicKey(pub [u8; BLS_PUBLIC_KEY_BYTES_LEN]);
    pub struct BlsSignature(pub [u8; BLS_SIGNATURE_BYTES_LEN]);
}

impl BlsSecretKey {
    #[must_use]
    pub fn public_key(&self) -> BlsPublicKey {
        BlsPublicKey::from(self.clone())
    }

    #[must_use]
    pub fn sign(&self, msg: &[u8]) -> BlsSignature {
        milagro_bls::Signature::new(msg, &self.0).into()
    }
}

impl From<milagro_bls::AggregatePublicKey> for BlsPublicKey {
    fn from(agg_pk: milagro_bls::AggregatePublicKey) -> Self {
        Self(
            milagro_bls::PublicKey {
                point: agg_pk.point,
            }
            .as_bytes(),
        )
    }
}

impl From<BlsSecretKey> for BlsPublicKey {
    fn from(key: BlsSecretKey) -> Self {
        Self(milagro_bls::PublicKey::from_secret_key(&key.0).as_bytes())
    }
}

impl TryFrom<&BlsPublicKey> for milagro_bls::PublicKey {
    type Error = AmclError;

    fn try_from(public_key: &BlsPublicKey) -> Result<Self, Self::Error> {
        Self::from_bytes(&public_key.0)
    }
}

impl BlsSignature {
    #[must_use]
    pub fn into_bytes(self) -> [u8; BLS_SIGNATURE_BYTES_LEN] {
        self.into()
    }
}

impl TryFrom<&BlsSignature> for milagro_bls::Signature {
    type Error = AmclError;

    fn try_from(signature: &BlsSignature) -> Result<Self, Self::Error> {
        Self::from_bytes(signature.0.as_ref())
    }
}

impl From<milagro_bls::Signature> for BlsSignature {
    fn from(signature: milagro_bls::Signature) -> Self {
        signature.as_bytes().into()
    }
}
