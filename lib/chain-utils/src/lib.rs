#![feature(return_position_impl_trait_in_trait)]
// #![warn(clippy::pedantic)]
#![allow(
     // required due to return_position_impl_trait_in_trait false positives
    clippy::manual_async_fn,
)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
};

use chrono::{DateTime, Utc};
use futures::{Future, Stream};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::{H256, U256},
    ethereum_consts_traits::ChainSpec,
    events::IbcEvent,
    ibc::{
        core::client::height::{Height, IsHeight},
        google::protobuf::any::Any,
        lightclients::{cometbls, ethereum, wasm},
    },
    id::ChannelId,
    traits::Id,
};

pub mod evm;
pub mod union;

pub mod private_key;

/// Represents a block chain. One [`Chain`] may have many related [`LightClient`]s for connecting to
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
        channel_id: ChannelId,
        port_id: String,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_;
}

pub trait EventSource {
    type Event;
    type Error: Debug;
    /// The initial state of this event source, if any.
    type Seed;

    fn events(&self, seed: Self::Seed)
        -> impl Stream<Item = Result<Self::Event, Self::Error>> + '_;
}

// Serialize, Deserialize
#[derive(Debug, Clone, PartialEq)]
pub struct ChainEvent<C: Chain> {
    /// The chain this event originated from.
    pub chain_id: <C::SelfClientState as ClientState>::ChainId,
    pub block_hash: H256,
    pub height: Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

pub trait ClientState {
    type ChainId: Debug + Display + PartialEq + Hash + Clone + Serialize + for<'de> Deserialize<'de>;
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

macro_rules! chain_client_id {
    (
        #[ty = $Ty:ident]
        pub enum $Enum:ident {
            $(
                // will wrap in `Id<_>`
                #[id(ty = $ty:literal)]
                $Variant:ident(Id<_>),
            )+
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum $Enum {
            $(
                $Variant(Id<$Variant>),
            )+
        }

        const _: () = {
            const STRINGS: &[&str] = &[$($ty),*];

            const LEN: usize = STRINGS.len();

            let mut i = 0;
            let mut j = 0;

            while i < LEN {
                while j < LEN {
                    if i == j {
                        j += 1;
                        continue;
                    } else {
                        assert!(!const_str_equal(STRINGS[i], STRINGS[j]), "strings are not unique");
                    }
                    j += 1;
                }
                i += 1;
            }

            // https://internals.rust-lang.org/t/why-i-cannot-compare-two-static-str-s-in-a-const-context/17726/8
            const fn const_bytes_equal(lhs: &[u8], rhs: &[u8]) -> bool {
                if lhs.len() != rhs.len() {
                    return false;
                }
                let mut i = 0;
                while i < lhs.len() {
                    if lhs[i] != rhs[i] {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            const fn const_str_equal(lhs: &str, rhs: &str) -> bool {
                const_bytes_equal(lhs.as_bytes(), rhs.as_bytes())
            }
        };

        $(
            impl From<Id<$Variant>> for $Enum {
                fn from(id: Id<$Variant>) -> Self {
                    Self::$Variant(id)
                }
            }

            impl TryFrom<$Enum> for Id<$Variant> {
                type Error = $Enum;

                fn try_from(value: $Enum) -> Result<Self, Self::Error> {
                    match value {
                        $Enum::$Variant(id) => Ok(id),
                        #[allow(unreachable_patterns)] // for when new types are added
                        _ => Err(value),
                    }
                }
            }

            impl From<$Variant> for $Ty {
                fn from(id: $Variant) -> Self {
                    Self::$Variant(id)
                }
            }

            impl TryFrom<$Ty> for $Variant {
                type Error = $Ty;

                fn try_from(value: $Ty) -> Result<Self, Self::Error> {
                    match value {
                        $Ty::$Variant(id) => Ok(id),
                        #[allow(unreachable_patterns)] // for when new types are added
                        _ => Err(value),
                    }
                }
            }
        )+

        #[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(untagged)]
        pub enum $Ty {
            $(
                $Variant($Variant),
            )+
        }

        $(
            id_type! {
                #[ty = $ty]
                pub struct $Variant;
            }
        )+

        impl crate::Id for $Enum {
            type FromStrErr = crate::ChainClientIdParseError;
        }

        impl FromStr for $Enum {
            type Err = crate::ChainClientIdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $(
                    if let Ok(ok) = s.parse::<Id<$Variant>>().map(Self::$Variant) {
                        return Ok(ok);
                    }
                )+

                Err(crate::ChainClientIdParseError {
                    expected: &[$($ty),+],
                    found: s.to_string(),
                })
            }
        }

        impl Display for $Enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(id) => f.write_fmt(format_args!("{id}")),
                    )+
                }
            }
        }


        impl FromStr for $Ty {
            type Err = crate::ChainClientIdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        $ty => Ok(Self::$Variant($Variant)),
                    )+
                    _ => Err(crate::ChainClientIdParseError {
                        expected: &[$($ty),+],
                        found: s.to_string(),
                    })
                }
            }
        }

        impl Display for $Ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$Variant(_) => f.write_str($ty),
                    )+
                }
            }
        }
    };
}

pub(crate) use chain_client_id;

// TODO: Make this a more generic error and put it in unionlabs::errors
#[derive(Debug, Clone, PartialEq)]
pub struct ChainClientIdParseError {
    expected: &'static [&'static str],
    found: String,
}

impl Display for ChainClientIdParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "expected one of `{}`, found `{}`",
            self.expected
                .iter()
                .map(|exp| format!("`{exp}`"))
                .collect::<Vec<_>>()
                .join(","),
            self.found,
        ))
    }
}

impl Error for ChainClientIdParseError {}
