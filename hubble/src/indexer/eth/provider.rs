use alloy::{
    eips::BlockId,
    network::{Ethereum, Network},
    primitives::TxHash,
    providers::{Provider as AlloyProvider, ProviderBuilder, RootProvider},
    rpc::types::{Block, BlockTransactionsKind, Filter, Log},
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use url::Url;

use crate::race_client::{RaceClient, RaceClientId, RaceClientResponse};

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<RootProvider<Http<Client>>>,
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
                    .map(|url| ProviderBuilder::new().on_http(url))
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
    ) -> Result<Option<RpcResult<Block>>, RpcError<TransportErrorKind>> {
        self.rpc_client
            .race_some(provider_id.map(Into::into), |c| c.get_block(id, kind))
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

    pub async fn get_transaction_by_hash(
        &self,
        tx_hash: TxHash,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        Option<RpcResult<<Ethereum as Network>::TransactionResponse>>,
        RpcError<TransportErrorKind>,
    > {
        self.rpc_client
            .race_some(provider_id.map(Into::into), |c| {
                c.get_transaction_by_hash(tx_hash)
            })
            .await
            .map(|op| op.map(Into::into))
    }
}
