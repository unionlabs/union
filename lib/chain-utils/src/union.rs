use std::{fmt::Debug, num::ParseIntError, sync::Arc};

use dashmap::DashMap;
use ethers::prelude::k256::ecdsa;
use futures::{stream, Future, FutureExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tendermint_rpc::{query::Query, Client, Order, WebSocketClient, WebSocketClientUrl};
use unionlabs::{
    encoding::{Decode, Proto},
    events::{IbcEvent, TryFromTendermintEventError, WriteAcknowledgement},
    google::protobuf::any::Any,
    hash::H256,
    ibc::{
        core::{
            client::height::{Height, IsHeight},
            commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
        },
        lightclients::cometbls,
    },
    id::ClientId,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
    signer::CosmosSigner,
    tendermint::abci::{event::Event, event_attribute::EventAttribute},
    traits::{Chain, ClientState, FromStrExact},
    TryFromProto, WasmClientType,
};

use crate::{cosmos_sdk::CosmosSdkChain, private_key::PrivateKey, ChainEvent, EventSource, Pool};

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

    type StateProof = MerkleProof;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    fn query_latest_height(&self) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        async move {
            self.tm_client
                .latest_block()
                .await
                .map(|height| self.make_height(height.block.header.height.value()))
        }
    }

    fn query_latest_height_as_destination(
        &self,
    ) -> impl Future<Output = Result<Height, Self::Error>> + '_ {
        self.query_latest_height()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = Result<i64, Self::Error>> + '_ {
        async move {
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
    }

    fn self_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
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

            let unbonding_period: u64 = params
                .unbonding_time
                .clone()
                .unwrap()
                .seconds
                .try_into()
                .unwrap();

            cometbls::client_state::ClientState {
                chain_id: self.chain_id.clone(),
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                trusting_period: unbonding_period * 85 / 100,
                unbonding_period,
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                max_clock_drift: 60 * 20,
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
    }

    fn self_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfConsensusState> + '_ {
        async move {
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
                    .unix_timestamp()
                    .try_into()
                    .unwrap(),
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
            }
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: unionlabs::id::PortId,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
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

            let wa = self
                .tm_client
                .tx_search(
                    Query::eq("tx.height", u64::from(block_height)),
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

impl EventSource for Union {
    type Event = ChainEvent<Self>;
    type Error = UnionEventSourceError;
    // TODO: Make this the height to start from
    type Seed = ();

    fn events(self, _seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> {
        async move {
            let chain_revision = self.chain_revision;

            let latest_height = self.query_latest_height().await.unwrap();

            stream::unfold(
                (self, latest_height),
                move |(this, previous_height)| async move {
                    tracing::info!("fetching events");

                    let current_height = loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let current_height = match this.query_latest_height().await {
                            Ok(current_height) => current_height,
                            Err(e) => {
                                tracing::error!(error = %e, "Error getting height from Union. Trying again in 1 second.");
                                continue;
                            }
                        };

                        tracing::debug!(%current_height, %previous_height);

                        if current_height > previous_height {
                            break current_height;
                        }
                    };

                    tracing::debug!(
                        previous_height = previous_height.revision_height,
                        current_height = current_height.revision_height
                    );

                    let mut events = vec![];

                    for h in
                        (previous_height.revision_height + 1)..=(current_height.revision_height)
                    {
                        let response = if let Ok(res) = this
                            .tm_client
                            .tx_search(Query::eq("tx.height", h), false, 1, 255, Order::Descending)
                            .await
                        {
                            res
                        } else {
                            return None;
                        };

                        let new_events = stream::iter(response.txs.into_iter().flat_map(|tx| {
                            tx.tx_result
                                .events
                                .into_iter()
                                .map(|event| Event {
                                    ty: event.kind,
                                    attributes: event
                                        .attributes
                                        .into_iter()
                                        .map(|attr| EventAttribute {
                                            key: attr.key,
                                            value: attr.value,
                                            index: attr.index,
                                        })
                                        .collect(),
                                })
                                .filter_map(IbcEvent::try_from_tendermint_event)
                                .map(move |res| {
                                    res.map(|x| (tx.height, x))
                                        .map_err(UnionEventSourceError::TryFromTendermintEvent)
                                })
                        }))
                        .then(|res| async {
                            match res {
                                Ok((height, event)) =>
                                    match this.tm_client.block(height).await {
                                    Ok(block) => Ok(ChainEvent {
                                            chain_id: this.chain_id(),
                                            block_hash: block
                                                .block_id
                                                .hash
                                                .as_bytes()
                                                .try_into()
                                                .unwrap(),
                                            height: Height {
                                                revision_number: chain_revision,
                                                revision_height: height.into(),
                                            }.increment(),
                                            event,
                                        }),
                                        Err(e) => Err(UnionEventSourceError::Subscription(e)), },
                                Err(err) => Err(err)
                            }
                        })
                        .collect::<Vec<_>>()
                        .await;

                        events.extend(new_events);
                    }

                    let iter = events;

                    Some((iter, (this, current_height)))
                },
            )
        }
        .flatten_stream()
        .map(futures::stream::iter)
        .flatten()
    }
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

pub trait AbciStateRead<Tr>: IbcPath<Union, Tr>
where
    Tr: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output;
}

impl<Tr> AbciStateRead<Tr> for ClientStatePath<<Union as Chain>::ClientId>
where
    Tr: Chain,
    Self::Output: Decode<Proto>,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        <Self::Output as Decode<Proto>>::decode(&bytes).unwrap()
    }
}

impl<Tr> AbciStateRead<Tr>
    for ClientConsensusStatePath<<Union as Chain>::ClientId, <Tr as Chain>::Height>
where
    Tr: Chain,
    Self::Output: Decode<Proto>,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        <Self::Output as Decode<Proto>>::decode(&bytes).unwrap()
    }
}

impl<Tr> AbciStateRead<Tr> for ConnectionPath
where
    Tr: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Tr> AbciStateRead<Tr> for ChannelEndPath
where
    Tr: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        Self::Output::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Tr> AbciStateRead<Tr> for CommitmentPath
where
    Tr: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        bytes.try_into().unwrap()
    }
}

impl<Tr> AbciStateRead<Tr> for AcknowledgementPath
where
    Tr: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output {
        bytes.try_into().unwrap()
    }
}
