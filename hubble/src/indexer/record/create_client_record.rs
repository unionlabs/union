use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{create_client_event::CreateClientEvent, types::BlockHeight},
    handler::EventContext,
    record::{ChainContext, InternalChainId, PgValue},
};

pub struct CreateClientRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub client_id: i64,
    pub client_type: String,
    pub counterparty_chain_id: String,
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, CreateClientEvent>> for CreateClientRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, CreateClientEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            client_id: value.event.client_id.pg_value()? as i64, // stored as bigint in postgres, should be i32
            client_type: value.event.client_type.pg_value()?,
            counterparty_chain_id: value.event.counterparty_chain_id.pg_value()?,
        })
    }
}

impl CreateClientRecord {
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.create_client_sync (
                internal_chain_id,
                block_hash,
                height,
                timestamp,
                transaction_hash,
                transaction_index,
                client_id,
                client_type,
                counterparty_chain_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.client_id,
            self.client_type,
            self.counterparty_chain_id,
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
            DELETE FROM v2_sync.create_client_sync
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
