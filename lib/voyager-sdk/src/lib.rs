pub mod hook;

use std::{collections::BTreeMap, fmt::Debug};

use jsonrpsee::{
    async_client, core::RpcResult, types::ErrorObject, ws_client::HeaderMap, Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::{Layer, Service};
use tracing::error;
use unionlabs::ErrorReporter;
use voyager_plugin::protocol::{ArcClient, IdThreadClient};
use voyager_rpc::FATAL_JSONRPC_ERROR_CODE;
#[doc(no_inline)]
pub use {
    anyhow, jsonrpsee, serde_json, tower_http, voyager_client as client,
    voyager_message as message, voyager_plugin as plugin, voyager_primitives as primitives,
    voyager_rpc as rpc, voyager_types as types, voyager_vm as vm,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcUrlConfig {
    Url(String),
    Config {
        url: String,
        headers: BTreeMap<String, String>,
    },
}

impl RpcUrlConfig {
    pub fn url(&self) -> &str {
        match self {
            RpcUrlConfig::Url(url) => url,
            RpcUrlConfig::Config { url, headers: _ } => url,
        }
    }

    pub fn headers(&self) -> &BTreeMap<String, String> {
        match self {
            RpcUrlConfig::Url(_) => const { &BTreeMap::new() },
            RpcUrlConfig::Config { url: _, headers } => headers,
        }
    }
}

pub struct SetMultipleHeadersLayer {
    pub headers: HeaderMap,
}

impl<S> Layer<S> for SetMultipleHeadersLayer {
    type Service = SetMultipleHeaders;

    fn layer(&self, inner: S) -> Self::Service {
        todo!()
    }
}

pub struct SetMultipleHeaders<S> {
    headers: HeaderMap,
    inner: S,
}

impl<ReqBody, ResBody, S, M> Service<Request<ReqBody>> for SetRequestHeader<S, M>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,

    M: MakeHeaderValue<Request<ReqBody>>,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = S::Future;

    #[inline]

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        self.mode.apply(&self.header_name, &mut req, &mut self.make);

        self.inner.call(req)
    }
}
