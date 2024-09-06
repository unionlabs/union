use enumorph::Enumorph;
use queue_msg::{queue_msg, SubsetOf};
use unionlabs::{ibc::core::client::height::Height, id::ClientId};
use voyager_message::data::{
    ChannelOpenAck, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck, ConnectionOpenInit,
    ConnectionOpenTry, SendPacket, WriteAcknowledgement,
};

#[queue_msg]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    BatchEvents(EventBatch),
}

#[queue_msg]
pub struct EventBatch {
    /// The client that will need an update to send these messages through.
    pub client_id: ClientId,
    /// The on-chain events that will need to be turned into messages to send to this chain.
    pub events: Vec<BatchableEvent>,
}

#[queue_msg]
pub struct BatchableEvent {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    // the 'provable height' of the event
    pub provable_height: Height,
    pub event: Event,
}

// REVIEW: We probably want to add a way to have "a packet timed out" be a variant here as well
/// A subset of [`FullIbcEvent`], containing only events that cause an action on the counterparty chain.
#[queue_msg]
#[derive(Enumorph)]
pub enum Event {
    ConnectionOpenInit(ConnectionOpenInit),
    ConnectionOpenTry(ConnectionOpenTry),
    ConnectionOpenAck(ConnectionOpenAck),

    ChannelOpenInit(ChannelOpenInit),
    ChannelOpenTry(ChannelOpenTry),
    ChannelOpenAck(ChannelOpenAck),

    SendPacket(SendPacket),
    WriteAcknowledgement(WriteAcknowledgement),
}
