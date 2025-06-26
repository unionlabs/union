use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{types::BlockHeight, wallet_mutation_entry_event::WalletMutationEntryEvent},
    handler::EventContext,
    record::{ChainContext, InternalChainId},
};

pub struct WalletMutationEntryRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: i64,
    // missing event_index
    pub contract_address_canonical: Vec<u8>,
    pub wallet_address_canonical: Vec<u8>,
    pub amount: BigDecimal,
    pub direction: String,
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, WalletMutationEntryEvent>>
    for WalletMutationEntryRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, WalletMutationEntryEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value_integer()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value_bigint()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            transaction_event_index: value.event.header.transaction_event_index.pg_value()?,
            contract_address_canonical: value.event.contract_address_canonical.pg_value()?,
            wallet_address_canonical: value.event.wallet_address_canonical.pg_value()?,
            amount: value.event.amount.pg_value()?,
            direction: value.event.direction.pg_value()?,
        })
    }
}

impl WalletMutationEntryRecord {
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.wallet_mutation_entry_test (
                internal_chain_id,
                block_hash,
                height,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                contract_address_canonical,
                wallet_address_canonical,
                amount,
                direction
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.transaction_event_index,
            &self.contract_address_canonical[..],
            &self.wallet_address_canonical[..],
            self.amount,
            self.direction,
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
            DELETE FROM v2_sync.wallet_mutation_entry_test
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
