#![warn(clippy::pedantic)]

use macros::{apply, model};
use serde_json::Value;
use unionlabs::{hash::H256, ibc::core::client::height::Height};

/// Represents the IBC interface of a chain.
///
/// Since multiple chains with different consensus mechanisms can have the same
/// execution environment, this value is used to describe how the IBC state is
/// stored on-chain and how the IBC stack is to be interacted with.
#[apply(str_newtype)]
pub struct IbcInterface;

/// Well-known IBC interfaces, defined as constants for reusability and to allow
/// for pattern matching.
impl IbcInterface<'static> {
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

    pub const IBC_MOVE_APTOS: &'static str = "ibc-move/aptos";

    // lots more to come - near, fuel - stay tuned
}

/// Newtype for client types. Clients of the same type have the same client
/// state, consensus state, and header (client update) types.
#[apply(str_newtype)]
pub struct ClientType;

/// Well-known client types, defined as constants for reusability and to allow
/// for pattern matching.
impl ClientType<'static> {
    /// A client tracking [CometBLS] consensus, verified by manually verifying the state transition.
    ///
    /// NOTE: This is currently unused. See <https://github.com/unionlabs/union/issues/3066> for more information.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS: &'static str = "cometbls";

    /// A client tracking [CometBLS] consensus, verified with a ZK proof of the state transition created by galois.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS_GROTH16: &'static str = "cometbls";

    /// A client tracking vanilla [CometBFT] (Tendermint) consensus, through the [`07-tendermint`] light client specification.
    ///
    /// [CometBFT]: https://github.com/cometbft/cometbft
    /// [`07-tendermint`]: https://github.com/cosmos/ibc/blob/main/spec/client/ics-007-tendermint-client/README.md
    pub const TENDERMINT: &'static str = "07-tendermint";

    // TODO: Consolidate these two into one client type after https://github.com/unionlabs/union/issues/3006
    /// A client tracking the Ethereum beacon chain consensus, with the mainnet
    /// configuration. Verified through the [Ethereum Proof-of-Stake Consensus Specifications](spec).
    ///
    /// [spec]: https://github.com/ethereum/consensus-specs
    pub const ETHEREUM_MAINNET: &'static str = "ethereum-mainnet";

    /// A client tracking the Ethereum beacon chain consensus, with the minimal
    /// configuration. Verified through the [Ethereum Proof-of-Stake Consensus Specifications](spec).
    ///
    /// [spec]: https://github.com/ethereum/consensus-specs
    pub const ETHEREUM_MINIMAL: &'static str = "ethereum-minimal";

    /// A client tracking the state of the [Scroll] zkevm L2, settling on
    /// Ethereum, verified by verifying the L2 settlement on the L1.
    ///
    /// [Scroll]: https://github.com/scroll-tech/scroll
    pub const SCROLL: &'static str = "scroll";

    /// A client tracking the state of the [Arbitrum] optimistic L2, settling on
    /// Ethereum, verified by verifying the L2 settlement on the L1.
    ///
    /// [Arbitrum]: https://github.com/OffchainLabs/nitro-contracts
    pub const ARBITRUM: &'static str = "arbitrum";

    /// A client tracking the state of a [BeaconKit] chain, verified by verifying the underlying [CometBFT] consensus.
    ///
    /// [BeaconKit]: https://github.com/berachain/beacon-kit
    /// [CometBFT]: https://github.com/cometbft/cometbft
    pub const BEACON_KIT: &'static str = "beacon-kit";

    /// A client tracking the state of a [Movement] chain.
    ///
    /// [Movement]: https://github.com/movementlabsxyz/movement
    pub const MOVEMENT: &'static str = "movement";

    // lots more to come - near, linea, polygon - stay tuned
}

/// Newtype for consensus types. A consensus is verifiable by potentially many [`ClientType`]s.
#[apply(str_newtype)]
pub struct ConsensusType;

/// Well-known consensus types, defined as constants for reusability and to allow
/// for pattern matching.
impl ConsensusType<'static> {
    /// [CometBLS] consensus.
    ///
    /// [CometBLS]: https://github.com/unionlabs/cometbls
    pub const COMETBLS: &'static str = "cometbls";

    /// [CometBFT] (Tendermint) consensus.
    ///
    /// [CometBFT]: https://github.com/cometbft/cometbft
    pub const TENDERMINT: &'static str = "tendermint";

    /// Ethereum beacon chain consensus, with the mainnet
    /// configuration.
    pub const ETHEREUM_MAINNET: &'static str = "ethereum-mainnet";

    /// Ethereum beacon chain consensus, with the minimal
    /// configuration.
    pub const ETHEREUM_MINIMAL: &'static str = "ethereum-minimal";

    /// [Scroll] zkevm L2, settling on Ethereum.
    ///
    /// [Scroll]: https://github.com/scroll-tech/scroll
    pub const SCROLL: &'static str = "scroll";

    /// [Arbitrum] optimistic L2, settling on Ethereum.
    ///
    /// [Arbitrum]: https://github.com/OffchainLabs/nitro-contracts
    pub const ARBITRUM: &'static str = "arbitrum";

    /// [BeaconKit] consensus.
    ///
    /// [BeaconKit]: https://github.com/berachain/beacon-kit
    pub const BEACON_KIT: &'static str = "beacon-kit";

    /// [Movement] consensus.
    ///
    /// [Movement]: https://github.com/movementlabsxyz/movement
    pub const MOVEMENT: &'static str = "movement";

    // lots more to come - near, linea, polygon - stay tuned
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
/// - 08-wasm client on union, tracking ethereum mainnet: `(ibc-go-v8/08-wasm,
///   ethereum_mainnet, {"checksum": "0x..."})`
/// - 07-tendermint client on stargaze, tracking osmosis: `(ibc-go-v8/native,
///   tendermint)`
/// - 08-wasm client on babylon, tracking union: `(ibc-go-v8/08-wasm, cometbls,
///   {"checksum": "0x..."}))`
/// - cometbls client on scroll, tracking union: `(ibc-solidity, cometbls)`
#[model]
pub struct ClientInfo {
    pub client_type: ClientType<'static>,
    pub ibc_interface: IbcInterface<'static>,
    /// Additional metadata about this client.
    ///
    /// This is currently only used for threading the checksum for ibc-go
    /// 08-wasm clients, and can likely be removed when support for that IBC
    /// interface is dropped.
    #[serde(default)]
    pub metadata: Value,
}

#[model]
pub struct ClientStateMeta {
    /// The counterparty height this client has been updated to. A consensus
    /// state will exist at this height.
    pub height: Height,

    /// The chain id of the counterparty chain this client tracks.
    pub chain_id: ChainId<'static>,
}

#[model]
pub struct ConsensusStateMeta {
    /// The timestamp of the counterparty at the height represented by this
    /// consensus state.
    pub timestamp_nanos: u64,
}

#[model]
pub struct IbcGo08WasmClientMetadata {
    pub checksum: H256,
}

#[macro_export]
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
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::schemars::JsonSchema
        )]
        // I tested this and apparently it's not required (newtype is automatically transparent?) but
        // keeping it here for clarity
        #[serde(transparent)]
        #[debug("{}({:?})", stringify!($Struct), self.0)]
        $vis struct $Struct<'a>(#[doc(hidden)] ::std::borrow::Cow<'a, str>);

        impl<'a> ::core::fmt::Display for $Struct<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        #[allow(unused)]
        impl<'a> $Struct<'a> {
            /// Construct a new [`
            #[doc = stringify!($Struct)]
            /// `].
            ///
            /// This will capture the lifetime of the passed in value:
            ///
            /// ```
            /// # use voyager_core::*;
            #[doc = concat!(
                "let _: ",
                stringify!($Struct),
                "<'static> = ",
                stringify!($Struct),
                "::new(\"static string\");"
            )]
            /// let owned_string: String = "owned string".into();
            ///
            /// // not static
            #[doc = concat!(
                "let _: ",
                stringify!($Struct),
                "<'_> = ",
                stringify!($Struct),
                "::new(&owned_string);"
            )]
            #[doc = concat!(
                "let _: ",
                stringify!($Struct),
                "<'static> = ",
                stringify!($Struct),
                "::new(owned_string);"
            )]
            pub fn new(s: impl Into<::std::borrow::Cow<'a, str>>) -> Self {
                Self(s.into())
            }

            /// Convert this [`
            #[doc = concat!(stringify!($Struct))]
            /// `] into an owned version of itself.
            ///
            /// This will allocate if the contained value is not already on the heap even if `'a == 'static`.
            #[must_use = concat!("converting to an owned version of ", stringify!($Struct), " has no effect other than possibly allocating, if the returned value is not needed then the call to this method can be removed altogether and the value dropped directly")]
            pub fn into_owned(self) -> $Struct<'static> {
                use std::borrow::Cow;

                $Struct(match self.0 {
                    Cow::Borrowed(x) => Cow::Owned(x.to_owned()),
                    Cow::Owned(x) => Cow::Owned(x),
                })
            }

            /// Extracts a string slice containing the entire contained value.
            #[must_use = "getting a reference to the contained string slice has no effect"]
            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }


            /// Borrow this [`
            #[doc = stringify!($Struct)]
            /// `], returning a new owned value pointing to the same data.
            ///
            /// ```
            /// # use voyager_core::*;
            #[doc = concat!("let t = ", stringify!($Struct), "::new_static(\"static\");")]
            ///
            /// takes_ownership(t.borrow());
            /// takes_ownership(t);
            ///
            #[doc = concat!("fn takes_ownership<'a>(c: ", stringify!($Struct), "<'a>) {}")]
            /// ```
            #[must_use = "borrowing the inner value has no effect"]
            pub fn borrow<'b>(&'a self) -> $Struct<'b>
            where
                'a: 'b,
            {
                use std::borrow::Cow;

                match self.0 {
                    Cow::Borrowed(s) => Self(Cow::Borrowed(s)),
                    Cow::Owned(ref s) => Self(Cow::Borrowed(s.as_str())),
                }
            }
        }

        impl $Struct<'static> {
            /// `const`-friendly version of [`Self::new`].
            #[must_use = concat!("constructing a ", stringify!($Struct), " has no effect")]
            pub const fn new_static(ibc_interface: &'static str) -> Self {
                Self(::std::borrow::Cow::Borrowed(ibc_interface))
            }
        }
    };
}
