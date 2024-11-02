use std::num::NonZeroU64;

use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use serde_json::Value;
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

use crate::{
    context::LoadedModulesInfo,
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, IbcInterface, QueryHeight},
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
    async fn query_latest_height(
        &self,
        chain_id: ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<Height>;

    #[method(name = "queryLatestTimestamp")]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(
        &self,
        chain_id: ChainId<'static>,
        finalized: bool,
    ) -> RpcResult<i64>;

    #[method(name = "queryClientPrefix")]
    async fn query_client_prefix(
        &self,
        chain_id: ChainId<'static>,
        raw_client_id: u32,
    ) -> RpcResult<String>;

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

    // =================
    // IBC state queries
    // =================

    #[method(name = "queryClientState")]
    async fn query_client_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
    ) -> RpcResult<IbcState<Bytes>>;

    #[method(name = "queryClientConsensusState")]
    async fn query_client_consensus_state(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<IbcState<Bytes>>;

    #[method(name = "queryConnection")]
    async fn query_connection(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        connection_id: ConnectionId,
    ) -> RpcResult<IbcState<Option<ConnectionEnd>>>;

    #[method(name = "queryChannelEnd")]
    async fn query_channel(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<Option<Channel>>>;

    #[method(name = "queryCommitment")]
    async fn query_commitment(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>>;

    #[method(name = "queryAcknowledgement")]
    async fn query_acknowledgement(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<Option<H256>>>;

    #[method(name = "queryReceipt")]
    async fn query_receipt(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<IbcState<bool>>;

    #[method(name = "queryNextSequenceSend")]
    async fn query_next_sequence_send(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextSequenceRecv")]
    async fn query_next_sequence_recv(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<IbcState<u64>>;

    #[method(name = "queryNextSequenceAck")]
    async fn query_next_sequence_ack(
        &self,
        chain_id: ChainId<'static>,
        height: QueryHeight,
        port_id: PortId,
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
    ) -> RpcResult<IbcProof>;

    // ========================================
    // self state queries, for creating clients
    // ========================================

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

    // ======================
    // state and proof codecs
    // ======================

    #[method(name = "encodeProof")]
    async fn encode_proof(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        proof: Value,
    ) -> RpcResult<Bytes>;

    #[method(name = "decodeClientStateMeta")]
    async fn decode_client_state_meta(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta>;

    #[method(name = "decodeClientState")]
    async fn decode_client_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        client_state: Bytes,
    ) -> RpcResult<Value>;

    #[method(name = "decodeConsensusState")]
    async fn decode_consensus_state(
        &self,
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
        consensus_state: Bytes,
    ) -> RpcResult<Value>;
}

#[model]
pub struct IbcState<State = Value> {
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
