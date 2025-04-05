// #![warn(clippy::unwrap_used)]

use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

use anyhow::{anyhow, Context};
use futures::TryFutureExt;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
    Extensions,
};
use serde_json::Value;
use tracing::{debug, info_span, instrument, trace};
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes, ErrorReporter};
use voyager_primitives::{ConsensusStateMeta, IbcSpecId, Timestamp};
use voyager_vm::ItemId;

// use valuable::Valuable;
// use voyager_primitives::IbcStoreFormat;
use crate::{
    context::{LoadedModulesInfo, Modules, WithId},
    into_value,
    module::{
        ClientBootstrapModuleClient, ClientModuleClient, ConsensusModuleClient,
        RawProofModuleClient, RawStateModuleClient,
    },
    primitives::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
    rpc::{
        json_rpc_error_to_error_object, server::cache::StateRequest, IbcProof, IbcState,
        SelfClientState, SelfConsensusState, VoyagerRpcServer,
    },
    ExtensionsExt, IbcSpec, IbcStorePathKey, RawClientId, FATAL_JSONRPC_ERROR_CODE,
};

pub mod cache {
    use std::{future::Future, time::Duration};

    use futures::TryFutureExt;
    use jsonrpsee::core::RpcResult;
    use moka::policy::EvictionPolicy;
    use opentelemetry::KeyValue;
    use schemars::JsonSchema;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json::Value;
    use tracing::trace;
    use unionlabs::ibc::core::client::height::Height;
    use voyager_primitives::{ChainId, IbcSpec, IbcSpecId, IbcStorePathKey};

    #[derive(Debug, Clone)]
    pub struct Cache {
        state_cache: moka::future::Cache<StateRequest, Value>,
        state_cache_size_metric: opentelemetry::metrics::Gauge<u64>,
        state_cache_hit_counter_metric: opentelemetry::metrics::Counter<u64>,
        state_cache_miss_counter_metric: opentelemetry::metrics::Counter<u64>,
        // proof_cache: moka::future::Cache,
    }

    impl Cache {
        #[allow(clippy::new_without_default)]
        pub fn new(config: Config) -> Self {
            Self {
                state_cache: moka::future::CacheBuilder::new(config.state.capacity)
                    // .expire_after()
                    .time_to_live(Duration::from_secs(config.state.time_to_live))
                    .time_to_idle(Duration::from_secs(config.state.time_to_idle))
                    .eviction_policy(EvictionPolicy::lru())
                    .build(),
                state_cache_size_metric: opentelemetry::global::meter("voyager.cache.state")
                    .u64_gauge("size")
                    .build(),
                state_cache_hit_counter_metric: opentelemetry::global::meter("voyager.cache.state")
                    .u64_counter("hit")
                    .build(),
                state_cache_miss_counter_metric: opentelemetry::global::meter(
                    "voyager.cache.state",
                )
                .u64_counter("miss")
                .build(),
            }
        }

        pub async fn state<T: Serialize + DeserializeOwned>(
            &self,
            state_request: StateRequest,
            fut: impl Future<Output = RpcResult<Option<T>>>,
        ) -> RpcResult<Option<T>> {
            let attributes = &[KeyValue::new(
                "chain_id",
                state_request.chain_id.to_string(),
            )];

            self.state_cache_size_metric
                .record(self.state_cache.entry_count(), attributes);

            let init = fut
                .map_ok(|state| {
                    serde_json::to_value(state).expect("serialization is infallible; qed;")
                })
                .await?;

            if init.is_null() {
                Ok(None)
            } else {
                let entry = self.state_cache.entry(state_request).or_insert(init).await;

                if entry.is_fresh() {
                    self.state_cache_miss_counter_metric.add(1, attributes);
                } else {
                    self.state_cache_hit_counter_metric.add(1, attributes);
                }

                let value = entry.into_value();

                trace!(%value, "cached value");

                Ok(serde_json::from_value(value)
                    .expect("infallible; only valid values are inserted into the cache; qed;"))
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
    pub struct Config {
        pub state: CacheConfig,
    }

    #[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
    pub struct CacheConfig {
        pub capacity: u64,
        pub time_to_live: u64,
        pub time_to_idle: u64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct StateRequest {
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: Height,
        path: Value,
    }

    impl StateRequest {
        pub fn new<P: IbcStorePathKey>(
            chain_id: ChainId,
            height: Height,
            path: <P::Spec as IbcSpec>::StorePath,
        ) -> Self {
            Self {
                chain_id,
                ibc_spec_id: P::Spec::ID,
                height,
                path: serde_json::to_value(path).expect("serialization is infallible; qed;"),
            }
        }

        pub fn new_raw(
            chain_id: ChainId,
            ibc_spec_id: IbcSpecId,
            height: Height,
            path: Value,
        ) -> Self {
            Self {
                chain_id,
                ibc_spec_id,
                height,
                path,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
    item_id: Option<ItemId>,
}

#[derive(Debug, Clone)]
pub struct ServerInner {
    modules: OnceLock<Arc<Modules>>,
    cache: cache::Cache,
}

impl Server {
    #[allow(clippy::new_without_default)]
    pub fn new(cache_config: cache::Config) -> Self {
        Server {
            inner: Arc::new(ServerInner {
                modules: OnceLock::new(),
                cache: cache::Cache::new(cache_config),
            }),
            item_id: None,
        }
    }

    pub fn start(&self, modules: Arc<Modules>) {
        let was_not_already_started = self.inner.modules.set(modules).is_ok();

        assert!(was_not_already_started, "server has already been started");
    }

    pub fn with_id(&self, item_id: Option<ItemId>) -> Server {
        Server {
            inner: self.inner.clone(),
            item_id,
        }
    }

    fn span(&self) -> tracing::Span {
        match self.item_id {
            Some(item_id) => info_span!("processing_request", item_id = item_id.raw()),
            None => info_span!("processing_request"),
        }
    }

    /// Returns the contained modules, if they have been loaded.
    pub fn modules(&self) -> RpcResult<&Modules> {
        self.inner.modules()
    }

    #[instrument(skip_all, fields(%height, %chain_id))]
    pub async fn query_height(&self, chain_id: &ChainId, height: QueryHeight) -> RpcResult<Height> {
        match height {
            QueryHeight::Latest => {
                let latest_height = self
                    .modules()?
                    .consensus_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_height(false)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                debug!(%latest_height, finalized = false, "queried latest height");

                Ok(latest_height)
            }
            QueryHeight::Finalized => {
                let latest_height = self
                    .modules()?
                    .consensus_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_height(true)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                debug!(%latest_height, finalized = true, "queried latest height");

                Ok(latest_height)
            }
            QueryHeight::Specific(height) => Ok(height),
        }
    }
}

impl ServerInner {
    /// Returns the contained modules, if they have been loaded.
    fn modules(&self) -> RpcResult<&Modules> {
        self.modules
            .get()
            .map(|x| &**x)
            .ok_or_else(|| ErrorObject::owned(-2, "server has not started", None::<()>))
    }
}

impl Server {
    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_height(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        self.span()
            .in_scope(|| async {
                trace!("querying latest height");

                let latest_height = self
                    .inner
                    .modules()?
                    .consensus_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_height(finalized)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    %latest_height,
                    "queried latest height"
                );

                Ok(latest_height)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_timestamp(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        self.span()
            .in_scope(|| async {
                trace!("querying latest timestamp");

                let latest_timestamp = self
                    .inner
                    .modules()?
                    .consensus_module(chain_id)?
                    .with_id(self.item_id)
                    .query_latest_timestamp(finalized)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    latest_timestamp = latest_timestamp.as_nanos(),
                    "queried latest timestamp"
                );

                Ok(latest_timestamp)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, client_id = %client_id.0))]
    pub async fn client_info(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching client info");

                let client_info = self
                    .inner
                    .modules()?
                    .state_module(chain_id, ibc_spec_id)?
                    .with_id(self.item_id)
                    .client_info_raw(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                match client_info {
                    Some(ref client_info) => {
                        trace!(
                            %client_info.ibc_interface,
                            %client_info.client_type,
                            "fetched client info"
                        );
                    }
                    None => {
                        trace!("client not found");
                    }
                }

                Ok(client_info)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.0))]
    pub async fn client_state_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching client state meta");

                let height = self.query_height(chain_id, at).await?;

                let modules = self.inner.modules()?;

                let state_module = modules
                    .state_module(chain_id, ibc_spec_id)?
                    .with_id(self.item_id);

                let client_info = state_module
                    .client_info_raw(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                let Some(client_info) = client_info else {
                    trace!("client info for client {client_id} not found at height {height} on chain {chain_id}");
                    return Ok(None);
                };

                let raw_client_state = state_module
                    .query_ibc_state_raw(
                        height,
                        (self
                            .modules()?
                            .ibc_spec_handlers
                            .handlers
                            .get(ibc_spec_id)
                            .ok_or_else(|| fatal_error(&*anyhow!("ibc spec {ibc_spec_id} is not supported in this build of voyager")))?
                            .client_state_path)(client_id.clone())
                        .map_err(|err| fatal_error(&*err))?,
                    )
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%raw_client_state);

                let client_state = serde_json::from_value::<Option<Bytes>>(raw_client_state)
                    .with_context(|| format!("querying client state for client {client_id} at {height} on {chain_id}"))
                    .map_err(|e| fatal_error(&*e))?;

                let Some(client_state) = client_state else {
                    trace!("client state for client {client_id} not found at height {height} on chain {chain_id}");
                    return Ok(None);
                };

                let meta = modules
                    .client_module(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        ibc_spec_id,
                    )
                    ?
                    .with_id(self.item_id)
                    .decode_client_state_meta(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    client_state_meta.height = %meta.counterparty_height,
                    client_state_meta.chain_id = %meta.counterparty_chain_id,
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched client state meta"
                );

                Ok(Some(meta))
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.0))]
    pub async fn consensus_state_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching consensus state meta");

                let height = self.query_height(chain_id, at).await?;

                let modules = self.inner.modules()?;

                let state_module = modules
                    .state_module(chain_id, ibc_spec_id)?
                    .with_id(self.item_id);

                let client_info = state_module
                    .client_info_raw(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                let Some(client_info) = client_info else {
                    trace!("client info for client {client_id} not found at height {height} on chain {chain_id}");
                    return Ok(None);
                };

                let raw_consensus_state = state_module
                    .query_ibc_state_raw(
                        height,
                        (self
                            .modules()?
                            .ibc_spec_handlers
                            .handlers
                            .get(ibc_spec_id)
                            .ok_or_else(|| fatal_error(&*anyhow!("ibc spec {ibc_spec_id} is not supported in this build of voyager")))?
                            .consensus_state_path)(client_id.clone(), counterparty_height.to_string())
                        .map_err(|err| fatal_error(&*err))?,
                    )
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%raw_consensus_state);

                let client_state = serde_json::from_value::<Option<Bytes>>(raw_consensus_state)
                    .with_context(|| format!("querying consensus state for client {client_id} at {height} on {chain_id}"))
                    .map_err(|e| fatal_error(&*e))?;

                let Some(consensus_state) = client_state else {
                    trace!("consensus state for client {client_id} not found at height {height} on chain {chain_id}");
                    return Ok(None);
                };

                let meta = modules
                    .client_module(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        ibc_spec_id,
                    )
                    ?
                    .with_id(self.item_id)
                    .decode_consensus_state_meta(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    consensus_state_meta.timestamp = %meta.timestamp,
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched consensus state meta"
                );

                Ok(Some(meta))
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %query))]
    async fn query_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        query: Value,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                trace!("query");

                let state_module = self
                    .inner
                    .modules()?
                    .state_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let value = state_module
                    .query_raw(into_value(query.clone()))
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%value, "queried");

                Ok(value)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_state_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcState<Value>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let height = self.query_height(&chain_id, height).await?;

                let state_module = self
                    .inner
                    .modules()?
                    .state_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let state = self
                    .inner
                    .cache
                    .state::<Value>(
                        StateRequest::new_raw(
                            chain_id.clone(),
                            ibc_spec_id.clone(),
                            height,
                            path.clone(),
                        ),
                        state_module
                            .query_ibc_state_raw(height, into_value(path.clone()))
                            .map_ok(|state| {
                                // TODO: Use valuable here
                                trace!(%state, "fetched ibc state");

                                if state.is_null() {
                                    None
                                } else {
                                    Some(state)
                                }
                            })
                            .map_err(|e| json_rpc_error_to_error_object(e)),
                    )
                    .await?;

                Ok(IbcState { height, state })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_proof_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProof> {
        self.span()
            .in_scope(|| async {
                let height = self.query_height(&chain_id, height).await?;

                debug!("fetching ibc proof");

                let proof_module = self
                    .inner
                    .modules()?
                    .proof_module(&chain_id, &ibc_spec_id)?
                    .with_id(self.item_id);

                let (proof, proof_type) = proof_module
                    .query_ibc_proof_raw(height, path)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                debug!(%proof, ?proof_type, "fetched ibc proof");

                Ok(IbcProof {
                    height,
                    proof,
                    proof_type,
                })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height, ?path))]
    pub async fn query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcState<P::Value>> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let state_module = self
                    .inner
                    .modules()?
                    .state_module(chain_id, &P::Spec::ID)?
                    .with_id(self.item_id);

                let state = self
                    .inner
                    .cache
                    .state::<P::Value>(
                        StateRequest::new::<P>(chain_id.clone(), height, path.clone()),
                        state_module
                            .query_ibc_state_raw(height, into_value(path.clone()))
                            .map_ok(|state| {
                                // TODO: Use valuable here
                                trace!(%state, "fetched ibc state");

                                serde_json::from_value(state).unwrap()
                            })
                            .map_err(|e| json_rpc_error_to_error_object(e)),
                    )
                    .await?;

                Ok(IbcState { height, state })
            })
            .await
    }

    #[instrument(
        skip_all,
        fields(
            %chain_id,
            ibc_spec_id = %P::Spec::ID,
            %height,
            path = %into_value(path.clone())
        )
    )]
    pub async fn query_ibc_proof<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcProof> {
        self.span()
            .in_scope(|| async {
                trace!("fetching ibc state");

                let proof_module = self
                    .inner
                    .modules()?
                    .proof_module(chain_id, &P::Spec::ID)?
                    .with_id(self.item_id);

                let (proof, proof_type) = proof_module
                    .query_ibc_proof_raw(height, into_value(path.clone()))
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                debug!(%proof, ?proof_type, "fetched ibc proof");

                Ok(IbcProof {
                    height,
                    proof,
                    proof_type,
                })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %client_type, %height))]
    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: Height,
        config: Value,
    ) -> RpcResult<SelfClientState> {
        self.span()
            .in_scope(|| async {
                trace!("querying self client state");

                let client_bootstrap_module = self
                    .inner
                    .modules()?
                    .client_bootstrap_module(&chain_id, &client_type)?
                    .with_id(self.item_id);

                let state = client_bootstrap_module
                    .self_client_state(height, config)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched self client state");

                Ok(SelfClientState { height, state })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %client_type, %height))]
    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusState> {
        self.span()
            .in_scope(|| async {
                trace!("querying self consensus state");

                let client_bootstrap_module = self
                    .inner
                    .modules()?
                    .client_bootstrap_module(&chain_id, &client_type)?
                    .with_id(self.item_id);

                let height = self.query_height(&chain_id, height).await?;

                let state = client_bootstrap_module
                    .self_consensus_state(height, config)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched self consensus state");

                Ok(SelfConsensusState { height, state })
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id, %proof))]
    pub async fn encode_proof(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                trace!("encoding proof");

                let client_module = self
                    .inner
                    .modules()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let proof = client_module
                    .encode_proof(proof)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%proof, "encoded proof");

                Ok(proof)
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id, %header))]
    pub async fn encode_header(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        header: Value,
    ) -> RpcResult<Bytes> {
        self.span()
            .in_scope(|| async {
                trace!("encoding header");

                let client_module = self
                    .inner
                    .modules()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let header = client_module
                    .encode_header(header)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(%header, "encoded header");

                Ok(header)
            })
            .await
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_client_state_meta(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.span()
            .in_scope(|| async {
                trace!("decoding client state meta");

                let client_module = self
                    .inner
                    .modules()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id);

                let meta = client_module
                    .decode_client_state_meta(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    height = %meta.counterparty_height,
                    chain_id = %meta.counterparty_chain_id,
                    "decoded client state meta"
                );

                Ok(meta)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_client_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                self.inner
                    .modules()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .decode_client_state(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_consensus_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.span()
            .in_scope(|| async {
                self.inner
                    .modules()?
                    .client_module(client_type, ibc_interface, ibc_spec_id)?
                    .with_id(self.item_id)
                    .decode_consensus_state(consensus_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)
            })
            .await
    }
}

/// rpc impl
#[async_trait]
impl VoyagerRpcServer for Server {
    async fn info(&self, _: &Extensions) -> RpcResult<LoadedModulesInfo> {
        Ok(self.modules()?.info())
    }

    async fn equivalent_chain_ids(
        &self,
        _: &Extensions,
        chain_id: ChainId,
    ) -> RpcResult<Vec<ChainId>> {
        Ok(self
            .inner
            .modules()?
            .equivalent_chain_ids()
            .equivalents(&chain_id)
            .cloned()
            .collect())
    }

    // =========
    // CONSENSUS
    // =========

    async fn query_latest_height(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        self.with_id(e.try_get().ok().cloned())
            .query_latest_height(&chain_id, finalized)
            .await
    }

    async fn query_latest_timestamp(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        self.with_id(e.try_get().ok().cloned())
            .query_latest_timestamp(&chain_id, finalized)
            .await
    }

    // =====
    // STATE
    // =====

    async fn client_info(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.with_id(e.try_get().ok().cloned())
            .client_info(&chain_id, &ibc_spec_id, client_id)
            .await
    }

    async fn client_state_meta(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.with_id(e.try_get().ok().cloned())
            .client_state_meta(&chain_id, &ibc_spec_id, at, client_id)
            .await
    }

    async fn consensus_state_meta(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.with_id(e.try_get().ok().cloned())
            .consensus_state_meta(&chain_id, &ibc_spec_id, at, client_id, counterparty_height)
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %query))]
    async fn query(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        query: Value,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .query_raw(chain_id, ibc_spec_id, query)
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcState<Value>> {
        self.with_id(e.try_get().ok().cloned())
            .query_ibc_state_raw(chain_id, ibc_spec_id, height, path)
            .await
    }

    // =====
    // PROOF
    // =====

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, %height, %path))]
    async fn query_ibc_proof(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProof> {
        self.with_id(e.try_get().ok().cloned())
            .query_ibc_proof_raw(chain_id, ibc_spec_id, height, path)
            .await
    }

    // ==========
    // SELF STATE
    // ==========

    async fn self_client_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfClientState> {
        let item_id = e.try_get().ok().cloned();
        let this = self.with_id(item_id);

        let height = this.query_height(&chain_id, height).await?;

        this.self_client_state(chain_id, client_type, height, config)
            .await
    }

    async fn self_consensus_state(
        &self,
        e: &Extensions,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusState> {
        self.with_id(e.try_get().ok().cloned())
            .self_consensus_state(chain_id, client_type, height, config)
            .await
    }

    // =====
    // CODEC
    // =====

    // TODO: Use valuable here
    async fn encode_proof(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_proof(&client_type, &ibc_interface, &ibc_spec_id, proof)
            .await
    }

    async fn encode_header(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        header: Value,
    ) -> RpcResult<Bytes> {
        self.with_id(e.try_get().ok().cloned())
            .encode_header(&client_type, &ibc_interface, &ibc_spec_id, header)
            .await
    }

    // TODO: Use valuable here
    async fn decode_client_state_meta(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.with_id(e.try_get().ok().cloned())
            .decode_client_state_meta(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_client_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .decode_client_state(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_consensus_state(
        &self,
        e: &Extensions,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.with_id(e.try_get().ok().cloned())
            .decode_consensus_state(&client_type, &ibc_interface, &ibc_spec_id, consensus_state)
            .await
    }
}

pub(crate) fn fatal_error(t: impl core::error::Error) -> ErrorObjectOwned {
    ErrorObject::owned(
        FATAL_JSONRPC_ERROR_CODE,
        ErrorReporter(t).to_string(),
        None::<()>,
    )
}
