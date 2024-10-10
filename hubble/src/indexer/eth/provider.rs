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

use crate::race_client::RaceClient;

#[derive(Clone, Debug)]
pub struct Provider {
    pub rpc_client: RaceClient<RootProvider<Http<Client>>>,
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
                    .map(|url| ProviderBuilder::new().on_http(url))
                    .collect(),
            ),
        }
    }

    fn rpc_client(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> RaceClient<RootProvider<Http<Client>>> {
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

    pub async fn get_chain_id(
        &self,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<u64>, RpcError<TransportErrorKind>> {
        let result = self.rpc_client(provider_id).get_chain_id().await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_block(
        &self,
        id: BlockId,
        kind: BlockTransactionsKind,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Option<Block>>, RpcError<TransportErrorKind>> {
        let result = self.rpc_client(provider_id).get_block(id, kind).await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_logs(
        &self,
        filter: &Filter,
        provider_id: Option<RpcProviderId>,
    ) -> Result<RpcResult<Vec<Log>>, RpcError<TransportErrorKind>> {
        let result = self.rpc_client(provider_id).get_logs(filter).await?;

        // TODO: improve race client to return index with result
        Ok(RpcResult::new(
            provider_id.map_or_else(
                || self.rpc_client.fastest_index(),
                |provider_id| provider_id.index,
            ),
            result,
        ))
    }

    pub async fn get_transaction_by_hash(
        &self,
        tx_hash: TxHash,
        provider_id: Option<RpcProviderId>,
    ) -> Result<
        RpcResult<Option<<Ethereum as Network>::TransactionResponse>>,
        RpcError<TransportErrorKind>,
    > {
        let result = self
            .rpc_client(provider_id)
            .get_transaction_by_hash(tx_hash)
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

impl RaceClient<RootProvider<Http<Client>>> {
    pub async fn get_chain_id(&self) -> Result<u64, RpcError<TransportErrorKind>> {
        self.race(|c| c.get_chain_id()).await
    }

    pub async fn get_block(
        &self,
        id: BlockId,
        kind: BlockTransactionsKind,
    ) -> Result<Option<Block>, RpcError<TransportErrorKind>> {
        self.race_some(|c| c.get_block(id, kind)).await
    }

    pub async fn get_logs(
        &self,
        filter: &Filter,
    ) -> Result<Vec<Log>, RpcError<TransportErrorKind>> {
        self.race(|c| c.get_logs(filter)).await
    }

    pub async fn get_transaction_by_hash(
        &self,
        tx_hash: TxHash,
    ) -> Result<Option<<Ethereum as Network>::TransactionResponse>, RpcError<TransportErrorKind>>
    {
        self.race_some(|c| c.get_transaction_by_hash(tx_hash)).await
    }
}
