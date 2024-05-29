use std::{ffi::OsString, marker::PhantomData, str::FromStr, sync::Arc};

use chain_utils::Chains;
use clap::{
    error::{ContextKind, ContextValue, ErrorKind},
    Args, FromArgMatches, Parser, Subcommand,
};
use frunk::{hlist_pat, HList};
use queue_msg::{aggregation::UseAggregate, run_to_completion, InMemoryQueue};
use relay_message::{
    data::{IbcProof, IbcState},
    use_aggregate::IsAggregateData,
    ChainExt, DoFetchProof, DoFetchState, Identified, RelayMessageTypes,
};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    ibc::core::{channel, client::height::Height},
    ics24::{
        self, AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, NextClientSequencePath,
        NextConnectionSequencePath, NextSequenceAckPath, NextSequenceRecvPath,
        NextSequenceSendPath, ReceiptPath,
    },
    id::{ClientId, ConnectionId, PortId},
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
    #[arg(long, global = true, default_value_t = 2 * 1024 * 1024)]
    pub stack_size: usize,
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

#[derive(Debug)]
pub struct Handshake {
    pub chain_a: String,
    pub chain_b: String,

    pub ty: HandshakeType,
}

impl Args for Handshake {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        HandshakeRaw::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        HandshakeRaw::augment_args_for_update(cmd)
    }
}

impl Handshake {
    pub fn from_raw(mut raw: HandshakeRaw) -> Result<Self, clap::Error> {
        use HandshakeType::*;

        let msg = |arg: &str| {
            let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::Strings(vec![format!("--{}", arg.replace('_', "-"))]),
            );
            err
        };

        macro_rules! fields {
            ($Ty:ident { $($field:ident,)+ }) => {
                $Ty {
                    $(
                        $field: raw.$field.take().ok_or_else(|| msg(stringify!($field)))?,
                    )+
                }
            };
        }

        let ty = match (raw.create_clients, raw.open_connection, raw.open_channel) {
            (true, true, true) => fields!(ClientConnectionChannel {
                client_a_config,
                client_b_config,
                port_a,
                port_b,
                channel_version,
                connection_ordering,
                channel_ordering,
            }),
            (true, true, false) => fields!(ClientConnection {
                client_a_config,
                client_b_config,
                connection_ordering,
            }),
            (true, false, true) => {
                let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
                err.insert(
                    ContextKind::Usage,
                    ContextValue::StyledStr(
                        "--open-connection is required when passing --create-clients and --open-channel".into()
                    ),
                );
                return Err(err);
            }
            (true, false, false) => fields!(Client {
                client_a_config,
                client_b_config,
            }),
            (false, true, true) => fields!(ConnectionChannel {
                client_a,
                client_b,
                port_a,
                port_b,
                channel_version,
                connection_ordering,
                channel_ordering,
            }),
            (false, true, false) => fields!(Connection {
                client_a,
                client_b,
                connection_ordering,
            }),
            (false, false, true) => fields!(Channel {
                connection_a,
                port_a,
                port_b,
                channel_version,
                channel_ordering,
            }),
            (false, false, false) => {
                let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
                err.insert(
                    ContextKind::Usage,
                    ContextValue::StyledStr(
                        "One of --create-clients, --open-connection, or --open-channel is required"
                            .into(),
                    ),
                );
                return Err(err);
            }
        };

        raw.assert_empty()?;

        Ok(Self {
            chain_a: raw.chain_a,
            chain_b: raw.chain_b,
            ty,
        })
    }
}

#[derive(Debug)]
pub enum HandshakeType {
    Client {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
    },
    ClientConnection {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
        connection_ordering: Vec<channel::order::Order>,
    },
    ClientConnectionChannel {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        connection_ordering: Vec<channel::order::Order>,
        channel_ordering: channel::order::Order,
    },
    ConnectionChannel {
        client_a: ClientId,
        client_b: ClientId,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        connection_ordering: Vec<channel::order::Order>,
        channel_ordering: channel::order::Order,
    },
    Connection {
        client_a: ClientId,
        client_b: ClientId,
        connection_ordering: Vec<channel::order::Order>,
    },
    Channel {
        connection_a: ConnectionId,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        channel_ordering: channel::order::Order,
    },
}

impl FromArgMatches for Handshake {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        HandshakeRaw::from_arg_matches(matches).and_then(Self::from_raw)
    }

    fn update_from_arg_matches(&mut self, _matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        todo!()
    }
}

#[derive(Debug, Args)]
pub struct HandshakeRaw {
    #[arg(required = true)]
    pub chain_a: String,
    #[arg(required = true)]
    pub chain_b: String,

    #[arg(long)]
    pub create_clients: bool,
    #[arg(
        long,
        value_parser(<serde_json::Value as FromStr>::from_str),
    )]
    pub client_a_config: Option<serde_json::Value>,
    #[arg(
        long,
        value_parser(<serde_json::Value as FromStr>::from_str),
    )]
    pub client_b_config: Option<serde_json::Value>,

    #[arg(long)]
    pub client_a: Option<ClientId>,
    #[arg(long)]
    pub client_b: Option<ClientId>,

    #[arg(long)]
    pub open_connection: bool,

    #[arg(long)]
    pub connection_a: Option<ConnectionId>,
    // #[arg(long, conflicts_with = "open_connection")]
    // connection_b: Option<ConnectionId>,
    #[arg(long)]
    pub open_channel: bool,
    #[arg(long)]
    pub port_a: Option<PortId>,
    #[arg(long)]
    pub port_b: Option<PortId>,
    #[arg(long)]
    pub channel_version: Option<String>,
    #[arg(
        long,
        value_parser(
            // don't ask questions you don't want the answer to
            |s: &str| serde_json::from_value::<unionlabs::ibc::core::channel::order::Order>(<serde_json::Value as From<&str>>::from(s))
        ),
    )]
    pub channel_ordering: Option<channel::order::Order>,
    #[arg(
        long,
        value_parser(
            // don't ask questions you don't want the answer to
            |s: &str| serde_json::from_value::<unionlabs::ibc::core::channel::order::Order>(<serde_json::Value as From<&str>>::from(s))
        ),
    )]
    pub connection_ordering: Option<Vec<channel::order::Order>>,
}

impl HandshakeRaw {
    fn assert_empty(&self) -> clap::error::Result<()> {
        macro_rules! none_or_err {
            ($($field:ident,)+) => {
                $(
                    if self.$field.is_some() {
                        let mut err = clap::Error::new(ErrorKind::UnknownArgument);
                        err.insert(
                            ContextKind::InvalidArg,
                            ContextValue::String(format!("--{}", stringify!($field).replace('_', "-"))),
                        );
                        return Err(err)
                    }
                )+
            };
        }

        none_or_err! {
            client_a_config,
            client_b_config,

            client_a,
            client_b,

            connection_a,
            port_a,
            port_b,
            channel_version,
            channel_ordering,
            connection_ordering,
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    RunMigrations,
    PrintConfig,
    Handshake(Handshake),
    InitFetch {
        on: String,
    },
    Relay,
    #[command(subcommand)]
    Queue(QueueCmd),
    #[command(subcommand)]
    Util(UtilCmd),
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
) -> serde_json::Value
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

    Identified<Hc, Tr, IbcState<NextConnectionSequencePath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<NextConnectionSequencePath, Hc, Tr>>: IsAggregateData,

    Identified<Hc, Tr, IbcState<NextClientSequencePath, Hc, Tr>>: IsAggregateData,
    Identified<Hc, Tr, IbcProof<NextClientSequencePath, Hc, Tr>>: IsAggregateData,
{
    use serde_json::to_value as json;

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
        ics24::Path::NextConnectionSequence(path) => json(
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
        ics24::Path::NextClientSequence(path) => json(
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
// TODO: Replace with TupleAggregator
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
pub enum UtilCmd {
    QueryLatestHeight {
        on: String,
    },
    #[command(subcommand)]
    Arbitrum(ArbitrumCmd),
}

#[derive(Debug, Subcommand)]
pub enum ArbitrumCmd {
    LatestConfirmedAtBeaconSlot { on: String, slot: u64 },
    ExecutionHeightOfBeaconSlot { on: String, slot: u64 },
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
