use std::collections::BTreeMap;

use chain_utils::private_key::PrivateKey;
use clap::{
    builder::{StringValueParser, TypedValueParser},
    Args,
};
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::ethereum::Address;

use crate::{chain::AnyChain, config::private_key::parse_private_key_arg};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Map of chain name to it's respective config.
    pub chain: BTreeMap<String, ChainConfig>,
}

impl Config {
    pub async fn get_chain(&self, name: &str) -> Option<AnyChain> {
        match self.chain.get(name) {
            Some(config) => Some(AnyChain::try_from_config(config.clone()).await),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "chain_type")]
pub enum ChainConfig {
    Evm(EvmChainConfig),
    Union(UnionChainConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "preset_base")]
pub enum EvmChainConfig {
    Mainnet(EvmChainConfigFields),
    Minimal(EvmChainConfigFields),
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct EvmChainConfigFields {
    // TODO: Move all except for ibc_handler into the client config?
    /// The address of the `CometblsClient` smart contract.
    #[arg(long)]
    pub cometbls_client_address: Address,
    /// The address of the `IBCHandler` smart contract.
    #[arg(long)]
    pub ibc_handler_address: Address,

    /// The signer that will be used to submit transactions by voyager.
    #[arg(
        long,
        value_parser = StringValueParser::new()
            .try_map(parse_private_key_arg::<ecdsa::SigningKey>)
    )]
    pub signer: PrivateKey<ecdsa::SigningKey>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    #[arg(long)]
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    #[arg(long)]
    pub eth_beacon_rpc_api: String,
    // #[arg(long)]
    // pub wasm_code_id: H256,
}

impl From<EvmChainConfigFields> for chain_utils::evm::Config {
    fn from(value: EvmChainConfigFields) -> Self {
        Self {
            ibc_handler_address: value.ibc_handler_address,
            signer: value.signer,
            eth_rpc_api: value.eth_rpc_api,
            eth_beacon_rpc_api: value.eth_beacon_rpc_api,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct UnionChainConfig {
    #[arg(
        long,
        value_parser = StringValueParser::new()
            .try_map(parse_private_key_arg::<ecdsa::SigningKey>)
    )]
    pub signer: PrivateKey<ecdsa::SigningKey>,
    #[arg(long)]
    pub ws_url: WebSocketClientUrl,
    // #[arg(long)]
    // pub wasm_code_id: H256,
    #[arg(long)]
    pub prover_endpoint: String,
    #[arg(long)]
    pub dump_path: String,
    #[arg(long)]
    pub grpc_url: String,
}

impl From<UnionChainConfig> for chain_utils::union::Config {
    fn from(value: UnionChainConfig) -> Self {
        Self {
            signer: value.signer,
            ws_url: value.ws_url,
            prover_endpoint: value.prover_endpoint,
            dump_path: value.dump_path,
            grpc_url: value.grpc_url,
        }
    }
}

pub mod private_key {
    use std::fmt::{Display, Write};

    use bip32::PrivateKeyBytes;
    use chain_utils::private_key::PrivateKey;
    use clap::error::{ContextKind, ContextValue};

    #[allow(clippy::needless_pass_by_value)] // required by StringValueParser::try_map
    pub fn parse_private_key_arg<T: bip32::PrivateKey + Clone + Send + Sync + 'static>(
        s: String,
    ) -> Result<PrivateKey<T>, clap::error::Error> {
        fn invalid_value<E: Display>(s: &String, maybe_error: Option<E>) -> clap::error::Error {
            let mut error = clap::Error::new(clap::error::ErrorKind::ValueValidation);

            error.insert(
                ContextKind::InvalidValue,
                ContextValue::String(s.to_string()),
            );

            let mut usage_and_note = "Usage: raw:<private key hex>".to_string();

            if let Some(note) = maybe_error {
                write!(&mut usage_and_note, "\n\nNote: {note}").unwrap();
            }

            error.insert(
                ContextKind::Usage,
                ContextValue::StyledStr(usage_and_note.into()),
            );
            error
        }

        match s.as_bytes() {
            [b'r', b'a', b'w', b':', pk @ ..] => serde_utils::parse_hex::<PrivateKeyBytes>(pk)
                .map_err(|err| invalid_value(&s, Some(err)))
                .and_then(|pk: PrivateKeyBytes| {
                    T::from_bytes(&pk).map_err(|err| invalid_value(&s, Some(err)))
                })
                .map(PrivateKey::Raw),
            // NOTE: &str holds no meaning here, but a type is required since it's not possible to set a
            // default for E: https://github.com/rust-lang/rust/issues/36887
            _ => Err(invalid_value(&s, None::<&str>)),
        }
    }
}
