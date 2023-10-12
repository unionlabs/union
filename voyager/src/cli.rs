use std::{ffi::OsString, str::FromStr};

use chain_utils::Chain;
use clap::{
    error::{ContextKind, ContextValue},
    Args, Parser, Subcommand,
};
use contracts::ucs01_relay::LocalToken;
use ethers::{
    signers::LocalWallet,
    types::{Address, H256},
};
use reqwest::Url;
use unionlabs::ibc::core::client::height::Height;

use crate::chain::{
    proof::{self, ClientConsensusStatePath, ClientStatePath, IbcStateReadPaths},
    HeightOf, QueryHeight,
};

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
pub struct AppArgs {
    #[arg(
        long,
        short = 'c',
        env,
        global = true,
        default_value = "~/.config/voyager/config.json"
    )]
    pub config_file_path: OsString,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    PrintConfig,
    Relay,
    #[command(subcommand)]
    Setup(SetupCmd),
    Query {
        #[arg(long)]
        on: String,
        #[arg(long, default_value_t = QueryHeight::<Height>::Latest)]
        at: QueryHeight<Height>,
        #[command(subcommand)]
        cmd: QueryCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryCmd {
    #[command(subcommand)]
    IbcPath(QueryIbcPathCmd),
}

#[derive(Debug, clap::Subcommand)]
pub enum QueryIbcPathCmd {
    ClientState(proof::ClientStatePath<String>),
    ClientConsensusState(proof::ClientConsensusStatePath<String, Height>),
    Connection(proof::ConnectionPath),
    ChannelEnd(proof::ChannelEndPath),
    Commitment(proof::CommitmentPath),
    Acknowledgement(proof::AcknowledgementPath),
}

impl QueryIbcPathCmd {
    pub async fn any_state_proof_to_json<
        Counterparty: Chain,
        This: IbcStateReadPaths<Counterparty>,
    >(
        self,
        c: This,
        height: QueryHeight<HeightOf<This>>,
    ) -> String {
        use serde_json::to_string_pretty as json;

        let height = match height {
            QueryHeight::Latest => c.query_latest_height().await,
            QueryHeight::Specific(height) => height,
        };

        match self {
            Self::ClientState(path) => json(
                &c.state_proof(
                    ClientStatePath {
                        client_id: path.client_id.parse().unwrap(),
                    },
                    height,
                )
                .await,
            ),
            Self::ClientConsensusState(path) => json(
                &c.state_proof(
                    ClientConsensusStatePath {
                        client_id: path.client_id.parse().unwrap(),
                        height: path.height.into(),
                    },
                    height,
                )
                .await,
            ),
            Self::Connection(path) => json(&c.state_proof(path, height).await),
            Self::ChannelEnd(path) => json(&c.state_proof(path, height).await),
            Self::Commitment(path) => json(&c.state_proof(path, height).await),
            Self::Acknowledgement(path) => json(&c.state_proof(path, height).await),
        }
        .unwrap()
    }
}

#[derive(Debug, Subcommand)]
pub enum SetupCmd {
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
