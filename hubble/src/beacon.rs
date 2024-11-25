use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use block::Block;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use reqwest::StatusCode;
use tracing::{debug, info, trace};

use crate::consensus::{Indexer, Querier};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub urls: Vec<url::Url>,
    pub chain_id: String,
    pub start_height: Option<i64>,
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

        let querier = Beacon::new(self.urls[0].clone(), reqwest::Client::new());

        Ok(Indexer::new(chain_id, db, querier, self.start_height))
    }
}

pub struct Beacon {
    url: url::Url,
    client: reqwest::Client,
}

pub enum BlockNumber {
    Head,
    Number(i64),
}

impl Beacon {
    pub fn new(url: url::Url, client: reqwest::Client) -> Self {
        Self { url, client }
    }

    async fn get_block_at_optional(&self, height: BlockNumber) -> Result<Option<Block>> {
        let path = match height {
            BlockNumber::Head => String::from("eth/v2/beacon/blocks/head"),
            BlockNumber::Number(height) => format!("eth/v2/beacon/blocks/{height}"),
        };
        let url = &self.url;
        let url = format!("{url}{path}");

        trace!("get block request: {url}");
        let response = self.client.clone().get(&url).send().await?;
        let response_status = response.status();
        trace!("get block response status: {response_status}");

        if let StatusCode::NOT_FOUND = response_status {
            return Ok(None);
        }

        let val: serde_json::Value = response.json().await?;
        trace!("get block body: {val}");

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

    pub async fn get_height_at_skip_missing(&self, height: i64) -> Result<Block> {
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
                        debug!("slot is greater than height, continuing");
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

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Block {
        pub data: Data,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Data {
        pub message: Message,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Message {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        pub slot: i64,
        pub body: Body,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Body {
        #[serde(rename = "execution_payload")]
        pub execution_payload: ExecutionPayload,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ExecutionPayload {
        #[serde(
            deserialize_with = "deserialize_number_from_string",
            rename = "block_number"
        )]
        pub block_number: i64,
    }
}
