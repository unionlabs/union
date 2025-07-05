use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{packet_send_event::PacketSendEvent, types::BlockHeight},
    handler::EventContext,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        ChainContext, InternalChainId, PgValue, PgValueExt,
    },
};

#[derive(Clone)]
pub struct PacketSendRecord {
    pub internal_chain_id: i32,
    pub block_hash: Vec<u8>,
    pub height: i64,
    pub event_index: i32, // should be i64?
    pub timestamp: OffsetDateTime,
    pub transaction_hash: Vec<u8>,
    pub transaction_index: i64,
    pub transaction_event_index: Option<i64>,
    pub channel_id: i32,
    pub packet_hash: Vec<u8>,
    pub source_channel_id: i32,
    pub destination_channel_id: i32,
    pub timeout_height: BigDecimal,
    pub timeout_timestamp: BigDecimal,
    pub data: Vec<u8>,
    pub network: String,
}
impl HasKind for PacketSendRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSend
    }
}

impl<'a> TryFrom<&'a EventContext<'a, ChainContext, PacketSendEvent>> for PacketSendRecord {
    type Error = IndexerError;

    fn try_from(
        value: &'a EventContext<'a, ChainContext, PacketSendEvent>,
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
            packet_hash: value.event.packet_hash.pg_value()?,
            source_channel_id: value.event.source_channel_id.pg_value()?,
            destination_channel_id: value.event.destination_channel_id.pg_value()?,
            timeout_height: value.event.timeout_height.pg_value_numeric()?,
            timeout_timestamp: value.event.timeout_timestamp.pg_value()?,
            data: value.event.data.pg_value()?,
            network: value.context.network.pg_value()?,
        })
    }
}

impl PacketSendRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_send_sync (
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
                source_channel_id,
                destination_channel_id,
                timeout_height,
                timeout_timestamp,
                data,
                network
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
            self.source_channel_id,
            self.destination_channel_id,
            self.timeout_height,
            self.timeout_timestamp,
            &self.data[..],
            self.network,
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
            DELETE FROM v2_sync.packet_send_sync
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
