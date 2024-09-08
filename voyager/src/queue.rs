#![allow(clippy::type_complexity)]

use std::{
    error::Error,
    fmt::{Debug, Display},
    net::SocketAddr,
    panic::AssertUnwindSafe,
};

use axum::{
    extract::State,
    routing::{get, post},
    Json,
};
use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{
    channel::mpsc::UnboundedSender, future::BoxFuture, stream::FuturesUnordered, Future, FutureExt,
    SinkExt, StreamExt, TryStreamExt,
};
use pg_queue::{PgQueue, PgQueueConfig};
use prometheus::TextEncoder;
use queue_msg::{
    optimize::{OptimizationResult, Pass, PurePass},
    Captures, Engine, InMemoryQueue, Op, Queue, QueueMessage,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, info_span, trace, trace_span};
use tracing_futures::Instrument;
use unionlabs::ErrorReporter;
use voyager_message::{
    context::Context, pass::JaqInterestFilter, plugin::OptimizationPassPluginClient,
    rpc::VoyagerRpcServer, VoyagerMessage,
};

use crate::config::Config;

type BoxDynError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct Voyager {
    pub context: Context,
    num_workers: u16,
    pub rest_laddr: SocketAddr,
    pub rpc_laddr: SocketAddr,
    // NOTE: pub temporarily
    pub queue: AnyQueue<VoyagerMessage>,
    // pub tx_batch: TxBatch,
    pub optimizer_delay_milliseconds: u64,
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
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + Captures<'a>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + Captures<'a>,
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
        tag: &'a str,
        optimizer: &'a O,
    ) -> Result<(), sqlx::Either<Self::Error, O::Error>> {
        match self {
            AnyQueue::InMemory(queue) => queue
                .optimize(tag, optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::InMemory)),
            AnyQueue::PgQueue(queue) => queue
                .optimize(tag, optimizer)
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
    #[error("error initializing queue")]
    QueueInit(#[source] AnyQueueError),
    #[error("error initializing plugins")]
    Plugin(#[source] BoxDynError),
}

impl Voyager {
    pub async fn new(config: Config) -> Result<Self, VoyagerInitError> {
        // let chains = chains_from_config(config.chain).await?;

        let queue = AnyQueue::new(config.voyager.queue.clone())
            .await
            .map_err(VoyagerInitError::QueueInit)?;

        Ok(Self {
            context: Context::new(config.plugins)
                .await
                .map_err(VoyagerInitError::Plugin)?,
            num_workers: config.voyager.num_workers,
            rest_laddr: config.voyager.rest_laddr,
            rpc_laddr: config.voyager.rpc_laddr,
            queue,
            // tx_batch: config.voyager.tx_batch,
            optimizer_delay_milliseconds: config.voyager.optimizer_delay_milliseconds,
        })
    }

    pub async fn run(self) -> Result<(), BoxDynError> {
        let interest_filter = JaqInterestFilter::new(
            self.context
                .interest_filters()
                .clone()
                .into_iter()
                .collect(),
        )?;

        {
            // set up msg server
            let (queue_tx, queue_rx) = futures::channel::mpsc::unbounded::<Op<VoyagerMessage>>();

            let app = axum::Router::new()
                .route("/msg", post(msg))
                .route("/msgs", post(msgs))
                .route("/health", get(|| async move { StatusCode::OK }))
                .route("/metrics", get(metrics))
                // .route(
                //     "/signer/balances",
                //     get({
                //         let chains = self.chains.clone();
                //         || async move { Json(signer_balances(&chains).await) }
                //     }),
                // )
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

            async fn metrics() -> Result<String, StatusCode> {
                TextEncoder::new()
                    .encode_to_string(&prometheus::gather())
                    .map_err(|err| {
                        error!(?err, "could not gather metrics");
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
            }

            tokio::spawn(axum::Server::bind(&self.rest_laddr).serve(app.into_make_service()));

            let mut tasks =
                FuturesUnordered::<BoxFuture<Result<Result<(), BoxDynError>, _>>>::new();

            tasks.push(Box::pin(
                AssertUnwindSafe(async {
                    let server = jsonrpsee::server::Server::builder()
                        .build(self.rpc_laddr)
                        .await?;
                    let mut module = jsonrpsee::RpcModule::new(&self.context);

                    module
                        .merge(self.context.rpc_server.clone().into_rpc())
                        .unwrap();

                    let addr = server.local_addr()?;
                    let handle = server.start(module);

                    info!("rpc listening on {addr}");

                    handle
                        .stopped()
                        .instrument(trace_span!("voyager_rpc_server"))
                        .await;

                    Err("rpc server exited".into())
                })
                .catch_unwind(),
            ));

            tasks.push(Box::pin(
                AssertUnwindSafe(async {
                    debug!("checking for new messages");

                    pin_utils::pin_mut!(queue_rx);

                    while let Some(msg) = queue_rx.next().await {
                        info!(
                            json = %serde_json::to_string(&msg).unwrap(),
                            "received new message",
                        );

                        self.queue.enqueue(msg, &interest_filter).await?;
                    }

                    Ok(())
                })
                .catch_unwind(),
            ));

            info!("spawning {} workers", self.num_workers);

            for id in 0..self.num_workers {
                info!("spawning worker {id}");

                // let engine = ;

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        Engine::new(&self.context, &self.queue, &interest_filter)
                            .run()
                            .for_each(|res| async move {
                                match res {
                                    Ok(data) => {
                                        info!(
                                            data = %serde_json::to_string(&data).unwrap(),
                                            "received data outside of an aggregation"
                                        );
                                    }
                                    Err(error) => {
                                        error!(
                                            error = %ErrorReporter(&*error),
                                            "error processing message"
                                        )
                                    }
                                }
                            })
                            .map(Ok)
                            .instrument(trace_span!("engine task", %id)),
                    )
                    .catch_unwind(),
                ));
            }

            struct PluginOptPass<T> {
                client: T,
            }

            impl<T: OptimizationPassPluginClient<Value, Value, Value> + Send + Sync>
                Pass<VoyagerMessage> for PluginOptPass<&'_ T>
            {
                type Error = jsonrpsee::core::client::Error;

                fn run_pass(
                    &self,
                    msgs: Vec<Op<VoyagerMessage>>,
                ) -> impl Future<Output = Result<OptimizationResult<VoyagerMessage>, Self::Error>> + Send
                {
                    self.client.run_pass(msgs)
                }
            }

            for (plugin_name, filter) in self.context.interest_filters() {
                info!(%plugin_name, %filter, "spawning optimizer");

                // let client = self
                //     .context
                //     .plugin_client_raw::<Value, Value, Value>(&plugin_name)
                //     .unwrap();

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        async {
                            let plugin_name = plugin_name.clone();

                            let pass = PluginOptPass {
                                client: self
                                    .context
                                    .plugin_client_raw(&plugin_name)
                                    .unwrap()
                                    .client(),
                            };

                            loop {
                                trace!("optimizing");

                                let res = self
                                    .queue
                                    .optimize(&plugin_name.to_owned(), &pass)
                                    .await
                                    .map_err(|e| {
                                        e.map_either::<_, _, BoxDynError, BoxDynError>(
                                            |x| Box::new(x),
                                            |x| Box::new(x),
                                        )
                                        .into_inner()
                                    });

                                if let Err(error) = res {
                                    error!(
                                        error = %ErrorReporter(&*error),
                                        "optimization pass returned with error"
                                    );
                                }

                                tokio::time::sleep(std::time::Duration::from_millis(
                                    self.optimizer_delay_milliseconds,
                                ))
                                .await;
                            }
                        }
                        .instrument(info_span!("optimize", %plugin_name, %filter)),
                    )
                    .catch_unwind(),
                ));
            }

            while let Some(res) = tasks.next().await {
                match res {
                    Ok(Ok(())) => {
                        info!("task exited gracefully");
                    }
                    Ok(Err(err)) => {
                        error!(%err, "task returned with an error");
                        break;
                    }
                    Err(_err) => {
                        // can't do anything with dyn Any
                        error!("task panicked");
                        break;
                    }
                }
            }
        }

        self.context.shutdown().await;

        Err(RunError { errs: vec![] }.into())
    }

    pub async fn shutdown(self) {
        self.context.shutdown().await
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
