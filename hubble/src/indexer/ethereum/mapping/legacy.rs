use alloy::{network::AnyRpcBlock, rpc::types::Log};
use alloy_primitives::FixedBytes;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    indexer::{
        api::{BlockHash, BlockHeight, IndexerError},
        ethereum::{
            abi::Abi,
            block_handle::{BlockInsert, TransactionInsert},
            fetcher_client::EthFetcherClient,
        },
        event::supported::SupportedBlockEvent,
    },
    postgres::ChainId,
};

impl EthFetcherClient {
    // legacy decoded log as json
    pub fn to_decoded_log(
        &self,
        abi: &Abi,
        block: &AnyRpcBlock,
        transaction_log_index: usize,
        log: &Log,
    ) -> Result<SupportedBlockEvent, IndexerError> {
        let decoded = abi.decode_to_json(log)?;

        Ok(SupportedBlockEvent::EthereumDecodedLog {
            internal_chain_id: self.chain_id.db,
            block_hash: block.header.hash.to_lower_hex(),
            height: block.header.number.into(),
            log_index: i32::try_from(log.log_index.expect("log index in log"))
                .expect("log index fits"),
            timestamp: OffsetDateTime::from_unix_timestamp(
                block
                    .header
                    .timestamp
                    .try_into()
                    .expect("timestamp fits in i64"),
            )
            .expect("timestamp can be converted"),
            transaction_hash: log
                .transaction_hash
                .expect("transaction-hash in log")
                .to_string(),
            transaction_index: i32::try_from(
                log.transaction_index.expect("transaction-index in log"),
            )
            .expect("transaction index fits"),
            transaction_log_index: i32::try_from(transaction_log_index)
                .expect("transaction log index fits"),
            raw_log: serde_json::to_value(log).unwrap(),
            log_to_jsonb: decoded,
        })
    }
}

pub trait ToLowerHex {
    fn to_lower_hex(&self) -> String;
}

impl ToLowerHex for FixedBytes<32> {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}

pub struct PgLog {
    pub chain_id: ChainId,
    pub block_hash: BlockHash,
    pub height: BlockHeight,
    pub time: OffsetDateTime,
    pub data: PgLogData,
}

#[derive(Serialize, Deserialize)]
pub struct PgLogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: AnyRpcBlock,
}

impl From<BlockInsert> for PgLog {
    fn from(block: BlockInsert) -> Self {
        PgLog {
            chain_id: block.chain_id,
            block_hash: block.hash.clone(),
            height: block.height.try_into().unwrap(),
            time: block.time,
            data: PgLogData {
                header: block.header,
                transactions: block.transactions,
            },
        }
    }
}

pub async fn get_legacy_events(
    logs: impl IntoIterator<Item = PgLog>,
) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
    Ok(logs
        .into_iter()
        .map(|l| SupportedBlockEvent::EthereumLog {
            internal_chain_id: l.chain_id.db,
            block_hash: l.block_hash.clone(),
            data: serde_json::to_value(&l.data).expect("data should be json serializable"),
            height: l.height.into(),
            time: l.time,
        })
        .collect_vec())
}
