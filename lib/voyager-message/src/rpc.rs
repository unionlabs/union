use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use unionlabs::{bytes::Bytes, ibc::core::client::height::Height, ErrorReporter};
use voyager_core::IbcSpecId;

use crate::{
    context::LoadedModulesInfo,
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
    RawClientId, FATAL_JSONRPC_ERROR_CODE,
};

pub mod server;

#[rpc(
    client,
    server,
    client_bounds(Self: Send + Sync),
    server_bounds(Self:),
    namespace = "voyager",
)]
// TODO: Ensure that height is always the last parameter for consistency
pub trait VoyagerRpc {
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<LoadedModulesInfo>;

    // =========
    // consensus
    // =========

    #[method(name = "queryLatestHeight")]
    async fn query_latest_height(&self, chain_id: ChainId, finalized: bool) -> RpcResult<Height>;

    #[method(name = "queryLatestTimestamp")]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(&self, chain_id: ChainId, finalized: bool) -> RpcResult<i64>;

    // =================
    // IBC state queries
    // =================

    #[method(name = "clientInfo")]
    async fn client_info(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<ClientInfo>;

    #[method(name = "clientMeta")]
    async fn client_meta(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "queryIbcState")]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcState<Value>>;

    #[method(name = "queryIbcProof")]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProof>;

    // ========================================
    // self state queries, for creating clients
    // ========================================

    #[method(name = "selfClientState")]
    async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
    ) -> RpcResult<SelfClientState>;

    #[method(name = "selfConsensusState")]
    async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
    ) -> RpcResult<SelfConsensusState>;

    // ======================
    // state and proof codecs
    // ======================

    #[method(name = "encodeProof")]
    async fn encode_proof(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes>;

    #[method(name = "decodeClientStateMeta")]
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "decodeClientState")]
    async fn decode_client_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value>;

    #[method(name = "decodeConsensusState")]
    async fn decode_consensus_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value>;
}

#[model]
pub struct IbcState<State> {
    /// The height that the state was read at.
    pub height: Height,
    pub state: State,
}

impl IbcState<Value> {
    pub fn decode_state<T: DeserializeOwned>(&self) -> RpcResult<T> {
        serde_json::from_value(self.state.clone()).map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("error decoding IBC state: {}", ErrorReporter(e)),
                Some(json!({
                    "raw_state": self.state
                })),
            )
        })
    }
}

#[model]
pub struct IbcProof {
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

pub fn json_rpc_error_to_error_object(e: jsonrpsee::core::client::Error) -> ErrorObjectOwned {
    match e {
        jsonrpsee::core::client::Error::Call(e) => e,
        jsonrpsee::core::client::Error::ParseError(e) => ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!("parse error: {}", ErrorReporter(e)),
            None::<()>,
        ),
        value => ErrorObject::owned(-1, format!("error: {}", ErrorReporter(value)), None::<()>),
    }
}

/// Some required state was missing (connection/channel end, packet commitment,
/// ..)
pub fn missing_state(
    message: impl Into<String>,
    data: Option<Value>,
) -> impl FnOnce() -> ErrorObjectOwned {
    move || ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, message, data)
}
