use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{
        types::{BlockHeight, InternalChainId, InternalChainIdContext},
        update_client_event::UpdateClientEvent,
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

impl<'a> TryFrom<&'a InternalChainIdContext<'a, UpdateClientEvent>> for UpdateClientRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a InternalChainIdContext<'a, UpdateClientEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.internal_chain_id.pg_value()?,
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
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.update_client_test (
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
            DELETE FROM v2_sync.update_client_test
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
