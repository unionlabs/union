use backon::{ExponentialBuilder, Retryable};
use color_eyre::{eyre::eyre, Result};
use futures::{stream::FuturesOrdered, TryStreamExt};
use tracing::{debug, info};

use crate::postgres::ChainId;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    url: url::Url,
    chain_id: String,
}

impl Config {
    pub async fn indexer(self, db: sqlx::PgPool) -> Result<Indexer> {
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

        Ok(Indexer {
            url: self.url,
            chain_id,
            client: reqwest::Client::new(),
            pool: db,
        })
    }
}

pub struct Indexer {
    url: url::Url,
    chain_id: ChainId,
    client: reqwest::Client,
    pool: sqlx::PgPool,
}

impl Indexer {
    pub async fn index(&self) -> Result<()> {
        loop {
            info!("fixing next batch of unmapped_execution_heights");
            debug!("getting unmapped consensus heights");
            let consensus_heights =
                crate::postgres::get_batch_of_unmapped_execution_heights(&self.pool, self.chain_id)
                    .await?;

            if consensus_heights.is_empty() {
                debug!("no unmapped heights found, sleeping");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
            }

            let futures = FuturesOrdered::from_iter(
                consensus_heights
                    .iter()
                    .map(|height| get_execution_height(&self.client, &self.url, *height)),
            );

            debug!("getting execution heights");
            let execution_heights: Vec<i64> = futures.try_collect().await?;

            debug!("inserting execution heights");
            crate::postgres::insert_mapped_execution_heights(
                &self.pool,
                execution_heights,
                consensus_heights,
                self.chain_id,
            )
            .await?;
        }
    }
}

/// Fetch the execution height from a given beacon block height.
async fn get_execution_height(
    client: &reqwest::Client,
    url: &url::Url,
    height: i64,
) -> Result<i64> {
    let path = format!("eth/v2/beacon/blocks/{height}");
    let url = format!("{url}{path}");
    debug!("fetching execution height for block: {}", height);
    let val: serde_json::Value = (|| client.get(&url).send())
        .retry(&ExponentialBuilder::default())
        .await?
        .json()
        .await?;
    // Equivalent of https://github.com/unionlabs/union/blob/84a7e5c3f3fbcc741369e398a7066c374c497a4d/lib/beacon-api/src/client.rs#L111 without
    // parsing into a struct.
    let height = (val["data"]["message"]["body"]["execution_payload"]["block_number"]
        .as_str()
        .unwrap())
    .parse()
    .expect("beacon node should return valid numbers");
    Ok(height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_execution_height() {
        let client = reqwest::Client::new();
        get_execution_height(
            &client,
            &"https://lodestar-sepolia.chainsafe.io/".try_into().unwrap(),
            500000,
        )
        .await
        .unwrap();
    }
}
