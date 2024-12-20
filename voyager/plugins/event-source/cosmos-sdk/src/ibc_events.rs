use std::{num::NonZeroU64, str::FromStr};

use cosmos_sdk_event::event;
use unionlabs::{
    bytes::Bytes,
    hash::hash_v2::HexUnprefixed,
    ibc::core::{channel::order::Order, client::height::Height},
    id::{ChannelId, ClientId, ConnectionId, PortId},
};

event! {
    pub enum IbcEvent {
        // standard ibc-go events for IBC classic
        // https://github.com/cosmos/ibc-go/blob/5c7f28634ecf9b6f275bfd5712778fedcf06d80d/docs/ibc/events.md
        #[event(tag = "create_client")]
        CreateClient {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            // TODO: Figure out if there's a better type we can use than string
            client_type: String,
            #[parse(Height::from_str_allow_zero_revision)]
            consensus_height: Height,
        },

        #[event(tag = "update_client", deprecated("consensus_height", "header"))]
        UpdateClient {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            client_type: String,
            #[parse(|s: &str| s.split(',').map(Height::from_str_allow_zero_revision).collect::<Result<_, _>>())]
            consensus_heights: Vec<Height>,
        },

        #[event(tag = "client_misbehaviour")]
        ClientMisbehaviour {
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            client_type: String,
            #[parse(Height::from_str_allow_zero_revision)]
            consensus_height: Height,
        },

        #[event(tag = "submit_evidence")]
        SubmitEvidence { evidence_hash: String },

        #[event(tag = "connection_open_init", deprecated("counterparty_connection_id"))]
        ConnectionOpenInit {
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientId::from_str)]
            counterparty_client_id: ClientId,
        },

        #[event(tag = "connection_open_try")]
        ConnectionOpenTry {
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientId::from_str)]
            counterparty_client_id: ClientId,
            #[parse(ConnectionId::from_str_prefixed)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "connection_open_ack")]
        ConnectionOpenAck {
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientId::from_str)]
            counterparty_client_id: ClientId,
            #[parse(ConnectionId::from_str_prefixed)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "connection_open_confirm")]
        ConnectionOpenConfirm {
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            #[parse(ClientId::from_str)]
            client_id: ClientId,
            #[parse(ClientId::from_str)]
            counterparty_client_id: ClientId,
            #[parse(ConnectionId::from_str_prefixed)]
            counterparty_connection_id: ConnectionId,
        },

        #[event(tag = "channel_open_init", deprecated("counterparty_channel_id"))]
        ChannelOpenInit {
            #[parse(PortId::from_str)]
            port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            channel_id: ChannelId,
            #[parse(PortId::from_str)]
            counterparty_port_id: PortId,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            version: String,
        },

        #[event(tag = "channel_open_try")]
        ChannelOpenTry {
            #[parse(PortId::from_str)]
            port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            channel_id: ChannelId,
            #[parse(PortId::from_str)]
            counterparty_port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
            version: String,
        },

        #[event(tag = "channel_open_ack")]
        ChannelOpenAck {
            #[parse(PortId::from_str)]
            port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            channel_id: ChannelId,
            #[parse(PortId::from_str)]
            counterparty_port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(tag = "channel_open_confirm")]
        ChannelOpenConfirm {
            #[parse(PortId::from_str)]
            port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            channel_id: ChannelId,
            #[parse(PortId::from_str)]
            counterparty_port_id: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            counterparty_channel_id: ChannelId,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(
            tag = "write_acknowledgement",
            deprecated("packet_data", "packet_ack", "packet_connection")
        )]
        WriteAcknowledgement {
            #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
            packet_data_hex: Bytes,
            #[parse(Height::from_str_allow_zero_revision)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(NonZeroU64::from_str)]
            packet_sequence: NonZeroU64,
            #[parse(PortId::from_str)]
            packet_src_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_src_channel: ChannelId,
            #[parse(PortId::from_str)]
            packet_dst_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_dst_channel: ChannelId,
            #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
            packet_ack_hex: Bytes,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(tag = "recv_packet", deprecated("packet_data", "packet_connection"))]
        RecvPacket {
            #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
            packet_data_hex: Bytes,
            #[parse(Height::from_str_allow_zero_revision)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(NonZeroU64::from_str)]
            packet_sequence: NonZeroU64,
            #[parse(PortId::from_str)]
            packet_src_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_src_channel: ChannelId,
            #[parse(PortId::from_str)]
            packet_dst_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(tag = "send_packet", deprecated("packet_data", "packet_connection"))]
        SendPacket {
            #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
            packet_data_hex: Bytes,
            #[parse(Height::from_str_allow_zero_revision)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(NonZeroU64::from_str)]
            packet_sequence: NonZeroU64,
            #[parse(PortId::from_str)]
            packet_src_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_src_channel: ChannelId,
            #[parse(PortId::from_str)]
            packet_dst_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(tag = "acknowledge_packet", deprecated("packet_connection"))]
        AcknowledgePacket {
            #[parse(Height::from_str_allow_zero_revision)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(NonZeroU64::from_str)]
            packet_sequence: NonZeroU64,
            #[parse(PortId::from_str)]
            packet_src_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_src_channel: ChannelId,
            #[parse(PortId::from_str)]
            packet_dst_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        #[event(tag = "timeout_packet")]
        TimeoutPacket {
            #[parse(Height::from_str_allow_zero_revision)]
            packet_timeout_height: Height,
            #[parse(u64::from_str)]
            packet_timeout_timestamp: u64,
            #[parse(NonZeroU64::from_str)]
            packet_sequence: NonZeroU64,
            #[parse(PortId::from_str)]
            packet_src_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_src_channel: ChannelId,
            #[parse(PortId::from_str)]
            packet_dst_port: PortId,
            #[parse(ChannelId::from_str_prefixed)]
            packet_dst_channel: ChannelId,
            #[parse(Order::from_str)]
            packet_channel_ordering: Order,
            #[parse(ConnectionId::from_str_prefixed)]
            connection_id: ConnectionId,
        },

        // events for the union IBC specification, emitted by the cosmwasm contract implementation.

        #[event(tag = "wasm-create_client")]
        UnionCreateClient {
            #[parse(u32::from_str)]
            client_id: u32,
            // TODO: Figure out if there's a better type we can use than string
            client_type: String,
            // #[parse(u64::from_str)]
            // height: u64,
        },

        #[event(tag = "wasm-update_client")]
        UnionUpdateClient {
            #[parse(u32::from_str)]
            client_id: u32,
            #[parse(u64::from_str)]
            counterparty_height: u64,
        },

        #[event(tag = "wasm-connection_open_init")]
        UnionConnectionOpenInit {
            #[parse(u32::from_str)]
            connection_id: u32,
            #[parse(u32::from_str)]
            client_id: u32,
            #[parse(u32::from_str)]
            counterparty_client_id: u32,
        },

        #[event(tag = "wasm-connection_open_try")]
        UnionConnectionOpenTry {
            #[parse(u32::from_str)]
            connection_id: u32,
            #[parse(u32::from_str)]
            client_id: u32,
            #[parse(u32::from_str)]
            counterparty_client_id: u32,
            #[parse(u32::from_str)]
            counterparty_connection_id: u32,
        },

        #[event(tag = "wasm-connection_open_ack")]
        UnionConnectionOpenAck {
            #[parse(u32::from_str)]
            connection_id: u32,
            #[parse(u32::from_str)]
            client_id: u32,
            #[parse(u32::from_str)]
            counterparty_client_id: u32,
            #[parse(u32::from_str)]
            counterparty_connection_id: u32,
        },

        #[event(tag = "wasm-connection_open_confirm")]
        UnionConnectionOpenConfirm {
            #[parse(u32::from_str)]
            connection_id: u32,
            #[parse(u32::from_str)]
            client_id: u32,
            #[parse(u32::from_str)]
            counterparty_client_id: u32,
            #[parse(u32::from_str)]
            counterparty_connection_id: u32,
        },

        // #[event(tag = "channel_open_init", deprecated("counterparty_channel_id"))]
        // ChannelOpenInit {
        //     #[parse(String::from_str)]
        //     port_id: String,
        //     #[parse(u32::from_str)]
        //     channel_id: u32,
        //     #[parse(String::from_str)]
        //     counterparty_port_id: String,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        //     version: String,
        // },

        #[event(tag = "wasm-channel_open_try")]
        UnionChannelOpenTry {
            port_id: String,
            #[parse(u32::from_str)]
            channel_id: u32,
            #[parse(<Bytes<HexUnprefixed>>::from_str)]
            counterparty_port_id: Bytes<HexUnprefixed>,
            #[parse(u32::from_str)]
            counterparty_channel_id: u32,
            #[parse(u32::from_str)]
            connection_id: u32,
            counterparty_version: String,
        },

        // #[event(tag = "channel_open_ack")]
        // ChannelOpenAck {
        //     #[parse(String::from_str)]
        //     port_id: String,
        //     #[parse(u32::from_str)]
        //     channel_id: u32,
        //     #[parse(String::from_str)]
        //     counterparty_port_id: String,
        //     #[parse(u32::from_str)]
        //     counterparty_channel_id: u32,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },

        #[event(tag = "wasm-channel_open_confirm")]
        UnionChannelOpenConfirm {
            #[parse(String::from_str)]
            port_id: String,
            #[parse(u32::from_str)]
            channel_id: u32,
            #[parse(<Bytes<HexUnprefixed>>::from_str)]
            counterparty_port_id: Bytes<HexUnprefixed>,
            #[parse(u32::from_str)]
            counterparty_channel_id: u32,
            #[parse(u32::from_str)]
            connection_id: u32,
        },

        #[event(tag = "wasm-packet_send")]
        UnionSendPacket {
            #[parse(serde_json::from_str)]
            packet: ibc_solidity::Packet,
        },

        // #[event(
        //     tag = "write_acknowledgement",
        //     deprecated("packet_data", "packet_ack", "packet_connection")
        // )]
        // WriteAcknowledgement {
        //     #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
        //     packet_data_hex: Bytes,
        //     #[parse(u64::from_str)]
        //     packet_timeout_height: Height,
        //     #[parse(u64::from_str)]
        //     packet_timeout_timestamp: u64,
        //     #[parse(NonZeroU64::from_str)]
        //     packet_sequence: NonZeroU64,
        //     #[parse(String::from_str)]
        //     packet_src_port: String,
        //     #[parse(u32::from_str)]
        //     packet_src_channel: u32,
        //     #[parse(String::from_str)]
        //     packet_dst_port: String,
        //     #[parse(u32::from_str)]
        //     packet_dst_channel: u32,
        //     #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
        //     packet_ack_hex: Bytes,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },

        // #[event(tag = "recv_packet", deprecated("packet_data", "packet_connection"))]
        // RecvPacket {
        //     #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
        //     packet_data_hex: Bytes,
        //     #[parse(u64::from_str)]
        //     packet_timeout_height: Height,
        //     #[parse(u64::from_str)]
        //     packet_timeout_timestamp: u64,
        //     #[parse(NonZeroU64::from_str)]
        //     packet_sequence: NonZeroU64,
        //     #[parse(String::from_str)]
        //     packet_src_port: String,
        //     #[parse(u32::from_str)]
        //     packet_src_channel: u32,
        //     #[parse(String::from_str)]
        //     packet_dst_port: String,
        //     #[parse(u32::from_str)]
        //     packet_dst_channel: u32,
        //     #[parse(Order::from_str)]
        //     packet_channel_ordering: Order,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },

        // #[event(tag = "send_packet", deprecated("packet_data", "packet_connection"))]
        // SendPacket {
        //     #[parse(|s: &str| s.parse::<Bytes<HexUnprefixed>>().map(|b| b.into_encoding()))]
        //     packet_data_hex: Bytes,
        //     #[parse(u64::from_str)]
        //     packet_timeout_height: Height,
        //     #[parse(u64::from_str)]
        //     packet_timeout_timestamp: u64,
        //     #[parse(NonZeroU64::from_str)]
        //     packet_sequence: NonZeroU64,
        //     #[parse(String::from_str)]
        //     packet_src_port: String,
        //     #[parse(u32::from_str)]
        //     packet_src_channel: u32,
        //     #[parse(String::from_str)]
        //     packet_dst_port: String,
        //     #[parse(u32::from_str)]
        //     packet_dst_channel: u32,
        //     #[parse(Order::from_str)]
        //     packet_channel_ordering: Order,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },

        // #[event(tag = "acknowledge_packet", deprecated("packet_connection"))]
        // AcknowledgePacket {
        //     #[parse(u64::from_str)]
        //     packet_timeout_height: Height,
        //     #[parse(u64::from_str)]
        //     packet_timeout_timestamp: u64,
        //     #[parse(NonZeroU64::from_str)]
        //     packet_sequence: NonZeroU64,
        //     #[parse(String::from_str)]
        //     packet_src_port: String,
        //     #[parse(u32::from_str)]
        //     packet_src_channel: u32,
        //     #[parse(String::from_str)]
        //     packet_dst_port: String,
        //     #[parse(u32::from_str)]
        //     packet_dst_channel: u32,
        //     #[parse(Order::from_str)]
        //     packet_channel_ordering: Order,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },

        // #[event(tag = "timeout_packet")]
        // TimeoutPacket {
        //     #[parse(u64::from_str)]
        //     packet_timeout_height: Height,
        //     #[parse(u64::from_str)]
        //     packet_timeout_timestamp: u64,
        //     #[parse(NonZeroU64::from_str)]
        //     packet_sequence: NonZeroU64,
        //     #[parse(String::from_str)]
        //     packet_src_port: String,
        //     #[parse(u32::from_str)]
        //     packet_src_channel: u32,
        //     #[parse(String::from_str)]
        //     packet_dst_port: String,
        //     #[parse(u32::from_str)]
        //     packet_dst_channel: u32,
        //     #[parse(Order::from_str)]
        //     packet_channel_ordering: Order,
        //     #[parse(u32::from_str)]
        //     connection_id: u32,
        // },
    }
}

impl IbcEvent {
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            IbcEvent::CreateClient(_) => "create_client",
            IbcEvent::UpdateClient(_) => "update_client",
            IbcEvent::ClientMisbehaviour(_) => "client_misbehaviour",
            IbcEvent::SubmitEvidence(_) => "submit_evidence",
            IbcEvent::ConnectionOpenInit(_) => "connection_open_init",
            IbcEvent::ConnectionOpenTry(_) => "connection_open_try",
            IbcEvent::ConnectionOpenAck(_) => "connection_open_ack",
            IbcEvent::ConnectionOpenConfirm(_) => "connection_open_confirm",
            IbcEvent::ChannelOpenInit(_) => "channel_open_init",
            IbcEvent::ChannelOpenTry(_) => "channel_open_try",
            IbcEvent::ChannelOpenAck(_) => "channel_open_ack",
            IbcEvent::ChannelOpenConfirm(_) => "channel_open_confirm",
            IbcEvent::WriteAcknowledgement(_) => "write_acknowledgement",
            IbcEvent::RecvPacket(_) => "recv_packet",
            IbcEvent::SendPacket(_) => "send_packet",
            IbcEvent::AcknowledgePacket(_) => "acknowledge_packet",
            IbcEvent::TimeoutPacket(_) => "timeout_packet",

            IbcEvent::UnionCreateClient(_) => "create_client",
            IbcEvent::UnionUpdateClient(_) => "update_client",
            // IbcEvent::UnionClientMisbehaviour(_) => "client_misbehaviour",
            // IbcEvent::UnionSubmitEvidence(_) => "submit_evidence",
            IbcEvent::UnionConnectionOpenInit(_) => "connection_open_init",
            IbcEvent::UnionConnectionOpenTry(_) => "connection_open_try",
            IbcEvent::UnionConnectionOpenAck(_) => "connection_open_ack",
            IbcEvent::UnionConnectionOpenConfirm(_) => "connection_open_confirm",
            // IbcEvent::UnionChannelOpenInit(_) => "channel_open_init",
            IbcEvent::UnionChannelOpenTry(_) => "channel_open_try",
            // IbcEvent::UnionChannelOpenAck(_) => "channel_open_ack",
            IbcEvent::UnionChannelOpenConfirm(_) => "channel_open_confirm",
            // IbcEvent::UnionWriteAcknowledgement(_) => "write_acknowledgement",
            // IbcEvent::UnionRecvPacket(_) => "recv_packet",
            IbcEvent::UnionSendPacket(_) => "send_packet",
            // IbcEvent::UnionAcknowledgePacket(_) => "acknowledge_packet",
            // IbcEvent::UnionTimeoutPacket(_) => "timeout_packet",
        }
    }
}
