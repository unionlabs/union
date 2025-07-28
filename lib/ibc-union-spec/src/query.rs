use enumorph::Enumorph;
use unionlabs::primitives::H256;
use voyager_primitives::IbcQuery;

use crate::{
    types::{ChannelId, ClientId},
    IbcUnion, Packet, Status,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Enumorph)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Query {
    /// Query the full details of a packet. This is likely not stored on-chain directly, but should be queryable from an event.
    PacketByHash(PacketByHash),
    /// Query the full details of all of the packets in a batch. This is likely not stored on-chain directly, but should be queryable from events.
    PacketsByBatchHash(PacketsByBatchHash),
    /// Query the status of a client.
    ClientStatus(ClientStatus),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketByHash {
    pub channel_id: ChannelId,
    pub packet_hash: H256,
}

impl IbcQuery for PacketByHash {
    type Spec = IbcUnion;

    type Value = Packet;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct PacketsByBatchHash {
    pub channel_id: ChannelId,
    pub batch_hash: H256,
}

impl IbcQuery for PacketsByBatchHash {
    type Spec = IbcUnion;

    type Value = Vec<Packet>;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct ClientStatus {
    pub client_id: ClientId,
    pub height: Option<u64>,
}

impl IbcQuery for ClientStatus {
    type Spec = IbcUnion;

    type Value = Status;
}
