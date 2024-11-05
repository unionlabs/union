use std::{
    fmt::Debug,
    num::NonZeroU64,
    sync::{Arc, OnceLock},
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    bytes::Bytes,
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    ics24::Path,
    id::{ChannelId, ClientId, ConnectionId, PortId},
    ErrorReporter,
};
use voyager_core::IbcVersion;

// use valuable::Valuable;
// use voyager_core::IbcStoreFormat;
use crate::{
    context::{LoadedModulesInfo, Modules},
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
    module::{ChainModuleClient, ClientModuleClient, ConsensusModuleClient},
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
    pub async fn query_height(
        &self,
        chain_id: &ChainId<'_>,
        height: QueryHeight,
    ) -> RpcResult<Height> {
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

    // async fn query_state_cached<'a, T: DeserializeOwned>(
    //     &self,
    //     chain_id: &ChainId<'a>,
    //     at: Height,
    //     query: StateQuery,
    // ) -> Result<IbcState<T>, jsonrpsee::core::client::Error> {
    //     let fut = async {
    //         debug!(%chain_id, path = query.as_value(), %at, "querying ibc state (not yet cached)");

    //         let chain_module = self
    //             .modules()?
    //             .chain_module(chain_id)
    //             .map_err(fatal_error)?;

    //         match query.kind {
    //             StateQueryKind::ClientState { client_id } => todo!(),
    //             StateQueryKind::ClientConsensusState {
    //                 client_id,
    //                 trusted_height,
    //             } => todo!(),
    //             StateQueryKind::Connection { connection_id } => todo!(),
    //             StateQueryKind::ChannelEnd { channel_id } => todo!(),
    //             StateQueryKind::Commitment {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::Acknowledgement {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::Receipt {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::NextSequenceSend {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::NextSequenceRecv {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::NextSequenceAck {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::NextConnectionSequence {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //             StateQueryKind::NextClientSequence {
    //                 channel_id,
    //                 sequence,
    //             } => todo!(),
    //         }

    //         RpcResult::Ok(state)
    //     }
    //     .instrument(info_span!("ibc state cache fetcher"));

    //     let state = self
    //         .ibc_state_cache
    //         .0
    //         .entry_by_ref(&query)
    //         .or_try_insert_with(fut)
    //         .await
    //         .map_err(Arc::unwrap_or_clone)?;

    //     let (chain_id, path, height) = key;

    //     if !state.is_fresh() {
    //         debug!(
    //             cache = %self.ibc_state_cache.0.name().unwrap_or_default(),
    //             key.chain_id = %chain_id,
    //             key.path = path.as_value(),
    //             key.height = %height,
    //             // raw_key = %format_args!("chain_id:{chain_id};path:{path};height:{height}"),
    //             "cache hit"
    //         );
    //     }

    //     Ok(IbcState {
    //         chain_id,
    //         path,
    //         height,
    //         state: state.into_value(),
    //     })

    //     // Ok(
    //     //     serde_json::from_value::<P::Value>(state.clone()).map_err(|e| {
    //     //         ErrorObject::owned(
    //     //             FATAL_JSONRPC_ERROR_CODE,
    //     //             format!("unable to deserialize state: {}",
    //     // ErrorReporter(e)),             Some(json!({
    //     //                 "path": path,
    //     //                 "state": state
    //     //             })),
    //     //         )
    //     //     })?,
    //     // )
    // }
}

impl Server {
    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_height(
        &self,
        chain_id: &ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<Height> {
        debug!("querying latest height");

        let latest_height = self
            .inner
            .modules()?
            .consensus_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_height(finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(
            %latest_height,
            "queried latest height"
        );

        Ok(latest_height)
    }

    #[instrument(skip_all, fields(%chain_id, finalized))]
    pub async fn query_latest_timestamp(
        &self,
        chain_id: &ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<i64> {
        debug!("querying latest timestamp");

        let latest_timestamp = self
            .inner
            .modules()?
            .consensus_module(chain_id)
            .map_err(fatal_error)?
            .query_latest_timestamp(finalized)
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

    #[instrument(skip_all, fields(%chain_id, height = %at, %client_id))]
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
            .query_client_state(height, client_id)
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
            client_state_meta.height = %meta.height,
            client_state_meta.chain_id = %meta.chain_id,
            %client_info.ibc_interface,
            %client_info.client_type,
            "fetched client meta"
        );

        Ok(meta)
    }

    #[instrument(skip_all, fields(%chain_id, %path, %height))]
    pub async fn query_ibc_proof(
        &self,
        chain_id: &ChainId<'_>,
        height: Height,
        path: Bytes,
        ibc_version: IbcVersion,
    ) -> RpcResult<IbcProof> {
        debug!("fetching ibc state");

        let chain_module = self
            .inner
            .modules()?
            .chain_module(chain_id)
            .map_err(fatal_error)?;

        // let height = self.inner.query_height(&chain_id, height).await?;

        let proof = chain_module
            .query_ibc_proof(height, path.clone(), ibc_version)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        // TODO: Use valuable here
        debug!(%proof, "fetched ibc proof");

        Ok(IbcProof {
            path,
            ibc_version,
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
        ibc_version: IbcVersion,
        proof: Value,
    ) -> RpcResult<Bytes> {
        debug!("encoding proof");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface, ibc_version)
            .map_err(fatal_error)?;

        let proof = client_module
            .encode_proof(proof)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        debug!(%proof, "encoded proof");

        Ok(proof)
    }

    // TODO: Use valuable here
    #[instrument(skip_all, fields(%client_type, %ibc_interface))]
    pub async fn decode_client_state_meta(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        debug!("decoding client state meta");

        let client_module = self
            .inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?;

        let meta = client_module
            .decode_client_state_meta(client_state)
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
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?
            .decode_client_state(client_state)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    #[instrument(skip_all, fields(%client_type, %ibc_interface))]
    pub async fn decode_consensus_state(
        &self,
        client_type: &ClientType<'static>,
        ibc_interface: &IbcInterface<'static>,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.inner
            .modules()?
            .client_module(client_type, ibc_interface)
            .map_err(fatal_error)?
            .decode_consensus_state(consensus_state)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    // pub async fn query_ibc_state_typed<
    //     P: IbcPath<Value: DeserializeOwned> + Serialize + Valuable,
    // >(
    //     &self,
    //     chain_id: &ChainId<'_>,
    //     at: Height,
    //     path: P,
    // ) -> Result<IbcState<P::Value, P>, jsonrpsee::core::client::Error> {
    //     debug!(%chain_id, path = path.as_value(), %at, "querying ibc state");

    //     let ibc_state = self.query_state(chain_id, at, path.clone().into()).await?;

    //     Ok(serde_json::from_value::<P::Value>(ibc_state.state.clone())
    //         .map(|value| IbcState {
    //             chain_id: ibc_state.chain_id,
    //             path: path.clone(),
    //             height: ibc_state.height,
    //             state: value,
    //         })
    //         .map_err(|e| {
    //             ErrorObject::owned(
    //                 FATAL_JSONRPC_ERROR_CODE,
    //                 format!("unable to deserialize state: {}", ErrorReporter(e)),
    //                 Some(json!({
    //                     "chain_id": chain_id,
    //                     "path": path,
    //                     "state": ibc_state.state
    //                 })),
    //             )
    //         })?)
    // }
}

/// rpc impl
#[async_trait]
impl VoyagerRpcServer for Server {
    async fn info(&self) -> RpcResult<LoadedModulesInfo> {
        Ok(self.modules()?.info())
    }

    async fn query_latest_height(
        &self,
        chain_id: ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<Height> {
        self.query_latest_height(&chain_id, finalized).await
    }

    async fn query_latest_timestamp(
        &self,
        chain_id: ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<i64> {
        self.query_latest_timestamp(&chain_id, finalized).await
    }

    async fn query_client_prefix(
        &self,
        chain_id: ChainId<'static>,
        raw_client_id: u32,
    ) -> RpcResult<String> {
        self.modules()?
            .chain_module(&chain_id)?
            .query_client_prefix(raw_client_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo> {
        self.client_info(&chain_id, client_id).await
    }

    // #[instrument(skip_all, fields(client_id))]
    // async fn client_type_ibc_store_format(
    //     &self,
    //     client_type: ClientType<'static>,
    // ) -> RpcResult<IbcStoreFormat<'static>> {
    //     Ok(self
    //         .modules()?
    //         .client_ibc_store_format(&client_type)?
    //         .clone())
    // }

    async fn client_meta(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.client_meta(chain_id, at, client_id).await
    }

    async fn query_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<IbcState<Bytes>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_client_state(height, client_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_client_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<IbcState<Bytes>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_client_consensus_state(height, client_id, trusted_height)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_connection(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        connection_id: ConnectionId,
    ) -> RpcResult<IbcState<Option<ConnectionEnd>>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_connection(height, connection_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_channel(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<Option<Channel>>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_channel(height, port_id, channel_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_commitment(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_commitment(height, port_id, channel_id, sequence)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_acknowledgement(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_acknowledgement(height, port_id, channel_id, sequence)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_receipt(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<bool>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_receipt(height, port_id, channel_id, sequence)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_next_sequence_send(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_next_sequence_send(height, port_id, channel_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_next_sequence_recv(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_next_sequence_recv(height, port_id, channel_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_next_sequence_ack(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_next_sequence_ack(height, port_id, channel_id)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_next_connection_sequence(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<IbcState<u64>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_next_connection_sequence(height)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_next_client_sequence(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<IbcState<u64>> {
        let height = self.query_height(&chain_id, height).await?;

        self.modules()?
            .chain_module(&chain_id)?
            .query_next_client_sequence(height)
            .await
            .map(|state| IbcState {
                chain_id,
                height,
                state,
            })
            .map_err(json_rpc_error_to_error_object)
    }

    async fn query_ibc_proof(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
        // ibc_store_format: IbcStoreFormat<'static>,
    ) -> RpcResult<IbcProof> {
        let height = self.query_height(&chain_id, height).await?;

        self.query_ibc_proof(
            &chain_id, height, path, // ibc_store_format
        )
        .await
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
    ) -> RpcResult<Bytes> {
        self.encode_proof(&client_type, &ibc_interface, proof).await
    }

    // TODO: Use valuable here
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        self.decode_client_state_meta(&client_type, &ibc_interface, client_state)
            .await
    }

    async fn decode_client_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Bytes,
    ) -> RpcResult<Value> {
        self.decode_client_state(&client_type, &ibc_interface, client_state)
            .await
    }

    async fn decode_consensus_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        self.decode_consensus_state(&client_type, &ibc_interface, consensus_state)
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
//     chain_id: ChainId<'static>,
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
