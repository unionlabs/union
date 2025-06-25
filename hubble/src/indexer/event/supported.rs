use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::indexer::event::{
    channel_open_ack_event::ChannelOpenAckEvent,
    channel_open_confirm_event::ChannelOpenConfirmEvent,
    channel_open_init_event::ChannelOpenInitEvent, channel_open_try_event::ChannelOpenTryEvent,
    types::BlockHeight,
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
        }
    }
}
