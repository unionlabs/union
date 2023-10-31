use std::{fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    id::{ChannelId, ConnectionId, PortId},
    proof,
    traits::Chain,
};

use crate::{
    chain::{ChainOf, HeightOf, LightClient, LightClientBase, QueryHeight},
    msg::{any_enum, ChainIdOf},
};

any_enum! {
    /// Fetch some data that will likely be used in a [`RelayerMsg::Aggregate`].
    #[any = AnyFetch]
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

impl<L: LightClient> Display for Fetch<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fetch::TrustedClientState(_) => write!(f, "TrustedClientState"),
            Fetch::StateProof(sp) => write!(f, "{sp}"),
            Fetch::SelfClientState(_) => write!(f, "SelfClientState"),
            Fetch::SelfConsensusState(_) => write!(f, "SelfConsensusState"),
            Fetch::ChannelEnd(_) => write!(f, "ChannelEnd"),
            Fetch::ConnectionEnd(_) => write!(f, "ConnectionEnd"),
            Fetch::PacketAcknowledgement(_) => write!(f, "PacketAcknowledgement"),
            Fetch::UpdateHeaders(_) => write!(f, "UpdateHeaders"),
            Fetch::LightClientSpecific(fetch) => write!(f, "LightClientSpecific({})", fetch.0),
        }
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
    pub client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchStateProof<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
    pub path: proof::Path<<ChainOf<L> as Chain>::ClientId, HeightOf<ChainOf<L::Counterparty>>>,
}

impl<L: LightClient> Display for FetchStateProof<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateProof::")?;

        match self.path {
            proof::Path::ClientStatePath(_) => write!(f, "ClientStatePath"),
            proof::Path::ClientConsensusStatePath(_) => write!(f, "ClientConsensusStatePath"),
            proof::Path::ConnectionPath(_) => write!(f, "ConnectionPath"),
            proof::Path::ChannelEndPath(_) => write!(f, "ChannelEndPath"),
            proof::Path::CommitmentPath(_) => write!(f, "CommitmentPath"),
            proof::Path::AcknowledgementPath(_) => write!(f, "AcknowledgementPath"),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct FetchChannelEnd<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
    pub port_id: PortId,
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
    pub destination_port_id: PortId,
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
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub update_from: HeightOf<L::HostChain>,
    pub update_to: HeightOf<L::HostChain>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificFetch<L: LightClient>(pub L::Fetch);
