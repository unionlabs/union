use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use chain_utils::{
    evm::Evm,
    union::{Union, UnionClientId},
    EventSource,
};
use futures::{stream, Future, FutureExt, StreamExt, TryStreamExt};
use lightclient::{
    cometbls::{CometblsMainnet, CometblsMinimal},
    ethereum::{EthereumMainnet, EthereumMinimal},
};
use pg_queue::ProcessFlow;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use unionlabs::{
    ethereum_consts_traits::{Mainnet, Minimal},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    traits::{Chain, ChainIdOf, ChainOf, LightClientBase},
    WasmClientType,
};
use voyager_message::{event, GetLc, LightClient, RelayerMsg};

use crate::{
    chain::{AnyChain, AnyChainTryFromConfigError},
    config::Config,
};

pub mod msg_server;

#[derive(Debug, Clone)]
pub struct Voyager<Q> {
    chains: Arc<Chains>,
    num_workers: u16,
    msg_server: msg_server::MsgServer,
    queue: Q,
}

#[derive(Debug, Clone)]
pub struct Worker {
    pub id: u16,
    pub chains: Arc<Chains>,
}

#[derive(Debug, Clone)]
pub struct Chains {
    // TODO: Use some sort of typemap here instead of individual fields
    evm_minimal: HashMap<ChainIdOf<Evm<Minimal>>, Evm<Minimal>>,
    evm_mainnet: HashMap<ChainIdOf<Evm<Mainnet>>, Evm<Mainnet>>,
    union: HashMap<ChainIdOf<Union>, Union>,
}

pub trait Queue: Clone + Send + Sync + Sized + 'static {
    /// Error type returned by this queue, representing errors that are out of control of the consumer (i.e. unable to connect to database, can't insert into row, can't deserialize row, etc)
    type Error: Debug + Display + Error + Send + Sync + 'static;
    type Config: Debug + Clone + Serialize + DeserializeOwned;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>>;

    fn enqueue(
        &mut self,
        item: RelayerMsg,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;

    fn process<F, Fut>(
        &mut self,
        f: F,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(RelayerMsg) -> Fut) + Send + 'static,
        Fut: Future<Output = ProcessFlow<RelayerMsg>> + Send + 'static;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum AnyQueueConfig {
    InMemory,
    PgQueue(<PgQueue as Queue>::Config),
}

#[derive(Debug, Clone)]
pub enum AnyQueue {
    InMemory(InMemoryQueue),
    PgQueue(PgQueue),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(#[from] <InMemoryQueue as Queue>::Error),
    PgQueue(#[from] <PgQueue as Queue>::Error),
}

impl Queue for AnyQueue {
    type Error = AnyQueueError;
    type Config = AnyQueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(match cfg {
                AnyQueueConfig::InMemory => Self::InMemory(InMemoryQueue::new(()).await?),
                AnyQueueConfig::PgQueue(cfg) => Self::PgQueue(PgQueue::new(cfg).await?),
            })
        }
    }

    fn enqueue(
        &mut self,
        item: RelayerMsg,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        async move {
            match self {
                AnyQueue::InMemory(queue) => queue.enqueue(item).await?,
                AnyQueue::PgQueue(queue) => queue.enqueue(item).await?,
            };

            Ok(())
        }
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(RelayerMsg) -> Fut) + Send + 'static,
        Fut: Future<Output = ProcessFlow<RelayerMsg>> + Send + 'static,
    {
        async move {
            match self {
                AnyQueue::InMemory(queue) => queue.process(f).await?,
                AnyQueue::PgQueue(queue) => queue.process(f).await?,
            };

            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryQueue(Arc<Mutex<VecDeque<RelayerMsg>>>);

impl Queue for InMemoryQueue {
    type Error = std::convert::Infallible;
    type Config = ();

    fn new(_cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        futures::future::ok(Self(Arc::new(Mutex::new(VecDeque::default()))))
    }

    fn enqueue(
        &mut self,
        item: RelayerMsg,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        self.0.lock().expect("mutex is poisoned").push_back(item);
        futures::future::ok(())
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(RelayerMsg) -> Fut) + Send + 'static,
        Fut: Future<Output = ProcessFlow<RelayerMsg>> + Send + 'static,
    {
        async move {
            let msg = {
                let mut queue = self.0.lock().expect("mutex is poisoned");
                let msg = queue.pop_front();

                drop(queue);

                msg
            };

            match msg {
                Some(msg) => {
                    tracing::info!(
                        json = %serde_json::to_string(&msg).unwrap(),
                    );

                    match f(msg.clone()).await {
                        ProcessFlow::Success(new_msgs) => {
                            let mut queue = self.0.lock().expect("mutex is poisoned");
                            queue.extend(new_msgs);
                            Ok(())
                        }
                        ProcessFlow::Requeue => {
                            let mut queue = self.0.lock().expect("mutex is poisoned");
                            queue.push_front(msg);
                            Ok(())
                        }
                        ProcessFlow::Fail(why) => panic!("{why}"),
                    }
                }
                None => Ok(()),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PgQueue(pg_queue::Queue<RelayerMsg>, sqlx::PgPool);

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PgQueueConfig {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl Queue for PgQueue {
    type Error = sqlx::Error;

    type Config = PgQueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(Self(
                pg_queue::Queue::new(),
                // 10 is the default
                PgPoolOptions::new()
                    .max_connections(cfg.max_connections.unwrap_or(10))
                    .min_connections(cfg.min_connections.unwrap_or(0))
                    .idle_timeout(cfg.idle_timeout)
                    .max_lifetime(cfg.max_lifetime)
                    .connect(&cfg.database_url)
                    .await?,
            ))
        }
    }

    fn enqueue(
        &mut self,
        item: RelayerMsg,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        self.0.enqueue(&self.1, item)
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(RelayerMsg) -> Fut) + Send + 'static,
        Fut: Future<Output = ProcessFlow<RelayerMsg>> + Send + 'static,
    {
        self.0.process(&self.1, f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerInitError<Q: Queue> {
    #[error("multiple configured chains have the same chain id `{chain_id}`")]
    DuplicateChainId { chain_id: String },
    #[error("error initializing chain")]
    ChainInit(#[from] AnyChainTryFromConfigError),
    #[error("error initializing queue")]
    QueueInit(#[source] Q::Error),
}

impl<Q: Queue> Voyager<Q> {
    fn worker(&self, id: u16) -> Worker {
        Worker {
            id,
            chains: self.chains.clone(),
        }
    }

    pub async fn new(config: Config<Q>) -> Result<Self, VoyagerInitError<Q>> {
        if config.voyager.hasura.is_none() {
            tracing::warn!("no hasura config supplied, no messages will be indexed");
        }

        let mut union = HashMap::new();
        let mut evm_minimal = HashMap::new();
        let mut evm_mainnet = HashMap::new();

        fn insert_into_chain_map<C: Chain, Q: Queue>(
            map: &mut HashMap<ChainIdOf<C>, C>,
            chain: C,
        ) -> Result<ChainIdOf<C>, VoyagerInitError<Q>> {
            let chain_id = chain.chain_id();
            map.insert(chain_id.clone(), chain)
                .map_or(Ok(chain_id), |prev| {
                    Err(VoyagerInitError::DuplicateChainId {
                        chain_id: prev.chain_id().to_string(),
                    })
                })
        }

        for (chain_name, chain_config) in config.chain {
            let chain = AnyChain::try_from_config(&config.voyager, chain_config).await?;

            match chain {
                AnyChain::Union(c) => {
                    let chain_id = insert_into_chain_map(&mut union, c)?;

                    tracing::info!(
                        chain_name,
                        chain_id,
                        chain_type = "Union",
                        "registered chain"
                    );
                }
                AnyChain::EvmMainnet(c) => {
                    let chain_id = insert_into_chain_map(&mut evm_mainnet, c)?;

                    tracing::info!(
                        chain_name,
                        %chain_id,
                        chain_type = "EvmMainnet",
                        "registered chain"
                    );
                }
                AnyChain::EvmMinimal(c) => {
                    let chain_id = insert_into_chain_map(&mut evm_minimal, c)?;

                    tracing::info!(
                        chain_name,
                        %chain_id,
                        chain_type = "EvmMinimal",
                        "registered chain"
                    );
                }
            }
        }

        let queue = Q::new(config.voyager.queue)
            .await
            .map_err(VoyagerInitError::QueueInit)?;

        Ok(Self {
            chains: Arc::new(Chains {
                evm_minimal,
                evm_mainnet,
                union,
            }),
            msg_server: msg_server::MsgServer,
            num_workers: config.voyager.num_workers,
            queue,
        })
    }

    pub async fn run(self) -> Result<(), RunError> {
        let union_events = stream::iter(self.chains.union.clone())
            .map(|(chain_id, chain)| {
                chain
                    .clone()
                    .events(())
                    .and_then(move |chain_event| {
                        let c = chain.clone();
                        let expected_chain_id = chain_id.clone();

                        async move {
                            if expected_chain_id != chain_event.chain_id {
                                tracing::warn!(
                                    "chain {expected_chain_id} produced an event from chain {}",
                                    chain_event.chain_id.clone()
                                );
                            }

                            Ok(
                                match client_type_from_ibc_event(&c, &chain_event.event).await {
                                    WasmClientType::EthereumMinimal => event::<EthereumMinimal>(
                                        chain_event.chain_id,
                                        voyager_message::event::IbcEvent {
                                            block_hash: chain_event.block_hash,
                                            height: chain_event.height,
                                            event: chain_event_to_lc_event::<EthereumMinimal>(
                                                chain_event.event,
                                            ),
                                        },
                                    ),
                                    WasmClientType::EthereumMainnet => event::<EthereumMainnet>(
                                        chain_event.chain_id,
                                        voyager_message::event::IbcEvent {
                                            block_hash: chain_event.block_hash,
                                            height: chain_event.height,
                                            event: chain_event_to_lc_event::<EthereumMainnet>(
                                                chain_event.event,
                                            ),
                                        },
                                    ),
                                    WasmClientType::Cometbls => unimplemented!(),
                                },
                            )
                        }
                    })
                    .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
            })
            .flatten()
            .boxed();
        let mut events = Box::pin(stream::select_all([
            stream::iter(self.chains.evm_minimal.clone())
                .map(|(chain_id, chain)| {
                    chain
                        .events(())
                        .map_ok(move |chain_event| {
                            if chain_id != chain_event.chain_id {
                                tracing::warn!(
                                    "chain {chain_id} produced an event from chain {}",
                                    chain_event.chain_id
                                );
                            }

                            event::<CometblsMinimal>(
                                chain_event.chain_id,
                                voyager_message::event::IbcEvent {
                                    block_hash: chain_event.block_hash,
                                    height: chain_event.height,
                                    event: chain_event_to_lc_event::<CometblsMinimal>(
                                        chain_event.event,
                                    ),
                                },
                            )
                        })
                        .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
                })
                .flatten()
                .boxed(),
            stream::iter(self.chains.evm_mainnet.clone())
                .map(|(chain_id, chain)| {
                    chain
                        .events(())
                        .map_ok(move |chain_event| {
                            if chain_id != chain_event.chain_id {
                                tracing::warn!(
                                    "chain {chain_id} produced an event from chain {}",
                                    chain_event.chain_id
                                );
                            }

                            event::<CometblsMainnet>(
                                chain_event.chain_id,
                                voyager_message::event::IbcEvent {
                                    block_hash: chain_event.block_hash,
                                    height: chain_event.height,
                                    event: chain_event_to_lc_event::<CometblsMainnet>(
                                        chain_event.event,
                                    ),
                                },
                            )
                        })
                        .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
                })
                .flatten()
                .boxed(),
            union_events,
            self.msg_server
                .clone()
                .events(())
                .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
                .boxed(),
        ]));

        let cancellation_token = CancellationToken::new();

        let mut join_set = JoinSet::new();

        let mut q = self.queue.clone();
        join_set.spawn({
            let ct = cancellation_token.clone();
            async move {
                tracing::debug!("checking for new messages");

                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
                            if ct.is_cancelled() {
                                tracing::info!("shutting down event listener");
                                break;
                            }
                        }
                        msg = events.select_next_some() => {
                            let msg = msg.map_err(|x| format!("{x:?}"))?;

                            tracing::info!(
                                json = %serde_json::to_string(&msg).unwrap(),
                                "received new message",
                            );

                            q.enqueue(msg).await?;
                        }
                    }
                }

                Ok(())
            }
        });

        for i in 0..self.num_workers {
            tracing::info!("spawning worker {i}");

            let worker = self.worker(i);

            join_set.spawn(worker.run(cancellation_token.child_token(), self.queue.clone()));
        }

        let mut errs = vec![];

        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    tracing::error!(%err, "error running task");
                    cancellation_token.cancel();
                    errs.push(err);
                }
                Err(err) => {
                    tracing::error!(%err, "error running task");
                    cancellation_token.cancel();
                    errs.push(Box::new(err));
                }
            }
        }

        Err(RunError { errs })
    }
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

impl Worker {
    fn run<Q: Queue>(
        self,
        ct: CancellationToken,
        mut q: Q,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static {
        async move {
            loop {
                if ct.is_cancelled() {
                    tracing::info!(worker = self.id, "shutting down");
                    break Ok(());
                } else {
                    let worker = self.clone();
                    q.process(move |msg| async move {
                        let new_msgs = msg.handle(&worker, 0).await;

                        match new_msgs {
                            Ok(ok) => ProcessFlow::Success(ok),
                            // REVIEW: Check if this error is recoverable or not - i.e. if this is an IO error,
                            // the msg can likely be retried
                            Err(err) => {
                                // ProcessFlow::Fail(err.to_string())
                                // HACK: panic is OK here since this is spawned in a task, and will be caught by the runtime worker
                                panic!("{err}");
                            }
                        }
                    })
                    .await?;
                }
            }
        }
    }
}

// pub enum AnyLcError_ {}

// impl AnyLightClient for AnyLcError_ {}

// TODO: Implement this on Chains, not Worker
impl GetLc<CometblsMinimal> for Worker {
    fn get_lc(&self, chain_id: &ChainIdOf<ChainOf<CometblsMinimal>>) -> CometblsMinimal {
        CometblsMinimal::from_chain(self.chains.evm_minimal.get(chain_id).unwrap().clone())
    }
}

impl GetLc<CometblsMainnet> for Worker {
    fn get_lc(&self, chain_id: &ChainIdOf<ChainOf<CometblsMainnet>>) -> CometblsMainnet {
        CometblsMainnet::from_chain(self.chains.evm_mainnet.get(chain_id).unwrap().clone())
    }
}

impl GetLc<EthereumMinimal> for Worker {
    fn get_lc(&self, chain_id: &ChainIdOf<ChainOf<EthereumMinimal>>) -> EthereumMinimal {
        // TODO: Ensure that the wasm code is for the correct config
        EthereumMinimal::from_chain(self.chains.union.get(chain_id).unwrap().clone())
    }
}

impl GetLc<EthereumMainnet> for Worker {
    fn get_lc(&self, chain_id: &ChainIdOf<ChainOf<EthereumMainnet>>) -> EthereumMainnet {
        // TODO: Ensure that the wasm code is for the correct config
        EthereumMainnet::from_chain(self.chains.union.get(chain_id).unwrap().clone())
    }
}

// /// For updating a client, the information we have originally is:
// ///
// /// - `chain_id`: the id of the chain that the client to be updated is on
// /// - `height`: the height to update *to*
// /// - `client_id`: id of the client to update
// /// - `counterparty_client_id`: id of the counterparty of the client to update
// ///
// /// Given this information, multiple aggregations are required:
// ///
// /// - given (`chain_id`, `client_id`), fetch the counterparty client's `chain_id`
// ///   (contained within the client's client state)
// ///   - `FetchLatestTrustedClientState<L>`, aggregated down into `UpdateClientData<L>`,
// ///     producing `UpdateClientWithCounterpartyChainIdData<L>`
// ///
// /// - then, with (`counterparty_chain_id`, `counterparty_client_id`), fetch the latest
// ///   client state of the counterparty client (which contains the latest trusted height)
// ///   - `FetchLatestTrustedClientState<L::Counterparty>`, aggregated down into
// ///     `UpdateClientWithCounterpartyChainIdData<L>`, producing `FetchUpdateHeaders<L>`
// ///
// /// - finally, with the latest client state, build the headers between
// ///   `latest_client_state..=update_to` (note that the client may be updated to a height
// ///   greater than `update_to`, but never less; as such the latest trusted height should
// ///   always be fetched whenever it's needed)
// ///   - `FetchUpdateHeaders<L>`, which delegates to `L::generate_counterparty_updates`
// fn mk_aggregate_update<L: LightClient>(
//     chain_id: ChainIdOf<ChainOf<L>>,
//     client_id: L::ClientId,
//     counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
//     event_height: HeightOf<ChainOf<L>>,
// ) -> RelayerMsg
// where
//     AnyLightClientIdentified<AnyLcMsg>: From<identified!(LcMsg<L>)>,
//     AggregateReceiver: From<identified!(Aggregate<L>)>,
// {
//     RelayerMsg::Aggregate {
//         queue: [fetch::<L>(
//             chain_id.clone(),
//             FetchTrustedClientState {
//                 at: QueryHeight::Latest,
//                 client_id: client_id.clone(),
//             },
//         )]
//         .into(),
//         data: [].into(),
//         receiver: AggregateReceiver::from(Identified::new(
//             chain_id,
//             Aggregate::<L>::UpdateClient(AggregateUpdateClient {
//                 // Proof is only valid at N + 1 for tendermint
//                 update_to: event_height.increment(),
//                 client_id: client_id.clone(),
//                 counterparty_client_id,
//             }),
//         )),
//     }
// }

fn chain_event_to_lc_event<L: LightClient>(
    event: IbcEvent<<L::HostChain as Chain>::ClientId, <L::HostChain as Chain>::ClientType, String>,
) -> IbcEvent<L::ClientId, L::ClientType, <L::Counterparty as LightClientBase>::ClientId>
where
    <L::ClientId as TryFrom<<L::HostChain as Chain>::ClientId>>::Error: Debug,
    <L::ClientType as TryFrom<<L::HostChain as Chain>::ClientType>>::Error: Debug,
    <<L::Counterparty as LightClientBase>::ClientId as FromStr>::Err: Debug,
{
    match event {
        IbcEvent::CreateClient(CreateClient {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::CreateClient(CreateClient {
            client_id: client_id.try_into().unwrap(),
            client_type: client_type.try_into().unwrap(),
            consensus_height,
        }),
        IbcEvent::UpdateClient(UpdateClient {
            client_id,
            client_type,
            consensus_heights,
            header,
        }) => IbcEvent::UpdateClient(UpdateClient {
            client_id: client_id.try_into().unwrap(),
            client_type: client_type.try_into().unwrap(),
            consensus_heights,
            header,
        }),
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id,
            client_type,
            consensus_height,
        }) => IbcEvent::ClientMisbehaviour(ClientMisbehaviour {
            client_id: client_id.try_into().unwrap(),
            client_type: client_type.try_into().unwrap(),
            consensus_height,
        }),
        IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash }) => {
            IbcEvent::SubmitEvidence(SubmitEvidence { evidence_hash })
        }
        IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
            connection_id,
            client_id: client_id.try_into().unwrap(),
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }) => IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
            connection_id,
            client_id: client_id.try_into().unwrap(),
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
            client_id: client_id.try_into().unwrap(),
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
            client_id: client_id.try_into().unwrap(),
            counterparty_client_id: counterparty_client_id.parse().unwrap(),
            counterparty_connection_id,
        }),
        IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_port_id,
            connection_id,
            version,
        }) => IbcEvent::ChannelOpenInit(ChannelOpenInit {
            port_id,
            channel_id,
            counterparty_channel_id,
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

async fn client_type_from_ibc_event(
    union: &Union,
    ibc_event: &IbcEvent<UnionClientId, String, String>,
) -> WasmClientType {
    match ibc_event {
        IbcEvent::CreateClient(CreateClient { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::UpdateClient(UpdateClient { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::SubmitEvidence(SubmitEvidence { .. }) => {
            // TODO: Not sure how to handle this one, since it only contains the hash
            // union
            //     .code_id_of_client_id(client_id)
            //     .then(|code_id| union.client_type_of_code_id(code_id))
            //     .await
            panic!()
        }
        IbcEvent::ConnectionOpenInit(ConnectionOpenInit { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ConnectionOpenTry(ConnectionOpenTry { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ConnectionOpenAck(ConnectionOpenAck { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm { client_id, .. }) => {
            union
                .code_id_of_client_id(client_id.clone())
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ChannelOpenInit(ChannelOpenInit { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ChannelOpenTry(ChannelOpenTry { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ChannelOpenAck(ChannelOpenAck { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::WriteAcknowledgement(WriteAcknowledgement { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::RecvPacket(RecvPacket { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::SendPacket(SendPacket { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::AcknowledgePacket(AcknowledgePacket { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
        IbcEvent::TimeoutPacket(TimeoutPacket { connection_id, .. }) => {
            union
                .client_id_of_connection(connection_id.clone())
                .then(|client_id| union.code_id_of_client_id(client_id))
                .then(|code_id| union.client_type_of_code_id(code_id))
                .await
        }
    }
}
