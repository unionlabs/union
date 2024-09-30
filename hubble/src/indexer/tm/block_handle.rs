use axum::async_trait;
use color_eyre::eyre::Report;
use futures::Stream;
use sqlx::Postgres;
use tendermint::block::Meta;
use tracing::debug;

use crate::{
    indexer::{
        api::{
            BlockHandle, BlockRange, BlockReference, BlockReferenceProvider, FetchMode,
            IndexerError,
        },
        tm::{
            fetcher_client::TmFetcherClient, postgres::delete_tm_block_transactions_events,
            provider::RpcProviderId,
        },
    },
    postgres,
    tm::{PgBlock, PgEvent, PgTransaction},
};

#[derive(Clone)]
pub struct BlockHeader {
    pub block_id: tendermint::block::Id,
    pub header: tendermint::block::Header,
}

impl From<tendermint_rpc::endpoint::block::Response> for BlockHeader {
    fn from(response: tendermint_rpc::endpoint::block::Response) -> Self {
        BlockHeader {
            block_id: response.block_id,
            header: response.block.header,
        }
    }
}

impl From<tendermint::block::Meta> for BlockHeader {
    fn from(meta: tendermint::block::Meta) -> Self {
        BlockHeader {
            block_id: meta.block_id,
            header: meta.header,
        }
    }
}

impl From<tendermint_rpc::endpoint::commit::Response> for BlockHeader {
    fn from(response: tendermint_rpc::endpoint::commit::Response) -> Self {
        BlockHeader {
            block_id: response.signed_header.commit.block_id,
            header: response.signed_header.header,
        }
    }
}

impl BlockReferenceProvider for BlockHeader {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.into(),
            hash: self.block_id.hash.to_string(),
            timestamp: self.header.time.into(),
        })
    }
}

impl BlockReferenceProvider for Meta {
    fn block_reference(&self) -> Result<BlockReference, Report> {
        Ok(BlockReference {
            height: self.header.height.into(),
            hash: self.block_id.hash.to_string(),
            timestamp: self.header.time.into(),
        })
    }
}

#[derive(Clone)]
pub enum BlockDetails {
    Lazy(Box<BlockHeader>),
    Eager(PgBlock, Vec<PgTransaction>, Vec<PgEvent>),
}

#[derive(Clone)]
pub struct TmBlockHandle {
    pub reference: BlockReference,
    pub details: BlockDetails,
    pub tm_client: TmFetcherClient,
    pub provider_id: RpcProviderId,
}

impl TmBlockHandle {
    async fn get_block_insert(
        &self,
    ) -> Result<(PgBlock, Vec<PgTransaction>, Vec<PgEvent>), Report> {
        Ok(match self.details.clone() {
            BlockDetails::Eager(block, transactions, events) => (block, transactions, events),
            BlockDetails::Lazy(block_header) => {
                self.tm_client
                    .fetch_details(&block_header, self.provider_id)
                    .await?
            }
        })
    }
}

#[async_trait]
impl BlockHandle for TmBlockHandle {
    fn reference(&self) -> BlockReference {
        self.reference.clone()
    }

    fn fetch_range(
        &self,
        block_range: BlockRange,
        fetch_mode: FetchMode,
    ) -> Result<impl Stream<Item = Result<Self, IndexerError>> + Send, IndexerError> {
        debug!("{}: fetching", block_range);

        self.tm_client
            .fetch_range_with_provider(block_range, fetch_mode, Some(self.provider_id))
    }

    async fn insert(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        let (block, transactions, events) = self.get_block_insert().await?;

        let mode = postgres::InsertMode::Insert;
        postgres::insert_batch_blocks(tx, vec![block], mode).await?;
        postgres::insert_batch_transactions(tx, transactions, mode).await?;
        postgres::insert_batch_events(tx, events, mode).await?;

        debug!("{}: done", reference);
        Ok(())
    }

    async fn update(&self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        let reference = self.reference();
        debug!("{}: updating", reference);

        delete_tm_block_transactions_events(tx, self.tm_client.chain_id.db, self.reference.height)
            .await?;
        self.insert(tx).await?;

        debug!("{}: done", reference);
        Ok(())
    }
}
