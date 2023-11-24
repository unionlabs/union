#![deny(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::{
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
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    errors::TryFromBranchError,
    hash::H256,
    ibc::core::client::height::{HeightFromStrError, IsHeight},
    id::Bounded,
    validated::Validated,
};

pub const DELAY_PERIOD: u64 = 0;

/// Wrapper types around protos defined in <https://github.com/cosmos/gogoproto/tree/main/protobuf/google/protobuf>, matching the proto module structure.
pub mod google;

pub mod cosmwasm;

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

pub mod bounded;

pub mod proof;

pub mod validated;

pub mod hash;

// TODO: Replace with something like <https://github.com/recmo/uint>
pub mod uint;

pub mod ics23;

pub(crate) mod macros;

pub mod errors {
    use std::fmt::{Debug, Display};

    use crate::hash::H256;

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
    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    #[error("invalid length: expected {expected}, found {found}")]
    pub struct InvalidLength {
        pub expected: ExpectedLength,
        pub found: usize,
    }

    #[derive(Debug, PartialEq, Eq, derive_more::Display)]
    pub enum ExpectedLength {
        #[display(fmt = "exactly {_0}")]
        Exact(usize),
        #[display(fmt = "less than {_0}")]
        LessThan(usize),
        #[display(fmt = "between ({_0}, {_1})")]
        Between(usize, usize),
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

/// Due to the broken eth abi rust library, some structures with dynamically
/// sized types are incorrectly encoded (missing a dynamic tuple wrapper)
#[cfg(feature = "ethabi")]
pub struct InlineFields<T>(pub T);

#[cfg(feature = "ethabi")]
impl<T> From<T> for InlineFields<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[cfg(feature = "ethabi")]
impl<T> ethers_core::abi::AbiEncode for InlineFields<T>
where
    T: ethers_core::abi::AbiEncode,
{
    fn encode(self) -> Vec<u8> {
        // Prefixed by the offset at which the 'dynamic' tuple is starting
        ethers_core::types::U256::from(32)
            .encode()
            .into_iter()
            .chain(self.0.encode())
            .collect::<Vec<_>>()
    }
}

#[cfg(feature = "ethabi")]
impl<T> ethers_core::abi::AbiDecode for InlineFields<T>
where
    T: ethers_core::abi::AbiDecode,
{
    fn decode(bytes: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
        // Wipe the prefix
        Ok(Self(T::decode(
            bytes
                .as_ref()
                .iter()
                .copied()
                .skip(core::mem::size_of::<ethers_core::types::U256>())
                .collect::<Vec<_>>(),
        )?))
    }
}

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
    use std::fmt::Debug;

    use crate::{
        errors::{ExpectedLength, InvalidLength},
        validated::{Validate, Validated},
    };

    pub type PortIdValidator = (Bounded<2, 128>, Ics024IdentifierCharacters);
    pub type PortId = Validated<String, PortIdValidator>;
    pub type ClientIdValidator = (Bounded<9, 64>, Ics024IdentifierCharacters);
    pub type ClientId = Validated<String, ClientIdValidator>;
    pub type ConnectionIdValidator = (Bounded<10, 64>, Ics024IdentifierCharacters);
    pub type ConnectionId = Validated<String, ConnectionIdValidator>;
    pub type ChannelIdValidator = (Bounded<8, 64>, Ics024IdentifierCharacters);
    pub type ChannelId = Validated<String, ChannelIdValidator>;

    #[test]
    fn cid2() {
        let c = ChannelId::new("channel-1".into()).unwrap();

        dbg!(c);
    }

    // https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators
    pub struct Ics024IdentifierCharacters;

    #[derive(Debug, thiserror::Error)]
    #[error("invalid ics-024 identifier character: `{0}`")]
    pub struct InvalidIcs024IdentifierCharacter(char);

    impl<T: AsRef<str>> Validate<T> for Ics024IdentifierCharacters {
        type Error = InvalidIcs024IdentifierCharacter;

        fn validate(t: T) -> Result<T, Self::Error> {
            for c in t.as_ref().chars() {
                match c {
                    'a'..='z'
                    | 'A'..='Z'
                    | '0'..='9'
                    | '.'
                    | '_'
                    | '+'
                    | '-'
                    | '#'
                    | '['
                    | ']'
                    | '<'
                    | '>' => {}
                    _ => return Err(InvalidIcs024IdentifierCharacter(c)),
                }
            }

            Ok(t)
        }
    }

    pub struct Bounded<const MIN: usize, const MAX: usize>;

    impl<T: AsRef<str>, const MIN: usize, const MAX: usize> Validate<T> for Bounded<MIN, MAX> {
        type Error = InvalidLength;

        fn validate(s: T) -> Result<T, Self::Error> {
            let len = s.as_ref().len();

            if (MIN..=MAX).contains(&len) {
                Ok(s)
            } else {
                Err(InvalidLength {
                    expected: ExpectedLength::Between(MIN, MAX),
                    found: len,
                })
            }
        }
    }
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
        ethereum::config::ChainSpec,
        google::protobuf::any::Any,
        hash::H256,
        ibc::{
            core::{
                channel::channel::Channel,
                client::height::{Height, IsHeight},
                connection::connection_end::ConnectionEnd,
            },
            lightclients::{cometbls, ethereum, wasm},
        },
        id::{ChannelId, ConnectionId, PortId},
        proof::IbcStateReadPaths,
        uint::U256,
        validated::{Validate, Validated},
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

    impl<T: Id, V: Validate<T> + 'static> Id for Validated<T, V>
    where
        T::FromStrErr: Error,
        V::Error: Error,
    {
        type FromStrErr = <Self as FromStr>::Err;
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

        type Error: Debug;

        fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId;

        fn query_latest_height(
            &self,
        ) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_;

        fn query_latest_height_as_destination(
            &self,
        ) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_;

        fn query_latest_timestamp(&self) -> impl Future<Output = Result<i64, Self::Error>> + '_;

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
            destination_port_id: PortId,
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

    impl ClientState for ethereum::client_state::ClientState {
        type ChainId = U256;
        type Height = Height;

        fn height(&self) -> Self::Height {
            Height {
                // TODO: Make EVM_REVISION_NUMBER a constant in this crate
                revision_number: 0,
                revision_height: self.latest_slot,
            }
        }

        fn chain_id(&self) -> Self::ChainId {
            self.chain_id
        }
    }

    impl<Data: ClientState> ClientState for wasm::client_state::ClientState<Data> {
        type ChainId = Data::ChainId;
        type Height = Data::Height;

        fn height(&self) -> Data::Height {
            self.data.height()
        }

        fn chain_id(&self) -> Self::ChainId {
            self.data.chain_id()
        }
    }

    impl ClientState for cometbls::client_state::ClientState {
        type ChainId = String;
        type Height = Height;

        fn height(&self) -> Height {
            self.latest_height
        }

        fn chain_id(&self) -> Self::ChainId {
            self.chain_id.clone()
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
    /// The IBC interface on a [`Chain`] that knows how to connect to a counterparty.
    pub trait LightClientBase: Send + Sync + Sized {
        /// The chain that this light client is on.
        type HostChain: Chain
            + IbcStateReadPaths<<Self::Counterparty as LightClientBase>::HostChain>;
        type Counterparty: LightClientBase<Counterparty = Self>;

        /// The config required to construct this light client.
        type Config: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de>;

        /// Get the underlying [`Self::HostChain`] that this client is on.
        fn chain(&self) -> &Self::HostChain;

        fn from_chain(chain: Self::HostChain) -> Self;

        fn channel(
            &self,
            channel_id: ChannelId,
            port_id: PortId,
            at: HeightOf<Self::HostChain>,
        ) -> impl Future<Output = Channel> + '_;

        fn connection(
            &self,
            connection_id: ConnectionId,
            at: HeightOf<Self::HostChain>,
        ) -> impl Future<
            Output = ConnectionEnd<
                <Self::HostChain as Chain>::ClientId,
                <<Self::Counterparty as LightClientBase>::HostChain as Chain>::ClientId,
                String,
            >,
        > + '_;

        // TODO: Use state_proof instead
        fn query_client_state(
            &self,
            // TODO: Make this Into<_>
            client_id: <Self::HostChain as Chain>::ClientId,
            height: HeightOf<Self::HostChain>,
        ) -> impl Future<Output = ClientStateOf<<Self::Counterparty as LightClientBase>::HostChain>> + '_;
    }

    pub type ClientStateOf<C> = <C as Chain>::SelfClientState;
    pub type ConsensusStateOf<C> = <C as Chain>::SelfConsensusState;
    pub type HeaderOf<C> = <C as Chain>::Header;
    pub type HeightOf<C> = <C as Chain>::Height;
    pub type ChainOf<L> = <L as LightClientBase>::HostChain;
    pub type ChainIdOf<L> =
        <<<L as LightClientBase>::HostChain as Chain>::SelfClientState as ClientState>::ChainId;
    pub type ClientIdOf<C> = <C as Chain>::ClientId;
    pub type ClientTypeOf<C> = <C as Chain>::ClientType;
}

/// An empty string. Will only parse/serialize to/from `""`.
pub type EmptyString<S = String> = Validated<S, EmptyStringValidator>;
pub type EmptyStringValidator = Bounded<0, 0>;

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
    };
}

/// This type is used to discriminate 08-wasm light clients.
/// We need to be able to determine the light client from the light client code itself (not instantiated yet).
/// Light clients supported by voyager must export a `#[no_mangle] static WASM_CLIENT_TYPE: WasmClientType = WasmClientType::...` variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmClientType {
    EthereumMinimal,
    EthereumMainnet,
    Cometbls,
}

impl FromStr for WasmClientType {
    type Err = WasmClientTypeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EthereumMinimal" => Ok(WasmClientType::EthereumMinimal),
            "EthereumMainnet" => Ok(WasmClientType::EthereumMainnet),
            "Cometbls" => Ok(WasmClientType::Cometbls),
            _ => Err(WasmClientTypeParseError::UnknownType(s.to_string())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WasmClientTypeParseError {
    #[error("unknown wasm client type `{0}`")]
    UnknownType(String),
}

pub fn parse_wasm_client_type(
    bz: impl AsRef<[u8]>,
) -> Result<Option<WasmClientType>, WasmClientTypeParseError> {
    wasmparser::Parser::new(0)
        .parse_all(bz.as_ref())
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
        .map(str::parse)
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "&str",
    into = "String",
    bound(serialize = "", deserialize = "")
)]
pub enum QueryHeight<H: IsHeight> {
    Latest,
    Specific(H),
}

impl<H: IsHeight> Display for QueryHeight<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryHeight::Latest => f.write_str("latest"),
            QueryHeight::Specific(height) => f.write_fmt(format_args!("{height}")),
        }
    }
}

impl<H: IsHeight> From<QueryHeight<H>> for String {
    fn from(val: QueryHeight<H>) -> Self {
        val.to_string()
    }
}

impl<H: IsHeight> FromStr for QueryHeight<H> {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            _ => s.parse().map(Self::Specific),
        }
    }
}

impl<H: IsHeight> TryFrom<&'_ str> for QueryHeight<H> {
    type Error = HeightFromStrError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

pub trait MaybeRecoverableError: std::error::Error {
    fn is_recoverable(&self) -> bool;
}

fn _is_object_safe(_: &dyn MaybeRecoverableError) {}
