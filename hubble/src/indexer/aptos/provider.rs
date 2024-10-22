use std::result::Result;

use aptos_rest_client::{
    aptos_api_types::{Block, IndexResponse},
    error::RestError,
    Client, Response, Transaction,
};
use url::Url;

use crate::{indexer::api::BlockHeight, race_client::{RaceClient, RaceClientId, RaceClientResponse}};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<Client>,
}

#[derive(Clone, Debug, Copy)]
pub struct RpcProviderId {
    race_client_id: RaceClientId,
}

impl From<RpcProviderId> for RaceClientId {    
    fn from(value: RpcProviderId) -> Self {
        value.race_client_id
    }
}

#[derive(Debug)]
pub struct RpcResult<T> {
    pub provider_id: RpcProviderId,
    pub response: T,
}

impl<T> RpcResult<T> {
    fn new(race_client_id: RaceClientId, result: T) -> Self {
        Self {
            provider_id: RpcProviderId {
                race_client_id,
            },
            response: result,
        }
    }
}

impl<T> From<RaceClientResponse<T>> for RpcResult<T> {
    fn from(value: RaceClientResponse<T>) -> Self {
        RpcResult::new(value.race_client_id, value.response)
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

    // RPC
    pub async fn get_index(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<IndexResponse>>, RestError> {
        self.rpc_client.race(provider_id.map(Into::into), |c| c.get_index()).await.map(Into::into)
    }

    pub async fn get_block_by_height(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Block>>, RestError> {
        self.rpc_client.race( provider_id.map(Into::into), |c| c.get_block_by_height(height, false)).await.map(Into::into)
    }

    pub async fn get_transactions(
        &self,
        start: BlockHeight,
        limit: u16,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Vec<Transaction>>>, RestError> {
        self.rpc_client.race(
            provider_id.map(Into::into), 
            |c| c.get_transactions(Some(start), Some(limit))
        ).await.map(Into::into)
    }

    pub async fn get_transaction_by_version(
        &self,
        version: u64,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Response<Transaction>>, RestError> {
        self.rpc_client.race(
            provider_id.map(Into::into), 
            |c| c.get_transaction_by_version(version)
        ).await.map(Into::into)
    }
}
