#![allow(clippy::type_complexity)]

use std::{fmt::Debug, net::SocketAddr, panic::AssertUnwindSafe};

use frame_support_procedural::{CloneNoBound, DebugNoBound};
use futures::{future::BoxFuture, stream::FuturesUnordered, Future, FutureExt, StreamExt};
use pg_queue::{PgQueue, PgQueueConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, info_span, trace, trace_span};
use tracing_futures::Instrument;
use unionlabs::ErrorReporter;
use voyager_message::{
    context::Context, filter::JaqInterestFilter, into_value, module::PluginInfo,
    pass::PluginOptPass, rpc::VoyagerRpcServer, VoyagerMessage,
};
use voyager_vm::{
    engine::Engine, in_memory::InMemoryQueue, pass::Pass, BoxDynError, Captures, Op, Queue,
};

use crate::{api, config::Config};

#[derive(Debug)]
pub struct Voyager {
    // TODO: Make private
    pub context: Context,
    num_workers: u16,
    rest_laddr: SocketAddr,
    rpc_laddr: SocketAddr,
    queue: QueueImpl,
    optimizer_delay_milliseconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum QueueConfig {
    InMemory,
    PgQueue(PgQueueConfig),
}

#[derive(DebugNoBound, CloneNoBound)]
pub enum QueueImpl {
    InMemory(InMemoryQueue<VoyagerMessage>),
    PgQueue(PgQueue<VoyagerMessage>),
}

#[derive(DebugNoBound, thiserror::Error)]
#[error(transparent)]
pub enum AnyQueueError {
    InMemory(std::convert::Infallible),
    PgQueue(sqlx::Error),
}

impl Queue<VoyagerMessage> for QueueImpl {
    type Error = AnyQueueError;
    type Config = QueueConfig;

    fn new(cfg: Self::Config) -> impl Future<Output = Result<Self, Self::Error>> {
        async move {
            Ok(match cfg {
                QueueConfig::InMemory => Self::InMemory(
                    InMemoryQueue::new(())
                        .await
                        .map_err(AnyQueueError::InMemory)?,
                ),
                QueueConfig::PgQueue(cfg) => {
                    Self::PgQueue(PgQueue::new(cfg).await.map_err(AnyQueueError::PgQueue)?)
                }
            })
        }
    }

    fn enqueue<'a>(
        &'a self,
        item: Op<VoyagerMessage>,
        filter: &'a JaqInterestFilter,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a {
        async move {
            match self {
                QueueImpl::InMemory(queue) => queue
                    .enqueue(item, filter)
                    .await
                    .map_err(AnyQueueError::InMemory)?,
                QueueImpl::PgQueue(queue) => queue
                    .enqueue(item, filter)
                    .await
                    .map_err(AnyQueueError::PgQueue)?,
            };

            trace!("queued");

            Ok(())
        }
    }

    fn process<'a, F, Fut, R>(
        &'a self,
        filter: &'a JaqInterestFilter,
        f: F,
    ) -> impl Future<Output = Result<Option<R>, Self::Error>> + Send + Captures<'a>
    where
        F: (FnOnce(Op<VoyagerMessage>) -> Fut) + Send + Captures<'a>,
        Fut: Future<Output = (R, Result<Vec<Op<VoyagerMessage>>, String>)> + Send + Captures<'a>,
        R: Send + Sync + 'static,
    {
        async move {
            let res = match self {
                QueueImpl::InMemory(queue) => queue
                    .process(filter, f)
                    .await
                    .map_err(AnyQueueError::InMemory),
                QueueImpl::PgQueue(queue) => queue
                    .process(filter, f)
                    .await
                    .map_err(AnyQueueError::PgQueue),
            };

            trace!("processed");

            res
        }
    }

    async fn optimize<'a, O: Pass<VoyagerMessage>>(
        &'a self,
        tag: &'a str,
        optimizer: &'a O,
    ) -> Result<(), sqlx::Either<Self::Error, O::Error>> {
        match self {
            QueueImpl::InMemory(queue) => queue
                .optimize(tag, optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::InMemory)),
            QueueImpl::PgQueue(queue) => queue
                .optimize(tag, optimizer)
                .await
                .map_err(|e| e.map_left(AnyQueueError::PgQueue)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VoyagerInitError {
    #[error("error initializing queue")]
    QueueInit(#[source] AnyQueueError),
    #[error("error initializing plugins")]
    Plugin(#[source] BoxDynError),
}

impl Voyager {
    pub async fn new(config: Config) -> Result<Self, VoyagerInitError> {
        let queue = QueueImpl::new(config.voyager.queue.clone())
            .await
            .map_err(VoyagerInitError::QueueInit)?;

        Ok(Self {
            context: Context::new(config.plugins, config.modules)
                .await
                .map_err(VoyagerInitError::Plugin)?,
            num_workers: config.voyager.num_workers,
            rest_laddr: config.voyager.rest_laddr,
            rpc_laddr: config.voyager.rpc_laddr,
            queue,
            optimizer_delay_milliseconds: config.voyager.optimizer_delay_milliseconds,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub async fn run(self) -> Result<(), BoxDynError> {
        let interest_filter = JaqInterestFilter::new(
            self.context
                .interest_filters()
                .clone()
                .into_iter()
                .map(|(name, interest_filter)| PluginInfo {
                    name,
                    interest_filter,
                })
                .collect(),
        )?;

        let queue_rx = api::run(&self.rest_laddr);

        {
            let mut tasks =
                FuturesUnordered::<BoxFuture<Result<Result<(), BoxDynError>, _>>>::new();

            tasks.push(Box::pin(
                AssertUnwindSafe(async {
                    let server = jsonrpsee::server::Server::builder()
                        .build(&self.rpc_laddr)
                        .await?;
                    let addr = server.local_addr()?;
                    let handle = server.start(self.context.rpc_server.clone().into_rpc());
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

                    while let Some(op) = queue_rx.next().await {
                        info!("received new message: {}", into_value(&op));

                        self.queue.enqueue(op, &interest_filter).await?;
                    }

                    Ok(())
                })
                .catch_unwind(),
            ));

            info!("spawning {} workers", self.num_workers);

            for id in 0..self.num_workers {
                info!("spawning worker {id}");

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        Engine::new(&self.context, &self.queue, &interest_filter)
                            .run()
                            .for_each(|res| async move {
                                match res {
                                    Ok(data) => {
                                        info!(
                                            "received data outside of an aggregation: {}",
                                            into_value(&data)
                                        );
                                    }
                                    Err(error) => {
                                        error!(
                                            error = %ErrorReporter(&*error),
                                            "error processing message"
                                        );
                                    }
                                }
                            })
                            .map(Ok)
                            .instrument(trace_span!("engine task", %id)),
                    )
                    .catch_unwind(),
                ));
            }

            for (plugin_name, filter) in self.context.interest_filters() {
                info!(%plugin_name, "spawning optimizer");

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        async {
                            let plugin_name = plugin_name.clone();

                            let pass = PluginOptPass::new(
                                self.context
                                    .plugin_client_raw(&plugin_name)
                                    .expect("plugin exists")
                                    .client(),
                            );

                            loop {
                                trace!("optimizing");

                                let res =
                                    self.queue.optimize(&plugin_name, &pass).await.map_err(|e| {
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
                        .instrument(info_span!("optimize", %plugin_name))
                        .instrument(trace_span!("optimize_verbose", %filter)),
                    )
                    .catch_unwind(),
                ));
            }

            while let Some(res) = tasks.next().await {
                match res {
                    Ok(Ok(())) => {
                        info!("task exited gracefully");
                    }
                    Ok(Err(error)) => {
                        error!(
                            error = %ErrorReporter(&*error),
                            "task returned with an error"
                        );
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

        Err("runtime error, exiting".to_owned().into())
    }

    pub async fn shutdown(self) {
        self.context.shutdown().await;
    }
}
