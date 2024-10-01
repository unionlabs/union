#![feature(trait_alias)]

use std::{env::VarError, fmt::Debug, marker::PhantomData, time::Duration};

use chain_utils::BoxDynError;
use jsonrpsee::{
    core::{client::BatchResponse, params::BatchRequestBuilder, traits::ToRpcParams, RpcResult},
    server::middleware::rpc::RpcServiceT,
    types::{
        error::{INVALID_PARAMS_CODE, METHOD_NOT_FOUND_CODE},
        ErrorObject,
    },
    Extensions,
};
use macros::model;
use queue_msg::{aggregation::SubsetOf, QueueError, QueueMessage};
use reth_ipc::{client::IpcClientBuilder, server::RpcServiceBuilder};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tonic::async_trait;
use tracing::{debug, debug_span, error, info, trace, Instrument};
use unionlabs::{never::Never, traits::Member, ErrorReporter};

use crate::{
    call::Call,
    callback::Callback,
    context::{Context, INVALID_CONFIG_EXIT_CODE, STARTUP_ERROR_EXIT_CODE},
    data::Data,
    module::{IModuleKindInfo, IntoRpc, ModuleInfo, ModuleKindInfo},
};

pub mod call;
pub mod callback;
pub mod data;

pub mod context;
pub mod module;
pub mod pass;

pub mod rpc;

pub use reconnecting_jsonrpc_ws_client;
pub use reth_ipc;
pub use voyager_core as core;

pub struct VoyagerMessage<D = Value, F = Value, A = Value> {
    #[allow(clippy::type_complexity)] // it's a phantom data bro fight me
    __marker: PhantomData<fn() -> (D, F, A)>,
    __unconstructable: Never,
}

impl<D: Member, C: Member, Cb: Member> QueueMessage for VoyagerMessage<D, C, Cb> {
    type Call = Call<C>;
    type Data = Data<D>;
    type Callback = Callback<Cb>;

    type Context = Context;
}

/// Error code for fatal errors. If a plugin or module responds with this error
/// code, it will be treated as failed and not retried.
pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

/// Convert a [`jsonrpsee::core::client::Error`] to a `queue-msg`
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

/// Convert a `jsonrpsee` [`ErrorObject`] to a `queue-msg` [`QueueError`].
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
///   was manually inserted into the queue via `/msg`.
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
pub struct PluginMessage<T = serde_json::Value> {
    pub plugin: String,
    pub message: T,
}

impl<T, U> SubsetOf<Data<T>> for PluginMessage<U>
where
    U: SubsetOf<T>,
{
    fn try_from_super(data: Data<T>) -> Result<Self, Data<T>> {
        match data {
            Data::Plugin(PluginMessage { plugin, message }) => match U::try_from_super(message) {
                Ok(message) => Ok(PluginMessage { plugin, message }),
                Err(message) => Err(Data::plugin(plugin, message)),
            },
            data => Err(data),
        }
    }

    fn into_super(self) -> Data<T> {
        Data::<T>::plugin(self.plugin, self.message.into_super())
    }
}

macro_rules! top_level_identifiable_enum {
    (
        $(#[$meta:meta])*
        pub enum $Enum:ident$(<$Inner:ident = serde_json::Value>)? {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident($VariantInner:ty$(,)?),
            )*
        }
    ) => {
        $(#[$meta])*
        #[allow(clippy::large_enum_variant)]
        pub enum $Enum$(<$Inner = serde_json::Value>)? {
            $(
                $(#[$inner_meta])*
                $Variant($VariantInner),
            )*
        }

        $(
            impl<$Inner> $Enum<$Inner> {
                pub fn plugin(plugin: impl Into<String>, message: impl Into<$Inner>) -> $Enum<$Inner> {
                    Self::Plugin(PluginMessage { plugin: plugin.into(), message: message.into() }).into()
                }
            }
        )*
    };
}
pub(crate) use top_level_identifiable_enum;

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
pub trait ModuleContext: Sized {
    type Config: DeserializeOwned + Clone;
    type Cmd: clap::Subcommand;
    type Info: IModuleKindInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError>;

    fn info(config: Self::Config) -> ModuleInfo<Self::Info>;

    async fn cmd(config: Self::Config, cmd: Self::Cmd);
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
                    debug!("connecting to socket at {voyager_socket}");
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
enum App<Cmd: clap::Subcommand> {
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

pub async fn run_module_server<T>()
where
    T: ModuleContext,
    (T::Info, T): IntoRpc<RpcModule = T>,
{
    init_log();

    let app = <App<T::Cmd> as clap::Parser>::parse();

    let parse_config = |config_str| match serde_json::from_str::<T::Config>(config_str) {
        Ok(ok) => ok,
        Err(err) => {
            error!("invalid config: {}", ErrorReporter(err));
            std::process::exit(INVALID_CONFIG_EXIT_CODE as i32);
        }
    };

    match app {
        App::Run {
            socket,
            voyager_socket,
            config,
        } => {
            let config = parse_config(&config);

            let ModuleInfo { kind } = T::info(config.clone());

            let name = kind.name();
            let name_ = name.clone();
            async move {
                let voyager_client = VoyagerClient::new(name_.clone(), voyager_socket);

                if let Err(err) = voyager_client
                    .0
                    .wait_until_connected(Duration::from_millis(500))
                    .await
                {
                    error!("unable to connect to voyager socket: {err}");
                    std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
                };

                info!("connected to voyager socket");

                let module_server = match T::new(config).await {
                    Ok(ctx) => ctx,
                    Err(err) => {
                        error!("startup error: {err:?}");
                        std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
                    }
                };

                let ipc_server = reth_ipc::server::Builder::default()
                    .set_rpc_middleware(RpcServiceBuilder::new().layer_fn(move |service| {
                        InjectClient {
                            client: voyager_client.clone(),
                            service,
                        }
                    }))
                    .build(socket);

                let rpcs = <(T::Info, T)>::into_rpc(module_server);

                trace!(methods = ?*rpcs, "registered methods");

                let addr = ipc_server.endpoint();

                let server_handle = ipc_server.start(rpcs).await.unwrap();

                info!("listening on {addr}");

                tokio::spawn(
                    server_handle
                        .stopped()
                        .instrument(debug_span!("module_server", name = %name_)),
                )
                .await
                .unwrap();
            }
            .instrument(debug_span!("run_module_server", %name))
            .await
        }
        App::Info { config } => {
            let info = T::info(parse_config(&config));

            let info = ModuleInfo::<ModuleKindInfo> {
                kind: info.kind.into(),
            };

            print!("{}", serde_json::to_string(&info).unwrap())
        }
        App::Cmd { cmd, config } => T::cmd(parse_config(&config), cmd).await,
    }
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
