use std::fmt::Display;

use bip32::secp256k1::{
    ecdsa::{self, Signature},
    schnorr::signature::Signer,
};
use sha2::Digest;

pub mod core;
pub mod google;
pub mod lightclients;

pub mod errors {
    #[derive(Debug, Clone)]
    pub struct UnknownEnumVariant<T>(pub T);

    /// A protobuf field was none unexpectedly.
    #[derive(Debug)]
    pub struct MissingField(pub &'static str);
}

pub trait IntoProto: Into<Self::Proto> {
    type Proto: prost::Message + TypeUrl;

    fn into_proto(self) -> Self::Proto {
        self.into()
    }
}

pub trait FromProto: From<Self::Proto> {
    type Proto: prost::Message;

    fn from_proto(proto: Self::Proto) -> Self {
        proto.into()
    }
}

pub trait TryFromProto: TryFrom<Self::Proto> {
    type Proto: prost::Message;

    fn try_from_proto(proto: Self::Proto) -> Result<Self, <Self as TryFrom<Self::Proto>>::Error> {
        proto.try_into()
    }
}

impl<T> TryFromProto for T
where
    T: FromProto,
{
    type Proto = T::Proto;
}

pub trait TypeUrl: prost::Message {
    const TYPE_URL: &'static str;
}

/// The various `msg` types for cosmos have an extra `signer` field that
/// the solidity equivalents don't have; this trait is required so that
/// the signer can be passed in.
pub trait MsgIntoProto {
    type Proto;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto;
}

pub struct CosmosAccountId {
    xprv: bip32::XPrv,
    prefix: String,
}

impl CosmosAccountId {
    pub fn new(xprv: bip32::XPrv, prefix: String) -> Self {
        Self { xprv, prefix }
    }

    pub fn public_key(&self) -> Vec<u8> {
        self.xprv.public_key().public_key().to_bytes().to_vec()
    }

    pub fn try_sign(&self, bytes: Vec<u8>) -> Result<Signature, ecdsa::Error> {
        Signer::<Signature>::try_sign(self.xprv.private_key(), &bytes)
    }
}

impl Display for CosmosAccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let encoded = subtle_encoding::bech32::encode(
            &self.prefix,
            ripemd::Ripemd160::new()
                .chain_update(
                    sha2::Sha256::new()
                        .chain_update(self.xprv.public_key().public_key().to_bytes())
                        .finalize(),
                )
                .finalize(),
        );

        f.write_str(&encoded)
    }
}
