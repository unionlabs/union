use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{
        create_wrapped_token::CreateWrappedTokenEvent,
        types::{BlockHeight, UniversalChainId},
    },
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
    pub message_index: Option<i64>,
    pub channel_id: i32,
    pub path: BigDecimal,
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
            message_index: value.event.header.message_index.pg_value()?,
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
                message_index,
                channel_id,
                path,
                base_token,
                quote_token,
                metadata,
                kind
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
            self.channel_id,
            self.path,
            &self.base_token[..],
            &self.quote_token[..],
            &self.metadata[..],
            self.kind
        )
        .execute(&mut **tx)
        .await?;

        Ok(Changes::with_single_insert::<Self>())
    }

    pub async fn find_by_chain_and_height(
        tx: &mut Transaction<'_, Postgres>,
        universal_chain_id: &UniversalChainId,
        height: &BlockHeight,
    ) -> Result<Vec<CreateWrappedTokenRecord>, IndexerError> {
        trace!("find_by_chain_and_height({universal_chain_id}, {height})");

        sqlx::query!(
            r#"
            SELECT
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                message_index,
                channel_id,
                path,
                base_token,
                quote_token,
                metadata,
                kind
            FROM v2_sync.create_wrapped_token_sync
            WHERE internal_chain_id = (SELECT id FROM config.chains WHERE family || '.' || chain_id = $1) AND height = $2
            "#,
            universal_chain_id.pg_value()?,
            height.pg_value()?
        )
        .fetch_all(tx.as_mut())
        .await?
        .into_iter()
        .map(|record| {
            Ok(CreateWrappedTokenRecord {
                // error handling can be removed when setting 'NOT NULL' property on columns
                internal_chain_id: record.internal_chain_id.ok_or_else(||IndexerError::InternalCannotMapFromDatabaseDomain("internal_chain_id".to_string(), "create_wrapped_token_sync".to_string()))?,
                block_hash: record.block_hash.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("block_hash".to_string(), "create_wrapped_token_sync".to_string()))?,
                height: record.height.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("height".to_string(), "create_wrapped_token_sync".to_string()))?,
                event_index: record.event_index.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("event_index".to_string(), "create_wrapped_token_sync".to_string()))?,
                timestamp: record.timestamp.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("timestamp".to_string(), "create_wrapped_token_sync".to_string()))?,
                transaction_hash: record.transaction_hash.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("transaction_hash".to_string(), "create_wrapped_token_sync".to_string()))?,
                transaction_index: record.transaction_index.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("transaction_index".to_string(), "create_wrapped_token_sync".to_string()))?,
                transaction_event_index: record.transaction_event_index,
                message_index: record.message_index,
                channel_id: record.channel_id.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("channel_id".to_string(), "create_wrapped_token_sync".to_string()))?,
                path: record.path.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("path".to_string(), "create_wrapped_token_sync".to_string()))?,
                base_token: record.base_token.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("base_token".to_string(), "create_wrapped_token_sync".to_string()))?,
                quote_token: record.quote_token.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("quote_token".to_string(), "create_wrapped_token_sync".to_string()))?,
                metadata: record.metadata.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("metadata".to_string(), "create_wrapped_token_sync".to_string()))?,
                kind: record.kind.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("kind".to_string(), "create_wrapped_token_sync".to_string()))?,
            })
        })
        .collect::<Result<Vec<CreateWrappedTokenRecord>, IndexerError>>()
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
