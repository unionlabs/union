use std::{fmt::Debug, num::ParseIntError, sync::Arc};

use dashmap::DashMap;
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    encoding::Proto,
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        lightclients::cometbls,
    },
    id::ClientId,
    signer::CosmosSigner,
    traits::{Chain, ClientState, FromStrExact},
    WasmClientType,
};

use crate::{
    cosmos_sdk::{CosmosKeyring, CosmosSdkChain, CosmosSdkChainRpcs, GasConfig},
    keyring::{ChainKeyring, ConcurrentKeyring, KeyringConfig, KeyringEntry, SignerBalance},
};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub keyring: CosmosKeyring,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoint: String,
    pub grpc_url: String,

    pub checksum_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
    pub gas_config: GasConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keyring: KeyringConfig,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
    pub gas_config: GasConfig,
}

impl ChainKeyring for Union {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnionChainType;

impl FromStrExact for UnionChainType {
    const EXPECTING: &'static str = "union";
}

impl Chain for Union {
    type ChainType = UnionChainType;
    type SelfClientState = cometbls::client_state::ClientState;
    type SelfConsensusState = cometbls::consensus_state::ConsensusState;

    type StoredClientState<Tr: Chain> = Any<Tr::SelfClientState>;
    type StoredConsensusState<Tr: Chain> = Any<Tr::SelfConsensusState>;

    type Header = cometbls::header::Header;

    type Height = Height;

    type ClientId = UnionClientId;

    type ClientType = String;

    type Error = tendermint_rpc::Error;

    type IbcStateEncoding = Proto;

    type StateProof = unionlabs::union::ics23::merkle_proof::MerkleProof;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    async fn query_latest_height(&self) -> Result<Height, Self::Error> {
        self.tm_client
            .latest_block()
            .await
            .map(|resonse| self.make_height(resonse.block.header.height.value()))
    }

    async fn query_latest_height_as_destination(&self) -> Result<Height, Self::Error> {
        self.query_latest_height().await
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

        // Expected to be nanos
        let unbonding_period =
            u64::try_from(params.unbonding_time.clone().unwrap().seconds).unwrap() * 1_000_000_000;

        cometbls::client_state::ClientState {
            chain_id: self.chain_id.clone(),
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            trusting_period: unbonding_period * 85 / 100,
            unbonding_period,
            // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
            max_clock_drift: (60 * 20) * 1_000_000_000,
            frozen_height: Height {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: Height {
                revision_number: self.chain_id.split('-').last().unwrap().parse().unwrap(),
                revision_height: height.value(),
            },
        }
    }

    async fn self_consensus_state(&self, height: Height) -> Self::SelfConsensusState {
        let commit = self
            .tm_client
            .commit(u32::try_from(height.revision_height).unwrap())
            .await
            .unwrap();

        cometbls::consensus_state::ConsensusState {
            timestamp: commit
                .signed_header
                .header
                .time
                .unix_timestamp_nanos()
                .try_into()
                .unwrap(),
            app_hash: MerkleRoot {
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
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UnionInitError {
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

impl Union {
    pub async fn new(config: Config) -> Result<Self, UnionInitError> {
        let (tm_client, driver) = WebSocketClient::builder(config.ws_url)
            .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
            .build()
            .await?;

        tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client.status().await?.node_info.network.to_string();

        let chain_revision = chain_id
            .split('-')
            .last()
            .ok_or_else(|| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: None,
            })?
            .parse()
            .map_err(|err| UnionInitError::ChainIdParse {
                found: chain_id.clone(),
                source: Some(err),
            })?;

        Ok(Self {
            // TODO: Deduplicate between this and cosmos.rs
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
                            "union".to_owned(),
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
            prover_endpoint: config.prover_endpoint,
            grpc_url: config.grpc_url,
            checksum_cache: Arc::new(DashMap::default()),
            gas_config: config.gas_config,
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

pub type UnionClientId = ClientId;

impl CosmosSdkChain for Union {
    fn checksum_cache(&self) -> &Arc<dashmap::DashMap<H256, WasmClientType>> {
        &self.checksum_cache
    }
}

impl CosmosSdkChainRpcs for Union {
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
