use std::result::Result;

use aptos_rest_client::{
    aptos_api_types::{Block, IndexResponse},
    error::RestError,
    Client, Response, Transaction,
};
use url::Url;

use crate::{indexer::api::BlockHeight, race_client::RaceClient};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<Client>,
}

#[derive(Clone, Debug, Copy)]
pub struct RpcProviderId {
    index: usize,
}

#[derive(Debug)]
pub struct RpcResult<T> {
    pub provider_id: RpcProviderId,
    pub response: T,
}

impl<T> RpcResult<T> {
    fn new(provider_index: usize, result: T) -> Self {
        Self {
            provider_id: RpcProviderId {
                index: provider_index,
            },
            response: result,
        }
    }
}

impl Provider {
    pub fn new(rpc_urls: Vec<Url>) -> Self {
        Self {
            rpc_client: RaceClient::new(
                rpc_urls
                    .into_iter()
                    .map(aptos_rest_client::Client::new)
                    .collect(),
            ),
        }
    }

    fn rpc_client(&self, provider_id: Option<RpcProviderId>) -> RaceClient<Client> {
        Self::select_client(self.rpc_client.clone(), provider_id.map(|id| id.index))
    }

    fn select_client<T: Clone>(
        client: RaceClient<T>,
        provider_index: Option<usize>,
    ) -> RaceClient<T> {
        match provider_index {
            Some(provider_index) => RaceClient::new(vec![client.clients[provider_index].clone()]),
            None => client,
        }
    }

    // RPC
    pub async fn get_index(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<IndexResponse>>, RestError> {
        let result = self.rpc_client(provider_id).get_index().await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_block_by_height(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Block>>, RestError> {
        let result = self
            .rpc_client(provider_id)
            .get_block_by_height(height)
            .await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_transactions(
        &self,
        start: BlockHeight,
        limit: u16,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Vec<Transaction>>>, RestError> {
        let result = self
            .rpc_client(provider_id)
            .get_transactions(start, limit)
            .await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_transaction_by_version(
        &self,
        version: u64,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Transaction>>, RestError> {
        let result = self
            .rpc_client(provider_id)
            .get_transaction_by_version(version)
            .await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }
}

impl RaceClient<Client> {
    pub async fn get_index(&self) -> Result<Response<IndexResponse>, RestError> {
        self.race(|c| c.get_index()).await
    }

    pub async fn get_block_by_height(
        &self,
        height: BlockHeight,
    ) -> Result<Response<Block>, RestError> {
        self.race(|c| c.get_block_by_height(height, false)).await
    }

    pub async fn get_transactions(
        &self,
        start: BlockHeight,
        limit: u16,
    ) -> Result<Response<Vec<Transaction>>, RestError> {
        self.race(|c| c.get_transactions(Some(start), Some(limit)))
            .await
    }

    pub async fn get_transaction_by_version(
        &self,
        version: u64,
    ) -> Result<Response<Transaction>, RestError> {
        self.race(|c| c.get_transaction_by_version(version)).await
    }
}
