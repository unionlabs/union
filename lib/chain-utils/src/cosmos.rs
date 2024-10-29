use std::{num::ParseIntError, sync::Arc};

use bip32::secp256k1::ecdsa;
use protos::cosmos::auth::v1beta1::Bech32PrefixRequest;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    hash::H256, ibc::core::client::height::Height, signer::CosmosSigner, WasmClientType,
};

use crate::{
    cosmos_sdk::{CosmosKeyring, CosmosSdkChain, CosmosSdkChainRpcs, GasConfig},
    keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, KeyringEntry, SignerBalance},
};

#[derive(Debug, Clone)]
pub struct Cosmos {
    pub chain_id: String,
    pub keyring: CosmosKeyring,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub grpc_url: String,
    pub gas_config: GasConfig,
    pub bech32_prefix: String,

    pub checksum_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keyring: KeyringConfig,
    pub ws_url: WebSocketClientUrl,
    pub grpc_url: String,
    pub gas_config: GasConfig,
}

impl ChainKeyring for Cosmos {
    type Address = String;

    type Signer = CosmosSigner;

    fn keyring(&self) -> &ConcurrentKeyring<Self::Address, Self::Signer> {
        &self.keyring
    }

    async fn balances(&self) -> Vec<SignerBalance<Self::Address>> {
        crate::cosmos_sdk::fetch_balances(
            &self.keyring,
            self.gas_config.gas_denom.clone(),
            self.grpc_url.clone(),
        )
        .await
    }
}

impl CosmosSdkChain for Cosmos {
    fn checksum_cache(&self) -> &Arc<dashmap::DashMap<H256, WasmClientType>> {
        &self.checksum_cache
    }
}

impl CosmosSdkChainRpcs for Cosmos {
    fn tm_chain_id(&self) -> String {
        self.chain_id.clone()
    }

    fn gas_config(&self) -> &GasConfig {
        &self.gas_config
    }

    fn grpc_url(&self) -> String {
        self.grpc_url.clone()
    }

    fn tm_client(&self) -> &WebSocketClient {
        &self.tm_client
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CosmosInitError {
    #[error("tendermint rpc error")]
    Tendermint(#[from] tendermint_rpc::Error),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    ChainIdParse {
        found: String,
        #[source]
        source: Option<ParseIntError>,
    },
}

impl Cosmos {
    pub async fn new(config: Config) -> Result<Self, CosmosInitError> {
        let (tm_client, driver) = WebSocketClient::builder(config.ws_url)
            .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
            .build()
            .await?;

        tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| CosmosInitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| CosmosInitError::ChainIdParse {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        let prefix = protos::cosmos::auth::v1beta1::query_client::QueryClient::connect(
            config.grpc_url.clone(),
        )
        .await
        .unwrap()
        .bech32_prefix(Bech32PrefixRequest {})
        .await
        .unwrap()
        .into_inner()
        .bech32_prefix;

        Ok(Self {
            keyring: CosmosKeyring::new(
                config.keyring.name,
                config
                    .keyring
                    .keys
                    .into_iter()
                    // TODO: Make this configurable or fetch it from the chain
                    .map(|entry| {
                        let signer = CosmosSigner::new(
                            ecdsa::SigningKey::from_bytes(entry.value().as_slice().into())
                                .expect("invalid private key"),
                            prefix.clone(),
                        );

                        KeyringEntry {
                            name: entry.name(),
                            address: signer.to_string(),
                            signer,
                        }
                    }),
            ),
            tm_client,
            chain_id,
            chain_revision,
            grpc_url: config.grpc_url,
            checksum_cache: Arc::new(dashmap::DashMap::default()),
            gas_config: config.gas_config,
            bech32_prefix: prefix,
        })
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new_with_revision(self.chain_revision, height)
    }
}
