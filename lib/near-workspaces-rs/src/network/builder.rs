use std::future::{Future, IntoFuture};
use std::marker::PhantomData;

use crate::network::Sandbox;
use crate::{Network, Worker};

use super::server::ValidatorKey;

pub(crate) type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// This trait provides a way to construct Networks out of a single builder. Currently,
/// not planned to offer this trait outside, since the custom networks can just construct
/// themselves however they want utilizing `Worker::new` like so:
/// ```ignore
/// Worker::new(CustomNetwork {
///   ... // fields
/// })
/// ```
#[async_trait::async_trait]
pub(crate) trait FromNetworkBuilder: Sized {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> crate::result::Result<Self>;
}

/// Builder for Networks. Only usable with workspaces provided Networks.
// Note, this is currently the aggregated state for all network types you can have since
// I didn't want to add additional reading complexity with another trait that associates the
// Network state.
pub struct NetworkBuilder<'a, T> {
    pub(crate) name: &'a str,
    pub(crate) rpc_addr: Option<String>,
    pub(crate) validator_key: Option<ValidatorKey>,
    pub(crate) api_key: Option<String>,
    _network: PhantomData<T>,
}

impl<'a, T> IntoFuture for NetworkBuilder<'a, T>
where
    T: FromNetworkBuilder + Network + Send + 'a,
{
    type Output = crate::result::Result<Worker<T>>;
    type IntoFuture = BoxFuture<'a, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let fut = async {
            let network = FromNetworkBuilder::from_builder(self).await?;
            Ok(Worker::new(network))
        };
        Box::pin(fut)
    }
}

impl<'a, T> NetworkBuilder<'a, T> {
    pub(crate) fn new(name: &'a str) -> Self {
        Self {
            name,
            rpc_addr: None,
            validator_key: None,
            api_key: None,
            _network: PhantomData,
        }
    }

    /// Sets the RPC addr for this network. Useful for setting the Url to a different RPC
    /// node than the default one provided by near.org. This enables certain features that
    /// the default node doesn't provide such as getting beyond the data cap when downloading
    /// state from the network.
    ///
    /// Note that, for sandbox, we are required to specify `validator_key` as well to connect to
    /// a manually spawned sandbox node.
    pub fn rpc_addr(mut self, addr: &str) -> Self {
        self.rpc_addr = Some(addr.into());
        self
    }

    /// Sets the API key for this network. Useful for setting the API key to an RPC
    /// server that requires it.
    ///
    /// Note that if you're using a custom network, the burden is on you to ensure that
    /// the methods you're calling are supported by the RPC server you're connecting to.
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
}

// So far, only Sandbox makes use of validator_key.
impl NetworkBuilder<'_, Sandbox> {
    /// Specify how to fetch the validator key of the manually spawned sandbox node.
    /// We are expected to init our own sandbox before running this builder. To learn more
    /// about initializing and  starting our own sandbox, go to [near-sandbox](https://github.com/near/sandbox).
    /// This can be either set to a known key value or to the home directory where all the chain data lives.
    /// This is the `my_home_folder` we passed into `near-sandbox --home {my_home_folder} init`.
    pub fn validator_key(mut self, validator_key: ValidatorKey) -> Self {
        self.validator_key = Some(validator_key);
        self
    }
}
