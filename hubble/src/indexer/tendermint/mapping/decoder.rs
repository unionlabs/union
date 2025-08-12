use std::{collections::HashMap, fmt::Display};

use bytes::Bytes;
use cometbft_rpc::{rpc_types::TxResponse, types::abci::event::Event};
use itertools::Itertools;
use ruint::Uint;
use time::OffsetDateTime;
use tracing::trace;
use unionlabs::primitives::{encoding::HexUnprefixed, FixedBytes};

use crate::{
    indexer::{
        api::IndexerError,
        event::{
            header::Header,
            types::{
                Acknowledgement, BlockHash, BlockHeight, CanonicalChainId, Capacity, ChannelId,
                ChannelVersion, ClientId, ClientType, ConnectionId, ContractAddress, Denom, Maker,
                MakerMsg, MutationAmount, PacketData, PacketHash, Path, PortId, RefillRate,
                TimeoutTimestamp, TransactionHash, WalletAddress,
            },
        },
        handler::types::{CreateWrappedTokenKind, Metadata},
        tendermint::block_handle::BlockHeader,
    },
    postgres::ChainId,
};

pub struct TmEvent {
    pub name: String,
    pub attributes: HashMap<String, Vec<String>>,
}

impl Display for TmEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.keys_as_string())
    }
}

impl From<&Event> for TmEvent {
    fn from(value: &Event) -> Self {
        Self {
            name: value.ty.clone(),
            attributes: value
                .attributes
                .iter()
                .fold(HashMap::new(), |mut acc, attr| {
                    acc.entry(attr.key.clone())
                        .or_default()
                        .push(attr.value.clone());
                    acc
                }),
        }
    }
}

pub struct Decoder<'a> {
    pub chain_id: ChainId,
    pub block_header: &'a BlockHeader,
    pub transaction: &'a TxResponse,
    pub event: &'a TmEvent,
    pub event_index: usize,
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
            block_hash: self
                .block_header
                .block_id
                .hash
                .ok_or_else(|| {
                    IndexerError::CannotMapToEventDomainMissingKey(
                        self.event.name.clone(),
                        "block_hash".to_string(),
                        "block_hash".to_string(),
                    )
                })?
                .into(),
            height: self.block_header.header.height.inner().try_into()?,
            event_index: self.event_index.try_into()?,
            timestamp: OffsetDateTime::from_unix_timestamp_nanos(
                self.block_header.header.time.as_unix_nanos().into(),
            )
            .map_err(|_| {
                IndexerError::CannotMapToEventDomainOutOfRange(
                    self.event.name.clone(),
                    "timestamp".to_string(),
                    self.block_header.header.time.to_string(),
                    "i64".to_string(),
                )
            })?
            .into(),
            transaction_hash: self.transaction.hash.into(),
            transaction_index: self.transaction.index.into(),
            transaction_event_index: None,
        })
    }
}

impl TmEvent {
    pub fn client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("client_id")
    }

    pub fn l1_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("l1_client_id")
    }

    pub fn l2_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("l2_client_id")
    }

    pub fn client_type(&self) -> Result<ClientType, IndexerError> {
        self.get_client_type("client_type")
    }

    pub fn counterparty_chain_id(&self) -> Result<CanonicalChainId, IndexerError> {
        self.get_chain_id("counterparty_chain_id")
    }

    pub fn l2_chain_id(&self) -> Result<CanonicalChainId, IndexerError> {
        self.get_chain_id("l2_chain_id")
    }

    pub fn counterparty_height(&self) -> Result<BlockHeight, IndexerError> {
        self.get_height("counterparty_height")
    }

    pub fn timeout_height(&self) -> Result<BlockHeight, IndexerError> {
        self.get_height("packet_timeout_height")
    }

    pub fn timeout_timestamp(&self) -> Result<TimeoutTimestamp, IndexerError> {
        self.get_timestamp("packet_timeout_timestamp")
    }

    pub fn counterparty_client_id(&self) -> Result<ClientId, IndexerError> {
        self.get_client_id("counterparty_client_id")
    }

    pub fn connection_id(&self) -> Result<ConnectionId, IndexerError> {
        self.get_connection_id("connection_id")
    }

    pub fn counterparty_connection_id(&self) -> Result<ConnectionId, IndexerError> {
        self.get_connection_id("counterparty_connection_id")
    }

    pub fn channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("channel_id")
    }

    pub fn source_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("packet_source_channel_id")
    }

    pub fn destination_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("packet_destination_channel_id")
    }

    pub fn counterparty_channel_id(&self) -> Result<ChannelId, IndexerError> {
        self.get_channel_id("counterparty_channel_id")
    }

    pub fn port_id(&self) -> Result<PortId, IndexerError> {
        self.get_port_id("port_id")
    }

    pub fn counterparty_port_id(&self) -> Result<PortId, IndexerError> {
        self.get_counterparty_port_id("counterparty_port_id")
    }

    pub fn version(&self) -> Result<ChannelVersion, IndexerError> {
        self.get_version("version")
    }

    pub fn counterparty_version(&self) -> Result<ChannelVersion, IndexerError> {
        self.get_version("counterparty_version")
    }

    pub fn packet_hash(&self) -> Result<PacketHash, IndexerError> {
        self.get_packet_hash("packet_hash")
    }

    pub fn data(&self) -> Result<PacketData, IndexerError> {
        self.get_packet_data("packet_data")
    }

    pub fn denom(&self) -> Result<Denom, IndexerError> {
        self.get_denom("denom")
    }

    pub fn capacity(&self) -> Result<Capacity, IndexerError> {
        self.get_capacity("capacity")
    }

    pub fn refill_rate(&self) -> Result<RefillRate, IndexerError> {
        self.get_refill_rate("refill_rate")
    }

    pub fn acknowledgement(&self) -> Result<Acknowledgement, IndexerError> {
        self.get_acknowledgement("acknowledgement")
    }

    pub fn maker(&self) -> Result<Maker, IndexerError> {
        self.get_maker("maker")
    }

    pub fn maker_msg(&self) -> Result<MakerMsg, IndexerError> {
        self.get_maker_msg("maker_msg")
    }

    pub fn action(&self) -> Result<String, IndexerError> {
        self.get_action("action")
    }

    pub fn amount(&self) -> Result<MutationAmount, IndexerError> {
        self.get_mutation_amount("amount")
    }

    pub fn contract_address(&self) -> Result<ContractAddress, IndexerError> {
        self.get_contract_address("_contract_address")
    }

    pub fn path(&self) -> Result<Path, IndexerError> {
        self.get_path("path")
    }

    pub fn base_token(&self) -> Result<Denom, IndexerError> {
        self.get_denom("base_token")
    }

    pub fn quote_token(&self) -> Result<Denom, IndexerError> {
        self.get_denom("quote_token")
    }

    pub fn metadata(&self) -> Result<Metadata, IndexerError> {
        self.get_metadata("metadata")
    }

    pub fn create_wrapped_token_kind(&self) -> Result<CreateWrappedTokenKind, IndexerError> {
        self.get_create_wrapped_token_kind("kind")
    }

    #[allow(clippy::wrong_self_convention)] // 'from' is the from address, not from conversion
    pub fn from_opt(&self) -> Result<Option<WalletAddress>, IndexerError> {
        self.get_wallet_address_opt("from")
    }

    pub fn to_opt(&self) -> Result<Option<WalletAddress>, IndexerError> {
        self.get_wallet_address_opt("to")
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

    /// in cosmos, port id is bech32
    fn get_port_id(&self, key: &str) -> Result<PortId, IndexerError> {
        Ok(self.get_bytes_utf8(key, "port-id")?.into())
    }

    /// in cosmos, counterparty port id is hex
    fn get_counterparty_port_id(&self, key: &str) -> Result<PortId, IndexerError> {
        Ok(self.get_bytes(key, "counterparty-port-id")?.into())
    }

    fn get_version(&self, key: &str) -> Result<ChannelVersion, IndexerError> {
        Ok(self.get_string(key, "version")?.into())
    }

    fn get_packet_hash(&self, key: &str) -> Result<PacketHash, IndexerError> {
        Ok(self.get_bytes(key, "packet-hash")?.into())
    }

    fn get_packet_data(&self, key: &str) -> Result<PacketData, IndexerError> {
        Ok(self.get_bytes(key, "packet_data")?.into())
    }

    fn get_denom(&self, key: &str) -> Result<Denom, IndexerError> {
        Ok(self.get_bytes_utf8(key, "denom")?.into())
    }

    fn get_metadata(&self, key: &str) -> Result<Metadata, IndexerError> {
        Ok(self.get_bytes(key, "metadata")?.into())
    }

    fn get_create_wrapped_token_kind(
        &self,
        key: &str,
    ) -> Result<CreateWrappedTokenKind, IndexerError> {
        Ok(self.get_u8(key, "kind")?.into())
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
        Ok(self.get_bytes_utf8(key, "maker")?.into())
    }

    fn get_maker_msg(&self, key: &str) -> Result<MakerMsg, IndexerError> {
        Ok(self.get_bytes(key, "maker-msg")?.into())
    }

    fn get_action(&self, key: &str) -> Result<String, IndexerError> {
        self.get_string(key, "action")
    }

    fn get_mutation_amount(&self, key: &str) -> Result<MutationAmount, IndexerError> {
        Ok(self.get_u128(key, "wallet-amount")?.into())
    }

    fn get_contract_address(&self, key: &str) -> Result<ContractAddress, IndexerError> {
        Ok(self.get_bech32_decoded(key, "contract-address")?.into())
    }

    fn get_wallet_address_opt(&self, key: &str) -> Result<Option<WalletAddress>, IndexerError> {
        Ok(self
            .get_bech32_decoded_opt(key, "wallet-address")?
            .map(|x| x.into()))
    }

    fn get_path(&self, key: &str) -> Result<Path, IndexerError> {
        Ok(self.get_u256(key, "path")?.into())
    }

    fn get_u8(&self, key: &str, expecting: &str) -> Result<u8, IndexerError> {
        self.get_value(key, expecting).and_then(|value| {
            value
                .parse()
                .map_err(|_| self.report_unexpected_type(key, &value, expecting))
        })
    }

    fn get_u32(&self, key: &str, expecting: &str) -> Result<u32, IndexerError> {
        self.get_value(key, expecting).and_then(|value| {
            value
                .parse()
                .map_err(|_| self.report_unexpected_type(key, &value, expecting))
        })
    }

    fn get_u64(&self, key: &str, expecting: &str) -> Result<u64, IndexerError> {
        self.get_value(key, expecting).and_then(|value| {
            value
                .parse()
                .map_err(|_| self.report_unexpected_type(key, &value, expecting))
        })
    }

    fn get_u128(&self, key: &str, expecting: &str) -> Result<u128, IndexerError> {
        self.get_value(key, expecting).and_then(|value| {
            value
                .parse()
                .map_err(|_| self.report_unexpected_type(key, &value, expecting))
        })
    }

    fn get_bech32_decoded(&self, key: &str, expecting: &str) -> Result<bytes::Bytes, IndexerError> {
        self.get_value(key, expecting)
            .and_then(|bech32_encoded| self.decode_bech32(key, expecting, &bech32_encoded))
    }

    fn get_bech32_decoded_opt(
        &self,
        key: &str,
        expecting: &str,
    ) -> Result<Option<bytes::Bytes>, IndexerError> {
        Ok(match self.get_value_opt(key, expecting)? {
            Some(bech32_encoded) => Some(self.decode_bech32(key, expecting, &bech32_encoded)?),
            None => None,
        })
    }

    fn decode_bech32(
        &self,
        key: &str,
        expecting: &str,
        bech32_encoded: &str,
    ) -> Result<bytes::Bytes, IndexerError> {
        bech32::decode(bech32_encoded)
            .map(|(_, data)| Bytes::from(data))
            .map_err(|_| self.report_unexpected_type(key, bech32_encoded, expecting))
    }

    fn get_u256(&self, key: &str, expecting: &str) -> Result<Uint<256, 4>, IndexerError> {
        self.get_value(key, expecting).and_then(|value| {
            value
                .parse()
                .map_err(|_| self.report_unexpected_type(key, &value, expecting))
        })
    }

    fn get_bytes_utf8(&self, key: &str, expecting: &str) -> Result<Bytes, IndexerError> {
        Ok(Bytes::from(self.get_value(key, expecting)?))
    }

    fn get_bytes(&self, key: &str, expecting: &str) -> Result<Bytes, IndexerError> {
        let value_with_or_without_0x = self.get_value(key, expecting)?;

        let value_without_0x = value_with_or_without_0x
            .strip_prefix("0x")
            .unwrap_or(&value_with_or_without_0x);

        Ok(Bytes::from(hex::decode(value_without_0x).map_err(
            |_| self.report_unexpected_type(key, value_without_0x, expecting),
        )?))
    }

    fn get_string(&self, key: &str, expecting: &str) -> Result<String, IndexerError> {
        self.get_value(key, expecting)
    }

    fn get_value(&self, key: &str, expecting: &str) -> Result<String, IndexerError> {
        match self.get_value_opt(key, expecting)? {
            None => Err(self.report_missing_key(key, expecting)),
            Some(value) => Ok(value),
        }
    }

    fn get_value_opt(&self, key: &str, expecting: &str) -> Result<Option<String>, IndexerError> {
        match self.attributes.get(key) {
            None => Ok(None),
            Some(values) => match &values[..] {
                [one] => Ok(Some(replace_escape_chars(one))),
                _ => Err(self.report_multiple_keys(key, expecting)),
            },
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

    fn report_multiple_keys(&self, key: &str, expecting: &str) -> IndexerError {
        trace!(
            "report_multiple_keys - {}.{key} (expecting: {expecting}, keys: {})",
            self.name,
            self.keys_as_string(),
        );

        IndexerError::CannotMapToEventDomainMultipleKey(
            self.name.to_string(),
            key.to_string(),
            expecting.to_string(),
        )
    }

    fn report_unexpected_type(&self, key: &str, value: &str, expecting: &str) -> IndexerError {
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

    pub fn keys_as_string(&self) -> String {
        self.attributes.keys().sorted().join(", ")
    }
}

impl From<FixedBytes<32, HexUnprefixed>> for BlockHash {
    fn from(value: FixedBytes<32, HexUnprefixed>) -> Self {
        Bytes::copy_from_slice(value.iter().as_slice()).into()
    }
}

impl From<FixedBytes<32, HexUnprefixed>> for TransactionHash {
    fn from(value: FixedBytes<32, HexUnprefixed>) -> Self {
        Bytes::copy_from_slice(value.iter().as_slice()).into()
    }
}

fn replace_escape_chars(string: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};

    if string.contains('\u{0000}') {
        // encode if data contains zero-character
        general_purpose::STANDARD.encode(string)
    } else {
        string.to_string()
    }
}
