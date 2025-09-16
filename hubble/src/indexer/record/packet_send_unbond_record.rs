use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    handler::types::{ChannelMetaData, Unbond},
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        packet_send_record::PacketSendRecord,
        InternalChainId, PgValue,
    },
};

pub struct PacketSendUnbondRecord {
    pub internal_chain_id: i32,
    pub universal_chain_id: String,
    pub internal_counterparty_chain_id: i32,
    pub counterparty_universal_chain_id: String,
    pub client_id: i32,
    pub counterparty_client_id: i32,
    pub connection_id: i32,
    pub counterparty_connection_id: i32,
    pub source_channel_id: i32,
    pub destination_channel_id: i32,
    pub port_id: Vec<u8>,
    pub counterparty_port_id: Vec<u8>,
    pub block_hash: Vec<u8>,
    pub transaction_hash: Vec<u8>,
    pub packet_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub sender_canonical: Vec<u8>,
    pub sender_display: String,
    pub sender_zkgm: Vec<u8>,
    pub base_token: Vec<u8>,
    pub base_amount: BigDecimal,
    pub unbond_amount: BigDecimal,
    pub packet_shape: String,
    pub sort_order: String,
    pub network: String,
    pub counterparty_network: String,
}
impl HasKind for PacketSendUnbondRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSendUnbond
    }
}

impl TryFrom<(&PacketSendRecord, &Unbond, &ChannelMetaData, &String)> for PacketSendUnbondRecord {
    type Error = IndexerError;

    fn try_from(
        (record, unbond, channel, sort_order): (
            &PacketSendRecord,
            &Unbond,
            &ChannelMetaData,
            &String,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: record.internal_chain_id,
            universal_chain_id: channel.universal_chain_id.pg_value()?,
            internal_counterparty_chain_id: channel.internal_counterparty_chain_id.pg_value()?,
            counterparty_universal_chain_id: channel.universal_counterparty_chain_id.pg_value()?,
            client_id: channel.client_id.pg_value()?,
            counterparty_client_id: channel.counterparty_client_id.pg_value()?,
            connection_id: channel.connection_id.pg_value()?,
            counterparty_connection_id: channel.counterparty_connection_id.pg_value()?,
            source_channel_id: record.source_channel_id,
            destination_channel_id: record.destination_channel_id,
            port_id: channel.port_id.pg_value()?,
            counterparty_port_id: channel.counterparty_port_id.pg_value()?,
            block_hash: record.block_hash.clone(),
            transaction_hash: record.transaction_hash.clone(),
            packet_hash: record.packet_hash.clone(),
            height: record.height,
            timestamp: record.timestamp,
            sender_canonical: unbond.sender_canonical.pg_value()?,
            sender_display: unbond.sender_display.pg_value()?,
            sender_zkgm: unbond.sender_zkgm.pg_value()?,
            base_token: unbond.base_token.pg_value()?,
            base_amount: unbond.base_amount.pg_value()?,
            unbond_amount: unbond.unbond_amount.pg_value()?,
            packet_shape: unbond.packet_shape.pg_value()?,
            sort_order: sort_order.clone(),
            network: channel.network.pg_value()?,
            counterparty_network: channel.counterparty_network.pg_value()?,
        })
    }
}

impl PacketSendUnbondRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_send_unbond_sync (
                internal_chain_id,
                universal_chain_id,
                internal_counterparty_chain_id,
                counterparty_universal_chain_id,
                client_id,

                counterparty_client_id,
                connection_id,
                counterparty_connection_id,
                source_channel_id,
                destination_channel_id,

                port_id,
                counterparty_port_id,
                block_hash,
                transaction_hash,
                packet_hash,

                height,
                timestamp,
                sender_canonical,
                sender_display,
                sender_zkgm,

                base_token,
                base_amount,
                unbond_amount,
                packet_shape,

                sort_order,
                network,
                counterparty_network
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)
            "#,
            // 1
            self.internal_chain_id,
            self.universal_chain_id,
            self.internal_counterparty_chain_id,
            self.counterparty_universal_chain_id,
            self.client_id,
            // 6
            self.counterparty_client_id,
            self.connection_id,
            self.counterparty_connection_id,
            self.source_channel_id,
            self.destination_channel_id,
            // 11
            self.port_id,
            &self.counterparty_port_id[..],
            &self.block_hash[..],
            &self.transaction_hash[..],
            &self.packet_hash[..],
            // 16
            self.height,
            self.timestamp,
            &self.sender_canonical[..],
            self.sender_display,
            &self.sender_zkgm[..],
            // 21
            &self.base_token[..],
            self.base_amount,
            self.unbond_amount,
            self.packet_shape,
            self.sort_order,
            // 26
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
            DELETE FROM v2_sync.packet_send_unbond_sync
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
