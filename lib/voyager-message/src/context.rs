use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

use futures::{stream::FuturesUnordered, TryStreamExt};
use macros::model;
use queue_msg::{BoxDynError, QueueError};
use serde_json::Value;
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};
use unionlabs::{ethereum::keccak256, traits::Member, ErrorReporter};

use crate::{
    plugin::{
        ChainModuleClient, ClientModuleClient, ConsensusModuleClient, ConsensusModuleInfo,
        PluginInfo, PluginKind, PluginModuleClient, SupportedInterface,
    },
    rpc::{server::Server, VoyagerRpcServer},
    ChainId, ClientType, IbcInterface,
};

pub const PLUGIN_NAME_CACHE_FILE: &str = "/tmp/voyager-plugin-name-cache.json";

#[derive(Debug)]
pub struct Context {
    modules: Arc<Modules>,

    plugins: HashMap<String, RpcClient>,

    interest_filters: HashMap<String, String>,

    cancellation_token: CancellationToken,

    plugin_servers: Vec<RpcServer>,

    handles: JoinSet<()>,
}

#[derive(Debug, Clone)]
pub struct Modules {
    /// map of chain id to chain module.
    chain_modules: HashMap<ChainId<'static>, RpcClient>,

    /// map of chain id to consensus module.
    consensus_modules: HashMap<ChainId<'static>, RpcClient>,

    /// map of client type to ibc interface to client module.
    client_modules: HashMap<ClientType<'static>, HashMap<IbcInterface<'static>, RpcClient>>,
}

impl queue_msg::Context for Context {
    fn tags(&self) -> Vec<&str> {
        self.interest_filters.keys().map(|s| s.as_str()).collect()
    }
}

#[derive(macros::Debug, Clone)]
pub struct RpcClient {
    #[debug(skip)]
    client: Arc<jsonrpsee::core::client::Client>,
    #[allow(dead_code)]
    socket: String,
    plugin_config: PluginConfig,
}

impl RpcClient {
    async fn new(path: String, config: Value) -> Result<Self, reth_ipc::client::IpcError> {
        let socket = Self::make_socket_path(path.clone(), config.clone());

        let client = reth_ipc::client::IpcClientBuilder::default()
            .build(&socket)
            .await?;

        Ok(Self {
            client: Arc::new(client),
            socket,
            plugin_config: PluginConfig {
                config: config.clone(),
                path: path.clone(),
                // if we're creating a client to this plugin then it must be enabled
                enabled: true,
            },
        })
    }

    fn make_socket_path(path: String, config: Value) -> String {
        format!(
            "/tmp/voyager-to-plugin-{}.sock",
            keccak256(&(config.to_string() + &path)).to_string_unprefixed()
        )
    }

    pub fn client(&self) -> &jsonrpsee::core::client::Client {
        &self.client
    }

    pub fn plugin_config(&self) -> &PluginConfig {
        &self.plugin_config
    }
}

#[derive(macros::Debug)]
pub struct RpcServer {
    #[allow(dead_code)]
    socket: String,
    #[debug(skip)]
    server_handle: Server,
    #[debug(skip)]
    #[allow(dead_code)]
    server_task_handle: JoinHandle<()>,
}

impl RpcServer {
    async fn new(path: String, config: Value) -> Result<Self, BoxDynError> {
        let socket = Self::make_socket_path(path.clone(), config.clone());
        let rpc_server = reth_ipc::server::Builder::default().build(socket.clone());

        let server_handle = Server::new();

        let server_task_handle = tokio::spawn(
            rpc_server
                .start(server_handle.clone().into_rpc())
                .await?
                .stopped(),
        );

        Ok(Self {
            socket,
            server_handle,
            server_task_handle,
        })
    }

    fn start(&self, modules: Arc<Modules>) {
        self.server_handle.start(modules)
    }

    fn make_socket_path(path: String, config: Value) -> String {
        format!(
            "/tmp/plugin-to-voyager-{}.sock",
            keccak256(&(config.to_string() + &path)).to_string_unprefixed()
        )
    }
}

#[model]
#[derive(Hash)]
pub struct PluginConfig {
    pub path: String,
    pub config: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Context {
    pub async fn new(plugin_configs: Vec<PluginConfig>) -> Result<Self, BoxDynError> {
        let cancellation_token = CancellationToken::new();

        let mut modules = Modules {
            chain_modules: Default::default(),
            client_modules: Default::default(),
            consensus_modules: Default::default(),
        };
        let mut plugins = HashMap::default();
        let mut plugin_servers = Vec::default();
        let mut interest_filters = HashMap::default();

        pub enum PluginKindFilled {
            Chain(ChainId<'static>),
            Client(ClientType<'static>, IbcInterface<'static>),
            Consensus(ChainId<'static>),
        }

        pub struct PluginInfoFilled {
            name: String,
            interest_filter: Option<String>,
            rpc_client: RpcClient,
            kind: Option<PluginKindFilled>,
            rpc_server: RpcServer,
        }

        let handles = std::sync::Mutex::new(JoinSet::new());

        let plugins_processed = plugin_configs
            .into_iter()
            .filter_map(|plugin_config| {
                if !plugin_config.enabled {
                    info!(plugin_path = %plugin_config.path, "plugin is not enabled, skipping");
                    return None;
                }

                let healthy = Arc::new(AtomicBool::new(true));

                handles.lock().unwrap().spawn(run_plugin_client(
                    plugin_config.clone(),
                    RpcClient::make_socket_path(
                        plugin_config.path.to_owned(),
                        plugin_config.config.to_owned(),
                    ),
                    cancellation_token.clone(),
                    healthy.clone(),
                ));

                Some(async move {
                    let rpc_server = RpcServer::new(
                        plugin_config.path.to_owned(),
                        plugin_config.config.to_owned(),
                    )
                    .await?;

                    let rpc_client = loop {
                        match RpcClient::new(
                            plugin_config.path.to_owned(),
                            plugin_config.config.to_owned(),
                        )
                        .await
                        {
                            Ok(client) => {
                                debug!(plugin_socket = %client.socket, "connected to socket");
                                break client;
                            }
                            Err(err) => {
                                if healthy.load(std::sync::atomic::Ordering::SeqCst) {
                                    debug!(
                                        err = %ErrorReporter(err),
                                        "unable to connect to socket, plugin has not started yet"
                                    );

                                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                } else {
                                    error!("plugin is unhealthy and could not start");
                                    return Err::<_, BoxDynError>("unhealthy plugin".into());
                                }
                            }
                        }
                    };

                    let PluginInfo {
                        name,
                        kind,
                        interest_filter,
                    } = PluginModuleClient::<Value, Value, Value>::info(rpc_client.client.as_ref())
                        .await?;

                    let kind = match kind {
                        Some(kind) => match kind {
                            PluginKind::Chain => {
                                let chain_id = ChainModuleClient::<Value, Value, Value>::chain_id(
                                    rpc_client.client.as_ref(),
                                )
                                .await?;

                                Some(PluginKindFilled::Chain(chain_id.clone()))
                            }
                            PluginKind::Client => {
                                let SupportedInterface {
                                    client_type,
                                    ibc_interface,
                                } = ClientModuleClient::<Value, Value, Value>::supported_interface(
                                    rpc_client.client.as_ref(),
                                )
                                .await?;

                                Some(PluginKindFilled::Client(client_type, ibc_interface))
                            }
                            PluginKind::Consensus => {
                                let ConsensusModuleInfo {
                                    chain_id,
                                    client_type: _,
                                } = ConsensusModuleClient::<Value, Value, Value>::consensus_info(
                                    rpc_client.client.as_ref(),
                                )
                                .await?;

                                Some(PluginKindFilled::Consensus(chain_id))
                            }
                        },
                        None => None,
                    };

                    Ok(PluginInfoFilled {
                        name,
                        interest_filter,
                        rpc_client,
                        kind,
                        rpc_server,
                    })
                })
            })
            .collect::<FuturesUnordered<_>>()
            .try_collect::<Vec<_>>()
            .await?;

        for PluginInfoFilled {
            name,
            interest_filter,
            rpc_client,
            kind,
            rpc_server,
        } in plugins_processed
        {
            plugins.insert(name.clone(), rpc_client.clone());

            plugin_servers.push(rpc_server);

            if let Some(interest_filter) = interest_filter {
                interest_filters.insert(name.clone(), interest_filter);
            }

            match kind {
                Some(kind) => match kind {
                    PluginKindFilled::Chain(chain_id) => {
                        let prev = modules.chain_modules.insert(chain_id.clone(), rpc_client);

                        if prev.is_some() {
                            return Err(format!(
                                "multiple chain modules configured for chain id `{chain_id}`"
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %chain_id,
                            "registered plugin"
                        );
                    }
                    PluginKindFilled::Client(client_type, ibc_interface) => {
                        let prev = modules
                            .client_modules
                            .entry(client_type.clone())
                            .or_default()
                            .insert(ibc_interface.clone(), rpc_client.clone());

                        if prev.is_some() {
                            return Err(format!(
                                "multiple client modules configured for \
                                    client type `{client_type}` and IBC \
                                    interface `{ibc_interface}`",
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %client_type,
                            %ibc_interface,
                            "registered plugin"
                        );
                    }
                    PluginKindFilled::Consensus(chain_id) => {
                        let prev = modules
                            .consensus_modules
                            .insert(chain_id.clone(), rpc_client.clone());

                        if prev.is_some() {
                            return Err(format!(
                                "multiple consensus modules configured for chain id `{chain_id}`"
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %chain_id,
                            "registered plugin"
                        );
                    }
                },
                None => {
                    info!(
                        %name,
                        "registered plugin"
                    );
                }
            }
        }

        if let Err(err) = tokio::fs::write(
            PLUGIN_NAME_CACHE_FILE,
            serde_json::to_string(
                &plugins
                    .iter()
                    .map(|(plugin_name, client)| (plugin_name, &client.plugin_config))
                    .collect::<HashMap<&String, &PluginConfig>>(),
            )
            .unwrap(),
        )
        .await
        {
            error!(err = %ErrorReporter(err), "unable to write cache file")
        }

        let modules = Arc::new(modules);

        for rpc_server in &plugin_servers {
            rpc_server.start(modules.clone());
        }

        Ok(Self {
            modules,
            plugins,
            interest_filters,
            cancellation_token,
            plugin_servers,
            handles: handles.into_inner().unwrap(),
        })
    }

    pub async fn shutdown(self) {
        self.cancellation_token.cancel();

        let mut handles = self.handles;

        while let Some(res) = handles.join_next().await {
            if let Err(err) = res {
                error!("error shutting down module: {}", ErrorReporter(err));
            }
        }
    }

    pub fn plugin<D: Member, C: Member, Cb: Member>(
        &self,
        plugin_name: impl AsRef<str>,
    ) -> Result<&(impl PluginModuleClient<D, C, Cb> + '_), PluginNotFound> {
        Ok(self
            .plugins
            .get(plugin_name.as_ref())
            .ok_or_else(|| PluginNotFound {
                plugin_name: plugin_name.as_ref().into(),
            })?
            .client
            .as_ref())
    }

    pub fn plugin_client_raw(
        &self,
        plugin_name: impl AsRef<str>,
    ) -> Result<&RpcClient, PluginNotFound> {
        self.plugins
            .get(plugin_name.as_ref())
            .ok_or_else(|| PluginNotFound {
                plugin_name: plugin_name.as_ref().into(),
            })
    }

    pub fn interest_filters(&self) -> &HashMap<String, String> {
        &self.interest_filters
    }

    pub fn chain_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        chain_id: &'b ChainId<'c>,
    ) -> Result<&'a (impl ChainModuleClient<D, C, Cb> + 'a), ChainModuleNotFound> {
        self.modules.chain_module(chain_id)
    }

    pub fn consensus_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        chain_id: &'b ChainId<'c>,
    ) -> Result<&'a (impl ConsensusModuleClient<D, C, Cb> + 'a), ConsensusModuleNotFound> {
        self.modules.consensus_module(chain_id)
    }

    pub fn client_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        client_type: &'b ClientType<'c>,
        ibc_interface: &'b IbcInterface<'c>,
    ) -> Result<&'a (impl ClientModuleClient<D, C, Cb> + 'a), ClientModuleNotFound> {
        self.modules.client_module(client_type, ibc_interface)
    }

    pub fn modules(&self) -> Arc<Modules> {
        self.modules.clone()
    }
}

impl Modules {
    pub fn loaded_chain_modules(&self) -> impl Iterator<Item = &ChainId<'static>> {
        self.chain_modules.keys()
    }

    pub fn loaded_consensus_modules(&self) -> impl Iterator<Item = &ChainId<'static>> {
        self.consensus_modules.keys()
    }

    pub fn loaded_client_modules(
        &self,
    ) -> impl Iterator<
        Item = (
            &ClientType<'static>,
            impl Iterator<Item = &IbcInterface<'static>>,
        ),
    > {
        self.client_modules
            .iter()
            .map(|(client_type, ibc_interfaces)| (client_type, ibc_interfaces.keys()))
    }

    pub fn chain_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        chain_id: &'b ChainId<'c>,
    ) -> Result<&'a (impl ChainModuleClient<D, C, Cb> + 'a), ChainModuleNotFound> {
        Ok(self
            .chain_modules
            .get(chain_id)
            .ok_or_else(|| ChainModuleNotFound(chain_id.clone().into_static()))?
            .client
            .as_ref())
    }

    pub fn consensus_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        chain_id: &'b ChainId<'c>,
    ) -> Result<&'a (impl ConsensusModuleClient<D, C, Cb> + 'a), ConsensusModuleNotFound> {
        Ok(self
            .consensus_modules
            .get(chain_id)
            .ok_or_else(|| ConsensusModuleNotFound(chain_id.clone().into_static()))?
            .client
            .as_ref())
    }

    pub fn client_module<'a: 'b, 'b, 'c: 'a, D: Member, C: Member, Cb: Member>(
        &'a self,
        client_type: &'b ClientType<'c>,
        ibc_interface: &'b IbcInterface<'c>,
    ) -> Result<&'a (impl ClientModuleClient<D, C, Cb> + 'a), ClientModuleNotFound> {
        match self.client_modules.get(client_type) {
            Some(ibc_interfaces) => match ibc_interfaces.get(ibc_interface) {
                Some(client_module) => Ok(client_module.client.as_ref()),
                None => Err(ClientModuleNotFound::IbcInterfaceNotFound {
                    client_type: client_type.clone().into_static(),
                    ibc_interface: ibc_interface.clone().into_static(),
                }),
            },
            None => Err(ClientModuleNotFound::ClientTypeNotFound {
                client_type: client_type.clone().into_static(),
            }),
        }
    }
}

#[instrument(skip_all, fields(%plugin_config.path))]
async fn run_plugin_client(
    plugin_config: PluginConfig,
    plugin_socket: String,
    cancellation_token: CancellationToken,
    healthy: Arc<AtomicBool>,
) {
    'outer: loop {
        debug!("spawning plugin child process");

        let mut cmd = tokio::process::Command::new(&plugin_config.path);
        cmd.arg("run");
        cmd.arg(&plugin_socket);
        cmd.arg(RpcServer::make_socket_path(
            plugin_config.path.clone(),
            plugin_config.config.clone(),
        ));
        cmd.arg(plugin_config.config.to_string());

        let mut child = loop {
            match cmd.spawn() {
                Ok(mut child) => {
                    let id = child.id().unwrap();

                    info!(%id, "spawned plugin");

                    // tokio::time::sleep(std::time::Duration::from_nanos(100)).await;

                    if let Ok(Some(status)) = child.try_wait() {
                        error!(%id, %status, %plugin_socket, "child exited after startup");

                        cancellation_token.cancel();

                        healthy.store(false, std::sync::atomic::Ordering::SeqCst);

                        break 'outer;
                    }

                    break child;
                }
                Err(err) => {
                    error!(
                        err = %ErrorReporter(err),
                        path = ?plugin_config.path,
                        "unable to spawn plugin"
                    );

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
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
                        info!(%id, %exit_status, "child exited")
                    }
                    Err(err) => {
                        error!(%id, err = %ErrorReporter(err), "child exited")
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for chain `{0}`")]
pub struct ChainModuleNotFound(pub ChainId<'static>);

impl From<ChainModuleNotFound> for QueueError {
    fn from(value: ChainModuleNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for consensus on chain `{0}`")]
pub struct ConsensusModuleNotFound(pub ChainId<'static>);

impl From<ConsensusModuleNotFound> for QueueError {
    fn from(value: ConsensusModuleNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for transaction submission on chain `{0}`")]
pub struct TransactionModuleNotFound(pub String);

impl From<TransactionModuleNotFound> for QueueError {
    fn from(value: TransactionModuleNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ClientModuleNotFound {
    #[error("no module loaded for client type `{}`", client_type.0)]
    ClientTypeNotFound { client_type: ClientType<'static> },
    #[error(
        "no module loaded supporting IBC interface `{}` and client type `{}`",
        client_type.0,
        ibc_interface.0,
    )]
    IbcInterfaceNotFound {
        client_type: ClientType<'static>,
        ibc_interface: IbcInterface<'static>,
    },
}

impl From<ClientModuleNotFound> for QueueError {
    fn from(value: ClientModuleNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("plugin `{plugin_name}` not found")]
pub struct PluginNotFound {
    pub plugin_name: String,
}

impl From<PluginNotFound> for QueueError {
    fn from(value: PluginNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}
