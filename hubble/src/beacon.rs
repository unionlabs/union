use std::time::Duration;

use backon::{ConstantBuilder, ExponentialBuilder, Retryable};
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};
use tracing::{debug, info};

use crate::consensus::{Indexer, Querier};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    url: url::Url,
    chain_id: String,
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

impl Beacon {
    fn new(url: url::Url, client: reqwest::Client) -> Self {
        Self { url, client }
    }
}

impl Querier for Beacon {
    async fn get_execution_height(&self, height: i64) -> Result<i64> {
        let mut tries = 0;
        let max_tries = 15;

        loop {
            let path = format!("eth/v2/beacon/blocks/{height}");
            let url = &self.url;
            let url = format!("{url}{path}");
            let client = &self.client;
            debug!("fetching execution height for block: {}", height);
            let val: serde_json::Value = (|| client.clone().get(&url).send())
                .retry(
                    &ConstantBuilder::default()
                        .with_delay(Duration::from_millis(500))
                        .with_max_times(90),
                )
                .await?
                .json()
                .await?;
            // Equivalent of https://github.com/unionlabs/union/blob/84a7e5c3f3fbcc741369e398a7066c374c497a4d/lib/beacon-api/src/client.rs#L111 without
            // parsing into a struct.

            let block_number = if let Some(Some(Some(Some(Some(block_number))))) =
                val.get("data").map(|m| {
                    m.get("message").map(|b| {
                        b.get("body")
                            .map(|e| e.get("execution_payload").map(|b| b.get("block_number")))
                    })
                }) {
                block_number.as_str().unwrap()
            } else {
                tokio::time::sleep(Duration::from_secs(3)).await;
                tries += 1;
                if tries > max_tries {
                    bail!(
                        "could not find execution height after {max_tries}. Got message {:?}",
                        val
                    )
                }
                continue;
            };

            let height = block_number
                .parse()
                .expect("beacon node should return valid numbers");
            return Ok(height);
        }
    }
}
