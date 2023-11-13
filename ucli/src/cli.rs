use std::ffi::OsString;

use beacon_api::client::BeaconApiClient;
use chain_utils::private_key::PrivateKey;
use clap::{Parser, Subcommand};
use ethers::{
    prelude::k256::ecdsa,
    providers::{Middleware, Provider, Ws},
    signers::LocalWallet,
    utils::secret_key_to_address,
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::{ethereum::config::ChainSpec, hash::H160, uint::U256};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(
        long,
        short = 'c',
        env,
        global = true,
        default_value = "~/.config/ucli/config.json"
    )]
    pub config_file_path: OsString,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(subcommand)]
    Tx(TxCmd),
    #[command(subcommand)]
    Query(QueryCmd),
}

#[derive(Debug, Subcommand)]
pub enum TxCmd {
    #[command(subcommand)]
    Evm(EvmTx),
}

#[derive(Debug, Subcommand)]
pub enum EvmTx {
    Transfer {
        #[arg(long)]
        relay_address: H160,
        // #[arg(long)]
        // from: Address,
        // #[arg(long)]
        // to: String,
        #[arg(long)]
        port_id: String,
        #[arg(long)]
        channel_id: String,
        #[arg(long)]
        receiver: String,
        #[arg(long)]
        amount: u64,
        #[arg(long)]
        denom: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryCmd {
    #[command(subcommand)]
    Evm(EvmQuery),
}

#[derive(Debug, Subcommand)]
pub enum EvmQuery {
    // TODO(aeryz): Check if native denoms present in the `denomToAddress` mapping.
    Ucs01Balance {
        #[arg(long)]
        contract_address: H160,
        #[arg(long)]
        denom: String,
        #[arg(long)]
        address: H160,
    },
    Erc20Balance {
        #[arg(long)]
        contract_address: H160,
        #[arg(long)]
        address: H160,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Config {
    pub evm: EvmChainConfig,
    pub union: UnionChainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "preset_base")]
pub enum EvmChainConfig {
    Mainnet(EvmChainConfigFields),
    Minimal(EvmChainConfigFields),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmChainConfigFields {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    pub signer: PrivateKey<ecdsa::SigningKey>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionChainConfig {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

#[derive(Debug, Clone)]
pub struct Evm<C: ChainSpec> {
    pub chain_id: U256,
    pub wallet: LocalWallet,
    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient<C>,
}

impl<C: ChainSpec> Evm<C> {
    pub async fn new(config: EvmChainConfigFields) -> Result<Self, ()> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await.unwrap());

        let chain_id = provider.get_chainid().await.unwrap();

        let signer = config.signer.value();
        let address = secret_key_to_address(&signer);
        let wallet = LocalWallet::new_with_signer(signer, address, chain_id.as_u64());

        Ok(Self {
            chain_id: U256(chain_id),
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
            wallet,
        })
    }
}
