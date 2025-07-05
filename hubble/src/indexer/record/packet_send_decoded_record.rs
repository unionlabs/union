use serde_json::Value;
use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    handler::types::ChannelMetaData,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        packet_send_record::PacketSendRecord,
        InternalChainId, PgValue,
    },
};

pub struct PacketSendDecodedRecord {
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
    pub data_decoded: Value,
    pub data_decoded_flattened: Value,
    pub chain_id: String,
    pub counterparty_chain_id: String,
    pub client_id: i32,
    pub counterparty_client_id: i32,
    pub connection_id: i32,
    pub counterparty_connection_id: i32,
    pub port_id: Vec<u8>,
    pub counterparty_port_id: Vec<u8>,
    pub channel_version: String,
    pub client_type: String,
    pub internal_counterparty_chain_id: i32,
    pub sort_order: String,
    pub rpc_type: String,
    pub counterparty_rpc_type: String,
    pub universal_chain_id: String,
    pub counterparty_universal_chain_id: String,
    pub structure: String,
    pub network: String,
    pub counterparty_network: String,
}
impl HasKind for PacketSendDecodedRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSendDecoded
    }
}

impl
    TryFrom<(
        &PacketSendRecord,
        &ChannelMetaData,
        &Value,
        &Value,
        &String,
        &String,
    )> for PacketSendDecodedRecord
{
    type Error = IndexerError;

    fn try_from(
        (record, channel, tree, flatten, structure, sort_order): (
            &PacketSendRecord,
            &ChannelMetaData,
            &Value,
            &Value,
            &String,
            &String,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: record.internal_chain_id,
            block_hash: record.block_hash.clone(),
            height: record.height,
            event_index: record.event_index,
            timestamp: record.timestamp,
            transaction_hash: record.transaction_hash.clone(),
            transaction_index: record.transaction_index,
            transaction_event_index: record.transaction_event_index,
            channel_id: record.channel_id,
            packet_hash: record.packet_hash.clone(),
            source_channel_id: record.source_channel_id,
            destination_channel_id: record.destination_channel_id,
            timeout_height: record.timeout_height.clone(),
            timeout_timestamp: record.timeout_timestamp.clone(),
            data: record.data.clone(),
            data_decoded: tree.clone(),
            data_decoded_flattened: flatten.clone(),
            chain_id: channel.canonical_chain_id.pg_value()?,
            counterparty_chain_id: channel.canonical_counterparty_chain_id.pg_value()?,
            client_id: channel.client_id.pg_value()?,
            counterparty_client_id: channel.counterparty_client_id.pg_value()?,
            connection_id: channel.connection_id.pg_value()?,
            counterparty_connection_id: channel.counterparty_connection_id.pg_value()?,
            port_id: channel.port_id.pg_value()?,
            counterparty_port_id: channel.counterparty_port_id.pg_value()?,
            channel_version: channel.channel_version.pg_value()?,
            client_type: channel.client_type.pg_value()?,
            internal_counterparty_chain_id: channel.internal_counterparty_chain_id.pg_value()?,
            sort_order: sort_order.clone(),
            rpc_type: channel.rpc_type.pg_value()?,
            counterparty_rpc_type: channel.counterparty_rpc_type.pg_value()?,
            universal_chain_id: channel.universal_chain_id.pg_value()?,
            counterparty_universal_chain_id: channel.universal_counterparty_chain_id.pg_value()?,
            structure: structure.clone(),
            network: channel.network.pg_value()?,
            counterparty_network: channel.counterparty_network.pg_value()?,
        })
    }
}

impl PacketSendDecodedRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_send_decoded_sync (
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

                data_decoded,
                data_decoded_flattened,
                chain_id,
                counterparty_chain_id,
                client_id,

                counterparty_client_id,
                connection_id,
                counterparty_connection_id,
                port_id,
                counterparty_port_id,

                channel_version,
                client_type,
                internal_counterparty_chain_id,
                sort_order,
                rpc_type,

                counterparty_rpc_type,
                universal_chain_id,
                counterparty_universal_chain_id,
                structure,
                network,

                counterparty_network
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
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
            self.data_decoded,
            self.data_decoded_flattened,
            self.chain_id,
            self.counterparty_chain_id,
            self.client_id,
            self.counterparty_client_id,
            self.connection_id,
            self.counterparty_connection_id,
            &self.port_id[..],
            &self.counterparty_port_id[..],
            self.channel_version,
            self.client_type,
            self.internal_counterparty_chain_id,
            self.sort_order,
            self.rpc_type,
            self.counterparty_rpc_type,
            self.universal_chain_id,
            self.counterparty_universal_chain_id,
            self.structure,
            self.network,
            self.counterparty_network,
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
            DELETE FROM v2_sync.packet_send_decoded_sync
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
