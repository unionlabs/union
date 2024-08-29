use jsonrpsee::{
    core::RpcResult,
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
};
use serde_json::Value;
use tonic::async_trait;
use tracing::{debug, instrument};
use unionlabs::{id::ClientId, ErrorReporter};

use crate::{
    context::Modules, data::ClientInfo, plugin::ChainModuleClient, ChainId,
    FATAL_JSONRPC_ERROR_CODE,
};

#[rpc(
    client,
    server,
    client_bounds(Self: Send + Sync),
    server_bounds(Self:),
    namespace = "voyager",
)]
pub trait VoyagerRpc {
    #[method(name = "clientInfo")]
    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo>;
}

fn fatal_error(t: impl std::error::Error) -> ErrorObjectOwned {
    ErrorObject::owned(
        FATAL_JSONRPC_ERROR_CODE,
        ErrorReporter(t).to_string(),
        None::<()>,
    )
}

pub struct Server {
    pub modules: Modules,
}

#[async_trait]
impl VoyagerRpcServer for Server {
    #[instrument(skip_all, fields(%chain_id, %client_id))]
    async fn client_info(
        &self,
        chain_id: ChainId<'static>,
        client_id: ClientId,
    ) -> RpcResult<ClientInfo> {
        debug!("fetching client info");

        let client_info = <_ as ChainModuleClient<Value, Value, Value>>::client_info(
            self.modules.chain_module(&chain_id).map_err(fatal_error)?,
            client_id,
        )
        .await
        .map_err(fatal_error)?;

        debug!(%client_info.ibc_interface, %client_info.client_type, "fetched client info");

        Ok(client_info)
    }
}
