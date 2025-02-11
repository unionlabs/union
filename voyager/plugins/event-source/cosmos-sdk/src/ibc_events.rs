use std::{io::Write, num::NonZeroU64};

use ibc_union_spec::types::Packet;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use unionlabs::{
    bech32::Bech32,
    ibc::core::{channel::order::Order, client::height::Height},
    id::{ChannelId, ClientId, ConnectionId, PortId},
    primitives::{encoding::HexUnprefixed, Bytes, H256},
    ErrorReporter,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode)]
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
    WasmCreateClient {
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        // TODO: Figure out if there's a better type we can use than string
        client_type: String,
        // #[serde(with = "serde_utils::string")]
        // height: u64,
    },

    #[serde(rename = "wasm-update_client")]
    WasmUpdateClient {
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_height: u64,
    },

    #[serde(rename = "wasm-connection_open_init")]
    WasmConnectionOpenInit {
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        #[serde(with = "serde_utils::string")]
        client_id: u32,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: u32,
    },

    #[serde(rename = "wasm-connection_open_try")]
    WasmConnectionOpenTry {
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
    WasmConnectionOpenAck {
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
    WasmConnectionOpenConfirm {
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
    WasmChannelOpenInit {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
        version: String,
    },

    #[serde(rename = "wasm-channel_open_try")]
    WasmChannelOpenTry {
        port_id: Bech32<H256>,
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
    WasmChannelOpenAck {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: u32,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
    },

    #[serde(rename = "wasm-channel_open_confirm")]
    WasmChannelOpenConfirm {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: u32,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: u32,
        #[serde(with = "serde_utils::string")]
        connection_id: u32,
    },

    #[serde(rename = "wasm-packet_send")]
    WasmPacketSend {
        #[serde(with = "stringified_json")]
        packet: Packet,
    },

    #[serde(rename = "wasm-packet_recv")]
    WasmPacketRecv {
        #[serde(with = "stringified_json")]
        packet: Packet,
        maker: Bech32<Bytes>,
        maker_msg: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-packet_ack")]
    WasmPacketAck {
        #[serde(with = "stringified_json")]
        packet: Packet,
        acknowledgement: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-write_ack")]
    WasmWriteAck {
        #[serde(with = "stringified_json")]
        packet: Packet,
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

    #[allow(clippy::ptr_arg)] // required by serde
    pub fn serialize<S>(data: &Vec<Height>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.iter()
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
    pub fn hash(&self) -> H256 {
        struct Sha256Writer(sha2::Sha256);

        // i don't really get why they needed their own trait that's not interoperable with std::io::Write but ok
        impl bincode::enc::write::Writer for Sha256Writer {
            fn write(&mut self, bytes: &[u8]) -> Result<(), bincode::error::EncodeError> {
                self.0
                    .write(bytes)
                    .map_err(|e| {
                        bincode::error::EncodeError::OtherString(ErrorReporter(e).to_string())
                    })
                    .map(|_| ())
            }
        }

        let mut hasher = Sha256Writer(sha2::Sha256::new());
        bincode::encode_into_writer(self, &mut hasher, bincode::config::standard()).unwrap();
        hasher.0.finalize().into()
    }

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

            IbcEvent::WasmCreateClient { .. } => "create_client",
            IbcEvent::WasmUpdateClient { .. } => "update_client",
            // IbcEvent::UnionClientMisbehaviour{..} => "client_misbehaviour",
            // IbcEvent::UnionSubmitEvidence{..} => "submit_evidence",
            IbcEvent::WasmConnectionOpenInit { .. } => "connection_open_init",
            IbcEvent::WasmConnectionOpenTry { .. } => "connection_open_try",
            IbcEvent::WasmConnectionOpenAck { .. } => "connection_open_ack",
            IbcEvent::WasmConnectionOpenConfirm { .. } => "connection_open_confirm",
            IbcEvent::WasmChannelOpenInit { .. } => "channel_open_init",
            IbcEvent::WasmChannelOpenTry { .. } => "channel_open_try",
            IbcEvent::WasmChannelOpenAck { .. } => "channel_open_ack",
            IbcEvent::WasmChannelOpenConfirm { .. } => "channel_open_confirm",
            IbcEvent::WasmPacketRecv { .. } => "recv_packet",
            IbcEvent::WasmPacketSend { .. } => "send_packet",
            IbcEvent::WasmPacketAck { .. } => "acknowledge_packet",
            IbcEvent::WasmWriteAck { .. } => "write_ack",
            // IbcEvent::UnionTimeoutPacket{..} => "timeout_packet",
        }
    }
}
