use chain_utils::private_key::PrivateKey;
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::{ethereum::Address, ethereum_consts_traits::Minimal};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionConfig {
    /// Signers that will be used to submit transactions from zerg.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    /// Denom of the currency being sent
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

impl UnionConfig {
    pub async fn get_union_for(&self, account_index: usize) -> chain_utils::union::Union {
        let config = chain_utils::union::Config {
            signer: self.signers[account_index].to_owned(),
            fee_denom: self.fee_denom.clone(),
            ws_url: self.ws_url.clone(),
            prover_endpoint: self.prover_endpoint.clone(),
            grpc_url: self.grpc_url.clone(),
        };

        chain_utils::union::Union::new(config).await.unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmConfig {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: Address,
    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

impl EvmConfig {
    pub async fn get_evm_for(&self, account_index: usize) -> chain_utils::evm::Evm<Minimal> {
        let config = chain_utils::evm::Config {
            signer: self.signers[account_index].to_owned(),
            ibc_handler_address: self.ibc_handler_address.clone(),
            eth_rpc_api: self.eth_rpc_api.clone(),
            eth_beacon_rpc_api: self.eth_beacon_rpc_api.clone(),
            hasura_config: None,
        };

        chain_utils::evm::Evm::<Minimal>::new(config).await.unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub union: UnionConfig,
    pub evm: EvmConfig,
    pub union_contract: String,
    pub evm_contract: Address,
    pub channel: String,
    pub rush_blocks: u64,
}
