use tendermint::block::Height;
use tendermint_rpc::{
    error::Error as TendermintRpcError, query::Query, Client as TendermintRpcClient, Order,
};

use crate::race_client::RaceClient;

impl<C: TendermintRpcClient + std::marker::Sync> RaceClient<C> {
    pub async fn status(
        &self,
    ) -> Result<tendermint_rpc::endpoint::status::Response, TendermintRpcError> {
        self.race(|c| c.status()).await
    }

    pub async fn blockchain<H: Into<Height>>(
        &self,
        min: H,
        max: H,
    ) -> Result<tendermint_rpc::endpoint::blockchain::Response, TendermintRpcError> {
        let min = min.into();
        let max = max.into();

        self.race(|c| c.blockchain(min, max)).await
    }

    #[allow(dead_code)]
    pub async fn tx_search(
        &self,
        query: Query,
        prove: bool,
        page: u32,
        per_page: u8,
        order: Order,
    ) -> Result<tendermint_rpc::endpoint::tx_search::Response, TendermintRpcError> {
        self.race(|c| c.tx_search(query.clone(), prove, page, per_page, order.clone()))
            .await
    }

    pub async fn latest_block(
        &self,
    ) -> Result<tendermint_rpc::endpoint::block::Response, TendermintRpcError> {
        self.race(|c| c.latest_block()).await
    }

    pub async fn commit<H: Into<Height>>(
        &self,
        height: H,
    ) -> Result<tendermint_rpc::endpoint::commit::Response, TendermintRpcError> {
        let height = height.into();
        self.race(|c| c.commit(height)).await
    }

    #[allow(dead_code)]
    pub async fn block_results<H: Into<Height>>(
        &self,
        height: H,
    ) -> Result<tendermint_rpc::endpoint::block_results::Response, TendermintRpcError> {
        let height = height.into();
        self.race(|c| c.block_results(height)).await
    }
}
