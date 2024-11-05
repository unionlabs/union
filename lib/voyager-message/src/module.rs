use std::{collections::VecDeque, num::NonZeroU64};

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use macros::model;
use schemars::JsonSchema;
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
    traits::Member,
};
use voyager_core::{ConsensusType, IbcVersion};
use voyager_vm::{pass::PassResult, BoxDynError, Op};
#[cfg(doc)]
use {
    crate::{callback::AggregateMsgUpdateClientsFromOrderedHeaders, data::OrderedHeaders},
    unionlabs::ibc::core::client::msg_update_client::MsgUpdateClient,
};

use crate::{
    core::{ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface},
    data::Data,
    VoyagerMessage,
};

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ChainModuleInfo {
    #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId<'static>,
}

impl ChainModuleInfo {
    pub fn id(&self) -> String {
        format!("chain/{}", self.chain_id)
    }

    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

fn ok<T>(t: T) -> Result<T, BoxDynError> {
    Ok(t)
}

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ConsensusModuleInfo {
    #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId<'static>,
    #[arg(value_parser(|s: &str| ok(ConsensusType::new(s.to_owned()))))]
    pub consensus_type: ConsensusType<'static>,
    // REVIEW: Maybe we need this? Do different client types for a single consensus necessarily have the same client and consensus state types?
    // /// The type of client this consensus module provides state for.
    // #[arg(value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
    // pub client_type: ClientType<'static>,
}

impl ConsensusModuleInfo {
    pub fn id(&self) -> String {
        format!("consensus/{}/{}", self.chain_id, self.consensus_type)
    }

    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_consensus_type(
        &self,
        consensus_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedConsensusTypeError> {
        if consensus_type.as_ref() != self.consensus_type.as_str() {
            Err(UnexpectedConsensusTypeError {
                expected: self.consensus_type.clone(),
                found: consensus_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid chain id: expected `{expected}` but the rpc responded with `{found}`")]
pub struct UnexpectedChainIdError {
    pub expected: ChainId<'static>,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid consensus type: this module provides functionality for consensus type `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedConsensusTypeError {
    pub expected: ConsensusType<'static>,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid client type: this module provides functionality for client type `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedClientTypeError {
    pub expected: ClientType<'static>,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid IBC interface: this module provides functionality for IBC interface `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedIbcInterfaceError {
    pub expected: IbcInterface<'static>,
    pub found: String,
}

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ClientModuleInfo {
    /// The client type that this client module provides functionality for.
    #[arg(value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
    pub client_type: ClientType<'static>,

    /// The consensus type that this client module verifies.
    #[arg(value_parser(|s: &str| ok(ConsensusType::new(s.to_owned()))))]
    pub consensus_type: ConsensusType<'static>,

    /// The IBC interface that this client module provides functionality for.
    #[arg(value_parser(|s: &str| ok(IbcInterface::new(s.to_owned()))))]
    pub ibc_interface: IbcInterface<'static>,
}

impl ClientModuleInfo {
    pub fn id(&self) -> String {
        format!(
            "client/{}/{}/{}",
            self.client_type, self.consensus_type, self.ibc_interface
        )
    }

    pub fn ensure_client_type(
        &self,
        client_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedClientTypeError> {
        if client_type.as_ref() != self.client_type.as_str() {
            Err(UnexpectedClientTypeError {
                expected: self.client_type.clone(),
                found: client_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_consensus_type(
        &self,
        consensus_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedConsensusTypeError> {
        if consensus_type.as_ref() != self.consensus_type.as_str() {
            Err(UnexpectedConsensusTypeError {
                expected: self.consensus_type.clone(),
                found: consensus_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_ibc_interface(
        &self,
        ibc_interface: impl AsRef<str>,
    ) -> Result<(), UnexpectedIbcInterfaceError> {
        if ibc_interface.as_ref() != self.ibc_interface.as_str() {
            Err(UnexpectedIbcInterfaceError {
                expected: self.ibc_interface.clone(),
                found: ibc_interface.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[model]
#[derive(clap::Args, JsonSchema)]
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

#[rpc(client, server, namespace = "plugin")]
pub trait Plugin<C: Member, Cb: Member> {
    #[method(name = "runPass", with_extensions)]
    async fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>>;

    /// Handle a custom `Call` message for this module.
    #[method(name = "call", with_extensions)]
    async fn call(&self, call: C) -> RpcResult<Op<VoyagerMessage>>;

    /// Handle a custom `Callback` message for this module.
    #[method(name = "callback", with_extensions)]
    async fn callback(&self, aggregate: Cb, data: VecDeque<Data>) -> RpcResult<Op<VoyagerMessage>>;
}

/// Chain modules provide functionality to interact with a single chain,
/// providing interfaces to interact with the
#[rpc(client, server, namespace = "chain")]
pub trait ChainModule {
    /// Query the prefix for a client, given it's raw numeric id.
    #[method(name = "queryClientPrefix", with_extensions)]
    async fn query_client_prefix(&self, raw_client_id: u32) -> RpcResult<String>;

    /// Query the latest raw, unfinalized trusted client state of the client
    /// `client_id`.
    // SEE: <https://github.com/unionlabs/union/issues/1813>
    #[method(name = "queryRawUnfinalizedTrustedClientState", with_extensions)]
    async fn query_raw_unfinalized_trusted_client_state(
        &self,
        client_id: ClientId,
    ) -> RpcResult<RawClientState>;

    /// Fetch the client info of a client on this chain.
    #[method(name = "clientInfo", with_extensions)]
    async fn client_info(&self, client_id: ClientId) -> RpcResult<ClientInfo>;

    #[method(name = "queryClientState", with_extensions)]
    async fn query_client_state(&self, height: Height, client_id: ClientId) -> RpcResult<Bytes>;

    #[method(name = "queryClientConsensusState", with_extensions)]
    async fn query_client_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
        trusted_height: Height,
    ) -> RpcResult<Bytes>;

    #[method(name = "queryConnection", with_extensions)]
    async fn query_connection(
        &self,
        height: Height,
        connection_id: ConnectionId,
    ) -> RpcResult<Option<ConnectionEnd>>;

    #[method(name = "queryChannelEnd", with_extensions)]
    async fn query_channel(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<Option<Channel>>;

    #[method(name = "queryCommitment", with_extensions)]
    async fn query_commitment(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>>;

    #[method(name = "queryAcknowledgement", with_extensions)]
    async fn query_acknowledgement(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<Option<H256>>;

    #[method(name = "queryReceipt", with_extensions)]
    async fn query_receipt(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: NonZeroU64,
    ) -> RpcResult<bool>;

    #[method(name = "queryNextSequenceSend", with_extensions)]
    async fn query_next_sequence_send(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64>;

    #[method(name = "queryNextSequenceRecv", with_extensions)]
    async fn query_next_sequence_recv(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64>;

    #[method(name = "queryNextSequenceAck", with_extensions)]
    async fn query_next_sequence_ack(
        &self,
        height: Height,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> RpcResult<u64>;

    #[method(name = "queryNextConnectionSequence", with_extensions)]
    async fn query_next_connection_sequence(&self, height: Height) -> RpcResult<u64>;

    #[method(name = "queryNextClientSequence", with_extensions)]
    async fn query_next_client_sequence(&self, height: Height) -> RpcResult<u64>;

    /// Query a proof of IBC state on this chain, at the specified [`Height`],
    /// returning the proof as a JSON [`Value`].
    // TODO: Pull this out into a module that iself is indexed by IBC version
    #[method(name = "queryIbcProof", with_extensions)]
    async fn query_ibc_proof(
        &self,
        at: Height,
        path: Bytes,
        ibc_version: IbcVersion,
    ) -> RpcResult<Value>;
}

/// Raw, un-decoded client state, as queried directly from the client store.
#[model]
pub struct RawClientState {
    pub client_type: ClientType<'static>,
    pub ibc_interface: IbcInterface<'static>,
    pub bytes: Bytes,
}

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
    async fn decode_client_state_meta(&self, client_state: Bytes) -> RpcResult<ClientStateMeta>;

    /// Decode the raw consensus state, returning the decoded metadata common
    /// between all consensus state types.
    #[method(name = "decodeConsensusStateMeta", with_extensions)]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta>;

    /// Decode the raw client state, returning the decoded state as JSON.
    #[method(name = "decodeClientState", with_extensions)]
    async fn decode_client_state(&self, client_state: Bytes) -> RpcResult<Value>;

    /// Decode the raw consensus state, returning the decoded state as JSON.
    #[method(name = "decodeConsensusState", with_extensions)]
    async fn decode_consensus_state(&self, consensus_state: Bytes) -> RpcResult<Value>;

    /// Encode the client state, provided as JSON.
    #[method(name = "encodeClientState", with_extensions)]
    async fn encode_client_state(&self, client_state: Value, metadata: Value) -> RpcResult<Bytes>;

    /// Encode the consensus state, provided as JSON.
    #[method(name = "encodeConsensusState", with_extensions)]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Bytes>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyClientState", with_extensions)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Bytes,
        client_type: ClientType<'static>,
    ) -> RpcResult<Bytes>;

    /// Re-encode the client state of the specified counterparty client type.
    ///
    /// This is required due to limitations with ibc-go v8, and can likely be
    /// removed once support for that IBC interface is dropped. In most cases,
    /// this will simply be a pass-through of the bytes provided.
    #[method(name = "reencodeCounterpartyConsensusState", with_extensions)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Bytes,
        client_type: ClientType<'static>,
    ) -> RpcResult<Bytes>;

    /// Encode the header, provided as JSON.
    #[method(name = "encodeHeader", with_extensions)]
    async fn encode_header(&self, header: Value) -> RpcResult<Bytes>;

    /// Encode the proof, provided as JSON.
    #[method(name = "encodeProof", with_extensions)]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Bytes>;
}

/// Client modules provide functionality for interacting with a specific chain
/// consensus and finality.
#[rpc(client, server, namespace = "consensus")]
pub trait ConsensusModule {
    /// Query the latest finalized height of this chain.
    #[method(name = "queryLatestHeight", with_extensions)]
    async fn query_latest_height(&self, finalized: bool) -> RpcResult<Height>;

    /// Query the latest finalized timestamp of this chain.
    #[method(name = "queryLatestTimestamp", with_extensions)]
    // TODO: Make this return a better type than i64
    async fn query_latest_timestamp(&self, finalized: bool) -> RpcResult<i64>;

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
