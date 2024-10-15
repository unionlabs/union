use std::{fmt::Debug, sync::Arc};

use serde::{Deserialize, Serialize};
use tendermint_rpc::{WebSocketClient, WebSocketClientUrl};
use unionlabs::{hash::H256, signer::CosmosSigner};

use crate::{
    cosmos_sdk::{CosmosKeyring, GasConfig},
    keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, SignerBalance},
};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub keyring: CosmosKeyring,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoints: Vec<String>,
    pub grpc_url: String,

    pub checksum_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
    pub gas_config: GasConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keyring: KeyringConfig,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoints: Vec<String>,
    pub grpc_url: String,
    pub gas_config: GasConfig,
}

impl ChainKeyring for Union {
    type Address = String;
    type Signer = CosmosSigner;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        crate::cosmos_sdk::fetch_balances(
            &self.keyring,
            self.gas_config.gas_denom.clone(),
            self.grpc_url.clone(),
        )
        .await
    }
}
