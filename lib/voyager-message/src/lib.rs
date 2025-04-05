#![feature(trait_alias, slice_partition_dedup)]

use std::{
    borrow::Cow,
    env::VarError,
    fmt::{self, Debug},
    future::Future,
    time::Duration,
};

use clap::builder::{StringValueParser, TypedValueParser, ValueParserFactory};
use futures::FutureExt;
use jsonrpsee::{
    core::{
        async_trait,
        client::{BatchResponse, ClientT},
        params::BatchRequestBuilder,
        traits::ToRpcParams,
        RpcResult,
    },
    server::middleware::rpc::RpcServiceT,
    types::{
        error::{INVALID_PARAMS_CODE, METHOD_NOT_FOUND_CODE, PARSE_ERROR_CODE},
        ErrorObject, Response, ResponsePayload,
    },
    Extensions, MethodResponse, RpcModule,
};
use macros::model;
use reth_ipc::{
    client::IpcClientBuilder,
    server::{RpcService, RpcServiceBuilder},
};
use rpc::{SelfClientState, SelfConsensusState};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, value::RawValue, Value};
use tracing::{
    debug, debug_span, error, info_span, instrument, instrument::Instrumented, trace, Instrument,
};
use unionlabs::{
    ibc::core::client::height::Height, primitives::Bytes, traits::Member, ErrorReporter,
};
use voyager_primitives::{
    ChainId, ClientInfo, ClientStateMeta, ClientType, ConsensusStateMeta, IbcInterface, IbcQuery,
    IbcSpec, IbcSpecId, IbcStorePathKey, QueryHeight, Timestamp,
};
use voyager_vm::{BoxDynError, ItemId, QueueError, QueueMessage};

use crate::{
    call::Call,
    callback::Callback,
    context::{Context, INVALID_CONFIG_EXIT_CODE, STARTUP_ERROR_EXIT_CODE},
    data::Data,
    filter::JaqInterestFilter,
    module::{
        ClientBootstrapModuleInfo, ClientBootstrapModuleServer, ClientModuleInfo,
        ClientModuleServer, ConsensusModuleInfo, ConsensusModuleServer, PluginInfo, PluginServer,
        ProofModuleInfo, ProofModuleServer, StateModuleInfo, StateModuleServer,
    },
    rpc::{json_rpc_error_to_error_object, IbcProof, IbcState, VoyagerRpcClient},
};

pub mod call;
pub mod callback;
pub mod data;

pub mod context;
pub mod filter;
pub mod module;
pub mod pass;

pub mod hook;

pub mod rpc;

pub use reconnecting_jsonrpc_ws_client;
pub use reth_ipc;
pub use voyager_primitives as primitives;
pub use voyager_vm as vm;

#[derive(Debug, Clone, PartialEq)]
pub enum VoyagerMessage {}

impl QueueMessage for VoyagerMessage {
    type Call = Call;
    type Data = Data;
    type Callback = Callback;

    type Filter = JaqInterestFilter;

    type Context = Context;
}

/// Simple wrapper around a [`Value`] for raw client ids.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RawClientId(Value);

#[derive(Clone)]
pub struct RawClientIdValueParser;

impl TypedValueParser for RawClientIdValueParser {
    type Value = RawClientId;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = StringValueParser::new().parse_ref(cmd, arg, value)?;

        // attempt to parse the string as json, if that fails just treat the whole string as a json
        // string value
        Ok(RawClientId(
            s.parse::<Value>()
                .unwrap_or_else(|_| Value::String(s.to_owned())),
        ))
    }
}

impl fmt::Display for RawClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ValueParserFactory for RawClientId {
    type Parser = RawClientIdValueParser;

    fn value_parser() -> Self::Parser {
        RawClientIdValueParser
    }
}

impl RawClientId {
    pub fn new(t: impl Serialize) -> Self {
        Self(serde_json::to_value(t).unwrap())
    }

    pub fn decode_spec<V: IbcSpec>(self) -> Result<V::ClientId, serde_json::Error> {
        serde_json::from_value(self.0)
    }

    pub fn as_raw(&self) -> &Value {
        &self.0
    }
}

/// Error code for fatal errors. If a plugin or module responds with this error code, it will be
/// treated as fatal and not retried.
pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

/// Error code for unprocessable messages. If a plugin or module responds with this error code, it
/// will be treated as fatal and not retried.
pub const UNPROCESSABLE_JSONRPC_ERROR_CODE: i32 = -0xDEADC0D; // 🐟

/// Error code for missing state. If a plugin or module responds with this error code, it will be
/// requeued and retried.
pub const MISSING_STATE_ERROR_CODE: i32 = -0xBADB10B;

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

/// A message specific to a plugin.
///
/// This is used in [`Call`], [`Callback`], and [`Data`] to route messages to plugins.
#[model]
pub struct PluginMessage {
    pub plugin: String,
    pub message: Value,
}

impl PluginMessage {
    pub fn new(plugin_name: impl Into<String>, message: impl Serialize) -> Self {
        Self {
            plugin: plugin_name.into(),
            message: serde_json::to_value(message).expect(
                "serialization must be infallible, this is a bug in the plugin implementation",
            ),
        }
    }

    pub fn downcast<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        if self.plugin == plugin_name.as_ref() {
            if let Ok(t) = serde_json::from_value(self.message.clone()) {
                Ok(t)
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }
}

#[derive(clap::Subcommand)]
pub enum DefaultCmd {}

fn init_log() {
    enum LogFormat {
        Text,
        Json,
    }

    let format = match std::env::var("RUST_LOG_FORMAT").as_deref() {
        Err(VarError::NotPresent) | Ok("text") => LogFormat::Text,
        Ok("json") => LogFormat::Json,
        Err(VarError::NotUnicode(invalid)) => {
            eprintln!("invalid non-utf8 log format {invalid:?}, defaulting to text");
            LogFormat::Text
        }
        Ok(invalid) => {
            eprintln!("invalid log format {invalid}, defaulting to text");
            LogFormat::Text
        }
    };

    match format {
        LogFormat::Text => {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
                // .with_span_events(FmtSpan::CLOSE)
                .json()
                .init();
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait Plugin: PluginServer<Self::Call, Self::Callback> + Sized {
    type Call: Member;
    type Callback: Member;

    type Config: DeserializeOwned + Clone;
    type Cmd: clap::Subcommand;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError>;

    fn info(config: Self::Config) -> PluginInfo;

    async fn cmd(config: Self::Config, cmd: Self::Cmd);

    async fn run() {
        init_log();

        let app = <PluginApp<Self::Cmd> as clap::Parser>::parse();

        match app {
            PluginApp::Run {
                socket,
                voyager_socket,
                config,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = Self::info(config.clone());

                let name = info.name;

                run_server(
                    name.clone(),
                    voyager_socket,
                    config,
                    socket,
                    Self::new,
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_plugin_server", %name))
                .await
            }
            PluginApp::Info { config } => {
                let info = Self::info(must_parse(&config));

                print!("{}", serde_json::to_string(&info).unwrap())
            }
            PluginApp::Cmd { cmd, config } => Self::cmd(must_parse(&config), cmd).await,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait StateModule<V: IbcSpec>: StateModuleServer<V> + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError>;

    async fn run() {
        init_log();

        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                socket,
                voyager_socket,
                config,
                info,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<StateModuleInfo>(&info);

                let name = info.id();

                run_server(
                    name.clone(),
                    voyager_socket,
                    (config, info),
                    socket,
                    |(config, info)| Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_state_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ProofModule<V: IbcSpec>: ProofModuleServer<V> + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ProofModuleInfo) -> Result<Self, BoxDynError>;

    async fn run() {
        init_log();

        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                socket,
                voyager_socket,
                config,
                info,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ProofModuleInfo>(&info);

                let name = info.id();

                run_server(
                    name.clone(),
                    voyager_socket,
                    (config, info),
                    socket,
                    |(config, info)| Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_proof_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ConsensusModule: ConsensusModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError>;

    async fn run() {
        init_log();

        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                socket,
                voyager_socket,
                config,
                info,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ConsensusModuleInfo>(&info);

                let name = info.id();

                run_server(
                    name.clone(),
                    voyager_socket,
                    (config, info),
                    socket,
                    |(config, info)| Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_consensus_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ClientModule: ClientModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError>;

    async fn run() {
        init_log();

        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                socket,
                voyager_socket,
                config,
                info,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientModuleInfo>(&info);

                let name = info.id();

                run_server(
                    name.clone(),
                    voyager_socket,
                    (config, info),
                    socket,
                    |(config, info)| Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_client_module_server", %name))
                .await
            }
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait ClientBootstrapModule: ClientBootstrapModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(
        config: Self::Config,
        info: ClientBootstrapModuleInfo,
    ) -> Result<Self, BoxDynError>;

    async fn run() {
        init_log();

        match <ModuleApp as clap::Parser>::parse() {
            ModuleApp::Run {
                socket,
                voyager_socket,
                config,
                info,
            } => {
                let config = must_parse::<Self::Config>(&config);

                let info = must_parse::<ClientBootstrapModuleInfo>(&info);

                let name = info.id();

                run_server(
                    name.clone(),
                    voyager_socket,
                    (config, info),
                    socket,
                    |(config, info)| Self::new(config, info),
                    Self::into_rpc,
                )
                .instrument(debug_span!("run_client_bootstrap_module_server", %name))
                .await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct IdThreadClient<Inner: ClientT + Send + Sync> {
    pub(crate) client: Inner,
    item_id: Option<ItemId>,
}

fn new_voyager_client(name: String, socket: String) -> reconnecting_jsonrpc_ws_client::Client {
    let client = reconnecting_jsonrpc_ws_client::Client::new({
        let voyager_socket: &'static str = socket.leak();
        let name = name.clone();
        move || {
            async move {
                trace!("connecting to socket at {voyager_socket}");
                IpcClientBuilder::default().build(voyager_socket).await
            }
            .instrument(debug_span!("voyager_ipc_client", %name))
        }
    });

    client
}

#[derive(Debug, Clone)]
pub struct VoyagerClient(IdThreadClient<reconnecting_jsonrpc_ws_client::Client>);

#[async_trait]
impl<Inner: ClientT + Send + Sync> ClientT for IdThreadClient<Inner> {
    async fn notification<Params>(
        &self,
        _method: &str,
        _params: Params,
    ) -> Result<(), jsonrpsee::core::client::Error>
    where
        Params: ToRpcParams + Send,
    {
        Err(jsonrpsee::core::client::Error::Custom(
            "notifications are not supported".to_owned(),
        ))
    }

    #[instrument(skip_all)]
    async fn request<R, Params>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<R, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        trace!(item_id = ?self.item_id);

        match self.item_id {
            Some(item_id) => {
                self.client
                    .request(
                        method,
                        ParamsWithItemId {
                            item_id,
                            params: params.to_rpc_params()?.map(Cow::Owned),
                        },
                    )
                    .await
            }
            None => self.client.request(method, params).await,
        }
    }

    async fn batch_request<'a, R>(
        &self,
        _batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        Err(jsonrpsee::core::client::Error::Custom(
            "batch requests are not supported".to_owned(),
        ))
    }
}

impl VoyagerClient {
    pub async fn query_latest_timestamp(
        &self,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Timestamp> {
        let latest_timestamp = self
            .0
            .query_latest_timestamp(chain_id, finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(latest_timestamp)
    }

    pub async fn self_client_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfClientState> {
        let client_state = self
            .0
            .self_client_state(chain_id, client_type, height, config)
            .await
            .map_err(json_rpc_error_to_error_object)?;
        Ok(client_state)
    }

    pub async fn self_consensus_state(
        &self,
        chain_id: ChainId,
        client_type: ClientType,
        height: QueryHeight,
        config: Value,
    ) -> RpcResult<SelfConsensusState> {
        let consensus_state = self
            .0
            .self_consensus_state(chain_id, client_type, height, config)
            .await
            .map_err(json_rpc_error_to_error_object)?;
        Ok(consensus_state)
    }

    pub async fn query_latest_height(
        &self,
        chain_id: ChainId,
        finalized: bool,
    ) -> RpcResult<Height> {
        let latest_height = self
            .0
            .query_latest_height(chain_id, finalized)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(latest_height)
    }

    #[instrument(
        skip_all,
        name = "voyager_client_encode_proof",
        fields(
            %client_type,
            %ibc_interface,
            %proof
        )
    )]
    pub async fn encode_proof<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        proof: Value,
    ) -> RpcResult<Bytes> {
        let proof = self
            .0
            .encode_proof(client_type, ibc_interface, V::ID, proof)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(proof)
    }

    #[instrument(
        skip_all,
        name = "voyager_client_encode_header",
        fields(
            %client_type,
            %ibc_interface,
            %header
        )
    )]
    pub async fn encode_header<V: IbcSpec>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        header: Value,
    ) -> RpcResult<Bytes> {
        let header = self
            .0
            .encode_header(client_type, ibc_interface, V::ID, header)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(header)
    }

    pub async fn decode_client_state<V: IbcSpec, T: DeserializeOwned>(
        &self,
        client_type: ClientType,
        ibc_interface: IbcInterface,
        client_state_bytes: Bytes,
    ) -> RpcResult<T> {
        let client_state = self
            .0
            .decode_client_state(client_type, ibc_interface, V::ID, client_state_bytes)
            .await
            .map_err(json_rpc_error_to_error_object)?;

        serde_json::from_value(client_state).map_err(|e| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                ErrorReporter(e).with_message("error decoding client state from json value"),
                None::<()>,
            )
        })
    }

    pub async fn query<Q: IbcQuery>(&self, chain_id: ChainId, query: Q) -> RpcResult<Q::Value> {
        self.0
            .query(
                chain_id.clone(),
                <Q::Spec as IbcSpec>::ID,
                into_value(<Q::Spec as IbcSpec>::Query::from(query.clone().into())),
            )
            .await
            .map_err(json_rpc_error_to_error_object)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(|e| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("error decoding query return value: {}", ErrorReporter(e)),
                        Some(json!({
                            "query": query
                        })),
                    )
                })
            })
    }

    pub async fn query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: Height,
        path: P,
    ) -> RpcResult<P::Value> {
        let state = self
            .maybe_query_ibc_state(
                chain_id.clone(),
                QueryHeight::Specific(height),
                path.clone(),
            )
            .await?
            .state
            .ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "state must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": height,
                        "path": path
                    })),
                )
            })?;

        Ok(state)
    }

    pub async fn maybe_query_ibc_state<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
        path: P,
    ) -> RpcResult<IbcState<P::Value>> {
        let ibc_state = self
            .0
            .query_ibc_state(
                chain_id,
                P::Spec::ID,
                height,
                into_value(<P::Spec as IbcSpec>::StorePath::from(path.into())),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(IbcState {
            height: ibc_state.height,
            state: ibc_state
                .state
                .map(|state| {
                    serde_json::from_value(state.clone()).map_err(|e| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("error decoding IBC state: {}", ErrorReporter(e)),
                            Some(json!({
                                "raw_state": state
                            })),
                        )
                    })
                })
                .transpose()?,
        })
    }

    pub async fn query_ibc_proof<P: IbcStorePathKey>(
        &self,
        chain_id: ChainId,
        height: QueryHeight,
        path: P,
    ) -> RpcResult<IbcProof> {
        let ibc_proof = self
            .0
            .query_ibc_proof(
                chain_id,
                P::Spec::ID,
                height,
                into_value(<P::Spec as IbcSpec>::StorePath::from(path.into())),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        Ok(ibc_proof)
    }

    pub async fn equivalent_chain_ids(&self, chain_id: ChainId) -> RpcResult<Vec<ChainId>> {
        self.0
            .equivalent_chain_ids(chain_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_info<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        client_id: V::ClientId,
    ) -> RpcResult<ClientInfo> {
        self.maybe_client_info::<V>(chain_id.clone(), client_id.clone())
            .await
            .and_then(|client_info| {
                client_info.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client info must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_info<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        client_id: V::ClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.0
            .client_info(chain_id, V::ID, RawClientId::new(client_id))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_info_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<ClientInfo> {
        self.maybe_client_info_raw(chain_id.clone(), ibc_spec_id, client_id.clone())
            .await
            .and_then(|client_info| {
                client_info.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client info must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_info_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientInfo>> {
        self.0
            .client_info(chain_id, ibc_spec_id, client_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.maybe_client_state_meta::<V>(chain_id.clone(), at, client_id.clone())
            .await
            .and_then(|client_state_meta| {
                client_state_meta.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client state meta must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "height": at,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.0
            .client_state_meta(chain_id, V::ID, at, RawClientId::new(client_id))
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn client_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<ClientStateMeta> {
        self.maybe_client_state_meta_raw(chain_id.clone(), ibc_spec_id, at, client_id.clone())
            .await
            .and_then(|client_state_meta| {
                client_state_meta.ok_or_else(|| {
                    ErrorObject::owned(
                        MISSING_STATE_ERROR_CODE,
                        "client state meta must exist",
                        Some(json!({
                            "chain_id": chain_id,
                            "height": at,
                            "client_id": client_id
                        })),
                    )
                })
            })
    }

    pub async fn maybe_client_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
    ) -> RpcResult<Option<ClientStateMeta>> {
        self.0
            .client_state_meta(chain_id, ibc_spec_id, at, client_id)
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn consensus_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
        counterparty_height: Height,
    ) -> RpcResult<ConsensusStateMeta> {
        self.maybe_consensus_state_meta::<V>(
            chain_id.clone(),
            at,
            client_id.clone(),
            counterparty_height,
        )
        .await
        .and_then(|consensus_state_meta| {
            consensus_state_meta.ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "client state meta must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": at,
                        "client_id": client_id,
                        "counterparty_height": counterparty_height
                    })),
                )
            })
        })
    }

    pub async fn maybe_consensus_state_meta<V: IbcSpec>(
        &self,
        chain_id: ChainId,
        at: QueryHeight,
        client_id: V::ClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.0
            .consensus_state_meta(
                chain_id,
                V::ID,
                at,
                RawClientId::new(client_id),
                counterparty_height,
            )
            .await
            .map_err(json_rpc_error_to_error_object)
    }

    pub async fn consensus_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<ConsensusStateMeta> {
        self.maybe_consensus_state_meta_raw(
            chain_id.clone(),
            ibc_spec_id,
            at,
            client_id.clone(),
            counterparty_height,
        )
        .await
        .and_then(|consensus_state_meta| {
            consensus_state_meta.ok_or_else(|| {
                ErrorObject::owned(
                    MISSING_STATE_ERROR_CODE,
                    "client state meta must exist",
                    Some(json!({
                        "chain_id": chain_id,
                        "height": at,
                        "client_id": client_id,
                        "counterparty_height": counterparty_height
                    })),
                )
            })
        })
    }

    pub async fn maybe_consensus_state_meta_raw(
        &self,
        chain_id: ChainId,
        ibc_spec_id: IbcSpecId,
        at: QueryHeight,
        client_id: RawClientId,
        counterparty_height: Height,
    ) -> RpcResult<Option<ConsensusStateMeta>> {
        self.0
            .consensus_state_meta(chain_id, ibc_spec_id, at, client_id, counterparty_height)
            .await
            .map_err(json_rpc_error_to_error_object)
    }
}

pub trait ExtensionsExt {
    /// Retrieve a value from this [`Extensions`], returning an [`RpcResult`] for more
    /// convenient handling in rpc server implementations.
    fn try_get<T: Send + Sync + 'static>(&self) -> RpcResult<&T>;
}

impl ExtensionsExt for Extensions {
    fn try_get<T: Send + Sync + 'static>(&self) -> RpcResult<&T> {
        match self.get() {
            Some(t) => Ok(t),
            None => Err(ErrorObject::owned(
                -1,
                format!(
                    "failed to retrieve value of type {} from extensions",
                    std::any::type_name::<T>(),
                ),
                None::<()>,
            )),
        }
    }
}

#[derive(clap::Parser)]
enum PluginApp<Cmd: clap::Subcommand> {
    Run {
        socket: String,
        voyager_socket: String,
        config: String,
    },
    Info {
        config: String,
    },
    Cmd {
        #[command(subcommand)]
        cmd: Cmd,
        #[arg(long)]
        config: String,
    },
}

#[derive(clap::Parser)]
enum ModuleApp {
    Run {
        socket: String,
        voyager_socket: String,
        config: String,
        info: String,
    },
}

#[instrument(level = "debug", fields(%config_str))]
fn must_parse<T: DeserializeOwned>(config_str: &str) -> T {
    match serde_json::from_str::<T>(config_str) {
        Ok(ok) => ok,
        Err(err) => {
            error!("invalid config: {}", ErrorReporter(err));
            std::process::exit(INVALID_CONFIG_EXIT_CODE as i32);
        }
    }
}

async fn run_server<
    T,
    NewF: FnOnce(NewT) -> Fut,
    NewT,
    Fut: Future<Output = Result<T, BoxDynError>>,
    IntoRpcF: FnOnce(T) -> RpcModule<T>,
>(
    id: String,
    voyager_socket: String,
    new_t: NewT,
    socket: String,
    new: NewF,
    into_rpc: IntoRpcF,
) {
    let voyager_client = new_voyager_client(id.clone(), voyager_socket);
    if let Err(err) = voyager_client
        .wait_until_connected(Duration::from_millis(500))
        .await
    {
        error!("unable to connect to voyager socket: {err}");
        std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
    };

    trace!("connected to voyager socket");

    let module_server = match new(new_t).await {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("startup error in {id}: {err:?}");
            std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
        }
    };

    let id_ = id.clone();
    let ipc_server = reth_ipc::server::Builder::default()
        .set_rpc_middleware(
            RpcServiceBuilder::new()
                .layer_fn(move |service| InjectClient {
                    client: voyager_client.clone(),
                    service,
                })
                .layer_fn(move |service: RpcService| ErrorContext {
                    service,
                    id: id_.clone(),
                }),
        )
        .build(socket);

    let rpcs = into_rpc(module_server);

    trace!(methods = ?*rpcs, "registered methods");
    let addr = ipc_server.endpoint();
    let server_handle = ipc_server.start(rpcs).await.unwrap();
    debug!("listening on {addr}");

    tokio::spawn(
        server_handle
            .stopped()
            .instrument(debug_span!("module_server", %id)),
    )
    .await
    .unwrap()
}

struct InjectClient<S> {
    client: reconnecting_jsonrpc_ws_client::Client,
    service: S,
}

impl<'a, S: RpcServiceT<'a> + Send + Sync> RpcServiceT<'a> for InjectClient<S> {
    type Future = futures::future::Either<Instrumented<S::Future>, S::Future>;

    fn call(&self, mut request: jsonrpsee::types::Request<'a>) -> Self::Future {
        if let Some(params) = request.params.take() {
            match serde_json::from_str(params.get()) {
                Ok(ParamsWithItemId { item_id, params }) => {
                    let mut request = jsonrpsee::types::Request {
                        params: params.map(|rv| Cow::Owned(rv.into_owned())),
                        ..request
                    };

                    trace!("request is for item_id {}", item_id.raw());

                    request.extensions.insert(item_id);

                    request.extensions.insert(VoyagerClient(IdThreadClient {
                        client: self.client.clone(),
                        item_id: Some(item_id),
                    }));

                    return self
                        .service
                        .call(request)
                        .instrument(info_span!("item_id", item_id = item_id.raw()))
                        .left_future();
                }
                Err(err) => {
                    trace!(
                        "unable to parse item_id from request: {}",
                        ErrorReporter(err)
                    );

                    request.params = Some(params);
                }
            }
        };

        trace!("request is not for a queue item");

        request.extensions.insert(VoyagerClient(IdThreadClient {
            client: self.client.clone(),
            item_id: None,
        }));

        self.service.call(request).right_future()
    }
}

struct ErrorContext<S> {
    service: S,
    id: String,
}

impl<'a, S: RpcServiceT<'a> + Send + Sync> RpcServiceT<'a> for ErrorContext<S> {
    type Future = futures::future::Map<
        S::Future,
        Box<dyn Fn(MethodResponse) -> MethodResponse + Send + Sync>,
    >;

    fn call(&self, request: jsonrpsee::types::Request<'a>) -> Self::Future {
        let id = self.id.clone();
        self.service
            .call(request)
            .map(Box::new(move |method_response| {
                if method_response.is_error() {
                    let result = method_response.into_result();

                    let response = serde_json::from_str::<Response<()>>(&result).unwrap();

                    let ResponsePayload::Error(error) = response.payload else {
                        panic!();
                    };

                    let error = ErrorObject::owned(
                        error.code(),
                        format!("error in {}: {}", id, error.message()),
                        error.data(),
                    )
                    .into_owned();

                    MethodResponse::error(response.id, error)
                } else {
                    method_response
                }
            }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsWithItemId<'a> {
    item_id: ItemId,
    #[serde(borrow)]
    params: Option<Cow<'a, RawValue>>,
}

impl ToRpcParams for ParamsWithItemId<'_> {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        Ok(Some(
            RawValue::from_string(serde_json::to_string(&self)?).unwrap(),
        ))
    }
}

#[track_caller]
pub fn into_value<T: Debug + Serialize>(t: T) -> Value {
    match serde_json::to_value(t) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                error = %ErrorReporter(err),
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );

            panic!(
                "error serializing value of type {}",
                std::any::type_name::<T>()
            );
        }
    }
}

pub fn ensure_null(value: Value) -> RpcResult<()> {
    if value == Value::Null {
        Ok(())
    } else {
        Err(ErrorObject::owned(
            FATAL_JSONRPC_ERROR_CODE,
            format!("expected null but found {value}"),
            None::<()>,
        ))
    }
}
