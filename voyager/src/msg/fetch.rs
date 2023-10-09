use std::marker::PhantomData;

use chain_utils::Chain;
use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    id::{ChannelId, ConnectionId},
};

use crate::{
    chain::{proof, ChainOf, HeightOf, LightClient, QueryHeight},
    msg::{any_enum, identified, ChainIdOf},
};

any_enum! {
    /// Fetch some data that will likely be used in a [`RelayerMsg::Aggregate`].
    #[any = AnyFetch(identified!(Fetch<L>))]
    pub enum Fetch<L: LightClient> {
        TrustedClientState(FetchTrustedClientState<L>),

        StateProof(FetchStateProof<L>),
        SelfClientState(FetchSelfClientState<L>),
        SelfConsensusState(FetchSelfConsensusState<L>),

        ChannelEnd(FetchChannelEnd<L>),
        ConnectionEnd(FetchConnectionEnd<L>),

        PacketAcknowledgement(FetchPacketAcknowledgement<L>),

        UpdateHeaders(FetchUpdateHeaders<L>),
        LightClientSpecific(LightClientSpecificFetch<L>),
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchSelfClientState<L: LightClient> {
    pub at: QueryHeight<HeightOf<ChainOf<L>>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchSelfConsensusState<L: LightClient> {
    pub at: QueryHeight<HeightOf<ChainOf<L>>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchTrustedClientState<L: LightClient> {
    pub at: QueryHeight<HeightOf<ChainOf<L>>>,
    pub client_id: L::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchCounterpartyTrustedClientState<L: LightClient> {
    pub at: QueryHeight<HeightOf<ChainOf<L::Counterparty>>>,
    pub client_id: <L::Counterparty as LightClient>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchStateProof<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
    pub path: proof::Path<<ChainOf<L> as Chain>::ClientId, HeightOf<ChainOf<L::Counterparty>>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchChannelEnd<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
    pub port_id: String,
    pub channel_id: ChannelId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchConnectionEnd<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
    pub connection_id: ConnectionId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchPacketAcknowledgement<L: LightClient> {
    pub block_hash: H256,
    pub destination_port_id: String,
    pub destination_channel_id: ChannelId,
    pub sequence: u64,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchUpdateHeaders<L: LightClient> {
    pub client_id: L::ClientId,
    pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    // id of the counterparty client that will be updated with the fetched headers
    pub counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    pub update_from: HeightOf<L::HostChain>,
    pub update_to: HeightOf<L::HostChain>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificFetch<L: LightClient>(pub L::Fetch);
