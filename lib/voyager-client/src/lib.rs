use std::{fmt::Debug, future::Future};

use jsonrpsee::{
    core::{
        client::{BatchResponse, ClientT},
        params::{ArrayParams, BatchRequestBuilder},
        traits::ToRpcParams,
        RpcResult,
    },
    types::ErrorObject,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tracing::instrument;
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes, ErrorReporter};
use voyager_primitives::{
    ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface, IbcQuery,
    IbcSpec, IbcSpecId, IbcStorePathKey, QueryHeight, Timestamp,
};
use voyager_rpc::{
    json_rpc_error_to_error_object,
    types::{
        IbcProofResponse, IbcStateResponse, SelfClientStateResponse, SelfConsensusStateResponse,
    },
    VoyagerRpcClient, FATAL_JSONRPC_ERROR_CODE, MISSING_STATE_ERROR_CODE,
};
use voyager_types::RawClientId;

#[derive(Debug, Clone)]
pub struct VoyagerClient<C: ClientT>(C);

impl<C: VoyagerRpcClient> VoyagerClient<C> {
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C: VoyagerRpcClient> VoyagerClient<C> {
    pub async fn query_latest_timestamp(
        &self,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_timestamp = self
            .0
            .query_latest_timestamp(chain_id, finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(latest_timestamp)
    }

    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfClientStateResponse> {
        let client_state = self
            .0
            .self_client_state(chain_id, client_type, height, config)
            .await
            .map_err(json_rpc_error_to_error_object)?;
        Ok(client_state)
    }

    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusStateResponse> {
        let consensus_state = self
            .0
            .self_consensus_state(chain_id, client_type, height, config)
            .await
            .map_err(json_rpc_error_to_error_object)?;
        Ok(consensus_state)
    }

    pub async fn query_latest_height(
        &self,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        let latest_height = self
            .0
            .query_latest_height(chain_id, finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(latest_height)
    }

    #[instrument(
        skip_all,
        name = "voyager_client_encode_proof",
        fields(
            %client_type,
            %ibc_interface,
            %proof
        )
    )]
    pub async fn encode_proof<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        proof: Value,
    ) -> RpcResult<Bytes> {
        let proof = self
            .0
            .encode_proof(client_type, ibc_interface, V::ID, proof)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(proof)
    }

    #[instrument(
        skip_all,
        name = "voyager_client_encode_header",
        fields(
            %client_type,
            %ibc_interface,
            %header
        )
    )]
    pub async fn encode_header<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        header: Value,
    ) -> RpcResult<Bytes> {
        let header = self
            .0
            .encode_header(client_type, ibc_interface, V::ID, header)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(header)
    }

    pub async fn decode_client_state<V: IbcSpec, T: DeserializeOwned>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        client_state_bytes: Bytes,
    ) -> RpcResult<T> {
        let client_state = self
            .0
            .decode_client_state(client_type, ibc_interface, V::ID, client_state_bytes)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        serde_json::from_value(client_state).map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message("error decoding client state from json value"),
                None::<()>,
            )
        })
    }

    pub async fn decode_consensus_state<V: IbcSpec, T: DeserializeOwned>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        consensus_state_bytes: Bytes,
    ) -> RpcResult<T> {
        let consensus_state = self
            .0
            .decode_consensus_state(client_type, ibc_interface, V::ID, consensus_state_bytes)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        serde_json::from_value(consensus_state).map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message("error decoding consensus state from json value"),
                None::<()>,
            )
        })
    }

    pub async fn encode_client_state<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        self.0
            .encode_client_state(client_type, ibc_interface, V::ID, client_state, metadata)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn encode_consensus_state<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        self.0
            .encode_consensus_state(client_type, ibc_interface, V::ID, consensus_state)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn query<Q: IbcQuery>(&self, chain_id: ChainId, query: Q) -> RpcResult<Q::Response> {
        self.0
            .query(
                chain_id.clone(),
                <Q::Spec as IbcSpec>::ID,
                serde_json::to_value(<Q::Spec as IbcSpec>::Query::from(query.clone().into()))
                    .unwrap(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(|e| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("error decoding query return value: {}", ErrorReporter(e)),
                        Some(json!({
                            "query": query
                        })),
                    )
                })
            })
    }

    pub async fn query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
        path: P,
    ) -> RpcResult<P::Value> {
        let state = self
            .maybe_query_ibc_state(chain_id.clone(), height, path.clone())
            .await?
            .state
            .ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "state must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": height,
                        "path": <P::Spec as IbcSpec>::StorePath::from(path.into())
                    })),
                )
            })?;

        Ok(state)
    }

    pub async fn maybe_query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
        path: P,
    ) -> RpcResult<IbcStateResponse<P::Value>> {
        let ibc_state = self
            .0
            .query_ibc_state(
                chain_id,
                P::Spec::ID,
                height,
                serde_json::to_value(<P::Spec as IbcSpec>::StorePath::from(path.into())).unwrap(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(IbcStateResponse {
            height: ibc_state.height,
            state: ibc_state
                .state
                .map(|state| {
                    serde_json::from_value(state.clone()).map_err(|e| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("error decoding IBC state: {}", ErrorReporter(e)),
                            Some(json!({
                                "raw_state": state
                            })),
                        )
                    })
                })
                .transpose()?,
        })
    }

    pub async fn query_ibc_proof<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
        path: P,
    ) -> RpcResult<IbcProofResponse> {
        let ibc_proof = self
            .0
            .query_ibc_proof(
                chain_id,
                P::Spec::ID,
                height,
                serde_json::to_value(<P::Spec as IbcSpec>::StorePath::from(path.into())).unwrap(),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(ibc_proof)
    }

    pub async fn equivalent_chain_ids(&self, chain_id: ChainId) -> RpcResult<Vec<ChainId>> {
        self.0
            .equivalent_chain_ids(chain_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_info<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        client_id: V::ClientId,
    ) -> RpcResult<ClientInfo> {
        self.maybe_client_info::<V>(chain_id.clone(), client_id.clone())
            .await
            .and_then(|client_info| {
                client_info.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client info must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_info<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        client_id: V::ClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.0
            .client_info(chain_id, V::ID, RawClientId::new(client_id))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_info_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<ClientInfo> {
        self.maybe_client_info_raw(chain_id.clone(), ibc_spec_id, client_id.clone())
            .await
            .and_then(|client_info| {
                client_info.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client info must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_info_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.0
            .client_info(chain_id, ibc_spec_id, client_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.maybe_client_state_meta::<V>(chain_id.clone(), at, client_id.clone())
            .await
            .and_then(|client_state_meta| {
                client_state_meta.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client state meta must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "height": at,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.0
            .client_state_meta(chain_id, V::ID, at, RawClientId::new(client_id))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.maybe_client_state_meta_raw(chain_id.clone(), ibc_spec_id, at, client_id.clone())
            .await
            .and_then(|client_state_meta| {
                client_state_meta.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client state meta must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "height": at,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.0
            .client_state_meta(chain_id, ibc_spec_id, at, client_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn consensus_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
        counterparty_height: Height,
    ) -> RpcResult<ConsensusStateMeta> {
        self.maybe_consensus_state_meta::<V>(
            chain_id.clone(),
            at,
            client_id.clone(),
            counterparty_height,
        )
        .await
        .and_then(|consensus_state_meta| {
            consensus_state_meta.ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "consensus state meta must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": at,
                        "client_id": client_id,
                        "counterparty_height": counterparty_height
                    })),
                )
            })
        })
    }

    pub async fn maybe_consensus_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.0
            .consensus_state_meta(
                chain_id,
                V::ID,
                at,
                RawClientId::new(client_id),
                counterparty_height,
            )
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn consensus_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<ConsensusStateMeta> {
        self.maybe_consensus_state_meta_raw(
            chain_id.clone(),
            ibc_spec_id,
            at,
            client_id.clone(),
            counterparty_height,
        )
        .await
        .and_then(|consensus_state_meta| {
            consensus_state_meta.ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "consensus state meta must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": at,
                        "client_id": client_id,
                        "counterparty_height": counterparty_height
                    })),
                )
            })
        })
    }

    pub async fn maybe_consensus_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.0
            .consensus_state_meta(chain_id, ibc_spec_id, at, client_id, counterparty_height)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub fn plugin_client(&self, plugin: impl Into<String>) -> VoyagerPluginClient<'_, C> {
        VoyagerPluginClient {
            inner: self,
            plugin: plugin.into(),
        }
    }
}

pub struct VoyagerPluginClient<'a, C: ClientT> {
    inner: &'a VoyagerClient<C>,
    plugin: String,
}

#[allow(clippy::manual_async_fn)] // false positive? fails with a Send bound error if i do it with the async syntax
impl<C: ClientT> ClientT for VoyagerPluginClient<'_, C> {
    fn notification<Params>(
        &self,
        _method: &str,
        _params: Params,
    ) -> impl Future<Output = Result<(), jsonrpsee::core::client::Error>> + Send
    where
        Params: ToRpcParams + Send,
    {
        async {
            Err(jsonrpsee::core::client::Error::Custom(
                "notifications are not supported for plugin calls".to_owned(),
            ))
        }
    }

    fn request<R, Params>(
        &self,
        method: &str,
        params: Params,
    ) -> impl Future<Output = Result<R, jsonrpsee::core::client::Error>> + Send
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        let mut p = ArrayParams::new();

        p.insert(&self.plugin)
            .expect("serializaiton is infallible; qed;");
        p.insert(method).expect("serializaiton is infallible; qed;");

        if let Some(params) = params
            .to_rpc_params()
            .expect("serialization is infallible; qed;")
        {
            p.insert(params).expect("serializaiton is infallible; qed;");
        } else {
            // just need an empty array
            p.insert([0u8; 0])
                .expect("serializaiton is infallible; qed;");
        };

        self.inner.0.request("voyager_pluginCustom", p)
    }

    fn batch_request<'a, R>(
        &self,
        _batch: BatchRequestBuilder<'a>,
    ) -> impl Future<Output = Result<BatchResponse<'a, R>, jsonrpsee::core::client::Error>> + Send
    where
        R: DeserializeOwned + Debug + 'a,
    {
        async {
            Err(jsonrpsee::core::client::Error::Custom(
                "batch requests are not supported for plugin calls".to_owned(),
            ))
        }
    }
}
