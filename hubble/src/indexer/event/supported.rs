use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::indexer::event::{
    channel_open_ack_event::ChannelOpenAckEvent,
    channel_open_confirm_event::ChannelOpenConfirmEvent,
    channel_open_init_event::ChannelOpenInitEvent, channel_open_try_event::ChannelOpenTryEvent,
    connection_open_ack_event::ConnectionOpenAckEvent,
    connection_open_confirm_event::ConnectionOpenConfirmEvent,
    connection_open_init_event::ConnectionOpenInitEvent,
    connection_open_try_event::ConnectionOpenTryEvent, create_client_event::CreateClientEvent,
    create_lens_client_event::CreateLensClientEvent, packet_ack_event::PacketAckEvent,
    packet_recv_event::PacketRecvEvent, packet_send_event::PacketSendEvent,
    packet_timeout_event::PacketTimeoutEvent, token_bucket_update_event::TokenBucketUpdateEvent,
    types::BlockHeight, update_client_event::UpdateClientEvent,
    wallet_mutation_entry_event::WalletMutationEntryEvent, write_ack_event::WriteAckEvent,
};

#[warn(clippy::enum_variant_names)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SupportedBlockEvent {
    // using database representation of fields. all these 'record representations' will
    // be replaced by USC events
    #[serde(rename = "ethereum-log")]
    EthereumLog {
        internal_chain_id: i32,
        block_hash: String,
        data: Value,
        height: BlockHeight,
        time: OffsetDateTime,
    },

    #[serde(rename = "ethereum-decoded-log")]
    EthereumDecodedLog {
        internal_chain_id: i32,
        block_hash: String,
        height: BlockHeight,
        log_index: i32,
        timestamp: OffsetDateTime,
        transaction_hash: String,
        transaction_index: i32,
        transaction_log_index: i32,
        raw_log: Value,
        log_to_jsonb: Value,
    },

    #[serde(rename = "tendermint-block")]
    TendermintBlock {
        internal_chain_id: i32,
        hash: String,
        data: Value,
        height: BlockHeight,
        time: OffsetDateTime,
    },

    #[serde(rename = "tendermint-transaction")]
    TendermintTransaction {
        internal_chain_id: i32,
        block_hash: String,
        height: BlockHeight,
        hash: String,
        data: Value,
        index: i32,
    },

    #[serde(rename = "tendermint-event")]
    TendermintEvent {
        internal_chain_id: i32,
        block_hash: String,
        height: BlockHeight,
        transaction_hash: Option<String>,
        index: i32,
        transaction_index: Option<i32>,
        data: Value,
        time: OffsetDateTime,
        flow: String,
    },

    #[serde(rename = "channel-open-init")]
    ChannelOpenInit {
        #[serde(flatten)]
        inner: ChannelOpenInitEvent,
    },
    #[serde(rename = "channel-open-try")]
    ChannelOpenTry {
        #[serde(flatten)]
        inner: ChannelOpenTryEvent,
    },
    #[serde(rename = "channel-open-ack")]
    ChannelOpenAck {
        #[serde(flatten)]
        inner: ChannelOpenAckEvent,
    },
    #[serde(rename = "channel-open-confirm")]
    ChannelOpenConfirm {
        #[serde(flatten)]
        inner: ChannelOpenConfirmEvent,
    },
    #[serde(rename = "connection-open-init")]
    ConnectionOpenInit {
        #[serde(flatten)]
        inner: ConnectionOpenInitEvent,
    },
    #[serde(rename = "connection-open-try")]
    ConnectionOpenTry {
        #[serde(flatten)]
        inner: ConnectionOpenTryEvent,
    },
    #[serde(rename = "connection-open-ack")]
    ConnectionOpenAck {
        #[serde(flatten)]
        inner: ConnectionOpenAckEvent,
    },
    #[serde(rename = "connection-open-confirm")]
    ConnectionOpenConfirm {
        #[serde(flatten)]
        inner: ConnectionOpenConfirmEvent,
    },
    #[serde(rename = "create-client")]
    CreateClient {
        #[serde(flatten)]
        inner: CreateClientEvent,
    },
    #[serde(rename = "create-lens-client")]
    CreateLensClient {
        #[serde(flatten)]
        inner: CreateLensClientEvent,
    },
    #[serde(rename = "update-client")]
    UpdateClient {
        #[serde(flatten)]
        inner: UpdateClientEvent,
    },
    #[serde(rename = "packet-send")]
    PacketSend {
        #[serde(flatten)]
        inner: PacketSendEvent,
    },
    #[serde(rename = "packet-recv")]
    PacketRecv {
        #[serde(flatten)]
        inner: PacketRecvEvent,
    },
    #[serde(rename = "write-ack")]
    WriteAck {
        #[serde(flatten)]
        inner: WriteAckEvent,
    },
    #[serde(rename = "packet-ack")]
    PacketAck {
        #[serde(flatten)]
        inner: PacketAckEvent,
    },
    #[serde(rename = "packet-timeout")]
    PacketTimeout {
        #[serde(flatten)]
        inner: PacketTimeoutEvent,
    },
    #[serde(rename = "token-bucket-update")]
    TokenBucketUpdate {
        #[serde(flatten)]
        inner: TokenBucketUpdateEvent,
    },
    #[serde(rename = "wallet-mutation-entry")]
    WalletMutationEntry {
        #[serde(flatten)]
        inner: WalletMutationEntryEvent,
    },
}

impl SupportedBlockEvent {
    pub fn height(&self) -> BlockHeight {
        match self {
            SupportedBlockEvent::TendermintBlock { height, .. } => *height,
            SupportedBlockEvent::TendermintTransaction { height, .. } => *height,
            SupportedBlockEvent::TendermintEvent { height, .. } => *height,
            SupportedBlockEvent::EthereumLog { height, .. } => *height,
            SupportedBlockEvent::EthereumDecodedLog { height, .. } => *height,
            SupportedBlockEvent::ChannelOpenInit { inner, .. } => inner.header.height,
            SupportedBlockEvent::ChannelOpenTry { inner, .. } => inner.header.height,
            SupportedBlockEvent::ChannelOpenAck { inner, .. } => inner.header.height,
            SupportedBlockEvent::ChannelOpenConfirm { inner, .. } => inner.header.height,
            SupportedBlockEvent::ConnectionOpenInit { inner, .. } => inner.header.height,
            SupportedBlockEvent::ConnectionOpenTry { inner, .. } => inner.header.height,
            SupportedBlockEvent::ConnectionOpenAck { inner, .. } => inner.header.height,
            SupportedBlockEvent::ConnectionOpenConfirm { inner, .. } => inner.header.height,
            SupportedBlockEvent::CreateClient { inner, .. } => inner.header.height,
            SupportedBlockEvent::CreateLensClient { inner, .. } => inner.header.height,
            SupportedBlockEvent::UpdateClient { inner, .. } => inner.header.height,
            SupportedBlockEvent::PacketSend { inner, .. } => inner.header.height,
            SupportedBlockEvent::PacketRecv { inner, .. } => inner.header.height,
            SupportedBlockEvent::WriteAck { inner, .. } => inner.header.height,
            SupportedBlockEvent::PacketAck { inner, .. } => inner.header.height,
            SupportedBlockEvent::PacketTimeout { inner, .. } => inner.header.height,
            SupportedBlockEvent::TokenBucketUpdate { inner, .. } => inner.header.height,
            SupportedBlockEvent::WalletMutationEntry { inner, .. } => inner.header.height,
        }
    }
}
