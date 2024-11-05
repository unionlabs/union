#![feature(trait_alias)]
#![allow(clippy::too_many_arguments)] // fight me

use std::{env::VarError, fmt::Debug, future::Future, time::Duration};

use chain_utils::BoxDynError;
use jsonrpsee::{
    core::{
        async_trait, client::BatchResponse, params::BatchRequestBuilder, traits::ToRpcParams,
        RpcResult,
    },
    server::middleware::rpc::RpcServiceT,
    types::{
        error::{INVALID_PARAMS_CODE, METHOD_NOT_FOUND_CODE},
        ErrorObject,
    },
    Extensions, RpcModule,
};
use macros::model;
use reth_ipc::{client::IpcClientBuilder, server::RpcServiceBuilder};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tracing::{debug, debug_span, error, info, trace, Instrument};
use unionlabs::{traits::Member, ErrorReporter};
use voyager_vm::{QueueError, QueueMessage};

use crate::{
    call::Call,
    callback::Callback,
    context::{Context, INVALID_CONFIG_EXIT_CODE, STARTUP_ERROR_EXIT_CODE},
    data::Data,
    filter::JaqInterestFilter,
    module::{
        ChainModuleInfo, ChainModuleServer, ClientModuleInfo, ClientModuleServer,
        ConsensusModuleInfo, ConsensusModuleServer, PluginInfo, PluginServer,
    },
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
pub use voyager_core as core;

pub enum VoyagerMessage {}

impl QueueMessage for VoyagerMessage {
    type Call = Call;
    type Data = Data;
    type Callback = Callback;

    type Filter = JaqInterestFilter;

    type Context = Context;
}

/// Error code for fatal errors. If a plugin or module responds with this error
/// code, it will be treated as failed and not retried.
pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

/// Convert a [`jsonrpsee::core::client::Error`] to a `voyager-vm`
/// [`QueueError`].
///
/// All errors are treated as retryable, unless `error` is a `Call` variant and
/// the contained [`ErrorObject`] is deemed to be fatal. See
/// [`error_object_to_queue_error`] for more information on the conversion from
/// [`ErrorObject`] to [`QueueError`].
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
/// - [`FATAL_JSONRPC_ERROR_CODE`]: Custom error code that can be returned by
///   plugin and modules to denote that a fatal error has occurred, and this
///   message is not retryable.
/// - [`METHOD_NOT_FOUND_CODE`]: The plugin or module does not expose the method
///   that was attempted to be called. This indicates a bug in the plugin or
///   module.
/// - [`INVALID_PARAMS_CODE`]: The custom message sent to the plugin or module
///   could not be deserialized. This could either be due a bug in the plugin or
///   module (JSON serialization not roundtripping correctly) or a message that
///   was manually inserted into the queue via `/enqueue`.
pub fn error_object_to_queue_error(error: ErrorObject<'_>) -> QueueError {
    if error.code() == FATAL_JSONRPC_ERROR_CODE
        || error.code() == METHOD_NOT_FOUND_CODE
        || error.code() == INVALID_PARAMS_CODE
    {
        QueueError::Fatal(Box::new(error.into_owned()))
    } else {
        QueueError::Retry(Box::new(error.into_owned()))
    }
}

/// A message specific to a plugin.
///
/// This is used in [`Call`], [`Callback`], and [`Data`] to route messages to
/// plugins.
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
}

#[allow(async_fn_in_trait)]
pub trait ChainModule: ChainModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ChainModuleInfo) -> Result<Self, BoxDynError>;
}

#[allow(async_fn_in_trait)]
pub trait ConsensusModule: ConsensusModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ConsensusModuleInfo) -> Result<Self, BoxDynError>;
}

#[allow(async_fn_in_trait)]
pub trait ClientModule: ClientModuleServer + Sized {
    type Config: DeserializeOwned + Clone;

    async fn new(config: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError>;
}

#[derive(Debug, Clone)]
pub struct VoyagerClient(pub(crate) reconnecting_jsonrpc_ws_client::Client);

impl VoyagerClient {
    pub fn new(name: String, socket: String) -> Self {
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
        Self(client)
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

#[async_trait]
impl jsonrpsee::core::client::ClientT for VoyagerClient {
    async fn notification<Params>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<(), jsonrpsee::core::client::Error>
    where
        Params: ToRpcParams + Send,
    {
        self.0.notification(method, params).await
    }

    async fn request<R, Params>(
        &self,
        method: &str,
        params: Params,
    ) -> Result<R, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        self.0.request(method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, jsonrpsee::core::client::Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        self.0.batch_request(batch).await
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

fn must_parse<T: DeserializeOwned>(config_str: &str) -> T {
    match serde_json::from_str::<T>(config_str) {
        Ok(ok) => ok,
        Err(err) => {
            error!("invalid config: {}", ErrorReporter(err));
            std::process::exit(INVALID_CONFIG_EXIT_CODE as i32);
        }
    }
}

pub async fn run_plugin_server<T: Plugin>() {
    init_log();

    let app = <PluginApp<T::Cmd> as clap::Parser>::parse();

    match app {
        PluginApp::Run {
            socket,
            voyager_socket,
            config,
        } => {
            let config = must_parse::<T::Config>(&config);

            let info = T::info(config.clone());

            let name = info.name;

            run_server(
                name.clone(),
                voyager_socket,
                config,
                socket,
                T::new,
                T::into_rpc,
            )
            .instrument(debug_span!("run_plugin_server", %name))
            .await
        }
        PluginApp::Info { config } => {
            let info = T::info(must_parse(&config));

            print!("{}", serde_json::to_string(&info).unwrap())
        }
        PluginApp::Cmd { cmd, config } => T::cmd(must_parse(&config), cmd).await,
    }
}

pub async fn run_chain_module_server<T: ChainModule>() {
    init_log();

    match <ModuleApp as clap::Parser>::parse() {
        ModuleApp::Run {
            socket,
            voyager_socket,
            config,
            info,
        } => {
            let config = must_parse::<T::Config>(&config);

            let info = must_parse::<ChainModuleInfo>(&info);

            let name = format!("chain/{}", info.chain_id);

            run_server(
                name.clone(),
                voyager_socket,
                (config, info),
                socket,
                |(config, info)| T::new(config, info),
                T::into_rpc,
            )
            .instrument(debug_span!("run_chain_module_server", %name))
            .await
        }
    }
}

pub async fn run_consensus_module_server<T: ConsensusModule>() {
    init_log();

    match <ModuleApp as clap::Parser>::parse() {
        ModuleApp::Run {
            socket,
            voyager_socket,
            config,
            info,
        } => {
            let config = must_parse::<T::Config>(&config);

            let info = must_parse::<ConsensusModuleInfo>(&info);

            let name = format!("chain/{}", info.chain_id);

            run_server(
                name.clone(),
                voyager_socket,
                (config, info),
                socket,
                |(config, info)| T::new(config, info),
                T::into_rpc,
            )
            .instrument(debug_span!("run_consensus_module_server", %name))
            .await
        }
    }
}

pub async fn run_client_module_server<T: ClientModule>() {
    init_log();

    match <ModuleApp as clap::Parser>::parse() {
        ModuleApp::Run {
            socket,
            voyager_socket,
            config,
            info,
        } => {
            let config = must_parse::<T::Config>(&config);

            let info = must_parse::<ClientModuleInfo>(&info);

            let id = info.id();

            run_server(
                id.clone(),
                voyager_socket,
                (config, info),
                socket,
                |(config, info)| T::new(config, info),
                T::into_rpc,
            )
            .instrument(debug_span!("run_client_module_server", %id))
            .await
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
    let voyager_client = VoyagerClient::new(id.clone(), voyager_socket);
    if let Err(err) = voyager_client
        .0
        .wait_until_connected(Duration::from_millis(500))
        .await
    {
        error!("unable to connect to voyager socket: {err}");
        std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
    };

    debug!("connected to voyager socket");

    let module_server = match new(new_t).await {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("startup error: {err:?}");
            std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
        }
    };

    let ipc_server = reth_ipc::server::Builder::default()
        .set_rpc_middleware(
            RpcServiceBuilder::new().layer_fn(move |service| InjectClient {
                client: voyager_client.clone(),
                service,
            }),
        )
        .build(socket);

    let rpcs = into_rpc(module_server);

    trace!(methods = ?*rpcs, "registered methods");
    let addr = ipc_server.endpoint();
    let server_handle = ipc_server.start(rpcs).await.unwrap();
    info!("listening on {addr}");

    tokio::spawn(
        server_handle
            .stopped()
            .instrument(debug_span!("module_server", %id)),
    )
    .await
    .unwrap()
}

struct InjectClient<S> {
    client: VoyagerClient,
    service: S,
}

impl<'a, S: RpcServiceT<'a> + Send + Sync> RpcServiceT<'a> for InjectClient<S> {
    type Future = S::Future;

    fn call(&self, mut request: jsonrpsee::types::Request<'a>) -> Self::Future {
        request.extensions.insert(self.client.clone());
        self.service.call(request)
    }
}

// TODO: Deduplicate this (it's also in the cosmos-sdk chain module), probably put it in voyager-message
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
