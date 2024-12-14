use std::{collections::HashMap, fmt::Display};

use alloy::{
    eips::BlockId,
    network::AnyRpcBlock,
    primitives::{Address, BloomInput, FixedBytes},
    rpc::types::{BlockTransactionsKind, Filter, Log},
};
use axum::async_trait;
use color_eyre::eyre::Report;
use itertools::Itertools;
use time::OffsetDateTime;
use tokio::task::JoinSet;
use tracing::{debug, info, warn, Instrument};

use crate::{
    indexer::{
        api::{
            BlockHeight, BlockRange, BlockReference, BlockSelection, FetchMode, FetcherClient,
            IndexerError,
        },
        ethereum::{
            block_handle::{
                BlockDetails, BlockInsert, EthBlockHandle, EventInsert, TransactionInsert,
            },
            context::EthContext,
            create_client_tracker::schedule_create_client_checker,
            postgres::transaction_filter,
            provider::{Provider, RpcProviderId},
        },
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
};

pub trait ToLowerHex {
    fn to_lower_hex(&self) -> String;
}

impl ToLowerHex for FixedBytes<32> {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}

trait BlockReferenceProvider {
    fn block_reference(&self) -> Result<BlockReference, Report>;
}

impl BlockReferenceProvider for AnyRpcBlock {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        let timestamp = OffsetDateTime::from_unix_timestamp(self.header.timestamp.try_into().map_err(|_| {
            IndexerError::ProviderError("Invalid timestamp".into())
        })?)?;

        Ok(BlockReference {
            height: self.header.number,
            hash: self.header.hash.to_lower_hex(),
            timestamp,
        })
    }
}

#[derive(Clone)]
pub struct EthFetcherClient {
    pub chain_id: ChainId,
    pub provider: Provider,
    pub transaction_filter: TransactionFilter,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct TransactionFilter {
    pub address_filters: Vec<AddressFilter>,
}

impl TransactionFilter {
    pub(crate) fn addresses_at(&self, height: BlockHeight) -> Vec<Address> {
        self.address_filters
            .iter()
            .filter(|filter| filter.block_range.contains(height))
            .map(|filter| filter.address)
            .collect()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AddressFilter {
    pub block_range: BlockRange,
    pub address: Address,
}

impl Display for EthFetcherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain_id: {}", self.chain_id)
    }
}

impl EthFetcherClient {
    pub async fn fetch_single_with_provider(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<EthBlockHandle, IndexerError> {
        let block = self
            .provider
            .get_block(
                match selection {
                    BlockSelection::LastFinalized => BlockId::finalized(),
                    BlockSelection::Height(height) => BlockId::number(height),
                },
                BlockTransactionsKind::Full,
                provider_id,
            )
            .await;

        match block {
            Ok(Some(result)) => {
                let block = result.response;
                debug!(block_selection = ?selection, provider_id = ?result.provider_id, "Fetched block");

                Ok(EthBlockHandle {
                    reference: block.block_reference()?,
                    details: self.fetch_block_details(&block, mode, result.provider_id).await?,
                    eth_client: self.clone(),
                    provider_id: result.provider_id,
                })
            }
            Ok(None) => Err(IndexerError::NoBlock(selection)),
            Err(err) => {
                warn!(block_selection = ?selection, error = ?err, "Failed to fetch block");
                Err(err.into())
            }
        }
    }

    async fn fetch_block_details(
        &self,
        block: &AnyRpcBlock,
        mode: FetchMode,
        provider_id: RpcProviderId,
    ) -> Result<BlockDetails, IndexerError> {
        match mode {
            FetchMode::Lazy => Ok(BlockDetails::Lazy(block.clone())),
            FetchMode::Eager => Ok(BlockDetails::Eager(self.fetch_eager_details(block, provider_id).await?)),
        }
    }

    async fn fetch_eager_details(
        &self,
        block: &AnyRpcBlock
