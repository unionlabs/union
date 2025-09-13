use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_proxy_account_event::CreateProxyAccountEvent, types::BlockHeight},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue, PgValueExt,
    },
};

pub struct CreateProxyAccountRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: Option<i64>,
    pub message_index: Option<i64>,
    pub path: BigDecimal,
    pub channel_id: i32,
    pub owner: Vec<u8>,
    pub proxy_account: Vec<u8>,
}
impl HasKind for CreateProxyAccountRecord {
    fn kind() -> RecordKind {
        RecordKind::CreateProxyAccount
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, CreateProxyAccountEvent>>
    for CreateProxyAccountRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, CreateProxyAccountEvent>,
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
            path: value.event.path.pg_value()?,
            channel_id: value.event.channel_id.pg_value()?,
            owner: value.event.owner.pg_value()?,
            proxy_account: value.event.proxy_account.pg_value()?,
        })
    }
}

impl CreateProxyAccountRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.create_proxy_account_sync (
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                message_index,
                path,
                channel_id,
                owner,
                proxy_account
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
            self.path,
            self.channel_id,
            &self.owner[..],
            &self.proxy_account[..],
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
            DELETE FROM v2_sync.create_proxy_account_sync
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
