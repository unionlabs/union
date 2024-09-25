use std::convert::Into;

use aptos_rest_client::{aptos_api_types::Block, Transaction};
use axum::async_trait;
use color_eyre::eyre::Report;
use futures::{stream::FuturesOrdered, Stream};
use sqlx::Postgres;
use time::OffsetDateTime;
use tracing::debug;

use super::{fetcher_client::AptosFetcherClient, provider::RpcProviderId};
use crate::indexer::api::{
            BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, BlockSelection, FetchMode, IndexerError
        };

impl BlockReferenceProvider for Block {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.block_height.into(),
            hash: self.block_hash.to_string(),
            timestamp: OffsetDateTime::from_unix_timestamp_nanos((self.block_timestamp.0 as i128) * 1000).map_err(Report::from)?
        })
    }
}

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Block),
    Eager(Block, Vec<Transaction>),
}

#[derive(Clone)]
pub struct AptosBlockHandle {
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub aptos_client: AptosFetcherClient,
    pub provider_id: RpcProviderId,
}

#[async_trait]
impl BlockHandle for AptosBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError> {
        debug!("{}: fetching", block_range);

        Ok(FuturesOrdered::from_iter(
            block_range.clone().into_iter().map(|height| async move {
                self.aptos_client
                    .fetch_single_with_provider(
                        BlockSelection::Height(height),
                        fetch_mode,
                        Some(self.provider_id),
                    )
                    .await
            }),
        ))
    }

    async fn insert(&self, _tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        // TODO

        debug!("{}: done", reference);
        Ok(())
    }

    async fn update(&self, _tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        // TODO

        debug!("{}: done", reference);
        Ok(())
    }
}
