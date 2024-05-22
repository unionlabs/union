use crate::network::{Info, NetworkClient, NetworkInfo};
use crate::result::Result;
use crate::rpc::client::Client;
use std::path::PathBuf;

use super::builder::{FromNetworkBuilder, NetworkBuilder};

/// Holds information about a custom network.
pub struct Custom {
    client: Client,
    info: Info,
}

#[async_trait::async_trait]
impl FromNetworkBuilder for Custom {
    async fn from_builder<'a>(build: NetworkBuilder<'a, Self>) -> Result<Self> {
        let rpc_url = build
            .rpc_addr
            .expect("rpc address should be provided for custom network");
        let client = Client::new(&rpc_url, build.api_key)?;
        client.wait_for_rpc().await?;

        Ok(Self {
            client,
            info: Info {
                name: build.name.into(),
                root_id: "near".parse().unwrap(),
                keystore_path: PathBuf::from(".near-credentials/mainnet/"),
                rpc_url: url::Url::parse(&rpc_url).expect("custom provided url should be valid"),
            },
        })
    }
}

impl std::fmt::Debug for Custom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Custom")
            .field("root_id", &self.info.root_id)
            .field("rpc_url", &self.info.rpc_url)
            .finish()
    }
}

impl NetworkClient for Custom {
    fn client(&self) -> &Client {
        &self.client
    }
}

impl NetworkInfo for Custom {
    fn info(&self) -> &Info {
        &self.info
    }
}
