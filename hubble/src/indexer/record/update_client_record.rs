use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{types::BlockHeight, update_client_event::UpdateClientEvent},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue,
    },
};

pub struct UpdateClientRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub transaction_hash: Vec<u8>,
    pub client_id: i32,
    pub timestamp: OffsetDateTime,
    pub counterparty_height: i64,
}
impl HasKind for UpdateClientRecord {
    fn kind() -> RecordKind {
        RecordKind::UpdateClient
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, UpdateClientEvent>> for UpdateClientRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, UpdateClientEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            client_id: value.event.client_id.pg_value()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            counterparty_height: value.event.counterparty_height.pg_value()?,
        })
    }
}

impl UpdateClientRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.update_client_sync (
                internal_chain_id,
                block_hash,
                height,
                transaction_hash,
                client_id,
                timestamp,
                counterparty_height
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            &self.transaction_hash[..],
            self.client_id,
            self.timestamp,
            self.counterparty_height,
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
            DELETE FROM v2_sync.update_client_sync
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
