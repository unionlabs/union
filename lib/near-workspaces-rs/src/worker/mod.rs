mod impls;

use std::fmt;
use std::sync::Arc;

use crate::network::builder::NetworkBuilder;
use crate::network::{Betanet, Custom, Mainnet, Sandbox, Testnet};
use crate::types::gas_meter::GasHook;
use crate::{Network, Result};

/// The `Worker` type allows us to interact with any NEAR related networks, such
/// as mainnet and testnet. This controls where the environment the worker is
/// running on top of it. Refer to this for all network related actions such as
/// deploying a contract, or interacting with transactions.
pub struct Worker<T: ?Sized> {
    pub(crate) workspace: Arc<T>,
    pub(crate) tx_callbacks: Vec<GasHook>,
}

impl<T> Worker<T>
where
    T: Network,
{
    pub(crate) fn new(network: T) -> Self {
        Self {
            workspace: Arc::new(network),
            tx_callbacks: vec![],
        }
    }
}

impl<T: Network + 'static> Worker<T> {
    pub(crate) fn coerce(self) -> Worker<dyn Network> {
        Worker {
            workspace: self.workspace,
            tx_callbacks: self.tx_callbacks,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Worker<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Worker")
            .field("workspace", &self.workspace)
            .finish()
    }
}

/// Spin up a new sandbox instance, and grab a [`Worker`] that interacts with it.
pub fn sandbox<'a>() -> NetworkBuilder<'a, Sandbox> {
    NetworkBuilder::new("sandbox")
}

/// Spin up a new sandbox instance, and grab a [`Worker`] that interacts with it.
pub async fn sandbox_with_version<'a>(version: &str) -> Result<Worker<Sandbox>> {
    let network_builder = NetworkBuilder::new("sandbox");
    let network = Sandbox::from_builder_with_version(network_builder, version).await?;
    Ok(Worker::new(network))
}

/// Connect to the [testnet](https://explorer.testnet.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub fn testnet<'a>() -> NetworkBuilder<'a, Testnet> {
    NetworkBuilder::new("testnet")
}

/// Connect to the [testnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub fn testnet_archival<'a>() -> NetworkBuilder<'a, Testnet> {
    NetworkBuilder::new("testnet-archival").rpc_addr(crate::network::testnet::ARCHIVAL_URL)
}

/// Connect to the [mainnet](https://explorer.near.org/) network, and grab
/// a [`Worker`] that can interact with it.
pub fn mainnet<'a>() -> NetworkBuilder<'a, Mainnet> {
    NetworkBuilder::new("mainnet")
}

/// Connect to the [mainnet archival](https://near-nodes.io/intro/node-types#archival-node)
/// network, and grab a [`Worker`] that can interact with it.
pub fn mainnet_archival<'a>() -> NetworkBuilder<'a, Mainnet> {
    NetworkBuilder::new("mainnet-archival").rpc_addr(crate::network::mainnet::ARCHIVAL_URL)
}

/// Connect to the betanet network, and grab a [`Worker`] that can interact with it.
pub fn betanet<'a>() -> NetworkBuilder<'a, Betanet> {
    NetworkBuilder::new("betanet")
}

/// Connect to a custom network, and grab a [`Worker`] that can interact with it.
///
/// Note: the burden of ensuring the methods that are able to be called are left up to the user.
pub fn custom<'a>(rpc_url: &str) -> NetworkBuilder<'a, Custom> {
    NetworkBuilder::new("custom").rpc_addr(rpc_url)
}

/// Run a locally scoped task where a [`sandbox`] instanced [`Worker`] is supplied.
pub async fn with_sandbox<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Sandbox>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(sandbox().await?).await)
}

/// Run a locally scoped task where a [`testnet`] instanced [`Worker`] is supplied.
pub async fn with_testnet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(testnet().await?).await)
}

/// Run a locally scoped task where a [`testnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_testnet_archival<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Testnet>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(testnet_archival().await?).await)
}

/// Run a locally scoped task where a [`mainnet`] instanced [`Worker`] is supplied.
pub async fn with_mainnet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(mainnet().await?).await)
}

/// Run a locally scoped task where a [`mainnet_archival`] instanced [`Worker`] is supplied.
pub async fn with_mainnet_archival<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Mainnet>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(mainnet_archival().await?).await)
}

/// Run a locally scoped task where a [`betanet`] instanced [`Worker`] is supplied.
pub async fn with_betanet<F, T>(task: F) -> Result<T::Output>
where
    F: Fn(Worker<Betanet>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(betanet().await?).await)
}

#[allow(dead_code)]
pub async fn with_custom<F, T>(task: F, rpc_url: &str) -> Result<T::Output>
where
    F: Fn(Worker<Custom>) -> T + Send + Sync,
    T: core::future::Future + Send,
{
    Ok(task(custom(rpc_url).await?).await)
}
