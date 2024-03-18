#![allow(clippy::type_complexity)]

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use block_message::BlockPollingTypes;
use chain_utils::{cosmos::Cosmos, evm::Evm, union::Union, wasm::Wasm, Chains};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{channel::mpsc::UnboundedSender, Future, SinkExt, StreamExt};
use queue_msg::{
    event, HandleAggregate, HandleData, HandleEvent, HandleFetch, HandleMsg, HandleWait,
    InMemoryQueue, Queue, QueueError, QueueMsg, QueueMsgTypes, Reactor,
};
use relay_message::{ChainExt, RelayerMsgTypes};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::task::JoinSet;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    traits::{Chain, ClientIdOf, ClientState, FromStrExact},
    WasmClientType,
};

use crate::{
    chain::{AnyChain, AnyChainTryFromConfigError},
    config::{ChainConfig, Config},
};

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Voyager {
    pub chains: Arc<Chains>,
    num_workers: u16,
    // NOTE: pub temporarily
    pub queue: AnyQueue<VoyagerMessageTypes>,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    tag = "type",
    bound(serialize = "", deserialize = "")
)]
pub enum AnyQueueConfig {
    InMemory,
    PgQueue(PgQueueConfig),
}

#[derive(DebugNoBound, CloneNoBound)]
pub enum AnyQueue<T: QueueMsgTypes> {
    InMemory(InMemoryQueue<T>),
    PgQueue(PgQueue<T>),
}

#[derive(DebugNoBound, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(std::convert::Infallible),
    PgQueue(sqlx::Error),
}

impl<T: QueueMsgTypes> Queue<T> for AnyQueue<T> {
    type Error = AnyQueueError;
    type Config = AnyQueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(match cfg {
                AnyQueueConfig::InMemory => Self::InMemory(
                    InMemoryQueue::new(())
                        .await
                        .map_err(AnyQueueError::InMemory)?,
                ),
                AnyQueueConfig::PgQueue(cfg) => {
                    Self::PgQueue(PgQueue::new(cfg).await.map_err(AnyQueueError::PgQueue)?)
                }
            })
        }
    }

    fn enqueue(
        &mut self,
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async move {
            match self {
                AnyQueue::InMemory(queue) => {
                    queue.enqueue(item).await.map_err(AnyQueueError::InMemory)?
                }
                AnyQueue::PgQueue(queue) => {
                    queue.enqueue(item).await.map_err(AnyQueueError::PgQueue)?
                }
            };

            tracing::debug!("queued");

            Ok(())
        }
    }

    fn process<F, Fut, R>(
        &mut self,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
    {
        async move {
            let res = match self {
                AnyQueue::InMemory(queue) => {
                    queue.process(f).await.map_err(AnyQueueError::InMemory)
                }
                AnyQueue::PgQueue(queue) => queue.process(f).await.map_err(AnyQueueError::PgQueue),
            };

            tracing::debug!("processed");

            res
        }
    }
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct PgQueue<T: QueueMsgTypes>(pg_queue::Queue<QueueMsg<T>>, sqlx::PgPool);

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PgQueueConfig {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl PgQueueConfig {
    pub async fn into_pg_pool(self) -> sqlx::Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(self.max_connections.unwrap_or(10))
            .min_connections(self.min_connections.unwrap_or(0))
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .connect(&self.database_url)
            .await
    }
}

impl<T: QueueMsgTypes> Queue<T> for PgQueue<T> {
    type Error = sqlx::Error;

    type Config = PgQueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move { Ok(Self(pg_queue::Queue::new(), cfg.into_pg_pool().await?)) }
    }

    fn enqueue(
        &mut self,
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        self.0.enqueue(&self.1, item)
    }

    fn process<F, Fut, R>(
        &mut self,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<QueueMsg<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
    {
        self.0.process(&self.1, f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerInitError {
    #[error("multiple configured chains have the same chain id `{chain_id}`")]
    DuplicateChainId { chain_id: String },
    #[error("error initializing chain")]
    ChainInit(#[from] AnyChainTryFromConfigError),
    #[error("error initializing queue")]
    QueueInit(#[source] AnyQueueError),
}

impl Voyager {
    pub async fn new(config: Config) -> Result<Self, VoyagerInitError> {
        let chains = chains_from_config(config.chain).await?;

        let queue = AnyQueue::new(config.voyager.queue.clone())
            .await
            .map_err(VoyagerInitError::QueueInit)?;

        Ok(Self {
            chains: Arc::new(chains),
            num_workers: config.voyager.num_workers,
            queue,
        })
    }

    pub fn worker(&self) -> Reactor<RelayerMsgTypes> {
        Reactor::new(self.chains.clone())
    }

    pub async fn run(self) -> Result<(), RunError> {
        // set up msg server
        let (queue_tx, queue_rx) =
            futures::channel::mpsc::unbounded::<QueueMsg<VoyagerMessageTypes>>();

        let app = axum::Router::new()
            .route("/msg", post(msg))
            .route("/msgs", post(msgs))
            .route("/health", get(|| async move { StatusCode::OK }))
            .with_state(queue_tx.clone());

        // #[axum::debug_handler]
        async fn msg<T: QueueMsgTypes>(
            State(mut sender): State<UnboundedSender<QueueMsg<T>>>,
            Json(msg): Json<QueueMsg<T>>,
        ) -> StatusCode {
            tracing::info!(?msg, "received msg");
            sender.send(msg).await.expect("receiver should not close");

            StatusCode::OK
        }

        // #[axum::debug_handler]
        async fn msgs<T: QueueMsgTypes>(
            State(mut sender): State<UnboundedSender<QueueMsg<T>>>,
            Json(msgs): Json<Vec<QueueMsg<T>>>,
        ) -> StatusCode {
            tracing::info!(?msgs, "received msgs");
            for msg in msgs {
                sender.send(msg).await.expect("receiver should not close");
            }

            StatusCode::OK
        }

        tokio::spawn(
            // TODO: Make this configurable
            axum::Server::bind(&"0.0.0.0:65534".parse().expect("valid SocketAddr; qed;"))
                .serve(app.into_make_service()),
        );

        let mut join_set = JoinSet::<Result<(), BoxDynError>>::new();

        let mut q = self.queue.clone();
        join_set.spawn({
            async move {
                tracing::debug!("checking for new messages");

                pin_utils::pin_mut!(queue_rx);

                while let Some(msg) = queue_rx.next().await {
                    tracing::info!(
                        json = %serde_json::to_string(&msg).unwrap(),
                        "received new message",
                    );

                    q.enqueue(msg).await?;
                }

                Ok(())
            }
        });

        for i in 0..self.num_workers {
            tracing::info!("spawning worker {i}");

            let reactor = Reactor::new(self.chains.clone());
            let q = self.queue.clone();

            join_set.spawn(async move {
                reactor
                    .run(q)
                    .for_each(|x| async {
                        let msg = x.unwrap();

                        dbg!(msg);
                    })
                    .await;
                Ok(())
            });
        }

        let errs = vec![];

        // TODO: figure out
        while let Some(res) = join_set.join_next().await {
            res.unwrap().unwrap();
        }

        // while let Some(res) = join_set.join_next().await {
        //     match res {
        //         Ok(Ok(())) => {}
        //         Ok(Err(err)) => {
        //             tracing::error!(%err, "error running task");
        //             errs.push(err);
        //         }
        //         Err(err) => {
        //             tracing::error!(%err, "error running task");
        //             errs.push(Box::new(err));
        //         }
        //     }
        // }

        Err(RunError { errs })
    }
}

pub async fn chains_from_config(
    config: BTreeMap<String, ChainConfig>,
) -> Result<Chains, AnyChainTryFromConfigError> {
    let mut union = HashMap::new();
    let mut cosmos = HashMap::new();
    let mut evm_minimal = HashMap::new();
    let mut evm_mainnet = HashMap::new();
    let mut scroll = HashMap::new();

    fn insert_into_chain_map<C: Chain>(
        map: &mut HashMap<<<C as Chain>::SelfClientState as ClientState>::ChainId, C>,
        chain: C,
    ) {
        let chain_id = chain.chain_id();
        map.insert(chain_id.clone(), chain);

        tracing::info!(
            %chain_id,
            chain_type = <C::ChainType as FromStrExact>::EXPECTING,
            "registered chain"
        );
    }

    for (chain_name, chain_config) in config {
        if !chain_config.enabled {
            tracing::info!(%chain_name, "chain not enabled, skipping");
            continue;
        }

        let chain = AnyChain::try_from_config(chain_config.ty).await?;

        match chain {
            AnyChain::Union(c) => {
                insert_into_chain_map(&mut union, c);
            }
            AnyChain::Cosmos(c) => {
                insert_into_chain_map(&mut cosmos, c);
            }
            AnyChain::EvmMainnet(c) => {
                insert_into_chain_map(&mut evm_mainnet, c);
            }
            AnyChain::EvmMinimal(c) => {
                insert_into_chain_map(&mut evm_minimal, c);
            }
            AnyChain::Scroll(c) => {
                insert_into_chain_map(&mut scroll, c);
            }
        }
    }

    Ok(Chains {
        scroll,
        evm_minimal,
        evm_mainnet,
        union,
        cosmos,
    })
}

#[derive(Debug)]
pub struct RunError {
    errs: Vec<Box<dyn Error + Send + Sync>>,
}

impl Error for RunError {}

impl Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.errs {
            writeln!(f, "{err}")?
        }

        Ok(())
    }
}

fn chain_event_to_lc_event<Hc: ChainExt, Tr: ChainExt>(
    event: IbcEvent<Hc::ClientId, Hc::ClientType, String>,
) -> IbcEvent<Hc::ClientId, Hc::ClientType, Hc::ClientId>
where
    <ClientIdOf<Tr> as FromStr>::Err: Debug,
{
    match event {
        IbcEvent::CreateClient(CreateClient {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::CreateClient(CreateClient {
            client_id,
            client_type,
            consensus_height,
        }),
        IbcEvent::UpdateClient(UpdateClient {
            client_id,
            client_type,
            consensus_heights,
        }) => IbcEvent::UpdateClient(UpdateClient {
            client_id,
            client_type,
            consensus_heights,
        }),
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id,
            client_type,
            consensus_height,
        }),
        IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash }) => {
            IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash })
        }
        IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id,
            counterparty_client_id,
        }) => IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
        }),
        IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
            connection_id,
            client_id,
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version,
        }) => IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version,
        }),
        IbcEvent::ChannelOpenTry(ChannelOpenTry {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            version,
        }) => IbcEvent::ChannelOpenTry(ChannelOpenTry {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            version,
        }),
        IbcEvent::ChannelOpenAck(ChannelOpenAck {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }) => IbcEvent::ChannelOpenAck(ChannelOpenAck {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }),
        IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }) => IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
        }),
        IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_ack_hex,
            connection_id,
        }) => IbcEvent::WriteAcknowledgement(WriteAcknowledgement {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_ack_hex,
            connection_id,
        }),
        IbcEvent::RecvPacket(RecvPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::RecvPacket(RecvPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::SendPacket(SendPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::SendPacket(SendPacket {
            packet_data_hex,
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::AcknowledgePacket(AcknowledgePacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::AcknowledgePacket(AcknowledgePacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
        IbcEvent::TimeoutPacket(TimeoutPacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }) => IbcEvent::TimeoutPacket(TimeoutPacket {
            packet_timeout_height,
            packet_timeout_timestamp,
            packet_sequence,
            packet_src_port,
            packet_src_channel,
            packet_dst_port,
            packet_dst_channel,
            packet_channel_ordering,
            connection_id,
        }),
    }
}

pub enum ClientType {
    Wasm(WasmClientType),
    Tendermint,
}

pub struct VoyagerMessageTypes;

pub trait FromQueueMsg<T: QueueMsgTypes>: QueueMsgTypes + Sized {
    fn from_queue_msg(value: QueueMsg<T>) -> QueueMsg<Self>;
}

impl FromQueueMsg<RelayerMsgTypes> for VoyagerMessageTypes {
    fn from_queue_msg(value: QueueMsg<RelayerMsgTypes>) -> QueueMsg<Self> {
        match value {
            QueueMsg::Event(event) => QueueMsg::Event(VoyagerEvent::Relay(event)),
            QueueMsg::Data(data) => QueueMsg::Data(VoyagerData::Relay(data)),
            QueueMsg::Fetch(fetch) => QueueMsg::Fetch(VoyagerFetch::Relay(fetch)),
            QueueMsg::Msg(msg) => QueueMsg::Msg(VoyagerMsg::Relay(msg)),
            QueueMsg::Wait(wait) => QueueMsg::Wait(VoyagerWait::Relay(wait)),
            QueueMsg::DeferUntil { point, seconds } => QueueMsg::DeferUntil { point, seconds },
            QueueMsg::Repeat { times, msg } => QueueMsg::Repeat {
                times,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Timeout {
                timeout_timestamp,
                msg,
            } => QueueMsg::Timeout {
                timeout_timestamp,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Sequence(seq) => {
                QueueMsg::Sequence(seq.into_iter().map(Self::from_queue_msg).collect())
            }
            QueueMsg::Concurrent(seq) => {
                QueueMsg::Concurrent(seq.into_iter().map(Self::from_queue_msg).collect())
            }
            QueueMsg::Retry { remaining, msg } => QueueMsg::Retry {
                remaining,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Aggregate {
                queue,
                data,
                receiver,
            } => QueueMsg::Aggregate {
                queue: queue.into_iter().map(Self::from_queue_msg).collect(),
                data: data.into_iter().map(VoyagerData::Relay).collect(),
                receiver: VoyagerAggregate::Relay(receiver),
            },
            QueueMsg::Noop => QueueMsg::Noop,
        }
    }
}

impl FromQueueMsg<BlockPollingTypes> for VoyagerMessageTypes {
    fn from_queue_msg(value: QueueMsg<BlockPollingTypes>) -> QueueMsg<Self> {
        match value {
            QueueMsg::Data(data) => QueueMsg::Data(VoyagerData::Block(data)),
            QueueMsg::Fetch(fetch) => QueueMsg::Fetch(VoyagerFetch::Block(fetch)),
            QueueMsg::Wait(wait) => QueueMsg::Wait(VoyagerWait::Block(wait)),
            QueueMsg::DeferUntil { point, seconds } => QueueMsg::DeferUntil { point, seconds },
            QueueMsg::Repeat { times, msg } => QueueMsg::Repeat {
                times,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Timeout {
                timeout_timestamp,
                msg,
            } => QueueMsg::Timeout {
                timeout_timestamp,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Sequence(seq) => {
                QueueMsg::Sequence(seq.into_iter().map(Self::from_queue_msg).collect())
            }
            QueueMsg::Concurrent(seq) => {
                QueueMsg::Concurrent(seq.into_iter().map(Self::from_queue_msg).collect())
            }
            QueueMsg::Retry { remaining, msg } => QueueMsg::Retry {
                remaining,
                msg: Box::new(Self::from_queue_msg(*msg)),
            },
            QueueMsg::Aggregate {
                queue,
                data,
                receiver,
            } => QueueMsg::Aggregate {
                queue: queue.into_iter().map(Self::from_queue_msg).collect(),
                data: data.into_iter().map(VoyagerData::Block).collect(),
                receiver: VoyagerAggregate::Block(receiver),
            },
            QueueMsg::Noop => QueueMsg::Noop,
        }
    }
}

impl QueueMsgTypes for VoyagerMessageTypes {
    type Event = VoyagerEvent;
    type Data = VoyagerData;
    type Fetch = VoyagerFetch;
    type Msg = VoyagerMsg;
    type Wait = VoyagerWait;
    type Aggregate = VoyagerAggregate;

    type Store = Chains;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerEvent {
    Block(<BlockPollingTypes as QueueMsgTypes>::Event),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Event),
}

impl HandleEvent<VoyagerMessageTypes> for VoyagerEvent {
    fn handle(
        self,
        store: &<VoyagerMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Relay(event) => {
                <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    HandleEvent::handle(event, store)?,
                )
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerData {
    Block(<BlockPollingTypes as QueueMsgTypes>::Data),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Data),
}

impl HandleData<VoyagerMessageTypes> for VoyagerData {
    fn handle(
        self,
        store: &<VoyagerMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Block(data) => match data.handle(store)? {
                QueueMsg::Data(block_message::AnyChainIdentified::Cosmos(
                    block_message::Identified {
                        chain_id,
                        t: block_message::data::Data::IbcEvent(ibc_event),
                    },
                )) => <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    match ibc_event.client_type {
                        unionlabs::ClientType::Wasm(unionlabs::WasmClientType::Cometbls) => {
                            event::<RelayerMsgTypes>(relay_message::id::<Wasm<Cosmos>, Union, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Wasm<Cosmos>, Union>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        _ => unimplemented!(),
                    },
                ),
                QueueMsg::Data(block_message::AnyChainIdentified::Union(
                    block_message::Identified {
                        chain_id,
                        t: block_message::data::Data::IbcEvent(ibc_event),
                    },
                )) => <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    match ibc_event.client_type {
                        unionlabs::ClientType::Wasm(unionlabs::WasmClientType::EthereumMinimal) => {
                            event(relay_message::id::<Wasm<Union>, Evm<Minimal>, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Union, Evm<Minimal>>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        unionlabs::ClientType::Wasm(unionlabs::WasmClientType::EthereumMainnet) => {
                            event(relay_message::id::<Wasm<Union>, Evm<Mainnet>, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Union, Evm<Mainnet>>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        unionlabs::ClientType::Tendermint => {
                            event(relay_message::id::<Union, Wasm<Cosmos>, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Union, Wasm<Cosmos>>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        _ => unimplemented!(),
                    },
                ),
                QueueMsg::Data(block_message::AnyChainIdentified::EthMainnet(
                    block_message::Identified {
                        chain_id,
                        t: block_message::data::Data::IbcEvent(ibc_event),
                    },
                )) => <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    match ibc_event.client_type {
                        unionlabs::ClientType::Cometbls => {
                            event(relay_message::id::<Evm<Mainnet>, Wasm<Union>, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Evm<Mainnet>, Wasm<Union>>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        _ => unimplemented!(),
                    },
                ),
                QueueMsg::Data(block_message::AnyChainIdentified::EthMinimal(
                    block_message::Identified {
                        chain_id,
                        t: block_message::data::Data::IbcEvent(ibc_event),
                    },
                )) => <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    match ibc_event.client_type {
                        unionlabs::ClientType::Cometbls => {
                            event(relay_message::id::<Evm<Minimal>, Wasm<Union>, _>(
                                chain_id,
                                relay_message::event::IbcEvent {
                                    tx_hash: ibc_event.tx_hash,
                                    height: ibc_event.height,
                                    event: chain_event_to_lc_event::<Evm<Minimal>, Wasm<Union>>(
                                        ibc_event.event,
                                    ),
                                },
                            ))
                        }
                        _ => unimplemented!(),
                    },
                ),
                msg => {
                    <VoyagerMessageTypes as FromQueueMsg<BlockPollingTypes>>::from_queue_msg(msg)
                }
            },
            Self::Relay(data) => {
                <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    data.handle(store)?,
                )
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerFetch {
    Block(<BlockPollingTypes as QueueMsgTypes>::Fetch),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Fetch),
}

impl HandleFetch<VoyagerMessageTypes> for VoyagerFetch {
    async fn handle(
        self,
        store: &<VoyagerMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Block(fetch) => {
                <VoyagerMessageTypes as FromQueueMsg<BlockPollingTypes>>::from_queue_msg(
                    fetch.handle(store).await?,
                )
            }
            Self::Relay(fetch) => {
                <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    fetch.handle(store).await?,
                )
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerMsg {
    Block(<BlockPollingTypes as QueueMsgTypes>::Msg),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Msg),
}

impl HandleMsg<VoyagerMessageTypes> for VoyagerMsg {
    async fn handle(
        self,
        store: &<VoyagerMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Relay(msg) => {
                <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    msg.handle(store).await?,
                )
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerWait {
    Block(<BlockPollingTypes as QueueMsgTypes>::Wait),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Wait),
}

impl HandleWait<VoyagerMessageTypes> for VoyagerWait {
    async fn handle(
        self,
        store: &<VoyagerMessageTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Block(msg) => {
                <VoyagerMessageTypes as FromQueueMsg<BlockPollingTypes>>::from_queue_msg(
                    HandleWait::<BlockPollingTypes>::handle(msg, store).await?,
                )
            }
            Self::Relay(msg) => {
                <VoyagerMessageTypes as FromQueueMsg<RelayerMsgTypes>>::from_queue_msg(
                    HandleWait::<RelayerMsgTypes>::handle(msg, store).await?,
                )
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(tag = "@type", content = "@value", rename_all = "snake_case")]
pub enum VoyagerAggregate {
    Block(<BlockPollingTypes as QueueMsgTypes>::Aggregate),
    Relay(<RelayerMsgTypes as QueueMsgTypes>::Aggregate),
}

impl HandleAggregate<VoyagerMessageTypes> for VoyagerAggregate {
    fn handle(
        self,
        data: VecDeque<<VoyagerMessageTypes as QueueMsgTypes>::Data>,
    ) -> Result<QueueMsg<VoyagerMessageTypes>, QueueError> {
        Ok(match self {
            Self::Block(aggregate) => VoyagerMessageTypes::from_queue_msg(
                aggregate.handle(
                    data.into_iter()
                        .map(|d| match d {
                            VoyagerData::Block(d) => d,
                            VoyagerData::Relay(_) => panic!(),
                        })
                        .collect(),
                )?,
            ),
            Self::Relay(aggregate) => VoyagerMessageTypes::from_queue_msg(
                aggregate.handle(
                    data.into_iter()
                        .map(|d| match d {
                            VoyagerData::Block(_) => panic!(),
                            VoyagerData::Relay(d) => d,
                        })
                        .collect(),
                )?,
            ),
        })
    }
}
