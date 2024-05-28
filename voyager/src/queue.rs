#![allow(clippy::type_complexity)]

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use chain_utils::{AnyChain, AnyChainTryFromConfigError, Chains};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{channel::mpsc::UnboundedSender, Future, SinkExt, StreamExt, TryStreamExt};
use queue_msg::{Engine, InMemoryQueue, Queue, QueueMessageTypes, QueueMsg};
use relay_message::RelayMessageTypes;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::task::JoinSet;
use unionlabs::traits::{Chain, ClientState, FromStrExact};
use voyager_message::VoyagerMessageTypes;

use crate::config::{ChainConfig, Config};

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Voyager {
    pub chains: Arc<Chains>,
    num_workers: u16,
    // NOTE: pub temporarily
    pub queue: AnyQueue<VoyagerMessageTypes>,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum AnyQueueConfig {
    InMemory,
    PgQueue(PgQueueConfig),
}

#[derive(DebugNoBound, CloneNoBound)]
pub enum AnyQueue<T: QueueMessageTypes> {
    InMemory(InMemoryQueue<T>),
    PgQueue(PgQueue<T>),
}

#[derive(DebugNoBound, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(std::convert::Infallible),
    PgQueue(sqlx::Error),
}

impl<T: QueueMessageTypes> Queue<T> for AnyQueue<T> {
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

            tracing::trace!("queued");

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

            tracing::trace!("processed");

            res
        }
    }
}

#[derive(DebugNoBound, CloneNoBound)]
pub struct PgQueue<T: QueueMessageTypes>(pg_queue::Queue<QueueMsg<T>>, sqlx::PgPool);

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

impl<T: QueueMessageTypes> Queue<T> for PgQueue<T> {
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

    pub fn worker(&self) -> Engine<RelayMessageTypes> {
        Engine::new(self.chains.clone())
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
        async fn msg<T: QueueMessageTypes>(
            State(mut sender): State<UnboundedSender<QueueMsg<T>>>,
            Json(msg): Json<QueueMsg<T>>,
        ) -> StatusCode {
            tracing::info!(?msg, "received msg");
            sender.send(msg).await.expect("receiver should not close");

            StatusCode::OK
        }

        // #[axum::debug_handler]
        async fn msgs<T: QueueMessageTypes>(
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

            let engine = Engine::new(self.chains.clone());
            let mut q = self.queue.clone();

            join_set.spawn(Box::pin(async move {
                engine
                    .run(&mut q)
                    .try_for_each(|data| async move {
                        tracing::info!(data = %serde_json::to_string(&data).unwrap(), "received data outside of an aggregation");

                        Ok(())
                    })
                    .await
            }));
        }

        let errs = vec![];

        // TODO: figure out
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    tracing::error!(%err, "error processing message");
                    panic!();
                }
                Err(err) => {
                    tracing::error!(%err, "error processing message");
                    panic!();
                }
            }
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
    let mut ethereum_minimal = HashMap::new();
    let mut ethereum_mainnet = HashMap::new();
    let mut scroll = HashMap::new();
    let mut arbitrum = HashMap::new();

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
            AnyChain::EthereumMainnet(c) => {
                insert_into_chain_map(&mut ethereum_mainnet, c);
            }
            AnyChain::EthereumMinimal(c) => {
                insert_into_chain_map(&mut ethereum_minimal, c);
            }
            AnyChain::Scroll(c) => {
                insert_into_chain_map(&mut scroll, c);
            }
            AnyChain::Arbitrum(c) => {
                insert_into_chain_map(&mut arbitrum, c);
            }
        }
    }

    Ok(Chains {
        scroll,
        ethereum_minimal,
        ethereum_mainnet,
        union,
        cosmos,
        arbitrum,
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
