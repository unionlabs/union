#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]
#![feature(return_position_impl_trait_in_trait)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
};

use bip32::{
    secp256k1::{
        ecdsa::{self, Signature},
        schnorr::signature::Signer,
    },
    PrivateKey, PublicKey,
};
use prost::Message;
use serde::{de, Deserialize, Serialize};
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

/// Well-known events emitted by ibc-enabled chains.
pub mod events;

pub mod ethereum_consts_traits;

pub mod bounded_int;

pub mod proof;

pub(crate) mod macros;

pub mod errors {
    use std::fmt::{Debug, Display};

    use crate::ethereum::H256;

    #[derive(Debug, Clone)]
    pub struct UnknownEnumVariant<T>(pub T);

    impl<T: Display> Display for UnknownEnumVariant<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("unknown enum variant: {}", self.0))
        }
    }

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
        pub expected: ExpectedLength,
        pub found: usize,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub enum ExpectedLength {
        Exact(usize),
        LessThan(usize),
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

// Into<Self::Proto>
pub trait Proto {
    // all prost generated code implements Default
    type Proto: Message + Default;
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

pub trait TryFromProto: Proto + TryFrom<<Self as Proto>::Proto> {
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

#[cfg(any(feature = "fuzzing", test))]
#[allow(clippy::missing_panics_doc)]
pub mod test_utils {
    use std::{
        fmt::{Debug, Display},
        str::FromStr,
    };

    use super::{IntoProto, TryFromProto, TryFromProtoErrorOf};

    pub fn assert_proto_roundtrip<T>(t: &T)
    where
        T: IntoProto + TryFromProto + Debug + Clone + PartialEq,
        TryFromProtoErrorOf<T>: Debug,
    {
        let try_from_proto = T::try_from_proto(t.clone().into_proto()).unwrap();

        assert_eq!(t, &try_from_proto, "proto roundtrip failed");
    }

    pub fn assert_json_roundtrip<T>(t: &T)
    where
        T: serde::Serialize + for<'a> serde::Deserialize<'a> + Debug + PartialEq,
    {
        let from_json = serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap()).unwrap();

        assert_eq!(t, &from_json, "json roundtrip failed");
    }

    pub fn assert_string_roundtrip<T>(t: &T)
    where
        T: Display + FromStr + Debug + PartialEq,
        <T as FromStr>::Err: Debug,
    {
        let from_str = t.to_string().parse::<T>().unwrap();

        assert_eq!(t, &from_str, "string roundtrip failed");
    }
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

pub mod id {
    use std::{
        error::Error,
        fmt::{Debug, Display},
        num::ParseIntError,
        str::FromStr,
    };

    use serde::{Deserialize, Serialize};

    /// An id of the form `<ty>-<id>`.
    #[derive(PartialEq, Serialize, Deserialize)]
    #[serde(
        bound(serialize = "Type: IdType", deserialize = "Type: IdType"),
        try_from = "&str",
        into = "String"
    )]
    pub struct Id<Type: IdType> {
        ty: Type,
        id: u32,
    }

    pub trait IdType:
        Display
        + FromStr<Err = InvalidIdType>
        + Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + Send
        + Sync
        + 'static
    {
        const TYPE: &'static str;
    }

    impl<Type: IdType> crate::traits::Id for Id<Type> {
        type FromStrErr = <Self as FromStr>::Err;
    }

    impl<Type: IdType> Clone for Id<Type> {
        fn clone(&self) -> Self {
            Self {
                ty: self.ty.clone(),
                id: self.id,
            }
        }
    }

    impl<Type: IdType> Debug for Id<Type> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{:?}({})", self.ty, self.id))
        }
    }

    impl<Type: IdType> Id<Type> {
        pub fn new(ty: Type, id: u32) -> Self {
            Self { ty, id }
        }
    }

    impl<Type: IdType> Display for Id<Type> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{}-{}", self.ty, self.id))
        }
    }

    impl<Type: IdType> From<Id<Type>> for String {
        fn from(value: Id<Type>) -> Self {
            value.to_string()
        }
    }

    #[derive(Debug)]
    pub enum IdParseError {
        Type(InvalidIdType),
        Id(ParseIntError),
        InvalidFormat { found: String },
    }

    impl Display for IdParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                IdParseError::Type(ty) => f.write_fmt(format_args!(
                    "unable to parse the type portion of the id: {ty}"
                )),
                IdParseError::Id(id) => f.write_fmt(format_args!(
                    "unable to parse the numeric portion of the id: {id}"
                )),
                IdParseError::InvalidFormat { found } => f.write_fmt(format_args!(
                    "the id was not in the expected format `<ty>-<id>`: {found}"
                )),
            }
        }
    }

    impl Error for IdParseError {}

    impl<Type: IdType> FromStr for Id<Type> {
        type Err = IdParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.rsplit_once('-') {
                Some((ty, id)) => Ok(Self {
                    ty: ty.parse().map_err(IdParseError::Type)?,
                    id: id.parse().map_err(IdParseError::Id)?,
                }),
                None => Err(IdParseError::InvalidFormat {
                    found: s.to_string(),
                }),
            }
        }
    }

    impl<Type: IdType> TryFrom<&str> for Id<Type> {
        type Error = <Id<Type> as FromStr>::Err;

        fn try_from(value: &str) -> Result<Self, Self::Error> {
            value.parse()
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct InvalidIdType {
        pub expected: &'static str,
        pub found: String,
    }

    impl Display for InvalidIdType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!(
                "expected `{}`, found `{}`",
                self.expected, self.found,
            ))
        }
    }

    impl Error for InvalidIdType {}

    #[macro_export]
    macro_rules! id_type {
        (
            $(#[doc = $doc:literal])*
            #[ty = $ty:literal]
            pub struct $Struct:ident;
        ) => {
            #[derive(Debug, Clone, PartialEq)]
            pub struct $Struct;

            impl ::std::str::FromStr for $Struct {
                type Err = $crate::id::InvalidIdType;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    matches!(s, $ty)
                        .then_some(Self)
                        .ok_or($crate::id::InvalidIdType {
                            expected: $ty,
                            found: s.to_string(),
                        })
                }
            }

            impl ::std::fmt::Display for $Struct {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str($ty)
                }
            }

            impl serde::Serialize for $Struct {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.collect_str(self)
                }
            }

            impl<'de> serde::Deserialize<'de> for $Struct {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <&str>::deserialize(deserializer).and_then(|s| {
                        s.parse()
                            // TODO fix error situation
                            // FromStr::Err has no bounds
                            .map_err(|_| {
                                serde::de::Error::invalid_value(serde::de::Unexpected::Str(s), &$ty)
                            })
                    })
                }
            }

            impl $crate::id::IdType for $Struct {
                const TYPE: &'static str = $ty;
            }
        };
    }

    pub use id_type;

    id_type! {
        /// Id type for `connection_id`.
        #[ty = "connection"]
        pub struct Connection;
    }

    id_type! {
        /// Id for `channel_id`.
        #[ty = "channel"]
        pub struct Channel;
    }

    pub type ConnectionId = Id<Connection>;
    pub type ChannelId = Id<Channel>;
}

pub mod traits {
    use std::{
        error::Error,
        fmt::{Debug, Display},
        future::Future,
        hash::Hash,
        str::FromStr,
    };

    use serde::{Deserialize, Serialize};

    use crate::{
        ethereum::{H256, U256},
        ethereum_consts_traits::ChainSpec,
        ibc::{
            core::client::height::{Height, IsHeight},
            google::protobuf::any::Any,
            lightclients::{cometbls, ethereum, wasm},
        },
        id::ChannelId,
    };

    /// A convenience trait for a string id (`ChainId`, `ClientId`, `ConnectionId`, etc)
    pub trait Id:
        Debug
        + Clone
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + FromStr<Err = Self::FromStrErr>
        + Display
        + Send
        + Sync
        + 'static
    {
        type FromStrErr: Error;
    }

    impl Id for String {
        // type FromStrErr = <String as FromStr>::Err;
        type FromStrErr = std::string::ParseError;
    }

    /// Represents a chain. One [`Chain`] may have many related [`LightClient`]s for connecting to
    /// various other [`Chain`]s, all sharing a common config.
    pub trait Chain {
        type SelfClientState: Debug
            + Clone
            + PartialEq
            + Serialize
            + for<'de> Deserialize<'de>
            // TODO: Bound ChainId in the same way
            + ClientState<Height = Self::Height>;
        type SelfConsensusState: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

        type Header: Header + Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

        // this is just Height
        type Height: IsHeight;

        type ClientId: Id;

        /// Available client types for this chain.
        type ClientType: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

        fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId;

        fn query_latest_height(&self) -> impl Future<Output = Self::Height> + '_;

        fn query_latest_height_as_destination(&self) -> impl Future<Output = Self::Height> + '_;

        fn query_latest_timestamp(&self) -> impl Future<Output = i64> + '_;

        /// The client state on this chain at the specified `Height`.
        fn self_client_state(
            &self,
            height: Self::Height,
        ) -> impl Future<Output = Self::SelfClientState> + '_;

        /// The consensus state on this chain at the specified `Height`.
        fn self_consensus_state(
            &self,
            height: Self::Height,
        ) -> impl Future<Output = Self::SelfConsensusState> + '_;

        fn read_ack(
            &self,
            block_hash: H256,
            destination_channel_id: ChannelId,
            destination_port_id: String,
            sequence: u64,
        ) -> impl Future<Output = Vec<u8>> + '_;
    }

    pub trait ClientState {
        type ChainId: Debug
            + Display
            + PartialEq
            + Eq
            + Hash
            + Clone
            + Serialize
            + for<'de> Deserialize<'de>;
        type Height: IsHeight;

        fn height(&self) -> Self::Height;
        fn chain_id(&self) -> Self::ChainId;
    }

    impl ClientState for wasm::client_state::ClientState<ethereum::client_state::ClientState> {
        type ChainId = U256;
        type Height = Height;

        fn height(&self) -> Height {
            Height {
                revision_number: 0,
                revision_height: self.data.latest_slot,
            }
        }

        fn chain_id(&self) -> Self::ChainId {
            self.data.chain_id
        }
    }

    impl ClientState for wasm::client_state::ClientState<cometbls::client_state::ClientState> {
        type ChainId = String;
        type Height = Height;

        fn height(&self) -> Height {
            // NOTE: cometbls::ClientState doesn't store a height, as it's always wrapped in
            // wasm::ClientState (for our use cases)
            // TODO: Add it back
            self.latest_height
        }

        fn chain_id(&self) -> Self::ChainId {
            self.data.chain_id.clone()
        }
    }

    impl<T> ClientState for Any<T>
    where
        T: ClientState,
    {
        type ChainId = T::ChainId;
        type Height = T::Height;

        fn height(&self) -> Self::Height {
            self.0.height()
        }

        fn chain_id(&self) -> Self::ChainId {
            self.0.chain_id()
        }
    }

    pub trait Header {
        fn timestamp(&self) -> u64;
    }

    impl<C: ChainSpec> Header for wasm::header::Header<ethereum::header::Header<C>> {
        fn timestamp(&self) -> u64 {
            self.data
                .consensus_update
                .attested_header
                .execution
                .timestamp
        }
    }

    impl Header for cometbls::header::Header {
        fn timestamp(&self) -> u64 {
            self.signed_header
                .header
                .time
                .seconds
                .inner()
                .try_into()
                .unwrap()
        }
    }
}

/// An empty string. Will only parse/serialize to/from `""`.
#[derive(Debug, Clone, PartialEq)]
pub struct EmptyString;

#[derive(Debug, Clone, PartialEq)]
pub struct EmptyStringParseError {
    found: String,
}

impl Error for EmptyStringParseError {}

impl Display for EmptyStringParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "expected empty string, found `{}`", self.found)
    }
}

impl FromStr for EmptyString {
    type Err = EmptyStringParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self)
        } else {
            Err(EmptyStringParseError {
                found: s.to_string(),
            })
        }
    }
}

impl Display for EmptyString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("")
    }
}

impl Serialize for EmptyString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for EmptyString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            s.parse()
                .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &"an empty string"))
        })
    }
}

impl traits::Id for EmptyString {
    type FromStrErr = EmptyStringParseError;
}

pub use paste::paste;

#[macro_export]
macro_rules! export_wasm_client_type {
    ($type:ident) => {
        const _: unionlabs::WasmClientType = unionlabs::WasmClientType::$type;
        unionlabs::paste! {
            #[no_mangle]
            #[used]
            static [ <WASM_CLIENT_TYPE_ $type> ]: u8 = 0;
        }
        #[no_mangle]
        #[used]
        static WASM_CLIENT_TYPE_DEFINED: u8 = 0;
    };
}

/// This type is used to discriminate 08-wasm light clients.
/// We need to be able to determine the light client from the light client code itself (not instantiated yet).
/// Light clients supported by voyager must export a `#[no_mangle] static WASM_CLIENT_TYPE: WasmClientType = WasmClientType::...` variable.
#[derive(Debug, PartialEq, Eq)]
pub enum WasmClientType {
    EthereumMinimal,
    EthereumMainnet,
    Cometbls,
}

impl TryFrom<&[u8]> for WasmClientType {
    type Error = ();
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let wasm_type_name = wasmparser::Parser::new(0)
            .parse_all(value)
            .find_map(|payload| {
                payload.ok().and_then(|payload| match payload {
                    wasmparser::Payload::ExportSection(e) => Some(e),
                    _ => None,
                })
            })
            .and_then(|exports| {
                exports.into_iter().find_map(|export| {
                    export
                        .ok()
                        .and_then(|export| export.name.strip_prefix("WASM_CLIENT_TYPE_"))
                })
            })
            .ok_or(())?;
        Ok(match wasm_type_name {
            "EthereumMinimal" => WasmClientType::EthereumMinimal,
            "EthereumMainnet" => WasmClientType::EthereumMainnet,
            "Cometbls" => WasmClientType::Cometbls,
            _ => Err(())?,
        })
    }
}
