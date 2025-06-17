use alloy::{network::AnyRpcBlock, primitives::Address};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::{
        api::{BlockHash, BlockHeight, IndexerError},
        ethereum::block_handle::{BlockInsert, TransactionInsert},
        event::BlockEvent,
    },
    postgres::ChainId,
};

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

pub async fn insert_batch_logs(
    logs: impl IntoIterator<Item = PgLog>,
) -> Result<Vec<BlockEvent>, IndexerError> {
    Ok(logs
        .into_iter()
        .map(|l| BlockEvent::EthereumLog {
            internal_chain_id: l.chain_id.db,
            block_hash: l.block_hash.clone(),
            data: serde_json::to_value(&l.data).expect("data should be json serializable"),
            height: l.height,
            time: l.time,
        })
        .collect_vec())
}

pub async fn active_contracts(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<Vec<Address>> {
    let height: i64 = height.try_into().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT    address
        FROM      v2_evm.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        "#,
        internal_chain_id,
        height,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| record.address.parse().expect("address to be valid"))
    .collect();

    Ok(result)
}
