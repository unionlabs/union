use std::{
    borrow::Cow,
    collections::HashMap,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use anyhow::anyhow;
use futures::{
    future,
    stream::{self, FuturesUnordered},
    Future, FutureExt, StreamExt, TryStreamExt,
};
use jsonrpsee::{
    core::{client::ClientT, RpcResult},
    server::middleware::rpc::RpcServiceT,
    types::{ErrorObject, ErrorObjectOwned},
};
use macros::model;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Map, Value};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{
    debug, debug_span, error, info, info_span, instrument, instrument::Instrumented, trace, warn,
    Instrument,
};
use unionlabs::{
    bytes::Bytes, ethereum::keccak256, hash::hash_v2::HexUnprefixed, traits::Member, ErrorReporter,
};
use voyager_core::{ConsensusType, IbcSpecId};
use voyager_vm::{ItemId, QueueError};

use crate::{
    core::{ChainId, ClientType, IbcInterface, IbcSpec},
    into_value,
    module::{
        ClientBootstrapModuleInfo, ClientModuleInfo, ConsensusModuleInfo, PluginClient, PluginInfo,
        ProofModuleInfo, StateModuleInfo,
    },
    rpc::{server::Server, VoyagerRpcServer},
    IdThreadClient, ParamsWithItemId, RawClientId, FATAL_JSONRPC_ERROR_CODE,
};

pub const INVALID_CONFIG_EXIT_CODE: u8 = 13;
pub const STARTUP_ERROR_EXIT_CODE: u8 = 14;

#[derive(macros::Debug)]
pub struct Context {
    pub rpc_server: Server,

    plugins: HashMap<String, ModuleRpcClient>,

    interest_filters: HashMap<String, String>,

    pub cancellation_token: CancellationToken,
}

#[derive(macros::Debug)]
pub struct Modules {
    state_modules: HashMap<(ChainId, IbcSpecId), ModuleRpcClient>,
    proof_modules: HashMap<(ChainId, IbcSpecId), ModuleRpcClient>,

    /// map of chain id to consensus module.
    consensus_modules: HashMap<ChainId, ModuleRpcClient>,

    client_modules: HashMap<(ClientType, IbcInterface, IbcSpecId), ModuleRpcClient>,

    client_bootstrap_modules: HashMap<(ChainId, ClientType), ModuleRpcClient>,

    chain_consensus_types: HashMap<ChainId, ConsensusType>,

    client_consensus_types: HashMap<ClientType, ConsensusType>,

    // ibc version id => handler
    #[debug(skip)]
    pub ibc_spec_handlers: IbcSpecHandlers,
}

pub struct IbcSpecHandlers {
    pub(crate) handlers: HashMap<IbcSpecId, IbcSpecHandler>,
}

impl IbcSpecHandlers {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::default(),
        }
    }

    pub fn register<S: IbcSpec>(&mut self) {
        self.handlers.insert(S::ID, IbcSpecHandler::new::<S>());
    }

    pub fn get(&self, ibc_spec_id: &IbcSpecId) -> RpcResult<&IbcSpecHandler> {
        self.handlers.get(ibc_spec_id).ok_or_else(|| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unknown IBC spec `{ibc_spec_id}`"),
                None::<()>,
            )
        })
    }
}

/// A type-erased version of the methods on [`IbcSpec`] (essentially a vtable).
pub struct IbcSpecHandler {
    pub client_state_path: fn(RawClientId) -> anyhow::Result<Value>,
    pub consensus_state_path: fn(RawClientId, String) -> anyhow::Result<Value>,
    pub msg_update_client: fn(RawClientId, Bytes) -> anyhow::Result<Value>,
}

impl IbcSpecHandler {
    pub const fn new<T: IbcSpec>() -> Self {
        Self {
            client_state_path: |client_id| {
                Ok(into_value(T::client_state_path(serde_json::from_value(
                    client_id.0,
                )?)))
            },
            consensus_state_path: |client_id, height| {
                Ok(into_value(T::consensus_state_path(
                    serde_json::from_value(client_id.0)?,
                    height.parse()?,
                )))
            },
            msg_update_client: |client_id, client_message| {
                Ok(into_value(T::update_client_datagram(
                    serde_json::from_value(client_id.0)?,
                    client_message,
                )))
            },
        }
    }
}

impl voyager_vm::ContextT for Context {}

#[derive(macros::Debug, Clone)]
pub struct ModuleRpcClient {
    #[debug(skip)]
    client: reconnecting_jsonrpc_ws_client::Client,
    #[allow(dead_code)]
    name: String,
}

impl ModuleRpcClient {
    fn new(name: &str) -> Self {
        let socket = Self::make_socket_path(name);

        let client = reconnecting_jsonrpc_ws_client::Client::new({
            // NOTE: This needs to be leaked because the return type of the .build() method
            // below captures the lifetime of the `name` parameter(?)
            let socket: &'static str = Box::leak(socket.clone().into_boxed_str());
            let name = name.to_owned();
            move || {
                async move {
                    trace!("connecting to socket at {socket}");
                    reth_ipc::client::IpcClientBuilder::default()
                        .build(socket)
                        .await
                }
                .instrument(debug_span!("module_ipc_client", %name))
            }
        });

        Self {
            client,
            name: name.to_owned(),
        }
    }

    fn make_socket_path(name: &str) -> String {
        let pid = std::process::id();

        format!(
            "/tmp/voyager-to-module-{}.sock",
            keccak256(
                name.as_bytes()
                    .iter()
                    .chain(pid.to_be_bytes().iter())
                    .copied()
                    .collect::<Vec<_>>()
            )
            .into_encoding::<HexUnprefixed>()
        )
    }

    // pub fn client(&self) -> &impl jsonrpsee::core::client::ClientT {
    //     &self.client
    // }

    pub fn client(&self) -> &reconnecting_jsonrpc_ws_client::Client {
        &self.client
    }
}

pub(crate) trait WithId: Sized + ClientT + Send + Sync
where
    for<'a> &'a Self: ClientT,
{
    fn with_id(&self, item_id: Option<ItemId>) -> IdThreadClient<&Self> {
        IdThreadClient {
            client: self,
            item_id,
        }
    }
}

impl<T: ClientT + Send + Sync> WithId for T where for<'a> &'a Self: ClientT {}

async fn module_rpc_server(name: &str, server: Server) -> anyhow::Result<impl Future<Output = ()>> {
    let socket = make_module_rpc_server_socket_path(name);
    let rpc_server = reth_ipc::server::Builder::default()
        .set_rpc_middleware(
            reth_ipc::server::RpcServiceBuilder::new()
                .layer_fn(|service| ExtractItemId { service }),
        )
        .build(socket.clone());

    debug!(%socket, "starting rpc server");

    let server = rpc_server.start(server.into_rpc()).await?;

    Ok(server
        .stopped()
        .instrument(debug_span!("module_rpc_server", %name)))
}

pub struct ExtractItemId<S> {
    service: S,
}

impl<'a, S: RpcServiceT<'a>> RpcServiceT<'a> for ExtractItemId<S> {
    type Future = futures::future::Either<Instrumented<S::Future>, S::Future>;

    fn call(&self, mut request: jsonrpsee::types::Request<'a>) -> Self::Future {
        if let Some(params) = request.params.take() {
            match serde_json::from_str(params.get()) {
                Ok(ParamsWithItemId { item_id, params }) => {
                    let mut request = jsonrpsee::types::Request {
                        params: params.map(|rv| Cow::Owned(rv.into_owned())),
                        ..request
                    };

                    request.extensions.insert(item_id);

                    return self
                        .service
                        .call(request)
                        .instrument(info_span!("item_id", item_id = item_id.raw()))
                        .left_future();
                }
                Err(_) => {
                    request.params = Some(params);
                }
            }
        };

        self.service.call(request).right_future()
    }
}

fn make_module_rpc_server_socket_path(name: &str) -> String {
    let pid = std::process::id();

    format!(
        "/tmp/module-to-voyager-{}.sock",
        keccak256(
            name.as_bytes()
                .iter()
                .chain(pid.to_be_bytes().iter())
                .copied()
                .collect::<Vec<_>>()
        )
        .into_encoding::<HexUnprefixed>()
    )
}

#[model]
#[derive(Hash, JsonSchema)]
pub struct PluginConfig {
    pub path: PathBuf,
    pub config: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[model]
#[derive(JsonSchema)]
pub struct ModulesConfig {
    pub state: Vec<ModuleConfig<StateModuleInfo>>,
    pub proof: Vec<ModuleConfig<ProofModuleInfo>>,
    pub consensus: Vec<ModuleConfig<ConsensusModuleInfo>>,
    pub client: Vec<ModuleConfig<ClientModuleInfo>>,
    pub client_bootstrap: Vec<ModuleConfig<ClientBootstrapModuleInfo>>,
}

#[model]
#[derive(JsonSchema)]
pub struct ModuleConfig<T> {
    pub path: PathBuf,
    pub info: T,
    #[serde(default = "default_config")]
    pub config: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_config() -> Value {
    Value::Object(Map::new())
}

const fn default_enabled() -> bool {
    true
}

impl Context {
    #[instrument(name = "context_new", skip_all)]
    pub async fn new(
        plugin_configs: Vec<PluginConfig>,
        module_configs: ModulesConfig,
        register_ibc_spec_handlers: fn(&mut IbcSpecHandlers),
    ) -> anyhow::Result<Self> {
        let cancellation_token = CancellationToken::new();

        let mut ibc_spec_handlers = IbcSpecHandlers {
            handlers: Default::default(),
        };

        register_ibc_spec_handlers(&mut ibc_spec_handlers);

        let mut modules = Modules {
            state_modules: Default::default(),
            proof_modules: Default::default(),
            client_modules: Default::default(),
            client_bootstrap_modules: Default::default(),
            consensus_modules: Default::default(),
            chain_consensus_types: Default::default(),
            client_consensus_types: Default::default(),
            ibc_spec_handlers,
        };

        let mut plugins = HashMap::default();

        let mut interest_filters = HashMap::default();

        let main_rpc_server = Server::new();

        info!("spawning {} plugins", plugin_configs.len());

        stream::iter(plugin_configs)
            .filter(|plugin_config| {
                future::ready(if !plugin_config.enabled {
                    info!(
                        module_path = %plugin_config.path.to_string_lossy(),
                        "module is not enabled, skipping"
                    );
                    false
                } else {
                    true
                })
            })
            .zip(stream::repeat(main_rpc_server.clone()))
            .map(Ok)
            .try_filter_map(|(plugin_config, server)| async move {
                if !plugin_config.enabled {
                    info!(
                        module_path = %plugin_config.path.to_string_lossy(),
                        "module is not enabled, skipping"
                    );
                    Ok(None)
                } else {
                    let plugin_info = get_plugin_info(&plugin_config)?;

                    debug!("starting rpc server for plugin {}", plugin_info.name);
                    tokio::spawn(module_rpc_server(&plugin_info.name, server).await?);

                    Ok(Some((plugin_config, plugin_info)))
                }
            })
            .try_for_each_concurrent(
                None,
                |(
                    plugin_config,
                    PluginInfo {
                        name,
                        interest_filter,
                    },
                )| {
                    info!("registering plugin {}", name);

                    tokio::spawn(plugin_child_process(
                        name.clone(),
                        plugin_config.clone(),
                        cancellation_token.clone(),
                    ));

                    let rpc_client = ModuleRpcClient::new(&name);

                    let prev = plugins.insert(name.clone(), rpc_client.clone());

                    if prev.is_some() {
                        return future::ready(Err(anyhow!(
                            "multiple plugins configured with name `{name}`"
                        )));
                    }

                    info!("registered plugin {name}");

                    interest_filters.insert(name, interest_filter);

                    future::ready(Ok(()))
                },
            )
            .await?;

        module_startup(
            module_configs.state,
            cancellation_token.clone(),
            main_rpc_server.clone(),
            |info| info.id(),
            |StateModuleInfo {
                 chain_id,
                 ibc_spec_id,
             },
             rpc_client| {
                let prev = modules
                    .state_modules
                    .insert((chain_id.clone(), ibc_spec_id.clone()), rpc_client);

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple state modules configured for chain id \
                        `{chain_id}` and IBC version `{ibc_spec_id}`",
                    ));
                }

                Ok(())
            },
        )
        .await?;

        module_startup(
            module_configs.proof,
            cancellation_token.clone(),
            main_rpc_server.clone(),
            |info| info.id(),
            |ProofModuleInfo {
                 chain_id,
                 ibc_spec_id,
             },
             rpc_client| {
                let prev = modules
                    .proof_modules
                    .insert((chain_id.clone(), ibc_spec_id.clone()), rpc_client);

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple proof modules configured for chain id \
                        `{chain_id}` and IBC version `{ibc_spec_id}`",
                    ));
                }

                Ok(())
            },
        )
        .await?;

        module_startup(
            module_configs.consensus,
            cancellation_token.clone(),
            main_rpc_server.clone(),
            |info| info.id(),
            |ConsensusModuleInfo {
                 chain_id,
                 consensus_type,
             },
             rpc_client| {
                let prev = modules
                    .consensus_modules
                    .insert(chain_id.clone(), rpc_client);

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple consensus modules configured for chain id `{}`",
                        chain_id
                    ));
                }

                let None = modules
                    .chain_consensus_types
                    .insert(chain_id.clone(), consensus_type.clone())
                else {
                    unreachable!()
                };

                Ok(())
            },
        )
        .await?;

        module_startup(
            module_configs.client,
            cancellation_token.clone(),
            main_rpc_server.clone(),
            |info| info.id(),
            |ClientModuleInfo {
                 client_type,
                 consensus_type,
                 ibc_interface,
                 ibc_spec_id,
             },
             rpc_client| {
                if !modules.ibc_spec_handlers.handlers.contains_key(ibc_spec_id) {
                    return Err(anyhow!(
                        "IBC version `{ibc_spec_id}` is not supported in this build of voyager"
                    ));
                }

                let prev = modules.client_modules.insert(
                    (
                        client_type.clone(),
                        ibc_interface.clone(),
                        ibc_spec_id.clone(),
                    ),
                    rpc_client.clone(),
                );

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple client modules configured for client \
                        type `{client_type}`, IBC interface `{ibc_interface}`, \
                        and IBC version `{ibc_spec_id}`",
                    ));
                }

                if let Some(previous_consensus_type) = modules
                    .client_consensus_types
                    .insert(client_type.clone(), consensus_type.clone())
                {
                    if previous_consensus_type != consensus_type {
                        return Err(anyhow!(
                            "inconsistency in client consensus types: \
                            client type `{client_type}` is registered \
                            as tracking both `{previous_consensus_type}` \
                            and `{consensus_type}`"
                        ));
                    }
                }

                Ok(())
            },
        )
        .await?;

        module_startup(
            module_configs.client_bootstrap,
            cancellation_token.clone(),
            main_rpc_server.clone(),
            |info| info.id(),
            |ClientBootstrapModuleInfo {
                 client_type,
                 chain_id,
             },
             rpc_client| {
                let prev = modules
                    .client_bootstrap_modules
                    .insert((chain_id.clone(), client_type.clone()), rpc_client.clone());

                if prev.is_some() {
                    return Err(anyhow!(
                        "multiple client bootstrap modules configured for client \
                        type `{client_type}` and chain id `{chain_id}`",
                    ));
                }

                // TODO: Check consistency with client_consensus_types and chain_id?

                // if let Some(previous_consensus_type) = modules
                //     .client_consensus_types
                //     .insert(client_type.clone(), consensus_type.clone())
                // {
                //     if previous_consensus_type != consensus_type {
                //         return Err(anyhow!(
                //             "inconsistency in client consensus types: \
                //             client type `{client_type}` is registered \
                //             as tracking both `{previous_consensus_type}` \
                //             and `{consensus_type}`"
                //         ));
                //     }
                // }

                Ok(())
            },
        )
        .await?;

        main_rpc_server.start(Arc::new(modules));

        info!("checking for plugin health...");

        {
            let mut futures = plugins
                .iter()
                .map(|(name, client)| async move {
                    match client
                        .client
                        .wait_until_connected(Duration::from_secs(10))
                        .instrument(debug_span!("health check", %name))
                        .await
                    {
                        Ok(_) => {
                            info!("plugin {name} connected")
                        }
                        Err(_) => {
                            warn!("plugin {name} failed to connect after 10 seconds")
                        }
                    }
                })
                .collect::<FuturesUnordered<_>>();

            match cancellation_token
                .run_until_cancelled(async { while let Some(()) = futures.next().await {} })
                .await
            {
                Some(()) => {}
                None => return Err(anyhow!("startup error")),
            }
        }

        Ok(Self {
            rpc_server: main_rpc_server,
            plugins,
            interest_filters,
            cancellation_token,
        })
    }

    pub async fn shutdown(self) {
        self.cancellation_token.cancel();

        for (name, client) in self.plugins {
            debug!("shutting down plugin client for {name}");
            client.client.shutdown();
        }
    }

    pub fn plugin(
        &self,
        name: impl AsRef<str>,
    ) -> Result<&(impl PluginClient<Value, Value> + '_), PluginNotFound> {
        Ok(self
            .plugins
            .get(name.as_ref())
            .ok_or_else(|| PluginNotFound {
                name: name.as_ref().into(),
            })?
            .client())
    }

    pub fn plugin_client_raw(
        &self,
        name: impl AsRef<str>,
    ) -> Result<&ModuleRpcClient, PluginNotFound> {
        self.plugins
            .get(name.as_ref())
            .ok_or_else(|| PluginNotFound {
                name: name.as_ref().into(),
            })
    }

    pub fn interest_filters(&self) -> &HashMap<String, String> {
        &self.interest_filters
    }
}

impl Modules {
    pub fn info(&self) -> LoadedModulesInfo {
        let state = self
            .state_modules
            .keys()
            .cloned()
            .map(|(chain_id, ibc_spec_id)| StateModuleInfo {
                chain_id,
                ibc_spec_id,
            })
            .collect();

        let proof = self
            .proof_modules
            .keys()
            .cloned()
            .map(|(chain_id, ibc_spec_id)| ProofModuleInfo {
                chain_id,
                ibc_spec_id,
            })
            .collect();

        let consensus = self
            .consensus_modules
            .keys()
            .cloned()
            .map(|chain_id| ConsensusModuleInfo {
                consensus_type: self.chain_consensus_types[&chain_id].clone(),
                chain_id,
            })
            .collect();

        let client = self
            .client_modules
            .keys()
            .map(
                |(client_type, ibc_interface, ibc_spec_id)| ClientModuleInfo {
                    consensus_type: self.client_consensus_types[client_type].clone(),
                    client_type: client_type.clone(),
                    ibc_interface: ibc_interface.clone(),
                    ibc_spec_id: ibc_spec_id.clone(),
                },
            )
            .collect();

        let client_bootstrap = self
            .client_bootstrap_modules
            .keys()
            .map(|(chain_id, client_type)| ClientBootstrapModuleInfo {
                client_type: client_type.clone(),
                chain_id: chain_id.clone(),
            })
            .collect();

        LoadedModulesInfo {
            state,
            proof,
            consensus,
            client,
            client_bootstrap,
        }
    }

    pub fn chain_consensus_type<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
    ) -> Result<&'a ConsensusType, ConsensusModuleNotFound> {
        self.chain_consensus_types
            .get(chain_id)
            .ok_or_else(|| ConsensusModuleNotFound(chain_id.clone()))
    }

    pub fn client_consensus_type<'a, 'b, 'c: 'a>(
        &'a self,
        client_type: &ClientType,
    ) -> Result<&'a ConsensusType, ClientModuleNotFound> {
        self.client_consensus_types.get(client_type).ok_or_else(|| {
            ClientModuleNotFound::ClientTypeNotFound {
                client_type: client_type.clone(),
            }
        })
    }

    pub fn state_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        // ) -> Result<&'a (impl RawStateModuleClient + 'a), StateModuleNotFound> {
    ) -> Result<&'a reconnecting_jsonrpc_ws_client::Client, StateModuleNotFound> {
        Ok(self
            .state_modules
            .get(&(chain_id.clone(), ibc_spec_id.clone()))
            .ok_or_else(|| StateModuleNotFound {
                chain_id: chain_id.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            })?
            .client())
    }

    pub fn proof_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
        // ) -> Result<&'a (impl RawProofModuleClient + 'a), ProofModuleNotFound> {
    ) -> Result<&'a reconnecting_jsonrpc_ws_client::Client, ProofModuleNotFound> {
        Ok(self
            .proof_modules
            .get(&(chain_id.clone(), ibc_spec_id.clone()))
            .ok_or_else(|| ProofModuleNotFound {
                chain_id: chain_id.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            })?
            .client())
    }

    pub fn consensus_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        // ) -> Result<&'a (impl jsonrpsee::core::client::ClientT + 'a), ConsensusModuleNotFound> {
    ) -> Result<&'a reconnecting_jsonrpc_ws_client::Client, ConsensusModuleNotFound> {
        Ok(self
            .consensus_modules
            .get(chain_id)
            .ok_or_else(|| ConsensusModuleNotFound(chain_id.clone()))?
            .client())
    }

    pub fn client_module<'a, 'b, 'c: 'a>(
        &'a self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
        // ) -> Result<&'a (impl ClientModuleClient + 'a), ClientModuleNotFound> {
    ) -> Result<&'a reconnecting_jsonrpc_ws_client::Client, ClientModuleNotFound> {
        match self.client_modules.get(&(
            client_type.clone(),
            ibc_interface.clone(),
            ibc_spec_id.clone(),
        )) {
            Some(client_module) => Ok(client_module.client()),
            None => Err(ClientModuleNotFound::NotFound {
                client_type: client_type.clone(),
                ibc_interface: ibc_interface.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            }),
        }
    }

    pub fn client_bootstrap_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        client_type: &ClientType,
        // ) -> Result<&'a (impl jsonrpsee::core::client::ClientT + 'a), ConsensusModuleNotFound> {
    ) -> Result<&'a reconnecting_jsonrpc_ws_client::Client, ClientBootstrapModuleNotFound> {
        Ok(self
            .client_bootstrap_modules
            .get(&(chain_id.clone(), client_type.clone()))
            .ok_or_else(|| ClientBootstrapModuleNotFound {
                chain_id: chain_id.clone(),
                client_type: client_type.clone(),
            })?
            .client())
    }
}

#[model]
pub struct LoadedModulesInfo {
    pub state: Vec<StateModuleInfo>,
    pub proof: Vec<ProofModuleInfo>,
    pub consensus: Vec<ConsensusModuleInfo>,
    pub client: Vec<ClientModuleInfo>,
    pub client_bootstrap: Vec<ClientBootstrapModuleInfo>,
}

#[instrument(skip_all, fields(%name))]
async fn plugin_child_process(
    name: String,
    module_config: PluginConfig,
    cancellation_token: CancellationToken,
) {
    let client_socket = ModuleRpcClient::make_socket_path(&name);
    let server_socket = make_module_rpc_server_socket_path(&name);

    info!(%client_socket, %server_socket, "starting plugin {name}");

    let mut cmd = tokio::process::Command::new(&module_config.path);
    cmd.arg("run");
    cmd.arg(&client_socket);
    cmd.arg(&server_socket);
    cmd.arg(module_config.config.to_string());

    lazarus_pit(
        &module_config.path,
        &[
            "run",
            &client_socket,
            &server_socket,
            &module_config.config.to_string(),
        ],
        cancellation_token,
    )
    .await
}

#[instrument(skip_all, fields(%name))]
async fn module_child_process<Info: Serialize>(
    name: String,
    module_config: ModuleConfig<Info>,
    cancellation_token: CancellationToken,
) {
    let client_socket = ModuleRpcClient::make_socket_path(&name);
    let server_socket = make_module_rpc_server_socket_path(&name);

    info!(%client_socket, %server_socket, "starting module {name}");

    lazarus_pit(
        &module_config.path,
        &[
            "run",
            &client_socket,
            &server_socket,
            &module_config.config.to_string(),
            &serde_json::to_string(&module_config.info).unwrap(),
        ],
        cancellation_token,
    )
    .await
}

async fn lazarus_pit(cmd: &Path, args: &[&str], cancellation_token: CancellationToken) {
    let mut attempt = 0;

    loop {
        let mut cmd = tokio::process::Command::new(cmd);
        cmd.args(args);

        debug!(%attempt, "spawning plugin child process");

        let mut child = loop {
            match cmd.spawn() {
                Ok(child) => {
                    let id = child.id().unwrap();

                    debug!(%id, "spawned plugin");

                    break child;
                }
                Err(err) => {
                    error!(
                        err = %ErrorReporter(err),
                        "unable to spawn plugin"
                    );

                    sleep(Duration::from_secs(1)).await;
                }
            }
        };

        let id = child.id().unwrap();

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                debug!(%id, "killing plugin");
                match child.kill().await {
                    Ok(()) => {
                        debug!(%id, "plugin received kill signal");
                        match child.wait().await {
                            Ok(exit_status) => {
                                debug!(%id, %exit_status, "child exited successfully")
                            }
                            Err(err) => {
                                error!(%id, err = %ErrorReporter(err), "child exited unsuccessfully")
                            }
                        }
                    }
                    Err(err) => {
                        error!(%id, err = %ErrorReporter(err), "unable to kill plugin")
                    }
                }

                break
            }
            res = child.wait() => {
                match res {
                    Ok(exit_status) => {
                        info!(%id, %exit_status, "child exited");

                        if exit_status
                            .code()
                            .is_some_and(|c| c == INVALID_CONFIG_EXIT_CODE as i32)
                        {
                            error!(%id, "invalid config for plugin");
                            cancellation_token.cancel();
                        }
                    }
                    Err(err) => {
                        error!(%id, err = %ErrorReporter(err), "child exited");
                    }
                }

                // TODO: Exponential backoff
                sleep(Duration::from_secs(1)).await;
            }
        }

        attempt += 1;
    }
}

macro_rules! module_error {
    ($Error:ident) => {
        impl From<$Error> for QueueError {
            fn from(value: $Error) -> Self {
                Self::Fatal(Box::new(value))
            }
        }

        impl From<$Error> for ErrorObjectOwned {
            fn from(value: $Error) -> Self {
                ErrorObject::owned(FATAL_JSONRPC_ERROR_CODE, value.to_string(), None::<()>)
            }
        }

        impl From<$Error> for jsonrpsee::core::client::Error {
            fn from(value: $Error) -> Self {
                ErrorObject::from(value).into()
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for state on chain `{chain_id}` and IBC version `{ibc_spec_id}`")]
pub struct StateModuleNotFound {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

module_error!(StateModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for proofs on chain `{chain_id}` and IBC version `{ibc_spec_id}`")]
pub struct ProofModuleNotFound {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

module_error!(ProofModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for consensus on chain `{0}`")]
pub struct ConsensusModuleNotFound(pub ChainId);

module_error!(ConsensusModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for client bootstrapping on chain `{chain_id}` for client type `{client_type}`")]
pub struct ClientBootstrapModuleNotFound {
    pub chain_id: ChainId,
    pub client_type: ClientType,
}

module_error!(ClientBootstrapModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ClientModuleNotFound {
    #[error("no client module loaded for client type `{}`", client_type)]
    ClientTypeNotFound { client_type: ClientType },
    #[error(
        "no client module loaded supporting client type `{client_type}`, IBC interface `{ibc_interface}`, and IBC version `{ibc_spec_id}`",
    )]
    NotFound {
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
    },
}

module_error!(ClientModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("plugin `{name}` not found")]
pub struct PluginNotFound {
    pub name: String,
}

module_error!(PluginNotFound);

pub fn get_plugin_info(module_config: &PluginConfig) -> anyhow::Result<PluginInfo> {
    debug!(
        "querying module info from plugin at {}",
        &module_config.path.to_string_lossy(),
    );

    let mut cmd = std::process::Command::new(&module_config.path);
    cmd.arg("info");
    cmd.arg(module_config.config.to_string());

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();

    if !output.status.success() {
        match output.status.code() {
            Some(code) if code == INVALID_CONFIG_EXIT_CODE as i32 => {
                return Err(anyhow!(
                    "invalid config for module at path {}:\n{}",
                    &module_config.path.to_string_lossy(),
                    String::from_utf8_lossy(&output.stdout)
                ));
            }
            Some(_) | None => {
                return Err(anyhow!(
                    "unable to query info for module at path {}:\n{}",
                    &module_config.path.to_string_lossy(),
                    String::from_utf8_lossy(&output.stdout)
                ))
            }
        }
    }

    trace!("plugin stdout: {}", String::from_utf8_lossy(&output.stdout));

    Ok(serde_json::from_slice(&output.stdout).unwrap())
}

async fn module_startup<Info: Serialize + Clone + Unpin + Send + 'static>(
    configs: Vec<ModuleConfig<Info>>,
    cancellation_token: CancellationToken,
    main_rpc_server: Server,
    id_f: fn(&Info) -> String,
    mut push_f: impl FnMut(&Info, ModuleRpcClient) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    stream::iter(configs)
        .filter(|module_config| {
            future::ready(if !module_config.enabled {
                info!(
                    module_path = %module_config.path.to_string_lossy(),
                    "module is not enabled, skipping"
                );
                false
            } else {
                true
            })
        })
        .zip(stream::repeat(main_rpc_server.clone()))
        .map::<anyhow::Result<_>, _>(anyhow::Result::Ok)
        .try_filter_map(|(module_config, server)| async move {
            if !module_config.enabled {
                info!(
                    module_path = %module_config.path.to_string_lossy(),
                    "module is not enabled, skipping"
                );
                anyhow::Result::Ok(None)
            } else {
                debug!(
                    "starting rpc server for module {}",
                    id_f(&module_config.info)
                );
                tokio::spawn(module_rpc_server(&id_f(&module_config.info), server).await?);

                anyhow::Result::Ok(Some(module_config))
            }
        })
        .try_collect::<FuturesUnordered<_>>()
        .await?
        .into_iter()
        .try_for_each(|module_config| {
            let id = id_f(&module_config.info);

            info!("registering module {}", id);

            tokio::spawn(module_child_process(
                id.clone(),
                module_config.clone(),
                cancellation_token.clone(),
            ));

            let rpc_client = ModuleRpcClient::new(&id);

            push_f(&module_config.info, rpc_client)?;

            info!("registered module {id}");

            Ok(())
        })
}
