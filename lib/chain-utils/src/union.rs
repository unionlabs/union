use std::{
    fmt::Debug,
    num::{NonZeroU64, ParseIntError},
    sync::Arc,
};

use dashmap::DashMap;
use ethers::prelude::k256::ecdsa;
use serde::{Deserialize, Serialize};
use tendermint_rpc::{Client, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    encoding::Proto,
    events::{TryFromTendermintEventError, WriteAcknowledgement},
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

use crate::{cosmos_sdk::CosmosSdkChain, private_key::PrivateKey, Pool};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub signers: Pool<CosmosSigner>,
    pub fee_denom: String,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoint: String,
    pub grpc_url: String,

    pub checksum_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
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
            .map(|height| self.make_height(height.block.header.height.value()))
    }

    async fn query_latest_height_as_destination(&self) -> Result<Height, Self::Error> {
        self.query_latest_height().await
    }

    async fn query_latest_timestamp(&self) -> Result<i64, Self::Error> {
        let height = self.query_latest_height().await?;
        Ok(self
            .tm_client
            .block(u32::try_from(height.revision_height).unwrap())
            .await?
            .block
            .header
            .time
            .unix_timestamp())
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

    async fn read_ack(
        &self,
        tx_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: NonZeroU64,
    ) -> Vec<u8> {
        tracing::info!(
            "Querying ack for {}/{}/{} from tx {tx_hash}",
            destination_port_id,
            destination_channel_id,
            sequence,
        );

        self.tm_client
            // TODO: Use tx_search here
            .tx(tx_hash.0.to_vec().try_into().unwrap(), false)
            .await
            .unwrap()
            .tx_result
            .events
            .into_iter()
            .find_map(|event| {
                let maybe_ack =
                    WriteAcknowledgement::try_from(unionlabs::tendermint::abci::event::Event {
                        ty: event.kind,
                        attributes: event
                            .attributes
                            .into_iter()
                            .map(|attr| {
                                unionlabs::tendermint::abci::event_attribute::EventAttribute {
                                    key: attr.key,
                                    value: attr.value,
                                    index: attr.index,
                                }
                            })
                            .collect(),
                    });
                match maybe_ack {
                    Ok(ok)
                        if ok.packet_sequence == sequence
                            && ok.packet_dst_port == destination_port_id
                            && ok.packet_dst_channel == destination_channel_id =>
                    {
                        Some(ok)
                    }
                    Ok(ok) => {
                        tracing::debug!("Found ack not matching our packet {:?}", ok);
                        None
                    }
                    Err(TryFromTendermintEventError::IncorrectType { .. }) => None,
                    Err(err) => {
                        panic!("{err:#?}")
                    }
                }
            })
            .unwrap()
            .packet_ack_hex
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
            signers: Pool::new(
                config
                    .signers
                    .into_iter()
                    .map(|signer| CosmosSigner::new(signer.value(), "union".to_string())),
            ),
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            grpc_url: config.grpc_url,
            fee_denom: config.fee_denom,
            checksum_cache: Arc::new(DashMap::default()),
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

#[derive(Debug)]
pub enum UnionEventSourceError {
    TryFromTendermintEvent(TryFromTendermintEventError),
    Subscription(tendermint_rpc::Error),
}

impl CosmosSdkChain for Union {
    fn grpc_url(&self) -> String {
        self.grpc_url.clone()
    }

    fn fee_denom(&self) -> String {
        self.fee_denom.clone()
    }

    fn tm_client(&self) -> &WebSocketClient {
        &self.tm_client
    }

    fn signers(&self) -> &Pool<CosmosSigner> {
        &self.signers
    }

    fn checksum_cache(&self) -> &Arc<dashmap::DashMap<H256, WasmClientType>> {
        &self.checksum_cache
    }
}
