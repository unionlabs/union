use std::{fmt::Debug, marker::PhantomData, ops::Div, str::FromStr, sync::Arc};

use beacon_api::client::BeaconApiClient;
use contracts::{
    ibc_handler::{
        GetChannelCall, GetChannelReturn, GetClientStateCall, GetClientStateReturn,
        GetConnectionCall, GetConnectionReturn, GetConsensusStateCall, GetConsensusStateReturn,
        GetHashedPacketAcknowledgementCommitmentCall,
        GetHashedPacketAcknowledgementCommitmentReturn, GetHashedPacketCommitmentCall,
        GetHashedPacketCommitmentReturn, IBCHandler, IBCHandlerEvents, WriteAcknowledgementFilter,
    },
    shared_types::{IbcCoreChannelV1ChannelData, IbcCoreConnectionV1ConnectionEndData},
};
use ethers::{
    abi::{AbiEncode, Tokenizable},
    contract::{ContractError, EthCall, EthLogDecode},
    core::k256::ecdsa,
    middleware::{NonceManagerMiddleware, SignerMiddleware},
    providers::{Middleware, Provider, ProviderError, Ws, WsClientError},
    signers::{LocalWallet, Wallet},
    utils::{keccak256, secret_key_to_address},
};
use futures::{stream, Future, FutureExt, Stream, StreamExt};
use hubble::hasura::{insert_demo_tx, Datastore, HasuraConfig, HasuraDataStore, InsertDemoTx};
use prost::Message;
use serde::{Deserialize, Serialize};
use typenum::Unsigned;
use unionlabs::{
    ethereum::{Address, H256, U256},
    ethereum_consts_traits::ChainSpec,
    events::{
        AcknowledgePacket, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry,
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
        CreateClient, IbcEvent, RecvPacket, SendPacket,
    },
    google::protobuf::any::Any,
    ibc::{
        core::{
            channel::channel::Channel,
            client::height::{Height, IsHeight},
            connection::connection_end::ConnectionEnd,
        },
        lightclients::{cometbls, ethereum, tendermint::fraction::Fraction, wasm},
    },
    id::{ChannelId, ClientId, ConnectionId, PortId},
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath, IbcStateRead,
    },
    traits::{Chain, ChainIdOf, ClientIdOf, ClientStateOf, Consensus, ConsensusStateOf, HeightOf},
    validated::ValidateT,
    EmptyString, TryFromEthAbiErrorOf, TryFromProto, TryFromProtoErrorOf,
};

use crate::{private_key::PrivateKey, ChainEvent, EventSource, Pool};

pub type CometblsMiddleware =
    SignerMiddleware<NonceManagerMiddleware<Provider<Ws>>, Wallet<ecdsa::SigningKey>>;

// TODO(benluelo): Generic over middleware?
#[derive(Debug, Clone)]
pub struct Evm<C: ChainSpec> {
    pub chain_id: U256,

    pub readonly_ibc_handler: IBCHandler<Provider<Ws>>,

    // pub wallet: LocalWallet,
    pub ibc_handlers: Pool<IBCHandler<CometblsMiddleware>>,
    pub provider: Provider<Ws>,
    pub beacon_api_client: BeaconApiClient<C>,

    pub hasura_client: Option<HasuraDataStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: Address,

    /// The signer that will be used to submit transactions by voyager.
    pub signers: Vec<PrivateKey<ecdsa::SigningKey>>,

    /// The RPC endpoint for the execution chain.
    pub eth_rpc_api: String,
    /// The RPC endpoint for the beacon chain.
    pub eth_beacon_rpc_api: String,

    pub hasura_config: Option<HasuraConfig>,
}

pub struct EthereumConsensus<C: ChainSpec>(PhantomData<fn() -> C>);

impl<C: ChainSpec> Consensus for EthereumConsensus<C> {
    // TODO: Unwrap these out of wasm, and re-wrap them in union
    type SelfClientState =
        Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>;

    type Header = wasm::header::Header<ethereum::header::Header<C>>;

    type Height = Height;
}

impl<C: ChainSpec> Chain for Evm<C> {
    type Consensus = EthereumConsensus<C>;

    type ClientId = EvmClientId;

    type ClientType = String;

    fn chain_id(&self) -> ChainIdOf<Self> {
        self.chain_id
    }

    fn query_latest_height(&self) -> impl Future<Output = HeightOf<Self>> + '_ {
        async move {
            let height = self
                .beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .beacon
                .slot;

            self.make_height(height)
        }
    }

    fn query_latest_height_as_destination(&self) -> impl Future<Output = HeightOf<Self>> + '_ {
        async move {
            let height = self
                .beacon_api_client
                .block(beacon_api::client::BlockId::Head)
                .await
                .unwrap()
                .data
                .message
                .slot;

            // HACK: we introduced this because we were using alchemy for the
            // execution endpoint and our custom beacon endpoint that rely on
            // its own execution chain. Alchemy was a bit delayed and the
            // execution height for the beacon head wasn't existing for few
            // secs. We wait for an extra beacon head to let alchemy catch up.
            // We should be able to remove that once we rely on an execution
            // endpoint that is itself used by the beacon endpoint (no different
            // POV).
            loop {
                let next_height = self
                    .beacon_api_client
                    .block(beacon_api::client::BlockId::Head)
                    .await
                    .unwrap()
                    .data
                    .message
                    .slot;
                if next_height > height {
                    break;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }

            self.make_height(height)
        }
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = i64> + '_ {
        async move {
            self.beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .execution
                .timestamp
                .try_into()
                .unwrap()
        }
    }

    fn self_client_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = ClientStateOf<Self>> + '_ {
        async move {
            let genesis = self.beacon_api_client.genesis().await.unwrap().data;

            Any(wasm::client_state::ClientState {
                data: ethereum::client_state::ClientState {
                    chain_id: self.chain_id,
                    genesis_validators_root: genesis.genesis_validators_root,
                    genesis_time: genesis.genesis_time,
                    fork_parameters: self
                        .beacon_api_client
                        .spec()
                        .await
                        .unwrap()
                        .data
                        .into_fork_parameters(),
                    // REVIEW: Is this a preset config param? Or a per-chain config?
                    seconds_per_slot: C::SECONDS_PER_SLOT::U64,
                    slots_per_epoch: C::SLOTS_PER_EPOCH::U64,
                    epochs_per_sync_committee_period: C::EPOCHS_PER_SYNC_COMMITTEE_PERIOD::U64,
                    trusting_period: 100_000_000,
                    latest_slot: beacon_height.revision_height,
                    min_sync_committee_participants: 0,
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    frozen_height: None,
                    counterparty_commitment_slot: U256::from(0),
                },
                code_id: H256::default(),
                latest_height: beacon_height,
            })
        }
    }

    fn self_consensus_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = ConsensusStateOf<Self>> + '_ {
        async move {
            let trusted_header = self
                .beacon_api_client
                .header(beacon_api::client::BlockId::Slot(
                    beacon_height.revision_height,
                ))
                .await
                .unwrap()
                .data;

            let bootstrap = self
                .beacon_api_client
                .bootstrap(trusted_header.root)
                .await
                .unwrap()
                .data;

            assert!(bootstrap.header.beacon.slot == beacon_height.revision_height);

            let light_client_update = {
                let current_period = beacon_height.revision_height.div(C::PERIOD::U64);

                tracing::info!(%current_period);

                let light_client_updates = self
                    .beacon_api_client
                    .light_client_updates(current_period, 1)
                    .await
                    .unwrap();

                let [light_client_update] = &*light_client_updates.0 else {
                    panic!()
                };

                light_client_update.data.clone()
            };

            let timestamp = bootstrap.header.execution.timestamp;
            Any(wasm::consensus_state::ConsensusState {
                data: ethereum::consensus_state::ConsensusState {
                    slot: bootstrap.header.beacon.slot,
                    // REVIEW: Should this be default?
                    storage_root: H256::default(),
                    timestamp,
                    current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                    next_sync_committee: light_client_update
                        .next_sync_committee
                        .map(|nsc| nsc.aggregate_pubkey),
                },
                timestamp,
            })
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: ChannelId,
        destination_port_id: PortId,
        sequence: u64,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let filter = self
                .readonly_ibc_handler
                .write_acknowledgement_filter()
                .filter
                .at_block_hash(block_hash);

            let log = self.provider.get_logs(&filter).await.unwrap();

            log.into_iter()
                .map(|log| <WriteAcknowledgementFilter as EthLogDecode>::decode_log(&log.into()))
                .find_map(|e| match e {
                    Ok(WriteAcknowledgementFilter {
                        destination_port_id: ack_dst_port_id,
                        destination_channel,
                        sequence: ack_sequence,
                        acknowledgement,
                    }) if ack_dst_port_id == destination_port_id.to_string()
                        && destination_channel == destination_channel_id.to_string()
                        && sequence == ack_sequence =>
                    {
                        Some(acknowledgement)
                    }
                    _ => None,
                })
                .unwrap_or_default()
                .to_vec()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EvmInitError {
    #[error("unable to connect to websocket")]
    Ws(#[from] WsClientError),
    #[error("provider error")]
    Provider(#[from] ProviderError),
}

impl<C: ChainSpec> Evm<C> {
    pub async fn new(config: Config) -> Result<Self, EvmInitError> {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await?);

        let chain_id = provider.get_chainid().await?;

        let ibc_handlers = config.signers.into_iter().map(|signer| {
            let signing_key: ecdsa::SigningKey = signer.value();
            let address = secret_key_to_address(&signing_key);

            let wallet = LocalWallet::new_with_signer(signing_key, address, chain_id.as_u64());

            let signer_middleware = Arc::new(SignerMiddleware::new(
                NonceManagerMiddleware::new(provider.clone(), address),
                wallet.clone(),
            ));

            IBCHandler::new(
                config.ibc_handler_address.clone(),
                signer_middleware.clone(),
            )
        });

        Ok(Self {
            chain_id: U256(chain_id),
            ibc_handlers: Pool::new(ibc_handlers),
            readonly_ibc_handler: IBCHandler::new(
                config.ibc_handler_address,
                provider.clone().into(),
            ),
            provider,
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
            // wallet,
            hasura_client: config.hasura_config.map(|hasura_config| {
                HasuraDataStore::new(
                    reqwest::Client::new(),
                    hasura_config.url,
                    hasura_config.secret,
                )
            }),
        })
    }

    // TODO: Change to take a beacon slot instead of a height
    pub async fn execution_height(&self, beacon_height: Height) -> u64 {
        let response = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Slot(
                beacon_height.revision_height,
            ))
            .await;

        let height = response
            .unwrap()
            .data
            .message
            .body
            .execution_payload
            .block_number;

        tracing::debug!("beacon height {beacon_height} is execution height {height}");

        height
    }

    pub fn make_height(&self, height: impl Into<u64>) -> Height {
        // NOTE: Revision is always 0 for EVM
        // REVIEW: Consider using the fork revision?
        {
            Height {
                revision_number: 0,
                revision_height: height.into(),
            }
        }
    }

    pub async fn read_ibc_state<Call>(
        &self,
        call: Call,
        at_execution_height: u64,
    ) -> Result<
        Option<<<Call as EthCallExt>::Return as TupleToOption>::Inner>,
        ContractError<Provider<Ws>>,
    >
    where
        Call: EthCallExt + 'static,
        Call::Return: TupleToOption,
    {
        self.readonly_ibc_handler
            .method_hash::<Call, Call::Return>(Call::selector(), call)
            .expect("valid contract selector")
            .block(at_execution_height)
            .call()
            .await
            .map(Call::Return::tuple_to_option)
    }
}

pub type EvmClientId = ClientId;

// TODO: Don't use debug here, instead impl Error for all error types
#[derive(Debug, thiserror::Error)]
pub enum EvmEventSourceError {
    #[error(transparent)]
    Contract(#[from] ContractError<Provider<Ws>>),
    #[error("channel `{channel_id}/{port_id}` not found")]
    ChannelNotFound { port_id: String, channel_id: String },
    #[error("{0:?}")]
    ChannelConversion(TryFromEthAbiErrorOf<Channel>),
    #[error("channel `{connection_id}` not found")]
    ConnectionNotFound { connection_id: String },
    #[error("{0:?}")]
    ConnectionConversion(TryFromEthAbiErrorOf<ConnectionEnd<EvmClientId, String>>),
    // this is a mess, should be cleaned up
    #[error("{0:?}")]
    ConnectionOpenInitConnectionConversion(
        TryFromEthAbiErrorOf<ConnectionEnd<EvmClientId, String, EmptyString>>,
    ),
    #[error(transparent)]
    ClientIdParse(<ClientId as FromStr>::Err),
    #[error(transparent)]
    ConnectionIdParse(<ConnectionId as FromStr>::Err),
    #[error(transparent)]
    ChannelIdParse(<ChannelId as FromStr>::Err),
    #[error(transparent)]
    PortIdParse(<PortId as FromStr>::Err),
    #[error(transparent)]
    EthAbi(#[from] ethers::core::abi::Error),
}

impl<C: ChainSpec> EventSource for Evm<C> {
    type Event = ChainEvent<Self>;
    type Error = EvmEventSourceError;

    // TODO: Make this the height to start from
    type Seed = ();

    fn events(self, _seed: Self::Seed) -> impl Stream<Item = Result<Self::Event, Self::Error>> {
        async move {
            let genesis_time = self
                .beacon_api_client
                .genesis()
                .await
                .unwrap()
                .data
                .genesis_time;

            let latest_height = self.query_latest_height().await;

            stream::unfold(
                (self, latest_height),
                move |(this, previous_beacon_height)| async move {
                    tracing::info!("fetching events");

                    let current_beacon_height = loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        let current_beacon_height = this.query_latest_height().await;

                        tracing::debug!(%current_beacon_height, %previous_beacon_height);

                        if current_beacon_height > previous_beacon_height {
                            break current_beacon_height;
                        }
                    };
                    tracing::debug!(
                        previous_beacon_height = previous_beacon_height.revision_height,
                        current_beacon_height = current_beacon_height.revision_height
                    );
                    let previous_execution_height =
                        this.execution_height(previous_beacon_height).await;
                    let current_execution_height =
                        this.execution_height(current_beacon_height).await;

                    let packets = futures::stream::iter(
                        this.provider
                            .get_logs(
                                &this
                                    .readonly_ibc_handler
                                    .events()
                                    .filter
                                    .from_block(previous_execution_height)
                                    .to_block(current_execution_height - 1),
                            )
                            .await
                            .unwrap(),
                    )
                    .then(|log| async {
                        // dbg!(&log);

                        let block_hash = log.block_hash.expect("log should have block_hash");

                        // let event_height = this.make_height(
                        //     log.block_number.expect("log should have block_number").0[0],
                        // );
                        let event_height =
                            log.block_number.expect("log should have block_number").0[0];

                        let event = IBCHandlerEvents::decode_log(&log.into());
                        let event = match event {
                            Ok(x) => x,
                            Err(e) => {
                                tracing::error!(
                                    error = ?e,
                                    "failed to decode ibc handler event"
                                );
                                return Ok::<_, EvmEventSourceError>(None::<ChainEvent<Evm<C>>>);
                            }
                        };

                        let read_channel = |port_id: String, channel_id: String| async {
                            this.read_ibc_state(
                                GetChannelCall {
                                    port_id: port_id.clone(),
                                    channel_id: channel_id.clone(),
                                },
                                event_height,
                            )
                            .await
                            .map_err(EvmEventSourceError::Contract)?
                            .ok_or(EvmEventSourceError::ChannelNotFound {
                                port_id,
                                channel_id,
                            })?
                            .try_into()
                            .map_err(EvmEventSourceError::ChannelConversion)
                        };

                        let read_connection = |connection_id: String| async {
                            this.read_ibc_state(
                                GetConnectionCall {
                                    connection_id: connection_id.clone(),
                                },
                                event_height,
                            )
                            .await
                            .map_err(EvmEventSourceError::Contract)?
                            .ok_or(EvmEventSourceError::ConnectionNotFound { connection_id })?
                            .try_into()
                            .map_err(EvmEventSourceError::ConnectionConversion)
                        };

                        let event = match event {
                            IBCHandlerEvents::AcknowledgePacketFilter(packet_ack) => {
                                // TODO: Would be nice if this info was passed through in the SendPacket event
                                let channel_data: Channel = read_channel(
                                    packet_ack.packet.source_port.clone(),
                                    packet_ack.packet.source_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::AcknowledgePacket(AcknowledgePacket {
                                    packet_timeout_height: packet_ack.packet.timeout_height.into(),
                                    packet_timeout_timestamp: packet_ack.packet.timeout_timestamp,
                                    packet_sequence: packet_ack.packet.sequence,
                                    packet_src_port: packet_ack
                                        .packet
                                        .source_port
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    packet_src_channel: packet_ack
                                        .packet
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    packet_dst_port: packet_ack
                                        .packet
                                        .destination_port
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    packet_dst_channel: packet_ack
                                        .packet
                                        .destination_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    packet_channel_ordering: channel_data.ordering,
                                    connection_id: channel_data.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelCloseConfirmFilter(_) => todo!(),
                            IBCHandlerEvents::ChannelCloseInitFilter(_) => todo!(),
                            IBCHandlerEvents::ChannelOpenAckFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenAck(ChannelOpenAck {
                                    port_id: event
                                        .port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    counterparty_port_id: channel.counterparty.port_id,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenConfirmFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenConfirm(ChannelOpenConfirm {
                                    port_id: event
                                        .port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    counterparty_port_id: channel.counterparty.port_id,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenInitFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenInit(ChannelOpenInit {
                                    port_id: event
                                        .port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    // TODO: Ensure that event.counterparty_channel_id is `EmptyString`
                                    counterparty_channel_id: ""
                                        .to_string()
                                        .validate()
                                        .expect("empty string is a valid empty string; qed;"),
                                    counterparty_port_id: event
                                        .counterparty_port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    version: channel.version,
                                }))
                            }
                            IBCHandlerEvents::ChannelOpenTryFilter(event) => {
                                let channel =
                                    read_channel(event.port_id.clone(), event.channel_id.clone())
                                        .await?;

                                Some(IbcEvent::ChannelOpenTry(ChannelOpenTry {
                                    port_id: event
                                        .port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    channel_id: event
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    counterparty_port_id: event
                                        .counterparty_port_id
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    counterparty_channel_id: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    version: event.version,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenAckFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenAck(ConnectionOpenAck {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenConfirmFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenConfirm(ConnectionOpenConfirm {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenInitFilter(event) => {
                                let connection: ConnectionEnd<
                                    <Self as Chain>::ClientId,
                                    String,
                                    EmptyString,
                                > = this
                                    .read_ibc_state(
                                        GetConnectionCall {
                                            connection_id: event.connection_id.clone(),
                                        },
                                        event_height,
                                    )
                                    .await
                                    .map_err(EvmEventSourceError::Contract)?
                                    .ok_or(EvmEventSourceError::ConnectionNotFound {
                                        connection_id: event.connection_id.clone(),
                                    })?
                                    .try_into()
                                    .map_err(
                                        EvmEventSourceError::ConnectionOpenInitConnectionConversion,
                                    )?;

                                Some(IbcEvent::ConnectionOpenInit(ConnectionOpenInit {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::ConnectionOpenTryFilter(event) => {
                                let connection: ConnectionEnd<<Self as Chain>::ClientId, String> =
                                    read_connection(event.connection_id.clone()).await?;

                                Some(IbcEvent::ConnectionOpenTry(ConnectionOpenTry {
                                    connection_id: event
                                        .connection_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ConnectionIdParse)?,
                                    client_id: connection.client_id,
                                    counterparty_client_id: connection.counterparty.client_id,
                                    counterparty_connection_id: connection
                                        .counterparty
                                        .connection_id,
                                }))
                            }
                            IBCHandlerEvents::GeneratedClientIdentifierFilter(event) => {
                                let client_type = this
                                    .readonly_ibc_handler
                                    .client_types(event.0.clone())
                                    .await
                                    .map_err(EvmEventSourceError::Contract)?;

                                let (client_state, success) = this
                                    .readonly_ibc_handler
                                    .get_client_state(event.0.clone())
                                    .await
                                    .unwrap();

                                assert!(success);

                                let client_state = <Any<
                                    wasm::client_state::ClientState<
                                        cometbls::client_state::ClientState,
                                    >,
                                >>::try_from_proto_bytes(
                                    &client_state
                                )
                                .unwrap();

                                Some(IbcEvent::CreateClient(CreateClient {
                                    client_id: event
                                        .0
                                        .parse()
                                        .map_err(EvmEventSourceError::ClientIdParse)?,
                                    client_type,
                                    consensus_height: client_state.0.latest_height,
                                }))
                            }
                            IBCHandlerEvents::RecvPacketFilter(event) => {
                                let channel = read_channel(
                                    event.packet.destination_port.clone(),
                                    event.packet.destination_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::RecvPacket(RecvPacket {
                                    packet_data_hex: event.packet.data.to_vec(),
                                    packet_timeout_height: event.packet.timeout_height.into(),
                                    packet_timeout_timestamp: event.packet.timeout_timestamp,
                                    packet_sequence: event.packet.sequence,
                                    packet_src_port: event
                                        .packet
                                        .source_port
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    packet_src_channel: event
                                        .packet
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    packet_dst_port: event
                                        .packet
                                        .destination_port
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    packet_dst_channel: event
                                        .packet
                                        .destination_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    packet_channel_ordering: channel.ordering,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::SendPacketFilter(event) => {
                                let channel = read_channel(
                                    event.source_port.clone(),
                                    event.source_channel.clone(),
                                )
                                .await?;

                                Some(IbcEvent::SendPacket(SendPacket {
                                    packet_data_hex: event.data.to_vec(),
                                    packet_timeout_height: event.timeout_height.into(),
                                    packet_timeout_timestamp: event.timeout_timestamp,
                                    packet_sequence: event.sequence,
                                    packet_src_port: event
                                        .source_port
                                        .parse()
                                        .map_err(EvmEventSourceError::PortIdParse)?,
                                    packet_src_channel: event
                                        .source_channel
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    // REVIEW: Should we query the packet instead? Or is that the same info? Is it even possible to
                                    // query packets from the evm?
                                    packet_dst_port: channel.counterparty.port_id,
                                    packet_dst_channel: channel
                                        .counterparty
                                        .channel_id
                                        .parse()
                                        .map_err(EvmEventSourceError::ChannelIdParse)?,
                                    packet_channel_ordering: channel.ordering,
                                    connection_id: channel.connection_hops[0].clone(),
                                }))
                            }
                            IBCHandlerEvents::WriteAcknowledgementFilter(_) => None,
                            IBCHandlerEvents::TimeoutPacketFilter(_) => None,
                        };

                        Ok(event.map(|event| {
                            ChainEvent::<Evm<C>> {
                                // TODO: Cache
                                chain_id: this.chain_id(),
                                block_hash: block_hash.into(),
                                height: current_beacon_height,
                                event,
                            }
                        }))
                    })
                    .filter_map(|x| async { x.transpose() })
                    .then(
                        |event: Result<ChainEvent<Evm<C>>, EvmEventSourceError>| async {
                            if let Ok(ref event) = event {
                                let current_slot = event.height.revision_height;

                                let next_epoch_ts =
                                    next_epoch_timestamp::<C>(current_slot, genesis_time);

                                if let Some(hc) = &this.hasura_client {
                                    hc.do_post::<InsertDemoTx>(insert_demo_tx::Variables {
                                        data: serde_json::json! {{
                                            "latest_execution_block_hash": event.block_hash,
                                            "timestamp": next_epoch_ts,
                                        }},
                                    })
                                    .await
                                    .unwrap();
                                }
                            }

                            // pass it back through
                            event
                        },
                    );

                    let iter = futures::stream::iter(packets.collect::<Vec<_>>().await);

                    Some((iter, (this, current_beacon_height)))
                },
            )
            .flatten()
        }
        .flatten_stream()
    }
}

/// Many contract calls return some form of [`(bool, T)`] as a way to emulate nullable/[`Option`].
/// This trait allows for easy conversion from the aforementioned tuple to an [`Option`].
pub trait TupleToOption {
    type Inner;

    fn tuple_to_option(self) -> Option<Self::Inner>;
}

impl TupleToOption for GetClientStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0.to_vec())
    }
}

impl TupleToOption for GetConsensusStateReturn {
    type Inner = Vec<u8>;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.p1.then_some(self.consensus_state_bytes.to_vec())
    }
}

impl TupleToOption for GetConnectionReturn {
    type Inner = IbcCoreConnectionV1ConnectionEndData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetChannelReturn {
    type Inner = IbcCoreChannelV1ChannelData;

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

impl TupleToOption for GetHashedPacketAcknowledgementCommitmentReturn {
    type Inner = [u8; 32];

    fn tuple_to_option(self) -> Option<Self::Inner> {
        self.1.then_some(self.0)
    }
}

/// Wrapper trait for a contract call's signature, to map the input type to the return type.
/// `ethers` generates both of these types, but doesn't correlate them.
pub trait EthCallExt: EthCall {
    type Return: Tokenizable;
}

macro_rules! impl_eth_call_ext {
    ($($Call:ident -> $Return:ident;)+) => {
        $(
            impl EthCallExt for $Call {
                type Return = $Return;
            }
        )+
    }
}

impl_eth_call_ext! {
    GetClientStateCall                           -> GetClientStateReturn;
    GetConsensusStateCall                        -> GetConsensusStateReturn;
    GetConnectionCall                            -> GetConnectionReturn;
    GetChannelCall                               -> GetChannelReturn;
    GetHashedPacketCommitmentCall                -> GetHashedPacketCommitmentReturn;
    GetHashedPacketAcknowledgementCommitmentCall -> GetHashedPacketAcknowledgementCommitmentReturn;
}

pub fn next_epoch_timestamp<C: ChainSpec>(slot: u64, genesis_timestamp: u64) -> u64 {
    let next_epoch_slot = slot + (C::SLOTS_PER_EPOCH::U64 - (slot % C::SLOTS_PER_EPOCH::U64));
    genesis_timestamp + (next_epoch_slot * C::SECONDS_PER_SLOT::U64)
}

impl<Counterparty, C, P> IbcStateRead<Counterparty, P> for Evm<C>
where
    Counterparty: Chain,
    C: ChainSpec,
    P: IbcPath<Evm<C>, Counterparty>
        + EthereumStateRead<
            C,
            Counterparty,
            Encoded = <<<P as EthereumStateRead<C, Counterparty>>::EthCall as EthCallExt>::Return as TupleToOption>::Inner,
        > + 'static,
    <P::EthCall as EthCallExt>::Return: TupleToOption,
{
    fn proof(
        &self,
        path: P,
        at: HeightOf<Self>,
    ) -> impl Future<Output = Vec<u8>> + '_ {
        async move {
            let execution_height = self.execution_height(at).await;

            let path = path.to_string();

            let location = keccak256(
                keccak256(path.as_bytes())
                    .into_iter()
                    .chain(ethers::types::U256::from(0).encode())
                    .collect::<Vec<_>>(),
            );

            let proof = self
                .provider
                .get_proof(
                    self.readonly_ibc_handler.address(),
                    vec![location.into()],
                    Some(execution_height.into()),
                )
                .await
                .unwrap();

            tracing::info!(?proof);

            let proof = match <[_; 1]>::try_from(proof.storage_proof) {
                Ok([proof]) => proof,
                Err(invalid) => {
                    panic!("received invalid response from eth_getProof, expected length of 1 but got `{invalid:#?}`");
                }
            };

                protos::union::ibc::lightclients::ethereum::v1::StorageProof {
                    proofs: [protos::union::ibc::lightclients::ethereum::v1::Proof {
                        key: proof.key.to_fixed_bytes().to_vec(),
                        // REVIEW(benluelo): Make sure this encoding works
                        value: proof.value.encode(),
                        proof: proof
                            .proof
                            .into_iter()
                            .map(|bytes| bytes.to_vec())
                            .collect(),
                    }]
                    .to_vec(),
                }
                .encode_to_vec()
        }
    }

    fn state(&self, path: P, at: HeightOf<Self>) -> impl Future<Output = <P as IbcPath<Evm<C>, Counterparty>>::Output> + '_
    {
        async move {
            let execution_block_number = self.execution_height(at).await;

            self.read_ibc_state(path.into_eth_call(), execution_block_number)
                .await
                .unwrap()
                .map(|x| P::decode_ibc_state(x))
                .unwrap()
        }
    }
}

pub trait EthereumStateRead<C, Counterparty>: IbcPath<Evm<C>, Counterparty>
where
    Counterparty: Chain,
    C: ChainSpec,
{
    /// The type of the encoded state returned from the contract. This may be bytes (see client state)
    /// or a type (see connection end)
    /// Since solidity doesn't support generics, it emulates generics by using bytes in interfaces and
    /// "downcasting" (via parsing) to expected types in implementations.
    type Encoded;

    type EthCall: EthCallExt + 'static;

    fn into_eth_call(self) -> Self::EthCall;

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output;
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty>
    for ClientStatePath<<Evm<C> as Chain>::ClientId>
where
    ClientStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ClientStateOf<Counterparty>>: Debug,
{
    type Encoded = Vec<u8>;

    type EthCall = GetClientStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        TryFromProto::try_from_proto_bytes(&encoded).unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty>
    for ClientConsensusStatePath<ClientIdOf<Evm<C>>, HeightOf<Counterparty>>
where
    ConsensusStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ConsensusStateOf<Counterparty>>: Debug,
{
    type Encoded = Vec<u8>;

    type EthCall = GetConsensusStateCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            client_id: self.client_id.to_string(),
            height: self.height.into_height().into(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        TryFromProto::try_from_proto_bytes(&encoded).unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for ConnectionPath {
    type Encoded = IbcCoreConnectionV1ConnectionEndData;

    type EthCall = GetConnectionCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            connection_id: self.connection_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.try_into().unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for ChannelEndPath {
    type Encoded = IbcCoreChannelV1ChannelData;

    type EthCall = GetChannelCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.try_into().unwrap()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for CommitmentPath {
    type Encoded = [u8; 32];

    type EthCall = GetHashedPacketCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence,
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.into()
    }
}

impl<C: ChainSpec, Counterparty: Chain> EthereumStateRead<C, Counterparty> for AcknowledgementPath {
    type Encoded = [u8; 32];

    type EthCall = GetHashedPacketAcknowledgementCommitmentCall;

    fn into_eth_call(self) -> Self::EthCall {
        Self::EthCall {
            port_id: self.port_id.to_string(),
            channel_id: self.channel_id.to_string(),
            sequence: self.sequence,
        }
    }

    fn decode_ibc_state(encoded: Self::Encoded) -> Self::Output {
        encoded.into()
    }
}

pub async fn bind_port<C: ChainSpec>(this: &Evm<C>, module_address: Address, port_id: String) {
    // HACK: This will pop the top item out of the queue, but binding the port requires the contract owner;
    // this will work as long as the first signer in the list is the owner.
    this.ibc_handlers
        .with(|ibc_handler| async move {
            let bind_port_result = ibc_handler.bind_port(port_id, module_address.into());

            match bind_port_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => eprintln!("{:?}", why.decode_revert::<String>()),
            };
        })
        .await
}

#[allow(unused_variables)]
pub async fn setup_initial_channel<C: ChainSpec>(
    this: &Evm<C>,
    module_address: Address,
    channel_id: String,
    port_id: String,
    counterparty_port_id: String,
) {
    // let signer_middleware = Arc::new(SignerMiddleware::new(
    //     this.provider.clone(),
    //     this.wallet.clone(),
    // ));

    // let ibc_handler = devnet_ownable_ibc_handler::DevnetOwnableIBCHandler::new(
    //     this.ibc_handler.address(),
    //     signer_middleware,
    // );

    // ibc_handler
    //     .setup_initial_channel(
    //         "connection-0".into(),
    //         IbcCoreConnectionV1ConnectionEndData {
    //             client_id: "cometbls-new-0".into(),
    //             versions: vec![IbcCoreConnectionV1VersionData {
    //                 identifier: "1".into(),
    //                 features: vec!["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()],
    //             }],
    //             state: 3,
    //             counterparty: IbcCoreConnectionV1CounterpartyData {
    //                 client_id: "08-wasm-0".into(),
    //                 connection_id: "connection-0".into(),
    //                 prefix: IbcCoreCommitmentV1MerklePrefixData {
    //                     key_prefix: b"ibc".to_vec().into(),
    //                 },
    //             },
    //             delay_period: 6,
    //         },
    //         port_id,
    //         channel_id.clone(),
    //         IbcCoreChannelV1ChannelData {
    //             state: 3,
    //             ordering: 1,
    //             counterparty: IbcCoreChannelV1CounterpartyData {
    //                 port_id: counterparty_port_id,
    //                 channel_id,
    //             },
    //             connection_hops: vec!["connection-0".into()],
    //             version: "ics20-1".into(),
    //         },
    //         module_address.into(),
    //     )
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .unwrap();
    todo!()
}
