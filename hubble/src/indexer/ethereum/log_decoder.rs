use std::fmt::Display;

use alloy::{dyn_abi::DynSolValue, network::AnyRpcBlock, primitives::FixedBytes, rpc::types::Log};
use bytes::Bytes;
use itertools::Itertools;
use time::OffsetDateTime;
use tracing::trace;

use crate::{
    indexer::{
        api::IndexerError,
        ethereum::abi::SolEvent,
        event::{
            header::Header,
            types::{BlockHash, ChannelId, ConnectionId, PortId, TransactionHash, Version},
        },
    },
    postgres::ChainId,
};

pub struct LogDecoder<'a> {
    pub chain_id: ChainId,
    pub block: &'a AnyRpcBlock,
    pub log: &'a Log,
    pub transaction_log_index: usize,
    pub event: &'a SolEvent,
}

impl<'a> Display for LogDecoder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.event.name, self.keys_as_string())
    }
}

impl<'a> LogDecoder<'a> {
    pub fn header(&'a self) -> Result<Header, IndexerError> {
        Ok(Header {
            universal_chain_id: self.chain_id.universal_chain_id.to_string().into(),
            block_hash: self.block.header.hash.into(),
            height: self.block.header.number.into(),
            event_index: self
                .log
                .log_index
                .ok_or_else(|| {
                    IndexerError::CannotMapToEventDomainMissingKey(
                        self.event.name.to_string(),
                        "log_index".to_string(),
                        "log_index".to_string(),
                    )
                })?
                .into(),
            timestamp: OffsetDateTime::from_unix_timestamp(
                self.block.header.timestamp.try_into().map_err(|_| {
                    IndexerError::CannotMapToEventDomainOutOfRange(
                        self.event.name.to_string(),
                        "timestamp".to_string(),
                        self.block.header.timestamp.to_string(),
                        "i64".to_string(),
                    )
                })?,
            )
            .map_err(|_| -> IndexerError {
                IndexerError::CannotMapToEventDomainOutOfRange(
                    self.event.name.to_string(),
                    "timestamp".to_string(),
                    self.block.header.timestamp.to_string(),
                    "unix timestamp".to_string(),
                )
            })?
            .into(),
            transaction_hash: self
                .log
                .transaction_hash
                .ok_or_else(|| {
                    IndexerError::CannotMapToEventDomainMissingKey(
                        self.event.name.to_string(),
                        "transaction_hash".to_string(),
                        "transaction_hash".to_string(),
                    )
                })?
                .into(),
            transaction_index: self
                .log
                .transaction_index
                .ok_or_else(|| {
                    IndexerError::CannotMapToEventDomainMissingKey(
                        self.event.name.to_string(),
                        "transaction_index".to_string(),
                        "transaction_index".to_string(),
                    )
                })?
                .into(),
            transaction_event_index: self.transaction_log_index.try_into()?,
        })
    }

    pub fn connection_id(&'a self) -> Result<ConnectionId, IndexerError> {
        self.get_connection_id("connectionId")
    }

    pub fn channel_id(&'a self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("channelId")
    }

    pub fn counterparty_channel_id(&'a self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("counterpartyChannelId")
    }

    pub fn port_id(&'a self) -> Result<PortId, IndexerError> {
        self.get_port_id("portId")
    }

    pub fn counterparty_port_id(&'a self) -> Result<PortId, IndexerError> {
        self.get_port_id("counterpartyPortId")
    }

    pub fn version(&'a self) -> Result<Version, IndexerError> {
        self.get_version("version")
    }

    pub fn counterparty_version(&'a self) -> Result<Version, IndexerError> {
        self.get_version("counterpartyVersion")
    }

    fn get_connection_id(&'a self, key: &str) -> Result<ConnectionId, IndexerError> {
        Ok(self.get_u32(key, "connection-id")?.into())
    }

    fn get_channel_id(&'a self, key: &str) -> Result<ChannelId, IndexerError> {
        Ok(self.get_u32(key, "channel-id")?.into())
    }

    fn get_port_id(&'a self, key: &str) -> Result<PortId, IndexerError> {
        Ok(self.get_bytes(key, "port-id")?.into())
    }

    fn get_version(&'a self, key: &str) -> Result<Version, IndexerError> {
        Ok(self.get_string(key, "version")?.into())
    }

    fn get_u32(&'a self, key: &str, expecting: &str) -> Result<u32, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Uint(v, 32) => Ok(v.to::<u32>()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_bytes(&'a self, key: &str, expecting: &str) -> Result<Bytes, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Address(address) => Ok(Bytes::copy_from_slice(address.as_slice())),
            DynSolValue::Bytes(bytes) => Ok(Bytes::copy_from_slice(bytes.as_slice())),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_string(&'a self, key: &str, expecting: &str) -> Result<String, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::String(string) => Ok(string.clone()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_value(&'a self, key: &str, expecting: &str) -> Result<&'a DynSolValue, IndexerError> {
        self.event
            .attributes
            .get(key)
            .ok_or(self.report_missing_key(key, expecting))
    }

    fn report_missing_key(&self, key: &str, expecting: &str) -> IndexerError {
        trace!(
            "report_missing_key - {}.{key} (expecting: {expecting}, keys: {})",
            self.event.name,
            self.keys_as_string(),
        );

        IndexerError::CannotMapToEventDomainMissingKey(
            self.event.name.to_string(),
            key.to_string(),
            expecting.to_string(),
        )
    }

    fn report_unexpected_type(
        &self,
        key: &str,
        value: &DynSolValue,
        expecting: &str,
    ) -> IndexerError {
        trace!(
            "report_unexpected_type - {}.{key} {value:?} (expecting: {expecting}, keys: {})",
            self.event.name,
            self.keys_as_string(),
        );

        IndexerError::CannotMapToEventDomainUnexpectedType(
            self.event.name.to_string(),
            key.to_string(),
            format!("{value:?}"),
            expecting.to_string(),
        )
    }

    // fn report_out_of_range(&self, key: &str, value: &DynSolValue, expecting: &str) -> IndexerError {
    //     trace!(
    //         "report_out_of_range - {}.{key} {value:?} (expecting: {expecting}, keys: {})",
    //         self.event.name,
    //         self.keys_as_string(),
    //     );

    //     IndexerError::CannotMapToEventDomainOutOfRange(
    //         self.event.name.to_string(),
    //         key.to_string(),
    //         format!("{value:?}"),
    //         expecting.to_string(),
    //     )
    // }

    fn keys_as_string(&self) -> String {
        self.event.attributes.keys().sorted().join(", ")
    }
}

impl From<FixedBytes<32>> for BlockHash {
    fn from(value: FixedBytes<32>) -> Self {
        Bytes::copy_from_slice(value.as_slice()).into()
    }
}

impl From<FixedBytes<32>> for TransactionHash {
    fn from(value: FixedBytes<32>) -> Self {
        Bytes::copy_from_slice(value.as_slice()).into()
    }
}
