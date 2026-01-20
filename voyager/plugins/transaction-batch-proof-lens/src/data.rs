use enumorph::Enumorph;
use ibc_union_spec::{
    ClientId,
    datagram::{
        MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenTry, MsgCommitMembershipProof,
        MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenTry,
        MsgPacketAcknowledgement, MsgPacketRecv,
    },
    event::{
        BatchSend, ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck,
        ConnectionOpenInit, ConnectionOpenTry, FullEvent, PacketSend, WriteAck,
    },
    path::{BatchPacketsPath, BatchReceiptsPath, ChannelPath, ConnectionPath, StorePath},
};
use macros::model;
use subset_of::SubsetOf;
use voyager_sdk::message::data::EventProvableHeight;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    BatchEvents(EventBatch),
    ProofUnavailable(ProofUnavailable),
    MsgWithCommitmentMsg(MsgWithCommitmentMsg),
}

/// Represents a single batch of events emitted on the source chain, all related to the `client_id`.
#[model]
pub struct EventBatch {
    /// The client that will need an update to send these messages through.
    ///
    /// This is the counterparty client of the source event.
    pub client_id: ClientId,
    /// The on-chain events that will need to be turned into messages to send to this chain.
    ///
    /// This will first be committed on the L1 chain of the relevant proof lens client in order to
    /// be verified on the destination.
    pub events: Vec<BatchableEvent>,
}

/// Represents a single event in a batch.
#[model]
pub struct BatchableEvent {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    // the 'provable height' of the event
    pub provable_height: EventProvableHeight,
    pub event: CommittableEvent,
}

/// An event of which a proof can be committed on the intermediate chain.
///
/// This can also be thought of as a "provable event". All of the events specified here cause an
/// action on the destination chain.
#[model]
#[derive(Enumorph)]
pub enum CommittableEvent {
    ConnectionOpenInit(ConnectionOpenInit),
    ConnectionOpenTry(ConnectionOpenTry),
    ConnectionOpenAck(ConnectionOpenAck),

    ChannelOpenInit(ChannelOpenInit),
    ChannelOpenTry(ChannelOpenTry),
    ChannelOpenAck(ChannelOpenAck),

    PacketSend(PacketSend),
    BatchSend(BatchSend),
    WriteAck(WriteAck),
}

#[model]
pub struct MsgWithCommitmentMsg {
    pub commitment_msg: MsgCommitMembershipProof,
    /// The unsaturated datagram (the proof is empty)
    pub msg: UnsaturatedMsgWithStoreKey,
}

#[model]
pub enum UnsaturatedMsgWithStoreKey {
    ConnectionOpenTry(MsgConnectionOpenTry, ConnectionPath),
    ConnectionOpenAck(MsgConnectionOpenAck, ConnectionPath),
    ConnectionOpenConfirm(MsgConnectionOpenConfirm, ConnectionPath),
    ChannelOpenTry(MsgChannelOpenTry, ChannelPath),
    ChannelOpenAck(MsgChannelOpenAck, ChannelPath),
    ChannelOpenConfirm(MsgChannelOpenConfirm, ChannelPath),
    PacketRecv(MsgPacketRecv, BatchPacketsPath),
    PacketAcknowledgement(MsgPacketAcknowledgement, BatchReceiptsPath),
}

impl UnsaturatedMsgWithStoreKey {
    pub fn proof_height(&self) -> u64 {
        match self {
            UnsaturatedMsgWithStoreKey::ConnectionOpenTry(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::ConnectionOpenAck(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::ConnectionOpenConfirm(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::ChannelOpenTry(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::ChannelOpenAck(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::ChannelOpenConfirm(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::PacketRecv(msg, _) => msg.proof_height,
            UnsaturatedMsgWithStoreKey::PacketAcknowledgement(msg, _) => msg.proof_height,
        }
    }

    pub fn path(&self) -> StorePath {
        match self {
            UnsaturatedMsgWithStoreKey::ConnectionOpenTry(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::ConnectionOpenAck(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::ConnectionOpenConfirm(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::ChannelOpenTry(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::ChannelOpenAck(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::ChannelOpenConfirm(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::PacketRecv(_, key) => key.clone().into(),
            UnsaturatedMsgWithStoreKey::PacketAcknowledgement(_, key) => key.clone().into(),
        }
    }
}

impl TryFrom<FullEvent> for CommittableEvent {
    type Error = ();

    fn try_from(value: FullEvent) -> Result<Self, Self::Error> {
        match value {
            FullEvent::ConnectionOpenInit(e) => Ok(Self::ConnectionOpenInit(e)),
            FullEvent::ConnectionOpenTry(e) => Ok(Self::ConnectionOpenTry(e)),
            FullEvent::ConnectionOpenAck(e) => Ok(Self::ConnectionOpenAck(e)),
            FullEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            FullEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            FullEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            FullEvent::PacketSend(e) => Ok(Self::PacketSend(e)),
            FullEvent::BatchSend(e) => Ok(Self::BatchSend(e)),
            FullEvent::WriteAck(e) => Ok(Self::WriteAck(e)),
            _ => Err(()),
        }
    }
}

/// A proof was not available for the contained event at it's provable height.
#[model]
pub struct ProofUnavailable {
    /// The client that will need an update to send these messages through.
    ///
    /// This is the counterparty client of the source event.
    pub client_id: ClientId,
    /// The on-chain event that will need to be turned into a message to send to this chain.
    pub event: BatchableEvent,
}
