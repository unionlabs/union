use std::collections::BTreeMap;

use chain_utils::{private_key::PrivateKey, union::Union};
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{WebSocketClient, WebSocketClientUrl};
use unionlabs::{ethereum::Address, CosmosAccountId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionConfig {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

impl UnionConfig {
    pub async fn get_union_for(&self, account_index: usize) -> chain_utils::union::Union {
        let config = chain_utils::union::Config {
            signer: self.signers.get(account_index).unwrap().to_owned(),
            fee_denom: self.fee_denom.clone(),
            ws_url: self.ws_url.clone(),
            prover_endpoint: self.prover_endpoint.clone(),
            grpc_url: self.grpc_url.clone(),
        };

        chain_utils::union::Union::new(config).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub union: UnionConfig,
    pub evm: chain_utils::evm::Config,
    pub contract: String,
    pub channel: String,
}
