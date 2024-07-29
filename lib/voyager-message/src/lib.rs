#![feature(trait_alias)]

use std::{env::VarError, fmt::Debug, marker::PhantomData, time::Duration};

use chain_utils::BoxDynError;
use jsonrpsee::types::{error::METHOD_NOT_FOUND_CODE, ErrorObject};
use macros::apply;
use queue_msg::{aggregation::SubsetOf, queue_msg, QueueError, QueueMessage};
use reth_ipc::client::IpcClientBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
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

/// Error code for fatal errors. If a plugin responds with this error code, it will be treated as failed and not retried.
pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

pub fn json_rpc_error_to_queue_error(error: jsonrpsee::core::client::Error) -> QueueError {
    match error {
        jsonrpsee::core::client::Error::Call(error) => error_object_to_queue_error(error),
        value => QueueError::Retry(Box::new(value)),
    }
}

pub fn error_object_to_queue_error(error: ErrorObject<'_>) -> QueueError {
    if error.code() == FATAL_JSONRPC_ERROR_CODE || error.code() == METHOD_NOT_FOUND_CODE {
        QueueError::Fatal(Box::new(error.into_owned()))
    } else {
        QueueError::Retry(Box::new(error.into_owned()))
    }
}

macro_rules! str_newtype {
    (
        $(#[doc = $doc:literal])+
        $vis:vis struct $Struct:ident;
    ) => {
        #[derive(macros::Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        // I tested this and apparently it's not required (newtype is automatically transparent?) but
        // keeping it here for clarity
        #[serde(transparent)]
        #[debug("{}({:?})", stringify!($Struct), self.0)]
        $vis struct $Struct<'a>(#[doc(hidden)] ::std::borrow::Cow<'a, str>);

        impl<'a> ::core::fmt::Display for $Struct<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        #[allow(unused)]
        impl $Struct<'static> {
            pub const fn new_static(ibc_interface: &'static str) -> Self {
                Self(::std::borrow::Cow::Borrowed(ibc_interface))
            }
        }


        #[allow(unused)]
        impl<'a> $Struct<'a> {
            pub fn new(s: impl Into<::std::borrow::Cow<'a, str>>) -> Self {
                Self(s.into())
            }

            pub fn into_static(self) -> $Struct<'static> {
                $Struct(match self.0 {
                    ::std::borrow::Cow::Borrowed(x) => ::std::borrow::Cow::Owned(x.to_owned()),
                    ::std::borrow::Cow::Owned(x) => ::std::borrow::Cow::Owned(x),
                })
            }

            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }


            /// Borrow this [`
            #[doc = stringify!($Struct)]
            /// `], returning a new owned value pointing to the same data.
            ///
            /// ```
            #[doc = concat!("let t = ", stringify!($Struct), "::new_static(\"static\");")]
            ///
            /// takes_ownership(t.borrow());
            /// takes_ownership(t);
            ///
            #[doc = concat!("fn takes_ownership<'a>(c: ", stringify!($Struct), "<'a>) {}")]
            /// ```
            pub fn borrow<'b>(&'a self) -> $Struct<'b>
            where
                'a: 'b,
            {
                use std::borrow::Cow;

                match self.0 {
                    Cow::Borrowed(s) => Self(Cow::Borrowed(s)),
                    Cow::Owned(ref s) => Self(Cow::Borrowed(s.as_str())),
                }
            }
        }
    };
}

/// Represents the IBC interface of a chain. Since multiple chains with
/// different consensus mechanisms can have the same execution environment, this
/// value is used to describe how the IBC state is stored on-chain and how the
/// IBC stack is to be interacted with.
#[apply(str_newtype)]
pub struct IbcInterface;

/// Well-known IBC interfaces, defined as constants for reusability and to allow
/// for pattern matching.
impl IbcInterface<'static> {
    /// Native light clients in ibc-go, through the client v1 router. This
    /// entrypoint uses protobuf [`Any`] wrapping to route to the correct
    /// module, such as "/ibc.lightclients.tendermint.v1.ClientState" for native
    /// 07-tendermint clients.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    pub const IBC_GO_V8_NATIVE: &'static str = "ibc-go-v8/native";

    /// 08-wasm light clients in ibc-go, through the client v1 router. Similar
    /// to the ibc-go-v8/native entrypoint, this module also uses [`Any`]
    /// wrapping for client routing, however, there is another level of
    /// indirection, since the `Any` routing only routes to the wasm module. All
    /// state for wasm clients is [wrapped](wasm-protos), with the internal
    /// state being opaque bytes to be interpreted by the light client.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    /// [wasm-protos]: https://github.com/cosmos/ibc-go/blob/release/v8.4.x/proto/ibc/lightclients/wasm/v1/wasm.proto
    pub const IBC_GO_V8_08_WASM: &'static str = "ibc-go-v8/08-wasm";

    /// Solidity light clients, run via Union's IBC solidity stack. This stack
    /// is fully virtualized in the EVM, and as such can be run on any chain
    /// running the EVM as part of their execution layer (ethereum, ethereum
    /// L2s, berachain, etc).
    pub const IBC_SOLIDITY: &'static str = "ibc-solidity";

    pub const IBC_MOVE_APTOS: &'static str = "ibc-move/aptos";

    // lots more to come - near, fuel - stay tuned
}

/// Newtype for client types. Clients of the same type have the same client
/// state, consensus state, and header (client update) types.
#[apply(str_newtype)]
pub struct ClientType;

/// Well-known client types, defined as constants for reusability and to allow
/// for pattern matching.
impl ClientType<'static> {
    /// A client tracking CometBLS consensus.
    pub const COMETBLS: &'static str = "cometbls";

    /// A client tracking vanilla Tendermint (CometBFT).
    pub const TENDERMINT: &'static str = "tendermint";

    /// A client tracking the Ethereum beacon chain consensus, with the mainnet
    /// configuration.
    pub const ETHEREUM_MAINNET: &'static str = "ethereum-mainnet";

    /// A client tracking the Ethereum beacon chain consensus, with the minimal
    /// configuration.
    pub const ETHEREUM_MINIMAL: &'static str = "ethereum-minimal";

    /// A client tracking the state of the Scroll zkevm L2, settling on
    /// Ethereum.
    pub const SCROLL: &'static str = "scroll";

    /// A client tracking the state of the Arbitrum optimistic L2, settling on
    /// Ethereum.
    pub const ARBITRUM: &'static str = "arbitrum";

    /// A client tracking the state of a BeaconKit chain.
    pub const BEACON_KIT: &'static str = "beacon-kit";

    /// A client tracking the state of a Movement chain.
    pub const MOVEMENT: &'static str = "movement";

    // lots more to come - near, linea, polygon - stay tuned
}

/// Identifier used to uniquely identify the chain, as provided by the chain itself.
///
/// # Examples
///
/// 1 -> ethereum mainnet
/// 11155111 -> ethereum sepolia testnet
/// union-testnet-8 -> union testnet
/// stargaze-1 -> stargaze mainnet
#[apply(str_newtype)]
pub struct ChainId;

#[queue_msg]
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

pub async fn default_subcommand_handler<T>(_: T, cmd: DefaultCmd) -> Result<(), Never> {
    match cmd {}
}

pub fn init_log() {
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
pub struct ModuleServer<Ctx: ModuleContext> {
    pub voyager_rpc_client: reconnecting_jsonrpc_ws_client::Client,
    pub ctx: Ctx,
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

pub async fn run_module_server<T, D: Member, C: Member, Cb: Member>()
where
    T: ModuleContext,
    (T::Info, T): IntoRpc<D, C, Cb, RpcModule = ModuleServer<T>>,
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

            let ModuleInfo { name, kind: _ } = T::info(config.clone());

            let name_ = name.clone();
            async move {
                let voyager_rpc_client = reconnecting_jsonrpc_ws_client::Client::new({
                    let voyager_socket: &'static str = voyager_socket.leak();
                    let name_ = name_.clone();
                    move || {
                        async move {
                            debug!("connecting to socket at {voyager_socket}");
                            IpcClientBuilder::default().build(voyager_socket).await
                        }
                        .instrument(debug_span!("voyager_ipc_client", name = %name_))
                    }
                });

                if let Err(err) = voyager_rpc_client
                    .wait_until_connected(Duration::from_millis(500))
                    .await
                {
                    error!("unable to connect to voyager socket: {err}");
                    std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
                };

                info!("connected to voyager socket");

                let module_server = match T::new(config).await {
                    Ok(ctx) => ModuleServer {
                        voyager_rpc_client,
                        ctx,
                    },
                    Err(err) => {
                        error!("startup error: {err:?}");
                        std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
                    }
                };

                let ipc_server = reth_ipc::server::Builder::default().build(socket);

                let addr = ipc_server.endpoint();

                let rpcs = <(T::Info, T)>::into_rpc(module_server);

                trace!(methods = ?*rpcs, "registered methods");

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
                name: info.name,
                kind: info.kind.into(),
            };

            print!("{}", serde_json::to_string(&info).unwrap())
        }
        App::Cmd { cmd, config } => T::cmd(parse_config(&config), cmd).await,
    }
}
