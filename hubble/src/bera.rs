use backon::{ExponentialBuilder, Retryable};
use color_eyre::{eyre::eyre, Result};
use cometbft_rpc::{AbciQueryResponse, Client};
use tracing::info;
use unionlabs::{
    berachain::BerachainChainSpec, encoding::DecodeAs,
    ibc::lightclients::ethereum::execution_payload_header::ExecutionPayloadHeader,
};

use crate::consensus::{Indexer, Querier};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    url: url::Url,
    chain_id: String,
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

        let client = Client::new(self.url.as_str()).await?;
        let querier = Bera::new(client);

        Ok(Indexer::new(chain_id, db, querier))
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
                Some((slot - 1).try_into().unwrap()),
                prove,
            )
            .await?;
        Ok(resp)
    }

    /// A thin wrapper around abci_query to fetch the latest execution payload header.
    pub async fn execution_header_at_beacon_slot(
        &self,
        slot: u64,
    ) -> Result<ExecutionPayloadHeader<BerachainChainSpec>> {
        use unionlabs::{berachain::LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX, encoding::Ssz};

        let header = ExecutionPayloadHeader::<BerachainChainSpec>::decode_as::<Ssz>(
            &self
                .beacon_store_abci_query([LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX], slot, false)
                .await?
                .response
                .value,
        )?;
        Ok(header)
    }
}

impl Querier for Bera {
    async fn get_execution_height(&self, height: i64) -> Result<i64> {
        let height = (|| self.execution_header_at_beacon_slot(height as u64))
            .retry(&ExponentialBuilder::default())
            .await?
            .block_number as i64;
        Ok(height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test to verify that the querier works. Will check that we correctly fetch
    // the slot etc.
    #[tokio::test]
    async fn test_querier_works() {
        let client = Client::new("wss://bartio-cosmos.berachain-devnet.com/websocket")
            .await
            .expect("instantiating client should work");
        let querier = Bera::new(client);
        let height = querier
            .get_execution_height(100)
            .await
            .expect("getting execution height should work");
        assert_eq!(height, 69);
    }
}
