use std::{collections::BTreeMap, ffi::OsString, str::FromStr};

use chain_utils::private_key::PrivateKey;
use clap::{
    error::{ContextKind, ContextValue},
    Args, Parser, Subcommand,
};
use ethers::{
    prelude::k256::ecdsa,
    signers::LocalWallet,
    types::{Address, H256},
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;

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
        on: String,
        #[arg(long)]
        relay_address: Address,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Config {
    /// Map of chain name to it's respective config.
    pub chain: BTreeMap<String, ChainConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "chain_type")]
pub enum ChainConfig {
    Evm(EvmChainConfig),
    Union(UnionChainConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmChainConfig {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: Address,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

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
