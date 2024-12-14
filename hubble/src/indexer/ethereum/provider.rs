use alloy::{
    eips::BlockId,
    network::{AnyNetwork, AnyRpcBlock, Network},
    primitives::TxHash,
    providers::{Provider as AlloyProvider, ProviderBuilder, RootProvider},
    rpc::types::{BlockTransactionsKind, Filter, Log},
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use url::Url;
use crate::race_client::{RaceClient, RaceClientId, RaceClientResponse};

/// Wrapper for the race client, providing RPC methods for Ethereum-like networks.
#[derive(Clone, Debug)]
pub struct Provider {
    rpc_client: RaceClient<RootProvider<Http<Client>, AnyNetwork>>,
}

/// Unique identifier for an RPC provider within the race client.
#[derive(Clone, Debug, Copy)]
pub struct RpcProviderId {
    race_client_id: RaceClientId,
}

impl From<RpcProviderId> for RaceClientId {
    fn from(value: RpcProviderId) -> Self {
        value.race_client_id
    }
}

/// Result of an RPC call, including the provider ID and response.
#[derive(Debug)]
pub struct RpcResult<T> {
    pub provider_id: RpcProviderId,
    pub response: T,
}

impl<T> RpcResult<T> {
    fn new(race_client_id: RaceClientId, response: T) -> Self {
        Self {
            provider_id: RpcProviderId { race_client_id },
            response,
        }
    }
}

impl<T> From<RaceClientResponse<T>> for RpcResult<T> {
    fn from(value: RaceClientResponse<T>) -> Self {
        RpcResult::new(value.race_client_id, value.response)
    }
}

impl Provider {
    /// Creates a new provider instance with the specified RPC URLs.
    pub fn new(rpc_urls: Vec<Url>) -> Self {
        let clients = rpc_urls
            .into_iter()
            .map(|url| ProviderBuilder::new().network::<AnyNetwork>().on_http(url))
            .collect();
        Self {
            rpc_client: RaceClient::new(clients),
        }
    }

    /// Retrieves the chain ID of the connected network.
    pub async fn get_chain_id(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<u64>, RpcError<TransportErrorKind>> {
        self.execute_race(provider_id, |client| client.get_chain_id(), "get_chain_id")
            .await
    }

    /// Fetches a block by ID with the specified transaction inclusion type.
    pub async fn get_block(
        &self,
        id: BlockId,
        kind: BlockTransactionsKind,
        provider_id: Option<RpcProviderId>,
    ) -> Result<Option<RpcResult<AnyRpcBlock>>, RpcError<TransportErrorKind>> {
        self.execute_race_some(provider_id, |client| client.get_block(id, kind), "get_block")
            .await
    }

    /// Fetches logs matching the given filter.
    pub async fn get_logs(
        &self,
        filter: &Filter,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Vec<Log>>, RpcError<TransportErrorKind>> {
        self.execute_race(provider_id, |client| client.get_logs(filter), "get_logs")
            .await
    }

    /// Fetches a transaction by hash.
    pub async fn get_transaction_by_hash(
        &self,
        tx_hash: TxHash,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        Option<RpcResult<<AnyNetwork as Network>::TransactionResponse>>,
        RpcError<TransportErrorKind>,
    > {
        self.execute_race_some(provider_id, |client| client.get_transaction_by_hash(tx_hash), "get_transaction_by_hash")
            .await
    }

    /// Helper to execute a race with guaranteed response.
    async fn execute_race<T>(
        &self,
        provider_id: Option<RpcProviderId>,
        action: impl Fn(&RootProvider<Http<Client>, AnyNetwork>) -> T,
        action_name: &str,
    ) -> Result<RpcResult<T::Output>, RpcError<TransportErrorKind>>
    where
        T: std::future::Future,
    {
        self.rpc_client
            .race(provider_id.map(Into::into), action)
            .await
            .map(Into::into)
            .map_err(|err| {
                tracing::error!(error = ?err, action = action_name, "RPC call failed");
                err
            })
    }

    /// Helper to execute a race that may return `None`.
    async fn execute_race_some<T>(
        &self,
        provider_id: Option<RpcProviderId>,
        action: impl Fn(&RootProvider<Http<Client>, AnyNetwork>) -> T,
        action_name: &str,
    ) -> Result<Option<RpcResult<T::Output>>, RpcError<TransportErrorKind>>
    where
        T: std::future::Future<Output = Option<T::Output>>,
    {
        self.rpc_client
            .race_some(provider_id.map(Into::into), action)
            .await
            .map(|opt| opt.map(Into::into))
            .map_err(|err| {
                tracing::error!(error = ?err, action = action_name, "RPC call failed");
                err
            })
    }
}
