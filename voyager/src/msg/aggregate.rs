use std::fmt::Display;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    events::{
        ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry, RecvPacket, SendPacket,
    },
};

use super::ChainIdOf;
use crate::{
    chain::{ChainOf, HeightOf, LightClient, LightClientBase},
    msg::{any_enum, fetch::FetchStateProof},
};

any_enum! {
    /// Aggregate data, using data from [`AggregateData`]
    #[any = AnyAggregate]
    pub enum Aggregate<L: LightClient> {
        ConnectionOpenTry(AggregateConnectionOpenTry<L>),
        ConnectionOpenAck(AggregateConnectionOpenAck<L>),
        ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

        ChannelOpenTry(AggregateChannelOpenTry<L>),
        ChannelOpenAck(AggregateChannelOpenAck<L>),
        ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

        RecvPacket(AggregateRecvPacket<L>),
        AckPacket(AggregateAckPacket<L>),

        ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd<L>),

        // Aggregate that fetches the connection info from the channel
        ChannelHandshakeUpdateClient(AggregateChannelHandshakeUpdateClient<L>),

        PacketUpdateClient(AggregatePacketUpdateClient<L>),

        WaitForTrustedHeight(AggregateWaitForTrustedHeight<L>),

        FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof<L>),

        UpdateClientFromClientId(AggregateUpdateClientFromClientId<L>),

        UpdateClient(AggregateUpdateClient<L>),
        UpdateClientWithCounterpartyChainIdData(AggregateUpdateClientWithCounterpartyChainId<L>),

        CreateClient(AggregateCreateClient<L>),

        AggregateMsgAfterUpdate(AggregateMsgAfterUpdate<L>),

        LightClientSpecific(LightClientSpecificAggregate<L>),
    }
}

impl<L: LightClient> Display for Aggregate<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregate::ConnectionOpenTry(_) => write!(f, "ConnectionOpenTry"),
            Aggregate::ConnectionOpenAck(_) => write!(f, "ConnectionOpenAck"),
            Aggregate::ConnectionOpenConfirm(_) => write!(f, "ConnectionOpenConfirm"),
            Aggregate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
            Aggregate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
            Aggregate::ChannelOpenConfirm(_) => write!(f, "ChannelOpenConfirm"),
            Aggregate::RecvPacket(_) => write!(f, "RecvPacket"),
            Aggregate::AckPacket(_) => write!(f, "AckPacket"),
            Aggregate::ConnectionFetchFromChannelEnd(_) => {
                write!(f, "ConnectionFetchFromChannelEnd")
            }
            Aggregate::ChannelHandshakeUpdateClient(_) => {
                write!(f, "ChannelHandshakeUpdateClient")
            }
            Aggregate::PacketUpdateClient(msg) => {
                write!(
                    f,
                    "PacketUpdateClient::{}",
                    match msg.packet_event {
                        PacketEvent::Send(_) => "Send",
                        PacketEvent::Recv(_) => "Recv",
                    }
                )
            }
            Aggregate::WaitForTrustedHeight(_) => write!(f, "WaitForTrustedHeight"),
            Aggregate::FetchCounterpartyStateproof(_) => {
                write!(f, "FetchCounterpartyStateproof")
            }
            Aggregate::UpdateClientFromClientId(_) => write!(f, "UpdateClientFromClientId"),
            Aggregate::UpdateClient(_) => write!(f, "UpdateClient"),
            Aggregate::UpdateClientWithCounterpartyChainIdData(_) => {
                write!(f, "UpdateClientWithCounterpartyChainIdData")
            }
            Aggregate::CreateClient(_) => write!(f, "CreateClient"),
            Aggregate::AggregateMsgAfterUpdate(msg) => {
                write!(f, "AggregateMsgAfterUpdate::")?;
                match msg {
                    AggregateMsgAfterUpdate::ConnectionOpenTry(_) => {
                        write!(f, "ConnectionOpenTry")
                    }
                    AggregateMsgAfterUpdate::ConnectionOpenAck(_) => {
                        write!(f, "ConnectionOpenAck")
                    }
                    AggregateMsgAfterUpdate::ConnectionOpenConfirm(_) => {
                        write!(f, "ConnectionOpenConfirm")
                    }
                    AggregateMsgAfterUpdate::ChannelOpenTry(_) => write!(f, "ChannelOpenTry"),
                    AggregateMsgAfterUpdate::ChannelOpenAck(_) => write!(f, "ChannelOpenAck"),
                    AggregateMsgAfterUpdate::ChannelOpenConfirm(_) => {
                        write!(f, "ChannelOpenConfirm")
                    }
                    AggregateMsgAfterUpdate::RecvPacket(_) => write!(f, "RecvPacket"),
                    AggregateMsgAfterUpdate::AckPacket(_) => write!(f, "AckPacket"),
                }
            }
            Aggregate::LightClientSpecific(agg) => write!(f, "LightClientSpecific({})", agg.0),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenTry<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenInit<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenAck<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenTry<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionOpenConfirm<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ConnectionOpenAck<L::ClientId, <L::Counterparty as LightClientBase>::ClientId>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenTry<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenInit,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenAck<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenTry,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelOpenConfirm<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: ChannelOpenAck,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateRecvPacket<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: SendPacket,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateAckPacket<L: LightClient> {
    pub event_height: HeightOf<L::HostChain>,
    pub event: RecvPacket,
    // HACK: Need to pass the block hash through, figure out a better/cleaner way to do this
    pub block_hash: H256,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateConnectionFetchFromChannelEnd<L: LightClient> {
    pub at: HeightOf<ChainOf<L>>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateChannelHandshakeUpdateClient<L: LightClient> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<L::HostChain>,
    pub event_height: HeightOf<L::HostChain>,
    pub channel_handshake_event: ChannelHandshakeEvent,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum ChannelHandshakeEvent {
    Init(ChannelOpenInit),
    Try(ChannelOpenTry),
    Ack(ChannelOpenAck),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregatePacketUpdateClient<L: LightClient> {
    // Will be threaded through to the update msg
    pub update_to: HeightOf<L::HostChain>,
    pub event_height: HeightOf<L::HostChain>,
    pub block_hash: H256,
    pub packet_event: PacketEvent,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum PacketEvent {
    Send(SendPacket),
    Recv(RecvPacket),
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateFetchCounterpartyStateProof<L: LightClient> {
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub fetch: FetchStateProof<L::Counterparty>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientFromClientId<L: LightClient> {
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClient<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateWaitForTrustedHeight<L: LightClient> {
    pub wait_for: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateUpdateClientWithCounterpartyChainId<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateMsgUpdateClient<L: LightClient> {
    pub update_to: HeightOf<L::HostChain>,
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AggregateCreateClient<L: LightClient> {
    pub config: L::Config,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct LightClientSpecificAggregate<L: LightClient>(pub L::Aggregate);

/// Messages that will be re-queued after an update.
#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum AggregateMsgAfterUpdate<L: LightClient> {
    ConnectionOpenTry(AggregateConnectionOpenTry<L>),
    ConnectionOpenAck(AggregateConnectionOpenAck<L>),
    ConnectionOpenConfirm(AggregateConnectionOpenConfirm<L>),

    ChannelOpenTry(AggregateChannelOpenTry<L>),
    ChannelOpenAck(AggregateChannelOpenAck<L>),
    ChannelOpenConfirm(AggregateChannelOpenConfirm<L>),

    RecvPacket(AggregateRecvPacket<L>),
    AckPacket(AggregateAckPacket<L>),
}

// #[derive(
//     DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
// )]
// #[serde(bound(serialize = "", deserialize = ""))]
// pub enum AggregateAnyStateProof<L: LightClient> {
//     #[display(fmt = "{_0}")]
//     ClientState(
//         AggregateStateProof<
//             L,
//             ClientStatePath<<L::HostChain as unionlabs::traits::Chain>::ClientId>,
//         >,
//     ),
//     #[display(fmt = "{_0}")]
//     ClientConsensusState(
//         AggregateStateProof<
//             L,
//             ClientConsensusStatePath<
//                 <ChainOf<L::Counterparty> as unionlabs::traits::Chain>::ClientId,
//                 HeightOf<ChainOf<L>>,
//             >,
//         >,
//     ),
//     #[display(fmt = "{_0}")]
//     Connection(AggregateStateProof<L, ConnectionPath>),
//     #[display(fmt = "{_0}")]
//     ChannelEnd(AggregateStateProof<L, ChannelEndPath>),
//     #[display(fmt = "{_0}")]
//     Commitment(AggregateStateProof<L, CommitmentPath>),
//     #[display(fmt = "{_0}")]
//     Acknowledgement(AggregateStateProof<L, AcknowledgementPath>),
// }

// #[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
// #[serde(bound(serialize = "", deserialize = ""))]
// pub struct AggregateStateProof<L: LightClient, P: IbcPathExt<L>> {
//     height: HeightOf<ChainOf<L>>,
//     #[serde(skip)]
//     pub __marker: PhantomData<P>,
// }
