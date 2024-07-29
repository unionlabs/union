use serde::{Deserialize, Serialize};
use unionlabs::id::ClientId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum IbcEvent {
    CreateClient {
        client_id: ClientId,
        // TODO: Figure out if there's a better type we can use than string
        client_type: String,
        consensus_height: Height,
    },

    UpdateClient {
        client_id: ClientId,
        client_type: String,
        consensus_heights: Vec<Height>,
    },
    // ConnectionOpenInit {
    //     connection_id: ConnectionId,
    //     client_id: ClientId,
    //     counterparty_client_id: ClientId,
    // },

    // ConnectionOpenTry {
    //     connection_id: ConnectionId,
    //     client_id: ClientId,
    //     counterparty_client_id: ClientId,
    //     counterparty_connection_id: ConnectionId,
    // },

    // ConnectionOpenAck {
    //     connection_id: ConnectionId,
    //     client_id: ClientId,
    //     counterparty_client_id: ClientId,
    //     counterparty_connection_id: ConnectionId,
    // },

    // ConnectionOpenConfirm {
    //     connection_id: ConnectionId,
    //     client_id: ClientId,
    //     counterparty_client_id: ClientId,
    //     counterparty_connection_id: ConnectionId,
    // },

    // ChannelOpenInit {
    //     port_id: PortId,
    //     channel_id: ChannelId,
    //     counterparty_port_id: PortId,
    //     connection_id: ConnectionId,
    //     version: String,
    // },

    // ChannelOpenTry {
    //     port_id: PortId,
    //     channel_id: ChannelId,
    //     counterparty_port_id: PortId,
    //     counterparty_channel_id: ChannelId,
    //     connection_id: ConnectionId,
    //     version: String,
    // },

    // ChannelOpenAck {
    //     port_id: PortId,
    //     channel_id: ChannelId,
    //     counterparty_port_id: PortId,
    //     counterparty_channel_id: ChannelId,
    //     connection_id: ConnectionId,
    // },

    // ChannelOpenConfirm {
    //     port_id: PortId,
    //     channel_id: ChannelId,
    //     counterparty_port_id: PortId,
    //     counterparty_channel_id: ChannelId,
    //     connection_id: ConnectionId,
    // },

    // WriteAcknowledgement {
    //     #[serde(with = "::serde_utils::hex_string")]
    //     #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    //     packet_data_hex: Vec<u8>,
    //     packet_timeout_height: Height,
    //     packet_timeout_timestamp: u64,
    //     packet_sequence: NonZeroU64,
    //     packet_src_port: PortId,
    //     packet_src_channel: ChannelId,
    //     packet_dst_port: PortId,
    //     packet_dst_channel: ChannelId,
    //     #[serde(with = "::serde_utils::hex_string")]
    //     #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    //     packet_ack_hex: Vec<u8>,
    //     connection_id: ConnectionId,
    // },

    // RecvPacket {
    //     #[serde(with = "::serde_utils::hex_string")]
    //     #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    //     packet_data_hex: Vec<u8>,
    //     packet_timeout_height: Height,
    //     packet_timeout_timestamp: u64,
    //     packet_sequence: NonZeroU64,
    //     packet_src_port: PortId,
    //     packet_src_channel: ChannelId,
    //     packet_dst_port: PortId,
    //     packet_dst_channel: ChannelId,
    //     packet_channel_ordering: Order,
    //     connection_id: ConnectionId,
    // },

    // SendPacket {
    //     #[serde(with = "::serde_utils::hex_string")]
    //     #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    //     packet_data_hex: Vec<u8>,
    //     // TODO: Make this generic height instead of concrete
    //     packet_timeout_height: Height,
    //     packet_timeout_timestamp: u64,
    //     packet_sequence: NonZeroU64,
    //     packet_src_port: PortId,
    //     packet_src_channel: ChannelId,
    //     packet_dst_port: PortId,
    //     packet_dst_channel: ChannelId,
    //     packet_channel_ordering: Order,
    //     connection_id: ConnectionId,
    // },

    // AcknowledgePacket {
    //     packet_timeout_height: Height,
    //     packet_timeout_timestamp: u64,
    //     packet_sequence: NonZeroU64,
    //     packet_src_port: PortId,
    //     packet_src_channel: ChannelId,
    //     packet_dst_port: PortId,
    //     packet_dst_channel: ChannelId,
    //     packet_channel_ordering: Order,
    //     connection_id: ConnectionId,
    // },

    // TimeoutPacket {
    //     packet_timeout_height: Height,
    //     packet_timeout_timestamp: u64,
    //     packet_sequence: NonZeroU64,
    //     packet_src_port: PortId,
    //     packet_src_channel: ChannelId,
    //     packet_dst_port: PortId,
    //     packet_dst_channel: ChannelId,
    //     packet_channel_ordering: Order,
    //     connection_id: ConnectionId,
    // },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Height {
    #[serde(with = "::serde_utils::string")]
    revision_height: u64,
    #[serde(with = "::serde_utils::string")]
    revision_number: u64,
}

impl From<Height> for unionlabs::ibc::core::client::height::Height {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}
