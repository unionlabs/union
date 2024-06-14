use std::collections::BTreeMap;

use chain_utils::{AnyChain, AnyChainTryFromConfigError, ChainConfigType};
use serde::{Deserialize, Serialize};

use crate::queue::AnyQueueConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)]
pub struct Config {
    /// Map of chain name to it's respective config.
    pub chain: BTreeMap<String, ChainConfig>,
    pub voyager: VoyagerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyagerConfig {
    pub num_workers: u16,
    pub queue: AnyQueueConfig,
}

impl Config {
    pub async fn get_chain(&self, name: &str) -> Result<AnyChain, GetChainError> {
        match self.chain.get(name) {
            Some(config) => Ok(AnyChain::try_from_config(config.ty.clone()).await?),
            None => Err(GetChainError::ChainNotFound {
                name: name.to_string(),
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetChainError {
    #[error("chain `{name}` not found in config")]
    ChainNotFound { name: String },
    #[error("error initializing chain")]
    AnyChainTryFromConfig(#[from] AnyChainTryFromConfigError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub ty: ChainConfigType,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_serde() {
        let json = r#"{
          "chain_type": "union",
          "enabled": true,
          "signers": [
            {
              "raw": "0xaa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
            },
            {
              "raw": "0xf562d20f0a4ffd8814d262f7023f33971cbcd14a96d60027585777f174b9cdeb"
            },
            {
              "raw": "0xa1f713e0f36404586085a599a45ca8233e23709e23cd54bc8d5452ef8f7bc1e6"
            },
            {
              "raw": "0xedc165ff1ebc27044ddc284c9cf5da656dcbff324f6ecbb9d3203cf5f4738d6d"
            },
            {
              "raw": "0x40c30853b7f3e6d7ec997fc72c78aef65fce2e82d5b71032a98cb8efaa4710ca"
            },
            {
              "raw": "0xaeff1a3cf6e96d1551c95677fff8399b1ee0c3ed2f610928520897202e5ae690"
            }
          ],
          "ws_url": "ws://localhost:26657/websocket",
          "prover_endpoint": "http://localhost:9999",
          "grpc_url": "http://localhost:9090",
          "gas_config": {
            "gas_price": "1.0",
            "gas_denom": "muno",
            "gas_multiplier": "1.1",
            "max_gas": 400000
          }
        }"#;

        let cfg = serde_json::from_str::<ChainConfig>(json).unwrap();

        dbg!(cfg);
    }
}
