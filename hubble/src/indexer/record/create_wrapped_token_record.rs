use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_wrapped_token::CreateWrappedTokenEvent, types::BlockHeight},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue, PgValueExt,
    },
};

pub struct CreateWrappedTokenRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: Option<i64>,
    pub channel_id: i32,
    pub path: Vec<u8>,
    pub base_token: Vec<u8>,
    pub quote_token: Vec<u8>,
    pub metadata: Vec<u8>,
    pub kind: i32,
}
impl HasKind for CreateWrappedTokenRecord {
    fn kind() -> RecordKind {
        RecordKind::CreateWrappedToken
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, CreateWrappedTokenEvent>>
    for CreateWrappedTokenRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, CreateWrappedTokenEvent>,
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
            channel_id: value.event.channel_id.pg_value()?,
            path: value.event.path.pg_value()?,
            base_token: value.event.base_token.pg_value()?,
            quote_token: value.event.quote_token.pg_value()?,
            metadata: value.event.metadata.pg_value()?,
            kind: value.event.kind.pg_value()?,
        })
    }
}

impl CreateWrappedTokenRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.create_wrapped_token_sync (
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                channel_id,
                path,
                base_token,
                quote_token,
                metadata,
                kind
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.event_index,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.transaction_event_index,
            self.channel_id,
            &self.path[..],
            &self.base_token[..],
            &self.quote_token[..],
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
            DELETE FROM v2_sync.create_wrapped_token_sync
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
