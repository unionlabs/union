use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        create_wrapped_token_record::CreateWrappedTokenRecord,
        InternalChainId, PgValue,
    },
};

pub struct CreateWrappedTokenRelationRecord {
    pub internal_wrapped_chain_id: i32,
    pub internal_unwrapped_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: Option<i64>,
    pub destination_channel_id: i32,
    pub path: BigDecimal,
    pub unwrapped_token: Vec<u8>,
    pub wrapped_token: Vec<u8>,
    pub metadata: Vec<u8>,
    pub kind: i32,
}

impl HasKind for CreateWrappedTokenRelationRecord {
    fn kind() -> RecordKind {
        RecordKind::CreateWrappedTokenRelation
    }
}

impl TryFrom<(&CreateWrappedTokenRecord, &InternalChainId)> for CreateWrappedTokenRelationRecord {
    type Error = IndexerError;

    fn try_from(
        (record, internal_unwrapped_chain_id): (&CreateWrappedTokenRecord, &InternalChainId),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_wrapped_chain_id: record.internal_chain_id,
            internal_unwrapped_chain_id: internal_unwrapped_chain_id.0,
            block_hash: record.block_hash.clone(),
            height: record.height,
            event_index: record.event_index,
            timestamp: record.timestamp,
            transaction_hash: record.transaction_hash.clone(),
            transaction_index: record.transaction_index,
            transaction_event_index: record.transaction_event_index,
            destination_channel_id: record.channel_id,
            path: record.path.clone(),
            unwrapped_token: record.base_token.clone(),
            wrapped_token: record.quote_token.clone(),
            metadata: record.metadata.clone(),
            kind: record.kind,
        })
    }
}

impl CreateWrappedTokenRelationRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.create_wrapped_token_relation_sync (
                internal_wrapped_chain_id,
                internal_unwrapped_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                destination_channel_id,
                path,
                unwrapped_token,
                wrapped_token,
                metadata,
                kind
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            self.internal_wrapped_chain_id,
            self.internal_unwrapped_chain_id,
            &self.block_hash[..],
            self.height,
            self.event_index,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.transaction_event_index,
            self.destination_channel_id,
            self.path,
            &self.unwrapped_token[..],
            &self.wrapped_token[..],
            &self.metadata[..],
            self.kind
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
            DELETE FROM v2_sync.create_wrapped_token_relation_sync
            WHERE internal_wrapped_chain_id = $1 AND height = $2
            "#,
            internal_chain_id.pg_value()?,
            height.pg_value()?
        )
        .execute(&mut **tx)
        .await?;

        Ok(Changes::with_deletes::<Self>(result.rows_affected()))
    }
}
