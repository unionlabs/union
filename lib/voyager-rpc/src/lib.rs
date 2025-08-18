#![feature(trait_alias)]

use std::{
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
};

use jsonrpsee::{
    self,
    core::RpcResult,
    proc_macros::rpc,
    types::{
        error::{INVALID_PARAMS_CODE, METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE},
        ErrorObject, ErrorObjectOwned,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unionlabs::{ibc::core::client::height::Height, primitives::Bytes, ErrorReporter};
use voyager_message::{data::Data, VoyagerMessage};
use voyager_primitives::{
    ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface, IbcSpec,
    IbcSpecId, QueryHeight, Timestamp,
};
use voyager_types::{ProofType, RawClientId};
use voyager_vm::{pass::PassResult, Op, QueueError};

use crate::types::{
    IbcProofResponse, IbcStateResponse, InfoResponse, SelfClientStateResponse,
    SelfConsensusStateResponse,
};

pub mod types;

/// Trait alias for traits commonly used together throughout this crate.
// TODO: Add `Eq`
pub trait Member = Debug
    + Clone
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + Send
    + Sync
    + Unpin
    + 'static;

/// Error code for fatal errors. If a plugin or module responds with this error code, it will be
/// treated as fatal and not retried.
pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

/// Error code for unprocessable messages. If a plugin or module responds with this error code, it
/// will be treated as fatal and not retried.
pub const UNPROCESSABLE_JSONRPC_ERROR_CODE: i32 = -0xDEADC0D; // 🐟

/// Error code for missing state. If a plugin or module responds with this error code, it will be
/// requeued and retried.
pub const MISSING_STATE_ERROR_CODE: i32 = -0xBADB10B;

/// Error code for any kind of external RPC error. If a plugin or module responds with this error code, it will be
/// requeued and retried.
pub const RPC_ERROR_ERROR_CODE: i32 = -0xBADACE;

/// Convert a [`jsonrpsee::core::client::Error`] to a `voyager-vm` [`QueueError`].
///
/// All errors are treated as retryable, unless `error` is a `Call` variant and the contained
/// [`ErrorObject`] is deemed to be fatal. See [`error_object_to_queue_error`] for more information
/// on the conversion from [`ErrorObject`] to [`QueueError`].
pub fn json_rpc_error_to_queue_error(error: jsonrpsee::core::client::Error) -> QueueError {
    match error {
        jsonrpsee::core::client::Error::Call(error) => error_object_to_queue_error(error),
        value => QueueError::Retry(Box::new(value)),
    }
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

pub enum ModuleError {
    Jsonrpsee(jsonrpsee::core::client::Error),
}

impl ModuleError {
    // fn from_error_object(error: ErrorObjectOwned) -> ModuleError {
    //     let message = error.message().to_owned();
    //     let data = error
    //         .data()
    //         .map(|v| serde_json::to_value(v).expect("infallible; qed;"));

    //     match error.code() {
    //         MISSING_STATE_ERROR_CODE => Self::MissingState { message, data },
    //         RPC_ERROR_ERROR_CODE => Self::RpcError {
    //             message,
    //             data,
    //             error: None,
    //         },
    //         FATAL_JSONRPC_ERROR_CODE => Self::Fatal {
    //             message,
    //             data,
    //             error: None,
    //         },
    //         -1 => Self::Fatal {
    //             message,
    //             data,
    //             error: None,
    //         },
    //         UNPROCESSABLE_JSONRPC_ERROR_CODE => Self::Unprocessable {
    //             message,
    //             data,
    //             error: None,
    //         },
    //         _ => Self::Other(error),
    //     }
    // }
}

impl From<jsonrpsee::core::client::Error> for ModuleError {
    fn from(value: jsonrpsee::core::client::Error) -> Self {
        // match value {
        //     jsonrpsee::core::client::Error::Call(error) => Self::from_error_object(error),
        //     value => Self::Retryable {
        //         message: String::new(),
        //         data: None,
        //         error: Some(Box::new(value)),
        //     },
        // }
        Self::Jsonrpsee(value)
    }
}

impl From<ErrorObjectOwned> for ModuleError {
    fn from(value: ErrorObjectOwned) -> Self {
        // match value {
        //     jsonrpsee::core::client::Error::Call(error) => Self::from_error_object(error),
        //     value => Self::Retryable {
        //         message: String::new(),
        //         data: None,
        //         error: Some(Box::new(value)),
        //     },
        // }
        Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(value))
    }
}

impl From<ModuleError> for ErrorObjectOwned {
    fn from(value: ModuleError) -> Self {
        match value {
            ModuleError::Jsonrpsee(error) => json_rpc_error_to_error_object(error),
        }
        // match value {
        //     ModuleError::MissingState { message, data } => {
        //         ErrorObject::owned(MISSING_STATE_ERROR_CODE, message, data)
        //     }
        //     ModuleError::RpcError {
        //         message,
        //         data,
        //         error,
        //     } => ErrorObject::owned(
        //         RPC_ERROR_ERROR_CODE,
        //         error.as_deref().map_or(String::new(), |error| {
        //             ErrorReporter(error).with_message(&message)
        //         }),
        //         data,
        //     ),
        //     ModuleError::Fatal {
        //         message,
        //         data,
        //         error,
        //     } => ErrorObject::owned(
        //         FATAL_JSONRPC_ERROR_CODE,
        //         error.as_deref().map_or(String::new(), |error| {
        //             ErrorReporter(error).with_message(&message)
        //         }),
        //         data,
        //     ),
        //     ModuleError::Retryable {
        //         message,
        //         data,
        //         error,
        //     } => ErrorObject::owned(
        //         -1,
        //         error.as_deref().map_or(String::new(), |error| {
        //             ErrorReporter(error).with_message(&message)
        //         }),
        //         data,
        //     ),
        //     ModuleError::Unprocessable {
        //         message,
        //         data,
        //         error,
        //     } => ErrorObject::owned(
        //         UNPROCESSABLE_JSONRPC_ERROR_CODE,
        //         error.as_deref().map_or(String::new(), |error| {
        //             ErrorReporter(error).with_message(&message)
        //         }),
        //         data,
        //     ),
        //     ModuleError::Other(other) => other,
        // }
    }
}

impl ModuleError {
    // /// Some required state was missing (connection/channel end, packet commitment,
    // /// ..)
    // pub fn missing_state(message: impl Display, data: Option<Value>) -> impl FnOnce() -> Self {
    //     move || Self::MissingState {
    //         message: message.to_string(),
    //         data,
    //     }
    // }

    pub fn retry<E: Error + 'static>(message: impl Display) -> impl FnOnce(E) -> Self {
        move |e| {
            Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
                -1,
                ErrorReporter(e).with_message(&message.to_string()),
                None::<()>,
            )))
        }
    }

    pub fn retry_with_data<E: Error + 'static>(
        message: impl Display,
        data: Value,
    ) -> impl FnOnce(E) -> Self {
        move |e| {
            Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
                -1,
                ErrorReporter(e).with_message(&message.to_string()),
                Some(data),
            )))
        }
    }

    pub fn fatal<E: Error + 'static>(
        message: impl Display,
        data: Option<Value>,
    ) -> impl FnOnce(E) -> Self {
        move |e| {
            Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message(&message.to_string()),
                data,
            )))
        }
    }

    pub fn fatal_no_error(message: impl Display, data: Option<Value>) -> Self {
        Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            message.to_string(),
            data,
        )))
    }

    pub fn new_with_data(message: impl Display, data: Value) -> Self {
        Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
            -1,
            message.to_string(),
            Some(data),
        )))
    }

    pub fn new(message: impl Display) -> Self {
        Self::Jsonrpsee(jsonrpsee::core::client::Error::Call(ErrorObject::owned(
            -1,
            message.to_string(),
            None::<()>,
        )))
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

pub fn rpc_error<E: Error>(
    message: impl Display,
    data: Option<Value>,
) -> impl FnOnce(E) -> ErrorObjectOwned {
    move |e| ErrorObject::owned(-1, format!("{message}: {}", ErrorReporter(e)), data)
}

/// Convert a `jsonrpsee` [`ErrorObject`] to a `voyager-vm` [`QueueError`].
///
/// Certain error codes are treated as fatal (i.e. not retryable):
///
/// - [`FATAL_JSONRPC_ERROR_CODE`]: Custom error code that can be returned by plugins and modules to
///   denote that a fatal error has occurred, and this message is not retryable.
/// - [`METHOD_NOT_FOUND_CODE`]: The plugin or module does not expose the method that was attempted
///   to be called. This indicates a bug in the plugin or module.
/// - [`PARSE_ERROR_CODE`] or [`INVALID_PARAMS_CODE`]: The custom message sent to the plugin or
///   module could not be deserialized. This could either be due a bug in the plugin or module (JSON
///   serialization not roundtripping correctly) or a message that was manually inserted into the
///   queue via `/enqueue`.
///
/// Certain error codes are treated as unprocessable (i.e. not retryable, but not due to a fatal
/// error):
///
/// - [`UNPROCESSABLE_JSONRPC_ERROR_CODE`]: Custom error code that can be returned by plugins and
///   modules to denote that a message cannot be processed.
pub fn error_object_to_queue_error(error: ErrorObject<'_>) -> QueueError {
    if error.code() == FATAL_JSONRPC_ERROR_CODE
        || error.code() == METHOD_NOT_FOUND_CODE
        || error.code() == INVALID_PARAMS_CODE
        || error.code() == PARSE_ERROR_CODE
    {
        QueueError::Fatal(Box::new(error.into_owned()))
    } else if error.code() == UNPROCESSABLE_JSONRPC_ERROR_CODE {
        QueueError::Unprocessable(Box::new(error.into_owned()))
    } else {
        QueueError::Retry(Box::new(error.into_owned()))
    }
}

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
    async fn info(&self) -> RpcResult<InfoResponse>;

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
    ) -> RpcResult<IbcStateResponse<Value>>;

    #[method(name = "queryIbcProof", with_extensions)]
    async fn query_ibc_proof(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        height: QueryHeight,
        path: Value,
    ) -> RpcResult<IbcProofResponse>;

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
    ) -> RpcResult<SelfClientStateResponse>;

    #[method(name = "selfConsensusState", with_extensions)]
    async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusStateResponse>;

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

    #[method(name = "encodeClientState", with_extensions)]
    async fn encode_client_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes>;

    #[method(name = "encodeConsensusState", with_extensions)]
    async fn encode_consensus_state(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
        consensus_state: Value,
    ) -> RpcResult<Bytes>;

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

    /// Handle a custom request for this module.
    ///
    /// The default implementetion returns an error.
    #[method(name = "custom", with_extensions)]
    async fn custom(&self, _method: String, _params: Vec<Value>) -> RpcResult<Value> {
        Err(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            "unimplemented",
            None::<()>,
        ))
    }
}

#[rpc(
    client,
    server,
    client_bounds(Self:),
    server_bounds(Self:),
    namespace = "state",
)]
pub trait StateModule<V: IbcSpec> {
    /// Execute a query on this chain returning the proof as a JSON [`Value`].
    #[method(name = "query", with_extensions)]
    async fn query(&self, query: V::Query) -> Result<Value, ModuleError>;

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
    #[method(name = "query", with_extensions)]
    async fn query_raw(&self, query: Value) -> RpcResult<Value>;

    #[method(name = "queryIbcState", with_extensions)]
    async fn query_ibc_state_raw(&self, at: Height, path: Value) -> RpcResult<Value>;

    #[method(name = "clientInfo", with_extensions)]
    async fn client_info_raw(&self, client_id: RawClientId) -> RpcResult<Option<ClientInfo>>;
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
    async fn query_ibc_proof(
        &self,
        at: Height,
        path: V::StorePath,
    ) -> RpcResult<Option<(Value, ProofType)>>;
}

/// Type-erased version of [`ProofModuleClient`].
#[rpc(client, namespace = "proof")]
pub trait RawProofModule {
    #[method(name = "queryIbcProof")]
    async fn query_ibc_proof_raw(
        &self,
        at: Height,
        path: Value,
    ) -> RpcResult<Option<(Value, ProofType)>>;
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
pub trait FinalityModule {
    /// Query the latest finalized height of this chain.
    #[method(name = "queryLatestHeight", with_extensions)]
    async fn query_latest_height(&self, finalized: bool) -> RpcResult<Height>;

    /// Query the latest finalized timestamp of this chain.
    #[method(name = "queryLatestTimestamp", with_extensions)]
    async fn query_latest_timestamp(&self, finalized: bool) -> RpcResult<Timestamp>;
}

/// Client bootstrap modules provide the initial client and consensus states for a client. This is
/// notably separate from the [`FinalityModule`], since it is possible for different client types
/// (with different state types) to track the same consensus.
#[rpc(client, server, namespace = "clientBootstrap")]
pub trait ClientBootstrapModule {
    /// The client state of this chain at the specified [`Height`].
    ///
    /// Returns the client state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    ///
    /// This also accepts a config value as arbitrary json that can be used for client-specific configuration parameters.
    #[method(name = "selfClientState", with_extensions)]
    async fn self_client_state(&self, height: Height, config: Value) -> RpcResult<Value>;

    /// The consensus state of this chain at the specified [`Height`].
    ///
    /// Returns the consensus state value as JSON, which will then be encoded to
    /// bytes by a ClientModule.
    ///
    /// This also accepts a config value as arbitrary json that can be used for client-specific configuration parameters.
    #[method(name = "selfConsensusState", with_extensions)]
    async fn self_consensus_state(&self, height: Height, config: Value) -> RpcResult<Value>;
}
