use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::debug;
use unionlabs::{
    ibc::core::client::height::Height,
    ics24::{IbcPath, Path},
    id::ClientId,
    ErrorReporter, QueryHeight,
};

use crate::{
    context::LoadedModulesInfo,
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface},
    FATAL_JSONRPC_ERROR_CODE,
};

pub mod server;

#[rpc(
    client,
    server,
    client_bounds(Self: Send + Sync),
    server_bounds(Self:),
    namespace = "voyager",
)]
pub trait VoyagerRpc {
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<LoadedModulesInfo>;

    #[method(name = "queryLatestHeight")]
    async fn query_latest_height(&self, chain_id: ChainId<'static>) -> RpcResult<Height>;

    #[method(name = "queryLatestTimestamp")]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(&self, chain_id: ChainId<'static>) -> RpcResult<i64>;

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

    #[method(name = "queryIbcState")]
    async fn query_ibc_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
    ) -> RpcResult<IbcState>;

    #[method(name = "queryIbcProof")]
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

    #[method(name = "encodeProof")]
    async fn encode_proof(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        proof: Value,
    ) -> RpcResult<Hex<Vec<u8>>>;

    #[method(name = "decodeClientStateMeta")]
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "decodeClientState")]
    async fn decode_client_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value>;

    #[method(name = "decodeConsensusState")]
    async fn decode_consensus_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value>;
}

#[model]
// TODO: Flip these parameters
pub struct IbcState<State = Value, P = Path> {
    pub chain_id: ChainId<'static>,
    pub path: P,
    /// The height that the state was read at.
    pub height: Height,
    pub state: State,
}

#[model]
pub struct IbcProof {
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
    ) -> Result<IbcState<P::Value, P>, jsonrpsee::core::client::Error> {
        debug!(%path, %at, "querying ibc state");

        let ibc_state = self
            .query_ibc_state(chain_id.clone(), at, path.clone().into())
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

impl<T> VoyagerRpcClientExt for T where T: VoyagerRpcClient {}

pub fn json_rpc_error_to_error_object(value: jsonrpsee::core::client::Error) -> ErrorObjectOwned {
    match value {
        jsonrpsee::core::client::Error::Call(error) => error,
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
