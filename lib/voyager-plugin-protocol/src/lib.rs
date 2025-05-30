//! # Voyager Plugin Protocol
//!
//! Voyager's plugin system uses a coordinator/worker architecture for message passing between the main voyager binary (the coordinator) and the plugins/modules (the workers), communicating via JSON-RPC-over-IPC.
//!
//! ## Coordinator
//!
//! The coordinator is responsible for managing the workers and routing messages to and between them.
//!
//! ## Worker
//!
//! The workers are managed by the coordinator. Their only responsibility is to respond to the messages from the coordinator.
//!
//! # Boot Sequence
//!
//! 1. Coordinator starts a server listening on [`coordinator_socket_path`], and creates a [`WorkerClient`] that will connect to [`worker_socket_path`].
//! 2. Coordinator spawns the worker, passing [`coordinator_socket_path`] and [`worker_socket_path`] as arguments.
//! 3. Worker starts it's server, listening on [`worker_socket_path`].
//! 4. Worker creates a client connecting to [`coordinator_socket_path`].
//! 5. Coordinator client now connects to the booted worker.

use std::{
    borrow::Cow,
    fmt::Debug,
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use futures::FutureExt;
use jsonrpsee::{
    core::{
        async_trait,
        client::{BatchResponse, ClientT},
        params::BatchRequestBuilder,
        traits::ToRpcParams,
    },
    server::middleware::rpc::RpcServiceT,
    types::{ErrorObject, Response, ResponsePayload},
    MethodResponse, RpcModule,
};
use reth_ipc::{
    client::IpcClientBuilder,
    server::{RpcService, RpcServiceBuilder},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::RawValue;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::{
    debug, debug_span, error, info, info_span, instrument, instrument::Instrumented, trace,
    Instrument,
};
use unionlabs::{ethereum::slot::keccak256, primitives::encoding::HexUnprefixed, ErrorReporter};
use voyager_client::VoyagerClient;
use voyager_rpc::VoyagerRpcServer;
use voyager_vm::ItemId;

pub const INVALID_CONFIG_EXIT_CODE: u8 = 13;
pub const STARTUP_ERROR_EXIT_CODE: u8 = 14;

/// Run the coordinator server.
///
/// This will listen to messages on [`coordinator_socket_path`]`(name)`.
pub async fn coordinator_server(
    name: &str,
    server: impl VoyagerRpcServer,
) -> anyhow::Result<impl Future<Output = ()>> {
    let coordinator_socket = coordinator_socket_path(name);
    let rpc_server = reth_ipc::server::Builder::default()
        .set_rpc_middleware(
            reth_ipc::server::RpcServiceBuilder::new()
                .layer_fn(|service| ExtractItemIdService { service }),
        )
        .build(coordinator_socket.clone());

    debug!(%coordinator_socket, "starting rpc server");

    let server = rpc_server.start(server.into_rpc()).await?;

    Ok(server
        .stopped()
        .instrument(debug_span!("module_rpc_server", %name)))
}

/// Run the worker server.
///
/// This will listen to messages from the coordinator on `coordinator_socket`, and send messages to the coordinator on `worker_socket`.
#[instrument(skip_all, fields(%id))]
pub async fn worker_server<T>(
    id: String,
    coordinator_socket: String,
    worker_socket: String,
    fut: impl Future<Output = anyhow::Result<T>>,
    into_rpc: impl FnOnce(T) -> RpcModule<T>,
) {
    let worker_server = match fut.await {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("startup error in {id}: {err:?}");
            std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
        }
    };

    let voyager_client = match IpcClientBuilder::default().build(&coordinator_socket).await {
        Ok(voyager_client) => ArcClient(Arc::new(voyager_client)),
        Err(err) => {
            trace!(
                error = %ErrorReporter(err),
                "unable to connect to coordinator"
            );
            std::process::exit(STARTUP_ERROR_EXIT_CODE as i32);
        }
    };

    trace!("connected to voyager socket");

    let ipc_server = reth_ipc::server::Builder::default()
        .set_rpc_middleware(
            RpcServiceBuilder::new()
                .layer_fn(move |service| ExtractItemIdService { service })
                .layer_fn(move |service| InjectVoyagerClientService {
                    client: voyager_client.clone(),
                    service,
                })
                .layer_fn({
                    let id = id.clone();
                    move |service: RpcService| ErrorContextService {
                        service,
                        id: id.clone(),
                    }
                }),
        )
        .build(worker_socket);

    let rpcs = into_rpc(worker_server);

    trace!(methods = ?*rpcs, "registered methods");
    let addr = ipc_server.endpoint();
    let server_handle = ipc_server.start(rpcs).await.unwrap();
    debug!("listening on {addr}");

    server_handle
        .stopped()
        .instrument(debug_span!("{id}"))
        .await
}

/// The RPC client to communicate with a worker from the coordinator.
///
/// This is a thin wrapper around a [`reconnecting_jsonrpc_ws_client::Client`]. If the worker crashes or restarts, it will automatically attempt to reconnect.
#[derive(Clone)]
pub struct WorkerClient {
    client: reconnecting_jsonrpc_ws_client::Client,
    #[allow(dead_code)]
    name: String,
}

impl WorkerClient {
    pub fn inner(&self) -> &reconnecting_jsonrpc_ws_client::Client {
        &self.client
    }
}

delegate_client_impl!(WorkerClient: |this| this.client);
delegate_client_impl!(&WorkerClient: |this| this.client);

impl WorkerClient {
    pub fn new(name: &str, request_timeout: Duration) -> Self {
        let worker_socket = worker_socket_path(name);

        trace!("creating socket at {worker_socket}");

        let client = reconnecting_jsonrpc_ws_client::Client::new({
            // NOTE: This needs to be leaked because the return type of the .build() method
            // below captures the lifetime of the `name` parameter(?)
            let socket: &'static str = Box::leak(worker_socket.clone().into_boxed_str());
            let name = name.to_owned();
            move || {
                async move {
                    trace!("connecting to socket at {socket}");
                    reth_ipc::client::IpcClientBuilder::default()
                        .request_timeout(request_timeout)
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

    pub fn client(&self) -> &reconnecting_jsonrpc_ws_client::Client {
        &self.client
    }
}

/// Make the socket path that the worker will listen on for messages from the coordinator.
///
/// This is salted with the PID of the current process to ensure there are no collisions between multiple workers with the same name.
pub fn worker_socket_path(name: &str) -> String {
    let pid = std::process::id();

    format!(
        "/tmp/coordinator-to-worker-{}.sock",
        // TODO: Use sha
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

/// Make the socket path that the coordinator will listen on for messages from the worker.
///
/// This is salted with the PID of the current process to ensure there are no collisions between multiple workers with the same name.
pub fn coordinator_socket_path(name: &str) -> String {
    let pid = std::process::id();

    format!(
        "/tmp/worker-to-coordinator-{}.sock",
        // TODO: Use sha
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

/// An [`RpcServiceT`] layer to extract the [`ItemId`] threaded by an [`IdThreadClient`].
///
/// The extracted item id, if any, is inserted into the request extensions, and the inner request is forwarded to `S`.
pub struct ExtractItemIdService<S> {
    service: S,
}

impl<'a, S: RpcServiceT<'a>> RpcServiceT<'a> for ExtractItemIdService<S> {
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

/// An [`RpcServiceT`] layer to inject a [`VoyagerClient`] into the request extensions.
///
/// If there is an item id present in the request extensions (likely extracted via [`ExtractItemIdService`]), this will also thread the id in the voyager client with [`IdThreadClient`].
struct InjectVoyagerClientService<S, C> {
    client: C,
    service: S,
}

impl<'a, S, C> RpcServiceT<'a> for InjectVoyagerClientService<S, C>
where
    S: RpcServiceT<'a> + Send + Sync,
    C: ClientT + Clone + Send + Sync + 'static,
{
    type Future = futures::future::Either<Instrumented<S::Future>, S::Future>;

    fn call(&self, mut request: jsonrpsee::types::Request<'a>) -> Self::Future {
        let item_id = request.extensions.get::<ItemId>().cloned();

        request
            .extensions
            .insert(VoyagerClient::new(IdThreadClient {
                client: self.client.clone(),
                item_id,
            }));

        self.service.call(request).right_future()
    }
}

/// An [`RpcServiceT`] layer to provide error context about the current worker.
///
/// This allows for "tracing" errors across many cross-worker calls.
struct ErrorContextService<S> {
    service: S,
    id: String,
}

impl<'a, S: RpcServiceT<'a> + Send + Sync> RpcServiceT<'a> for ErrorContextService<S> {
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

/// Structure of a message containing a threaded item id.
///
/// The field names are intentionally mangled in order to prevent collisions with real RPC request parameters. If your method parameters clash with this struct, you should probably re-think what you're doing.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamsWithItemId<'a> {
    #[serde(rename = "$$__id__$$")]
    item_id: ItemId,
    #[serde(rename = "$$__params__$$", borrow)]
    params: Option<Cow<'a, RawValue>>,
}

impl ToRpcParams for ParamsWithItemId<'_> {
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, serde_json::Error> {
        Ok(Some(
            RawValue::from_string(serde_json::to_string(&self)?).unwrap(),
        ))
    }
}

/// An RPC client that will thread the given item id, if any, through all requests.
///
/// NOTE: It is expected that the receiving end has registered the [`ExtractItemIdService`] layer.
#[derive(Debug, Clone)]
pub struct IdThreadClient<Inner: ClientT + Send + Sync> {
    pub(crate) client: Inner,
    item_id: Option<ItemId>,
}

/// Convenience trait to wrap any [`ClientT`] type in an [`IdThreadClient`].
pub trait WithId: Sized + ClientT + Send + Sync
where
    for<'a> &'a Self: ClientT,
{
    fn with_id(&self, item_id: Option<ItemId>) -> IdThreadClient<&Self> {
        trace!(?item_id, "threading id");

        IdThreadClient {
            client: self,
            item_id,
        }
    }
}

impl<T: ClientT + Send + Sync> WithId for T where for<'a> &'a Self: ClientT {}

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

        // thread the item id through the request if it is present
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

#[instrument(skip_all, fields(%name))]
pub async fn worker_child_process(
    name: String,
    path: PathBuf,
    cancellation_token: CancellationToken,
    args: impl IntoIterator<Item: Into<String>>,
) {
    let coordinator_to_worker_socket = worker_socket_path(&name);
    let worker_to_coordinator_socket = coordinator_socket_path(&name);

    debug!(%coordinator_to_worker_socket, %worker_to_coordinator_socket);

    lazarus_pit(
        &path,
        [
            "run".to_owned(),
            coordinator_to_worker_socket,
            worker_to_coordinator_socket,
        ]
        .into_iter()
        .chain(args.into_iter().map(Into::into))
        .collect(),
        cancellation_token,
    )
    .await
}

/// Spawn a worker process with the given args, re-spawning it indefinitely unless it exits with [`INVALID_CONFIG_EXIT_CODE`] or the passed in cancellation token is cancelled.
#[instrument(skip_all)]
async fn lazarus_pit(cmd: &Path, args: Vec<String>, cancellation_token: CancellationToken) {
    let mut attempt = 0;

    loop {
        let mut cmd = tokio::process::Command::new(cmd);
        cmd.args(&args);

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
                            error!(%id, "invalid config for plugin or module");
                            cancellation_token.cancel();
                            break;
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

// https://github.com/paritytech/jsonrpsee/issues/1578
#[derive(Debug)]
pub struct ArcClient<C>(Arc<C>);

impl<C> Clone for ArcClient<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

delegate_client_impl!({C: ClientT + Send + Sync} ArcClient<C>: |this| this.0);
delegate_client_impl!({C: ClientT + Send + Sync} &ArcClient<C>: |this| this.0);

#[macro_export]
macro_rules! delegate_client_impl {
    ($({$($generics:tt)+})? $T:ty: |$binding:ident| $expr:expr) => {
        #[async_trait]
        impl $(<$($generics)+>)? jsonrpsee::core::client::ClientT for $T {
            async fn notification<Params>(
                &self,
                method: &str,
                params: Params,
            ) -> Result<(), jsonrpsee::core::client::Error>
            where
                Params: jsonrpsee::core::traits::ToRpcParams + Send,
            {
                #[allow(unused_imports)]
                use jsonrpsee::core::client::ClientT;

                let $binding = self;
                ($expr).notification(method, params).await
            }

            async fn request<R, Params>(
                &self,
                method: &str,
                params: Params,
            ) -> Result<R, jsonrpsee::core::client::Error>
            where
                R: serde::de::DeserializeOwned,
                Params: jsonrpsee::core::traits::ToRpcParams + Send,
            {
                #[allow(unused_imports)]
                use jsonrpsee::core::client::ClientT;

                let $binding = self;
                ($expr).request(method, params).await
            }

            async fn batch_request<'a, R>(
                &self,
                batch: jsonrpsee::core::params::BatchRequestBuilder<'a>,
            ) -> Result<jsonrpsee::core::client::BatchResponse<'a, R>, jsonrpsee::core::client::Error>
            where
                R: serde::de::DeserializeOwned + Debug + 'a,
            {
                #[allow(unused_imports)]
                use jsonrpsee::core::client::ClientT;

                let $binding = self;
                ($expr).batch_request(batch).await
            }
        }
    };
}
