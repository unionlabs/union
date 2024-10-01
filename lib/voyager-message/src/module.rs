use std::{borrow::Cow, collections::VecDeque};

use enumorph::Enumorph;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::ErrorObject, RpcModule};
use macros::model;
use queue_msg::{optimize::OptimizationResult, Op};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::debug;
use unionlabs::{
    ibc::core::client::height::Height,
    ics24::{IbcPath, Path},
    id::ClientId,
    traits::Member,
    ErrorReporter,
};
#[cfg(doc)]
use {
    crate::{callback::AggregateMsgUpdateClientsFromOrderedHeaders, data::OrderedHeaders},
    unionlabs::ibc::core::client::msg_update_client::MsgUpdateClient,
};

use crate::{
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface},
    data::Data,
    ModuleContext, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

#[model]
pub struct ModuleInfo<K> {
    pub kind: K,
}

impl ModuleKindInfo {
    pub fn name(&self) -> String {
        match self {
            ModuleKindInfo::Chain(info) => info.name(),
            ModuleKindInfo::Consensus(info) => info.name(),
            ModuleKindInfo::Client(info) => info.name(),
            ModuleKindInfo::Plugin(info) => info.name(),
        }
    }
}

#[model]
#[derive(Enumorph)]
pub enum ModuleKindInfo {
    Chain(ChainModuleInfo),
    Consensus(ConsensusModuleInfo),
    Client(ClientModuleInfo),
    Plugin(PluginInfo),
}

pub trait IModuleKindInfo: Member + Into<ModuleKindInfo> {
    fn name(&self) -> String;
}

#[model]
pub struct ChainModuleInfo {
    pub chain_id: ChainId<'static>,
}

impl IModuleKindInfo for ChainModuleInfo {
    fn name(&self) -> String {
        format!("chain/{}", self.chain_id)
    }
}

#[model]
pub struct ConsensusModuleInfo {
    pub chain_id: ChainId<'static>,
    pub client_type: ClientType<'static>,
}

impl IModuleKindInfo for ConsensusModuleInfo {
    fn name(&self) -> String {
        format!("consensus/{}/{}", self.chain_id, self.client_type)
    }
}

#[model]
pub struct ClientModuleInfo {
    /// The client type that this client module provides functionality for.
    pub client_type: ClientType<'static>,

    /// The IBC interface that this client module provides functionality for.
    pub ibc_interface: IbcInterface<'static>,
}

impl IModuleKindInfo for ClientModuleInfo {
    fn name(&self) -> String {
        format!("client/{}/{}", self.client_type, self.ibc_interface)
    }
}

#[model]
pub struct PluginInfo {
    /// The name of this plugin. Any plugin messages with this name will be
    /// routed to this plugin.
    pub name: String,
    /// A jaq filter to run on every message before pushing them to the queue.
    /// This ***MUST*** return a bool. If this returns `true`, the message will
    /// be pushed to the optimization queue with this plugin's name as the tag,
    /// otherwise it will be passed on to the next plugin to be filtered.
    pub interest_filter: String,
}

impl IModuleKindInfo for PluginInfo {
    fn name(&self) -> String {
        format!("plugin/{}", self.name)
    }
}

pub trait IntoRpc: Sized {
    type RpcModule;

    fn into_rpc(t: Self::RpcModule) -> RpcModule<Self::RpcModule>;
}

impl<T> IntoRpc for (PluginInfo, T)
where
    T: ModuleContext<Info = PluginInfo>
        + PluginTypes
        + PluginServer<<T as PluginTypes>::D, <T as PluginTypes>::C, <T as PluginTypes>::Cb>,
{
    type RpcModule = T;

    fn into_rpc(t: Self::RpcModule) -> RpcModule<Self::RpcModule> {
        PluginServer::into_rpc(t)
    }
}

// .try_get::<VoyagerClient>()?

impl<T> IntoRpc for (ChainModuleInfo, T)
where
    T: ModuleContext<Info = ChainModuleInfo> + ChainModuleServer,
{
    type RpcModule = T;

    fn into_rpc(t: Self::RpcModule) -> RpcModule<Self::RpcModule> {
        ChainModuleServer::into_rpc(t)
    }
}

impl<T> IntoRpc for (ClientModuleInfo, T)
where
    T: ModuleContext<Info = ClientModuleInfo> + ClientModuleServer,
{
    type RpcModule = T;

    fn into_rpc(t: Self::RpcModule) -> RpcModule<Self::RpcModule> {
        ClientModuleServer::into_rpc(t)
    }
}

impl<T> IntoRpc for (ConsensusModuleInfo, T)
where
    T: ModuleContext<Info = ConsensusModuleInfo> + ConsensusModuleServer,
{
    type RpcModule = T;

    fn into_rpc(t: Self::RpcModule) -> RpcModule<Self::RpcModule> {
        ConsensusModuleServer::into_rpc(t)
    }
}

pub trait PluginTypes {
    type D: Member;
    type C: Member;
    type Cb: Member;
}

#[rpc(client, server, namespace = "plugin")]
pub trait Plugin<D: Member, C: Member, Cb: Member> {
    #[method(name = "runPass", with_extensions)]
    async fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage<D, C, Cb>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<D, C, Cb>>>;

    /// Handle a custom `Call` message for this module.
    #[method(name = "call", with_extensions)]
    async fn call(&self, call: C) -> RpcResult<Op<VoyagerMessage<D, C, Cb>>>;

    /// Handle a custom `Callback` message for this module.
    #[method(name = "callback", with_extensions)]
    async fn callback(
        &self,
        aggregate: Cb,
        data: VecDeque<Data<D>>,
    ) -> RpcResult<Op<VoyagerMessage<D, C, Cb>>>;
}

/// Chain modules provide functionality to interact with a single chain,
/// providing interfaces to interact with the
#[rpc(client, server, namespace = "chain")]
pub trait ChainModule {
    /// Query the latest finalized height of this chain.
    #[method(name = "queryLatestHeight", with_extensions)]
    async fn query_latest_height(&self) -> RpcResult<Height>;

    /// Query the latest (non-finalized) height of this chain.
    #[method(name = "queryLatestHeightAsDestination", with_extensions)]
    async fn query_latest_height_as_destination(&self) -> RpcResult<Height>;

    /// Query the latest finalized timestamp of this chain.
    #[method(name = "queryLatestTimestamp", with_extensions)]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(&self) -> RpcResult<i64>;

    /// Query the latest raw, unfinalized trusted client state of the client
    /// `client_id`.
    // SEE: <https://github.com/unionlabs/union/issues/1813>
    #[method(name = "queryRawUnfinalizedTrustedClientState", with_extensions)]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState<'static>>;

    /// Fetch the client info of a client on this chain.
    #[method(name = "clientInfo", with_extensions)]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo>;

    /// Query IBC state on this chain, at the specified [`Height`], returning
    /// the value as a JSON [`Value`].
    #[method(name = "queryIbcState", with_extensions)]
    async fn query_ibc_state(&self, at: Height, path: Path) -> RpcResult<Value>;

    /// Query a proof of IBC state on this chain, at the specified [`Height`],
    /// returning the proof as a JSON [`Value`].
    #[method(name = "queryIbcProof", with_extensions)]
    async fn query_ibc_proof(&self, at: Height, path: Path) -> RpcResult<Value>;
}

/// Raw, un-decoded client state, as queried directly from the client store.
#[model]
pub struct RawClientState<'a> {
    pub client_type: ClientType<'a>,
    pub ibc_interface: IbcInterface<'a>,
    pub bytes: Cow<'a, [u8]>,
}

pub trait ChainModuleClientExt: ChainModuleClient + Send + Sync {
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

impl<T> ChainModuleClientExt for T where T: ChainModuleClient + Send + Sync {}

/// Client modules provide functionality to interact with a single light client
/// type, on a single IBC interface. This can also be thought of as a "client
/// codec", as all of the endpoints it exposes are related to encoding and
/// decoding state.
#[rpc(client, server, namespace = "client")]
// TODO: Rename to client codec module
pub trait ClientModule {
    /// Decode the raw client state, returning the decoded metadata common
    /// between all client state types.
    #[method(name = "decodeClientStateMeta", with_extensions)]
    async fn decode_client_state_meta(
        &self,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta>;

    /// Decode the raw consensus state, returning the decoded metadata common
    /// between all consensus state types.
    #[method(name = "decodeConsensusStateMeta", with_extensions)]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta>;

    /// Decode the raw client state, returning the decoded state as JSON.
    #[method(name = "decodeClientState", with_extensions)]
    async fn decode_client_state(&self, client_state: Hex<Vec<u8>>) -> RpcResult<Value>;

    /// Decode the raw consensus state, returning the decoded state as JSON.
    #[method(name = "decodeConsensusState", with_extensions)]
    async fn decode_consensus_state(&self, consensus_state: Hex<Vec<u8>>) -> RpcResult<Value>;

    /// Encode the client state, provided as JSON.
    #[method(name = "encodeClientState", with_extensions)]
    async fn encode_client_state(
        &self,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>>;

    /// Encode the consensus state, provided as JSON.
    #[method(name = "encodeConsensusState", with_extensions)]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Hex<Vec<u8>>>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyClientState", with_extensions)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Hex<Vec<u8>>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyConsensusState", with_extensions)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Hex<Vec<u8>>,
        client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>>;

    /// Encode the header, provided as JSON.
    #[method(name = "encodeHeader", with_extensions)]
    async fn encode_header(&self, header: Value) -> RpcResult<Hex<Vec<u8>>>;

    /// Encode the proof, provided as JSON.
    #[method(name = "encodeProof", with_extensions)]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Hex<Vec<u8>>>;
}

/// Client modules provide functionality for interacting with a specific chain
/// consensus.
#[rpc(client, server, namespace = "consensus")]
pub trait ConsensusModule {
    /// The client state of this chain at the specified [`Height`].
    ///
    /// Returns the client state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    #[method(name = "selfClientState", with_extensions)]
    async fn self_client_state(&self, height: Height) -> RpcResult<Value>;

    /// The consensus state of this chain at the specified [`Height`].
    ///
    /// Returns the consensus state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    #[method(name = "selfConsensusState", with_extensions)]
    async fn self_consensus_state(&self, height: Height) -> RpcResult<Value>;
}
