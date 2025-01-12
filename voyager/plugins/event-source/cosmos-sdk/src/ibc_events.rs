use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::{channel::order::Order, client::height::Height},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    primitives::{encoding::HexUnprefixed, Bytes},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum IbcEvent {
    // standard ibc-go events for IBC classic
    // https://github.com/cosmos/ibc-go/blob/5c7f28634ecf9b6f275bfd5712778fedcf06d80d/docs/ibc/events.md
    #[serde(rename = "create_client")]
    CreateClient {
        client_id: ClientId,
        // TODO: Figure out if there's a better type we can use than string
        client_type: String,
        consensus_height: Height,
    },

    #[serde(rename = "update_client")]
    UpdateClient {
        client_id: ClientId,
        client_type: String,
        #[serde(with = "height_list_comma_separated")]
        consensus_heights: Vec<Height>,
    },

    #[serde(rename = "client_misbehaviour")]
    ClientMisbehaviour {
        client_id: ClientId,
        client_type: String,
        consensus_height: Height,
    },

    #[serde(rename = "connection_open_init")]
    ConnectionOpenInit {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
    },

    #[serde(rename = "connection_open_try")]
    ConnectionOpenTry {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "connection_open_ack")]
    ConnectionOpenAck {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "connection_open_confirm")]
    ConnectionOpenConfirm {
        connection_id: ConnectionId,
        client_id: ClientId,
        counterparty_client_id: ClientId,
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "channel_open_init")]
    ChannelOpenInit {
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        connection_id: ConnectionId,
        version: String,
    },

    #[serde(rename = "channel_open_try")]
    ChannelOpenTry {
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
        version: String,
    },

    #[serde(rename = "channel_open_ack")]
    ChannelOpenAck {
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    },

    #[serde(rename = "channel_open_confirm")]
    ChannelOpenConfirm {
        port_id: PortId,
        channel_id: ChannelId,
        counterparty_port_id: PortId,
        counterparty_channel_id: ChannelId,
        connection_id: ConnectionId,
    },

    #[serde(rename = "write_acknowledgement")]
    WriteAcknowledgement {
        packet_data_hex: Bytes<HexUnprefixed>,
        packet_timeout_height: Height,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: u64,
        #[serde(with = "serde_utils::string")]
        packet_sequence: NonZeroU64,
        packet_src_port: PortId,
        packet_src_channel: ChannelId,
        packet_dst_port: PortId,
        packet_dst_channel: ChannelId,
        packet_ack_hex: Bytes<HexUnprefixed>,
        connection_id: ConnectionId,
    },

    #[serde(rename = "recv_packet")]
    RecvPacket {
        packet_data_hex: Bytes<HexUnprefixed>,
        packet_timeout_height: Height,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: u64,
        #[serde(with = "serde_utils::string")]
        packet_sequence: NonZeroU64,
        packet_src_port: PortId,
        packet_src_channel: ChannelId,
        packet_dst_port: PortId,
        packet_dst_channel: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_channel_ordering: Order,
        connection_id: ConnectionId,
    },

    #[serde(rename = "send_packet")]
    SendPacket {
        packet_data_hex: Bytes<HexUnprefixed>,
        packet_timeout_height: Height,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: u64,
        #[serde(with = "serde_utils::string")]
        packet_sequence: NonZeroU64,
        packet_src_port: PortId,
        packet_src_channel: ChannelId,
        packet_dst_port: PortId,
        packet_dst_channel: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_channel_ordering: Order,
        connection_id: ConnectionId,
    },

    #[serde(rename = "acknowledge_packet")]
    AcknowledgePacket {
        packet_timeout_height: Height,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: u64,
        #[serde(with = "serde_utils::string")]
        packet_sequence: NonZeroU64,
        packet_src_port: PortId,
        packet_src_channel: ChannelId,
        packet_dst_port: PortId,
        packet_dst_channel: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_channel_ordering: Order,
        connection_id: ConnectionId,
    },

    #[serde(rename = "timeout_packet")]
    TimeoutPacket {
        packet_timeout_height: Height,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: u64,
        #[serde(with = "serde_utils::string")]
        packet_sequence: NonZeroU64,
        packet_src_port: PortId,
        packet_src_channel: ChannelId,
        packet_dst_port: PortId,
        packet_dst_channel: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_channel_ordering: Order,
        connection_id: ConnectionId,
    },

    // events for the union IBC specification, emitted by the cosmwasm contract implementation.
    #[serde(rename = "wasm-create_client")]
    UnionCreateClient {
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        // TODO: Figure out if there's a better type we can use than string
        client_type: String,
        // #[serde(with = "serde_utils::string")]
        // height: u64,
    },

    #[serde(rename = "wasm-update_client")]
    UnionUpdateClient {
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_height: u64,
    },

    #[serde(rename = "wasm-connection_open_init")]
    UnionConnectionOpenInit {
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: u32,
    },

    #[serde(rename = "wasm-connection_open_try")]
    UnionConnectionOpenTry {
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: u32,
    },

    #[serde(rename = "wasm-connection_open_ack")]
    UnionConnectionOpenAck {
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: u32,
    },

    #[serde(rename = "wasm-connection_open_confirm")]
    UnionConnectionOpenConfirm {
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: u32,
    },

    #[serde(rename = "wasm-channel_open_init")]
    UnionChannelOpenInit {
        port_id: String,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        version: String,
    },

    #[serde(rename = "wasm-channel_open_try")]
    UnionChannelOpenTry {
        port_id: String,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: u32,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        counterparty_version: String,
    },

    #[serde(rename = "wasm-channel_open_ack")]
    UnionChannelOpenAck {
        port_id: String,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: u32,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
    },

    #[serde(rename = "wasm-channel_open_confirm")]
    UnionChannelOpenConfirm {
        port_id: String,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: u32,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
    },

    #[serde(rename = "wasm-send_packet")]
    UnionSendPacket {
        #[serde(with = "stringified_json")]
        packet: ibc_solidity::Packet,
    },

    #[serde(rename = "wasm-recv_packet")]
    UnionRecvPacket {
        #[serde(with = "stringified_json")]
        packet: ibc_solidity::Packet,
        relayer_msg: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-acknowledge_packet")]
    UnionAcknowledgePacket {
        #[serde(with = "stringified_json")]
        packet: ibc_solidity::Packet,
        acknowledgement: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-write_ack")]
    UnionWriteAck {
        #[serde(with = "stringified_json")]
        packet: ibc_solidity::Packet,
        acknowledgement: Bytes<HexUnprefixed>,
    },
}

// TODO: Check if human readable
pub mod stringified_json {
    use std::string::String;

    use serde::{
        de::{Deserialize, DeserializeOwned},
        Deserializer, Serialize, Serializer,
    };

    pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        serde_json::to_string(&data)
            .expect("serialization is infallible; qed;")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        String::deserialize(deserializer)
            .and_then(|s| serde_json::from_str(&s).map_err(serde::de::Error::custom))
    }
}

// TODO: Check if human readable
pub mod height_list_comma_separated {
    use std::string::String;

    use serde::{
        de::{self, Deserialize},
        Deserializer, Serialize, Serializer,
    };
    use unionlabs::ibc::core::client::height::Height;

    pub fn serialize<S>(data: &Vec<Height>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.into_iter()
            .map(|height| format!("{height:#}"))
            .collect::<Vec<_>>()
            .join(",")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Height>, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .split(',')
            .map(Height::from_str_allow_zero_revision)
            .collect::<Result<_, _>>()
            .map_err(de::Error::custom)
    }
}

impl IbcEvent {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            IbcEvent::CreateClient { .. } => "create_client",
            IbcEvent::UpdateClient { .. } => "update_client",
            IbcEvent::ClientMisbehaviour { .. } => "client_misbehaviour",
            IbcEvent::ConnectionOpenInit { .. } => "connection_open_init",
            IbcEvent::ConnectionOpenTry { .. } => "connection_open_try",
            IbcEvent::ConnectionOpenAck { .. } => "connection_open_ack",
            IbcEvent::ConnectionOpenConfirm { .. } => "connection_open_confirm",
            IbcEvent::ChannelOpenInit { .. } => "channel_open_init",
            IbcEvent::ChannelOpenTry { .. } => "channel_open_try",
            IbcEvent::ChannelOpenAck { .. } => "channel_open_ack",
            IbcEvent::ChannelOpenConfirm { .. } => "channel_open_confirm",
            IbcEvent::WriteAcknowledgement { .. } => "write_ack",
            IbcEvent::RecvPacket { .. } => "recv_packet",
            IbcEvent::SendPacket { .. } => "send_packet",
            IbcEvent::AcknowledgePacket { .. } => "acknowledge_packet",
            IbcEvent::TimeoutPacket { .. } => "timeout_packet",

            IbcEvent::UnionCreateClient { .. } => "create_client",
            IbcEvent::UnionUpdateClient { .. } => "update_client",
            // IbcEvent::UnionClientMisbehaviour{..} => "client_misbehaviour",
            // IbcEvent::UnionSubmitEvidence{..} => "submit_evidence",
            IbcEvent::UnionConnectionOpenInit { .. } => "connection_open_init",
            IbcEvent::UnionConnectionOpenTry { .. } => "connection_open_try",
            IbcEvent::UnionConnectionOpenAck { .. } => "connection_open_ack",
            IbcEvent::UnionConnectionOpenConfirm { .. } => "connection_open_confirm",
            IbcEvent::UnionChannelOpenInit { .. } => "channel_open_init",
            IbcEvent::UnionChannelOpenTry { .. } => "channel_open_try",
            IbcEvent::UnionChannelOpenAck { .. } => "channel_open_ack",
            IbcEvent::UnionChannelOpenConfirm { .. } => "channel_open_confirm",
            IbcEvent::UnionRecvPacket { .. } => "recv_packet",
            IbcEvent::UnionSendPacket { .. } => "send_packet",
            IbcEvent::UnionAcknowledgePacket { .. } => "acknowledge_packet",
            IbcEvent::UnionWriteAck { .. } => "write_ack",
            // IbcEvent::UnionTimeoutPacket{..} => "timeout_packet",
        }
    }
}
