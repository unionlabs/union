use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use block::Block;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use tracing::{debug, info};

use crate::consensus::{Indexer, Querier};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub url: url::Url,
    pub chain_id: String,
}

impl Config {
    pub async fn indexer(self, db: sqlx::PgPool) -> Result<Indexer<Beacon>> {
        info!("fetching db chain_id for chain {}", &self.chain_id);
        let chain_id = (|| async {
            let chain_id = crate::postgres::get_chain_id(&db, self.chain_id.clone())
                .await?
                // This can reasonably fail because the other indexer is creating the chain_id. Otherwise
                // this should always succeed.
                .ok_or(eyre!("chain not found"))?;
            Ok::<_, color_eyre::Report>(chain_id)
        })
        .retry(&ExponentialBuilder::default())
        .await?;

        let querier = Beacon::new(self.url, reqwest::Client::new());

        Ok(Indexer::new(chain_id, db, querier))
    }
}

pub struct Beacon {
    url: url::Url,
    client: reqwest::Client,
}

enum BlockNumber {
    Head,
    Number(i64),
}

impl Beacon {
    fn new(url: url::Url, client: reqwest::Client) -> Self {
        Self { url, client }
    }

    async fn get_block_at_optional(&self, height: BlockNumber) -> Result<Option<Block>> {
        let path = match height {
            BlockNumber::Head => String::from("eth/v2/beacon/blocks/head"),
            BlockNumber::Number(height) => format!("eth/v2/beacon/blocks/{height}"),
        };
        let url = &self.url;
        let url = format!("{url}{path}");

        let val: serde_json::Value = self.client.clone().get(&url).send().await?.json().await?;

        if val.get("statusCode").is_none() {
            Ok(Some(serde_json::from_value(val).unwrap()))
        } else {
            if val.get("error")
                == Some(serde_json::Value::String(String::from("Not Found"))).as_ref()
            {
                return Ok(None);
            }
            bail!("unrecognized response: {}", val)
        }
    }

    async fn get_height_at_skip_missing(&self, height: i64) -> Result<Block> {
        let mut height = height;
        loop {
            debug!("getting consensus block for height {}", height);
            match self
                .get_block_at_optional(BlockNumber::Number(height))
                .await?
            {
                Some(block) => return Ok(block),
                None => {
                    let head = self
                        .get_block_at_optional(BlockNumber::Head)
                        .await?
                        .expect("head should exist");
                    let slot = head.data.message.slot;

                    // The requested slot is missing, and the current one is the next, hence we can immediately return.
                    // Note that slot == height is valid, because the block might be produced since our last get_block_at_optional
                    // call.
                    if (slot == height + 1) || (slot == height) {
                        debug!("found block at {slot} for requested consensus height {height}");
                        return Ok(head);
                    }

                    // We are asking for a block in the future and need to retry later until we have caught up.
                    if slot < height {
                        debug!("requested height is ahead of slot, sleeping.");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }

                    // We are asking for a height that is in the past, yet missing. Thus the slot was skipped
                    // and we need to ask for the next one.
                    if slot > height {
                        debug!("slot is greater than hight, continuing");
                        height += 1;
                        continue;
                    };
                    unreachable!("slot is not smaller, not equal and not great than requested height. Impossible")
                }
            }
        }
    }
}

impl Querier for Beacon {
    async fn get_execution_height(&self, height: i64) -> Result<(i64, i64)> {
        let block = (|| self.get_height_at_skip_missing(height))
            .retry(&ExponentialBuilder::default())
            .await?;
        Ok((
            block.data.message.slot,
            block.data.message.body.execution_payload.block_number,
        ))
    }
}

mod block {
    use serde::{Deserialize, Serialize};
    use serde_aux::prelude::*;
    use serde_json::Value;

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Block {
        pub data: Data,
        pub version: String,
        #[serde(rename = "execution_optimistic")]
        pub execution_optimistic: bool,
        pub finalized: bool,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub message: Message,
        pub signature: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Message {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        pub slot: i64,
        #[serde(rename = "proposer_index")]
        pub proposer_index: String,
        #[serde(rename = "parent_root")]
        pub parent_root: String,
        #[serde(rename = "state_root")]
        pub state_root: String,
        pub body: Body,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Body {
        #[serde(rename = "randao_reveal")]
        pub randao_reveal: String,
        #[serde(rename = "eth1_data")]
        pub eth1_data: Eth1Data,
        pub graffiti: String,
        #[serde(rename = "proposer_slashings")]
        pub proposer_slashings: Vec<Value>,
        #[serde(rename = "attester_slashings")]
        pub attester_slashings: Vec<Value>,
        pub attestations: Vec<Attestation>,
        pub deposits: Vec<Value>,
        #[serde(rename = "voluntary_exits")]
        pub voluntary_exits: Vec<Value>,
        #[serde(rename = "sync_aggregate")]
        pub sync_aggregate: SyncAggregate,
        #[serde(rename = "execution_payload")]
        pub execution_payload: ExecutionPayload,
        #[serde(rename = "bls_to_execution_changes")]
        pub bls_to_execution_changes: Vec<Value>,
        #[serde(rename = "blob_kzg_commitments")]
        pub blob_kzg_commitments: Vec<String>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Eth1Data {
        #[serde(rename = "deposit_root")]
        pub deposit_root: String,
        #[serde(rename = "deposit_count")]
        pub deposit_count: String,
        #[serde(rename = "block_hash")]
        pub block_hash: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attestation {
        #[serde(rename = "aggregation_bits")]
        pub aggregation_bits: String,
        pub data: Data2,
        pub signature: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data2 {
        pub slot: String,
        pub index: String,
        #[serde(rename = "beacon_block_root")]
        pub beacon_block_root: String,
        pub source: Source,
        pub target: Target,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Source {
        pub epoch: String,
        pub root: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Target {
        pub epoch: String,
        pub root: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SyncAggregate {
        #[serde(rename = "sync_committee_bits")]
        pub sync_committee_bits: String,
        #[serde(rename = "sync_committee_signature")]
        pub sync_committee_signature: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExecutionPayload {
        #[serde(rename = "parent_hash")]
        pub parent_hash: String,
        #[serde(rename = "fee_recipient")]
        pub fee_recipient: String,
        #[serde(rename = "state_root")]
        pub state_root: String,
        #[serde(rename = "receipts_root")]
        pub receipts_root: String,
        #[serde(rename = "logs_bloom")]
        pub logs_bloom: String,
        #[serde(rename = "prev_randao")]
        pub prev_randao: String,
        #[serde(
            deserialize_with = "deserialize_number_from_string",
            rename = "block_number"
        )]
        pub block_number: i64,
        #[serde(rename = "gas_limit")]
        pub gas_limit: String,
        #[serde(rename = "gas_used")]
        pub gas_used: String,
        pub timestamp: String,
        #[serde(rename = "extra_data")]
        pub extra_data: String,
        #[serde(rename = "base_fee_per_gas")]
        pub base_fee_per_gas: String,
        #[serde(rename = "block_hash")]
        pub block_hash: String,
        pub transactions: Vec<String>,
        pub withdrawals: Vec<Withdrawal>,
        #[serde(rename = "blob_gas_used")]
        pub blob_gas_used: String,
        #[serde(rename = "excess_blob_gas")]
        pub excess_blob_gas: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Withdrawal {
        pub index: String,
        #[serde(rename = "validator_index")]
        pub validator_index: String,
        pub address: String,
        pub amount: String,
    }
}
