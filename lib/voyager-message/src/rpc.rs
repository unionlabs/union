use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes, ErrorReporter};
use voyager_primitives::{ConsensusStateMeta, IbcSpecId, Timestamp};

use crate::{
    context::LoadedModulesInfo,
    primitives::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
    RawClientId, FATAL_JSONRPC_ERROR_CODE, MISSING_STATE_ERROR_CODE,
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
    #[method(name = "info", with_extensions)]
    async fn info(&self) -> RpcResult<LoadedModulesInfo>;

    #[method(name = "equivalentChainIds", with_extensions)]
    async fn equivalent_chain_ids(&self, chain_id: ChainId) -> RpcResult<Vec<ChainId>>;

    // =========
    // consensus
    // =========

    #[method(name = "queryLatestHeight", with_extensions)]
    async fn query_latest_height(&self, chain_id: ChainId, finalized: bool) -> RpcResult<Height>;

    #[method(name = "queryLatestTimestamp", with_extensions)]
    async fn query_latest_timestamp(
        &self,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp>;

    // =================
    // IBC state queries
    // =================

    #[method(name = "clientInfo", with_extensions)]
    async fn client_info(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>>;

    #[method(name = "clientStateMeta", with_extensions)]
    async fn client_state_meta(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>>;

    #[method(name = "consensusStateMeta", with_extensions)]
    async fn consensus_state_meta(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>>;

    #[method(name = "query", with_extensions)]
    async fn query(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        query: Value,
    ) -> RpcResult<Value>;

    #[method(name = "queryIbcState", with_extensions)]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcState<Value>>;

    #[method(name = "queryIbcProof", with_extensions)]
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

    #[method(name = "selfClientState", with_extensions)]
    async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfClientState>;

    #[method(name = "selfConsensusState", with_extensions)]
    async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusState>;

    // ======================
    // state and proof codecs
    // ======================

    #[method(name = "encodeProof", with_extensions)]
    async fn encode_proof(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        proof: Value,
    ) -> RpcResult<Bytes>;

    #[method(name = "encodeHeader", with_extensions)]
    async fn encode_header(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        header: Value,
    ) -> RpcResult<Bytes>;

    #[method(name = "decodeClientStateMeta", with_extensions)]
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "decodeClientState", with_extensions)]
    async fn decode_client_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Bytes,
    ) -> RpcResult<Value>;

    #[method(name = "decodeConsensusState", with_extensions)]
    async fn decode_consensus_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Bytes,
    ) -> RpcResult<Value>;

    // ===================
    // custom plugin calls
    // ===================

    #[method(name = "pluginCustom", with_extensions)]
    async fn plugin_custom(
        &self,
        plugin: String,
        method: String,
        params: Vec<Value>,
    ) -> RpcResult<Value>;
}

#[model]
pub struct IbcState<State> {
    /// The height that the state was read at.
    pub height: Height,
    /// The state from the chain, as read at `height`.
    ///
    /// If the state does not exist on chain at `height`, this will be `None`.
    pub state: Option<State>,
}

impl IbcState<Value> {
    // pub fn decode_state<T: DeserializeOwned>(&self) -> RpcResult<T> {
    //     serde_json::from_value(self.state.clone()).map_err(|e| {
    //         ErrorObject::owned(
    //             FATAL_JSONRPC_ERROR_CODE,
    //             format!("error decoding IBC state: {}", ErrorReporter(e)),
    //             Some(json!({
    //                 "raw_state": self.state
    //             })),
    //         )
    //     })
    // }
}

#[model]
pub struct IbcProof {
    pub proof_type: ProofType,
    /// The height that the proof was read at.
    pub height: Height,
    pub proof: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    Membership,
    NonMembership,
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
    move || ErrorObject::owned(MISSING_STATE_ERROR_CODE, message, data)
}
