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
    events::{TryFromTendermintEventError, WriteAcknowledgement},
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
    option_unwrap, promote,
    signer::CosmosSigner,
    traits::{Chain, ClientState, FromStrExact},
    WasmClientType,
};

use crate::{cosmos_sdk::CosmosSdkChain, private_key::PrivateKey, Pool};

#[derive(Debug, Clone)]
pub struct Cosmos {
    pub chain_id: String,
    pub signers: Pool<CosmosSigner>,
    pub fee_denom: String,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub grpc_url: String,

    pub checksum_cache: Arc<dashmap::DashMap<H256, WasmClientType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub grpc_url: String,
}

impl CosmosSdkChain for Cosmos {
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
            .map(|height| self.make_height(height.block.header.height.value()))
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        self.query_latest_height()
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
                denominator: promote!(NonZeroU64: option_unwrap!(NonZeroU64::new(3))),
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
            max_clock_drift: unionlabs::google::protobuf::duration::Duration::new(60 * 10, 0)
                .unwrap(),
            frozen_height: None,
            latest_height: Height {
                revision_number: self.chain_revision,
                revision_height: height.value(),
            },
            proof_specs: SDK_SPECS.into(),
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
            allow_update_after_expiry: false,
            allow_update_after_misbehavior: false,
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

    async fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: NonZeroU64,
    ) -> Vec<u8> {
        let block_height = self
            .tm_client
            .block_by_hash(block_hash.0.to_vec().try_into().unwrap())
            .await
            .unwrap()
            .block
            .unwrap()
            .header
            .height;

        tracing::info!(
            "Querying ack for {}/{}/{} at {}",
            destination_port_id,
            destination_channel_id,
            sequence,
            block_height
        );

        let wa =
            self.tm_client
                .tx_search(
                    tendermint_rpc::query::Query::eq("tx.height", u64::from(block_height)),
                    false,
                    1,
                    255,
                    tendermint_rpc::Order::Ascending,
                )
                .await
                .unwrap()
                .txs
                .into_iter()
                .find_map(|tx| {
                    tx.tx_result.events.into_iter().find_map(|event| {
                        let maybe_ack = WriteAcknowledgement::try_from(
                            unionlabs::tendermint::abci::event::Event {
                                ty: event.kind,
                                attributes: event.attributes.into_iter().map(|attr| {
                                    unionlabs::tendermint::abci::event_attribute::EventAttribute {
                                        key: attr.key,
                                        value: attr.value,
                                        index: attr.index,
                                    }
                                }).collect()
                            },
                        );
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
                })
                .unwrap();

        wa.packet_ack_hex
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
            signers: Pool::new(
                config
                    .signers
                    .into_iter()
                    // TODO: Make this configurable or fetch it from the chain
                    .map(|signer| CosmosSigner::new(signer.value(), prefix.clone())),
            ),
            tm_client,
            chain_id,
            chain_revision,
            grpc_url: config.grpc_url,
            fee_denom: config.fee_denom,
            checksum_cache: Arc::new(dashmap::DashMap::default()),
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
