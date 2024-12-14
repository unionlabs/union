use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroU8},
    result::Result,
};

use color_eyre::eyre::{eyre, Report};
use cometbft_rpc::{
    rpc_types::{
        BlockResponse, BlockResultsResponse, BlockchainResponse, Order, StatusResponse,
        TxSearchResponse,
    },
    Client, JsonRpcError,
};
use futures::future;
use protos::ibc::{
    core::client::v1::{QueryClientStateRequest, QueryClientStateResponse},
    lightclients::wasm::v1::{QueryCodeRequest, QueryCodeResponse},
};
use tonic::Response;
use tracing::{debug, error};
use url::Url;

use crate::{
    indexer::api::IndexerError,
    race_client::{RaceClient, RaceClientId, RaceClientResponse},
};

#[derive(Clone, Debug)]
pub struct Provider {
    rpc_client: RaceClient<Client>,
    grpc_client: RaceClient<GrpcClient>,
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

#[derive(Clone, Debug, Copy)]
pub struct GrpcProviderId {
    race_client_id: RaceClientId,
}

impl From<GrpcProviderId> for RaceClientId {
    fn from(value: GrpcProviderId) -> Self {
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
            provider_id: RpcProviderId { race_client_id },
            response: result,
        }
    }
}

impl<T> From<RaceClientResponse<T>> for RpcResult<T> {
    fn from(value: RaceClientResponse<T>) -> Self {
        RpcResult::new(value.race_client_id, value.response)
    }
}

#[derive(Debug)]
pub struct GrpcResult<T> {
    pub provider_id: GrpcProviderId,
    pub response: T,
}

impl<T> GrpcResult<T> {
    fn new(race_client_id: RaceClientId, result: T) -> Self {
        Self {
            provider_id: GrpcProviderId { race_client_id },
            response: result,
        }
    }
}

impl<T> From<RaceClientResponse<T>> for GrpcResult<T> {
    fn from(value: RaceClientResponse<T>) -> Self {
        GrpcResult::new(value.race_client_id, value.response)
    }
}

impl Provider {
    pub async fn new(rpc_urls: Vec<Url>, grpc_urls: Vec<Url>) -> Result<Self, IndexerError> {
        let rpc_clients = future::join_all(
            rpc_urls
                .into_iter()
                .map(|url| Client::new(url.as_str().to_owned())),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            rpc_client: RaceClient::new(rpc_clients),
            grpc_client: RaceClient::new(grpc_urls.into_iter().map(GrpcClient::new).collect()),
        })
    }

    /// Fetches status information from the RPC client.
    pub async fn status(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<StatusResponse>, JsonRpcError> {
        self.execute_rpc(provider_id, |c| c.status(), "status").await
    }

    /// Fetches blockchain data within a block range.
    pub async fn blockchain(
        &self,
        min_inclusive: BlockHeight,
        max_inclusive: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockchainResponse>, JsonRpcError> {
        self.execute_rpc(provider_id, |c| {
            c.blockchain(
                NonZeroU64::try_from(min_inclusive).map_err(IndexerError::from)?,
                NonZeroU64::try_from(max_inclusive).map_err(IndexerError::from)?,
            )
        }, "blockchain").await
    }

    /// Fetches the latest block.
    pub async fn latest_block(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResponse>, JsonRpcError> {
        self.execute_rpc(provider_id, |c| c.block(None), "latest_block").await
    }

    /// Fetches a specific block by height.
    pub async fn block(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResponse>, JsonRpcError> {
        self.execute_rpc(provider_id, |c| {
            c.block(Some(NonZeroU64::try_from(height).map_err(IndexerError::from)?))
        }, "block").await
    }

    /// Fetches results for a specific block.
    pub async fn block_results(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResultsResponse>, JsonRpcError> {
        self.execute_rpc(provider_id, |c| {
            c.block_results(Some(NonZeroU64::try_from(height).map_err(IndexerError::from)?))
        }, "block_results").await
    }

    /// Generic helper for executing RPC calls.
    async fn execute_rpc<F, T>(
        &self,
        provider_id: Option<RpcProviderId>,
        action: F,
        action_name: &str,
    ) -> Result<RpcResult<T>, JsonRpcError>
    where
        F: FnOnce(&Client) -> T + Send,
        T: std::future::Future<Output = Result<T::Output, JsonRpcError>> + Send,
        T::Output: Send,
    {
        self.rpc_client
            .race(provider_id.map(Into::into), action)
            .await
            .map(Into::into)
            .map_err(|err| {
                error!(action = action_name, error = ?err, "RPC call failed");
                err
            })
    }
}

/// gRPC client for Tendermint-compatible networks.
#[derive(Clone, Debug)]
pub struct GrpcClient {
    pub url: Url,
}

impl GrpcClient {
    fn new(url: Url) -> Self {
        Self { url }
    }

    async fn client_state(
        &self,
        request: QueryClientStateRequest,
    ) -> Result<Response<QueryClientStateResponse>, IndexerError> {
        let mut query_client =
            protos::ibc::core::client::v1::query_client::QueryClient::connect(self.url.to_string())
                .await?;

        query_client
            .client_state(request)
            .await
            .map_err(IndexerError::from)
    }

    async fn code(
        &self,
        request: QueryCodeRequest,
    ) -> Result<Response<QueryCodeResponse>, IndexerError> {
        let mut query_client =
            protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
                self.url.to_string(),
            )
            .await?;

        query_client.code(request).await.map_err(IndexerError::from)
    }
}
