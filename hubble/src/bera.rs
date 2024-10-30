use std::time::Duration;

use backon::{ConstantBuilder, ExponentialBuilder, Retryable};
use beacon_api_types::{
    execution_payload_header::ExecutionPayloadHeader, ExecutionPayloadHeaderSsz, Mainnet,
};
use color_eyre::{eyre::eyre, Result};
use cometbft_rpc::{types::AbciQueryResponse, Client};
use tracing::info;
use unionlabs::encoding::DecodeAs;

use crate::consensus::{Indexer, Querier};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub label: String,
    pub urls: Vec<url::Url>,
    pub chain_id: String,
    pub start_height: Option<i64>,
}

impl Config {
    pub async fn indexer(self, db: sqlx::PgPool) -> Result<Indexer<Bera>> {
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

        let client = Client::new(self.urls[0].as_str()).await?;
        let querier = Bera::new(client);

        Ok(Indexer::new(chain_id, db, querier, self.start_height))
    }
}

pub struct Bera {
    client: Client,
}

impl Bera {
    fn new(client: Client) -> Self {
        Self { client }
    }

    /// Perform an abci query on the `beacon` store ("store/beacon/key").
    async fn beacon_store_abci_query(
        &self,
        data: impl AsRef<[u8]>,
        slot: u64,
        prove: bool,
    ) -> Result<AbciQueryResponse> {
        let resp = self
            .client
            .abci_query(
                "store/beacon/key",
                data,
                Some(
                    (slot as i64 - 1)
                        .try_into()
                        .expect("converting slot to abci_query slot"),
                ),
                prove,
            )
            .await?;
        Ok(resp)
    }

    /// A thin wrapper around abci_query to fetch the latest execution payload header.
    pub async fn execution_header_at_beacon_slot(
        &self,
        slot: u64,
    ) -> Result<ExecutionPayloadHeader> {
        use unionlabs::encoding::Ssz;

        // https://github.com/unionlabs/union/blob/2ce63ba3e94b13444d69ac03995958dc74b8f8c9/lib/unionlabs/src/berachain.rs#L9
        pub const LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX: u8 = 17;

        let header = ExecutionPayloadHeaderSsz::<Mainnet>::decode_as::<Ssz>(
            &self
                .beacon_store_abci_query([LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX], slot, false)
                .await?
                .response
                .value
                .unwrap(),
        )?;

        Ok(header.into())
    }
}

impl Querier for Bera {
    async fn get_execution_height(&self, slot: i64) -> Result<(i64, i64)> {
        let height = (|| self.execution_header_at_beacon_slot(slot as u64))
            .retry(
                &ConstantBuilder::default()
                    .with_delay(Duration::from_millis(500))
                    .with_max_times(60),
            )
            .await?
            .block_number as i64;
        Ok((slot, height))
    }
}
