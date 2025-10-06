#![feature(trait_alias, slice_partition_dedup)]

use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    panic::AssertUnwindSafe,
    sync::{Arc, OnceLock},
    time::Duration,
};

use anyhow::{anyhow, Context as _};
use futures::{
    future::{self, BoxFuture},
    stream::{self, FuturesUnordered},
    Future, FutureExt, StreamExt, TryFutureExt, TryStreamExt,
};
use itertools::Itertools;
use jsonrpsee::core::middleware::{RpcServiceBuilder, RpcServiceT};
use opentelemetry::{metrics::Counter, KeyValue};
use serde::Serialize;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use tracing::{
    debug, debug_span, error, info, info_span, instrument, trace, trace_span, warn, Instrument,
};
use unionlabs::ErrorReporter;
use voyager_message::{
    call::{
        Call, FetchUpdateHeaders, Index, IndexRange, SubmitTx, WaitForClientUpdate, WaitForHeight,
        WaitForHeightRelative, WaitForTimestamp, WaitForTrustedHeight, WaitForTrustedTimestamp,
    },
    callback::{AggregateSubmitTxFromOrderedHeaders, Callback},
    data::{Data, IbcDatagram, OrderedHeaders},
    PluginMessage, VoyagerMessage,
};
use voyager_plugin_protocol::{
    coordinator_server, worker_child_process, WithId, WorkerClient, INVALID_CONFIG_EXIT_CODE,
};
use voyager_primitives::{ClientInfo, IbcSpec, QueryHeight};
use voyager_rpc::{
    error_object_to_queue_error, json_rpc_error_to_queue_error, missing_state,
    types::{
        ClientBootstrapModuleInfo, ClientModuleInfo, FinalityModuleInfo, PluginInfo,
        ProofModuleInfo, StateModuleInfo,
    },
    ClientModuleClient, PluginClient, VoyagerRpcServer,
};
use voyager_vm::{
    defer,
    in_memory::InMemoryQueue,
    noop, now,
    pass::{Pass, PassResult},
    seq, BoxDynError, HandlerFactory, ItemId, Op, Queue, QueueError,
};

use crate::{
    context::{Context, ModuleConfig, ModulesConfig, PluginConfig},
    equivalent_chain_ids::EquivalentChainIds,
    filter::InterestFilters,
    ibc_spec_handlers::IbcSpecHandlers,
    server::Server,
};

pub mod cache;
pub mod context;
pub mod equivalent_chain_ids;
pub mod filter;
pub mod ibc_spec_handlers;
pub mod server;

pub struct Engine<Q: Queue<VoyagerMessage>> {
    context: Arc<OnceLock<Context>>,
    interest_filters: InterestFilters,
    cache: cache::Cache,
    queue: Q,
    cancellation_token: CancellationToken,
    // NOTE: non-zero
    num_workers: usize,
    rest_laddr: SocketAddr,
    rpc_laddr: SocketAddr,
    optimizer_delay_milliseconds: u64,
    // TODO: Make this generic
    rpc_middleware: LoggerMiddlewareLayer,
}

impl Engine<InMemoryQueue<VoyagerMessage>> {
    #[allow(clippy::new_without_default)]
    pub fn builder() -> EngineBuilder {
        EngineBuilder {
            ibc_spec_handlers: IbcSpecHandlers::new(),
            plugin_configs: Default::default(),
            module_configs: Default::default(),
            equivalent_chain_ids: Default::default(),
            ipc_client_request_timeout: Default::default(),
            cache_config: Default::default(),
            metrics_endpoint: Default::default(),
            num_workers: 1,
            rest_laddr: default_rest_laddr(),
            rpc_laddr: default_rpc_laddr(),
            optimizer_delay_milliseconds: default_optimizer_delay_milliseconds(),
            queue_config: (),
        }
    }
}

impl HandlerFactory<VoyagerMessage> for Server {
    type Handler = Handler;

    fn make_handler(&self, item_id: ItemId) -> Self::Handler {
        Handler {
            server: self.with_id(Some(item_id)),
        }
    }
}

impl<Q: Queue<VoyagerMessage>> Engine<Q> {
    pub fn shutdown(self) {
        self.cancellation_token.cancel();
    }

    pub fn server(&self) -> Server {
        Server::new(self.cache.clone(), self.context.clone())
    }

    #[allow(clippy::too_many_lines)]
    pub fn run(&self) -> impl Future<Output = ()> + use<'_, Q> {
        let queue_rx = api::run(self.rest_laddr.clone());

        let mut tasks = FuturesUnordered::<BoxFuture<Result<Result<(), BoxDynError>, _>>>::new();

        {
            tasks.push(Box::pin(
                AssertUnwindSafe(async {
                    let server = jsonrpsee::server::Server::builder()
                        .set_http_middleware(
                            tower::ServiceBuilder::new()
                                .layer(tower_http::cors::CorsLayer::permissive()),
                        )
                        .set_rpc_middleware(
                            RpcServiceBuilder::new().layer(self.rpc_middleware.clone()),
                        )
                        .build(&self.rpc_laddr)
                        .await?;

                    let addr = server.local_addr()?;

                    let handle = server.start(self.server().into_rpc());

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
                        info!(
                            "received new message: {}",
                            serde_json::to_value(&op).unwrap()
                        );

                        self.queue.enqueue(op, &self.interest_filters).await?;
                    }

                    Ok(())
                })
                .catch_unwind(),
            ));

            info!("spawning {} workers", self.num_workers);

            for id in 0..self.num_workers {
                debug!("spawning worker {id}");

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        voyager_vm::engine::Engine::new(
                            self.server(),
                            &self.queue,
                            &self.interest_filters,
                        )
                        .run()
                        .for_each(async |res| match res {
                            Ok(data) => {
                                debug!(
                                    data = %serde_json::to_value(&data).unwrap(),
                                    "received data outside of an aggregation",
                                );
                            }
                            Err(error) => {
                                error!(
                                    error = %ErrorReporter(&*error),
                                    "error processing message"
                                );
                            }
                        })
                        .map(Ok)
                        .instrument(trace_span!("engine task", %id)),
                    )
                    .catch_unwind(),
                ));
            }

            for (_filter, plugin_name) in &self.interest_filters.filters {
                info!(%plugin_name, "spawning optimizer");

                tasks.push(Box::pin(
                    AssertUnwindSafe(
                        async {
                            let plugin_name = plugin_name.clone();

                            let pass = PluginOptPass::new(
                                self.context
                                    .get()
                                    .unwrap()
                                    .plugin(&plugin_name)
                                    .expect("plugin exists")
                                    .client(),
                            );

                            loop {
                                trace!("optimizing");

                                let res = self
                                    .queue
                                    .optimize(&plugin_name, &self.interest_filters, &pass)
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
                        .instrument(info_span!("optimize", %plugin_name)), // .instrument(trace_span!("optimize_verbose", filter = %filter.to_string())),
                    )
                    .catch_unwind(),
                ));
            }
        }

        self.cancellation_token
            .run_until_cancelled(async move {
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
            })
            .map(|_| ())
    }
}

pub struct EngineBuilder<Q: Queue<VoyagerMessage> = InMemoryQueue<VoyagerMessage>> {
    queue_config: Q::Config,
    plugin_configs: Vec<PluginConfig>,
    module_configs: ModulesConfig,
    equivalent_chain_ids: EquivalentChainIds,
    ipc_client_request_timeout: Duration,
    cache_config: cache::Config,
    metrics_endpoint: Option<String>,
    ibc_spec_handlers: IbcSpecHandlers,
    num_workers: usize,
    rest_laddr: SocketAddr,
    rpc_laddr: SocketAddr,
    optimizer_delay_milliseconds: u64,
}

impl<Q: Queue<VoyagerMessage>> EngineBuilder<Q> {
    pub fn with_equivalent_chain_ids(self, equivalent_chain_ids: EquivalentChainIds) -> Self {
        Self {
            equivalent_chain_ids,
            ..self
        }
    }

    pub fn with_plugins(self, plugins: Vec<PluginConfig>) -> Self {
        Self {
            plugin_configs: plugins,
            ..self
        }
    }

    pub fn with_modules(self, modules: ModulesConfig) -> Self {
        Self {
            module_configs: modules,
            ..self
        }
    }

    pub fn with_ipc_client_request_timeout(self, ipc_client_request_timeout: Duration) -> Self {
        Self {
            ipc_client_request_timeout,
            ..self
        }
    }

    pub fn with_cache_config(self, cache_config: cache::Config) -> Self {
        Self {
            cache_config,
            ..self
        }
    }

    pub fn with_metrics_endpoint(self, metrics_endpoint: String) -> Self {
        Self {
            metrics_endpoint: Some(metrics_endpoint),
            ..self
        }
    }

    pub fn with_num_workers(self, num_workers: usize) -> Self {
        Self {
            num_workers,
            ..self
        }
    }

    pub fn with_rest_laddr(self, rest_laddr: SocketAddr) -> Self {
        Self { rest_laddr, ..self }
    }

    pub fn with_rpc_laddr(self, rpc_laddr: SocketAddr) -> Self {
        Self { rpc_laddr, ..self }
    }

    pub fn with_optimizer_delay_milliseconds(self, optimizer_delay_milliseconds: u64) -> Self {
        Self {
            optimizer_delay_milliseconds,
            ..self
        }
    }

    pub fn register_ibc_spec_handler<S: IbcSpec>(mut self) -> Self {
        self.ibc_spec_handlers.register::<S>();
        self
    }

    pub fn with_queue<NewQ: Queue<VoyagerMessage>>(
        self,
        queue_config: NewQ::Config,
    ) -> EngineBuilder<NewQ> {
        EngineBuilder {
            queue_config,
            plugin_configs: self.plugin_configs,
            module_configs: self.module_configs,
            equivalent_chain_ids: self.equivalent_chain_ids,
            ipc_client_request_timeout: self.ipc_client_request_timeout,
            cache_config: self.cache_config,
            metrics_endpoint: self.metrics_endpoint,
            ibc_spec_handlers: self.ibc_spec_handlers,
            num_workers: self.num_workers,
            rest_laddr: self.rest_laddr,
            rpc_laddr: self.rpc_laddr,
            optimizer_delay_milliseconds: self.optimizer_delay_milliseconds,
        }
    }
}

impl<Q: Queue<VoyagerMessage>> EngineBuilder<Q> {
    pub async fn build(self) -> anyhow::Result<Engine<Q>> {
        let cancellation_token = CancellationToken::new();

        let queue = Q::new(self.queue_config).await?;

        let mut context_inner = Context {
            state_modules: Default::default(),
            proof_modules: Default::default(),
            client_modules: Default::default(),
            client_bootstrap_modules: Default::default(),
            finality_modules: Default::default(),
            chain_consensus_types: Default::default(),
            client_consensus_types: Default::default(),
            plugins: Default::default(),
            equivalent_chain_ids: self.equivalent_chain_ids,
            ibc_spec_handlers: self.ibc_spec_handlers,
        };

        let logger_middleware_layer = LoggerMiddlewareLayer::new();

        let context = Arc::new(OnceLock::new());

        let mut interest_filters = HashMap::new();

        let cache = cache::Cache::new(self.cache_config);

        info!("spawning {} plugins", self.plugin_configs.len());

        stream::iter(self.plugin_configs.into_iter().enumerate())
            .filter(|(_, plugin_config)| {
                future::ready(if !plugin_config.enabled {
                    info!(
                        plugin_path = %plugin_config.path.to_string_lossy(),
                        "plugin is not enabled, skipping"
                    );
                    false
                } else {
                    true
                })
            })
            .zip(stream::repeat(Server::new(cache.clone(), context.clone())))
            .then(async |((idx, plugin_config), server)| {
                let plugin_info = info_span!("get_plugin_info", %idx)
                    .in_scope(|| get_plugin_info(&plugin_config))?;

                debug!("starting rpc server for plugin {}", plugin_info.name);

                tokio::spawn(
                    coordinator_server(&plugin_info.name, server, logger_middleware_layer.clone())
                        .await?,
                );

                debug!("started rpc server for plugin {}", plugin_info.name);

                Ok((idx, plugin_config, plugin_info))
            })
            .try_for_each_concurrent(
                None,
                |(
                    idx,
                    plugin_config,
                    PluginInfo {
                        name,
                        interest_filter,
                    },
                )| {
                    debug!("registering plugin {}", name);

                    tokio::spawn(worker_child_process(
                        name.clone(),
                        plugin_config.path,
                        cancellation_token.clone(),
                        [plugin_config.config.to_string()]
                            .into_iter()
                            .chain(self.metrics_endpoint.clone()),
                    ));

                    let rpc_client = WorkerClient::new(&name, self.ipc_client_request_timeout);

                    let prev = context_inner
                        .plugins
                        .insert(name.clone(), rpc_client.clone());

                    if prev.is_some() {
                        return future::ready(Err(anyhow!(
                            "multiple plugins configured with name `{name}`"
                        )));
                    }

                    info!("registered plugin {name}");

                    interest_filters.insert((idx, name), interest_filter);

                    future::ready(Ok(()))
                },
            )
            .await?;

        modules_startup(
            self.module_configs.state,
            logger_middleware_layer.clone(),
            cancellation_token.clone(),
            Server::new(cache.clone(), context.clone()),
            self.ipc_client_request_timeout,
            |info| info.id(),
            |StateModuleInfo {
                 chain_id,
                 ibc_spec_id,
             },
             rpc_client| {
                for equivalent_chain_id in context_inner
                    .equivalent_chain_ids
                    .equivalents(chain_id)
                    .chain([chain_id])
                {
                    let prev = context_inner.state_modules.insert(
                        (equivalent_chain_id.clone(), ibc_spec_id.clone()),
                        rpc_client.clone(),
                    );

                    if prev.is_some() {
                        return Err(anyhow!(
                            "multiple state modules configured for chain id \
                            `{equivalent_chain_id}` and IBC version `{ibc_spec_id}`",
                        ));
                    }
                }

                Ok(())
            },
            self.metrics_endpoint.clone(),
        )
        .await?;

        modules_startup(
            self.module_configs.proof,
            logger_middleware_layer.clone(),
            cancellation_token.clone(),
            Server::new(cache.clone(), context.clone()),
            self.ipc_client_request_timeout,
            |info| info.id(),
            |ProofModuleInfo {
                 chain_id,
                 ibc_spec_id,
             },
             rpc_client| {
                for equivalent_chain_id in context_inner
                    .equivalent_chain_ids
                    .equivalents(chain_id)
                    .chain([chain_id])
                {
                    let prev = context_inner.proof_modules.insert(
                        (equivalent_chain_id.clone(), ibc_spec_id.clone()),
                        rpc_client.clone(),
                    );

                    if prev.is_some() {
                        return Err(anyhow!(
                            "multiple proof modules configured for chain id \
                            `{equivalent_chain_id}` and IBC version `{ibc_spec_id}`",
                        ));
                    }
                }

                Ok(())
            },
            self.metrics_endpoint.clone(),
        )
        .await?;

        modules_startup(
            self.module_configs.consensus,
            logger_middleware_layer.clone(),
            cancellation_token.clone(),
            Server::new(cache.clone(), context.clone()),
            self.ipc_client_request_timeout,
            |info| info.id(),
            |FinalityModuleInfo {
                 chain_id,
                 consensus_type,
             },
             rpc_client| {
                for equivalent_chain_id in context_inner
                    .equivalent_chain_ids
                    .equivalents(chain_id)
                    .chain([chain_id])
                {
                    let prev = context_inner
                        .finality_modules
                        .insert(equivalent_chain_id.clone(), rpc_client.clone());

                    if prev.is_some() {
                        return Err(anyhow!(
                            "multiple consensus modules configured for chain id `{}`",
                            equivalent_chain_id
                        ));
                    }

                    let None = context_inner
                        .chain_consensus_types
                        .insert(equivalent_chain_id.clone(), consensus_type.clone())
                    else {
                        unreachable!()
                    };
                }

                Ok(())
            },
            self.metrics_endpoint.clone(),
        )
        .await?;

        modules_startup(
            self.module_configs.client,
            logger_middleware_layer.clone(),
            cancellation_token.clone(),
            Server::new(cache.clone(), context.clone()),
            self.ipc_client_request_timeout,
            |info| info.id(),
            |ClientModuleInfo {
                 client_type,
                 consensus_type,
                 ibc_interface,
                 ibc_spec_id,
             },
             rpc_client| {
                if !context_inner
                    .ibc_spec_handlers
                    .handlers
                    .contains_key(ibc_spec_id)
                {
                    return Err(anyhow!(
                        "IBC version `{ibc_spec_id}` is not supported in this build of voyager"
                    ));
                }

                let prev = context_inner.client_modules.insert(
                    (
                        client_type.clone(),
                        ibc_interface.clone(),
                        ibc_spec_id.clone(),
                    ),
                    rpc_client.clone(),
                );

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple client modules configured for client \
                        type `{client_type}`, IBC interface `{ibc_interface}`, \
                        and IBC version `{ibc_spec_id}`",
                    ));
                }

                if let Some(previous_consensus_type) = context_inner
                    .client_consensus_types
                    .insert(client_type.clone(), consensus_type.clone())
                {
                    if previous_consensus_type != consensus_type {
                        return Err(anyhow!(
                            "inconsistency in client consensus types: \
                            client type `{client_type}` is registered \
                            as tracking both `{previous_consensus_type}` \
                            and `{consensus_type}`"
                        ));
                    }
                }

                Ok(())
            },
            self.metrics_endpoint.clone(),
        )
        .await?;

        modules_startup(
            self.module_configs.client_bootstrap,
            logger_middleware_layer.clone(),
            cancellation_token.clone(),
            Server::new(cache.clone(), context.clone()),
            self.ipc_client_request_timeout,
            |info| info.id(),
            |ClientBootstrapModuleInfo {
                 client_type,
                 chain_id,
             },
             rpc_client| {
                for equivalent_chain_id in context_inner
                    .equivalent_chain_ids
                    .equivalents(chain_id)
                    .chain([chain_id])
                {
                    let prev = context_inner.client_bootstrap_modules.insert(
                        (equivalent_chain_id.clone(), client_type.clone()),
                        rpc_client.clone(),
                    );

                    if prev.is_some() {
                        return Err(anyhow!(
                            "multiple client bootstrap modules configured for client \
                            type `{client_type}` and chain id `{equivalent_chain_id}`",
                        ));
                    }

                    // TODO: Check consistency with client_consensus_types and chain_id?

                    // if let Some(previous_consensus_type) = modules
                    //     .client_consensus_types
                    //     .insert(client_type.clone(), consensus_type.clone())
                    // {
                    //     if previous_consensus_type != consensus_type {
                    //         return Err(anyhow!(
                    //             "inconsistency in client consensus types: \
                    //             client type `{client_type}` is registered \
                    //             as tracking both `{previous_consensus_type}` \
                    //             and `{consensus_type}`"
                    //         ));
                    //     }
                    // }
                }

                Ok(())
            },
            self.metrics_endpoint.clone(),
        )
        .await?;

        info!("checking for plugin health...");

        let futures = context_inner
            .plugins
            .iter()
            .map(|(name, client)| async move {
                match client
                    .inner()
                    .wait_until_connected(Duration::from_secs(10))
                    .instrument(debug_span!("health check", %name))
                    .await
                {
                    Ok(()) => {
                        info!("plugin {name} connected")
                    }
                    Err(_) => {
                        warn!("plugin {name} failed to connect after 10 seconds")
                    }
                }
            })
            .collect::<FuturesUnordered<_>>();

        match cancellation_token
            .run_until_cancelled(futures.collect::<Vec<_>>())
            .await
        {
            Some(_) => {}
            None => return Err(anyhow!("startup error")),
        }

        context
            .set(context_inner)
            .ok()
            .expect("context is only set once");

        info!("started");

        let interest_filters = InterestFilters::new(
            interest_filters
                .into_iter()
                .sorted_unstable_by(|((a, _), _), ((b, _), _)| a.cmp(b))
                .map(|((_, k), v)| (k, v))
                .map(|(name, interest_filter)| PluginInfo {
                    name,
                    interest_filter,
                })
                .collect(),
        )?;

        Ok(Engine {
            interest_filters,
            cancellation_token,
            context,
            cache,
            queue,
            num_workers: self.num_workers,
            rest_laddr: self.rest_laddr,
            rpc_laddr: self.rpc_laddr,
            optimizer_delay_milliseconds: self.optimizer_delay_milliseconds,
            rpc_middleware: logger_middleware_layer,
        })
    }
}

pub struct Handler {
    server: Server,
}

impl voyager_vm::Handler<VoyagerMessage> for Handler {
    #[instrument(skip_all)]
    async fn call(&self, call: Call) -> Result<Op<VoyagerMessage>, QueueError> {
        match call {
            Call::Index(Index {
                start_height,
                chain_id,
            }) => {
                let message = format!(
                    "fetch blocks request received for chain `{chain_id}` at height \
                    {start_height} but it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }
            Call::IndexRange(IndexRange { chain_id, range }) => {
                let message = format!(
                    "fetch block range request received for chain `{chain_id}` for range \
                    {}..={} but it was not picked up by a plugin",
                    range.from_height(),
                    range.to_height()
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            Call::FetchUpdateHeaders(FetchUpdateHeaders {
                client_type,
                chain_id,
                counterparty_chain_id,
                client_id,
                update_from,
                update_to,
            }) => {
                let message = format!(
                    "client update request received for a {client_type} client \
                    (id {client_id}) on {counterparty_chain_id} tracking {chain_id} from \
                    height {update_from} to {update_to} but it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            Call::SubmitTx(SubmitTx { chain_id, .. }) => {
                let message = format!(
                    "transaction submission request received for chain {chain_id} but \
                    it was not picked up by a plugin"
                );

                Err(QueueError::Unprocessable(message.into()))
            }

            Call::WaitForHeight(WaitForHeight {
                chain_id,
                height,
                finalized,
            }) => {
                let chain_height = self
                    .server
                    // .with_id(Some(self.id))
                    .query_latest_height(&chain_id, finalized)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if !chain_height.revision_matches(&height) {
                    return Err(QueueError::Fatal(
                        format!(
                            "revision number mismatch, \
                            chain_height: {chain_height}, height: {height}"
                        )
                        .into(),
                    ));
                }

                trace!("latest height is {chain_height}, waiting for {height}");

                if chain_height.height() >= height.height() {
                    Ok(noop())
                } else {
                    Ok(seq([
                        defer(now() + 1),
                        voyager_vm::call(WaitForHeight {
                            chain_id,
                            height,
                            finalized,
                        }),
                    ]))
                }
            }

            Call::WaitForTimestamp(WaitForTimestamp {
                chain_id,
                timestamp,
                finalized,
            }) => {
                let chain_timestamp = self
                    .server
                    // .with_id(Some(self.id))
                    .query_latest_timestamp(&chain_id, finalized)
                    .await
                    .map_err(error_object_to_queue_error)?;

                if chain_timestamp >= timestamp {
                    info!(%chain_id, %timestamp, %chain_timestamp, "timestamp reached");
                    Ok(noop())
                } else {
                    debug!(%chain_id, %timestamp, %chain_timestamp, "timestamp not yet reached");
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new
                        // method on chain
                        defer(now() + 1),
                        voyager_vm::call(WaitForTimestamp {
                            chain_id,
                            timestamp,
                            finalized,
                        }),
                    ]))
                }
            }

            Call::WaitForHeightRelative(WaitForHeightRelative {
                chain_id,
                height_diff,
                finalized,
            }) => {
                let chain_height = self
                    .server
                    // .with_id(Some(self.id))
                    .query_latest_height(&chain_id, finalized)
                    .await
                    .map_err(error_object_to_queue_error)?;

                Ok(seq([
                    defer(now() + 1),
                    voyager_vm::call(WaitForHeight {
                        chain_id,
                        height: chain_height.increment_by(height_diff),
                        finalized,
                    }),
                ]))
            }

            Call::WaitForTrustedHeight(WaitForTrustedHeight {
                chain_id,
                ibc_spec_id,
                client_id,
                height,
                finalized,
            }) => {
                let trusted_client_state_meta = self
                    .server
                    // .with_id(Some(self.id))
                    .client_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        if finalized {
                            QueryHeight::Finalized
                        } else {
                            QueryHeight::Latest
                        },
                        client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let continuation = seq([
                    // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                    // require a new method on chain
                    defer(now() + 1),
                    voyager_vm::call(WaitForTrustedHeight {
                        chain_id: chain_id.clone(),
                        ibc_spec_id,
                        client_id: client_id.clone(),
                        height,
                        finalized,
                    }),
                ]);

                match trusted_client_state_meta {
                    Some(trusted_client_state_meta) => {
                        if trusted_client_state_meta.counterparty_height.height() >= height.height()
                        {
                            debug!(
                                "client height reached ({} >= {})",
                                trusted_client_state_meta.counterparty_height, height
                            );

                            Ok(noop())
                        } else {
                            Ok(continuation)
                        }
                    }
                    None => {
                        debug!("client {client_id} not found on chain {chain_id}");
                        Ok(continuation)
                    }
                }
            }

            Call::WaitForTrustedTimestamp(WaitForTrustedTimestamp {
                chain_id,
                ibc_spec_id,
                client_id,
                timestamp,
                finalized,
            }) => {
                let trusted_client_state_meta = self
                    .server
                    // .with_id(Some(self.id))
                    .client_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        if finalized {
                            QueryHeight::Finalized
                        } else {
                            QueryHeight::Latest
                        },
                        client_id.clone(),
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                let continuation = seq([
                    // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would
                    // require a new method on chain
                    defer(now() + 1),
                    voyager_vm::call(WaitForTrustedTimestamp {
                        chain_id: chain_id.clone(),
                        ibc_spec_id: ibc_spec_id.clone(),
                        client_id: client_id.clone(),
                        timestamp,
                        finalized,
                    }),
                ]);

                match trusted_client_state_meta {
                    Some(trusted_client_state_meta) => {
                        let trusted_consensus_state_meta = self
                            .server
                            // .with_id(Some(self.id))
                            .consensus_state_meta(
                                &chain_id,
                                &ibc_spec_id,
                                if finalized {
                                    QueryHeight::Finalized
                                } else {
                                    QueryHeight::Latest
                                },
                                client_id.clone(),
                                trusted_client_state_meta.counterparty_height,
                            )
                            .await
                            .map_err(error_object_to_queue_error)?;

                        match trusted_consensus_state_meta {
                            Some(trusted_consensus_state_meta)
                                if trusted_consensus_state_meta.timestamp >= timestamp =>
                            {
                                debug!(
                                    "client timestamp reached ({} >= {})",
                                    trusted_client_state_meta.counterparty_height, timestamp
                                );

                                Ok(noop())
                            }
                            _ => Ok(continuation),
                        }
                    }
                    None => {
                        debug!("client {client_id} not found on chain {chain_id}");
                        Ok(continuation)
                    }
                }
            }

            Call::WaitForClientUpdate(WaitForClientUpdate {
                chain_id,
                ibc_spec_id,
                client_id,
                height,
                // finalized,
            }) => {
                let consensus_state_meta = self
                    .server
                    // .with_id(Some(self.id))
                    .consensus_state_meta(
                        &chain_id,
                        &ibc_spec_id,
                        QueryHeight::Latest,
                        client_id.clone(),
                        height,
                    )
                    .await
                    .map_err(error_object_to_queue_error)?;

                match consensus_state_meta {
                    Some(consensus_state_meta) => {
                        debug!(
                            consensus_state_meta.timestamp = %consensus_state_meta.timestamp,
                            "consensus state exists"
                        );
                        Ok(noop())
                    }
                    None => {
                        debug!("consensus state for client {client_id} not found at height {height} on chain {chain_id}");
                        Ok(seq([
                            defer(now() + 1),
                            voyager_vm::call(WaitForClientUpdate {
                                chain_id: chain_id.clone(),
                                ibc_spec_id,
                                client_id: client_id.clone(),
                                height,
                                // finalized,
                            }),
                        ]))
                    }
                }
            }

            Call::Plugin(PluginMessage { plugin, message }) => {
                Ok(PluginClient::<Value, Value>::call(
                    &self
                        .server
                        .context()
                        .map_err(error_object_to_queue_error)?
                        .plugin(&plugin)?
                        .with_id(self.server.id()),
                    message,
                )
                .await
                .map_err(json_rpc_error_to_queue_error)?)
            }
        }
    }

    #[instrument(skip_all)]
    async fn callback(
        &self,
        callback: Callback,
        data: VecDeque<Data>,
    ) -> Result<Op<VoyagerMessage>, QueueError> {
        match callback {
            Callback::AggregateSubmitTxFromOrderedHeaders(
                AggregateSubmitTxFromOrderedHeaders {
                    ibc_spec_id,
                    chain_id,
                    client_id,
                },
            ) => {
                let OrderedHeaders { headers } = data
                    .into_iter()
                    .exactly_one()
                    .map_err(|found| serde_json::to_string(&found.collect::<Vec<_>>()).unwrap())
                    .and_then(|d| {
                        d.try_into()
                            .map_err(|found| serde_json::to_string(&found).unwrap())
                    })
                    .map_err(|found| {
                        QueueError::Fatal(
                            format!(
                                "OrderedHeaders not present in data queue \
                                for AggregateSubmitTxFromOrderedHeaders, \
                                found {found}",
                            )
                            .into(),
                        )
                    })?;

                let ClientInfo {
                    client_type,
                    ibc_interface,
                    ..
                } = self
                    .server
                    // .with_id(Some(self.id))
                    .client_info(&chain_id, &ibc_spec_id, client_id.clone())
                    .await
                    .map_err(error_object_to_queue_error)?
                    .ok_or_else(missing_state("client not found", None))
                    .map_err(error_object_to_queue_error)?;

                let client_module = self
                    .server
                    .context()
                    .map_err(error_object_to_queue_error)?
                    .client_module(&client_type, &ibc_interface, &ibc_spec_id)?
                    .with_id(self.server.id());

                let ibc_spec_handler = self
                    .server
                    .context()
                    .map_err(error_object_to_queue_error)?
                    .ibc_spec_handlers
                    .get(&ibc_spec_id)
                    .map_err(error_object_to_queue_error)?;

                // OrderedClientUpdates

                Ok(voyager_vm::call(SubmitTx {
                    chain_id,
                    // REVIEW: Use FuturesOrdered here?
                    datagrams: stream::iter(headers.into_iter())
                        .then(|(_, header)| {
                            client_module
                                .encode_header(header)
                                .map_err(json_rpc_error_to_queue_error)
                                .and_then(|encoded_header| {
                                    futures::future::ready(
                                        (ibc_spec_handler.msg_update_client)(
                                            client_id.clone(),
                                            encoded_header,
                                        )
                                        .map_err(|e| {
                                            QueueError::Fatal(<BoxDynError>::from(format!("{e:#}")))
                                        })
                                        .map(|datagram| {
                                            IbcDatagram {
                                                ibc_spec_id: ibc_spec_id.clone(),
                                                datagram,
                                            }
                                        }),
                                    )
                                })
                        })
                        .try_collect::<Vec<_>>()
                        .await?,
                }))
            }
            Callback::Plugin(PluginMessage { plugin, message }) => {
                Ok(PluginClient::<Value, Value>::callback(
                    &self
                        .server
                        .context()
                        .map_err(error_object_to_queue_error)?
                        .plugin(&plugin)?
                        .with_id(self.server.id()),
                    message,
                    data,
                )
                .await
                .map_err(json_rpc_error_to_queue_error)?)
            }
        }
    }
}

pub fn get_plugin_info(plugin_config: &PluginConfig) -> anyhow::Result<PluginInfo> {
    debug!(
        "querying plugin info from plugin at {}",
        &plugin_config.path.to_string_lossy(),
    );

    let mut cmd = std::process::Command::new(&plugin_config.path);
    cmd.arg("info");
    cmd.arg(plugin_config.config.to_string());

    let output = cmd
        .output()
        .with_context(|| format!("spawning plugin at {}", plugin_config.path.display()))?;

    if !output.status.success() {
        match output.status.code() {
            Some(code) if code == INVALID_CONFIG_EXIT_CODE as i32 => {
                return Err(anyhow!(
                    "invalid config for plugin at path {}: stdout:\n{}\nstderr:\n{}",
                    &plugin_config.path.to_string_lossy(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ));
            }
            Some(_) | None => {
                return Err(anyhow!(
                    "unable to query info for plugin at path {}: stdout:\n{}\nstderr:\n{}",
                    &plugin_config.path.to_string_lossy(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ));
            }
        }
    }

    trace!("plugin stdout: {}", String::from_utf8_lossy(&output.stdout));

    Ok(serde_json::from_slice(&output.stdout).unwrap())
}

#[allow(clippy::too_many_arguments)] // coward
async fn modules_startup<Info: Serialize + Clone + Unpin + Send + 'static>(
    configs: Vec<ModuleConfig<Info>>,
    logger_middleware_layer: LoggerMiddlewareLayer,
    cancellation_token: CancellationToken,
    server: Server,
    ipc_client_request_timeout: Duration,
    id_f: fn(&Info) -> String,
    mut push_f: impl FnMut(&Info, WorkerClient) -> anyhow::Result<()>,
    metrics_endpoint: Option<String>,
) -> anyhow::Result<()> {
    stream::iter(configs)
        .filter(|module_config| {
            future::ready(if !module_config.enabled {
                info!(
                    module_path = %module_config.path.to_string_lossy(),
                    "module is not enabled, skipping"
                );
                false
            } else {
                true
            })
        })
        .zip(stream::repeat((
            server.clone(),
            logger_middleware_layer.clone(),
        )))
        .map::<anyhow::Result<_>, _>(anyhow::Result::Ok)
        .try_filter_map(
            |(module_config, (server, logger_middleware_layer))| async move {
                if !module_config.enabled {
                    info!(
                        module_path = %module_config.path.to_string_lossy(),
                        "module is not enabled, skipping"
                    );
                    anyhow::Result::Ok(None)
                } else {
                    debug!(
                        "starting rpc server for module {}",
                        id_f(&module_config.info)
                    );
                    tokio::spawn(
                        coordinator_server(
                            &id_f(&module_config.info),
                            server,
                            logger_middleware_layer,
                        )
                        .await?,
                    );

                    anyhow::Result::Ok(Some(module_config))
                }
            },
        )
        .try_collect::<FuturesUnordered<_>>()
        .await?
        .into_iter()
        .try_for_each(|module_config| {
            let id = id_f(&module_config.info);

            debug!("registering module {}", id);

            tokio::spawn(worker_child_process(
                id.clone(),
                module_config.path,
                cancellation_token.clone(),
                [
                    module_config.config.to_string(),
                    serde_json::to_string(&module_config.info).unwrap(),
                ]
                .into_iter()
                .chain(metrics_endpoint.clone()),
            ));

            let rpc_client = WorkerClient::new(&id, ipc_client_request_timeout);

            push_f(&module_config.info, rpc_client)?;

            info!("registered module {id}");

            Ok(())
        })
}

pub mod api {
    use std::net::SocketAddr;

    use axum::{
        extract::State,
        http::StatusCode,
        routing::{get, post},
        Json,
    };
    use futures::{
        channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
        SinkExt,
    };
    use voyager_message::VoyagerMessage;
    use voyager_vm::Op;

    pub fn run(laddr: SocketAddr) -> UnboundedReceiver<Op<VoyagerMessage>> {
        let (queue_tx, queue_rx) = unbounded::<Op<VoyagerMessage>>();

        tokio::spawn(async move {
            let app = axum::Router::new()
                .route("/enqueue", post(enqueue))
                .route("/health", get(async || StatusCode::OK))
                .with_state(queue_tx.clone());

            let listener = tokio::net::TcpListener::bind(laddr).await.unwrap();

            axum::serve(listener, app)
        });

        queue_rx
    }

    // #[axum::debug_handler]
    async fn enqueue(
        State(mut sender): State<UnboundedSender<Op<VoyagerMessage>>>,
        Json(op): Json<Op<VoyagerMessage>>,
    ) -> StatusCode {
        sender.send(op).await.expect("receiver should not close");

        StatusCode::OK
    }
}

pub struct PluginOptPass<T> {
    client: T,
}

impl<T> PluginOptPass<T> {
    pub fn new(client: T) -> Self {
        Self { client }
    }
}

impl<T: PluginClient<Value, Value> + Send + Sync> Pass<VoyagerMessage> for PluginOptPass<&'_ T> {
    type Error = jsonrpsee::core::client::Error;

    fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> impl Future<Output = Result<PassResult<VoyagerMessage>, Self::Error>> + Send {
        self.client.run_pass(msgs)
    }
}

#[must_use]
#[inline]
pub const fn default_rest_laddr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7177)
}

#[must_use]
#[inline]
pub const fn default_rpc_laddr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 7178)
}

#[must_use]
pub fn default_metrics_endpoint() -> String {
    "http://localhost:4318".to_owned()
}

#[must_use]
#[inline]
pub const fn default_optimizer_delay_milliseconds() -> u64 {
    100
}

#[must_use]
#[inline]
pub const fn default_ipc_client_request_timeout() -> Duration {
    Duration::new(60, 0)
}

#[derive(Clone)]
pub struct LoggerMiddlewareLayer {
    request_counter: Counter<u64>,
}

impl LoggerMiddlewareLayer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            request_counter: opentelemetry::global::meter("voyager")
                .u64_counter("rpc.requests.count")
                .build(),
        }
    }
}

impl<S> tower::Layer<S> for LoggerMiddlewareLayer {
    type Service = LoggerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggerMiddleware {
            request_counter: self.request_counter.clone(),
            service: inner,
        }
    }
}

#[derive(Clone)]
pub struct LoggerMiddleware<S> {
    request_counter: Counter<u64>,
    service: S,
}

impl<S: RpcServiceT> RpcServiceT for LoggerMiddleware<S> {
    type MethodResponse = S::MethodResponse;
    type NotificationResponse = S::NotificationResponse;
    type BatchResponse = S::BatchResponse;

    fn call<'a>(
        &self,
        request: jsonrpsee::types::Request<'a>,
    ) -> impl Future<Output = Self::MethodResponse> + Send + 'a {
        self.request_counter.add(
            1,
            &[KeyValue::new("method", request.method.clone().into_owned())],
        );
        self.service.call(request)
    }

    fn batch<'a>(
        &self,
        requests: jsonrpsee::core::middleware::Batch<'a>,
    ) -> impl Future<Output = Self::BatchResponse> + Send + 'a {
        self.service.batch(requests)
    }

    fn notification<'a>(
        &self,
        n: jsonrpsee::core::middleware::Notification<'a>,
    ) -> impl Future<Output = Self::NotificationResponse> + Send + 'a {
        self.service.notification(n)
    }
}
