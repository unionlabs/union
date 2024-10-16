use std::num::NonZeroU64;

use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use serde_utils::Hex;
use unionlabs::{
    hash::H256,
    ibc::core::{
        channel::{order::Order, state::State},
        client::height::Height,
        connection::connection_end::ConnectionEnd,
    },
    ics24::Path,
    id::{ChannelId, ClientId, ConnectionId, PortId},
    ErrorReporter, QueryHeight,
};
use valuable::Valuable;
use voyager_core::IbcStoreFormat;

use crate::{
    context::LoadedModulesInfo,
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface},
    macros::model,
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

    #[method(name = "clientTypeIbcStoreFormat")]
    async fn client_type_ibc_store_format(
        &self,
        client_id: ClientType<'static>,
    ) -> RpcResult<IbcStoreFormat<'static>>;

    #[method(name = "clientMeta")]
    async fn client_meta(
        &self,
        chain_id: ChainId<'static>,
        at: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "queryClientState")]
    async fn query_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<IbcState<Hex<Vec<u8>>>>;

    #[method(name = "queryClientConsensusState")]
    async fn query_client_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<IbcState<Hex<Vec<u8>>>>;

    #[method(name = "queryConnection")]
    async fn query_connection(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        connection_id: ConnectionId,
    ) -> RpcResult<IbcState<Option<ConnectionInfo>>>;

    #[method(name = "queryChannelEnd")]
    async fn query_channel(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<Option<ChannelInfo>>>;

    #[method(name = "queryCommitment")]
    async fn query_commitment(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>>;

    #[method(name = "queryAcknowledgement")]
    async fn query_acknowledgement(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>>;

    #[method(name = "queryReceipt")]
    async fn query_receipt(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<bool>>;

    #[method(name = "queryNextSequenceSend")]
    async fn query_next_sequence_send(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextSequenceRecv")]
    async fn query_next_sequence_recv(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextSequenceAck")]
    async fn query_next_sequence_ack(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextConnectionSequence")]
    async fn query_next_connection_sequence(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextClientSequence")]
    async fn query_next_client_sequence(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryIbcProof")]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        path: Path,
        ibc_store_format: IbcStoreFormat<'static>,
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
pub struct IbcState<State> {
    pub chain_id: ChainId<'static>,
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

// /// Serves the same purpose as `ChainModuleClientExt`.
// pub trait VoyagerRpcClientExt: VoyagerRpcClient {
//     // TODO: Maybe rename? Cor likes "_checked"
//     // TODO: Maybe take by ref here?
//     #[allow(async_fn_in_trait)]
//     async fn query_ibc_state_typed<P: IbcPath<Value: DeserializeOwned> + Serialize + Valuable>(
//         &self,
//         chain_id: ChainId<'static>,
//         at: QueryHeight,
//         path: P,
//     ) -> Result<IbcState<P::Value, P>, jsonrpsee::core::client::Error> {
//         debug!(path = path.as_value(), %at, "querying ibc state");

//         let ibc_state = self
//             .query_ibc_state(chain_id.clone(), at, path.clone().into())
//             .await?;

//         Ok(serde_json::from_value::<P::Value>(ibc_state.state.clone())
//             .map(|value| IbcState {
//                 chain_id: ibc_state.chain_id,
//                 path: path.clone(),
//                 height: ibc_state.height,
//                 state: value,
//             })
//             .map_err(|e| {
//                 ErrorObject::owned(
//                     FATAL_JSONRPC_ERROR_CODE,
//                     format!("unable to deserialize state: {}", ErrorReporter(e)),
//                     Some(json!({
//                         "chain_id": chain_id,
//                         "path": path,
//                         "state": ibc_state.state
//                     })),
//                 )
//             })?)
//     }
// }

// impl<T> VoyagerRpcClientExt for T where T: VoyagerRpcClient {}

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

pub type ConnectionInfo = ConnectionEnd;

#[model]
#[derive(Valuable)]
pub struct ChannelInfo {
    pub port_id: PortId,
    pub state: State,
    pub ordering: Order,
    pub counterparty_channel_id: Option<ChannelId>,
    pub connection_hops: Vec<ConnectionId>,
    pub version: String,
}
