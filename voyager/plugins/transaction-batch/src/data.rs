use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use macros::model;
use subset_of::SubsetOf;
use voyager_sdk::message::data::EventProvableHeight;

use crate::IbcSpecExt;

#[model]
#[derive(Enumorph, SubsetOf)]
pub enum ModuleData {
    #[serde(alias = "batch_events_classicc")] // lol whoops
    BatchEventsClassic(EventBatch<IbcClassic>),
    BatchEventsUnion(EventBatch<IbcUnion>),
    ProofUnavailableClassic(ProofUnavailable<IbcClassic>),
    ProofUnavailableUnion(ProofUnavailable<IbcUnion>),
}

#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct EventBatch<V: IbcSpecExt> {
    /// The client that will need an update to send these messages through.
    ///
    /// This is the counterparty client of the source event.
    pub client_id: V::ClientId,
    /// The on-chain events that will need to be turned into messages to send to this chain.
    pub events: Vec<BatchableEvent<V>>,
}

// TODO: Add a "now" constructor
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BatchableEvent<V: IbcSpecExt> {
    /// unix timestamp (in ms) of when this event was first seen by this plugin.
    pub first_seen_at: u64,
    // the 'provable height' of the event
    pub provable_height: EventProvableHeight,
    pub event: V::BatchableEvent,
}

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

#[model]
#[derive(Enumorph)]
pub enum EventUnion {
    ConnectionOpenInit(ibc_union_spec::event::ConnectionOpenInit),
    ConnectionOpenTry(ibc_union_spec::event::ConnectionOpenTry),
    ConnectionOpenAck(ibc_union_spec::event::ConnectionOpenAck),

    ChannelOpenInit(ibc_union_spec::event::ChannelOpenInit),
    ChannelOpenTry(ibc_union_spec::event::ChannelOpenTry),
    ChannelOpenAck(ibc_union_spec::event::ChannelOpenAck),

    PacketSend(ibc_union_spec::event::PacketSend),
    BatchSend(ibc_union_spec::event::BatchSend),
    WriteAck(ibc_union_spec::event::WriteAck),
}

impl TryFrom<ibc_union_spec::event::FullEvent> for EventUnion {
    type Error = ();

    fn try_from(value: ibc_union_spec::event::FullEvent) -> Result<Self, Self::Error> {
        match value {
            ibc_union_spec::event::FullEvent::ConnectionOpenInit(e) => {
                Ok(Self::ConnectionOpenInit(e))
            }
            ibc_union_spec::event::FullEvent::ConnectionOpenTry(e) => {
                Ok(Self::ConnectionOpenTry(e))
            }
            ibc_union_spec::event::FullEvent::ConnectionOpenAck(e) => {
                Ok(Self::ConnectionOpenAck(e))
            }
            ibc_union_spec::event::FullEvent::ChannelOpenInit(e) => Ok(Self::ChannelOpenInit(e)),
            ibc_union_spec::event::FullEvent::ChannelOpenTry(e) => Ok(Self::ChannelOpenTry(e)),
            ibc_union_spec::event::FullEvent::ChannelOpenAck(e) => Ok(Self::ChannelOpenAck(e)),
            ibc_union_spec::event::FullEvent::PacketSend(e) => Ok(Self::PacketSend(e)),
            ibc_union_spec::event::FullEvent::BatchSend(e) => Ok(Self::BatchSend(e)),
            ibc_union_spec::event::FullEvent::WriteAck(e) => Ok(Self::WriteAck(e)),
            _ => Err(()),
        }
    }
}

/// A proof was not available for the contained event at it's provable height.
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct ProofUnavailable<V: IbcSpecExt> {
    /// The client that will need an update to send these messages through.
    ///
    /// This is the counterparty client of the source event.
    pub client_id: V::ClientId,
    /// The on-chain event that will need to be turned into a message to send to this chain.
    pub event: BatchableEvent<V>,
}
