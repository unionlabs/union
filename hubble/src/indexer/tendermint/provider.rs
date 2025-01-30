use std::{
    num::{NonZeroU32, NonZeroU64, NonZeroU8},
    result::Result,
};

use color_eyre::eyre::Report;
use cometbft_rpc::{
    rpc_types::{
        BlockResponse, BlockResultsResponse, BlockchainResponse, Order, StatusResponse,
        TxSearchResponse,
    },
    Client, JsonRpcError,
};
use futures::future;
use unionlabs::aptos::block_info::BlockHeight;
use url::Url;

use crate::{
    indexer::api::IndexerError,
    race_client::{RaceClient, RaceClientId, RaceClientResponse},
};

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

impl Provider {
    pub async fn new(rpc_urls: Vec<Url>) -> Result<Self, IndexerError> {
        Ok(Self {
            rpc_client: {
                RaceClient::new(
                    future::join_all(
                        rpc_urls
                            .into_iter()
                            .map(|rpc_url| Client::new(rpc_url.as_str().to_owned())),
                    )
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?,
                )
            },
        })
    }

    pub async fn status(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<StatusResponse>, JsonRpcError> {
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
    ) -> Result<RpcResult<BlockchainResponse>, JsonRpcError> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.blockchain(
                    NonZeroU64::try_from(min_inclusive).expect("non-zero min"),
                    NonZeroU64::try_from(max_inclusive).expect("non-zero max"),
                )
            })
            .await
            .map(Into::into)
    }

    pub async fn latest_block(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResponse>, JsonRpcError> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.block(None))
            .await
            .map(Into::into)
    }

    pub async fn block(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResponse>, JsonRpcError> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.block(Some(NonZeroU64::try_from(height).expect("non-zero height")))
            })
            .await
            .map(Into::into)
    }

    pub async fn block_results(
        &self,
        height: BlockHeight,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<BlockResultsResponse>, JsonRpcError> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.block_results(Some(NonZeroU64::try_from(height).expect("non-zero height")))
            })
            .await
            .map(Into::into)
    }

    pub async fn tx_search(
        &self,
        height: BlockHeight,
        prove: bool,
        page: u32,
        per_page: u8,
        order: Order,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<TxSearchResponse>, JsonRpcError> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| {
                c.tx_search(
                    format!("tx.height={}", height),
                    prove,
                    NonZeroU32::try_from(page).expect("non-zero page"),
                    NonZeroU8::try_from(per_page).expect("non-zero per-page"),
                    order.clone(),
                )
            })
            .await
            .map(Into::into)
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
