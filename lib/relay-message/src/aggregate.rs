use std::{collections::VecDeque, marker::PhantomData, num::NonZeroU64};

use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregate,
    aggregation::{do_aggregate, UseAggregate},
    defer_relative, effect, fetch, noop, queue_msg, race, seq, wait, HandleAggregate, Op,
    QueueError, QueueMessage,
};
use tracing::{debug, info, instrument};
use unionlabs::{
    events::{
        ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
        ConnectionOpenTry, SendPacket, WriteAcknowledgement,
    },
    hash::H256,
    ibc::core::{
        channel::{
            self, channel::Channel, msg_acknowledgement::MsgAcknowledgement,
            msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_try::MsgChannelOpenTry, msg_recv_packet::MsgRecvPacket,
            msg_timeout::MsgTimeout, packet::Packet,
        },
        client::{
            height::{Height, IsHeight},
            msg_create_client::MsgCreateClient,
        },
        commitment::merkle_prefix::MerklePrefix,
        connection::{
            self, msg_connection_open_ack::MsgConnectionOpenAck,
            msg_connection_open_confirm::MsgConnectionOpenConfirm,
            msg_connection_open_try::MsgConnectionOpenTry,
        },
    },
    ics24::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, NextClientSequencePath, NextConnectionSequencePath,
        NextSequenceRecvPath, ReceiptPath,
    },
    id::{ChannelId, ConnectionId, PortId},
    traits::{ChainIdOf, ClientIdOf, ClientState, HeightOf},
    QueryHeight, DELAY_PERIOD,
};

use crate::{
    any_enum, any_lc,
    data::{AnyData, Data, IbcProof, IbcState, LatestHeight, SelfClientState, SelfConsensusState},
    effect::{
        AnyEffect, Effect, MsgAckPacketData, MsgChannelOpenAckData, MsgChannelOpenConfirmData,
        MsgChannelOpenTryData, MsgConnectionOpenAckData, MsgConnectionOpenConfirmData,
        MsgConnectionOpenTryData, MsgCreateClientData, MsgRecvPacketData, MsgTimeoutData,
    },
    fetch::{AnyFetch, Fetch, FetchLatestHeight, FetchProof, FetchState, FetchUpdateHeaders},
    id, identified,
    use_aggregate::IsAggregateData,
    wait::{AnyWait, Wait, WaitForHeight, WaitForTimestamp, WaitForTrustedHeight},
    AnyLightClientIdentified, ChainExt, DoAggregate, Identified, RelayMessage,
};

// #[apply(any_enum)]
// /// Aggregate data, using data from [`AggregateData`]
// #[any = AnyAggregate]
// #[specific = LightClientSpecificAggregate]
pub enum Aggregate {
    // put together the final pieces (proofs, states, etc) to build the msgs that will be sent
    MsgConnectionOpenTry(AggregateMsgConnectionOpenTry),
    MsgConnectionOpenAck(AggregateMsgConnectionOpenAck),
    MsgConnectionOpenConfirm(AggregateMsgConnectionOpenConfirm),

    MsgChannelOpenTry(AggregateMsgChannelOpenTry),
    MsgChannelOpenAck(AggregateMsgChannelOpenAck),
    MsgChannelOpenConfirm(AggregateMsgChannelOpenConfirm),

    MsgRecvPacket(AggregateMsgRecvPacket),
    MsgAckPacket(AggregateMsgAckPacket),
    MsgTimeout(AggregateMsgTimeout),

    // construct one of the above messages after a required client update
    AggregateMsgAfterUpdate(AggregateMsgAfterUpdate),

    MsgCreateClient(AggregateMsgCreateClient),

    PacketTimeout(AggregatePacketTimeout),

    // composite fetches
    ReceiptPathProofFromChannelAndPort(AggregateFetchReceiptPathProofFromChannelAndPort),
    ClientStateFromConnectionId(AggregateClientStateFromConnection),
    ConnectionFetchFromChannelEnd(AggregateConnectionFetchFromChannelEnd),

    /// Aggregate that fetches the connection info from the channel, requeueing [`Self::AggregateMsgAfterUpdate`]
    ChannelHandshakeMsgAfterUpdate(AggregateChannelHandshakeMsgAfterUpdate),

    PacketUpdateClient(AggregatePacketMsgAfterUpdate),

    WaitForConnectionOpen(AggregateWaitForConnectionOpen),
    WaitForCounterpartyTrustedHeight(AggregateWaitForCounterpartyTrustedHeight),
    WaitForTrustedHeight(AggregateWaitForTrustedHeight),
    WaitForNextConnectionSequence(AggregateWaitForNextConnectionSequence),
    WaitForNextClientSequence(AggregateWaitForNextClientSequence),
    WaitForPacketReceipt(AggregateWaitForPacketReceipt),

    FetchCounterpartyStateproof(AggregateFetchCounterpartyStateProof),

    UpdateClient(AggregateUpdateClient),
    UpdateClientFromHeight(AggregateUpdateClientFromHeight),

    #[serde(untagged)]
    LightClientSpecific(LightClientSpecificAggregate),
}

impl HandleAggregate<RelayMessage> for AnyLightClientIdentified<AnyAggregate> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    fn handle(
        self,
        data: VecDeque<<RelayMessage as QueueMessage>::Data>,
    ) -> Result<Op<RelayMessage>, QueueError> {
        let aggregate = self;

        any_lc! {
            |aggregate| Ok(aggregate.handle(data))
        }
    }
}

impl identified!(Aggregate) {
    pub fn handle(self, data: VecDeque<AnyLightClientIdentified<AnyData>>) -> Op<RelayMessage>
    where
        identified!(SelfClientState<Tr, Hc>): IsAggregateData,
        identified!(SelfConsensusState<Tr, Hc>): IsAggregateData,

        identified!(LatestHeight<Tr, Hc>): IsAggregateData,

        identified!(LatestHeight): IsAggregateData,

        // state
        Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
        Identified<Tr, Hc, IbcState<ClientStatePath<Tr::ClientId>, Tr, Hc>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<NextSequenceRecvPath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<NextConnectionSequencePath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<NextClientSequencePath, Hc, Tr>>: IsAggregateData,

        Identified<Hc, Tr, IbcState<ReceiptPath, Hc, Tr>>: IsAggregateData,
        Identified<Tr, Hc, IbcState<ReceiptPath, Tr, Hc>>: IsAggregateData,

        // proof
        Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
            IsAggregateData,
        Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<CommitmentPath, Hc, Tr>>: IsAggregateData,
        Identified<Hc, Tr, IbcProof<AcknowledgementPath, Hc, Tr>>: IsAggregateData,
        Identified<Tr, Hc, IbcProof<ReceiptPath, Tr, Hc>>: IsAggregateData,

        Identified<Hc, Tr, Hc::Aggregate<Tr>>: DoAggregate,

        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch)>,
        AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,

        AnyLightClientIdentified<AnyWait>: From<identified!(Wait)>,
        AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Tr, Hc>)>,

        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect)>,
        AnyLightClientIdentified<AnyEffect>: From<identified!(Effect<Tr, Hc>)>,

        AnyLightClientIdentified<AnyData>: From<identified!(Data)>,
        AnyLightClientIdentified<AnyData>: From<identified!(Data<Tr, Hc>)>,

        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate)>,
        AnyLightClientIdentified<AnyAggregate>: From<identified!(Aggregate<Tr, Hc>)>,
    {
        let chain_id = self.chain_id;

        match self.t {
            Aggregate::MsgConnectionOpenTry(init) => do_aggregate(id(chain_id, init), data),
            Aggregate::MsgConnectionOpenAck(ack) => do_aggregate(id(chain_id, ack), data),
            Aggregate::MsgConnectionOpenConfirm(confirm) => {
                do_aggregate(id(chain_id, confirm), data)
            }
            Aggregate::MsgChannelOpenTry(try_) => do_aggregate(id(chain_id, try_), data),
            Aggregate::MsgChannelOpenAck(ack) => do_aggregate(id(chain_id, ack), data),
            Aggregate::MsgChannelOpenConfirm(confirm) => do_aggregate(id(chain_id, confirm), data),
            Aggregate::UpdateClient(update_client) => {
                do_aggregate(id(chain_id, update_client), data)
            }
            Aggregate::UpdateClientFromHeight(update_client) => {
                do_aggregate(id(chain_id, update_client), data)
            }
            Aggregate::MsgCreateClient(create_client) => {
                do_aggregate(id(chain_id, create_client), data)
            }
            Aggregate::AggregateMsgAfterUpdate(aggregate) => {
                do_aggregate(id(chain_id, aggregate), data)
            }
            Aggregate::LightClientSpecific(LightClientSpecificAggregate(aggregate)) => {
                <Identified<_, _, Hc::Aggregate<Tr>> as DoAggregate>::do_aggregate(
                    id(chain_id, aggregate),
                    data,
                )
            }
            Aggregate::ConnectionFetchFromChannelEnd(aggregate) => {
                do_aggregate(id(chain_id, aggregate), data)
            }
            Aggregate::ReceiptPathProofFromChannelAndPort(aggregate) => {
                do_aggregate(id(chain_id, aggregate), data)
            }
            Aggregate::ChannelHandshakeMsgAfterUpdate(channel_handshake_update_client) => {
                do_aggregate(id(chain_id, channel_handshake_update_client), data)
            }
            Aggregate::PacketUpdateClient(packet_update_client) => {
                do_aggregate(id(chain_id, packet_update_client), data)
            }
            Aggregate::MsgRecvPacket(recv_packet) => do_aggregate(id(chain_id, recv_packet), data),
            Aggregate::MsgAckPacket(ack_packet) => do_aggregate(id(chain_id, ack_packet), data),
            Aggregate::MsgTimeout(timeout_packet) => {
                do_aggregate(id(chain_id, timeout_packet), data)
            }
            Aggregate::PacketTimeout(timeout_packet) => {
                do_aggregate(id(chain_id, timeout_packet), data)
            }
            Aggregate::WaitForCounterpartyTrustedHeight(agg) => {
                do_aggregate(id(chain_id, agg), data)
            }
            Aggregate::WaitForTrustedHeight(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::FetchCounterpartyStateproof(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::ClientStateFromConnectionId(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::WaitForConnectionOpen(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::WaitForNextConnectionSequence(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::WaitForNextClientSequence(agg) => do_aggregate(id(chain_id, agg), data),
            Aggregate::WaitForPacketReceipt(agg) => do_aggregate(id(chain_id, agg), data),
        }
    }
}
