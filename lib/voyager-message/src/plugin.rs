use std::{borrow::Cow, collections::VecDeque};

use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObject};
use macros::model;
use queue_msg::{optimize::OptimizationResult, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::debug;
use unionlabs::{
    hash::H256,
    ibc::core::client::height::Height,
    ics24::{IbcPath, Path},
    id::ClientId,
    traits::Member,
    ErrorReporter,
};

use crate::{
    data::{ClientInfo, Data},
    ClientType, IbcInterface, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

// REVIEW: Rename?
#[rpc(client, server, namespace = "plugin")]
pub trait PluginModule<D: Member, F: Member, A: Member> {
    /// The name of this plugin that will be used as the `Plugin.plugin_name` to
    /// direct custom messages to this module.
    #[method(name = "info")]
    async fn info(&self) -> RpcResult<PluginInfo>;

    /// Handle a custom `Fetch` message for this module.
    #[method(name = "handleFetch")]
    async fn handle_fetch(&self, fetch: F) -> RpcResult<Op<VoyagerMessage<D, F, A>>>;

    /// Handle a custom `Aggregate` message for this module.
    #[method(name = "handleAggregate")]
    fn handle_aggregate(
        &self,
        aggregate: A,
        data: VecDeque<Data<D>>,
    ) -> RpcResult<Op<VoyagerMessage<D, F, A>>>;
}

#[model]
pub struct PluginInfo {
    /// The name of this plugin. Any [`PluginMessage`]s with a `plugin` field
    /// matching this will be routed to this module.
    pub name: String,
    /// Optional plugin kind. If `None`, this module will not be loaded as a
    /// specific chain module type, only as a generic module.
    pub kind: Option<PluginKind>,
    /// A jaq filter to run on every message before pushing them to the queue.
    /// This ***MUST*** return a bool. If this returns true, the message will be
    /// pushed to the optimization queue with this plugin's name as the tag,
    /// else it will be passed on to the next plugin to be filtered. If this is
    /// None, this plugin will not be registered as an
    /// [`OptimizationPassPlugin`].
    pub interest_filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum PluginKind {
    Chain,
    Client,
    Consensus,
    // Transaction,
}

impl std::fmt::Display for PluginKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Chain => "chain",
            Self::Client => "client",
            Self::Consensus => "consensus",
            // Self::Transaction => "transaction",
        })
    }
}

#[rpc(
    client,
    server,
    client_bounds(Self: PluginModuleClient<D, F, A>),
    server_bounds(Self: PluginModuleServer<D, F, A>),
    namespace = "optimizeModule"
)]
pub trait OptimizationPassPlugin<D: Member, F: Member, A: Member> {
    #[method(name = "runPass")]
    fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage<D, F, A>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<D, F, A>>>;
}

/// Chain modules provide functionality to interact with a single chain,
/// providing interfaces to interact with the
#[rpc(
    client,
    server,
    client_bounds(Self: PluginModuleClient<D, F, A>),
    server_bounds(Self: PluginModuleServer<D, F, A>),
    namespace = "chainModule"
)]
pub trait ChainModule<D: Member, F: Member, A: Member> {
    /// Register this chain module with Voyager, returning the chain id of the
    /// chain this module tracks.
    #[method(name = "chainId")]
    fn chain_id(&self) -> RpcResult<String>;

    /// Query the latest finalized height of this chain.
    #[method(name = "queryLatestHeight")]
    async fn query_latest_height(&self) -> RpcResult<Height>;

    /// Query the latest (non-finalized) height of this chain.
    #[method(name = "queryLatestHeightAsDestination")]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height>;

    /// Query the latest finalized timestamp of this chain.
    #[method(name = "queryLatestTimestamp")]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(&self) -> RpcResult<i64>;

    #[method(name = "fetchBlockRange")]
    async fn fetch_block_range(
        &self,
        from_height: Height,
        to_height: Height,
    ) -> RpcResult<Op<VoyagerMessage<D, F, A>>>;

    /// Query the latest raw, unfinalized trusted client state of the client
    /// `client_id`.
    // SEE: <https://github.com/unionlabs/union/issues/1813>
    #[method(name = "queryRawUnfinalizedTrustedClientState")]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>>;

    // TODO: For the state and proof query functions, it would be best if they
    // weren't concerned with the encoding of them; this should be handed off to a
    // different module

    /// Fetch the client info of a client on this chain.
    #[method(name = "clientInfo")]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo>;

    /// Query IBC state on this chain, at the specified [`Height`], returning
    /// the value as a JSON [`Value`].
    #[method(name = "queryIbcState")]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value>;

    /// Query a proof of IBC state on this chain, at the specified [`Height`],
    /// returning the proof as a JSON [`Value`].
    #[method(name = "queryIbcProof")]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value>;
}

pub trait ChainModuleClientExt: ChainModuleClient<Value, Value, Value> + Send + Sync {
    // TODO: Maybe rename? Cor likes "_checked"
    // TODO: Maybe take by ref here?
    #[allow(async_fn_in_trait)]
    async fn query_ibc_state_typed<P: IbcPath>(
        &self,
        at: Height,
        path: P,
    ) -> Result<P::Value, jsonrpsee::core::client::Error> {
        debug!(%path, %at, "querying ibc state");

        let state = self.query_ibc_state(at, path.clone().into()).await?;

        Ok(
            serde_json::from_value::<P::Value>(state.clone()).map_err(|e| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize state: {}", ErrorReporter(e)),
                    Some(json!({
                        "path": path,
                        "state": state
                    })),
                )
            })?,
        )
    }
}

impl<T> ChainModuleClientExt for T where T: ChainModuleClient<Value, Value, Value> + Send + Sync {}

/// Raw, un-decoded client state, as queried directly from the client store.
#[model]
pub struct RawClientState<'a> {
    pub client_type: ClientType<'a>,
    pub ibc_interface: IbcInterface<'a>,
    pub bytes: Cow<'a, [u8]>,
}

/// Client modules provide functionality to interact with a single light client
/// type, on a single IBC interface. This can also be thought of as a "client
/// codec", as all of the endpoints it exposes are related to encoding and
/// decoding state.
#[rpc(
    client,
    server,
    client_bounds(Self: PluginModuleClient<D, F, A>),
    server_bounds(Self: PluginModuleServer<D, F, A>),
    namespace = "clientModule"
)]
// TODO: Rename to client codec module
pub trait ClientModule<D: Member, F: Member, A: Member> {
    /// Register this module with Voyager.
    #[method(name = "supportedInterface")]
    async fn supported_interface(&self) -> RpcResult<SupportedInterface>;

    /// Decode the raw client state, returning the decoded metadata common
    /// between all client state types.
    #[method(name = "decodeClientStateMeta")]
    async fn decode_client_state_meta(
        &self,
        client_state: Cow<'static, [u8]>,
    ) -> RpcResult<ClientStateMeta>;

    /// Decode the raw consensus state, returning the decoded metadata common
    /// between all consensus state types.
    #[method(name = "decodeConsensusStateMeta")]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Cow<'static, [u8]>,
    ) -> RpcResult<ConsensusStateMeta>;

    /// Decode the raw client state, returning the decoded state as JSON.
    #[method(name = "decodeClientState")]
    async fn decode_client_state(&self, client_state: Cow<'static, [u8]>) -> RpcResult<Value>;

    /// Decode the raw consensus state, returning the decoded state as JSON.
    #[method(name = "decodeConsensusState")]
    async fn decode_consensus_state(&self, consensus_state: Cow<'static, [u8]>)
        -> RpcResult<Value>;

    /// Encode the client state, provided as JSON.
    #[method(name = "encodeClientState")]
    async fn encode_client_state(&self, client_state: Value, metadata: Value)
        -> RpcResult<Vec<u8>>;

    /// Encode the consensus state, provided as JSON.
    #[method(name = "encodeConsensusState")]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Vec<u8>>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyClientState")]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Cow<'static, [u8]>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyConsensusState")]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Cow<'static, [u8]>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Vec<u8>>;

    /// Encode the header, provided as JSON.
    #[method(name = "encodeHeader")]
    async fn encode_header(&self, header: Value) -> RpcResult<Vec<u8>>;

    /// Encode the proof, provided as JSON.
    #[method(name = "encodeProof")]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Vec<u8>>;
}

/// Client modules provide functionality for interacting with a specific chain
/// consensus.
#[rpc(
    client,
    server,
    client_bounds(Self: PluginModuleClient<D, F, A>),
    server_bounds(Self: PluginModuleServer<D, F, A>),
    namespace = "consensusModule"
)]
pub trait ConsensusModule<D: Member, F: Member, A: Member> {
    /// Register this module with Voyager.
    #[method(name = "info")]
    async fn consensus_info(&self) -> RpcResult<ConsensusModuleInfo>;

    /// The client state of this chain at the specified [`Height`].
    ///
    /// Returns the client state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    #[method(name = "selfClientState")]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value>;

    /// The consensus state of this chain at the specified [`Height`].
    ///
    /// Returns the consensus state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    #[method(name = "selfConsensusState")]
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value>;

    /// Generate a client update for this module's client type.
    ///
    /// # Implementor's Note
    ///
    /// The returned [`Op`] ***MUST*** resolve to an [`OrderdHeaders`] data.
    /// This is the entrypoint called when a client update is requested, and
    /// will be called in the queue of an
    /// [`AggregateMsgUpdateClientsFromOrderedHeaders`], which will be used to
    /// build the actual [`MsgUpdateClient`]s.
    #[method(name = "fetchUpdateHeaders")]
    fn fetch_update_headers(
        &self,
        update_from: Height,
        update_to: Height,
    ) -> RpcResult<Op<VoyagerMessage<D, F, A>>>;
}

#[model]
pub struct ConsensusModuleInfo {
    pub chain_id: String,
    pub client_type: ClientType<'static>,
}

#[model]
pub struct SupportedInterface {
    /// The client type that this module provides functionality for.
    pub client_type: ClientType<'static>,

    /// The IBC interface that this module provides functionality for.
    pub ibc_interface: IbcInterface<'static>,
}

#[model]
pub struct ClientStateMeta {
    /// The counterparty height this client has been updated to. A consensus
    /// state will exist at this height.
    pub height: Height,

    /// The chain id of the counterparty chain this client tracks.
    pub chain_id: String,
}

#[model]
pub struct ConsensusStateMeta {
    /// The timestamp of the counterparty at the height represented by this
    /// consensus state.
    pub timestamp_nanos: u64,
}

// /// Transaction submission modules provide functionality to submit transactions
// /// on-chain. This is independent from chain modules
// #[rpc(
//     client,
//     server,
//     client_bounds(Self: PluginModuleClient<D, F, A>),
//     server_bounds(Self: PluginModuleServer<D, F, A>),
//     namespace = "transactionModule"
// )]
// pub trait TransactionSubmissionModule<D: Member, F: Member, A: Member> {
//     /// Register this transaction submission module with Voyager, returning the
//     /// chain id of the chain this module supports.
//     #[method(name = "register")]
//     fn register(&self) -> RpcResult<String>;

//     /// Submit a transaction to this chain. The messages contain state
//     /// pre-encoded for their respective client types and counterparty IBC
//     /// interfaces, which should then be transformed to the format expected
//     /// by transaction submission for this chain and signed.
//     #[method(name = "sendTransaction")]
//     async fn send_transaction(&self, msg: Vec<Msg>) -> RpcResult<Op<VoyagerMessage<D, F, A>>>;
// }

#[model]
pub struct IbcGo08WasmClientMetadata {
    pub checksum: H256,
}
