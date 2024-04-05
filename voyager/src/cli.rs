use std::{ffi::OsString, marker::PhantomData, sync::Arc};

use chain_utils::Chains;
use clap::{Parser, Subcommand};
use ethers::types::Address;
use frunk::{hlist_pat, HList};
use queue_msg::{aggregation::UseAggregate, run_to_completion, InMemoryQueue};
use relay_message::{
    data::{IbcProof, IbcState},
    use_aggregate::IsAggregateData,
    ChainExt, DoFetchProof, DoFetchState, Identified, RelayMessageTypes,
};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    ibc::core::client::height::Height,
    ics24::{
        self, AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath, ReceiptPath,
    },
    id::ClientId,
    result_unwrap,
    traits::HeightOf,
    QueryHeight,
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
    #[arg(long, short = 'l', env, global = true, default_value_t = LogFormat::default())]
    pub log_format: LogFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Default, clap::ValueEnum, derive_more::Display)]
pub enum LogFormat {
    #[default]
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "json")]
    Json,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    RunMigrations,
    PrintConfig,
    Relay,
    #[command(subcommand)]
    Queue(QueueCmd),
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
    IbcPath(ics24::Path<ClientId, Height>),
}

pub async fn any_state_proof_to_json<Hc, Tr>(
    chains: Arc<Chains>,
    path: ics24::Path<Hc::ClientId, Tr::Height>,
    c: Hc,
    height: QueryHeight<HeightOf<Hc>>,
) -> String
where
    Hc: ChainExt + DoFetchState<Hc, Tr> + DoFetchProof<Hc, Tr>,
    Tr: ChainExt,

    Identified<Hc, Tr, IbcState<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientStatePath<Hc::ClientId>, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
        IsAggregateData,
    Identified<Hc, Tr, IbcProof<ClientConsensusStatePath<Hc::ClientId, Tr::Height>, Hc, Tr>>:
        IsAggregateData,

    Identified<Hc, Tr, IbcState<ConnectionPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ConnectionPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<ChannelEndPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ChannelEndPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<CommitmentPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<CommitmentPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<AcknowledgementPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<AcknowledgementPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<ReceiptPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<ReceiptPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<NextSequenceSendPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<NextSequenceSendPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<NextSequenceRecvPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<NextSequenceRecvPath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<NextSequenceAckPath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<NextSequenceAckPath, Hc, Tr>>: IsAggregateData,
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
        ics24::Path::ClientState(path) => json(
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
        ics24::Path::ClientConsensusState(path) => json(
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
        ics24::Path::Connection(path) => json(
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
        ics24::Path::ChannelEnd(path) => json(
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
        ics24::Path::Commitment(path) => json(
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
        ics24::Path::Acknowledgement(path) => json(
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
        ics24::Path::Receipt(path) => json(
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
        ics24::Path::NextSequenceSend(path) => json(
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
        ics24::Path::NextSequenceRecv(path) => json(
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
        ics24::Path::NextSequenceAck(path) => json(
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
    state: P::Value,
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
    UseAggregate<RelayMessageTypes, StateProof<Hc, Tr, P>> for FetchStateProof<Hc, Tr, P>
where
    Identified<Hc, Tr, IbcState<P, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<P, Hc, Tr>>: IsAggregateData,
{
    type AggregatedData =
        HList![Identified<Hc, Tr, IbcState<P, Hc, Tr>>, Identified<Hc, Tr, IbcProof<P, Hc, Tr>>];

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

type PgId = BoundedI64<1, { i64::MAX }>;
type Pg64 = BoundedI64<0, { i64::MAX }>;
type Pg32 = BoundedI32<1, { i32::MAX }>;

#[derive(Debug, Subcommand)]
pub enum QueueCmd {
    History {
        id: PgId,
        #[arg(long, default_value_t = result_unwrap!(Pg32::new(10)))]
        max_depth: Pg32,
    },
    Failed {
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        page: Pg64,
        #[arg(long, default_value_t = result_unwrap!(Pg64::new(1)))]
        per_page: Pg64,
    },
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
