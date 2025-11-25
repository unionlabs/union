use std::io::Write;

use ibc_union_spec::{ChannelId, ClientId, ConnectionId, Timestamp};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use unionlabs::{
    ErrorReporter,
    primitives::{Bech32, Bytes, H256, encoding::HexUnprefixed},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bincode::Encode)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum IbcEvent {
    // events for the union IBC specification, emitted by the cosmwasm contract implementation.
    #[serde(rename = "wasm-create_client")]
    WasmCreateClient {
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        // TODO: Figure out if there's a better type we can use than string
        client_type: String,
        // #[serde(with = "serde_utils::string")]
        // height: u64,
    },

    #[serde(rename = "wasm-update_client")]
    WasmUpdateClient {
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_height: u64,
    },

    #[serde(rename = "wasm-connection_open_init")]
    WasmConnectionOpenInit {
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: ClientId,
    },

    #[serde(rename = "wasm-connection_open_try")]
    WasmConnectionOpenTry {
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-connection_open_ack")]
    WasmConnectionOpenAck {
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-connection_open_confirm")]
    WasmConnectionOpenConfirm {
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-channel_open_init")]
    WasmChannelOpenInit {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        version: String,
    },

    #[serde(rename = "wasm-channel_open_try")]
    WasmChannelOpenTry {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        counterparty_version: String,
    },

    #[serde(rename = "wasm-channel_open_ack")]
    WasmChannelOpenAck {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-channel_open_confirm")]
    WasmChannelOpenConfirm {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-packet_send")]
    WasmPacketSend {
        #[serde(with = "serde_utils::string")]
        packet_source_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_destination_channel_id: ChannelId,
        packet_data: Bytes,
        #[serde(with = "serde_utils::string")]
        packet_timeout_height: u64,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: Timestamp,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
    },

    #[serde(rename = "wasm-batch_send")]
    WasmBatchSend {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        batch_hash: H256,
    },

    #[serde(rename = "wasm-packet_recv")]
    WasmPacketRecv {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        maker: Bech32<Bytes>,
        maker_msg: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-packet_ack")]
    WasmPacketAck {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        acknowledgement: Bytes<HexUnprefixed>,
    },

    #[serde(rename = "wasm-write_ack")]
    WasmWriteAck {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        acknowledgement: Bytes<HexUnprefixed>,
    },
}

impl IbcEvent {
    pub fn is_trivial(&self) -> bool {
        matches!(
            self,
            Self::WasmCreateClient { .. }
                | Self::WasmUpdateClient { .. }
                | Self::WasmPacketRecv { .. }
                // | Self::WasmPacketIntentRecv { .. }
                // | Self::WasmPacketTimeout { .. }
                | Self::WasmPacketAck { .. }
        )
    }
}

// TODO: Check if human readable
pub mod height_list_comma_separated {
    use std::string::String;

    use serde::{
        Deserializer, Serialize, Serializer,
        de::{self, Deserialize},
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
        bincode::encode_into_writer(self, &mut hasher, bincode::config::standard())
            .expect("encoding is infallible; qed;");
        hasher.0.finalize().into()
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            IbcEvent::WasmCreateClient { .. } => "create_client",
            IbcEvent::WasmUpdateClient { .. } => "update_client",
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
            IbcEvent::WasmBatchSend { .. } => "batch_send",
            IbcEvent::WasmPacketAck { .. } => "acknowledge_packet",
            IbcEvent::WasmWriteAck { .. } => "write_ack",
            // IbcEvent::UnionTimeoutPacket{..} => "timeout_packet",
        }
    }
}
