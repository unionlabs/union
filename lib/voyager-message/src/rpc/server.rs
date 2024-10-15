use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, error, info_span, instrument, Instrument};
use unionlabs::{
    ibc::core::client::height::Height,
    ics24::{ClientStatePath, IbcPath, Path},
    id::ClientId,
    ErrorReporter, QueryHeight,
};
use valuable::Valuable;

use crate::{
    context::{LoadedModulesInfo, Modules},
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface},
    module::{ChainModuleClient, ChainModuleClientExt, ClientModuleClient, ConsensusModuleClient},
    rpc::{
        json_rpc_error_to_error_object, IbcProof, IbcState, SelfClientState, SelfConsensusState,
        VoyagerRpcServer,
    },
    FATAL_JSONRPC_ERROR_CODE,
};

#[derive(Debug, Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
}

#[derive(Debug, Clone)]
pub struct ServerInner {
    modules: OnceLock<Arc<Modules>>,
    ibc_state_cache: Cache,
}

#[derive(Clone)]
struct Cache(moka::future::Cache<(ChainId<'static>, Path, Height), Value>);

impl Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cache({:?}, {})", self.0.name(), self.0.entry_count())
    }
}

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct IbcStateCacheKey<'a> {
//     chain_id: ChainId<'a>,
//     path: Cow<'a, Path>,
//     // height is copy so don't bother wrapping it
//     height: Height,
// }

// impl<'a> IbcStateCacheKey<'a> {
//     pub fn new(chain_id: ChainId<'a>, path: &'a Path, height: Height) -> Self
// {         Self {
//             chain_id,
//             path: Cow::Borrowed(path),
//             height,
//         }
//     }
// }

// impl IbcStateCacheKey<'static> {
//     pub fn new_owned(chain_id: ChainId<'static>, path: Path, height: Height)
// -> Self {         Self {
//             chain_id,
//             path: Cow::Owned(path),
//             height,
//         }
//     }
// }

// enum CacheKey {
//     IbcState {
//         chain_id: ChainId<'static>,
//         path: Path,
//         height: Height,
//     },
//     ClientMeta {
//         chain_id: ChainId<'static>,
//         height: Height,
//     },
//     ClientInfo {
//         chain_id: ChainId<'static>,
//     },
// }

impl Server {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Server {
            inner: Arc::new(ServerInner {
                modules: OnceLock::new(),
                ibc_state_cache: Cache(
                    moka::future::Cache::builder()
                        .eviction_listener(|k, v, why| {
                            error!(?k, ?v, ?why, "value evicted from the cache")
                        })
                        .max_capacity(10_000)
                        .name("ibc_state_cache")
                        .build(),
                ),
            }),
        }
    }

    pub fn start(&self, modules: Arc<Modules>) {
        let was_not_already_started = self.inner.modules.set(modules).is_ok();

        assert!(was_not_already_started, "server has already been started");
    }

    /// Returns the contained modules, if they have been loaded.
    pub fn modules(&self) -> RpcResult<&Modules> {
        self.inner.modules()
    }

    #[instrument(skip_all, fields(%height, %chain_id))]
    pub async fn query_height(
        &self,
        chain_id: &ChainId<'_>,
        height: QueryHeight,
    ) -> RpcResult<Height> {
        match height {
            QueryHeight::Latest => {
                let latest_height = self
                    .modules()?
                    .chain_module(chain_id)
                    .map_err(fatal_error)?
                    .query_latest_height()
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                debug!(%latest_height, "queried latest height");

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

    async fn query_ibc_state_cached<'a>(
        &self,
        chain_id: &ChainId<'a>,
        at: Height,
        path: Path,
    ) -> Result<IbcState, jsonrpsee::core::client::Error> {
        let key = (chain_id.clone().into_owned(), path.clone(), at);

        let state = self
            .ibc_state_cache
            .0
            .entry_by_ref(&key)
            .or_try_insert_with(
                async {
                    debug!(%chain_id, path = path.as_value(), %at, "querying ibc state (not yet cached)");

                    let state = self
                        .modules()?
                        .chain_module(chain_id)
                        .map_err(fatal_error)?
                        .query_ibc_state(at, path)
                        .await
                        .map_err(json_rpc_error_to_error_object)?;

                    RpcResult::Ok(state)
                }
                .instrument(info_span!("ibc state cache fetcher")),
            )
            .await
            .map_err(Arc::unwrap_or_clone)?;

        let (chain_id, path, height) = key;

        if !state.is_fresh() {
            debug!(
                cache = %self.ibc_state_cache.0.name().unwrap_or_default(),
                key.chain_id = %chain_id,
                key.path = path.as_value(),
                key.height = %height,
                // raw_key = %format_args!("chain_id:{chain_id};path:{path};height:{height}"),
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
        //             format!("unable to deserialize state: {}",
        // ErrorReporter(e)),             Some(json!({
        //                 "path": path,
        //                 "state": state
        //             })),
        //         )
        //     })?,
        // )
    }
}

impl Server {
    #[instrument(skip_all, fields(%chain_id))]
    pub async fn query_latest_height(&self, chain_id: &ChainId<'static>) -> RpcResult<Height> {
        debug!("querying latest height");

        let latest_height = self
            .inner
            .modules()?
            .chain_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_height()
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(
            latest_height = latest_height.as_value(),
            "queried latest height"
        );

        Ok(latest_height)
    }

    #[instrument(skip_all, fields(%chain_id))]
    pub async fn query_latest_timestamp(&self, chain_id: &ChainId<'static>) -> RpcResult<i64> {
        debug!("querying latest timestamp");

        let latest_timestamp = self
            .inner
            .modules()?
            .chain_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_timestamp()
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(latest_timestamp, "queried latest timestamp");

        Ok(latest_timestamp)
    }

    #[instrument(skip_all, fields(chain_id, client_id))]
    pub async fn client_info(
        &self,
        chain_id: &ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo> {
        debug!("fetching client info");

        let client_info = self
            .inner
            .modules()?
            .chain_module(chain_id)
            .map_err(fatal_error)?
            .client_info(client_id)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(
            %client_info.ibc_interface,
            %client_info.client_type,
            "fetched client info"
        );

        Ok(client_info)
    }

    #[instrument(skip_all, fields(%chain_id, height = %at, client_id = client_id.as_value()))]
    pub async fn client_meta(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<ClientStateMeta> {
        debug!("fetching client meta");

        let height = self.query_height(&chain_id, at).await?;

        let client_info = self
            .inner
            .modules()?
            .chain_module(&chain_id)
            .map_err(fatal_error)?
            .client_info(client_id.clone())
            .await
            .map_err(json_rpc_error_to_error_object)?;

        let client_state = self
            .inner
            .modules()?
            .chain_module(&chain_id)
            .map_err(fatal_error)?
            .query_ibc_state_typed(height, ClientStatePath { client_id })
            .await
            .map_err(json_rpc_error_to_error_object)?;

        let meta = self
            .inner
            .modules()?
            .client_module(&client_info.client_type, &client_info.ibc_interface)
            .map_err(fatal_error)?
            .decode_client_state_meta(client_state)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(
            client_state_meta = meta.as_value(),
            client_info = client_info.as_value(),
            "fetched client meta"
        );

        Ok(meta)
    }

    #[instrument(skip_all, fields(%chain_id, path = path.as_value(), %height))]
    pub async fn query_ibc_state(
        &self,
        chain_id: &ChainId<'_>,
        height: Height,
        path: Path,
    ) -> RpcResult<IbcState> {
        debug!("fetching ibc state");

        // let height = self.inner.query_height(&chain_id, height).await?;

        let state = self
            .inner
            .query_ibc_state_cached(chain_id, height, path.clone())
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(state = %state.state, "fetched ibc state");

        Ok(state)
    }

    #[instrument(skip_all, fields(%chain_id, path = path.as_value(), %height))]
    pub async fn query_ibc_proof(
        &self,
        chain_id: &ChainId<'_>,
        height: Height,
        path: Path,
    ) -> RpcResult<IbcProof> {
        debug!("fetching ibc state");

        let chain_module = self
            .inner
            .modules()?
            .chain_module(chain_id)
            .map_err(fatal_error)?;

        // let height = self.inner.query_height(&chain_id, height).await?;

        let proof = chain_module
            .query_ibc_proof(height, path.clone())
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%proof, "fetched ibc proof");

        Ok(IbcProof {
            path,
            height,
            proof,
        })
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: Height,
    ) -> RpcResult<SelfClientState> {
        debug!("querying self client state");

        let chain_module = self
            .inner
            .modules()?
            .consensus_module(&chain_id)
            .map_err(fatal_error)?;

        let state = chain_module
            .self_client_state(height)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%state, "fetched self client state");

        Ok(SelfClientState { height, state })
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState> {
        debug!("querying self consensus state");

        let chain_module = self
            .inner
            .modules()?
            .consensus_module(&chain_id)
            .map_err(fatal_error)?;

        let height = self.query_height(&chain_id, height).await?;

        let state = chain_module
            .self_consensus_state(height)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%state, "fetched self consensus state");

        Ok(SelfConsensusState { height, state })
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface, %proof))]
    pub async fn encode_proof(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        proof: Value,
    ) -> RpcResult<Vec<u8>> {
        debug!("encoding proof");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?;

        let proof = client_module
            .encode_proof(proof)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(%proof, "encoded proof");

        Ok(proof.0)
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface))]
    pub async fn decode_client_state_meta(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        client_state: Vec<u8>,
    ) -> RpcResult<ClientStateMeta> {
        debug!("decoding client state meta");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?;

        let meta = client_module
            .decode_client_state_meta(Hex(client_state))
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(
            height = %meta.height,
            chain_id = %meta.chain_id,
            "decoded client state meta"
        );

        Ok(meta)
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface))]
    pub async fn decode_client_state(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        client_state: Vec<u8>,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?
            .decode_client_state(Hex(client_state))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface))]
    pub async fn decode_consensus_state(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        consensus_state: Vec<u8>,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?
            .decode_consensus_state(Hex(consensus_state))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn query_ibc_state_typed<
        P: IbcPath<Value: DeserializeOwned> + Serialize + Valuable,
    >(
        &self,
        chain_id: &ChainId<'_>,
        at: Height,
        path: P,
    ) -> Result<IbcState<P::Value, P>, jsonrpsee::core::client::Error> {
        debug!(%chain_id, path = path.as_value(), %at, "querying ibc state");

        let ibc_state = self
            .query_ibc_state(chain_id, at, path.clone().into())
            .await?;

        Ok(serde_json::from_value::<P::Value>(ibc_state.state.clone())
            .map(|value| IbcState {
                chain_id: ibc_state.chain_id,
                path: path.clone(),
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

/// rpc impl
#[async_trait]
impl VoyagerRpcServer for Server {
    async fn info(&self) -> RpcResult<LoadedModulesInfo> {
        Ok(self.modules()?.info())
    }

    async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height> {
        self.query_latest_height(&chain_id).await
    }

    async fn query_latest_timestamp(&self, chain_id: ChainId<'static>) -> RpcResult<i64> {
        self.query_latest_timestamp(&chain_id).await
    }

    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo> {
        self.client_info(&chain_id, client_id).await
    }

    async fn client_meta(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.client_meta(chain_id, at, client_id).await
    }

    async fn query_ibc_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcState> {
        let height = self.query_height(&chain_id, height).await?;

        self.query_ibc_state(&chain_id, height, path).await
    }

    async fn query_ibc_proof(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcProof> {
        let height = self.query_height(&chain_id, height).await?;

        self.query_ibc_proof(&chain_id, height, path).await
    }

    async fn self_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfClientState> {
        let height = self.query_height(&chain_id, height).await?;

        self.self_client_state(chain_id, height).await
    }

    async fn self_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState> {
        self.self_consensus_state(chain_id, height).await
    }

    // TODO: Use valuable here
    async fn encode_proof(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        proof: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        self.encode_proof(&client_type, &ibc_interface, proof)
            .await
            .map(Hex)
    }

    // TODO: Use valuable here
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        self.decode_client_state_meta(&client_type, &ibc_interface, client_state.0)
            .await
    }

    async fn decode_client_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        self.decode_client_state(&client_type, &ibc_interface, client_state.0)
            .await
    }

    async fn decode_consensus_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        self.decode_consensus_state(&client_type, &ibc_interface, consensus_state.0)
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
