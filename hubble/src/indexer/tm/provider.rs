use std::result::Result;

use color_eyre::eyre::Report;
use protos::ibc::{
    core::client::v1::{QueryClientStateRequest, QueryClientStateResponse},
    lightclients::wasm::v1::{QueryCodeRequest, QueryCodeResponse},
};
use tendermint_rpc::{query::Query, HttpClient, Order};
use tonic::Response;
use unionlabs::aptos::block_info::BlockHeight;
use url::Url;

use crate::{indexer::api::IndexerError, race_client::RaceClient};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<HttpClient>,
    pub grpc_client: RaceClient<GrpcClient>,
}

#[derive(Clone, Debug, Copy)]
pub struct RpcProviderId {
    index: usize,
}

#[derive(Clone, Debug, Copy)]
pub struct GrpcProviderId {
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

#[derive(Debug)]
pub struct GrpcResult<T> {
    pub provider_id: GrpcProviderId,
    pub response: T,
}

impl<T> GrpcResult<T> {
    fn new(provider_index: usize, result: T) -> Self {
        Self {
            provider_id: GrpcProviderId {
                index: provider_index,
            },
            response: result,
        }
    }
}

impl Provider {
    pub fn new(rpc_urls: Vec<Url>, grpc_urls: Vec<Url>) -> Self {
        Self {
            rpc_client: RaceClient::new(
                rpc_urls
                    .into_iter()
                    .map(|rpc_url| {
                        HttpClient::new(rpc_url.as_str()).expect("rpc-client can be created")
                    })
                    .collect(),
            ),
            grpc_client: RaceClient::new(grpc_urls.into_iter().map(GrpcClient::new).collect()),
        }
    }

    // RPC
    pub async fn status(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<tendermint_rpc::endpoint::status::Response>, tendermint_rpc::error::Error>
    {
        let result = self.rpc_client(provider_id).status().await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn blockchain(
        &self,
        min_inclusive: BlockHeight,
        max_inclusive: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        RpcResult<tendermint_rpc::endpoint::blockchain::Response>,
        tendermint_rpc::error::Error,
    > {
        let result = self
            .rpc_client(provider_id)
            .blockchain(min_inclusive as u32, max_inclusive as u32)
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

    pub async fn latest_block(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<tendermint_rpc::endpoint::block::Response>, tendermint_rpc::error::Error>
    {
        let result = self.rpc_client(provider_id).latest_block().await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn commit(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<tendermint_rpc::endpoint::commit::Response>, tendermint_rpc::error::Error>
    {
        let result = self.rpc_client(provider_id).commit(height as u32).await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn block_results(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        RpcResult<tendermint_rpc::endpoint::block_results::Response>,
        tendermint_rpc::error::Error,
    > {
        let result = self
            .rpc_client(provider_id)
            .block_results(height as u32)
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

    pub async fn tx_search(
        &self,
        query: Query,
        prove: bool,
        page: u32,
        per_page: u8,
        order: Order,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        RpcResult<tendermint_rpc::endpoint::tx_search::Response>,
        tendermint_rpc::error::Error,
    > {
        let result = self
            .rpc_client(provider_id)
            .tx_search(query, prove, page, per_page, order)
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

    // GRPC
    pub async fn client_state(
        &self,
        request: QueryClientStateRequest,
        provider_id: Option<GrpcProviderId>,
    ) -> Result<GrpcResult<Response<QueryClientStateResponse>>, IndexerError> {
        let result = self.grpc_client(provider_id).client_state(request).await?;

        // TODO: improve race client to return index with result
        Ok(GrpcResult::new(
            provider_id.map_or_else(
                || self.grpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn code(
        &self,
        request: QueryCodeRequest,
        provider_id: Option<GrpcProviderId>,
    ) -> Result<GrpcResult<Response<QueryCodeResponse>>, IndexerError> {
        let result = self.grpc_client(provider_id).code(request).await?;

        // TODO: improve race client to return index with result
        Ok(GrpcResult::new(
            provider_id.map_or_else(
                || self.grpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    fn rpc_client(&self, provider_id: Option<RpcProviderId>) -> RaceClient<HttpClient> {
        Self::select_client(self.rpc_client.clone(), provider_id.map(|id| id.index))
    }

    fn grpc_client(&self, provider_id: Option<GrpcProviderId>) -> RaceClient<GrpcClient> {
        Self::select_client(self.grpc_client.clone(), provider_id.map(|id| id.index))
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
}

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
        let mut query_client = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.url.to_string().clone(),
        )
        .await?;

        query_client
            .client_state(request.clone())
            .await
            .map_err(IndexerError::from)
    }

    async fn code(
        &self,
        request: QueryCodeRequest,
    ) -> Result<Response<QueryCodeResponse>, IndexerError> {
        let mut query_client =
            protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
                self.url.to_string().clone(),
            )
            .await?;

        query_client
            .code(request.clone())
            .await
            .map_err(IndexerError::from)
    }
}

impl RaceClient<GrpcClient> {
    pub async fn client_state(
        &self,
        request: QueryClientStateRequest,
    ) -> Result<Response<QueryClientStateResponse>, IndexerError> {
        self.race(|client| client.client_state(request.clone()))
            .await
    }

    pub async fn code(
        &self,
        request: QueryCodeRequest,
    ) -> Result<Response<QueryCodeResponse>, IndexerError> {
        self.race(|client| client.code(request.clone())).await
    }
}

impl From<tonic::Status> for IndexerError {
    fn from(error: tonic::Status) -> Self {
        Self::ProviderError(Report::from(error))
    }
}

impl From<tonic::transport::Error> for IndexerError {
    fn from(error: tonic::transport::Error) -> Self {
        Self::ProviderError(Report::from(error))
    }
}
