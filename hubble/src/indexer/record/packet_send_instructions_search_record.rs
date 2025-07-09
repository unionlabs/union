use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;
use tracing::trace;

use crate::indexer::{
    api::IndexerError,
    event::types::BlockHeight,
    handler::types::{ChannelMetaData, Instruction},
    record::{
        change_counter::{Changes, HasKind, RecordKind},
        packet_send_record::PacketSendRecord,
        InternalChainId, PgValue, PgValueExt,
    },
};

pub struct PacketSendInstructionsSearchRecord {
    pub internal_chain_id: i32,
    pub internal_counterparty_chain_id: i32,
    pub height: i64,
    pub packet_hash: Vec<u8>,
    pub transaction_hash: Vec<u8>,
    pub block_hash: Vec<u8>,
    pub timestamp: OffsetDateTime,
    pub instruction_index: i64,
    pub instruction_hash: Vec<u8>,
    pub instruction_type: String,
    pub path: Vec<u8>,
    pub salt: Vec<u8>,
    pub instruction_path: String,
    pub version: i32,
    pub opcode: i32,
    pub operand_sender: Option<String>,
    pub operand_contract_address: Option<String>,
    pub network: String,
    pub counterparty_network: String,
    pub sort_order: String,
}
impl HasKind for PacketSendInstructionsSearchRecord {
    fn kind() -> RecordKind {
        RecordKind::PacketSendInstructionsSearch
    }
}

impl TryFrom<(&PacketSendRecord, &Instruction, &ChannelMetaData, &String)>
    for PacketSendInstructionsSearchRecord
{
    type Error = IndexerError;

    fn try_from(
        (record, instruction, channel, sort_order): (
            &PacketSendRecord,
            &Instruction,
            &ChannelMetaData,
            &String,
        ),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            internal_chain_id: record.internal_chain_id,
            internal_counterparty_chain_id: channel.internal_counterparty_chain_id.pg_value()?,
            height: record.height,
            packet_hash: record.packet_hash.clone(),
            transaction_hash: record.transaction_hash.clone(),
            block_hash: record.block_hash.clone(),
            timestamp: record.timestamp,
            instruction_index: instruction.instruction_index.pg_value()?,
            instruction_hash: instruction.instruction_hash.pg_value()?,
            instruction_type: instruction.instruction_type.pg_value()?,
            path: instruction.path.pg_value()?,
            salt: instruction.salt.pg_value()?,
            instruction_path: instruction.instruction_path.pg_value()?,
            version: instruction.version.pg_value()?,
            opcode: instruction.opcode.pg_value()?,
            operand_sender: instruction.operand_sender.pg_value()?,
            operand_contract_address: instruction.operand_contract_address.pg_value()?,
            network: channel.network.pg_value()?,
            counterparty_network: channel.counterparty_network.pg_value()?,
            sort_order: sort_order.clone(),
        })
    }
}

impl PacketSendInstructionsSearchRecord {
    pub async fn insert_batch(
        tx: &mut Transaction<'_, Postgres>,
        records: &[PacketSendInstructionsSearchRecord],
    ) -> Result<Changes, IndexerError> {
        trace!("insert_batch({} records)", records.len());

        if records.is_empty() {
            return Ok(Changes::default());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO v2_sync.packet_send_instructions_search_sync (
                internal_chain_id,
                internal_counterparty_chain_id,
                height,
                packet_hash,
                transaction_hash,

                block_hash,
                timestamp,
                instruction_index,
                instruction_hash,
                instruction_type,

                path,
                salt,
                instruction_path,
                version,
                opcode,

                operand_sender,
                operand_contract_address,
                network,
                counterparty_network,
                sort_order
            ) ",
        );

        query_builder.push_values(records, |mut b, record| {
            b.push_bind(record.internal_chain_id)
                .push_bind(record.internal_counterparty_chain_id)
                .push_bind(record.height)
                .push_bind(&record.packet_hash[..])
                .push_bind(&record.transaction_hash[..])
                .push_bind(&record.block_hash[..])
                .push_bind(record.timestamp)
                .push_bind(record.instruction_index)
                .push_bind(&record.instruction_hash[..])
                .push_bind(&record.instruction_type)
                .push_bind(&record.path[..])
                .push_bind(&record.salt[..])
                .push_bind(&record.instruction_path)
                .push_bind(record.version)
                .push_bind(record.opcode)
                .push_bind(&record.operand_sender)
                .push_bind(&record.operand_contract_address)
                .push_bind(&record.network)
                .push_bind(&record.counterparty_network)
                .push_bind(&record.sort_order);
        });

        let query = query_builder.build();
        query.execute(&mut **tx).await?;

        Ok(Changes::with_inserts::<Self>(records.len() as u64))
    }

    pub async fn delete_by_chain_and_height(
        tx: &mut Transaction<'_, Postgres>,
        internal_chain_id: InternalChainId,
        height: BlockHeight,
    ) -> Result<Changes, IndexerError> {
        trace!("delete_by_chain_and_height({internal_chain_id}, {height})");

        let result = sqlx::query!(
            r#"
            DELETE FROM v2_sync.packet_send_instructions_search_sync
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
