use std::{ffi::OsString, marker::PhantomData, str::FromStr, sync::Arc};

use chain_utils::Chains;
use clap::{
    error::{ContextKind, ContextValue},
    Args, Parser, Subcommand,
};
use ethers::{
    signers::LocalWallet,
    types::{Address, H256},
};
use frunk::{hlist_pat, HList};
use queue_msg::{aggregation::UseAggregate, run_to_completion, InMemoryQueue};
use reqwest::Url;
use unionlabs::{
    ibc::core::client::height::Height,
    id::ClientId,
    proof::{
        self, AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    traits::HeightOf,
    QueryHeight,
};
use voyager_message::{
    data::{IbcProof, IbcState},
    use_aggregate::IsAggregateData,
    ChainExt, DoFetchProof, DoFetchState, Identified, RelayerMsgTypes,
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
    RunMigrations,
    PrintConfig,
    Relay,
    #[command(subcommand)]
    Setup(SetupCmd),
    Query {
        #[arg(long)]
        on: String,
        #[arg(long)]
        tracking: String,
        #[arg(long, default_value_t = QueryHeight::<Height>::Latest)]
        at: QueryHeight<Height>,
        #[command(subcommand)]
        cmd: QueryCmd,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryCmd {
    #[command(subcommand)]
    IbcPath(proof::Path<ClientId, Height>),
}

pub async fn any_state_proof_to_json<Hc, Tr>(
    chains: Arc<Chains>,
    path: proof::Path<Hc::ClientId, Tr::Height>,
    c: Hc,
    height: QueryHeight<HeightOf<Hc>>,
) -> String
where
    Hc: ChainExt + DoFetchState<Hc, Tr> + DoFetchProof<Hc, Tr>,
    Tr: ChainExt,
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientStatePath<Hc::ClientId>>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ClientConsensusStatePath<Hc::ClientId, Tr::Height>>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ConnectionPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ConnectionPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, ChannelEndPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, CommitmentPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, CommitmentPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcState<Hc, Tr, AcknowledgementPath>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, AcknowledgementPath>>: IsAggregateData,
{
    use serde_json::to_string_pretty as json;

    let height = match height {
        QueryHeight::Latest => c.query_latest_height().await.unwrap(),
        QueryHeight::Specific(height) => height,
    };

    tracing::info!("latest height is {height}");

    let msgs = [
        Hc::state(&c, height, path.clone()),
        Hc::proof(&c, height, path.clone()),
    ];

    match path {
        proof::Path::ClientStatePath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
        proof::Path::ClientConsensusStatePath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
        proof::Path::ConnectionPath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
        proof::Path::ChannelEndPath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
        proof::Path::CommitmentPath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
        proof::Path::AcknowledgementPath(path) => json(
            &run_to_completion::<_, _, _, InMemoryQueue<_>>(
                FetchStateProof {
                    path,
                    height,
                    __marker: PhantomData,
                },
                chains,
                (),
                msgs,
            )
            .await,
        ),
    }
    .unwrap()
}

#[derive(Debug, serde::Serialize)]
#[serde(bound(serialize = ""))]
struct StateProof<Hc: ChainExt, Tr: ChainExt, P: IbcPath<Hc, Tr>> {
    #[serde(with = "::serde_utils::string")]
    path: P,
    state: P::Output,
    proof: Hc::StateProof,
    height: HeightOf<Hc>,
}

#[derive(Debug, serde::Serialize)]
#[serde(bound(serialize = ""))]
struct FetchStateProof<Hc: ChainExt, Tr: ChainExt, P: IbcPath<Hc, Tr>> {
    #[serde(with = "::serde_utils::string")]
    path: P,
    height: HeightOf<Hc>,
    #[serde(skip)]
    pub __marker: PhantomData<fn() -> Tr>,
}

impl<Hc: ChainExt, Tr: ChainExt, P: IbcPath<Hc, Tr>>
    UseAggregate<RelayerMsgTypes, StateProof<Hc, Tr, P>> for FetchStateProof<Hc, Tr, P>
where
    Identified<Hc, Tr, IbcState<Hc, Tr, P>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<Hc, Tr, P>>: IsAggregateData,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<Hc, Tr, P>>, Identified<Hc, Tr, IbcProof<Hc, Tr, P>>];

    fn aggregate(
        this: Self,
        hlist_pat![
            Identified {
                chain_id: _state_chain_id,
                t: IbcState {
                    path: state_path,
                    height: state_height,
                    state,
                },
                __marker: _,
            },
            Identified {
                chain_id: _proof_chain_id,
                t: IbcProof {
                    path: proof_path,
                    height: proof_height,
                    proof,
                    __marker: _,
                },
                __marker: _,
            },
        ]: Self::AggregatedData,
    ) -> StateProof<Hc, Tr, P> {
        assert_eq!(state_path, proof_path);
        assert_eq!(this.path, proof_path);
        assert_eq!(state_height, proof_height);
        assert_eq!(this.height, proof_height);

        StateProof {
            path: this.path,
            state,
            proof,
            height: this.height,
        }
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
