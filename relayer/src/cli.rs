use std::{ffi::OsString, str::FromStr};

use clap::{
    error::{ContextKind, ContextValue},
    Args, Parser, Subcommand,
};
use ethers::{
    signers::LocalWallet,
    types::{Address, H256},
};
use reqwest::Url;
use unionlabs::ethereum_consts_traits::PresetBaseKind;

use crate::chain::{
    evm::CometblsConfig, proof, union::EthereumConfig, Chain, LightClient, QueryHeight,
};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(
        long,
        short = 'c',
        env,
        global = true,
        default_value = "~/.config/relayer/config.json"
    )]
    pub config_file_path: OsString,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    PrintConfig,
    #[command(subcommand)]
    Chain(ChainCmd),
    #[command(subcommand)]
    Client(ClientCmd),
    #[command(subcommand)]
    Connection(ConnectionCmd),
    #[command(subcommand)]
    Channel(ChannelCmd),
    Relay(RelayCmd),
    #[command(subcommand)]
    SubmitPacket(SubmitPacketCmd),
    #[command(subcommand)]
    Query(QueryCmd),
    #[command(subcommand)]
    Setup(SetupCmd),
    #[command(subcommand)]
    Ibc(IbcCmd),
}

#[derive(Debug, Subcommand)]
pub enum IbcCmd {
    Query {
        #[arg(long)]
        on: String,
        #[arg(long, default_value_t = QueryHeight::Latest)]
        at: QueryHeight,
        #[command(subcommand)]
        cmd: IbcQueryCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum IbcQueryCmd {
    #[command(subcommand)]
    Path(IbcQueryPathCmd),
}

#[derive(Debug, clap::Subcommand)]
pub enum IbcQueryPathCmd {
    ClientState(proof::ClientStatePath),
    ClientConsensusState(proof::ClientConsensusStatePath),
    Connection(proof::ConnectionPath),
    ChannelEnd(proof::ChannelEndPath),
    Commitment(proof::CommitmentPath),
}

impl IbcQueryPathCmd {
    pub async fn any_state_proof_to_json<L: LightClient>(
        self,
        l: &L,
        height: QueryHeight,
    ) -> String {
        use serde_json::to_string_pretty as json;

        let height = match height {
            QueryHeight::Latest => l.chain().query_latest_height().await,
            QueryHeight::Specific(height) => height,
        };

        match self {
            Self::ClientState(path) => json(&l.state_proof(path, height).await),
            Self::ClientConsensusState(path) => json(&l.state_proof(path, height).await),
            Self::Connection(path) => json(&l.state_proof(path, height).await),
            Self::ChannelEnd(path) => json(&l.state_proof(path, height).await),
            Self::Commitment(path) => json(&l.state_proof(path, height).await),
        }
        .unwrap()
    }
}

#[derive(Debug, Subcommand)]
pub enum SetupCmd {
    SetOperator {
        #[arg(long)]
        on: String,
    },
    BindPort {
        #[arg(long)]
        on: String,
        #[arg(long)]
        module_address: Address,
        #[arg(long)]
        port_id: String,
    },
    InitialChannel {
        #[arg(long)]
        on: String,
        #[arg(long)]
        module_address: Address,
        #[arg(long)]
        channel_id: String,
        #[arg(long)]
        port_id: String,
        #[arg(long)]
        counterparty_port_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SubmitPacketCmd {
    Transfer {
        #[arg(long)]
        on: String,
        #[arg(long)]
        denom: String,
        #[arg(long)]
        amount: u64,
        #[arg(long)]
        receiver: String,
        #[arg(long)]
        source_port: String,
        #[arg(long)]
        source_channel: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryCmd {
    Client {
        #[arg(long)]
        on: String,
        #[arg(long)]
        client_id: String,
    },
    Connection {},
    Channel {},
    Balances {
        #[arg(long)]
        on: String,
        #[arg(long)]
        who: Address,
        #[arg(long)]
        denom: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ChainCmd {
    #[command(subcommand)]
    Add(ChainAddCmd),
}

#[derive(Debug, Subcommand)]
pub enum ChainAddCmd {
    Evm {
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        name: String,
        #[arg(long)]
        preset_base: PresetBaseKind,
        #[command(flatten)]
        config: crate::config::EvmChainConfigFields,
    },
    Union {
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        name: String,
        #[command(flatten)]
        config: crate::config::UnionChainConfig,
    },
    Cosmos {
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        name: String,
        #[command(flatten)]
        config: crate::config::UnionChainConfig,
    },
}

#[derive(Debug, Subcommand)]
pub enum ClientCmd {
    #[command(subcommand)]
    Create(ClientCreateCmd),
}

#[derive(Debug, Subcommand)]
pub enum ClientCreateCmd {
    #[command(subcommand)]
    Evm(EvmClientType),
    #[command(subcommand)]
    Union(CometblsClientType),
}

#[derive(Debug, Args)]
pub struct ClientQueryCmd {
    #[arg(long)]
    pub client_id: String,
    #[arg(long)]
    pub on: String,
}

#[derive(Debug, Subcommand)]
pub enum EvmClientType {
    Cometbls {
        /// The name of the chain to create the client on, as specified in the config file.
        #[arg(long)]
        on: String,
        /// The name of the chain that the client will connect to, as specified in the config file.
        #[arg(long)]
        counterparty: String,
        #[command(flatten)]
        config: CometblsConfig,
    },
}

#[derive(Debug, Subcommand)]
pub enum CometblsClientType {
    Ethereum08Wasm {
        /// The name of the chain to create the client on, as specified in the config file.
        #[arg(long)]
        on: String,
        /// The name of the chain that the client will connect to, as specified in the config file.
        #[arg(long)]
        counterparty: String,
        #[command(flatten)]
        config: EthereumConfig,
    },
    Tendermint {
        /// The name of the chain to create the client on, as specified in the config file.
        #[arg(long)]
        on: String,
        /// The name of the chain that the client will connect to, as specified in the config file.
        #[arg(long)]
        counterparty: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConnectionCmd {
    // #[command(subcommand)]
    // Query(ConnectionQueryCmd),
    Open {
        #[arg(long)]
        from_chain: String,
        #[arg(long)]
        from_client: String,

        #[arg(long)]
        to_chain: String,
        #[arg(long)]
        to_client: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ChannelCmd {
    Open {
        #[arg(long)]
        from_chain: String,
        #[arg(long)]
        from_connection: String,
        #[arg(long)]
        from_port: String,
        #[arg(long)]
        from_version: String,

        #[arg(long)]
        to_chain: String,
        #[arg(long)]
        to_connection: String,
        #[arg(long)]
        to_port: String,
        #[arg(long)]
        to_version: String,
    },
}

#[derive(Debug, Parser)]
pub struct OpenConnectionArgs {
    #[command(flatten)]
    pub args: ClientArgs,
}

#[derive(Debug, Parser)]
pub struct OpenChannelArgs {
    #[command(flatten)]
    pub args: ClientArgs,

    #[arg(long)]
    pub cometbls_port_id: String,
    #[arg(long)]
    pub ethereum_port_id: String,

    /// format is client_id/connection_id
    #[arg(long)]
    pub cometbls: ConnectionEndInfo,
    /// format is client_id/connection_id
    #[arg(long)]
    pub ethereum: ConnectionEndInfo,
}

#[derive(Debug, Parser)]
pub struct RelayCmd {
    #[arg(long)]
    pub between: Vec<Between>,
}

#[derive(Debug, Clone)]
pub struct Between(pub String, pub String);

impl FromStr for Between {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once(':')
            .map(|(a, b)| Self(a.to_string(), b.to_string()))
            .ok_or_else(|| {
                let mut error = clap::Error::new(clap::error::ErrorKind::ValueValidation);

                error.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String(s.to_string()),
                );

                error.insert(
                    ContextKind::Usage,
                    ContextValue::String("<chain id>:<chain id>".to_string()),
                );

                error
            })
    }
}

#[derive(Debug, Parser)]
pub struct TransferArgs {
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long)]
    pub denom: String,
    #[arg(long)]
    pub amount: u64,
    #[arg(long)]
    pub receiver: String,
    #[arg(long)]
    pub source_port: String,
    #[arg(long)]
    pub source_channel: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionEndInfo {
    pub client_id: String,
    pub connection_id: String,
}

impl FromStr for ConnectionEndInfo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (client_id, connection_id) = s
            .split_once('/')
            .ok_or("invalid: expected chain_name/client_id")?;

        if connection_id.contains('/') {
            Err("too many segments".to_string())
        } else {
            Ok(Self {
                client_id: client_id.to_string(),
                connection_id: connection_id.to_string(),
            })
        }
    }
}

#[derive(Debug, Parser)]
pub struct ClientArgs {
    #[command(flatten)]
    pub cometbls: CometblsClientArgs,
    #[command(flatten)]
    pub ethereum: EthereumClientArgs,
}

#[derive(Debug, Args)]
pub struct CometblsClientArgs {
    /// OwnableIBCHandler => address
    #[arg(long)]
    pub ibc_handler_address: Address,
    /// CometblsClient => address
    #[arg(long)]
    pub cometbls_client_address: Address,
    /// ICS20TransferBank => address
    #[arg(long)]
    pub ics20_transfer_address: Address,
    /// ICS20Bank => address
    #[arg(long)]
    pub ics20_bank_address: Address,

    #[arg(long)]
    pub wallet: LocalWallet,

    #[arg(long)]
    pub eth_rpc_api: Url,

    #[arg(long)]
    pub eth_beacon_rpc_api: String,
}

#[derive(Debug, Args)]
pub struct EthereumClientArgs {
    #[arg(long = "code-id")]
    pub wasm_code_id: H256,
}

#[derive(Debug, Subcommand)]
pub enum CreateClientArgs {
    Cometbls { ibc_handler_address: Address },
    Ethereum { wasm_code_id: H256 },
}
