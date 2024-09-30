use std::fmt::Display;

use aptos_rest_client::{aptos_api_types::Block, Transaction};
use axum::async_trait;
use color_eyre::Result;
use tokio::task::JoinSet;
use tracing::{debug, info, info_span, trace, Instrument};

use crate::{
    indexer::{
        api::{
            BlockHeight, BlockReferenceProvider, BlockSelection, FetchMode, FetcherClient,
            IndexerError,
        },
        aptos::{
            block_handle::{AptosBlockHandle, BlockDetails},
            context::AptosContext,
            create_client_tracker::schedule_create_client_checker,
            provider::{Provider, RpcProviderId},
        },
    },
    postgres::{fetch_or_insert_chain_id_tx, ChainId},
};

#[derive(Clone)]
pub struct AptosFetcherClient {
    pub chain_id: ChainId,
    pub provider: Provider,
    pub tx_search_max_page_size: u16,
}

impl Display for AptosFetcherClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "chain_id: {}", self.chain_id)
    }
}

impl AptosFetcherClient {
    pub async fn fetch_single_with_provider(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<AptosBlockHandle, IndexerError> {
        debug!("{}: fetching", selection);

        match selection {
            BlockSelection::LastFinalized => self.fetch_last_finalized(mode, provider_id).await,
            BlockSelection::Height(height) => self.fetch_at_height(mode, provider_id, height).await,
        }
    }

    async fn fetch_last_finalized(
        &self,
        mode: FetchMode,
        provider_id: Option<RpcProviderId>,
    ) -> Result<AptosBlockHandle, IndexerError> {
        trace!("fetch block height");

        let (provider_id, height) = self
            .provider
            .get_index(provider_id)
            .await
            .map(|result| (result.provider_id, result.response.inner().block_height))?;

        trace!(
            "current height: {height} using {:?} to fetch block",
            provider_id
        );

        self.fetch_at_height(mode, Some(provider_id), height.into())
            .await
    }

    async fn fetch_at_height(
        &self,
        mode: FetchMode,
        provider_id: Option<RpcProviderId>,
        height: BlockHeight,
    ) -> Result<AptosBlockHandle, IndexerError> {
        trace!("fetching block at height {height}");

        let result = self
            .provider
            .get_block_by_height(height, provider_id)
            .await?;
        let block = result.response.inner();

        trace!(
            "fetched block at height {height} using {:?}: {}-{}",
            provider_id,
            block.first_version,
            block.last_version
        );

        Ok(AptosBlockHandle {
            internal_chain_id: self.chain_id.db,
            reference: block.block_reference()?,
            details: match mode {
                FetchMode::Lazy => BlockDetails::Lazy(block.clone()),
                FetchMode::Eager => BlockDetails::Eager(
                    block.clone(),
                    self.fetch_transactions(block, result.provider_id).await?,
                ),
            },
            aptos_client: self.clone(),
            provider_id: result.provider_id,
        })
    }

    pub async fn fetch_transactions(
        &self,
        block: &Block,
        provider_id: RpcProviderId,
    ) -> Result<Vec<Transaction>, IndexerError> {
        trace!(
            "fetching transactions for block {} - versions: [{},{}]",
            block.block_height,
            block.first_version,
            block.last_version
        );

        let complete_start_inclusive: BlockHeight = block.first_version.into();
        let complete_end_inclusive: BlockHeight = block.last_version.into();

        let mut result =
            Vec::with_capacity((complete_end_inclusive + 1 - complete_start_inclusive) as usize);

        for chunk_start_inclusive in (complete_start_inclusive..=complete_end_inclusive)
            .step_by(self.tx_search_max_page_size as usize)
        {
            let chunk_end_exclusive = (chunk_start_inclusive + self.tx_search_max_page_size as u64)
                .min(complete_end_inclusive + 1); // +1, because end is inclusive

            let chunk_limit = (chunk_end_exclusive - chunk_start_inclusive) as u16;

            trace!(
                "fetching chunk for block {} - versions: [{},{}]",
                block.block_height,
                chunk_start_inclusive,
                chunk_end_exclusive - 1
            );

            let chunk_transactions = self
                .provider
                .get_transactions(chunk_start_inclusive, chunk_limit, Some(provider_id))
                .await?
                .response
                .inner()
                .clone();

            result.extend(chunk_transactions);
        }

        trace!(
            "fetched transactions for block {} - versions: [{},{}] - transactions: {}",
            block.block_height,
            block.first_version,
            block.last_version,
            result.len()
        );

        Ok(result)
    }
}

#[async_trait]
impl FetcherClient for AptosFetcherClient {
    type BlockHandle = AptosBlockHandle;
    type Context = AptosContext;

    async fn create(
        pg_pool: sqlx::PgPool,
        join_set: &mut JoinSet<Result<(), IndexerError>>,
        context: AptosContext,
    ) -> Result<Self, IndexerError> {
        let provider = Provider::new(context.rpc_urls);

        info!("fetching chain-id from node");
        let chain_id = provider
            .get_index(None)
            .await
            .inspect_err(|e| debug!(?e, "error fetching chain-id: {}", e))?
            .response
            .inner()
            .chain_id
            .to_string();

        info!("fetched chain-id from node: {}", chain_id);

        let indexing_span = info_span!("indexer", chain_id = chain_id).or_current();
        async move {
            let mut tx = pg_pool.begin().await?;

            let chain_id = fetch_or_insert_chain_id_tx(&mut tx, chain_id.to_string())
                .await?
                .get_inner_logged();

            tx.commit().await?;

            schedule_create_client_checker(pg_pool, join_set, provider.clone(), chain_id.db);

            Ok(AptosFetcherClient {
                chain_id,
                provider,
                tx_search_max_page_size: context.tx_search_max_page_size,
            })
        }
        .instrument(indexing_span)
        .await
    }

    async fn fetch_single(
        &self,
        selection: BlockSelection,
        mode: FetchMode,
    ) -> Result<Self::BlockHandle, IndexerError> {
        self.fetch_single_with_provider(selection, mode, None).await
    }
}
