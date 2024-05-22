use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::result::Result;
use crate::rpc::client::Client;
use std::path::PathBuf;

use super::builder::{FromNetworkBuilder, NetworkBuilder};

/// URL to the mainnet RPC node provided by near.org.
pub const RPC_URL: &str = "https://rpc.mainnet.near.org";

/// URL to the mainnet archival RPC node provided by near.org.
pub const ARCHIVAL_URL: &str = "https://archival-rpc.mainnet.near.org";

/// Mainnet related configuration for interacting with mainnet. Look at
/// [`workspaces::mainnet`] and [`workspaces::mainnet_archival`] for how to
/// spin up a [`Worker`] that can be used to interact with mainnet. Note that
/// mainnet account creation is not currently supported, and these calls into
/// creating a mainnet worker is meant for retrieving data and/or making
/// queries only.
///
/// [`workspaces::mainnet`]: crate::mainnet
/// [`workspaces::mainnet_archival`]: crate::mainnet_archival
/// [`Worker`]: crate::Worker
pub struct Mainnet {
    client: Client,
    info: Info,
}

#[async_trait::async_trait]
impl FromNetworkBuilder for Mainnet {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> Result<Self> {
        let rpc_url = build.rpc_addr.unwrap_or_else(|| RPC_URL.into());
        let client = Client::new(&rpc_url, build.api_key)?;
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: build.name.into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: url::Url::parse(&rpc_url).expect("url is hardcoded"),
            },
        })
    }
}

impl std::fmt::Debug for Mainnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mainnet")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .finish()
    }
}

impl NetworkClient for Mainnet {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Mainnet {
    fn info(&self) -> &Info {
        &self.info
    }
}
