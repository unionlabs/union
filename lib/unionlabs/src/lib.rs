#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::fmt::{Debug, Display};

use bip32::{
    secp256k1::{
        ecdsa::{self, Signature},
        schnorr::signature::Signer,
    },
    PrivateKey, PublicKey,
};
use prost::Message;
use sha2::Digest;

use crate::{errors::TryFromBranchError, ethereum::H256};

/// Defines types that wrap the IBC specification, matching the proto module structure. This also includes `union` extensions to ibc (i.e. types defined in `union.ibc`).
pub mod ibc;

/// Defines types that wrap the tendermint specification, matching the proto module structure.
pub mod tendermint;

/// Defines types that wrap the cosmos specification, matching the proto module structure.
pub mod cosmos;

/// Various ethereum types. Types that have an IBC counterpart are defined in [`ibc`].
pub mod ethereum;

/// Types specific to the union protocol.
pub mod union;

/// Wrapper types around [`milagro_bls`] types, providing more conversions and a simpler signing interface.
pub mod bls;

pub mod ethereum_consts_traits;

pub mod bounded_int;

pub(crate) mod macros;

pub mod errors {
    use std::fmt::Debug;

    use crate::ethereum::H256;

    #[derive(Debug, Clone)]
    pub struct UnknownEnumVariant<T>(pub T);

    /// A protobuf field was none unexpectedly.
    #[derive(Debug)]
    pub struct MissingField(pub &'static str);

    /// For fields that are "fake options" from prost, for use in `TryFrom<<Self as Proto>::Proto>`.
    ///
    /// `Self::Error` is expected to have a `MissingField(`[`MissingField`]`)` variant.
    macro_rules! required {
        ($struct_var:ident.$field:ident) => {
            $struct_var
                .$field
                .ok_or(<Self::Error>::MissingField(MissingField(stringify!(
                    $field
                ))))
        };
    }

    // https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files
    pub(crate) use required;

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

pub trait Proto: Into<Self::Proto> {
    // all prost generated code implements Default
    type Proto: TypeUrl + Default;
}

pub trait IntoProto: Proto + Into<Self::Proto> {
    fn into_proto(self) -> Self::Proto {
        self.into()
    }

    fn into_proto_bytes(self) -> Vec<u8> {
        self.into_proto().encode_to_vec()
    }
}

/// A type that can be infallibly converted from it's protobuf representation.
pub trait FromProto: Proto + From<Self::Proto> {
    fn from_proto(proto: Self::Proto) -> Self {
        proto.into()
    }

    fn from_proto_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
        <Self::Proto as Message>::decode(bytes).map(Into::into)
    }
}

pub trait TryFromProto: Proto + TryFrom<Self::Proto> {
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

impl<T> IntoProto for T where T: Proto + Into<T::Proto> {}
impl<T> FromProto for T where T: Proto + From<T::Proto> {}
impl<T> TryFromProto for T where T: Proto + TryFrom<T::Proto> {}

#[derive(Debug)]
pub enum TryFromProtoBytesError<E> {
    TryFromProto(E),
    Decode(prost::DecodeError),
}

pub type TryFromProtoErrorOf<T> = <T as TryFrom<<T as Proto>::Proto>>::Error;

pub trait TypeUrl: Message {
    const TYPE_URL: &'static str;
}

#[cfg(test)]
fn assert_proto_roundtrip<T>(t: &T)
where
    T: IntoProto + TryFromProto + Debug + Clone + PartialEq,
    TryFromProtoErrorOf<T>: Debug,
{
    let try_from_proto = T::try_from_proto(t.clone().into_proto()).unwrap();

    assert_eq!(t, &try_from_proto, "proto roundtrip failed");
}

#[cfg(test)]
fn assert_json_roundtrip<T>(t: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + Debug + PartialEq,
{
    let from_json = serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap()).unwrap();

    assert_eq!(t, &from_json, "json roundtrip failed");
}

/// The various `msg` types for cosmos have an extra `signer` field that
/// the solidity equivalents don't have; this trait is required to allow
/// the signer to be passed in.
pub trait MsgIntoProto {
    type Proto: TypeUrl;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto;
}

#[cfg(feature = "ethabi")]
pub trait EthAbi {
    type EthAbi: ethers_core::abi::AbiEncode + ethers_core::abi::AbiDecode;
}

#[cfg(feature = "ethabi")]
pub trait IntoEthAbi: EthAbi + Into<Self::EthAbi> {
    fn into_eth_abi(self) -> Self::EthAbi {
        self.into()
    }

    fn into_eth_abi_bytes(self) -> Vec<u8> {
        ethers_core::abi::AbiEncode::encode(self.into())
    }
}

#[cfg(feature = "ethabi")]
pub trait FromEthAbi: EthAbi + From<Self::EthAbi> {
    fn from_eth_abi(proto: Self::EthAbi) -> Self {
        proto.into()
    }

    fn from_eth_abi_bytes(bytes: &[u8]) -> Result<Self, ethers_core::abi::AbiError> {
        <Self::EthAbi as ethers_core::abi::AbiDecode>::decode(bytes).map(Into::into)
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiBytesError<E> {
    TryFromEthAbi(E),
    Decode(ethers_core::abi::AbiError),
}

#[cfg(feature = "ethabi")]
pub type TryFromEthAbiErrorOf<T> = <T as TryFrom<<T as EthAbi>::EthAbi>>::Error;

#[cfg(feature = "ethabi")]
pub trait TryFromEthAbi: EthAbi + TryFrom<Self::EthAbi> {
    fn try_from_eth_abi(proto: Self::EthAbi) -> Result<Self, TryFromEthAbiErrorOf<Self>> {
        proto.try_into()
    }

    fn try_from_eth_abi_bytes(
        bytes: &[u8],
    ) -> Result<Self, TryFromEthAbiBytesError<TryFromEthAbiErrorOf<Self>>> {
        <Self::EthAbi as ethers_core::abi::AbiDecode>::decode(bytes)
            .map_err(TryFromEthAbiBytesError::Decode)
            .and_then(|proto| {
                proto
                    .try_into()
                    .map_err(TryFromEthAbiBytesError::TryFromEthAbi)
            })
    }
}

#[cfg(feature = "ethabi")]
impl<T> IntoEthAbi for T where T: EthAbi + Into<T::EthAbi> {}
#[cfg(feature = "ethabi")]
impl<T> FromEthAbi for T where T: EthAbi + From<T::EthAbi> {}
#[cfg(feature = "ethabi")]
impl<T> TryFromEthAbi for T where T: EthAbi + TryFrom<T::EthAbi> {}

/// A simple wrapper around a cosmos account, easily representable as a bech32 string.
#[derive(Debug, Clone)]
pub struct CosmosAccountId {
    signing_key: k256::ecdsa::SigningKey,
    prefix: String,
}

impl CosmosAccountId {
    #[must_use]
    pub fn new(signing_key: k256::ecdsa::SigningKey, prefix: String) -> Self {
        Self {
            signing_key,
            prefix,
        }
    }

    #[must_use]
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.public_key().to_bytes().to_vec()
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

impl Display for CosmosAccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
