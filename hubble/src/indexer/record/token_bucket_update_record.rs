use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{token_bucket_update_event::TokenBucketUpdateEvent, types::BlockHeight},
    handler::EventContext,
    record::{ChainContext, InternalChainId},
};

pub struct TokenBucketUpdateRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i32,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: i64,
    pub denom: Vec<u8>,
    pub capacity: BigDecimal,
    pub refill_rate: BigDecimal,
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, TokenBucketUpdateEvent>>
    for TokenBucketUpdateRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, TokenBucketUpdateEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value_integer()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value_bigint()?,
            event_index: value.event.header.event_index.pg_value_integer()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            transaction_event_index: value.event.header.transaction_event_index.pg_value()?,
            denom: value.event.denom.pg_value()?,
            capacity: value.event.capacity.pg_value()?,
            refill_rate: value.event.refill_rate.pg_value()?,
        })
    }
}

impl TokenBucketUpdateRecord {
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.token_bucket_update_test (
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                denom,
                capacity,
                refill_rate
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.event_index,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.transaction_event_index,
            &self.denom[..],
            self.capacity,
            self.refill_rate,
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn delete_by_chain_and_height(
        tx: &mut Transaction<'_, Postgres>,
        internal_chain_id: InternalChainId,
        height: BlockHeight,
    ) -> Result<u64, IndexerError> {
        trace!("delete_by_chain_and_height({internal_chain_id}, {height})");

        let result = sqlx::query!(
            r#"
            DELETE FROM v2_sync.token_bucket_update_test
            WHERE internal_chain_id = $1 AND height = $2
            "#,
            internal_chain_id.pg_value_integer()?,
            height.pg_value_bigint()?
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }
}
