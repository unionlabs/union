use std::time::Duration;

use alloy::{
    providers::{Provider, ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};
use backon::{ConstantBuilder, ExponentialBuilder, Retryable};
use color_eyre::eyre::{ContextCompat, Result, WrapErr};
use scroll_api::ScrollClient;
use tracing::{debug, info};
use unionlabs::{hash::H160, uint::U256};

use crate::{
    beacon::Beacon,
    consensus::{Indexer, Querier},
};

pub struct Scroll {
    pub l1_client: RootProvider<Http<Client>>,
    #[allow(unused)]
    pub l2_client: RootProvider<Http<Client>>,

    pub beacon: Beacon,

    pub scroll_api_client: ScrollClient,

    pub rollup_finalization_config: RollupFinalizationConfig,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,

    pub l1_url: url::Url,
    pub l2_url: url::Url,

    pub beacon_url: url::Url,

    pub scroll_api_url: url::Url,

    pub rollup_finalization_config: RollupFinalizationConfig,

    pub start_height: Option<i64>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct RollupFinalizationConfig {
    pub rollup_contract_address: H160,
    pub rollup_last_finalized_batch_index_slot: U256,
}

impl Config {
    pub async fn indexer(self, db: sqlx::PgPool) -> Result<Indexer<Scroll>> {
        let l2_client = ProviderBuilder::new().on_http(self.l2_url);

        let l2_chain_id = U256::from(
            l2_client
                .get_chain_id()
                .await
                .wrap_err("unable to fetch chain id from l2")?,
        )
        .to_string();

        info!("fetching db chain_id for chain {}", l2_chain_id);

        let chain_id = (|| async {
            let chain_id = crate::postgres::get_chain_id(&db, l2_chain_id.clone())
                .await?
                // This can reasonably fail because the other indexer is creating the chain_id. Otherwise
                // this should always succeed.
                .wrap_err("chain not found")?;
            Ok::<_, color_eyre::Report>(chain_id)
        })
        .retry(&ExponentialBuilder::default())
        .await?;

        let querier = Scroll {
            l1_client: ProviderBuilder::new().on_http(self.l1_url),
            l2_client,

            beacon: Beacon::new(self.beacon_url, reqwest::Client::new()),

            scroll_api_client: ScrollClient::new(self.scroll_api_url),

            rollup_finalization_config: self.rollup_finalization_config,
        };

        Ok(Indexer::new(chain_id, db, querier, self.start_height))
    }
}

impl Scroll {
    // NOTE: Copied from chain_utils
    async fn execution_height_of_beacon_slot(&self, slot: u64) -> Result<u64> {
        Ok(self
            .scroll_height_of_batch_index(self.batch_index_of_beacon_slot(slot).await?)
            .await)
    }

    pub async fn batch_index_of_beacon_slot(&self, slot: u64) -> Result<u64> {
        let l1_height = self
            .beacon
            .get_height_at_skip_missing(slot.try_into().expect("negative slot?"))
            .await?
            .data
            .message
            .body
            .execution_payload
            .block_number;

        let storage = self
            .l1_client
            .get_storage_at(
                alloy::primitives::Address::new(
                    (self.rollup_finalization_config.rollup_contract_address).0,
                ),
                alloy::primitives::Uint::from_be_bytes(
                    self.rollup_finalization_config
                        .rollup_last_finalized_batch_index_slot
                        .to_be_bytes(),
                ),
            )
            .await
            .wrap_err("error fetching l1 rollup contract storage")?;
        let batch_index: u64 = storage
            .try_into()
            .expect("value is a u64 in the contract; qed;");

        debug!("execution height {l1_height} is batch index {batch_index}");

        Ok(batch_index)
    }

    pub async fn scroll_height_of_batch_index(&self, batch_index: u64) -> u64 {
        let batch = self.scroll_api_client.batch(batch_index).await.batch;

        debug!(
            "batch index {batch_index} is scroll height range {}..={}",
            batch.start_block_number, batch.end_block_number
        );

        batch.end_block_number
    }
}

impl Querier for Scroll {
    async fn get_execution_height(&self, slot: i64) -> Result<(i64, i64)> {
        let height = (|| self.execution_height_of_beacon_slot(slot as u64))
            .retry(
                &ConstantBuilder::default()
                    .with_delay(Duration::from_millis(500))
                    .with_max_times(60),
            )
            .await?;
        Ok((slot, height as i64))
    }
}
