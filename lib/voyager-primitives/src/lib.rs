#![feature(trait_alias)]
#![warn(clippy::pedantic)]

use core::{fmt, str::FromStr};
use std::fmt::{Debug, Display};

pub use consensus_primitives::{Duration, Timestamp};
use macros::apply;
use serde_json::Value;
use unionlabs::{
    ibc::core::client::height::{Height, HeightFromStrError},
    primitives::{encoding::HexUnprefixed, Bytes, H256},
};

/// Represents the IBC interface of a chain.
///
/// Since multiple chains with different consensus mechanisms can have the same
/// execution environment, this value is used to describe how the IBC state is
/// stored on-chain and how the IBC stack is to be interacted with. This notably
/// is NOT the IBC specification - an IBC interface can support multiple [`IbcSpec`]s.
#[apply(str_newtype)]
pub struct IbcInterface;

/// Well-known IBC interfaces, defined as constants for reusability and to allow
/// for pattern matching.
impl IbcInterface {
    /// Native light clients in ibc-go, through the client v1 router. This
    /// entrypoint uses protobuf [`Any`] wrapping to route to the correct
    /// module, such as "/ibc.lightclients.tendermint.v1.ClientState" for native
    /// 07-tendermint clients.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    pub const IBC_GO_V8_NATIVE: &'static str = "ibc-go-v8/native";

    /// 08-wasm light clients in ibc-go, through the client v1 router. Similar
    /// to the ibc-go-v8/native entrypoint, this module also uses [`Any`]
    /// wrapping for client routing, however, there is another level of
    /// indirection, since the `Any` routing only routes to the wasm module. All
    /// state for wasm clients is [wrapped](wasm-protos), with the internal
    /// state being opaque bytes to be interpreted by the light client.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    /// [wasm-protos]: https://github.com/cosmos/ibc-go/blob/release/v8.4.x/proto/ibc/lightclients/wasm/v1/wasm.proto
    pub const IBC_GO_V8_08_WASM: &'static str = "ibc-go-v8/08-wasm";

    /// Solidity light clients, run via Union's IBC solidity stack. This stack
    /// is fully virtualized in the EVM, and as such can be run on any chain
    /// running the EVM as part of their execution layer (ethereum, ethereum
    /// L2s, berachain, etc).
    pub const IBC_SOLIDITY: &'static str = "ibc-solidity";

    /// Light clients running on Union's cosmwasm IBC implementation.
    pub const IBC_COSMWASM: &'static str = "ibc-cosmwasm";

    pub const IBC_MOVE_APTOS: &'static str = "ibc-move/aptos";

    // lots more to come - near, fuel - stay tuned
}

/// Newtype for client types. Clients of the same type have the same client
/// state, consensus state, and header (client update) types.
#[apply(str_newtype)]
pub struct ClientType;

/// Well-known client types, defined as constants for reusability and to allow
/// for pattern matching.
impl ClientType {
    /// A client tracking [CometBLS] consensus, verified by manually verifying the state transition.
    ///
    /// NOTE: This is currently unused. See <https://github.com/unionlabs/union/issues/3066> for more information.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS: &'static str = "cometbls";

    /// A client tracking [CometBLS] consensus, verified with a ZK proof of the state transition
    /// created by galois.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS_GROTH16: &'static str = "cometbls";

    /// A client tracking vanilla [CometBFT] (Tendermint) consensus, through the [`07-tendermint`]
    /// light client specification.
    ///
    /// [CometBFT]: https://github.com/cometbft/cometbft
    /// [`07-tendermint`]: https://github.com/cosmos/ibc/blob/main/spec/client/ics-007-tendermint-client/README.md
    pub const TENDERMINT: &'static str = "tendermint";

    /// A client tracking the Ethereum beacon chain consensus verified through the
    /// [Ethereum Proof-of-Stake Consensus Specifications][spec].
    ///
    /// [spec]: https://github.com/ethereum/consensus-specs
    pub const ETHEREUM: &'static str = "ethereum";

    /// A client tracking the state of the [Scroll] zkEVM L2, settling on
    /// Ethereum, verified by verifying the L2 settlement on the L1.
    ///
    /// [Scroll]: https://github.com/scroll-tech/scroll
    pub const SCROLL: &'static str = "scroll";

    /// A client tracking the state of the [Arbitrum] optimistic L2, settling on
    /// Ethereum, verified by verifying the L2 settlement on the L1.
    ///
    /// [Arbitrum]: https://github.com/OffchainLabs/nitro-contracts
    pub const ARBITRUM: &'static str = "arbitrum";

    /// A client tracking the state of a Berachain chain, verified by verifying the underlying
    /// [BeaconKit] consensus.
    ///
    /// [BeaconKit]: https://github.com/berachain/beacon-kit
    pub const BERACHAIN: &'static str = "berachain";

    /// A client tracking the state of a [Movement] chain.
    ///
    /// [Movement]: https://github.com/movementlabsxyz/movement
    pub const MOVEMENT: &'static str = "movement";

    /// A client tracking the state of an [Ethermint] chain.
    ///
    /// [Ethermint]: https://github.com/0glabs/ethermint
    pub const ETHERMINT: &'static str = "ethermint";

    /// A client tracking an EVM-compatible chain utilizing [MPT] as the storage layer, verified
    /// through that chain's consensus as settled on an intermediary [ICS-23] chain.
    ///
    /// [MPT]: https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie
    /// [ICS-23]: https://github.com/cosmos/ics23
    pub const STATE_LENS_ICS23_MPT: &'static str = "state-lens/ics23/mpt";

    /// A client tracking an [ICS-23] chain, verified through that chain's consensus as
    /// settled on an intermediary [ICS-23] chain.
    ///
    /// [ICS23]: https://github.com/cosmos/ics23
    pub const STATE_LENS_ICS23_ICS23: &'static str = "state-lens/ics23/ics23";

    /// A client tracking an [Aptos] chain, verified through that chain's consensus as
    /// settled on an intermediary [ICS-23] chain.
    ///
    /// [ICS23]: https://github.com/cosmos/ics23
    /// [Aptos]: https://github.com/aptos-labs/aptos-core
    pub const STATE_LENS_ICS23_SMT: &'static str = "state-lens/ics23/smt";

    /// A client tracking an EVM-compatible chain utilizing [MPT] as the storage layer. The consensus
    /// level verification is trusted, hence no beacon chain configuration is needed.
    ///
    /// [MPT]: https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie
    pub const TRUSTED_MPT: &'static str = "trusted/evm/mpt";

    /// A client tracking the state of the [Bob] optimistic L2, settling on
    /// Ethereum, verified by verifying the L2 settlement on the L1.
    ///
    /// [Bob]: https://github.com/ethereum-optimism/optimism/blob/v1.7.2/packages/contracts-bedrock/src/L1/L2OutputOracle.sol
    pub const BOB: &'static str = "bob";

    // lots more to come - near, linea, polygon - stay tuned
}

/// Newtype for consensus types. A consensus is verifiable by potentially many [`ClientType`]s.
#[apply(str_newtype)]
pub struct ConsensusType;

/// Well-known consensus types, defined as constants for reusability and to allow
/// for pattern matching.
impl ConsensusType {
    /// [CometBLS] consensus.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS: &'static str = "cometbls";

    /// [CometBFT] (Tendermint) consensus.
    ///
    /// [CometBFT]: https://github.com/cometbft/cometbft
    pub const TENDERMINT: &'static str = "tendermint";

    /// Ethereum beacon chain consensus.
    pub const ETHEREUM: &'static str = "ethereum";

    /// [Scroll] zkEVM L2, settling on Ethereum.
    ///
    /// [Scroll]: https://github.com/scroll-tech/scroll
    pub const SCROLL: &'static str = "scroll";

    /// [Arbitrum] optimistic L2, settling on Ethereum.
    ///
    /// [Arbitrum]: https://github.com/OffchainLabs/nitro-contracts
    pub const ARBITRUM: &'static str = "arbitrum";

    /// Berachain consensus ([BeaconKit]).
    ///
    /// [BeaconKit]: https://github.com/berachain/beacon-kit
    pub const BERACHAIN: &'static str = "berachain";

    /// [Movement] consensus.
    ///
    /// [Movement]: https://github.com/movementlabsxyz/movement
    pub const MOVEMENT: &'static str = "movement";

    /// [Bob] optimistic L2, settling on Ethereum.
    ///
    /// [Bob]: https://github.com/ethereum-optimism/optimism/blob/v1.7.2/packages/contracts-bedrock/src/L1/L2OutputOracle.sol
    pub const BOB: &'static str = "bob";

    /// Trusted consensus of an [Ethereum JSON-RPC] compatible chain.
    ///
    /// <div class="warning">
    /// Blocks are considered finalized after a configurable delay period. This is not secure, and should only be used in development or testing environments.
    /// </div>
    ///
    /// [Ethereum JSON-RPC]: https://ethereum.github.io/execution-apis/api-documentation/
    pub const TRUSTED_EVM: &'static str = "trusted/evm";

    // lots more to come - near, linea, polygon - stay tuned
}

#[cfg(feature = "serde")]
pub trait MaybeSerde = serde::Serialize + serde::de::DeserializeOwned;

#[cfg(not(feature = "serde"))]
pub trait MaybeSerde =;

/// An IBC specification describes the format of the store, datagrams, and events.
///
/// Typically, an IBC interface will support exactly one IBC version, however
/// it is possible to support multiple. For example, the union virtualized IBC
/// stack on cosmwasm will support both IBC classic *and* the union IBC
/// specification.
///
/// [State lenses] are possible between clients on IBC interfaces that support the
/// same IBC spec.
///
/// [State lenses]: https://research.union.build/State-Lenses-9e3d6578ec0e48fca8e502a0d28f485c
pub trait IbcSpec {
    const ID: IbcSpecId;

    type ClientId: Display + Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;

    /// The type used to index into the IBC store.
    type StorePath: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;

    /// Arbitrary queries that must be supported by all implementations of this spec.
    type Query: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;

    /// The messages submitted on chain.
    type Datagram: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;

    /// Events emitted on chain.
    type Event: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;

    // TODO: Move this to Datagram
    fn update_client_datagram(client_id: Self::ClientId, client_message: Bytes) -> Self::Datagram;

    // TODO: Move these to Path
    fn client_state_path(client_id: Self::ClientId) -> Self::StorePath;
    fn consensus_state_path(client_id: Self::ClientId, height: Height) -> Self::StorePath;
}

/// A subset of [`IbcSpec::StorePath`]. This should be implemented by all variants of the
/// `StorePath` enum for an `IbcSpec` implementation.
pub trait IbcStorePathKey:
    Debug
    + Clone
    + PartialEq
    + MaybeSerde
    + Send
    + Sync
    + Unpin
    + 'static
    + TryFrom<<Self::Spec as IbcSpec>::StorePath, Error = <Self::Spec as IbcSpec>::StorePath>
    + Into<<Self::Spec as IbcSpec>::StorePath>
{
    /// The [`IbcSpec`] that this store path key indexes into.
    type Spec: IbcSpec;

    /// The value stored under this key. Note that this is NOT the *commitment*, but rather the
    /// actual value.
    type Value: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;
}

/// A subset of [`IbcSpec::Query`]. This should be implemented by all variants of the
/// `Query` enum for an `IbcSpec` implementation.
pub trait IbcQuery:
    Debug
    + Clone
    + PartialEq
    + MaybeSerde
    + Send
    + Sync
    + Unpin
    + 'static
    + TryFrom<<Self::Spec as IbcSpec>::Query, Error = <Self::Spec as IbcSpec>::Query>
    + Into<<Self::Spec as IbcSpec>::Query>
{
    /// The [`IbcSpec`] that this query is for.
    type Spec: IbcSpec;

    /// The type returned by this query.
    type Value: Debug + Clone + PartialEq + MaybeSerde + Send + Sync + Unpin + 'static;
}

/// An identifier for an [`IbcSpec`].
#[apply(str_newtype)]
pub struct IbcSpecId;

/// Well-known IBC spec identifiers, defined as constants for reusability and to allow
/// for pattern matching.
impl IbcSpecId {
    /// IBC classic, as per the [ICS-003 connection semantics][ics3].
    ///
    /// [ics3]: https://github.com/cosmos/ibc/blob/main/spec/core/ics-003-connection-semantics/README.md#versioning
    pub const CLASSIC: &'static str = "ibc-classic";

    /// IBC union, as per the [union IBC specification](ibc-union).
    ///
    /// [ibc-union]: https://docs.union.build/protocol/specifications/ibc/
    pub const UNION: &'static str = "ibc-union";
}

/// Identifier used to uniquely identify a chain, as provided by the chain
/// itself.
///
/// # Examples
///
/// | chain id        | chain                    |
/// | --------------- | ------------------------ |
/// | 1               | ethereum mainnet         |
/// | 11155111        | ethereum sepolia testnet |
/// | union-testnet-8 | union testnet            |
/// | stargaze-1      | stargaze mainnet         |
#[apply(str_newtype)]
pub struct ChainId;

/// The type of a light client on a chain, along with the IBC interface it's on
/// (and any associated metadata).
///
/// # Examples
///
/// - 08-wasm client on union, tracking ethereum mainnet: `(ibc-go-v8/08-wasm, ethereum_mainnet,
///   {"checksum": "0x..."})`
/// - 07-tendermint client on stargaze, tracking osmosis: `(ibc-go-v8/native, tendermint)`
/// - 08-wasm client on babylon, tracking union: `(ibc-go-v8/08-wasm, cometbls, {"checksum":
///   "0x..."}))`
/// - cometbls client on scroll, tracking union: `(ibc-solidity, cometbls)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub ibc_interface: IbcInterface,
    /// Additional metadata about this client.
    ///
    /// This is currently only used for threading the checksum for ibc-go
    /// 08-wasm clients, and can likely be removed when support for that IBC
    /// interface is dropped.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Value::is_null")
    )]
    pub metadata: Value,
}

/// Metadata about a client, as it exists at a specific height.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
// TODO: Add status
pub struct ClientStateMeta {
    /// The counterparty height this client has been updated to. A consensus
    /// state will exist at this height.
    #[doc(alias = "trusted_height")]
    pub counterparty_height: Height,

    /// The chain id of the counterparty chain this client tracks.
    pub counterparty_chain_id: ChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ConsensusStateMeta {
    /// The timestamp of the counterparty at the height represented by this
    /// consensus state.
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct IbcGo08WasmClientMetadata {
    pub checksum: H256<HexUnprefixed>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub enum QueryHeight {
    /// The latest, potentially unfinalized block (the head of the chain).
    #[cfg_attr(feature = "serde", serde(rename = "latest"))]
    Latest,
    /// The latest finalized block.
    #[cfg_attr(feature = "serde", serde(rename = "finalized"))]
    Finalized,
    /// A specific block that may or not be finalized.
    #[cfg_attr(feature = "serde", serde(untagged))]
    Specific(Height),
}

impl From<Height> for QueryHeight {
    fn from(height: Height) -> Self {
        Self::Specific(height)
    }
}

impl fmt::Display for QueryHeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryHeight::Latest => f.write_str("latest"),
            QueryHeight::Finalized => f.write_str("finalized"),
            QueryHeight::Specific(height) => f.write_fmt(format_args!("{height}")),
        }
    }
}

impl FromStr for QueryHeight {
    type Err = HeightFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            "finalized" => Ok(Self::Finalized),
            _ => s.parse().map(Self::Specific),
        }
    }
}

macro_rules! str_newtype {
    (
        $(#[doc = $doc:literal])+
        $vis:vis struct $Struct:ident;
    ) => {
        $(#[doc = $doc])+
        #[derive(
            macros::Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
        )]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            // I tested this and apparently it's not required (newtype is automatically transparent?) but
            // keeping it here for clarity
            serde(transparent)
        )]
        #[cfg_attr(feature = "schemars", derive(::schemars::JsonSchema))]
        #[debug("{}({:?})", stringify!($Struct), self.0)]
        $vis struct $Struct(#[doc(hidden)] ::std::borrow::Cow<'static, str>);

        impl ::core::fmt::Display for $Struct {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        impl PartialEq<&$Struct> for $Struct {
            fn eq(&self, other: &&$Struct) -> bool {
                self == *other
            }
        }

        impl PartialEq<$Struct> for &$Struct {
            fn eq(&self, other: &$Struct) -> bool {
                *self == other
            }
        }

        #[allow(unused)]
        impl $Struct {
            /// Construct a new [`
            #[doc = stringify!($Struct)]
            /// `].
            pub fn new(s: impl Into<::std::borrow::Cow<'static, str>>) -> Self {
                Self(s.into())
            }

            /// Extracts a string slice containing the entire contained value.
            #[must_use = "getting a reference to the contained string slice has no effect"]
            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl $Struct {
            /// `const`-friendly version of [`Self::new`].
            #[must_use = concat!("constructing a ", stringify!($Struct), " has no effect")]
            pub const fn new_static(ibc_interface: &'static str) -> Self {
                Self(::std::borrow::Cow::Borrowed(ibc_interface))
            }
        }
    };
}
pub(crate) use str_newtype;
