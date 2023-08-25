use std::collections::BTreeMap;

use clap::{
    builder::{StringValueParser, TypedValueParser},
    Args,
};
use serde::{Deserialize, Serialize};
use tendermint_rpc::WebSocketClientUrl;
use unionlabs::ethereum::{Address, H256};

use crate::{
    chain::AnyChain,
    config::private_key::{parse_private_key_arg, PrivateKey},
};

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
    Cosmos(UnionChainConfig),
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
    /// The address of the `ICS20TransferBank` smart contract.
    #[arg(long)]
    pub ics20_transfer_bank_address: Address,
    /// The address of the `ICS20Bank` smart contract.
    #[arg(long)]
    pub ics20_bank_address: Address,

    /// The signer that will be used to submit transactions by the relayer.
    #[arg(
        long,
        value_parser = StringValueParser::new()
            .try_map(parse_private_key_arg::<ethers::prelude::k256::ecdsa::SigningKey>)
    )]
    pub signer: PrivateKey<ethers::prelude::k256::ecdsa::SigningKey>,

    // TODO(benluelo): Use `Url` or something similar
    /// The RPC endpoint for the execution chain.
    #[arg(long)]
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    #[arg(long)]
    pub eth_beacon_rpc_api: String,

    #[arg(long)]
    pub wasm_code_id: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize, Args)]
pub struct UnionChainConfig {
    #[arg(
        long,
        value_parser = StringValueParser::new()
            .try_map(parse_private_key_arg::<ethers::prelude::k256::ecdsa::SigningKey>)
    )]
    pub signer: PrivateKey<ethers::prelude::k256::ecdsa::SigningKey>,
    #[arg(long)]
    pub ws_url: WebSocketClientUrl,
    #[arg(long)]
    pub wasm_code_id: H256,
    #[arg(long)]
    pub prover_endpoint: String,
    #[arg(long)]
    pub dump_path: String,
    #[arg(long)]
    pub grpc_url: String,
}

pub mod private_key {
    use std::fmt::{Display, Write};

    use bip32::PrivateKeyBytes;
    use clap::error::{ContextKind, ContextValue};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum PrivateKey<T: bip32::PrivateKey> {
        /// The key stored in plaintext.
        Raw(#[serde(with = "private_key_hex_string")] T), // TODO: Other key types (i.e. keyring)
    }

    mod private_key_hex_string {
        use bip32::{PrivateKey, PrivateKeyBytes};
        use serde::de::Error;

        pub fn serialize<S, T: PrivateKey>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serde_utils::hex_string::serialize(data.to_bytes(), serializer)
        }

        pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
        where
            D: serde::Deserializer<'de>,
            T: PrivateKey,
        {
            serde_utils::hex_string::deserialize(deserializer).and_then(|data: PrivateKeyBytes| {
                <T as PrivateKey>::from_bytes(&data).map_err(|x| D::Error::custom(x.to_string()))
            })
        }
    }

    impl<T: bip32::PrivateKey> PrivateKey<T> {
        pub fn value(self) -> T {
            match self {
                PrivateKey::Raw(raw) => raw,
            }
        }
    }

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
