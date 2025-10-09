pub mod hook;

use std::fmt::Debug;

use jsonrpsee::{Extensions, async_client, core::RpcResult, types::ErrorObject};
use serde::Serialize;
use serde_json::Value;
use tracing::error;
use unionlabs::ErrorReporter;
use voyager_plugin::protocol::{ArcClient, IdThreadClient};
use voyager_rpc::FATAL_JSONRPC_ERROR_CODE;
#[doc(no_inline)]
pub use {
    anyhow, jsonrpsee, serde_json, voyager_client as client, voyager_message as message,
    voyager_plugin as plugin, voyager_primitives as primitives, voyager_rpc as rpc,
    voyager_types as types, voyager_vm as vm,
};

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

pub type VoyagerClient =
    voyager_client::VoyagerClient<IdThreadClient<ArcClient<async_client::Client>>>;

pub trait ExtensionsExt {
    fn voyager_client(&self) -> RpcResult<&VoyagerClient>;
}

impl ExtensionsExt for Extensions {
    fn voyager_client(&self) -> RpcResult<&VoyagerClient> {
        match self.get() {
            Some(t) => Ok(t),
            None => Err(ErrorObject::owned(
                -1,
                "failed to retrieve voyager client from extensions",
                None::<()>,
            )),
        }
    }
}

#[derive(clap::Subcommand)]
pub enum DefaultCmd {}
