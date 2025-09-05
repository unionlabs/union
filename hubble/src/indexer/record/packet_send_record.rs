use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::{
        packet_send_event::PacketSendEvent,
        types::{BlockHeight, UniversalChainId},
    },
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
    pub message_index: Option<i64>,
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
            message_index: value.event.header.message_index.pg_value()?,
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
                message_index,
                channel_id,
                packet_hash,
                source_channel_id,
                destination_channel_id,
                timeout_height,
                timeout_timestamp,
                data,
                network
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
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

    pub async fn find_by_chain_and_height(
        tx: &mut Transaction<'_, Postgres>,
        universal_chain_id: &UniversalChainId,
        height: &BlockHeight,
    ) -> Result<Vec<PacketSendRecord>, IndexerError> {
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
                packet_hash,
                source_channel_id,
                destination_channel_id,
                timeout_height,
                timeout_timestamp,
                data,
                network
            FROM v2_sync.packet_send_sync
            WHERE internal_chain_id = (SELECT id FROM config.chains WHERE family || '.' || chain_id = $1) AND height = $2
            "#,
            universal_chain_id.pg_value()?,
            height.pg_value()?
        )
        .fetch_all(tx.as_mut())
        .await?
        .into_iter()
        .map(|record| {
            Ok(PacketSendRecord {
                // error handling can be removed when setting 'NOT NULL' property on columns
                internal_chain_id: record.internal_chain_id.ok_or_else(||IndexerError::InternalCannotMapFromDatabaseDomain("internal_chain_id".to_string(), "packet_send_sync".to_string()))?,
                block_hash: record.block_hash.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("block_hash".to_string(), "packet_send_sync".to_string()))?,
                height: record.height.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("height".to_string(), "packet_send_sync".to_string()))?,
                event_index: record.event_index.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("event_index".to_string(), "packet_send_sync".to_string()))?,
                timestamp: record.timestamp.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("timestamp".to_string(), "packet_send_sync".to_string()))?,
                transaction_hash: record.transaction_hash.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("transaction_hash".to_string(), "packet_send_sync".to_string()))?,
                transaction_index: record.transaction_index.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("transaction_index".to_string(), "packet_send_sync".to_string()))?,
                transaction_event_index: record.transaction_event_index,
                message_index: record.message_index,
                channel_id: record.channel_id.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("channel_id".to_string(), "packet_send_sync".to_string()))?,
                packet_hash: record.packet_hash.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("packet_hash".to_string(), "packet_send_sync".to_string()))?,
                source_channel_id: record.source_channel_id.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("source_channel_id".to_string(), "packet_send_sync".to_string()))?,
                destination_channel_id: record.destination_channel_id.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("destination_channel_id".to_string(), "packet_send_sync".to_string()))?,
                timeout_height: record.timeout_height.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("timeout_height".to_string(), "packet_send_sync".to_string()))?,
                timeout_timestamp: record.timeout_timestamp.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("timeout_timestamp".to_string(), "packet_send_sync".to_string()))?,
                data: record.data.ok_or_else(||IndexerError::InternalCannotMapToDatabaseDomain("data".to_string(), "packet_send_sync".to_string()))?,
                network: record.network,
            })
        })
        .collect::<Result<Vec<PacketSendRecord>, IndexerError>>()
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
