use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    id::{ChannelId, ConnectionId, PortId},
    proof,
    traits::{Chain, ChainIdOf, ChainOf, HeightOf, LightClientBase},
    QueryHeight,
};

use crate::{
    any_enum, data,
    data::{
        AnyData, ChannelEnd, ConnectionEnd, Data, PacketAcknowledgement, SelfClientState,
        SelfConsensusState, TrustedClientState,
    },
    identified, AnyLightClientIdentified, LightClient, RelayerMsg,
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

impl<L: LightClient> Fetch<L> {
    pub async fn handle(self, l: L) -> Vec<RelayerMsg>
    where
        AnyLightClientIdentified<AnyData>: From<identified!(Data<L>)>,
        // TODO: Remove once we no longer unwrap
        <<L as LightClientBase>::ClientId as TryFrom<
            <<L as LightClientBase>::HostChain as Chain>::ClientId,
        >>::Error: Debug,
        <<L::Counterparty as LightClientBase>::ClientId as TryFrom<
            <<L::Counterparty as LightClientBase>::HostChain as Chain>::ClientId,
        >>::Error: Debug,
    {
        let relayer_msg = match self {
            Fetch::TrustedClientState(FetchTrustedClientState { at, client_id }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match at {
                    QueryHeight::Latest => l.chain().query_latest_height().await,
                    QueryHeight::Specific(h) => h,
                };

                [data(
                    l.chain().chain_id(),
                    TrustedClientState {
                        fetched_at: height,
                        client_id: client_id.clone(),
                        trusted_client_state: l.query_client_state(client_id.into(), height).await,
                    },
                )]
                .into()
            }
            Fetch::StateProof(msg) => [l.proof(msg)].into(),
            Fetch::SelfClientState(FetchSelfClientState { at: height }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => l.chain().query_latest_height().await,
                    QueryHeight::Specific(h) => h,
                };

                [data(
                    l.chain().chain_id(),
                    SelfClientState(l.chain().self_client_state(height).await),
                )]
                .into()
            }
            Fetch::SelfConsensusState(FetchSelfConsensusState { at: height }) => {
                // TODO: Split this into a separate query and aggregate
                let height = match height {
                    QueryHeight::Latest => l.chain().query_latest_height().await,
                    QueryHeight::Specific(h) => h,
                };

                [data(
                    l.chain().chain_id(),
                    SelfConsensusState(l.chain().self_consensus_state(height).await),
                )]
                .into()
            }
            Fetch::PacketAcknowledgement(FetchPacketAcknowledgement {
                block_hash,
                destination_port_id,
                destination_channel_id,
                sequence,
                __marker,
            }) => {
                let ack = l
                    .chain()
                    .read_ack(
                        block_hash.clone(),
                        destination_channel_id.clone(),
                        destination_port_id.clone(),
                        sequence,
                    )
                    .await;

                [data(
                    l.chain().chain_id(),
                    PacketAcknowledgement {
                        fetched_by: FetchPacketAcknowledgement {
                            block_hash,
                            destination_port_id,
                            destination_channel_id,
                            sequence,
                            __marker,
                        },
                        ack,
                    },
                )]
                .into()
            }
            Fetch::UpdateHeaders(fetch_update_headers) => {
                l.generate_counterparty_updates(fetch_update_headers)
            }
            Fetch::LightClientSpecific(LightClientSpecificFetch(fetch)) => l.do_fetch(fetch).await,
            Fetch::ChannelEnd(FetchChannelEnd {
                at,
                port_id,
                channel_id,
            }) => [data(
                l.chain().chain_id(),
                ChannelEnd {
                    channel: l.channel(channel_id, port_id, at).await,
                    __marker: PhantomData,
                },
            )]
            .into(),
            Fetch::ConnectionEnd(FetchConnectionEnd { at, connection_id }) => [data(
                l.chain().chain_id(),
                ConnectionEnd(l.connection(connection_id, at).await),
            )]
            .into(),
        };

        relayer_msg
    }
}
