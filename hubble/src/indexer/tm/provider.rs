use std::result::Result;

use color_eyre::eyre::Report;
use protos::ibc::{
    core::client::v1::{QueryClientStateRequest, QueryClientStateResponse},
    lightclients::wasm::v1::{QueryCodeRequest, QueryCodeResponse},
};
use tendermint_rpc::{query::Query, Client, HttpClient, Order};
use tonic::Response;
use unionlabs::aptos::block_info::BlockHeight;
use url::Url;

use crate::{
    indexer::api::IndexerError,
    race_client::{RaceClient, RaceClientId, RaceClientResponse},
};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<HttpClient>,
    pub grpc_client: RaceClient<GrpcClient>,
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
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.status())
            .await
            .map(Into::into)
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
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.blockchain(min_inclusive as u32, max_inclusive as u32)
            })
            .await
            .map(Into::into)
    }

    pub async fn latest_block(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<tendermint_rpc::endpoint::block::Response>, tendermint_rpc::error::Error>
    {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.latest_block())
            .await
            .map(Into::into)
    }

    pub async fn commit(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<tendermint_rpc::endpoint::commit::Response>, tendermint_rpc::error::Error>
    {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.commit(height as u32))
            .await
            .map(Into::into)
    }

    pub async fn block_results(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        RpcResult<tendermint_rpc::endpoint::block_results::Response>,
        tendermint_rpc::error::Error,
    > {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.block_results(height as u32)
            })
            .await
            .map(Into::into)
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
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.tx_search(query.clone(), prove, page, per_page, order.clone())
            })
            .await
            .map(Into::into)
    }

    // GRPC
    pub async fn client_state(
        &self,
        request: QueryClientStateRequest,
        provider_id: Option<GrpcProviderId>,
    ) -> Result<GrpcResult<Response<QueryClientStateResponse>>, IndexerError> {
        self.grpc_client
            .race(provider_id.map(Into::into), |c| {
                c.client_state(request.clone())
            })
            .await
            .map(Into::into)
    }

    pub async fn code(
        &self,
        request: QueryCodeRequest,
        provider_id: Option<GrpcProviderId>,
    ) -> Result<GrpcResult<Response<QueryCodeResponse>>, IndexerError> {
        self.grpc_client
            .race(provider_id.map(Into::into), |c| c.code(request.clone()))
            .await
            .map(Into::into)
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
