use std::future::IntoFuture;

use alloy::{
    eips::BlockId,
    network::{AnyNetwork, AnyRpcBlock},
    providers::{DynProvider, Provider as AlloyProvider, ProviderBuilder},
    rpc::types::{BlockTransactionsKind, Filter, Log},
    transports::{RpcError, TransportErrorKind},
};
use url::Url;

use crate::race_client::{RaceClient, RaceClientId, RaceClientResponse};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<DynProvider<AnyNetwork>>,
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
    pub fn new(rpc_urls: Vec<Url>) -> Self {
        Self {
            rpc_client: RaceClient::new(
                rpc_urls
                    .into_iter()
                    .map(|url| {
                        DynProvider::new(
                            ProviderBuilder::new().network::<AnyNetwork>().on_http(url),
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub async fn get_chain_id(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<u64>, RpcError<TransportErrorKind>> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.get_chain_id())
            .await
            .map(Into::into)
    }

    pub async fn get_block(
        &self,
        id: BlockId,
        kind: BlockTransactionsKind,
        provider_id: Option<RpcProviderId>,
    ) -> Result<Option<RpcResult<AnyRpcBlock>>, RpcError<TransportErrorKind>> {
        self.rpc_client
            .race_some(provider_id.map(Into::into), |c| {
                c.get_block(id).kind(kind).into_future()
            })
            .await
            .map(|op| op.map(Into::into))
    }

    pub async fn get_logs(
        &self,
        filter: &Filter,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Vec<Log>>, RpcError<TransportErrorKind>> {
        self.rpc_client
            .race(provider_id.map(Into::into), |c| c.get_logs(filter))
            .await
            .map(Into::into)
    }
}
