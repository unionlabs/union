use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{connection_open_confirm_event::ConnectionOpenConfirmEvent, types::BlockHeight},
    handler::EventContext,
    record::{ChainContext, InternalChainId, PgValue},
};

pub struct ConnectionOpenConfirmRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub connection_id: i32,
    pub client_id: i32,
    pub counterparty_client_id: i32,
    pub counterparty_connection_id: i32,
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, ConnectionOpenConfirmEvent>>
    for ConnectionOpenConfirmRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, ConnectionOpenConfirmEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            // missing transaction_event_index in datamodel
            connection_id: value.event.connection_id.pg_value()?,
            client_id: value.event.client_id.pg_value()?,
            counterparty_client_id: value.event.counterparty_client_id.pg_value()?,
            counterparty_connection_id: value.event.counterparty_connection_id.pg_value()?,
        })
    }
}

impl ConnectionOpenConfirmRecord {
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.connection_open_confirm_sync (
                internal_chain_id,
                block_hash,
                height,
                timestamp,
                transaction_hash,
                transaction_index,
                connection_id,
                client_id,
                counterparty_client_id,
                counterparty_connection_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.connection_id,
            self.client_id,
            self.counterparty_client_id,
            self.counterparty_connection_id,
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
            DELETE FROM v2_sync.connection_open_confirm_sync
            WHERE internal_chain_id = $1 AND height = $2
            "#,
            internal_chain_id.pg_value()?,
            height.pg_value()?
        )
        .execute(&mut **tx)
        .await?;

        Ok(result.rows_affected())
    }
}
