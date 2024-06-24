#![allow(clippy::type_complexity)]

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fmt::{Debug, Display},
    net::SocketAddr,
    num::NonZeroUsize,
    sync::Arc,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use chain_utils::{any_chain, AnyChain, AnyChainTryFromConfigError, Chains};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{channel::mpsc::UnboundedSender, Future, SinkExt, StreamExt, TryStreamExt};
use pg_queue::{PgQueue, PgQueueConfig};
use queue_msg::{
    optimize::{passes::NormalizeFinal, Pass, Pure, PurePass},
    Engine, InMemoryQueue, Op, Queue, QueueMessage,
};
use relay_message::RelayMessage;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;
use tracing::{debug, error, info, trace};
use unionlabs::traits::{Chain, FromStrExact};
use voyager_message::VoyagerMessage;

use crate::{
    config::{ChainConfig, Config},
    passes::tx_batch::TxBatch,
    signer_balances,
};

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Voyager {
    pub chains: Arc<Chains>,
    num_workers: u16,
    pub laddr: SocketAddr,
    // NOTE: pub temporarily
    pub queue: AnyQueue<VoyagerMessage>,
    pub max_batch_size: NonZeroUsize,
}

#[derive(DebugNoBound, CloneNoBound, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum AnyQueueConfig {
    InMemory,
    PgQueue(PgQueueConfig),
}

#[derive(DebugNoBound, CloneNoBound)]
pub enum AnyQueue<T: QueueMessage> {
    InMemory(InMemoryQueue<T>),
    PgQueue(PgQueue<T>),
}

#[derive(DebugNoBound, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(std::convert::Infallible),
    PgQueue(sqlx::Error),
}

impl<T: QueueMessage> Queue<T> for AnyQueue<T> {
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

    fn enqueue<'a, O: PurePass<T>>(
        &'a self,
        item: Op<T>,
        pre_enqueue_passes: &'a O,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        async move {
            match self {
                AnyQueue::InMemory(queue) => queue
                    .enqueue(item, pre_enqueue_passes)
                    .await
                    .map_err(AnyQueueError::InMemory)?,
                AnyQueue::PgQueue(queue) => queue
                    .enqueue(item, pre_enqueue_passes)
                    .await
                    .map_err(AnyQueueError::PgQueue)?,
            };

            trace!("queued");

            Ok(())
        }
    }

    fn process<'a, F, Fut, R, O>(
        &'a self,
        pre_reenqueue_passes: &'a O,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
        O: PurePass<T>,
    {
        async move {
            let res = match self {
                AnyQueue::InMemory(queue) => queue
                    .process(pre_reenqueue_passes, f)
                    .await
                    .map_err(AnyQueueError::InMemory),
                AnyQueue::PgQueue(queue) => queue
                    .process(pre_reenqueue_passes, f)
                    .await
                    .map_err(AnyQueueError::PgQueue),
            };

            trace!("processed");

            res
        }
    }

    async fn optimize<'a, O: Pass<T>>(
        &'a self,
        optimizer: &'a O,
    ) -> Result<(), sqlx::Either<Self::Error, O::Error>> {
        match self {
            AnyQueue::InMemory(queue) => queue
                .optimize(optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::InMemory)),
            AnyQueue::PgQueue(queue) => queue
                .optimize(optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::PgQueue)),
        }
    }
}

// #[derive(DebugNoBound, CloneNoBound)]
// pub struct PgQueue<T: QueueMessage>(pg_queue::PgQueue<Op<T>>, sqlx::PgPool);

// impl<T: QueueMessage> Queue<T> for PgQueue<T> {
//     type Error = sqlx::Error;

//     type Config = PgQueueConfig;

//     fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
//         async move { Ok(Self(pg_queue::Queue::new(), cfg.into_pg_pool().await?)) }
//     }

//     fn enqueue<'a, O: PurePass<T>>(
//         &'a self,
//         item: Op<T>,
//         pre_enqueue_passes: &'a O,
//     ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
//         async move {
//             let res = pre_enqueue_passes.run_pass_pure(vec![item]);

//             for (_, msg) in res.optimize_further {
//                 self.0
//                     .enqueue(&self.1, msg, vec![], EnqueueStatus::Optimize)
//                     .await?
//             }

//             for (_, msg) in res.ready {
//                 self.0
//                     .enqueue(&self.1, msg, vec![], EnqueueStatus::Ready)
//                     .await?
//             }

//             Ok(())
//         }
//     }

//     fn process<'a, F, Fut, R, O>(
//         &'a self,
//         pre_reenqueue_passes: &'a O,
//         f: F,
//     ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + '_
//     where
//         F: (FnOnce(Op<T>) -> Fut) + Send + 'static,
//         Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + 'static,
//         R: Send + Sync + 'static,
//         O: PurePass<T>,
//     {
//         async move {
//             self.0
//                 .process(&self.1, f, |msgs| {
//                     let res = pre_reenqueue_passes.run_pass_pure(msgs);

//                     (res.optimize_further, res.ready)
//                 })
//                 .await
//         }
//     }

//     fn optimize<'a, O: Pass<T>>(
//         &'a self,
//         optimizer: &'a O,
//     ) -> impl Future<Output = Result<(), Either<Self::Error, O::Error>>> + 'a {
//         self.0.optimize(&self.1, move |msgs| async move {
//             optimizer
//                 .run_pass(msgs)
//                 .map_ok(|x| (x.optimize_further, x.ready))
//                 .await
//         })
//     }
// }

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
            laddr: config.voyager.laddr,
            queue,
            max_batch_size: config.voyager.max_batch_size,
        })
    }

    pub fn worker(&self) -> Engine<RelayMessage> {
        Engine::new(self.chains.clone())
    }

    pub async fn run(self) -> Result<(), RunError> {
        // set up msg server
        let (queue_tx, queue_rx) = futures::channel::mpsc::unbounded::<Op<VoyagerMessage>>();

        let app = axum::Router::new()
            .route("/msg", post(msg))
            .route("/msgs", post(msgs))
            .route("/health", get(|| async move { StatusCode::OK }))
            .route(
                "/signer/balances",
                get({
                    let chains = self.chains.clone();
                    || async move { Json(signer_balances(&chains).await) }
                }),
            )
            .with_state(queue_tx.clone());

        // #[axum::debug_handler]
        async fn msg<T: QueueMessage>(
            State(mut sender): State<UnboundedSender<Op<T>>>,
            Json(msg): Json<Op<T>>,
        ) -> StatusCode {
            sender.send(msg).await.expect("receiver should not close");

            StatusCode::OK
        }

        // #[axum::debug_handler]
        async fn msgs<T: QueueMessage>(
            State(mut sender): State<UnboundedSender<Op<T>>>,
            Json(msgs): Json<Vec<Op<T>>>,
        ) -> StatusCode {
            for msg in msgs {
                sender.send(msg).await.expect("receiver should not close");
            }

            StatusCode::OK
        }

        tokio::spawn(axum::Server::bind(&self.laddr).serve(app.into_make_service()));

        let mut join_set = JoinSet::<Result<(), BoxDynError>>::new();

        let q = self.queue.clone();
        join_set.spawn({
            async move {
                debug!("checking for new messages");

                pin_utils::pin_mut!(queue_rx);

                while let Some(msg) = queue_rx.next().await {
                    info!(
                        json = %serde_json::to_string(&msg).unwrap(),
                        "received new message",
                    );

                    q.enqueue(msg, &NormalizeFinal::default()).await?;
                }

                Ok(())
            }
        });

        for i in 0..self.num_workers {
            info!("spawning worker {i}");

            let engine = Engine::new(self.chains.clone());
            let q = self.queue.clone();

            join_set.spawn(Box::pin(async move {
                engine
                    .run(&q, &NormalizeFinal::default())
                    .try_for_each(|data| async move {
                        info!(data = %serde_json::to_string(&data).unwrap(), "received data outside of an aggregation");

                        Ok(())
                    })
                    .await
            }));
        }

        join_set.spawn(async move {
            let q = self.queue.clone();

            loop {
                debug!("optimizing");

                q.optimize(&Pure((
                    TxBatch {
                        size_limit: self.max_batch_size,
                    },
                    NormalizeFinal::default(),
                )))
                .await
                .map_err(|e| {
                    e.map_either::<_, _, BoxDynError, BoxDynError>(|x| Box::new(x), |x| Box::new(x))
                        .into_inner()
                })?;

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });

        let errs = vec![];

        // TODO: figure out
        while let Some(res) = join_set.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    error!(%err, "error processing message");
                    panic!();
                }
                Err(err) => {
                    error!(%err, "error processing message");
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
    let mut chains = HashMap::new();

    fn insert_into_chain_map<C: Chain + Into<AnyChain>>(
        map: &mut HashMap<String, AnyChain>,
        chain: C,
    ) {
        let chain_id = chain.chain_id();
        map.insert(chain_id.to_string(), chain.into());

        info!(
            %chain_id,
            chain_type = <C::ChainType as FromStrExact>::EXPECTING,
            "registered chain"
        );
    }

    for (chain_name, chain_config) in config {
        if !chain_config.enabled {
            info!(%chain_name, "chain not enabled, skipping");
            continue;
        }

        let chain = AnyChain::try_from_config(chain_config.ty).await?;

        any_chain!(|chain| {
            insert_into_chain_map(&mut chains, chain);
        });
    }

    Ok(Chains { chains })
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
