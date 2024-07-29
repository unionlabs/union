use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

use macros::model;
use queue_msg::{BoxDynError, QueueError};
use serde_json::Value;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};
use unionlabs::{ethereum::keccak256, traits::Member, ErrorReporter};

use crate::{
    plugin::{
        ChainModuleClient, ClientModuleClient, ConsensusModuleClient, ConsensusModuleInfo,
        PluginInfo, PluginKind, PluginModuleClient, SupportedInterface,
        TransactionSubmissionModuleClient,
    },
    ClientType, IbcInterface,
};

#[derive(Debug)]
pub struct Context {
    /// map of chain id to chain module.
    chain_modules: HashMap<String, RpcClient>,

    /// map of chain id to consensus module.
    consensus_modules: HashMap<String, RpcClient>,

    /// map of client type to ibc interface to client module.
    client_modules: HashMap<ClientType<'static>, HashMap<IbcInterface<'static>, RpcClient>>,

    /// map of chain id to transaction module.
    transaction_modules: HashMap<String, RpcClient>,

    plugins: HashMap<String, RpcClient>,

    interest_filters: HashMap<String, String>,

    cancellation_token: CancellationToken,

    handles: JoinSet<()>,
}

#[derive(macros::Debug, Clone)]
pub struct RpcClient {
    #[debug(skip)]
    client: Arc<jsonrpsee::core::client::Client>,
    #[allow(dead_code)]
    socket: String,
}

impl RpcClient {
    async fn new(socket: String) -> Result<Self, reth_ipc::client::IpcError> {
        let client = reth_ipc::client::IpcClientBuilder::default()
            .build(&socket)
            .await?;

        Ok(Self {
            client: Arc::new(client),
            socket,
        })
    }
}

#[model]
pub struct PluginConfig {
    pub path: String,
    pub config: Value,
}

impl Context {
    pub async fn new(plugins: Vec<PluginConfig>) -> Result<Self, BoxDynError> {
        let cancellation_token = CancellationToken::new();

        let mut this = Self {
            chain_modules: Default::default(),
            client_modules: Default::default(),
            consensus_modules: Default::default(),
            transaction_modules: Default::default(),
            plugins: Default::default(),
            interest_filters: Default::default(),
            cancellation_token: cancellation_token.clone(),
            handles: JoinSet::new(),
        };

        for plugin_info in plugins {
            let plugin_socket = format!(
                "/tmp/voyager-plugin-{}.sock",
                keccak256(&(plugin_info.config.to_string() + &plugin_info.path))
                    .to_string_unprefixed()
            );

            let healthy = Arc::new(AtomicBool::new(true));

            this.handles.spawn(run_plugin_client(
                plugin_info,
                plugin_socket.clone(),
                cancellation_token.clone(),
                healthy.clone(),
            ));

            let rpc_client = loop {
                match RpcClient::new(plugin_socket.clone()).await {
                    Ok(client) => {
                        debug!(%plugin_socket, "connected to socket");
                        break client;
                    }
                    Err(err) => {
                        if healthy.load(std::sync::atomic::Ordering::SeqCst) {
                            debug!(
                                err = %ErrorReporter(err),
                                %plugin_socket,
                                "unable to connect to socket, plugin has not started yet"
                            );

                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        } else {
                            error!(
                                %plugin_socket,
                                "plugin is unhealthy and could not start"
                            );
                            return Err("unhealthy plugin".into());
                        }
                    }
                }
            };

            let PluginInfo {
                name,
                kind,
                interest_filter,
            } = PluginModuleClient::<Value, Value, Value>::info(rpc_client.client.as_ref()).await?;

            debug!(%name, "registering plugin");

            match this.plugins.insert(name.clone(), rpc_client.clone()) {
                Some(_) => {
                    return Err(format!("plugin name collision: `{name}`").into());
                }
                None => {
                    //
                }
            }

            if let Some(interest_filter) = interest_filter {
                this.interest_filters.insert(name.clone(), interest_filter);
            }

            match kind {
                Some(kind) => match kind {
                    PluginKind::Chain => {
                        let chain_id = ChainModuleClient::<Value, Value, Value>::chain_id(
                            rpc_client.client.as_ref(),
                        )
                        .await?;

                        let prev = this.chain_modules.insert(chain_id.clone(), rpc_client);

                        if prev.is_some() {
                            return Err(format!(
                                "multiple chain modules configured for chain id `{chain_id}`"
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %kind,
                            %chain_id,
                            %plugin_socket,
                            "registered plugin"
                        );
                    }
                    PluginKind::Client => {
                        let SupportedInterface {
                            client_type,
                            ibc_interface,
                        } = ClientModuleClient::<Value, Value, Value>::supported_interface(
                            rpc_client.client.as_ref(),
                        )
                        .await?;

                        let entry = this.client_modules.entry(client_type.clone()).or_default();

                        let prev = entry.insert(ibc_interface.clone(), rpc_client.clone());

                        if prev.is_some() {
                            return Err(format!(
                                "multiple chain modules configured for \
                                    client type `{client_type}` and IBC \
                                    interface `{ibc_interface}`",
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %kind,
                            %client_type,
                            %ibc_interface,
                            %plugin_socket,
                            "registered plugin"
                        );
                    }
                    PluginKind::Consensus => {
                        let ConsensusModuleInfo {
                            chain_id,
                            client_type,
                        } = ConsensusModuleClient::<Value, Value, Value>::consensus_info(
                            rpc_client.client.as_ref(),
                        )
                        .await?;

                        let prev = this
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
                            %kind,
                            %client_type,
                            %plugin_socket,
                            "registered plugin"
                        );
                    }
                    PluginKind::Transaction => {
                        let chain_id =
                            TransactionSubmissionModuleClient::<Value, Value, Value>::register(
                                rpc_client.client.as_ref(),
                            )
                            .await?;

                        let prev = this
                            .transaction_modules
                            .insert(chain_id.clone(), rpc_client.clone());

                        if prev.is_some() {
                            return Err(format!(
                                "multiple transaction modules configured for chain id `{chain_id}`"
                            )
                            .into());
                        }

                        info!(
                            %name,
                            %kind,
                            %chain_id,
                            %plugin_socket,
                            "registered plugin"
                        );
                    }
                },
                None => {
                    info!(
                        %name,
                        %plugin_socket,
                        "registered plugin"
                    );
                }
            }
        }

        dbg!(&this);

        Ok(this)
    }

    pub async fn shutdown(mut self) {
        self.cancellation_token.cancel();

        while let Some(res) = self.handles.join_next().await {
            if let Err(err) = res {
                error!("error shutting down module: {}", ErrorReporter(err));
            }
        }
    }

    pub fn chain_module<D: Member, F: Member, A: Member>(
        &self,
        chain_id: impl AsRef<str>,
    ) -> Result<&(impl ChainModuleClient<D, F, A> + '_), ChainModuleNotFound> {
        Ok(self
            .chain_modules
            .get(chain_id.as_ref())
            .ok_or_else(|| ChainModuleNotFound(chain_id.as_ref().to_string()))?
            .client
            .as_ref())
    }

    pub fn consensus_module<D: Member, F: Member, A: Member>(
        &self,
        chain_id: impl AsRef<str>,
    ) -> Result<&(impl ConsensusModuleClient<D, F, A> + '_), ConsensusModuleNotFound> {
        Ok(self
            .consensus_modules
            .get(chain_id.as_ref())
            .ok_or_else(|| ConsensusModuleNotFound(chain_id.as_ref().to_string()))?
            .client
            .as_ref())
    }

    pub fn transaction_module<D: Member, F: Member, A: Member>(
        &self,
        chain_id: impl AsRef<str>,
    ) -> Result<&(impl TransactionSubmissionModuleClient<D, F, A> + '_), TransactionModuleNotFound>
    {
        Ok(self
            .transaction_modules
            .get(chain_id.as_ref())
            .ok_or_else(|| TransactionModuleNotFound(chain_id.as_ref().to_string()))?
            .client
            .as_ref())
    }

    pub fn client_module<'a: 'b, 'b, 'c: 'a, D: Member, F: Member, A: Member>(
        &'a self,
        client_type: &'b ClientType<'c>,
        ibc_interface: &'b IbcInterface<'c>,
    ) -> Result<&'a (impl ClientModuleClient<D, F, A> + 'a), ClientModuleNotFound> {
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

    pub fn plugin<D: Member, F: Member, A: Member>(
        &self,
        plugin_name: impl AsRef<str>,
    ) -> Result<&(impl PluginModuleClient<D, F, A> + '_), PluginNotFound> {
        Ok(self
            .plugins
            .get(plugin_name.as_ref())
            .ok_or_else(|| PluginNotFound {
                plugin_name: plugin_name.as_ref().into(),
            })?
            .client
            .as_ref())
    }

    pub fn plugin_client_raw<D: Member, F: Member, A: Member>(
        &self,
        plugin_name: impl AsRef<str>,
    ) -> Result<&jsonrpsee::core::client::Client, PluginNotFound> {
        Ok(self
            .plugins
            .get(plugin_name.as_ref())
            .ok_or_else(|| PluginNotFound {
                plugin_name: plugin_name.as_ref().into(),
            })?
            .client
            .as_ref())
    }

    pub fn interest_filters(&self) -> &HashMap<String, String> {
        &self.interest_filters
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
        cmd.arg(&plugin_socket);
        cmd.arg(plugin_config.config.to_string());

        let mut child = loop {
            match cmd.spawn() {
                Ok(mut child) => {
                    let id = child.id().unwrap();

                    info!(%id, "spawned plugin");

                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

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
pub struct ChainModuleNotFound(pub String);

impl From<ChainModuleNotFound> for QueueError {
    fn from(value: ChainModuleNotFound) -> Self {
        Self::Fatal(Box::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for consensus on chain `{0}`")]
pub struct ConsensusModuleNotFound(pub String);

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
