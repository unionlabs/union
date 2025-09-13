use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{types::BlockHeight, unbond_event::UnbondEvent},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue, PgValueExt,
    },
};

pub struct UnbondRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: Option<i64>,
    pub message_index: Option<i64>,
    pub amount: BigDecimal,
    pub batch: i64,
    pub is_new_request: bool,
    pub staker: Vec<u8>,
}
impl HasKind for UnbondRecord {
    fn kind() -> RecordKind {
        RecordKind::Unbond
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, UnbondEvent>> for UnbondRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, UnbondEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value()?,
            event_index: value.event.header.event_index.pg_value()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            transaction_event_index: value.event.header.transaction_event_index.pg_value()?,
            message_index: value.event.header.message_index.pg_value()?,
            amount: value.event.amount.pg_value()?,
            batch: value.event.batch.pg_value()?,
            is_new_request: value.event.is_new_request.pg_value()?,
            staker: value.event.staker.pg_value()?,
        })
    }
}

impl UnbondRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.unbond_sync (
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                message_index,
                amount,
                batch,
                is_new_request,
                staker
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.event_index,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.transaction_event_index,
            self.message_index,
            self.amount,
            self.batch,
            self.is_new_request,
            &self.staker[..],
        )
        .execute(&mut **tx)
        .await?;

        Ok(Changes::with_single_insert::<Self>())
    }

    pub async fn delete_by_chain_and_height(
        tx: &mut Transaction<'_, Postgres>,
        internal_chain_id: InternalChainId,
        height: BlockHeight,
    ) -> Result<Changes, IndexerError> {
        trace!("delete_by_chain_and_height({internal_chain_id}, {height})");

        let result = sqlx::query!(
            r#"
            DELETE FROM v2_sync.unbond_sync
            WHERE internal_chain_id = $1 AND height = $2
            "#,
            internal_chain_id.pg_value()?,
            height.pg_value()?
        )
        .execute(&mut **tx)
        .await?;

        Ok(Changes::with_deletes::<Self>(result.rows_affected()))
    }
}
