use std::{fmt::Display, num::ParseIntError, str::FromStr};

use ethers::prelude::k256::ecdsa;
use futures::{stream, Future, FutureExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tendermint_rpc::{
    event::EventData,
    query::{Condition, EventType, Operand, Query},
    Client, SubscriptionClient, WebSocketClient, WebSocketClientUrl,
};
use unionlabs::{
    ethereum::H256,
    events::{IbcEvent, TryFromTendermintEventError, WriteAcknowledgement},
    ibc::{
        core::{client::height::Height, commitment::merkle_root::MerkleRoot},
        google::protobuf::{any::Any, duration::Duration},
        lightclients::{cometbls, wasm},
    },
    id::Id,
    id_type,
    tendermint::abci::{event::Event, event_attribute::EventAttribute},
    CosmosAccountId,
};

use crate::{private_key::PrivateKey, Chain, ChainEvent, ClientState, EventSource};

#[derive(Debug, Clone)]
pub struct Union {
    pub chain_id: String,
    pub signer: CosmosAccountId,
    pub fee_denom: String,
    pub tm_client: WebSocketClient,
    pub chain_revision: u64,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signer: PrivateKey<ecdsa::SigningKey>,
    pub fee_denom: String,
    pub ws_url: WebSocketClientUrl,
    pub prover_endpoint: String,
    pub grpc_url: String,
}

impl Chain for Union {
    type SelfClientState =
        Any<wasm::client_state::ClientState<cometbls::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>>;

    type Header = cometbls::header::Header;

    type Height = Height;

    type ClientId = UnionClientId;

    type ClientType = UnionClientType;

    fn chain_id(&self) -> <Self::SelfClientState as ClientState>::ChainId {
        self.chain_id.clone()
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .tm_client
                .latest_block()
                .await
                .unwrap()
                .block
                .header
                .height
                .value()
                // HACK: for some reason, abci_query on latest block return null
                // value sometimes, probably a racy condition if we use the
                // actually latest block being built?
                .saturating_sub(1);

            self.make_height(height)
        }
    }

    fn query_latest_height_as_destination(&self) -> impl Future<Output = Height> + '_ {
        self.query_latest_height()
    }

    fn query_latest_timestamp(&self) -> impl Future<Output = i64> + '_ {
        async move {
            let height = self.query_latest_height().await;
            self.tm_client
                .block(u32::try_from(height.revision_height).unwrap())
                .await
                .unwrap()
                .block
                .header
                .time
                .unix_timestamp()
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

            Any(wasm::client_state::ClientState {
                data: cometbls::client_state::ClientState {
                    chain_id: self.chain_id.clone(),
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    trusting_period: Duration::new(
                        (unbonding_period * 85 / 100).as_secs() as i64,
                        (unbonding_period * 85 / 100).subsec_nanos() as i32,
                    )
                    .unwrap(),
                    unbonding_period: Duration::new(
                        unbonding_period.as_secs() as i64,
                        unbonding_period.subsec_nanos() as i32,
                    )
                    .unwrap(),
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    max_clock_drift: Duration::new(60 * 10, 0).unwrap(),
                    frozen_height: Height {
                        revision_number: 0,
                        revision_height: 0,
                    },
                },
                // TODO: Get this somehow
                code_id: H256::default(),
                latest_height: Height {
                    revision_number: self.chain_id.split('-').last().unwrap().parse().unwrap(),
                    revision_height: height.value(),
                },
            })
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

            let state = cometbls::consensus_state::ConsensusState {
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
            };

            Any(wasm::consensus_state::ConsensusState {
                data: state,
                timestamp: commit
                    .signed_header
                    .header
                    .time
                    .unix_timestamp()
                    .try_into()
                    .unwrap(),
            })
        }
    }

    fn read_ack(
        &self,
        block_hash: H256,
        destination_channel_id: unionlabs::id::ChannelId,
        destination_port_id: String,
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

            let wa = self
                .tm_client
                .tx_search(
                    Query::from(EventType::Tx).and_eq("tx.height", u64::from(block_height)),
                    false,
                    0,
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
                                    && ok.packet_src_port == destination_port_id
                                    && ok.packet_src_channel == destination_channel_id =>
                            {
                                Some(ok)
                            }
                            Ok(_) => None,
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
            signer: CosmosAccountId::new(config.signer.value(), "union".to_string()),
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            grpc_url: config.grpc_url,
            fee_denom: config.fee_denom,
        })
    }

    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any>,
    ) {
        use protos::cosmos::tx;

        let account = self.account_info_of_signer(&self.signer).await;

        let sign_doc = tx::v1beta1::SignDoc {
            body_bytes: tx::v1beta1::TxBody {
                messages: messages.into_iter().collect(),
                // TODO(benluelo): What do we want to use as our memo?
                memo: String::new(),
                timeout_height: 123_123_123,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }
            .encode_to_vec(),
            auth_info_bytes: tx::v1beta1::AuthInfo {
                signer_infos: [tx::v1beta1::SignerInfo {
                    public_key: Some(protos::google::protobuf::Any {
                        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                        value: self.signer.public_key().encode_to_vec(),
                    }),
                    mode_info: Some(tx::v1beta1::ModeInfo {
                        sum: Some(tx::v1beta1::mode_info::Sum::Single(
                            tx::v1beta1::mode_info::Single {
                                mode: tx::signing::v1beta1::SignMode::Direct.into(),
                            },
                        )),
                    }),
                    sequence: account.sequence,
                }]
                .to_vec(),
                fee: Some(tx::v1beta1::Fee {
                    amount: vec![protos::cosmos::base::v1beta1::Coin {
                        // TODO: This needs to be configurable
                        denom: self.fee_denom.clone(),
                        amount: "1".to_string(),
                    }],
                    gas_limit: 5_000_000_000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }
            .encode_to_vec(),
            chain_id: self.chain_id.clone(),
            account_number: account.account_number,
        };

        let alice_signature = self
            .signer
            .try_sign(&sign_doc.encode_to_vec())
            .unwrap()
            .to_vec();

        let tx_raw = tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes,
            auth_info_bytes: sign_doc.auth_info_bytes,
            signatures: [alice_signature].to_vec(),
        };

        let tx_raw_bytes = tx_raw.encode_to_vec();

        let tx_hash = hex::encode_upper(sha2::Sha256::new().chain_update(&tx_raw_bytes).finalize());

        let query = Query {
            event_type: Some(EventType::Tx),
            conditions: [Condition::eq(
                "tx.hash".to_string(),
                Operand::String(tx_hash.clone()),
            )]
            .into(),
        };

        loop {
            if self
                .tm_client
                .tx(tx_hash.parse().unwrap(), false)
                .await
                .is_ok()
            {
                // TODO: Log an error if this is unsuccessful
                let _ = self.tm_client.unsubscribe(query).await;
                return;
            }

            // dbg!(maybe_tx);

            let response_result = self.tm_client.broadcast_tx_sync(tx_raw_bytes.clone()).await;

            // dbg!(&response_result);

            let response = response_result.unwrap();

            assert_eq!(tx_hash, response.hash.to_string());

            tracing::debug!(%tx_hash);

            tracing::info!(check_tx_code = ?response.code, check_tx_log = %response.log);

            if response.code.is_err() {
                panic!("check_tx failed: {:?}", response)
            };

            // HACK: wait for a block to verify inclusion
            tokio::time::sleep(std::time::Duration::from_secs(7)).await;

            let tx_inclusion = self.tm_client.tx(tx_hash.parse().unwrap(), false).await;

            tracing::debug!(?tx_inclusion);

            match tx_inclusion {
                Ok(x) => x,
                Err(_) => {
                    // TODO: we don't handle this case, either we got an error or the tx hasn't been received
                    // we need to discriminate
                    tracing::warn!("tx inclusion couldn't be retrieved after 1 block");
                    panic!()
                }
            };
        }
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn account_info_of_signer(
        &self,
        signer: &CosmosAccountId,
    ) -> protos::cosmos::auth::v1beta1::BaseAccount {
        let account = protos::cosmos::auth::v1beta1::query_client::QueryClient::connect(
            self.grpc_url.clone(),
        )
        .await
        .unwrap()
        .account(protos::cosmos::auth::v1beta1::QueryAccountRequest {
            address: signer.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

        // TODO: Type in `unionlabs` for this
        assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

        protos::cosmos::auth::v1beta1::BaseAccount::decode(&*account.value).unwrap()
    }
}

// TODO: This is for all cosmos chains; rename?
crate::chain_client_id! {
    #[ty = UnionClientType]
    pub enum UnionClientId {
        #[id(ty = "08-wasm")]
        Wasm(Id<_>),
        #[id(ty = "07-tendermint")]
        Tendermint(Id<_>),
    }
}

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

    fn events(
        &self,
        _seed: Self::Seed,
    ) -> impl Stream<Item = Result<Self::Event, Self::Error>> + '_ {
        let chain_revision = self.chain_revision;

        // TODO: Change this to fetch new blocks instead of subscribing to txs
        let new_events = self
            .tm_client
            .subscribe(EventType::Tx.into())
            .map_err(<Self::Error>::Subscription)
            .map_ok(|s| s.map_err(<Self::Error>::Subscription))
            .into_stream()
            .try_flatten()
            .map_ok(move |event| {
                tracing::info!(?event.data);

                if let EventData::Tx { tx_result } = event.data {
                    Some(
                        stream::iter(tx_result.result.events.into_iter().map(|event| {
                            Event {
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
                            }
                        }))
                        .filter_map(move |event| async move {
                            IbcEvent::try_from_tendermint_event(event)
                        })
                        .map_err(UnionEventSourceError::TryFromTendermintEvent)
                        .and_then(
                            move |event: IbcEvent<_, _, String>| async move {
                                Ok(ChainEvent {
                                    chain_id: self.chain_id.clone(),
                                    block_hash: (self
                                        .tm_client
                                        .block(u32::try_from(tx_result.height).unwrap())
                                        .await
                                        .unwrap()
                                        .block_id
                                        .hash
                                        .as_bytes()
                                        .try_into()
                                        .unwrap()),
                                    height: Height {
                                        revision_number: chain_revision,
                                        revision_height: tx_result.height.try_into().unwrap(),
                                    },
                                    event,
                                })
                            },
                        ),
                    )
                } else {
                    None
                }
            })
            .filter_map(|x| async { x.transpose() })
            .try_flatten();

        // Box::pin(new_events)
        //     .into_future()
        //     .map(move |(read_until_height, new_events)| {
        //         read_until_height
        //             .expect("stream should not be finished")
        //             .map_err(|x| match x {
        //                 fatal @ queue::Error::Fatal(_) => fatal,
        //                 queue::Error::Recoverable(err) => queue::Error::Fatal(Box::new(format!(
        //                     "unable to read first event out of stream: {err:?}"
        //                 ))),
        //             })
        //             .map(|read_until_height| {
        //                 stream::iter(
        //                     (read_until_height
        //                         .height
        //                         .revision_height
        //                         .checked_sub(50)
        //                         .unwrap_or(1))
        //                         ..read_until_height.height.revision_height,
        //                 )
        //                 .then(move |height| {
        //                     let client = self.tm_client.clone();

        //                     async move {
        //                         tracing::info!("querying block results at block {height}");
        //                         client
        //                             .block_results(u32::try_from(height).unwrap())
        //                             .await
        //                             .map_err(|e| queue::Error::Fatal(Box::new(e)))
        //                             .map(|response| {
        //                                 response.txs_results.into_iter().flatten().flat_map(
        //                                     move |tx_result| {
        //                                         ibc_events_from_tx(
        //                                             tx_result.events,
        //                                             response.height.into(),
        //                                             chain_revision,
        //                                         )
        //                                     },
        //                                 )
        //                             })
        //                     }
        //                 })
        //                 .map_ok(stream::iter)
        //                 .try_flatten()
        //                 .chain(stream::iter([Ok(read_until_height)]))
        //                 .chain(new_events)
        //             })
        //     })
        //     .try_flatten_stream()

        // let missed_events = rx
        //     .map(|x| x.expect("channel should not have been cancelled; qed;"))
        //     .map(move |read_until_height| {
        //         stream::iter((read_until_height.checked_sub(50).unwrap_or(1))..read_until_height)
        //             .then(move |height| {
        //                 let client = self.tm_client.clone().clone();

        //                 async move {
        //                     client
        //                         .block_results(u32::try_from(height).unwrap())
        //                         .await
        //                         .map_err(|e| queue::Error::Fatal(Box::new(e)))
        //                         .map(|response| {
        //                             response.txs_results.into_iter().flatten().flat_map(
        //                                 move |tx_result| {
        //                                     ibc_events_from_tx(
        //                                         tx_result.events,
        //                                         response.height.into(),
        //                                         chain_revision,
        //                                     )
        //                                 },
        //                             )
        //                         })
        //                 }
        //             })
        //     })
        //     .flatten_stream()
        //     .map_ok(stream::iter)
        //     .try_flatten()

        new_events
    }
}
