use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    handler::types::Bond,
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        packet_send_record::PacketSendRecord,
        InternalChainId, PgValue,
    },
};

pub struct PacketSendBondRecord {
    pub internal_chain_id: i32,
    pub universal_chain_id: String,
    pub remote_universal_chain_id: String,
    pub destination_universal_chain_id: String,
    pub source_network: String,
    pub remote_network: String,
    pub destination_network: String,
    pub internal_remote_chain_id: i32,
    pub internal_destination_chain_id: i32,
    pub source_client_id: i32,
    pub remote_source_client_id: i32,
    pub remote_destination_client_id: i32,
    pub destination_client_id: i32,
    pub source_connection_id: i32,
    pub remote_source_connection_id: i32,
    pub remote_destination_connection_id: i32,
    pub destination_connection_id: i32,
    pub source_channel_id: i32,
    pub remote_source_channel_id: i32,
    pub remote_destination_channel_id: i32,
    pub destination_channel_id: i32,
    pub source_port_id: Vec<u8>,
    pub remote_source_port_id: Vec<u8>,
    pub remote_destination_port_id: Vec<u8>,
    pub destination_port_id: Vec<u8>,

    pub block_hash: Vec<u8>,
    pub transaction_hash: Vec<u8>,
    pub packet_hash: Vec<u8>,
    pub height: i64,
    pub timestamp: OffsetDateTime,
    pub sender_canonical: Vec<u8>,
    pub sender_display: String,
    pub sender_zkgm: Vec<u8>,
    pub receiver_canonical: Vec<u8>,
    pub receiver_display: String,
    pub receiver_zkgm: Vec<u8>,
    pub base_token: Vec<u8>,
    pub base_amount: BigDecimal,
    pub quote_token: Vec<u8>,
    pub quote_amount: BigDecimal,
    pub remote_base_token: Vec<u8>,
    pub remote_base_amount: BigDecimal,
    pub remote_quote_token: Vec<u8>,
    pub remote_quote_amount: BigDecimal,
    pub delivery_packet_hash: Vec<u8>,
    pub packet_shape: String,
    pub sort_order: String,
}
impl HasKind for PacketSendBondRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSendBond
    }
}

impl TryFrom<(&PacketSendRecord, &Bond, &String)> for PacketSendBondRecord {
    type Error = IndexerError;

    fn try_from(
        (record, bond, sort_order): (&PacketSendRecord, &Bond, &String),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: record.internal_chain_id,
            universal_chain_id: bond.universal_chain_id.pg_value()?,
            remote_universal_chain_id: bond.remote_universal_chain_id.pg_value()?,
            destination_universal_chain_id: bond.destination_universal_chain_id.pg_value()?,
            source_network: bond.source_network.pg_value()?,
            remote_network: bond.remote_network.pg_value()?,
            destination_network: bond.destination_network.pg_value()?,
            internal_remote_chain_id: bond.internal_remote_chain_id.pg_value()?,
            internal_destination_chain_id: bond.internal_destination_chain_id.pg_value()?,
            source_client_id: bond.source_client_id.pg_value()?,
            remote_source_client_id: bond.remote_source_client_id.pg_value()?,
            remote_destination_client_id: bond.remote_destination_client_id.pg_value()?,
            destination_client_id: bond.destination_client_id.pg_value()?,
            source_connection_id: bond.source_connection_id.pg_value()?,
            remote_source_connection_id: bond.remote_source_connection_id.pg_value()?,
            remote_destination_connection_id: bond.remote_destination_connection_id.pg_value()?,
            destination_connection_id: bond.destination_connection_id.pg_value()?,
            source_channel_id: bond.source_channel_id.pg_value()?,
            remote_source_channel_id: bond.remote_source_channel_id.pg_value()?,
            remote_destination_channel_id: bond.remote_destination_channel_id.pg_value()?,
            destination_channel_id: bond.destination_channel_id.pg_value()?,
            source_port_id: bond.source_port_id.pg_value()?,
            remote_source_port_id: bond.remote_source_port_id.pg_value()?,
            remote_destination_port_id: bond.remote_destination_port_id.pg_value()?,
            destination_port_id: bond.destination_port_id.pg_value()?,
            block_hash: record.block_hash.clone(),
            transaction_hash: record.transaction_hash.clone(),
            packet_hash: record.packet_hash.clone(),
            height: record.height,
            timestamp: record.timestamp,
            sender_canonical: bond.sender_canonical.pg_value()?,
            sender_display: bond.sender_display.pg_value()?,
            sender_zkgm: bond.sender_zkgm.pg_value()?,
            receiver_canonical: bond.receiver_canonical.pg_value()?,
            receiver_display: bond.receiver_display.pg_value()?,
            receiver_zkgm: bond.receiver_zkgm.pg_value()?,
            base_token: bond.base_token.pg_value()?,
            base_amount: bond.base_amount.pg_value()?,
            quote_token: bond.quote_token.pg_value()?,
            quote_amount: bond.quote_amount.pg_value()?,
            remote_base_token: bond.remote_base_token.pg_value()?,
            remote_base_amount: bond.remote_base_amount.pg_value()?,
            remote_quote_token: bond.remote_quote_token.pg_value()?,
            remote_quote_amount: bond.remote_quote_amount.pg_value()?,
            delivery_packet_hash: bond.delivery_packet_hash.pg_value()?,
            packet_shape: bond.packet_shape.pg_value()?,
            sort_order: sort_order.clone(),
        })
    }
}

impl PacketSendBondRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_send_bond_sync (
                internal_chain_id,
                universal_chain_id,
                remote_universal_chain_id,
                destination_universal_chain_id,
                source_network,

                remote_network,
                destination_network,
                internal_remote_chain_id,
                internal_destination_chain_id,
                source_client_id,

                remote_source_client_id,
                remote_destination_client_id,
                destination_client_id,
                source_connection_id,
                remote_source_connection_id,

                remote_destination_connection_id,
                destination_connection_id,
                source_channel_id,
                remote_source_channel_id,
                remote_destination_channel_id,

                destination_channel_id,
                source_port_id,
                remote_source_port_id,
                remote_destination_port_id,
                destination_port_id,

                block_hash,
                transaction_hash,
                packet_hash,
                height,
                timestamp,

                sender_canonical,
                sender_display,
                sender_zkgm,
                receiver_canonical,
                receiver_display,

                receiver_zkgm,
                base_token,
                base_amount,
                quote_token,
                quote_amount,

                remote_base_token,
                remote_base_amount,
                remote_quote_token,
                remote_quote_amount,
                delivery_packet_hash,

                packet_shape,
                sort_order
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42, $43, $44, $45, $46, $47)
            "#,
            // 1
            &self.internal_chain_id,
            &self.universal_chain_id,
            &self.remote_universal_chain_id,
            &self.destination_universal_chain_id,
            &self.source_network,
            // 6
            &self.remote_network,
            &self.destination_network,
            &self.internal_remote_chain_id,
            &self.internal_destination_chain_id,
            &self.source_client_id,
            // 11
            &self.remote_source_client_id,
            &self.remote_destination_client_id,
            &self.destination_client_id,
            &self.source_connection_id,
            &self.remote_source_connection_id,
            // 16
            &self.remote_destination_connection_id,
            &self.destination_connection_id,
            &self.source_channel_id,
            &self.remote_source_channel_id,
            &self.remote_destination_channel_id,
            // 21
            &self.destination_channel_id,
            &self.source_port_id,
            &self.remote_source_port_id,
            &self.remote_destination_port_id,
            &self.destination_port_id,
            // 26
            &self.block_hash[..],
            &self.transaction_hash[..],
            &self.packet_hash[..],
            self.height,
            self.timestamp,
            // 31
            &self.sender_canonical[..],
            self.sender_display,
            &self.sender_zkgm[..],
            &self.receiver_canonical[..],
            self.receiver_display,
            // 36
            &self.receiver_zkgm[..],
            &self.base_token[..],
            self.base_amount,
            &self.quote_token[..],
            self.quote_amount,
            // 41
            &self.remote_base_token[..],
            self.remote_base_amount,
            &self.remote_quote_token[..],
            self.remote_quote_amount,
            self.delivery_packet_hash,
            // 46
            self.packet_shape,
            self.sort_order,
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
            DELETE FROM v2_sync.packet_send_bond_sync
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
