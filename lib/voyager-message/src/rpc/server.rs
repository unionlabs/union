use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use tracing::{debug, instrument, trace};
use unionlabs::{bytes::Bytes, ibc::core::client::height::Height, ErrorReporter};
use voyager_core::IbcSpecId;

// use valuable::Valuable;
// use voyager_core::IbcStoreFormat;
use crate::{
    context::{LoadedModulesInfo, Modules},
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
}

#[derive(Debug, Clone)]
pub struct ServerInner {
    modules: OnceLock<Arc<Modules>>,
    // ibc_state_cache: Cache,
}

// #[derive(Clone)]
// struct Cache(moka::future::Cache<StateQuery, Value>);

// impl Debug for Cache {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Cache({:?}, {})", self.0.name(), self.0.entry_count())
//     }
// }

impl Server {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Server {
            inner: Arc::new(ServerInner {
                modules: OnceLock::new(),
                // ibc_state_cache: Cache(
                //     moka::future::Cache::builder()
                //         .eviction_listener(|k, v, why| {
                //             error!(?k, ?v, ?why, "value evicted from the cache")
                //         })
                //         .max_capacity(10_000)
                //         .name("ibc_state_cache")
                //         .build(),
                // ),
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
    pub async fn query_height(&self, chain_id: &ChainId, height: QueryHeight) -> RpcResult<Height> {
        match height {
            QueryHeight::Latest => {
                let latest_height = self
                    .modules()?
                    .consensus_module(chain_id)
                    .map_err(fatal_error)?
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
        trace!("querying latest height");

        let latest_height = self
            .inner
            .modules()?
            .consensus_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_height(finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(
            %latest_height,
            "queried latest height"
        );

        Ok(latest_height)
    }

    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_timestamp(
        &self,
        chain_id: &ChainId,
        finalized: bool,
    ) -> RpcResult<i64> {
        trace!("querying latest timestamp");

        let latest_timestamp = self
            .inner
            .modules()?
            .consensus_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_timestamp(finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(latest_timestamp, "queried latest timestamp");

        Ok(latest_timestamp)
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, client_id = %client_id.0))]
    pub async fn client_info(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<ClientInfo> {
        trace!("fetching client info");

        let client_info = self
            .inner
            .modules()?
            .state_module(chain_id, ibc_spec_id)
            .map_err(fatal_error)?
            .client_info_raw(client_id.clone())
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(
            %client_info.ibc_interface,
            %client_info.client_type,
            "fetched client info"
        );

        Ok(client_info)
    }

    #[instrument(skip_all, fields(%chain_id, %ibc_spec_id, height = %at, client_id = %client_id.0))]
    pub async fn client_meta(
        &self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta> {
        trace!("fetching client meta");

        let height = self.query_height(chain_id, at).await?;

        let modules = self.inner.modules()?;

        let state_module = modules.state_module(chain_id, ibc_spec_id)?;

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
            .decode_client_state_meta(client_state.as_str().unwrap().parse().unwrap())
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(
            client_state_meta.height = %meta.height,
            client_state_meta.chain_id = %meta.chain_id,
            %client_info.ibc_interface,
            %client_info.client_type,
            "fetched client meta"
        );

        Ok(meta)
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcState<P::Value>> {
        trace!("fetching ibc state");

        let state_module = self
            .inner
            .modules()?
            .state_module(chain_id, &P::Spec::ID)
            .map_err(fatal_error)?;

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
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn query_ibc_proof<P: IbcStorePathKey>(
        &self,
        chain_id: &ChainId,
        height: Height,
        path: <P::Spec as IbcSpec>::StorePath,
    ) -> RpcResult<IbcProof> {
        trace!("fetching ibc state");

        let proof_module = self
            .inner
            .modules()?
            .proof_module(chain_id, &P::Spec::ID)
            .map_err(fatal_error)?;

        let proof = proof_module
            .query_ibc_proof_raw(height, into_value(path.clone()))
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        trace!(%proof, "fetched ibc proof");

        Ok(IbcProof { height, proof })
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        height: Height,
    ) -> RpcResult<SelfClientState> {
        trace!("querying self client state");

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
        trace!(%state, "fetched self client state");

        Ok(SelfClientState { height, state })
    }

    #[instrument(skip_all, fields(%chain_id, %height))]
    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState> {
        trace!("querying self consensus state");

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
        trace!(%state, "fetched self consensus state");

        Ok(SelfConsensusState { height, state })
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
        trace!("encoding proof");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface, ibc_spec_id)
            .map_err(fatal_error)?;

        let proof = client_module
            .encode_proof(proof)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(%proof, "encoded proof");

        Ok(proof)
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
        trace!("decoding client state meta");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface, ibc_spec_id)
            .map_err(fatal_error)?;

        let meta = client_module
            .decode_client_state_meta(client_state)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        trace!(
            height = %meta.height,
            chain_id = %meta.chain_id,
            "decoded client state meta"
        );

        Ok(meta)
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_client_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface, ibc_spec_id)
            .map_err(fatal_error)?
            .decode_client_state(client_state)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface, %ibc_spec_id))]
    pub async fn decode_consensus_state(
        &self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface, ibc_spec_id)
            .map_err(fatal_error)?
            .decode_consensus_state(consensus_state)
            .await
            .map_err(json_rpc_error_to_error_object)
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

    // async fn query_client_state(
    //     &self,
    //     chain_id: ChainId,
    //     height: QueryHeight,
    //     client_id: ClientId,
    // ) -> RpcResult<IbcState<Bytes>> {
    //     let height = self.query_height(&chain_id, height).await?;

    //     self.modules()?
    //         .chain_module(&chain_id)?
    //         .query_client_state(height, client_id)
    //         .await
    //         .map(|state| IbcState {
    //             chain_id,
    //             height,
    //             state,
    //         })
    //         .map_err(json_rpc_error_to_error_object)
    // }

    // async fn query_client_consensus_state(
    //     &self,
    //     chain_id: ChainId,
    //     height: QueryHeight,
    //     client_id: ClientId,
    //     trusted_height: Height,
    // ) -> RpcResult<IbcState<Bytes>> {
    //     let height = self.query_height(&chain_id, height).await?;

    //     self.modules()?
    //         .chain_module(&chain_id)?
    //         .query_client_consensus_state(height, client_id, trusted_height)
    //         .await
    //         .map(|state| IbcState {
    //             chain_id,
    //             height,
    //             state,
    //         })
    //         .map_err(json_rpc_error_to_error_object)
    // }

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
            .map_err(fatal_error)?;

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

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// struct StateQuery {
//     chain_id: ChainId,
//     height: Height,
//     kind: StateQueryKind,
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// enum StateQueryKind {
//     ClientState {
//         client_id: ClientId,
//     },

//     ClientConsensusState {
//         client_id: ClientId,
//         trusted_height: Height,
//     },

//     Connection {
//         connection_id: ConnectionId,
//     },

//     ChannelEnd {
//         channel_id: ChannelId,
//     },

//     Commitment {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     Acknowledgement {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     Receipt {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     NextSequenceSend {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     NextSequenceRecv {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     NextSequenceAck {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     NextConnectionSequence {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },

//     NextClientSequence {
//         channel_id: ChannelId,
//         sequence: NonZeroU64,
//     },
// }
