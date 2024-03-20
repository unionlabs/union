use std::collections::BTreeMap;

use chain_utils::{cosmos, private_key::PrivateKey, scroll, union};
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use unionlabs::{ethereum::config::PresetBaseKind, hash::H160};

use crate::{
    chain::{AnyChain, AnyChainTryFromConfigError},
    queue::AnyQueueConfig,
};

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
#[serde(rename_all = "snake_case", tag = "chain_type")]
pub enum ChainConfigType {
    Union(union::Config),
    Cosmos(cosmos::Config),
    Ethereum(EthereumChainConfig),
    Scroll(scroll::Config),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct ChainConfig {
    pub enabled: bool,
    #[serde(flatten)]
    pub ty: ChainConfigType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumChainConfig {
    pub preset_base: PresetBaseKind,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,
}

// pub mod private_key {
//     use std::fmt::{Display, Write};

//     use bip32::PrivateKeyBytes;
//     use chain_utils::private_key::PrivateKey;
//     use clap::error::{ContextKind, ContextValue};

//     #[allow(clippy::needless_pass_by_value)] // required by StringValueParser::try_map
//     pub fn parse_private_key_arg<T: bip32::PrivateKey + Clone + Send + Sync + 'static>(
//         s: String,
//     ) -> Result<PrivateKey<T>, clap::error::Error> {
//         fn invalid_value<E: Display>(s: &String, maybe_error: Option<E>) -> clap::error::Error {
//             let mut error = clap::Error::new(clap::error::ErrorKind::ValueValidation);

//             error.insert(
//                 ContextKind::InvalidValue,
//                 ContextValue::String(s.to_string()),
//             );

//             let mut usage_and_note = "Usage: raw:<private key hex>".to_string();

//             if let Some(note) = maybe_error {
//                 write!(&mut usage_and_note, "\n\nNote: {note}").unwrap();
//             }

//             error.insert(
//                 ContextKind::Usage,
//                 ContextValue::StyledStr(usage_and_note.into()),
//             );
//             error
//         }

//         match s.as_bytes() {
//             [b'r', b'a', b'w', b':', pk @ ..] => serde_utils::parse_hex::<PrivateKeyBytes>(pk)
//                 .map_err(|err| invalid_value(&s, Some(err)))
//                 .and_then(|pk: PrivateKeyBytes| {
//                     T::from_bytes(&pk).map_err(|err| invalid_value(&s, Some(err)))
//                 })
//                 .map(PrivateKey::Raw),
//             // NOTE: &str holds no meaning here, but a type is required since it's not possible to set a
//             // default for E: https://github.com/rust-lang/rust/issues/36887
//             _ => Err(invalid_value(&s, None::<&str>)),
//         }
//     }
// }
