use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use alloy::{json_abi::JsonAbi, network::AnyRpcBlock, primitives::Address};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use time::OffsetDateTime;

use crate::{
    indexer::{
        api::{BlockHash, BlockHeight, IndexerError},
        ethereum::{
            block_handle::{BlockInsert, TransactionInsert},
            log_parser::Parser,
        },
        event::SupportedBlockEvent,
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
) -> Result<Vec<SupportedBlockEvent>, IndexerError> {
    Ok(logs
        .into_iter()
        .map(|l| SupportedBlockEvent::EthereumLog {
            internal_chain_id: l.chain_id.db,
            block_hash: l.block_hash.clone(),
            data: serde_json::to_value(&l.data).expect("data should be json serializable"),
            height: l.height,
            time: l.time,
        })
        .collect_vec())
}

pub struct Abi {
    pub definition: String,
}

pub struct AbiRegistration {
    administration: HashMap<Address, Abi>,
}

impl AbiRegistration {
    pub fn addresses(&self) -> Vec<Address> {
        self.administration.keys().cloned().collect_vec()
    }

    pub fn decode(&self, log: &alloy::rpc::types::Log) -> Result<Value, IndexerError> {
        let Some(abi_input) = self.administration.get(&log.address()) else {
            return Err(IndexerError::AbiNoAbiForAddress(log.address()));
        };

        let abi: JsonAbi =
            serde_json::from_str(&abi_input.definition).expect("deserializing json abi failed");
        let parser = Parser::new(&abi);
        let result = parser
            .parse(log)
            .expect("could not parse log into keyed data");
        let json = serde_json::to_value(result).expect("could not convert keyed events to json");

        Ok(json)
    }
}

impl Display for AbiRegistration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let addresses: Vec<String> = self
            .administration
            .keys()
            .map(|addr| addr.to_string())
            .collect();
        write!(f, "{}", addresses.join(", "))
    }
}

pub async fn get_abi_registration(
    tx: &mut Transaction<'_, Postgres>,
    internal_chain_id: i32,
    height: BlockHeight,
) -> sqlx::Result<AbiRegistration> {
    let height: i64 = height.try_into().unwrap();

    let result = sqlx::query!(
        r#"
        SELECT    address, abi
        FROM      v2_evm.contracts
        WHERE     internal_chain_id = $1
        AND       $2 between start_height and end_height
        AND       abi IS NOT NULL
        "#,
        internal_chain_id,
        height,
    )
    .fetch_all(tx.as_mut())
    .await?
    .into_iter()
    .map(|record| {
        (
            record
                .address
                .parse::<Address>()
                .expect("abi can be parsed"),
            Abi {
                definition: record.abi.expect("abi not null"),
            },
        )
    })
    .collect::<HashMap<Address, Abi>>();

    Ok(AbiRegistration {
        administration: result,
    })
}
