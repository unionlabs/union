use std::{collections::HashMap, fmt::Display};

use alloy::{dyn_abi::DynSolValue, network::AnyRpcBlock, primitives::FixedBytes, rpc::types::Log};
use bytes::Bytes;
use itertools::Itertools;
use ruint::aliases::U256;
use time::OffsetDateTime;
use tracing::trace;

use crate::{
    indexer::{
        api::IndexerError,
        ethereum::abi::SolEvent,
        event::{
            header::Header,
            types::{
                Acknowledgement, BlockHash, BlockHeight, CanonicalChainId, Capacity, ChannelId,
                ChannelVersion, ClientId, ClientType, ConnectionId, Denom, Maker, MakerMsg,
                PacketData, PacketHash, Path, PortId, RefillRate, TimeoutTimestamp,
                TransactionHash,
            },
        },
        handler::types::{CreateWrappedTokenKind, Metadata},
    },
    postgres::ChainId,
};

pub struct Decoder<'a> {
    pub chain_id: ChainId,
    pub block: &'a AnyRpcBlock,
    pub log: &'a Log,
    pub transaction_log_index: usize,
    pub event: &'a SolEvent,
}

impl<'a> Display for Decoder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.event.name, self.event.keys_as_string())
    }
}

impl<'a> Decoder<'a> {
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
            transaction_event_index: Some(self.transaction_log_index.try_into()?),
        })
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

impl SolEvent {
    pub fn client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("clientId")
    }

    pub fn l1_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("l1ClientId")
    }

    pub fn l2_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("l2ClientId")
    }

    pub fn client_type(&self) -> Result<ClientType, IndexerError> {
        self.get_client_type("clientType")
    }

    pub fn counterparty_chain_id(&self) -> Result<CanonicalChainId, IndexerError> {
        self.get_chain_id("counterpartyChainId")
    }

    pub fn l2_chain_id(&self) -> Result<CanonicalChainId, IndexerError> {
        self.get_chain_id("l2ChainId")
    }

    pub fn counterparty_height(&self) -> Result<BlockHeight, IndexerError> {
        self.get_height("height")
    }

    pub fn timeout_height(&self) -> Result<BlockHeight, IndexerError> {
        self.get_height("timeoutHeight")
    }

    pub fn timeout_timestamp(&self) -> Result<TimeoutTimestamp, IndexerError> {
        self.get_timestamp("timeoutTimestamp")
    }

    pub fn counterparty_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("counterpartyClientId")
    }

    pub fn connection_id(&self) -> Result<ConnectionId, IndexerError> {
        self.get_connection_id("connectionId")
    }

    pub fn counterparty_connection_id(&self) -> Result<ConnectionId, IndexerError> {
        self.get_connection_id("counterpartyConnectionId")
    }

    pub fn channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("channelId")
    }

    pub fn source_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("sourceChannelId")
    }

    pub fn destination_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("destinationChannelId")
    }

    pub fn counterparty_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("counterpartyChannelId")
    }

    pub fn port_id(&self) -> Result<PortId, IndexerError> {
        self.get_port_id("portId")
    }

    pub fn counterparty_port_id(&self) -> Result<PortId, IndexerError> {
        self.get_port_id("counterpartyPortId")
    }

    pub fn version(&self) -> Result<ChannelVersion, IndexerError> {
        self.get_version("version")
    }

    pub fn counterparty_version(&self) -> Result<ChannelVersion, IndexerError> {
        self.get_version("counterpartyVersion")
    }

    pub fn packet_hash(&self) -> Result<PacketHash, IndexerError> {
        self.get_packet_hash("packetHash")
    }

    pub fn data(&self) -> Result<PacketData, IndexerError> {
        self.get_packet_data("data")
    }

    pub fn denom(&self) -> Result<Denom, IndexerError> {
        self.get_denom("token")
    }

    pub fn capacity(&self) -> Result<Capacity, IndexerError> {
        self.get_capacity("capacity")
    }

    pub fn refill_rate(&self) -> Result<RefillRate, IndexerError> {
        self.get_refill_rate("refillRate")
    }

    pub fn acknowledgement(&self) -> Result<Acknowledgement, IndexerError> {
        self.get_acknowledgement("acknowledgement")
    }

    pub fn maker(&self) -> Result<Maker, IndexerError> {
        self.get_maker("maker")
    }

    pub fn maker_msg(&self) -> Result<MakerMsg, IndexerError> {
        self.get_maker_msg("makerMsg")
    }

    pub fn packet(&self) -> Result<SolEvent, IndexerError> {
        self.get_packet("packet")
    }

    pub fn path(&self) -> Result<Path, IndexerError> {
        self.get_path("path")
    }

    pub fn base_token(&self) -> Result<Denom, IndexerError> {
        self.get_denom("baseToken")
    }

    pub fn quote_token(&self) -> Result<Denom, IndexerError> {
        self.get_denom("quoteToken")
    }

    pub fn metadata(&self) -> Result<Metadata, IndexerError> {
        self.get_metadata("metadata")
    }

    pub fn create_wrapped_token_kind(&self) -> Result<CreateWrappedTokenKind, IndexerError> {
        self.get_create_wrapped_token_kind("kind")
    }

    fn get_height(&self, key: &str) -> Result<BlockHeight, IndexerError> {
        Ok(self.get_u64(key, "height")?.into())
    }

    fn get_timestamp(&self, key: &str) -> Result<TimeoutTimestamp, IndexerError> {
        Ok(self.get_u64(key, "timestamp")?.into())
    }

    fn get_client_id(&self, key: &str) -> Result<ClientId, IndexerError> {
        Ok(self.get_u32(key, "client-id")?.into())
    }

    fn get_client_type(&self, key: &str) -> Result<ClientType, IndexerError> {
        Ok(self.get_string(key, "client-type")?.into())
    }

    fn get_chain_id(&self, key: &str) -> Result<CanonicalChainId, IndexerError> {
        Ok(self.get_string(key, "chain-id")?.into())
    }

    fn get_connection_id(&self, key: &str) -> Result<ConnectionId, IndexerError> {
        Ok(self.get_u32(key, "connection-id")?.into())
    }

    fn get_channel_id(&self, key: &str) -> Result<ChannelId, IndexerError> {
        Ok(self.get_u32(key, "channel-id")?.into())
    }

    fn get_port_id(&self, key: &str) -> Result<PortId, IndexerError> {
        Ok(self.get_bytes(key, "port-id")?.into())
    }

    fn get_version(&self, key: &str) -> Result<ChannelVersion, IndexerError> {
        Ok(self.get_string(key, "version")?.into())
    }

    fn get_packet_hash(&self, key: &str) -> Result<PacketHash, IndexerError> {
        Ok(self.get_bytes(key, "packet-hash")?.into())
    }

    fn get_packet_data(&self, key: &str) -> Result<PacketData, IndexerError> {
        Ok(self.get_bytes(key, "packet-data")?.into())
    }

    fn get_denom(&self, key: &str) -> Result<Denom, IndexerError> {
        Ok(self.get_bytes(key, "denom")?.into())
    }

    fn get_metadata(&self, key: &str) -> Result<Metadata, IndexerError> {
        Ok(self.get_bytes(key, "metadata")?.into())
    }

    fn get_create_wrapped_token_kind(
        &self,
        key: &str,
    ) -> Result<CreateWrappedTokenKind, IndexerError> {
        Ok(self.get_u8(key, "create-wrapped-token-kind")?.into())
    }

    fn get_capacity(&self, key: &str) -> Result<Capacity, IndexerError> {
        Ok(self.get_u256(key, "capacity")?.into())
    }

    fn get_refill_rate(&self, key: &str) -> Result<RefillRate, IndexerError> {
        Ok(self.get_u256(key, "refill-rate")?.into())
    }

    fn get_acknowledgement(&self, key: &str) -> Result<Acknowledgement, IndexerError> {
        Ok(self.get_bytes(key, "acknowledgement")?.into())
    }

    fn get_maker(&self, key: &str) -> Result<Maker, IndexerError> {
        Ok(self.get_bytes(key, "maker")?.into())
    }

    fn get_maker_msg(&self, key: &str) -> Result<MakerMsg, IndexerError> {
        Ok(self.get_bytes(key, "maker-msg")?.into())
    }

    pub fn get_path(&self, key: &str) -> Result<Path, IndexerError> {
        Ok(self.get_bytes(key, "path")?.into())
    }

    pub fn get_packet(&self, key: &str) -> Result<SolEvent, IndexerError> {
        self.get_event(key, "packet")
    }

    fn get_u8(&self, key: &str, expecting: &str) -> Result<u8, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Uint(v, 8) => Ok(v.to::<u8>()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_u32(&self, key: &str, expecting: &str) -> Result<u32, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Uint(v, 32) => Ok(v.to::<u32>()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_u64(&self, key: &str, expecting: &str) -> Result<u64, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Uint(v, 64) => Ok(v.to::<u64>()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_u256(&self, key: &str, expecting: &str) -> Result<U256, IndexerError> {
        match self.get_value(key, expecting)? {
            #[allow(clippy::useless_conversion)] // DynSolValue::Uint != ruint::Uint
            DynSolValue::Uint(value, 256) => Ok((*value).into()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_bytes(&self, key: &str, expecting: &str) -> Result<Bytes, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::Address(address) => Ok(Bytes::copy_from_slice(address.as_slice())),
            DynSolValue::Bytes(bytes) => Ok(Bytes::copy_from_slice(bytes.as_slice())),
            DynSolValue::FixedBytes(bytes, ..) => Ok(Bytes::copy_from_slice(bytes.as_slice())),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_string(&self, key: &str, expecting: &str) -> Result<String, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::String(string) => Ok(string.clone()),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn get_value(&self, key: &str, expecting: &str) -> Result<&DynSolValue, IndexerError> {
        self.attributes
            .get(key)
            .ok_or_else(|| self.report_missing_key(key, expecting))
    }

    fn get_event(&self, key: &str, expecting: &str) -> Result<SolEvent, IndexerError> {
        match self.get_value(key, expecting)? {
            DynSolValue::CustomStruct {
                name,
                prop_names,
                tuple,
            } => Ok(SolEvent {
                name: name.to_string(),
                attributes: prop_names
                    .iter()
                    .zip(tuple)
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect::<HashMap<String, DynSolValue>>(),
            }),
            value => Err(self.report_unexpected_type(key, value, expecting)),
        }
    }

    fn report_missing_key(&self, key: &str, expecting: &str) -> IndexerError {
        trace!(
            "report_missing_key - {}.{key} (expecting: {expecting}, keys: {})",
            self.name,
            self.keys_as_string(),
        );

        IndexerError::CannotMapToEventDomainMissingKey(
            self.name.to_string(),
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
            self.name,
            self.keys_as_string(),
        );

        IndexerError::CannotMapToEventDomainUnexpectedType(
            self.name.to_string(),
            key.to_string(),
            format!("{value:?}"),
            expecting.to_string(),
        )
    }

    // fn report_out_of_range(&self, key: &str, value: &DynSolValue, expecting: &str) -> IndexerError {
    //     trace!(
    //         "report_out_of_range - {}.{key} {value:?} (expecting: {expecting}, keys: {})",
    //         self.name,
    //         self.keys_as_string(),
    //     );

    //     IndexerError::CannotMapToEventDomainOutOfRange(
    //         self.event.name.to_string(),
    //         key.to_string(),
    //         format!("{value:?}"),
    //         expecting.to_string(),
    //     )
    // }

    pub fn keys_as_string(&self) -> String {
        self.attributes.keys().sorted().join(", ")
    }
}
