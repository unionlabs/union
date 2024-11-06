use enumorph::Enumorph;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    ibc_union::{self, IbcUnion},
    ibc_v1::{self, IbcV1},
};

use crate::IbcSpecExt;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    BatchEventsV1(EventBatch<IbcV1>),
    BatchEventsUnion(EventBatch<IbcUnion>),
}

#[model]
pub struct EventBatch<V: IbcSpecExt> {
    /// The client that will need an update to send these messages through.
    pub client_id: V::ClientId,
    /// The on-chain events that will need to be turned into messages to send to this chain.
    pub events: Vec<BatchableEvent<V>>,
}

#[model]
pub struct BatchableEvent<V: IbcSpecExt> {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    // the 'provable height' of the event
    pub provable_height: Height,
    pub event: V::BatchableEvent,
}

// REVIEW: We probably want to add a way to have "a packet timed out" be a variant here as well
/// A subset of [`FullIbcEvent`], containing only events that cause an action on the counterparty chain.
#[model]
#[derive(Enumorph)]
pub enum EventV1 {
    ConnectionOpenInit(ibc_v1::ConnectionOpenInit),
    ConnectionOpenTry(ibc_v1::ConnectionOpenTry),
    ConnectionOpenAck(ibc_v1::ConnectionOpenAck),

    ChannelOpenInit(ibc_v1::ChannelOpenInit),
    ChannelOpenTry(ibc_v1::ChannelOpenTry),
    ChannelOpenAck(ibc_v1::ChannelOpenAck),

    SendPacket(ibc_v1::SendPacket),
    WriteAcknowledgement(ibc_v1::WriteAcknowledgement),
}

impl TryFrom<ibc_v1::FullIbcEvent> for EventV1 {
    type Error = ();

    fn try_from(value: ibc_v1::FullIbcEvent) -> Result<Self, Self::Error> {
        match value {
            ibc_v1::FullIbcEvent::ConnectionOpenInit(e) => Ok(Self::ConnectionOpenInit(e)),
            ibc_v1::FullIbcEvent::ConnectionOpenTry(e) => Ok(Self::ConnectionOpenTry(e)),
            ibc_v1::FullIbcEvent::ConnectionOpenAck(e) => Ok(Self::ConnectionOpenAck(e)),
            ibc_v1::FullIbcEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            ibc_v1::FullIbcEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            ibc_v1::FullIbcEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            ibc_v1::FullIbcEvent::SendPacket(e) => Ok(Self::SendPacket(e)),
            ibc_v1::FullIbcEvent::WriteAcknowledgement(e) => Ok(Self::WriteAcknowledgement(e)),
            _ => Err(()),
        }
    }
}

// REVIEW: We probably want to add a way to have "a packet timed out" be a variant here as well
/// A subset of [`FullIbcEvent`], containing only events that cause an action on the counterparty chain.
#[model]
#[derive(Enumorph)]
pub enum EventUnion {
    ConnectionOpenInit(ibc_union::ConnectionOpenInit),
    ConnectionOpenTry(ibc_union::ConnectionOpenTry),
    ConnectionOpenAck(ibc_union::ConnectionOpenAck),

    ChannelOpenInit(ibc_union::ChannelOpenInit),
    ChannelOpenTry(ibc_union::ChannelOpenTry),
    ChannelOpenAck(ibc_union::ChannelOpenAck),

    SendPacket(ibc_union::SendPacket),
    WriteAcknowledgement(ibc_union::WriteAcknowledgement),
}

impl TryFrom<ibc_union::FullIbcEvent> for EventUnion {
    type Error = ();

    fn try_from(value: ibc_union::FullIbcEvent) -> Result<Self, Self::Error> {
        match value {
            ibc_union::FullIbcEvent::ConnectionOpenInit(e) => Ok(Self::ConnectionOpenInit(e)),
            ibc_union::FullIbcEvent::ConnectionOpenTry(e) => Ok(Self::ConnectionOpenTry(e)),
            ibc_union::FullIbcEvent::ConnectionOpenAck(e) => Ok(Self::ConnectionOpenAck(e)),
            ibc_union::FullIbcEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            ibc_union::FullIbcEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            ibc_union::FullIbcEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            ibc_union::FullIbcEvent::SendPacket(e) => Ok(Self::SendPacket(e)),
            ibc_union::FullIbcEvent::WriteAcknowledgement(e) => Ok(Self::WriteAcknowledgement(e)),
            _ => Err(()),
        }
    }
}
