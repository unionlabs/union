use url::Url;

use crate::network::builder::{FromNetworkBuilder, NetworkBuilder};
use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::rpc::client::Client;

use std::path::PathBuf;

/// URL to the betanet RPC node provided by near.org.
pub const RPC_URL: &str = "https://rpc.betanet.near.org";

/// Betanet related configuration for interacting with betanet. Look at
/// [`workspaces::betanet`] for how to spin up a [`Worker`] that can be
/// used to interact with betanet. Note that betanet account creation
/// is not currently supported, and these calls into creating a betanet
/// worker is meant for retrieving data and/or making queries only.
/// Also, note that betanet can be unstable and does not provide an
/// archival endpoint similar to that of mainnet.
///
/// [`workspaces::betanet`]: crate::betanet
/// [`workspaces::betanet_archival`]: crate::betanet_archival
/// [`Worker`]: crate::Worker
pub struct Betanet {
    client: Client,
    info: Info,
}

#[async_trait::async_trait]
impl FromNetworkBuilder for Betanet {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> crate::result::Result<Self> {
        let rpc_url = build.rpc_addr.unwrap_or_else(|| RPC_URL.into());
        let client = Client::new(&rpc_url, build.api_key)?;
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: build.name.into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/betanet/"),
                rpc_url: Url::parse(&rpc_url).expect("url is hardcoded"),
            },
        })
    }
}

impl NetworkClient for Betanet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Betanet {
    fn info(&self) -> &Info {
        &self.info
    }
}
