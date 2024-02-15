#![allow(clippy::type_complexity)]

use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fmt::{Debug, Display},
    str::FromStr,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use chain_utils::{
    cosmos::Cosmos,
    cosmos_sdk::{CosmosSdkChain, CosmosSdkChainExt},
    evm::Evm,
    union::Union,
    EventSource,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{stream, Future, FutureExt, StreamExt, TryStreamExt};
use queue_msg::{event, Queue, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinSet;
use unionlabs::{
    ethereum::config::{Mainnet, Minimal},
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ClientMisbehaviour, ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit,
        ConnectionOpenTry, CreateClient, IbcEvent, RecvPacket, SendPacket, SubmitEvidence,
        TimeoutPacket, UpdateClient, WriteAcknowledgement,
    },
    id::ClientId,
    traits::{Chain, ClientIdOf, ClientState, FromStrExact},
    WasmClientType,
};
use voyager_message::{ChainExt, Chains, Identified, RelayerMsgTypes, Wasm};

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
    // NOTE: pub temporarily
    pub relay_queue: Q,
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct Worker<T: QueueMsgTypes> {
    pub id: u16,
    pub store: Arc<T::Store>,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(
    rename_all = "kebab-case",
    tag = "type",
    bound(serialize = "", deserialize = "")
)]
pub enum AnyQueueConfig<T: QueueMsgTypes> {
    InMemory,
    PgQueue(<PgQueue<T> as Queue<T>>::Config),
}

#[derive(DebugNoBound, CloneNoBound)]
pub enum AnyQueue<T: QueueMsgTypes> {
    InMemory(InMemoryQueue<T>),
    PgQueue(PgQueue<T>),
}

#[derive(DebugNoBound, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError<T: QueueMsgTypes> {
    InMemory(<InMemoryQueue<T> as Queue<T>>::Error),
    PgQueue(<PgQueue<T> as Queue<T>>::Error),
}

impl<T: QueueMsgTypes> Queue<T> for AnyQueue<T> {
    type Error = AnyQueueError<T>;
    type Config = AnyQueueConfig<T>;

    fn new(cfg: Self::Config, topic: String) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(match cfg {
                AnyQueueConfig::InMemory => Self::InMemory(
                    InMemoryQueue::new((), topic)
                        .await
                        .map_err(AnyQueueError::InMemory)?,
                ),
                AnyQueueConfig::PgQueue(cfg) => Self::PgQueue(
                    PgQueue::new(cfg, topic)
                        .await
                        .map_err(AnyQueueError::PgQueue)?,
                ),
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

            Ok(())
        }
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = Result<Option<QueueMsg<T>>, String>> + Send + 'static,
    {
        async move {
            match self {
                AnyQueue::InMemory(queue) => {
                    queue.process(f).await.map_err(AnyQueueError::InMemory)?
                }
                AnyQueue::PgQueue(queue) => {
                    queue.process(f).await.map_err(AnyQueueError::PgQueue)?
                }
            };

            Ok(())
        }
    }
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct InMemoryQueue<T: QueueMsgTypes>(Arc<Mutex<VecDeque<QueueMsg<T>>>>);

impl<T: QueueMsgTypes> Queue<T> for InMemoryQueue<T> {
    type Error = std::convert::Infallible;
    type Config = ();

    fn new(_cfg: Self::Config, _topic: String) -> impl Future<Output = Result<Self, Self::Error>> {
        futures::future::ok(Self(Arc::new(Mutex::new(VecDeque::default()))))
    }

    fn enqueue(
        &mut self,
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        tracing::warn!(%item, "enqueueing new item");
        self.0.lock().expect("mutex is poisoned").push_back(item);
        futures::future::ok(())
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = Result<Option<QueueMsg<T>>, String>> + Send + 'static,
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
                        Ok(new_msg) => {
                            if let Some(new_msg) = new_msg {
                                let mut queue = self.0.lock().expect("mutex is poisoned");
                                queue.push_back(new_msg);
                            }
                            Ok(())
                        }
                        // ProcessFlow::Requeue => {
                        //     let mut queue = self.0.lock().expect("mutex is poisoned");
                        //     queue.push_front(msg);
                        //     Ok(())
                        // }
                        Err(why) => panic!("{why}"),
                    }
                }
                None => Ok(()),
            }
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

impl<T: QueueMsgTypes> Queue<T> for PgQueue<T> {
    type Error = sqlx::Error;

    type Config = PgQueueConfig;

    fn new(cfg: Self::Config, topic: String) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(Self(
                pg_queue::Queue::new(topic),
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
        item: QueueMsg<T>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + '_ {
        self.0.enqueue(&self.1, item)
    }

    fn process<F, Fut>(&mut self, f: F) -> impl Future<Output = Result<(), Self::Error>> + Send + '_
    where
        F: (FnOnce(QueueMsg<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = Result<Option<QueueMsg<T>>, String>> + Send + 'static,
    {
        self.0.process(&self.1, f)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerInitError<Q: Queue<RelayerMsgTypes>> {
    #[error("multiple configured chains have the same chain id `{chain_id}`")]
    DuplicateChainId { chain_id: String },
    #[error("error initializing chain")]
    ChainInit(#[from] AnyChainTryFromConfigError),
    #[error("error initializing queue")]
    QueueInit(#[source] Q::Error),
}

impl<Q: Queue<RelayerMsgTypes>> Voyager<Q> {
    pub fn worker(&self, id: u16) -> Worker<RelayerMsgTypes> {
        Worker {
            id,
            store: self.chains.clone(),
        }
    }

    pub async fn new(config: Config<Q>) -> Result<Self, VoyagerInitError<Q>> {
        let mut union = HashMap::new();
        let mut cosmos = HashMap::new();
        let mut evm_minimal = HashMap::new();
        let mut evm_mainnet = HashMap::new();

        fn insert_into_chain_map<C: Chain, Q: Queue<RelayerMsgTypes>>(
            map: &mut HashMap<<<C as Chain>::SelfClientState as ClientState>::ChainId, C>,
            chain: C,
        ) -> Result<(), VoyagerInitError<Q>> {
            let chain_id = chain.chain_id();
            let chain_id = map
                .insert(chain_id.clone(), chain)
                .map_or(Ok(chain_id), |prev| {
                    Err(VoyagerInitError::DuplicateChainId {
                        chain_id: prev.chain_id().to_string(),
                    })
                })?;

            tracing::info!(
                %chain_id,
                chain_type = <C::ChainType as FromStrExact>::EXPECTING,
                "registered chain"
            );

            Ok(())
        }

        for (chain_name, chain_config) in config.chain {
            if !chain_config.enabled {
                tracing::info!(%chain_name, "chain not enabled, skipping");
                continue;
            }

            let chain = AnyChain::try_from_config(chain_config.ty).await?;

            match chain {
                AnyChain::Union(c) => {
                    insert_into_chain_map(&mut union, c)?;
                }
                AnyChain::Cosmos(c) => {
                    insert_into_chain_map(&mut cosmos, c)?;
                }
                AnyChain::EvmMainnet(c) => {
                    insert_into_chain_map(&mut evm_mainnet, c)?;
                }
                AnyChain::EvmMinimal(c) => {
                    insert_into_chain_map(&mut evm_minimal, c)?;
                }
            }
        }

        let queue = Q::new(config.voyager.queue, "relay".to_owned())
            .await
            .map_err(VoyagerInitError::QueueInit)?;

        Ok(Self {
            chains: Arc::new(Chains {
                evm_minimal,
                evm_mainnet,
                union,
                cosmos,
            }),
            msg_server: msg_server::MsgServer,
            num_workers: config.voyager.num_workers,
            relay_queue: queue,
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
                                    ClientType::Wasm(WasmClientType::EthereumMinimal) => {
                                        event(
                                            Identified::<Wasm<Union>, Evm<Minimal>, _>::new(
                                                chain_event.chain_id,
                                                voyager_message::event::IbcEvent {
                                                    block_hash: chain_event.block_hash,
                                                    height: chain_event.height,
                                                    event: chain_event_to_lc_event::<
                                                        Union,
                                                        Evm<Minimal>,
                                                    >(
                                                        chain_event.event
                                                    ),
                                                },
                                            ),
                                        )
                                    }
                                    ClientType::Wasm(WasmClientType::EthereumMainnet) => {
                                        event(
                                            Identified::<Wasm<Union>, Evm<Mainnet>, _>::new(
                                                chain_event.chain_id,
                                                voyager_message::event::IbcEvent {
                                                    block_hash: chain_event.block_hash,
                                                    height: chain_event.height,
                                                    event: chain_event_to_lc_event::<
                                                        Union,
                                                        Evm<Mainnet>,
                                                    >(
                                                        chain_event.event
                                                    ),
                                                },
                                            ),
                                        )
                                    }
                                    ClientType::Tendermint => event(Identified::<
                                        Union,
                                        Wasm<Cosmos>,
                                        _,
                                    >::new(
                                        chain_event.chain_id,
                                        voyager_message::event::IbcEvent {
                                            block_hash: chain_event.block_hash,
                                            height: chain_event.height,
                                            event: chain_event_to_lc_event::<Union, Wasm<Cosmos>>(
                                                chain_event.event,
                                            ),
                                        },
                                    )),
                                    _ => unimplemented!(),
                                },
                            )
                        }
                    })
                    .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
            })
            .flatten()
            .boxed();
        let cosmos_events = stream::iter(self.chains.cosmos.clone())
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
                                        ClientType::Wasm(WasmClientType::Cometbls) => {
                                            event(Identified::<Wasm<Cosmos>, Union, _>::new(
                                                chain_event.chain_id,
                                                voyager_message::event::IbcEvent {
                                                    block_hash: chain_event.block_hash,
                                                    height: chain_event.height,
                                                    event: chain_event_to_lc_event::<
                                                        Wasm<Cosmos>,
                                                        Union,
                                                    >(
                                                        chain_event.event
                                                    ),
                                                },
                                            ))
                                        }
                                        _ => unimplemented!(),
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

                            event(Identified::<Evm<Minimal>, Wasm<Union>, _>::new(
                                chain_event.chain_id,
                                voyager_message::event::IbcEvent {
                                    block_hash: chain_event.block_hash,
                                    height: chain_event.height,
                                    event: chain_event_to_lc_event::<Evm<Minimal>, Wasm<Union>>(
                                        chain_event.event,
                                    ),
                                },
                            ))
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

                            event(Identified::<Evm<Mainnet>, Wasm<Union>, _>::new(
                                chain_event.chain_id,
                                voyager_message::event::IbcEvent {
                                    block_hash: chain_event.block_hash,
                                    height: chain_event.height,
                                    event: chain_event_to_lc_event::<Evm<Mainnet>, Wasm<Union>>(
                                        chain_event.event,
                                    ),
                                },
                            ))
                        })
                        .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
                })
                .flatten()
                .boxed(),
            union_events,
            cosmos_events,
            self.msg_server
                .clone()
                .events(())
                .map_err(|x| Box::new(x) as Box<dyn Debug + Send + Sync>)
                .boxed(),
        ]));

        let mut join_set = JoinSet::new();

        let mut q = self.relay_queue.clone();
        join_set.spawn({
            async move {
                tracing::debug!("checking for new messages");

                while let Some(msg) = events.next().await {
                    let msg = msg.map_err(|x| format!("{x:?}"))?;

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

            let worker = self.worker(i);

            join_set.spawn(worker.run(self.relay_queue.clone(), None));
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

impl<T: QueueMsgTypes> Worker<T> {
    pub fn run<Q>(
        self,
        mut q: Q,
        data_out_stream: Option<mpsc::Sender<T::Data>>,
    ) -> impl Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static
    where
        Q: Queue<T>,
    {
        async move {
            loop {
                // yield back to the runtime
                tokio::time::sleep(Duration::from_millis(10)).await;

                let worker = self.clone();
                let mut data_out_stream = data_out_stream.clone();

                q.process(move |msg| {
                    async move {
                        let new_msgs = msg.handle(&*worker.store, 0).await;

                        match new_msgs {
                            Ok(ok) => {
                                let ok = if let Some(ref mut data_out_stream) = data_out_stream {
                                    match ok {
                                        Some(QueueMsg::Data(data)) => {
                                            tracing::warn!(%data, "received data in worker");
                                            data_out_stream.send(data).unwrap();
                                            None
                                        }
                                        _ => ok,
                                    }
                                } else {
                                    ok
                                };

                                Ok(ok)
                            }
                            // REVIEW: Check if this error is recoverable or not - i.e. if this is an IO error,
                            // the msg can likely be retried
                            Err(err) => {
                                // ProcessFlow::Fail(err.to_string())
                                // HACK: panic is OK here since this is spawned in a task, and will be caught by the runtime worker
                                panic!("{err}");
                            }
                        }
                    }
                })
                .await?;
            }
        }
    }
}

// pub enum AnyLcError_ {}

// impl AnyLightClient for AnyLcError_ {}

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
// fn mk_aggregate_update<Hc: ChainExt, Tr: ChainExt>(
//     chain_id: ChainIdOf<HostChain>,
//     client_id: Hc::ClientId,
//     counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
//     event_height: HeightOf<HostChain>,
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

async fn client_type_from_ibc_event<Hc: ChainExt + CosmosSdkChain>(
    c: &Hc,
    ibc_event: &IbcEvent<ClientId, String, String>,
) -> ClientType {
    let client_type_from_client_id = |client_id: ClientId| async {
        if client_id.rsplit_once('-').unwrap().0 == "07-tendermint" {
            ClientType::Tendermint
        } else {
            ClientType::Wasm(
                c.checksum_of_client_id(client_id)
                    .then(|checksum| c.client_type_of_checksum(checksum))
                    .await,
            )
        }
    };

    match ibc_event {
        IbcEvent::CreateClient(CreateClient { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::UpdateClient(UpdateClient { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::ClientMisbehaviour(ClientMisbehaviour { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::SubmitEvidence(SubmitEvidence { .. }) => {
            // TODO: Not sure how to handle this one, since it only contains the hash
            // union
            //     .code_id_of_client_id(client_id)
            //     .then(|checksum| union.client_type_of_code_id(checksum))
            //     .await
            panic!()
        }
        IbcEvent::ConnectionOpenInit(ConnectionOpenInit { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::ConnectionOpenTry(ConnectionOpenTry { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::ConnectionOpenAck(ConnectionOpenAck { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm { client_id, .. }) => {
            client_type_from_client_id(client_id.clone()).await
        }
        IbcEvent::ChannelOpenInit(ChannelOpenInit { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::ChannelOpenTry(ChannelOpenTry { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::ChannelOpenAck(ChannelOpenAck { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::WriteAcknowledgement(WriteAcknowledgement { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::RecvPacket(RecvPacket { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::SendPacket(SendPacket { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::AcknowledgePacket(AcknowledgePacket { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
        IbcEvent::TimeoutPacket(TimeoutPacket { connection_id, .. }) => {
            c.client_id_of_connection(connection_id.clone())
                .then(|client_id| client_type_from_client_id(client_id.clone()))
                .await
        }
    }
}
