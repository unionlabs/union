use core::{
    fmt::{Debug, Display},
    future::Future,
    hash::Hash,
    num::NonZeroU64,
    str::FromStr,
};
use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::{
    encoding::Encoding,
    ethereum::config::ChainSpec,
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::client::height::{Height, IsHeight},
        lightclients::{cometbls, ethereum, scroll, tendermint, wasm},
    },
    id::{ChannelId, PortId},
    uint::U256,
    validated::{Validate, Validated},
    MaybeArbitrary, TypeUrl,
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
    type FromStrErr: Error + Send + Sync + 'static;
}

impl Id for String {
    // type FromStrErr = <String as FromStr>::Err;
    type FromStrErr = alloc::string::ParseError;
}

impl<T: Id, V: Validate<T> + 'static> Id for Validated<T, V>
where
    T::FromStrErr: Error,
    V::Error: Error + Send + Sync + 'static,
{
    type FromStrErr = <Self as FromStr>::Err;
}

pub trait FromStrExact: Default + Sized {
    const EXPECTING: &'static str;
}

pub mod from_str_exact {
    use serde::{de, Deserialize, Deserializer};

    use crate::traits::FromStrExact;

    pub fn serialize<S, T: FromStrExact>(_: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(T::EXPECTING)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStrExact,
    {
        let s = <&str>::deserialize(deserializer)?;
        if s == T::EXPECTING {
            Ok(T::default())
        } else {
            Err(de::Error::invalid_value(
                de::Unexpected::Str(s),
                &T::EXPECTING,
            ))
        }
    }
}

pub trait Member = Debug
    + Clone
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + Send
    + Sync
    + Unpin
    + MaybeArbitrary
    + 'static;

/// Represents a chain. One [`Chain`] may have many related [`LightClient`]s for connecting to
/// various other [`Chain`]s, all sharing a common config.
pub trait Chain: Sized + Send + Sync + 'static {
    /// Expected to be unique across all implementations. Note that Wasm<_> implements this by passing through to the host chain, as `Wasm<A> <-> Wasm<B>` and `A <-> B` simultaneously is not currently supported.
    type ChainType: FromStrExact;
    type SelfClientState: Member
        + TypeUrl // hack
        // TODO: Bound ChainId in the same way
        + ClientState<Height = Self::Height>;
    type SelfConsensusState: Member + ConsensusState + TypeUrl; // hack

    type StoredClientState<Tr: Chain>: Member
        + ClientState<ChainId = ChainIdOf<Tr>, Height = Tr::Height>;
    type StoredConsensusState<Tr: Chain>: Member;

    type Header: Member + Header;

    // this is just Height
    type Height: Member + IsHeight + MaybeArbitrary;

    type ClientId: Member + Id + MaybeArbitrary;

    type IbcStateEncoding: Encoding;

    type StateProof: Member;

    /// Available client types for this chain.
    type ClientType: Member + Id;

    type Error: Debug;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId;

    // fn encode_stored_client_state(cs: &Self::StoredClientState)

    fn query_latest_height(&self) -> impl Future<Output = Result<Self::Height, Self::Error>> + '_;

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
        tx_hash: H256,
        destination_channel_id: ChannelId,
        destination_port_id: PortId,
        sequence: NonZeroU64,
    ) -> impl Future<Output = Vec<u8>> + '_;

    // fn fetch_ibc_state<P: IbcPath<Self, Tr>, Tr: Chain>(
    //     &self,
    //     path: P,
    //     height: Self::Height,
    // ) -> impl Future<Output = P::Output>;
}

pub trait ClientState {
    type ChainId: Member + Display + Eq + Hash;
    type Height: Member + IsHeight + MaybeArbitrary;

    fn height(&self) -> Self::Height;
    fn chain_id(&self) -> Self::ChainId;
}

impl ClientState for ethereum::client_state::ClientState {
    type ChainId = U256;
    type Height = Height;

    fn height(&self) -> Self::Height {
        Height {
            // TODO: Make ETHEREUM_REVISION_NUMBER a constant in this crate
            revision_number: 0,
            revision_height: self.latest_slot,
        }
    }

    fn chain_id(&self) -> Self::ChainId {
        self.chain_id
    }
}

impl ClientState for scroll::client_state::ClientState {
    type ChainId = U256;
    type Height = Height;

    fn height(&self) -> Self::Height {
        Height {
            // TODO: Make ETHEREUM_REVISION_NUMBER a constant in this crate
            revision_number: 0,
            revision_height: self.latest_batch_index,
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

impl ClientState for tendermint::client_state::ClientState {
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
    fn trusted_height(&self) -> Height;
}

impl<C: ChainSpec> Header for ethereum::header::Header<C> {
    fn trusted_height(&self) -> Height {
        self.trusted_sync_committee.trusted_height
    }
}

impl Header for scroll::header::Header {
    fn trusted_height(&self) -> Height {
        self.l1_height
    }
}

impl<Data: Header> Header for wasm::client_message::ClientMessage<Data> {
    fn trusted_height(&self) -> Height {
        self.data.trusted_height()
    }
}

impl Header for cometbls::header::Header {
    fn trusted_height(&self) -> Height {
        self.trusted_height
    }
}

impl Header for tendermint::header::Header {
    fn trusted_height(&self) -> Height {
        self.trusted_height
    }
}

pub trait ConsensusState {
    fn timestamp(&self) -> u64;
}

impl ConsensusState for ethereum::consensus_state::ConsensusState {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl ConsensusState for scroll::consensus_state::ConsensusState {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl<Data: ConsensusState> ConsensusState for wasm::consensus_state::ConsensusState<Data> {
    fn timestamp(&self) -> u64 {
        self.data.timestamp()
    }
}

impl ConsensusState for cometbls::consensus_state::ConsensusState {
    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl ConsensusState for tendermint::consensus_state::ConsensusState {
    fn timestamp(&self) -> u64 {
        // REVIEW: Perhaps this fn should return Timestamp
        self.timestamp.seconds.inner().try_into().unwrap()
    }
}

pub type ClientStateOf<C> = <C as Chain>::SelfClientState;
pub type ConsensusStateOf<C> = <C as Chain>::SelfConsensusState;
pub type HeaderOf<C> = <C as Chain>::Header;
pub type HeightOf<C> = <C as Chain>::Height;
pub type IbcStateEncodingOf<C> = <C as Chain>::IbcStateEncoding;
pub type ChainIdOf<C> = <<C as Chain>::SelfClientState as ClientState>::ChainId;
pub type ClientIdOf<C> = <C as Chain>::ClientId;
pub type ClientTypeOf<C> = <C as Chain>::ClientType;
