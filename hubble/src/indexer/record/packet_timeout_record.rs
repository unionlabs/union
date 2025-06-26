use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_timeout_event::PacketTimeoutEvent, types::BlockHeight},
    handler::EventContext,
    record::{ChainContext, InternalChainId},
};

pub struct PacketTimeoutRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i32, // should be i64?
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: i64,
    pub channel_id: i32,
    pub packet_hash: Vec<u8>,
    pub maker: Vec<u8>,
    pub network: String,
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, PacketTimeoutEvent>> for PacketTimeoutRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, PacketTimeoutEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value_integer()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value_bigint()?,
            event_index: value.event.header.event_index.pg_value_integer()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            transaction_event_index: value.event.header.transaction_event_index.pg_value()?,
            channel_id: value.event.channel_id.pg_value()?,
            packet_hash: value.event.packet_hash.pg_value()?,
            maker: value.event.maker.pg_value()?,
            network: value.context.network.pg_value()?,
        })
    }
}

impl PacketTimeoutRecord {
    pub async fn insert(&self, tx: &mut Transaction<'_, Postgres>) -> Result<(), IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_timeout_test (
                internal_chain_id,
                block_hash,
                height,
                event_index,
                timestamp,
                transaction_hash,
                transaction_index,
                transaction_event_index,
                channel_id,
                packet_hash,
                maker,
                network
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
            &self.packet_hash[..],
            &self.maker[..],
            self.network,
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
            DELETE FROM v2_sync.packet_timeout_test
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
