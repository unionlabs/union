use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use serde_json::{json, Value};
use tracing::debug;
use unionlabs::{
    ibc::core::client::height::Height,
    ics24::{IbcPath, Path},
    id::ClientId,
    ErrorReporter, QueryHeight,
};

use crate::{
    data::ClientInfo, plugin::ClientStateMeta, ChainId, ClientType, IbcInterface,
    FATAL_JSONRPC_ERROR_CODE,
};

#[rpc(
    client,
    server,
    client_bounds(Self: Send + Sync),
    server_bounds(Self:),
    namespace = "voyager",
)]
pub trait VoyagerRpc {
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<Info>;

    #[method(name = "queryLatestHeight")]
    async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height>;

    #[method(name = "clientInfo")]
    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo>;

    #[method(name = "clientMeta")]
    async fn client_meta(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "queryibcState")]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcState>;

    #[method(name = "queryibcProof")]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcProof>;

    #[method(name = "selfClientState")]
    async fn self_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfClientState>;

    #[method(name = "selfConsensusState")]
    async fn self_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState>;
}

#[model]
pub struct Info {
    chain: Vec<ChainId<'static>>,
    consensus: Vec<ChainId<'static>>,
    client: Vec<(ClientType<'static>, IbcInterface<'static>)>,
}

#[model]
pub struct IbcState<State = Value> {
    pub chain_id: ChainId<'static>,
    pub path: Path,
    /// The height that the state was read at.
    pub height: Height,
    pub state: State,
}

#[model]
pub struct IbcProof {
    pub chain_id: ChainId<'static>,
    pub path: Path,
    /// The height that the proof was read at.
    pub height: Height,
    pub proof: Value,
}

#[model]
pub struct SelfClientState {
    pub height: Height,
    pub state: Value,
}

#[model]
pub struct SelfConsensusState {
    pub height: Height,
    pub state: Value,
}

/// Serves the same purpose as `ChainModuleClientExt`.
pub trait VoyagerRpcClientExt: VoyagerRpcClient {
    // TODO: Maybe rename? Cor likes "_checked"
    // TODO: Maybe take by ref here?
    #[allow(async_fn_in_trait)]
    async fn query_ibc_state_typed<P: IbcPath>(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        path: P,
    ) -> Result<IbcState<P::Value>, jsonrpsee::core::client::Error> {
        debug!(%path, %at, "querying ibc state");

        let ibc_state = self
            .query_ibc_state(chain_id.clone(), at, path.clone().into())
            .await?;

        Ok(serde_json::from_value::<P::Value>(ibc_state.state.clone())
            .map(|value| IbcState {
                chain_id: ibc_state.chain_id,
                path: ibc_state.path,
                height: ibc_state.height,
                state: value,
            })
            .map_err(|e| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize state: {}", ErrorReporter(e)),
                    Some(json!({
                        "chain_id": chain_id,
                        "path": path,
                        "state": ibc_state.state
                    })),
                )
            })?)
    }
}

impl<T> VoyagerRpcClientExt for T where T: VoyagerRpcClient {}

// State(FetchState),
// RawProof(FetchRawProof),

// LatestHeight(FetchLatestHeight),

// ClientInfo(FetchClientInfo),

// UnfinalizedTrustedClientState(FetchUnfinalizedTrustedClientState),

// SelfClientState(FetchSelfClientState),
// SelfConsensusState(FetchSelfConsensusState),

// DecodeClientStateMeta(DecodeClientStateMeta),
// DecodeConsensusStateMeta(DecodeConsensusStateMeta),

// EncodeClientState(EncodeClientState),
// EncodeConsensusState(EncodeConsensusState),
// EncodeHeader(EncodeHeader),

// EncodeProof(EncodeProof),

// UpdateHeaders(FetchUpdateHeaders),

// MakeMsgCreateClient(MakeMsgCreateClient),

// MakeMsgConnectionOpenTry(MakeMsgConnectionOpenTry),
// MakeMsgConnectionOpenAck(MakeMsgConnectionOpenAck),
// MakeMsgConnectionOpenConfirm(MakeMsgConnectionOpenConfirm),

// MakeMsgChannelOpenTry(MakeMsgChannelOpenTry),
// MakeMsgChannelOpenAck(MakeMsgChannelOpenAck),
// MakeMsgChannelOpenConfirm(MakeMsgChannelOpenConfirm),

// MakeMsgAcknowledgement(MakeMsgAcknowledgement),
// MakeMsgRecvPacket(MakeMsgRecvPacket),

// Height(WaitForHeight),
// HeightRelative(WaitForHeightRelative),
// Timestamp(WaitForTimestamp),
// TrustedHeight(WaitForTrustedHeight),

// Plugin(PluginMessage<C>),

pub mod server {
    use std::sync::{Arc, OnceLock};

    use jsonrpsee::{
        core::RpcResult,
        types::{ErrorObject, ErrorObjectOwned},
    };
    use serde_json::Value;
    use tonic::async_trait;
    use tracing::{debug, error, info_span, instrument, Instrument};
    use unionlabs::{
        ibc::core::client::height::Height,
        ics24::{ClientStatePath, Path},
        id::ClientId,
        ErrorReporter, QueryHeight,
    };

    use crate::{
        context::Modules,
        data::ClientInfo,
        plugin::{
            ChainModuleClient, ChainModuleClientExt, ClientModuleClient, ClientStateMeta,
            ConsensusModuleClient,
        },
        rpc::{
            json_rpc_error_to_rpc_error, IbcProof, IbcState, Info, SelfClientState,
            SelfConsensusState, VoyagerRpcServer,
        },
        ChainId, FATAL_JSONRPC_ERROR_CODE,
    };

    #[derive(Debug, Clone)]
    pub struct Server {
        inner: Arc<ServerInner>,
    }

    #[derive(macros::Debug, Clone)]
    pub struct ServerInner {
        modules: OnceLock<Arc<Modules>>,
        #[debug("Cache({:?}, {})", self.ibc_state_cache.name(), self.ibc_state_cache.entry_count())]
        ibc_state_cache: moka::future::Cache<(ChainId<'static>, Path, Height), Value>,
    }

    // #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    // pub struct IbcStateCacheKey<'a> {
    //     chain_id: ChainId<'a>,
    //     path: Cow<'a, Path>,
    //     // height is copy so don't bother wrapping it
    //     height: Height,
    // }

    // impl<'a> IbcStateCacheKey<'a> {
    //     pub fn new(chain_id: ChainId<'a>, path: &'a Path, height: Height) -> Self {
    //         Self {
    //             chain_id,
    //             path: Cow::Borrowed(path),
    //             height,
    //         }
    //     }
    // }

    // impl IbcStateCacheKey<'static> {
    //     pub fn new_owned(chain_id: ChainId<'static>, path: Path, height: Height) -> Self {
    //         Self {
    //             chain_id,
    //             path: Cow::Owned(path),
    //             height,
    //         }
    //     }
    // }

    impl Server {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Server {
                inner: Arc::new(ServerInner {
                    modules: OnceLock::new(),
                    ibc_state_cache: moka::future::Cache::builder()
                        .eviction_listener(|k, v, why| {
                            error!(?k, ?v, ?why, "value evicted from the cache")
                        })
                        .max_capacity(10_000)
                        .name("ibc_state_cache")
                        .build(),
                }),
            }
        }

        pub fn start(&self, modules: Arc<Modules>) {
            assert!(
                self.inner.modules.set(modules).is_ok(),
                "server has already been started"
            );
        }

        /// Returns the contained modules, if they have been loaded.
        pub fn modules(&self) -> RpcResult<&Modules> {
            self.inner.modules()
        }
    }

    impl ServerInner {
        /// Returns the contained modules, if tey have been loaded.
        fn modules(&self) -> RpcResult<&Modules> {
            self.modules
                .get()
                .map(|x| &**x)
                .ok_or_else(|| ErrorObject::owned(-2, "server has not started", None::<()>))
        }

        #[instrument(skip_all, fields(%height, %chain_id))]
        async fn query_height(
            &self,
            chain_id: &ChainId<'_>,
            height: QueryHeight,
        ) -> RpcResult<Height> {
            match height {
                QueryHeight::Latest => {
                    let latest_height = self
                        .modules()?
                        .chain_module::<Value, Value, Value>(chain_id)
                        .map_err(fatal_error)?
                        .query_latest_height()
                        .await
                        .map_err(json_rpc_error_to_rpc_error)?;

                    debug!(%latest_height, "queried latest height");

                    Ok(latest_height)
                }
                QueryHeight::Specific(height) => Ok(height),
            }
        }

        async fn query_ibc_state_cached<'a>(
            &self,
            chain_id: ChainId<'a>,
            at: Height,
            path: Path,
        ) -> Result<IbcState, jsonrpsee::core::client::Error> {
            let key = (chain_id.clone().into_static(), path.clone(), at);

            let state = self
                .ibc_state_cache
                .entry_by_ref(&key)
                .or_try_insert_with(
                    async {
                        debug!(%path, %at, "querying ibc state");

                        let state = self
                            .modules()?
                            .chain_module::<Value, Value, Value>(&chain_id)
                            .map_err(fatal_error)?
                            .query_ibc_state(at, path)
                            .await
                            .map_err(json_rpc_error_to_rpc_error)?;

                        RpcResult::Ok(state)
                    }
                    .instrument(info_span!("ibc state cache fetcher")),
                )
                .await
                .map_err(Arc::unwrap_or_clone)?;

            let (chain_id, path, height) = key;

            if !state.is_fresh() {
                debug!(
                    cache = %self.ibc_state_cache.name().unwrap_or_default(),
                    key.chain_id = %chain_id,
                    key.path = %path,
                    key.height = %height,
                    "cache hit"
                );
            }

            Ok(IbcState {
                chain_id,
                path,
                height,
                state: state.into_value(),
            })

            // Ok(
            //     serde_json::from_value::<P::Value>(state.clone()).map_err(|e| {
            //         ErrorObject::owned(
            //             FATAL_JSONRPC_ERROR_CODE,
            //             format!("unable to deserialize state: {}", ErrorReporter(e)),
            //             Some(json!({
            //                 "path": path,
            //                 "state": state
            //             })),
            //         )
            //     })?,
            // )
        }
    }

    #[async_trait]
    impl VoyagerRpcServer for Server {
        #[instrument(skip_all)]
        async fn info(&self) -> RpcResult<Info> {
            dbg!(self.inner.ibc_state_cache.iter().collect::<Vec<_>>());

            let chain = self
                .inner
                .modules()?
                .loaded_chain_modules()
                .cloned()
                .collect();
            let consensus = self
                .inner
                .modules()?
                .loaded_consensus_modules()
                .cloned()
                .collect();
            let client = self
                .inner
                .modules()?
                .loaded_client_modules()
                .flat_map(|(c, is)| is.map(|i| (c.clone(), i.clone())))
                .collect();

            Ok(Info {
                chain,
                consensus,
                client,
            })
        }

        #[instrument(skip_all, fields(%chain_id))]
        async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height> {
            debug!("querying latest height");

            let latest_height = self
                .inner
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?
                .query_latest_height()
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            debug!(%latest_height, "queried latest height");

            Ok(latest_height)
        }

        #[instrument(skip_all, fields(%chain_id, %client_id))]
        async fn client_info(
            &self,
            chain_id: ChainId<'static>,
            client_id: ClientId,
        ) -> RpcResult<ClientInfo> {
            debug!("fetching client info");

            let client_info = self
                .inner
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?
                .client_info(client_id)
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            debug!(%client_info.ibc_interface, %client_info.client_type, "fetched client info");

            Ok(client_info)
        }

        #[instrument(skip_all, fields(%chain_id, height = %at, %client_id))]
        async fn client_meta(
            &self,
            chain_id: ChainId<'static>,
            at: QueryHeight,
            client_id: ClientId,
        ) -> RpcResult<ClientStateMeta> {
            debug!("fetching client meta");

            let height = self.inner.query_height(&chain_id, at).await?;

            let client_info = self
                .inner
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?
                .client_info(client_id.clone())
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            let client_state = self
                .inner
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?
                .query_ibc_state_typed(height, ClientStatePath { client_id })
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            let meta = self
                .inner
                .modules()?
                .client_module::<Value, Value, Value>(
                    &client_info.client_type,
                    &client_info.ibc_interface,
                )
                .map_err(fatal_error)?
                .decode_client_state_meta(client_state.0.into())
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            debug!(
                client_state_meta.height = %meta.height,
                client_state_meta.chain_id = %meta.chain_id,
                %client_info.ibc_interface,
                %client_info.client_type,
                "fetched client meta"
            );

            Ok(meta)
        }

        #[instrument(skip_all, fields(%chain_id, %path, %height))]
        async fn query_ibc_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
            path: Path,
        ) -> RpcResult<IbcState> {
            debug!("fetching ibc state");

            let height = self.inner.query_height(&chain_id, height).await?;

            let state = self
                .inner
                .query_ibc_state_cached(chain_id, height, path.clone())
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            // TODO: Use valuable here
            debug!(state = %state.state, "fetched ibc state");

            Ok(state)
        }

        #[instrument(skip_all, fields(%chain_id, %path, %height))]
        async fn query_ibc_proof(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
            path: Path,
        ) -> RpcResult<IbcProof> {
            debug!("fetching ibc state");

            let chain_module = self
                .inner
                .modules()?
                .chain_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.inner.query_height(&chain_id, height).await?;

            let proof = chain_module
                .query_ibc_proof(height, path.clone())
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            // TODO: Use valuable here
            debug!(%proof, "fetched ibc proof");

            Ok(IbcProof {
                chain_id,
                path,
                height,
                proof,
            })
        }

        #[instrument(skip_all, fields(%chain_id, %height))]
        async fn self_client_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
        ) -> RpcResult<SelfClientState> {
            debug!("querying self client state");

            let chain_module = self
                .inner
                .modules()?
                .consensus_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.inner.query_height(&chain_id, height).await?;

            let state = chain_module
                .self_client_state(height)
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            // TODO: Use valuable here
            debug!(%state, "fetched self client state");

            Ok(SelfClientState { height, state })
        }

        #[instrument(skip_all, fields(%chain_id, %height))]
        async fn self_consensus_state(
            &self,
            chain_id: ChainId<'static>,
            height: QueryHeight,
        ) -> RpcResult<SelfConsensusState> {
            debug!("querying self consensus state");

            let chain_module = self
                .inner
                .modules()?
                .consensus_module::<Value, Value, Value>(&chain_id)
                .map_err(fatal_error)?;

            let height = self.inner.query_height(&chain_id, height).await?;

            let state = chain_module
                .self_consensus_state(height)
                .await
                .map_err(json_rpc_error_to_rpc_error)?;

            // TODO: Use valuable here
            debug!(%state, "fetched self consensus state");

            Ok(SelfConsensusState { height, state })
        }
    }

    pub(crate) fn fatal_error(t: impl std::error::Error) -> ErrorObjectOwned {
        ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            ErrorReporter(t).to_string(),
            None::<()>,
        )
    }
}

pub fn json_rpc_error_to_rpc_error(value: jsonrpsee::core::client::Error) -> ErrorObjectOwned {
    match value {
        jsonrpsee::core::client::Error::Call(error) => error,
        value => ErrorObject::owned(-1, format!("error: {}", ErrorReporter(value)), None::<()>),
    }
}
