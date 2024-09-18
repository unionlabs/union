#![allow(clippy::disallowed_types)]
use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    network::{Ethereum, Network},
    primitives::TxHash,
    providers::{Provider, RootProvider},
    rpc::types::{Block, BlockTransactionsKind, Filter, Log, TransactionReceipt},
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};

use crate::race_client::RaceClient;

impl RaceClient<RootProvider<Http<Client>>> {
    pub async fn get_chain_id(&self) -> Result<u64, RpcError<TransportErrorKind>> {
        self.race(|c| c.get_chain_id()).await
    }
    #[allow(dead_code)]
    pub async fn get_block_receipts(
        &self,
        block: BlockNumberOrTag,
    ) -> Result<Option<Vec<TransactionReceipt>>, RpcError<TransportErrorKind>> {
        self.race(|c| c.get_block_receipts(block)).await
    }

    pub async fn get_block(
        &self,
        id: BlockId,
        kind: BlockTransactionsKind,
    ) -> Result<Option<Block>, RpcError<TransportErrorKind>> {
        self.race_some(|c| c.get_block(id, kind)).await
    }
    #[allow(dead_code)]
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
