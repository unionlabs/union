use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{channel_open_init_event::ChannelOpenInitEvent, types::BlockHeight},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue, PgValueExt,
    },
};

pub struct ChannelOpenInitRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub message_index: Option<i64>,
    pub port_id: Vec<u8>,
    pub channel_id: i32,
    pub connection_id: i32,
    pub counterparty_port_id: Vec<u8>,
    pub version: String,
}

impl HasKind for ChannelOpenInitRecord {
    fn kind() -> RecordKind {
        RecordKind::ChannelOpenInit
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, ChannelOpenInitEvent>>
    for ChannelOpenInitRecord
{
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, ChannelOpenInitEvent>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: value.context.internal_chain_id.pg_value()?,
            block_hash: value.event.header.block_hash.pg_value()?,
            height: value.event.header.height.pg_value()?,
            timestamp: value.event.header.timestamp.pg_value()?,
            transaction_hash: value.event.header.transaction_hash.pg_value()?,
            transaction_index: value.event.header.transaction_index.pg_value()?,
            message_index: value.event.header.message_index.pg_value()?,
            connection_id: value.event.connection_id.pg_value()?,
            channel_id: value.event.channel_id.pg_value()?,
            port_id: value.event.port_id.pg_value()?,
            counterparty_port_id: value.event.counterparty_port_id.pg_value()?,
            version: value.event.version.pg_value()?,
        })
    }
}

impl ChannelOpenInitRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.channel_open_init_sync (
                internal_chain_id,
                block_hash,
                height,
                timestamp,
                transaction_hash,
                transaction_index,
                message_index,
                port_id,
                channel_id,
                connection_id,
                counterparty_port_id,
                version
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            self.internal_chain_id,
            &self.block_hash[..],
            self.height,
            self.timestamp,
            &self.transaction_hash[..],
            self.transaction_index,
            self.message_index,
            &self.port_id[..],
            self.channel_id,
            self.connection_id,
            &self.counterparty_port_id[..],
            self.version,
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
            DELETE FROM v2_sync.channel_open_init_sync
            WHERE internal_chain_id = $1 AND height = $2
            "#,
            internal_chain_id.pg_value()?,
            height.pg_value()?,
        )
        .execute(&mut **tx)
        .await?;

        Ok(Changes::with_deletes::<Self>(result.rows_affected()))
    }
}
