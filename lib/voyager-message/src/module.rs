use std::collections::VecDeque;

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use macros::model;
use schemars::JsonSchema;
use serde_json::Value;
use unionlabs::{bytes::Bytes, ibc::core::client::height::Height, traits::Member};
use voyager_core::{ConsensusType, IbcSpecId};
use voyager_vm::{pass::PassResult, BoxDynError, Op};

use crate::{
    core::{
        ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface, IbcSpec,
    },
    data::Data,
    RawClientId, VoyagerMessage,
};

fn ok<T>(t: T) -> Result<T, BoxDynError> {
    Ok(t)
}

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct StateModuleInfo {
    #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId,
    #[arg(value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
    pub ibc_spec_id: IbcSpecId,
}

impl StateModuleInfo {
    pub fn id(&self) -> String {
        format!("state/{}/{}", self.ibc_spec_id, self.chain_id)
    }

    // TODO: Add this for ibc_spec_id
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

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ProofModuleInfo {
    #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId,
    #[arg(value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
    pub ibc_spec_id: IbcSpecId,
}

impl ProofModuleInfo {
    pub fn id(&self) -> String {
        format!("proof/{}/{}", self.ibc_spec_id, self.chain_id)
    }

    // TODO: Add this for ibc_spec_id
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

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ConsensusModuleInfo {
    #[arg(value_parser(|s: &str| ok(ChainId::new(s.to_owned()))))]
    pub chain_id: ChainId,
    #[arg(value_parser(|s: &str| ok(ConsensusType::new(s.to_owned()))))]
    pub consensus_type: ConsensusType,
    // REVIEW: Maybe we need this? Do different client types for a single consensus necessarily have the same client and consensus state types?
    // /// The type of client this consensus module provides state for.
    // #[arg(value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
    // pub client_type: ClientType,
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

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ClientModuleInfo {
    /// The client type that this client module provides functionality for.
    #[arg(value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
    pub client_type: ClientType,

    /// The consensus type that this client module verifies.
    #[arg(value_parser(|s: &str| ok(ConsensusType::new(s.to_owned()))))]
    pub consensus_type: ConsensusType,

    /// The IBC interface that this client module provides functionality for.
    #[arg(value_parser(|s: &str| ok(IbcInterface::new(s.to_owned()))))]
    pub ibc_interface: IbcInterface,

    /// The IBC version that this client module provides functionality for.
    #[arg(value_parser(|s: &str| ok(IbcSpecId::new(s.to_owned()))))]
    pub ibc_spec_id: IbcSpecId,
}

impl ClientModuleInfo {
    pub fn id(&self) -> String {
        format!(
            "client/{}/{}/{}/{}",
            self.client_type, self.consensus_type, self.ibc_interface, self.ibc_spec_id
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

    pub fn ensure_ibc_spec_id(
        &self,
        ibc_spec_id: impl AsRef<str>,
    ) -> Result<(), UnexpectedIbcVersionIdError> {
        if ibc_spec_id.as_ref() != self.ibc_spec_id.as_str() {
            Err(UnexpectedIbcVersionIdError {
                expected: self.ibc_spec_id.clone(),
                found: ibc_spec_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[model]
#[derive(clap::Args, JsonSchema)]
pub struct ClientBootstrapModuleInfo {
    /// The client type that this client bootstrap module provides functionality for.
    #[arg(value_parser(|s: &str| ok(ClientType::new(s.to_owned()))))]
    pub client_type: ClientType,

    /// The id of the chain that this client bootstrap module provides initial state for.
    #[arg(value_parser(|s: &str| ok(ConsensusType::new(s.to_owned()))))]
    pub chain_id: ChainId,
}

impl ClientBootstrapModuleInfo {
    pub fn id(&self) -> String {
        format!("client-bootstrap/{}/{}", self.client_type, self.chain_id)
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

    // TODO: Add this for ibc_spec_id
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

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid chain id: expected `{expected}` but the rpc responded with `{found}`")]
pub struct UnexpectedChainIdError {
    pub expected: ChainId,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid consensus type: this module provides functionality for consensus type `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedConsensusTypeError {
    pub expected: ConsensusType,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid client type: this module provides functionality for client type `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedClientTypeError {
    pub expected: ClientType,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid IBC interface: this module provides functionality for IBC interface `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedIbcInterfaceError {
    pub expected: IbcInterface,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid IBC version: this module provides functionality for IBC version `{expected}`, but the config specifies `{found}`")]
pub struct UnexpectedIbcVersionIdError {
    pub expected: IbcSpecId,
    pub found: String,
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

#[rpc(
    client,
    server,
    client_bounds(Self:),
    server_bounds(Self:),
    namespace = "state",
)]
pub trait StateModule<V: IbcSpec> {
    /// Query a proof of IBC state on this chain, at the specified [`Height`],
    /// returning the proof as a JSON [`Value`].
    #[method(name = "queryIbcState", with_extensions)]
    async fn query_ibc_state(&self, at: Height, path: V::StorePath) -> RpcResult<Value>;

    /// Fetch the client info of a client on this chain.
    #[method(name = "clientInfo", with_extensions)]
    async fn client_info(&self, client_id: V::ClientId) -> RpcResult<ClientInfo>;
}

/// Type-erased version of [`StateModuleClient`].
#[rpc(client, namespace = "state")]
pub trait RawStateModule {
    #[method(name = "queryIbcState")]
    async fn query_ibc_state_raw(&self, at: Height, path: Value) -> RpcResult<Value>;

    #[method(name = "clientInfo")]
    async fn client_info_raw(&self, client_id: RawClientId) -> RpcResult<ClientInfo>;
}

#[rpc(client,
    server,
    client_bounds(Self:),
    server_bounds(Self:),
    namespace = "proof",
)]
pub trait ProofModule<V: IbcSpec> {
    /// Query a proof of IBC state on this chain, at the specified [`Height`],
    /// returning the state as a JSON [`Value`].
    #[method(name = "queryIbcProof", with_extensions)]
    async fn query_ibc_proof(&self, at: Height, path: V::StorePath) -> RpcResult<Value>;
}

/// Type-erased version of [`ProofModuleClient`].
#[rpc(client, namespace = "proof")]
pub trait RawProofModule {
    #[method(name = "queryIbcProof")]
    async fn query_ibc_proof_raw(&self, at: Height, path: Value) -> RpcResult<Value>;
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
        client_type: ClientType,
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
        client_type: ClientType,
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
}

/// Client bootstrap modules provide the initial client and consensus states for a client. This is notably separate from the [`ConsensusModule`], since it is possible for different client types (with different state types) to track the same consensus.
#[rpc(client, server, namespace = "clientBootstrap")]
pub trait ClientBootstrapModule {
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
