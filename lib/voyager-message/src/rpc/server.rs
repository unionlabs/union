use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use tracing::{debug, info_span, instrument, trace};
use unionlabs::{bytes::Bytes, ibc::core::client::height::Height, ErrorReporter};
use voyager_core::IbcSpecId;
use voyager_vm::ItemId;

// use valuable::Valuable;
// use voyager_core::IbcStoreFormat;
use crate::{
    context::{LoadedModulesInfo, Modules, WithId},
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
    into_value,
    module::{
        ClientModuleClient, ConsensusModuleClient, RawProofModuleClient, RawStateModuleClient,
    },
    rpc::{
        json_rpc_error_to_error_object, IbcProof, IbcState, SelfClientState, SelfConsensusState,
        VoyagerRpcServer,
    },
    IbcSpec, IbcStorePathKey, RawClientId, FATAL_JSONRPC_ERROR_CODE,
};

#[derive(Debug, Clone)]
pub struct Server {
    inner: Arc<ServerInner>,
    item_id: Option<ItemId>,
}

#[derive(Debug, Clone)]
pub struct ServerInner {
    modules: OnceLock<Arc<Modules>>,
}

impl Server {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Server {
            inner: Arc::new(ServerInner {
                modules: OnceLock::new(),
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
            Some(item_id) => info_span!("item", item_id = item_id.raw()),
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
                    .consensus_module(chain_id)
                    .map_err(fatal_error)?
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
                    .consensus_module(chain_id)
                    .map_err(fatal_error)?
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
                    .consensus_module(chain_id)
                    .map_err(fatal_error)?
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
    ) -> RpcResult<i64> {
        self.span()
            .in_scope(|| async {
                trace!("querying latest timestamp");

                let latest_timestamp = self
                    .inner
                    .modules()?
                    .consensus_module(chain_id)
                    .map_err(fatal_error)?
                    .with_id(self.item_id)
                    .query_latest_timestamp(finalized)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(latest_timestamp, "queried latest timestamp");

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
    ) -> RpcResult<ClientInfo> {
        self.span()
            .in_scope(|| async {
                trace!("fetching client info");

                let client_info = self
                    .inner
                    .modules()?
                    .state_module(chain_id, ibc_spec_id)
                    .map_err(fatal_error)?
                    .with_id(self.item_id)
                    .client_info_raw(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched client info"
                );

                Ok(client_info)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.0))]
    pub async fn client_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.span()
            .in_scope(|| async {
                trace!("fetching client meta");

                let height = self.query_height(chain_id, at).await?;

                let modules = self.inner.modules()?;

                let state_module = modules
                    .state_module(chain_id, ibc_spec_id)?
                    .with_id(self.item_id);

                let client_info = state_module
                    .client_info_raw(client_id.clone())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                let client_state = state_module
                    .query_ibc_state_raw(
                        height,
                        (self
                            .modules()?
                            .ibc_spec_handlers
                            .handlers
                            .get(ibc_spec_id)
                            .unwrap()
                            .client_state_path)(client_id.clone())
                        .unwrap(),
                    )
                    .await
                    .map_err(fatal_error)?;

                trace!(%client_state);

                let meta = modules
                    .client_module(
                        &client_info.client_type,
                        &client_info.ibc_interface,
                        ibc_spec_id,
                    )
                    .map_err(fatal_error)?
                    .with_id(self.item_id)
                    .decode_client_state_meta(client_state.as_str().unwrap().parse().unwrap())
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    client_state_meta.height = %meta.counterparty_height,
                    client_state_meta.chain_id = %meta.chain_id,
                    %client_info.ibc_interface,
                    %client_info.client_type,
                    "fetched client meta"
                );

                Ok(meta)
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
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
                    .state_module(chain_id, &P::Spec::ID)
                    .map_err(fatal_error)?
                    .with_id(self.item_id);

                let state = state_module
                    .query_ibc_state_raw(height, into_value(path.clone()))
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched ibc state");

                Ok(IbcState {
                    height,
                    state: serde_json::from_value(state).unwrap(),
                })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
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
                    .proof_module(chain_id, &P::Spec::ID)
                    .map_err(fatal_error)?
                    .with_id(self.item_id);

                let proof = proof_module
                    .query_ibc_proof_raw(height, into_value(path.clone()))
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%proof, "fetched ibc proof");

                Ok(IbcProof { height, proof })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        height: Height,
    ) -> RpcResult<SelfClientState> {
        self.span()
            .in_scope(|| async {
                trace!("querying self client state");

                let chain_module = self
                    .inner
                    .modules()?
                    .consensus_module(&chain_id)
                    .map_err(fatal_error)?
                    .with_id(self.item_id);

                let state = chain_module
                    .self_client_state(height)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                // TODO: Use valuable here
                trace!(%state, "fetched self client state");

                Ok(SelfClientState { height, state })
            })
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState> {
        self.span()
            .in_scope(|| async {
                trace!("querying self consensus state");

                let chain_module = self
                    .inner
                    .modules()?
                    .consensus_module(&chain_id)
                    .map_err(fatal_error)?
                    .with_id(self.item_id);

                let height = self.query_height(&chain_id, height).await?;

                let state = chain_module
                    .self_consensus_state(height)
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
                    .client_module(client_type, ibc_interface, ibc_spec_id)
                    .map_err(fatal_error)?
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
                    .client_module(client_type, ibc_interface, ibc_spec_id)
                    .map_err(fatal_error)?
                    .with_id(self.item_id);

                let meta = client_module
                    .decode_client_state_meta(client_state)
                    .await
                    .map_err(json_rpc_error_to_error_object)?;

                trace!(
                    height = %meta.counterparty_height,
                    chain_id = %meta.chain_id,
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
                    .client_module(client_type, ibc_interface, ibc_spec_id)
                    .map_err(fatal_error)?
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
                    .client_module(client_type, ibc_interface, ibc_spec_id)
                    .map_err(fatal_error)?
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
    async fn info(&self) -> RpcResult<LoadedModulesInfo> {
        Ok(self.modules()?.info())
    }

    // =========
    // CONSENSUS
    // =========

    async fn query_latest_height(&self, chain_id: ChainId, finalized: bool) -> RpcResult<Height> {
        self.query_latest_height(&chain_id, finalized).await
    }

    async fn query_latest_timestamp(&self, chain_id: ChainId, finalized: bool) -> RpcResult<i64> {
        self.query_latest_timestamp(&chain_id, finalized).await
    }

    // =====
    // STATE
    // =====

    async fn client_info(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<ClientInfo> {
        self.client_info(&chain_id, &ibc_spec_id, client_id).await
    }

    async fn client_meta(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.client_meta(&chain_id, &ibc_spec_id, at, client_id)
            .await
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcState<Value>> {
        let height = self.query_height(&chain_id, height).await?;

        debug!("fetching ibc state");

        let state_module = self
            .inner
            .modules()?
            .state_module(&chain_id, &ibc_spec_id)
            .map_err(fatal_error)?
            .with_id(self.item_id);

        let state = state_module
            .query_ibc_state_raw(height, path)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%state, "fetched ibc state");

        Ok(IbcState { height, state })
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProof> {
        let height = self.query_height(&chain_id, height).await?;

        debug!("fetching ibc proof");

        let proof_module = self
            .inner
            .modules()?
            .proof_module(&chain_id, &ibc_spec_id)
            .map_err(fatal_error)?;

        let proof = proof_module
            .query_ibc_proof_raw(height, path)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%proof, "fetched ibc proof");

        Ok(IbcProof { height, proof })
    }

    async fn self_client_state(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
    ) -> RpcResult<SelfClientState> {
        let height = self.query_height(&chain_id, height).await?;

        self.self_client_state(chain_id, height).await
    }

    async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState> {
        self.self_consensus_state(chain_id, height).await
    }

    // TODO: Use valuable here
    async fn encode_proof(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes> {
        self.encode_proof(&client_type, &ibc_interface, &ibc_spec_id, proof)
            .await
    }

    // TODO: Use valuable here
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.decode_client_state_meta(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_client_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.decode_client_state(&client_type, &ibc_interface, &ibc_spec_id, client_state)
            .await
    }

    async fn decode_consensus_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.decode_consensus_state(&client_type, &ibc_interface, &ibc_spec_id, consensus_state)
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
