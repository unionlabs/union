#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::fmt::{Debug, Display};

use bip32::secp256k1::{
    ecdsa::{self, Signature},
    schnorr::signature::Signer,
};
use prost::Message;
use sha2::Digest;

use crate::{errors::TryFromBranchError, ethereum::H256};

/// Defines types that wrap the IBC specification, matching the proto module structure.
pub mod ibc;

/// Various ethereum types. Types that have an IBC counterpart are defined in [`ibc`].
pub mod ethereum;

/// Wrapper types around [`milagro_bls`] types, providing more conversions and a simpler signing interface.
pub mod bls;

pub mod ethereum_consts_traits;

pub mod errors {
    use std::fmt::Debug;

    use crate::ethereum::H256;

    #[derive(Debug, Clone)]
    pub struct UnknownEnumVariant<T>(pub T);

    /// A protobuf field was none unexpectedly.
    #[derive(Debug)]
    pub struct MissingField(pub &'static str);

    // Expected one length, but found another.
    #[derive(Debug, PartialEq, Eq)]
    pub struct InvalidLength {
        pub expected: usize,
        pub found: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum TryFromBranchError<T>
    where
        T: TryFrom<Vec<H256>>,
        <T as TryFrom<Vec<H256>>>::Error: Debug + PartialEq + Eq,
    {
        ExecutionBranch(<T as TryFrom<Vec<H256>>>::Error),
        ExecutionBranchNode(InvalidLength),
    }
}

pub fn try_from_proto_branch<T>(proto: Vec<Vec<u8>>) -> Result<T, TryFromBranchError<T>>
where
    T: TryFrom<Vec<H256>>,
    <T as TryFrom<Vec<H256>>>::Error: Debug + PartialEq + Eq,
{
    proto
        .into_iter()
        .map(H256::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(TryFromBranchError::ExecutionBranchNode)?
        .try_into()
        .map_err(TryFromBranchError::ExecutionBranch)
}

// TODO: Move these traits into `ibc`

pub trait IntoProto: Into<Self::Proto> {
    type Proto: Message + TypeUrl;

    fn into_proto(self) -> Self::Proto {
        self.into()
    }

    fn into_proto_bytes(self) -> Vec<u8> {
        self.into_proto().encode_to_vec()
    }
}

/// A type that can be infallibly converted from it's protobuf representation.
pub trait FromProto: From<Self::Proto> {
    type Proto: Message;

    fn from_proto(proto: Self::Proto) -> Self {
        proto.into()
    }
}

pub trait TryFromProto: TryFrom<Self::Proto> {
    type Proto: Message + Default;

    fn try_from_proto(proto: Self::Proto) -> Result<Self, TryFromProtoErrorOf<Self>> {
        proto.try_into()
    }

    fn try_from_proto_bytes(
        bytes: &[u8],
    ) -> Result<Self, TryFromProtoBytesError<TryFromProtoErrorOf<Self>>> {
        <Self::Proto as Message>::decode(bytes)
            .map_err(TryFromProtoBytesError::Decode)
            .and_then(|proto| {
                proto
                    .try_into()
                    .map_err(TryFromProtoBytesError::TryFromProto)
            })
    }
}

#[derive(Debug)]
pub enum TryFromProtoBytesError<E> {
    TryFromProto(E),
    Decode(prost::DecodeError),
}

pub type TryFromProtoErrorOf<T> = <T as TryFrom<<T as TryFromProto>::Proto>>::Error;

impl<T> TryFromProto for T
where
    T: FromProto,
    <T as FromProto>::Proto: Default,
{
    type Proto = T::Proto;
}

pub trait TypeUrl: Message {
    const TYPE_URL: &'static str;
}

/// The various `msg` types for cosmos have an extra `signer` field that
/// the solidity equivalents don't have; this trait is required to allow
/// the signer to be passed in.
pub trait MsgIntoProto {
    type Proto;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto;
}

/// A simple wrapper around a cosmos account, easily representable as a bech32 string.
pub struct CosmosAccountId {
    xprv: bip32::XPrv,
    prefix: String,
}

impl CosmosAccountId {
    #[must_use]
    pub fn new(xprv: bip32::XPrv, prefix: String) -> Self {
        Self { xprv, prefix }
    }

    #[must_use]
    pub fn public_key(&self) -> Vec<u8> {
        self.xprv.public_key().public_key().to_bytes().to_vec()
    }

    /// Attempt to sign the given bytes.
    ///
    /// # Errors
    ///
    /// See [`Signer::try_sign`].
    pub fn try_sign(&self, bytes: &[u8]) -> Result<Signature, ecdsa::Error> {
        Signer::<Signature>::try_sign(self.xprv.private_key(), bytes)
    }
}

impl Display for CosmosAccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: benchmark this, and consider caching it in the struct
        // bech32(prefix, ripemd(sha256(pubkey)))
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
