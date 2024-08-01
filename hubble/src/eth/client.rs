#![allow(clippy::disallowed_types)]
use ethers::{
    core::types::BlockId,
    providers::{Http, Middleware, Provider, ProviderError},
    types::{Block, BlockNumber, Filter, Log, TransactionReceipt, TxHash, U256},
};

use crate::race_client::RaceClient;

impl RaceClient<Provider<Http>> {
    pub async fn get_chainid(&self) -> Result<U256, ProviderError> {
        self.race(|c| c.get_chainid()).await
    }

    #[allow(dead_code)]
    pub async fn get_block_receipts<T>(
        &self,
        block: T,
    ) -> Result<Vec<TransactionReceipt>, ProviderError>
    where
        T: Into<BlockNumber> + Send + Sync,
    {
        let block = block.into();
        self.race(|c| c.get_block_receipts(block)).await
    }

    pub async fn get_block<T>(&self, id: T) -> Result<Option<Block<TxHash>>, ProviderError>
    where
        T: Into<BlockId> + Send + Sync,
    {
        let id = id.into();
        self.race_some(|c| c.get_block(id)).await
    }

    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>, ProviderError> {
        self.race(|c| c.get_logs(filter)).await
    }
}
