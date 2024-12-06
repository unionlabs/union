use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use macros::model;
use subset_of::SubsetOf;
use unionlabs::ibc::core::client::height::Height;

use crate::IbcSpecExt;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    BatchEventsV1(EventBatch<IbcClassic>),
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
/// A subset of [`FullEvent`], containing only events that cause an action on the counterparty chain.
#[model]
#[derive(Enumorph)]
pub enum EventClassic {
    ConnectionOpenInit(ibc_classic_spec::ConnectionOpenInit),
    ConnectionOpenTry(ibc_classic_spec::ConnectionOpenTry),
    ConnectionOpenAck(ibc_classic_spec::ConnectionOpenAck),

    ChannelOpenInit(ibc_classic_spec::ChannelOpenInit),
    ChannelOpenTry(ibc_classic_spec::ChannelOpenTry),
    ChannelOpenAck(ibc_classic_spec::ChannelOpenAck),

    SendPacket(ibc_classic_spec::SendPacket),
    WriteAcknowledgement(ibc_classic_spec::WriteAcknowledgement),
}

impl TryFrom<ibc_classic_spec::FullEvent> for EventClassic {
    type Error = ();

    fn try_from(value: ibc_classic_spec::FullEvent) -> Result<Self, Self::Error> {
        match value {
            ibc_classic_spec::FullEvent::ConnectionOpenInit(e) => Ok(Self::ConnectionOpenInit(e)),
            ibc_classic_spec::FullEvent::ConnectionOpenTry(e) => Ok(Self::ConnectionOpenTry(e)),
            ibc_classic_spec::FullEvent::ConnectionOpenAck(e) => Ok(Self::ConnectionOpenAck(e)),
            ibc_classic_spec::FullEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            ibc_classic_spec::FullEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            ibc_classic_spec::FullEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            ibc_classic_spec::FullEvent::SendPacket(e) => Ok(Self::SendPacket(e)),
            ibc_classic_spec::FullEvent::WriteAcknowledgement(e) => {
                Ok(Self::WriteAcknowledgement(e))
            }
            _ => Err(()),
        }
    }
}

// REVIEW: We probably want to add a way to have "a packet timed out" be a variant here as well
/// A subset of [`FullEvent`], containing only events that cause an action on the counterparty chain.
#[model]
#[derive(Enumorph)]
pub enum EventUnion {
    ConnectionOpenInit(ibc_union_spec::ConnectionOpenInit),
    ConnectionOpenTry(ibc_union_spec::ConnectionOpenTry),
    ConnectionOpenAck(ibc_union_spec::ConnectionOpenAck),

    ChannelOpenInit(ibc_union_spec::ChannelOpenInit),
    ChannelOpenTry(ibc_union_spec::ChannelOpenTry),
    ChannelOpenAck(ibc_union_spec::ChannelOpenAck),

    SendPacket(ibc_union_spec::SendPacket),
    WriteAcknowledgement(ibc_union_spec::WriteAcknowledgement),
}

impl TryFrom<ibc_union_spec::FullEvent> for EventUnion {
    type Error = ();

    fn try_from(value: ibc_union_spec::FullEvent) -> Result<Self, Self::Error> {
        match value {
            ibc_union_spec::FullEvent::ConnectionOpenInit(e) => Ok(Self::ConnectionOpenInit(e)),
            ibc_union_spec::FullEvent::ConnectionOpenTry(e) => Ok(Self::ConnectionOpenTry(e)),
            ibc_union_spec::FullEvent::ConnectionOpenAck(e) => Ok(Self::ConnectionOpenAck(e)),
            ibc_union_spec::FullEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            ibc_union_spec::FullEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            ibc_union_spec::FullEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            ibc_union_spec::FullEvent::SendPacket(e) => Ok(Self::SendPacket(e)),
            ibc_union_spec::FullEvent::WriteAcknowledgement(e) => Ok(Self::WriteAcknowledgement(e)),
            _ => Err(()),
        }
    }
}
