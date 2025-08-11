use sqlx::{types::BigDecimal, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    handler::types::{ChannelMetaData, Transfer},
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        packet_send_record::PacketSendRecord,
        InternalChainId, PgValue, PgValueExt,
    },
};

pub struct PacketSendTransfersRecord {
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
    pub transfer_index: i32,
    pub sender_canonical: Vec<u8>,
    pub sender_display: String,
    pub sender_zkgm: Vec<u8>,
    pub receiver_canonical: Vec<u8>,
    pub receiver_display: String,
    pub receiver_zkgm: Vec<u8>,
    pub wrap_direction: Option<String>,
    pub base_token: Vec<u8>,
    pub base_amount: BigDecimal,
    pub base_token_name: Option<String>,
    pub base_token_path: Option<Vec<u8>>,
    pub base_token_symbol: Option<String>,
    pub base_token_decimals: Option<i32>,
    pub quote_token: Vec<u8>,
    pub quote_amount: BigDecimal,
    pub fee_type: String,
    pub fee_token: Option<Vec<u8>>,
    pub fee_amount: Option<BigDecimal>,
    pub packet_shape: String,
    pub sort_order: String,
    pub network: String,
    pub counterparty_network: String,
    pub kind: Option<i32>,
    pub metadata: Option<Vec<u8>>,
}
impl HasKind for PacketSendTransfersRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSendTransfers
    }
}

impl TryFrom<(&PacketSendRecord, &Transfer, &ChannelMetaData, &String)>
    for PacketSendTransfersRecord
{
    type Error = IndexerError;

    fn try_from(
        (record, transfer, channel, sort_order): (
            &PacketSendRecord,
            &Transfer,
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
            transfer_index: transfer.transfer_index.pg_value()?,
            sender_canonical: transfer.sender_canonical.pg_value()?,
            sender_display: transfer.sender_display.pg_value()?,
            sender_zkgm: transfer.sender_zkgm.pg_value()?,
            receiver_canonical: transfer.receiver_canonical.pg_value()?,
            receiver_display: transfer.receiver_display.pg_value()?,
            receiver_zkgm: transfer.receiver_zkgm.pg_value()?,
            wrap_direction: transfer.wrap_direction.pg_value()?,
            base_token: transfer.base_token.pg_value()?,
            base_amount: transfer.base_amount.pg_value()?,
            base_token_name: transfer.base_token_name.pg_value()?,
            base_token_path: transfer.base_token_path.pg_value()?,
            base_token_symbol: transfer.base_token_symbol.pg_value()?,
            base_token_decimals: transfer.base_token_decimals.pg_value()?,
            quote_token: transfer.quote_token.pg_value()?,
            quote_amount: transfer.quote_amount.pg_value()?,
            fee_type: transfer.fee.pg_value()?,
            fee_token: transfer.fee.token().pg_value()?,
            fee_amount: transfer.fee.amount().pg_value()?,
            packet_shape: transfer.packet_shape.pg_value()?,
            sort_order: sort_order.clone(),
            network: channel.network.pg_value()?,
            counterparty_network: channel.counterparty_network.pg_value()?,
            kind: transfer.kind.pg_value()?,
            metadata: transfer.metadata.pg_value()?,
        })
    }
}

impl PacketSendTransfersRecord {
    pub async fn insert(
        &self,
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Changes, IndexerError> {
        trace!("insert({})", self.height);

        sqlx::query!(
            r#"
            INSERT INTO v2_sync.packet_send_transfers_sync (
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
                transfer_index,
                sender_canonical,
                sender_display,

                sender_zkgm,
                receiver_canonical,
                receiver_display,
                receiver_zkgm,
                wrap_direction,

                base_token,
                base_amount,
                base_token_name,
                base_token_path,
                base_token_symbol,

                base_token_decimals,
                quote_token,
                quote_amount,
                fee_type,
                fee_token,

                fee_amount,
                packet_shape,
                sort_order,
                network,
                counterparty_network,

                kind,
                metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36, $37, $38, $39, $40, $41, $42)
            "#,
            self.internal_chain_id,
            self.universal_chain_id,
            self.internal_counterparty_chain_id,
            self.counterparty_universal_chain_id,
            self.client_id,
            self.counterparty_client_id,
            self.connection_id,
            self.counterparty_connection_id,
            self.source_channel_id,
            self.destination_channel_id,
            self.port_id,
            &self.counterparty_port_id[..],
            &self.block_hash[..],
            &self.transaction_hash[..],
            &self.packet_hash[..],
            self.height,
            self.timestamp,
            self.transfer_index,
            &self.sender_canonical[..],
            self.sender_display,
            &self.sender_zkgm[..],
            &self.receiver_canonical[..],
            self.receiver_display,
            &self.receiver_zkgm[..],
            self.wrap_direction,
            &self.base_token[..],
            self.base_amount,
            self.base_token_name,
            self.base_token_path.as_deref(),
            self.base_token_symbol,
            self.base_token_decimals,
            &self.quote_token[..],
            self.quote_amount,
            self.fee_type,
            self.fee_token.as_deref(),
            self.fee_amount,
            self.packet_shape,
            self.sort_order,
            self.network,
            self.counterparty_network,
            self.kind,
            self.metadata,
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
            DELETE FROM v2_sync.packet_send_transfers_sync
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
