use std::{
    num::{NonZeroU64, ParseIntError},
    sync::Arc,
};

use bip32::secp256k1::ecdsa;
use futures::Future;
use ics23::ibc_api::SDK_SPECS;
use protos::cosmos::auth::v1beta1::Bech32PrefixRequest;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    constants::metric::NANOS_PER_SECOND,
    encoding::Proto,
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::{
            client::height::Height,
            commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
        },
        lightclients::tendermint::{self, fraction::Fraction},
    },
    id::ClientId,
    option_unwrap, result_unwrap,
    signer::CosmosSigner,
    traits::{Chain, ClientState, FromStrExact},
    WasmClientType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CosmosChainType;

impl FromStrExact for CosmosChainType {
    const EXPECTING: &'static str = "cosmos";
}

impl Chain for Cosmos {
    type ChainType = CosmosChainType;

    type SelfClientState = tendermint::client_state::ClientState;
    type SelfConsensusState = tendermint::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> = Any<Tr::SelfClientState>;
    type StoredConsensusState<Tr: Chain> = Any<Tr::SelfConsensusState>;

    type Header = tendermint::header::Header;

    type Height = Height;

    type ClientId = ClientId;

    type ClientType = String;

    type Error = tendermint_rpc::Error;

    type IbcStateEncoding = Proto;

    type StateProof = MerkleProof;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    async fn query_latest_height(&self) -> Result<Height, Self::Error> {
        self.tm_client
            .latest_block()
            .await
            .map(|response| self.make_height(response.block.header.height.value()))
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        self.query_latest_height()
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        self.tm_client
            .latest_block()
            .await
            .map(|resonse| resonse.block.header.time.unix_timestamp())
    }

    async fn self_client_state(&self, height: Height) -> Self::SelfClientState {
        let params = protos::cosmos::staking::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .params(protos::cosmos::staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

        let commit = self
            .tm_client
            .commit(u32::try_from(height.revision_height).unwrap())
            .await
            .unwrap();

        let height = commit.signed_header.header.height;

        let unbonding_period = std::time::Duration::new(
            params
                .unbonding_time
                .clone()
                .unwrap()
                .seconds
                .try_into()
                .unwrap(),
            params
                .unbonding_time
                .clone()
                .unwrap()
                .nanos
                .try_into()
                .unwrap(),
        );

        tendermint::client_state::ClientState {
            chain_id: self.chain_id(),
            // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
            trust_level: Fraction {
                numerator: 1,
                denominator: const { option_unwrap!(NonZeroU64::new(3)) },
            },
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            trusting_period: unionlabs::google::protobuf::duration::Duration::new(
                (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                (unbonding_period * 85 / 100)
                    .subsec_nanos()
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
            unbonding_period: unionlabs::google::protobuf::duration::Duration::new(
                unbonding_period.as_secs().try_into().unwrap(),
                unbonding_period.subsec_nanos().try_into().unwrap(),
            )
            .unwrap(),
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            max_clock_drift: const {
                result_unwrap!(unionlabs::google::protobuf::duration::Duration::new(
                    60 * 10,
                    0
                ))
            },
            frozen_height: None,
            latest_height: Height {
                revision_number: self.chain_revision,
                revision_height: height.value(),
            },
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
        }
    }

    async fn self_consensus_state(&self, height: Height) -> Self::SelfConsensusState {
        let commit = self
            .tm_client
            .commit(u32::try_from(height.revision_height).unwrap())
            .await
            .unwrap();

        tendermint::consensus_state::ConsensusState {
            root: MerkleRoot {
                hash: commit
                    .signed_header
                    .header
                    .app_hash
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .unwrap(),
            },
            next_validators_hash: commit
                .signed_header
                .header
                .next_validators_hash
                .as_bytes()
                .to_vec()
                .try_into()
                .unwrap(),
            timestamp: {
                let timestamp_nanos = commit.signed_header.header.time.unix_timestamp_nanos();
                let seconds = timestamp_nanos / NANOS_PER_SECOND as i128;
                let nanos = timestamp_nanos % NANOS_PER_SECOND as i128;
                unionlabs::google::protobuf::timestamp::Timestamp {
                    seconds: BoundedI64::new(seconds.try_into().unwrap()).unwrap(),
                    nanos: BoundedI32::new(nanos.try_into().unwrap()).unwrap(),
                }
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CosmosInitError {
    #[error("tendermint rpc error")]
    Tendermint(#[from] tendermint_rpc::Error),
    #[error(
        "unable to parse chain id: expected format `<chain>-<revision-number>`, found `{found}`"
    )]
    // TODO: Once the `Id` trait in unionlabs is cleaned up to no longer use static id types, this error should just wrap `IdParseError`
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
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }
}
