use std::{fmt::Debug, marker::PhantomData};

use clap::Args;
use futures::{Future, FutureExt, Stream, StreamExt};
use num_bigint::BigUint;
use prost::Message;
use protos::{
    cosmos::{
        auth,
        base::tendermint::v1beta1::AbciQueryRequest,
        ics23::v1 as ics23_v1,
        staking::{self, v1beta1::BondStatus},
        tx,
    },
    google,
    ibc::core::{client::v1 as client_v1, commitment::v1 as commitment_v1},
    union::galois::api::v1::{union_prover_api_client, ProveRequest},
};
use sha2::Digest;
use tendermint_rpc::{
    event::EventData, query::EventType, Client, SubscriptionClient, WebSocketClient,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use unionlabs::{
    bounded_int::BoundedI64,
    cosmos::staking::query_validators_response::QueryValidatorsResponse,
    ethereum::H256,
    ethereum_consts_traits::{ChainSpec, PresetBaseKind},
    ibc::{
        core::{
            channel::{
                msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, packet::Packet,
            },
            client::{
                height::Height, msg_create_client::MsgCreateClient,
                msg_update_client::MsgUpdateClient,
            },
            commitment::merkle_root::MerkleRoot,
            connection::{
                msg_connection_open_ack::MsgConnectionOpenAck,
                msg_connection_open_confirm::MsgConnectionOpenConfirm,
                msg_connection_open_init::MsgConnectionOpenInit,
                msg_connection_open_try::MsgConnectionOpenTry,
            },
        },
        google::protobuf::{any::Any, duration::Duration, timestamp::Timestamp},
        lightclients::{cometbls, ethereum, tendermint::fraction::Fraction, wasm},
    },
    tendermint::{
        abci::{event::Event, event_attribute::EventAttribute},
        crypto::public_key::PublicKey,
        types::{
            block_id::BlockId, commit::Commit, part_set_header::PartSetHeader,
            signed_header::SignedHeader, simple_validator::SimpleValidator,
        },
    },
    CosmosAccountId, MsgIntoProto, TryFromProto, TryFromProtoErrorOf,
};

use super::events::TryFromTendermintEventError;
use crate::{
    chain::{
        dumper::Dumper,
        events::{
            ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck,
            ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, SendPacket, UpdateClient,
        },
        evm::{Cometbls, Evm},
        proof::{
            ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
            ConnectionPath, IbcPath, StateProof,
        },
        Chain, ChainConnection, ClientStateOf, Connect, ConsensusStateOf, CreateClient,
        IbcStateRead, LightClient,
    },
    config::UnionChainConfig,
};

/// The 08-wasm light client running on the union chain.
pub struct Ethereum<C: ChainSpec> {
    chain: <Self as LightClient>::HostChain,
    dumper: Dumper,
    _marker: PhantomData<C>,
}

#[derive(Debug, Clone, Args)]
pub struct EthereumConfig {
    #[arg(long)]
    pub evm_preset: PresetBaseKind,
}

#[derive(Debug, Clone)]
pub struct Union {
    pub signer: CosmosAccountId,
    tm_client: WebSocketClient,
    chain_id: String,
    chain_revision: u64,
    // TODO: Move this field back into `Ethereum` once the cometbls states are unwrapped out of the wasm states
    wasm_code_id: H256,
    prover_endpoint: String,
    grpc_url: String,
    dump_path: String,
}

impl<C: ChainSpec> ChainConnection<Evm<C>> for Union {
    type LightClient = Ethereum<C>;

    fn light_client(&self) -> Self::LightClient {
        Ethereum {
            chain: self.clone(),
            dumper: Dumper::new(self.dump_path.clone()),
            _marker: PhantomData,
        }
    }
}

impl<C: ChainSpec> CreateClient<Ethereum<C>> for Union {
    fn create_client(
        &self,
        _config: <Ethereum<C> as LightClient>::Config,
        counterparty_chain: <Ethereum<C> as LightClient>::CounterpartyChain,
    ) -> impl Future<Output = (String, Ethereum<C>)> + '_ {
        async move {
            let latest_height = counterparty_chain.query_latest_height().await;

            let client_state = counterparty_chain.self_client_state(latest_height).await;
            let consensus_state = counterparty_chain.self_consensus_state(latest_height).await;

            let msg = Any(MsgCreateClient {
                client_state,
                consensus_state,
            })
            .into_proto_with_signer(&self.signer);

            let client_id = self
                .broadcast_tx_commit([msg])
                .await
                .deliver_tx
                .events
                .into_iter()
                .find(|event| event.kind == "create_client")
                .unwrap()
                .attributes
                .into_iter()
                .find(|attr| attr.key == "client_id")
                .unwrap()
                .value;

            (
                client_id,
                Ethereum {
                    chain: self.clone(),
                    dumper: Dumper::new(self.dump_path.clone()),
                    _marker: PhantomData,
                },
            )
        }
    }
}

impl Union {
    pub async fn new(config: UnionChainConfig) -> Self {
        let (tm_client, driver) = WebSocketClient::builder(config.ws_url)
            .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
            .build()
            .await
            .unwrap();

        tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client
            .status()
            .await
            .unwrap()
            .node_info
            .network
            .to_string();

        let chain_revision = chain_id.split('-').last().unwrap().parse().unwrap();

        Self {
            signer: CosmosAccountId::new(config.signer.value(), "union".to_string()),
            wasm_code_id: config.wasm_code_id,
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            dump_path: config.dump_path,
            grpc_url: config.grpc_url,
        }
    }

    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = google::protobuf::Any>,
    ) -> tendermint_rpc::endpoint::broadcast::tx_commit::Response {
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
                    public_key: Some(google::protobuf::Any {
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
                        denom: "stake".to_string(),
                        amount: "1".to_string(),
                    }],
                    gas_limit: 5_000_000_000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }
            .encode_to_vec(),
            chain_id: self.chain_id().await,
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

        let response = self
            .tm_client
            .broadcast_tx_commit(tx_raw.encode_to_vec())
            .await
            .unwrap();

        tracing::info!(check_tx_code = ?response.check_tx.code, check_tx_log = %response.check_tx.log);
        tracing::info!(deliver_tx_code = ?response.deliver_tx.code, deliver_tx_log = %response.deliver_tx.log);

        if let tendermint::abci::Code::Err(code) = response.check_tx.code {
            panic!("check_tx failed: {code}")
        };

        if let tendermint::abci::Code::Err(code) = response.deliver_tx.code {
            panic!("deliver_tx failed: {code}")
        };

        response
    }

    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }

    async fn account_info_of_signer(&self, signer: &CosmosAccountId) -> auth::v1beta1::BaseAccount {
        let account = auth::v1beta1::query_client::QueryClient::connect(self.grpc_url.clone())
            .await
            .unwrap()
            .account(auth::v1beta1::QueryAccountRequest {
                address: signer.to_string(),
            })
            .await
            .unwrap()
            .into_inner()
            .account
            .unwrap();

        assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

        auth::v1beta1::BaseAccount::decode(&*account.value).unwrap()
    }
}

impl Chain for Union {
    type SelfClientState =
        Any<wasm::client_state::ClientState<cometbls::client_state::ClientState>>;
    type SelfConsensusState =
        Any<wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>>;

    fn chain_id(&self) -> impl Future<Output = String> + '_ {
        async move { self.chain_id.clone() }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = self
                .tm_client
                .latest_commit()
                .await
                .unwrap()
                .signed_header
                .header
                .height
                .value();

            loop {
                if self
                    .tm_client
                    .latest_commit()
                    .await
                    .unwrap()
                    .signed_header
                    .header
                    .height
                    .value()
                    > height
                {
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }

            self.make_height(height)
        }
    }

    fn self_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = Self::SelfClientState> + '_ {
        async move {
            let params =
                staking::v1beta1::query_client::QueryClient::connect(self.grpc_url.clone())
                    .await
                    .unwrap()
                    .params(staking::v1beta1::QueryParamsRequest {})
                    .await
                    .unwrap()
                    .into_inner()
                    .params
                    .unwrap();

            let commit = self
                .tm_client
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
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
                    chain_id: self.chain_id().await,
                    // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
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
                code_id: self.wasm_code_id.clone(),
                latest_height: Height {
                    revision_number: self
                        .chain_id()
                        .await
                        .split('-')
                        .last()
                        .unwrap()
                        .parse()
                        .unwrap(),
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
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
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

    fn packet_stream(
        &self,
    ) -> impl Future<Output = impl Stream<Item = (Height, Packet)> + '_> + '_ {
        async move {
            let (events_from_now_tx, events_from_now_rx) = tokio::sync::mpsc::unbounded_channel();

            let chain_revision = self.chain_revision;

            let event_stream = self
                .tm_client
                .clone()
                .subscribe(EventType::Tx.into())
                .await
                .unwrap()
                .filter_map(move |event| async move {
                    let event = event.unwrap();
                    tracing::info!(?event.data);
                    match event.data {
                        EventData::Tx { tx_result } => {
                            let send_packet =
                                get_event_from_tx_response::<SendPacket>(tx_result.result.events)?;

                            let event_height = {
                                let height = tx_result.height.try_into().unwrap();
                                Height {
                                    revision_number: chain_revision,
                                    revision_height: height,
                                }
                            };

                            Some((
                                event_height,
                                Packet {
                                    sequence: send_packet.packet_sequence,
                                    source_port: send_packet.packet_src_port,
                                    source_channel: send_packet.packet_src_channel,
                                    destination_port: send_packet.packet_dst_port,
                                    destination_channel: send_packet.packet_dst_channel,
                                    data: send_packet.packet_data_hex,
                                    timeout_height: send_packet.packet_timeout_height,
                                    timeout_timestamp: send_packet.packet_timeout_timestamp,
                                },
                            ))
                        }
                        _ => None,
                    }
                });

            tokio::spawn(event_stream.for_each(move |event| {
                let tx = events_from_now_tx.clone();

                tx.send(event).unwrap();

                futures::future::ready(())
            }));

            // NOTE(benluelo): Commented out for now as this is currently untested. Leaving it here
            // as I'm pretty sure this is the general flow for querying missed packets, and could be useful down the line.

            let (_missed_events_tx, missed_events_rx) = tokio::sync::mpsc::unbounded_channel();

            // let latest_height = self.query_latest_height().await.revision_height;

            // for height in (latest_height.checked_sub(50).unwrap_or(1))..=latest_height {
            //     let block_results = self
            //         .tm_client
            //         .block_results(u32::try_from(height).unwrap())
            //         .await
            //         .unwrap();

            //     if let Some(txs_results) = block_results.txs_results {
            //         for tx_result in txs_results {
            //             if let Some(send_packet) =
            //                 get_event_from_tx_response::<SendPacket>(tx_result.events)
            //             {
            //                 let event_height = self.make_height(height);

            //                 missed_events_tx
            //                     .send((
            //                         event_height,
            //                         Packet {
            //                             sequence: send_packet.packet_sequence.parse().unwrap(),
            //                             source_port: send_packet.packet_src_port,
            //                             source_channel: send_packet.packet_src_channel,
            //                             destination_port: send_packet.packet_dst_port,
            //                             destination_channel: send_packet.packet_dst_channel,
            //                             data: hex::decode(send_packet.packet_data_hex).unwrap(),
            //                             timeout_height: send_packet
            //                                 .packet_timeout_height
            //                                 .parse()
            //                                 .unwrap(),
            //                             timeout_timestamp: send_packet
            //                                 .packet_timeout_timestamp
            //                                 .parse()
            //                                 .unwrap(),
            //                         },
            //                     ))
            //                     .unwrap();
            //             }
            //         }
            //     }
            // }

            UnboundedReceiverStream::new(missed_events_rx)
                .chain(UnboundedReceiverStream::new(events_from_now_rx))
        }
    }
}

impl<C: ChainSpec> LightClient for Ethereum<C> {
    type UpdateClientMessage = wasm::header::Header<ethereum::header::Header<C>>;

    type HostChain = Union;

    type CounterpartyChain = Evm<C>;

    type Config = EthereumConfig;

    fn chain(&self) -> &Self::HostChain {
        &self.chain
    }

    fn update_client(
        &self,
        client_id: String,
        msg: Self::UpdateClientMessage,
    ) -> impl Future<Output = (Height, UpdateClient)> + '_ {
        self.send_msg_and_read_event(MsgUpdateClient {
            client_id,
            client_message: msg,
        })
    }

    fn query_client_state(
        &self,
        client_id: String,
    ) -> impl Future<Output = ClientStateOf<Self::CounterpartyChain>> + '_ {
        async move {
            client_v1::query_client::QueryClient::connect(self.chain.grpc_url.clone())
                .await
                .unwrap()
                .client_state(client_v1::QueryClientStateRequest { client_id })
                .await
                .unwrap()
                .into_inner()
                .client_state
                .unwrap()
                .try_into()
                .unwrap()
        }
    }

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_ {
        async move { height }
    }
}

impl<C: ChainSpec> Connect<Cometbls<C>> for Ethereum<C> {
    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl Future<Output = (Height, ConnectionOpenInit)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<ClientStateOf<<Cometbls<C> as LightClient>::CounterpartyChain>>,
    ) -> impl Future<Output = (Height, ConnectionOpenTry)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<ClientStateOf<<Cometbls<C> as LightClient>::CounterpartyChain>>,
    ) -> impl Future<Output = (Height, ConnectionOpenAck)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = (Height, ConnectionOpenConfirm)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn channel_open_init(
        &self,
        msg: MsgChannelOpenInit,
    ) -> impl Future<Output = (Height, ChannelOpenInit)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn channel_open_try(
        &self,
        msg: MsgChannelOpenTry,
    ) -> impl Future<Output = (Height, ChannelOpenTry)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn channel_open_ack(
        &self,
        msg: MsgChannelOpenAck,
    ) -> impl Future<Output = (Height, ChannelOpenAck)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn channel_open_confirm(
        &self,
        msg: MsgChannelOpenConfirm,
    ) -> impl Future<Output = (Height, ChannelOpenConfirm)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn recv_packet(&self, msg: MsgRecvPacket) -> impl Future<Output = ()> + '_ {
        async move {
            self.chain
                .broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.channel.v1.MsgRecvPacket".to_string(),
                    value: msg
                        .into_proto_with_signer(&self.chain.signer)
                        .encode_to_vec(),
                }])
                .await;
        }
    }

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Cometbls<C>,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let trusted_commit = self
                .chain
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(update_from.revision_height)
                        .unwrap(),
                )
                .await
                .unwrap();

            self.dumper.dump(
                format!(
                    "commit-{:06}",
                    trusted_commit.signed_header.header.height.value()
                ),
                &trusted_commit,
            );

            let commit = self
                .chain
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(update_to.revision_height)
                        .unwrap(),
                )
                .await
                .unwrap();

            self.dumper.dump(
                format!("commit-{:06}", commit.signed_header.header.height.value()),
                &commit,
            );

            tracing::debug!("New block {:?}", commit.signed_header.header.height);

            // TODO: Add to self
            let mut staking_client =
                staking::v1beta1::query_client::QueryClient::connect(self.chain.grpc_url.clone())
                    .await
                    .unwrap();

            // TODO: the query should be done for a specific block here, namely the trusted and untrusted commit if the valset is drifting
            tracing::debug!("Query validators...");
            let mut validators = QueryValidatorsResponse::try_from(
                staking_client
                    .validators(staking::v1beta1::QueryValidatorsRequest {
                        status: BondStatus::Bonded.as_str_name().to_string(),
                        pagination: None,
                    })
                    .await
                    .unwrap()
                    .into_inner(),
            )
            .unwrap()
            .validators;

            // Validators must be sorted to match the root, by token then address
            validators.sort_by(|a, b| {
                let a_tokens = a.tokens.parse::<u128>().unwrap();
                let b_tokens = b.tokens.parse::<u128>().unwrap();

                if a_tokens == b_tokens {
                    let a_key = &a
                        .consensus_pubkey
                        .as_bn254()
                        .expect("validator key for cometbls is bn254")
                        .key;

                    let b_key = &b
                        .consensus_pubkey
                        .as_bn254()
                        .expect("validator key for cometbls is bn254")
                        .key;

                    // Tendermint address are sha256(pubkey)[0:20]
                    let a_address = &sha2::Sha256::new().chain_update(a_key).finalize()[..20];
                    let b_address = &sha2::Sha256::new().chain_update(b_key).finalize()[..20];

                    a_address.cmp(b_address)
                } else {
                    a_tokens.cmp(&b_tokens)
                }
            });

            let simple_validators = validators
                .iter()
                .map(|v| {
                    SimpleValidator {
                        pub_key: PublicKey::Bn254(
                            v.consensus_pubkey
                                .as_bn254()
                                .expect("validator key for cometbls is bn254")
                                .key
                                .to_vec(),
                        ),
                        // Equivalent of sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction)
                        voting_power: (v.tokens.parse::<u128>().unwrap() / 1_000_000_u128)
                            .try_into()
                            .unwrap(),
                    }
                })
                .collect::<Vec<_>>();

            let mut bitmap = BigUint::default();
            let mut signatures =
                Vec::<Vec<u8>>::with_capacity(commit.signed_header.commit.signatures.len());
            // NOTE: we assume that the signatures are correctly ordered. i.e. they follow the validator set order as the index is used to aggregate validator pks.
            for (i, sig) in commit.signed_header.commit.signatures.iter().enumerate() {
                match sig {
                    tendermint::block::CommitSig::BlockIdFlagAbsent => {
                        bitmap.set_bit(i as _, false);
                        tracing::debug!("Validator at index {} did not sign", i);
                    }
                    tendermint::block::CommitSig::BlockIdFlagCommit {
                        signature,
                        validator_address,
                        ..
                    } => {
                        bitmap.set_bit(i as _, true);
                        signatures.push(signature.clone().unwrap().into_bytes());
                        tracing::debug!("Validator {:?} signed", validator_address);
                    }
                    // TODO: not sure about this case
                    tendermint::block::CommitSig::BlockIdFlagNil { .. } => {
                        bitmap.set_bit(i as _, false);
                        tracing::warn!(
                            "Validator at index {} has a null flag for the signature commit",
                            i
                        );
                    }
                }
            }

            let validators_trusted_commit =
                Some(protos::union::galois::api::v1::ValidatorSetCommit {
                    validators: simple_validators.into_iter().map(Into::into).collect(),
                    signatures,
                    bitmap: bitmap.to_bytes_be(),
                });

            // The untrusted commit is the same as we don't support validator set drift for now.
            let validators_untrusted_commit = validators_trusted_commit.clone();

            tracing::debug!("Generate ZKP...");

            let mut prover_client = union_prover_api_client::UnionProverApiClient::connect(
                tonic::transport::Endpoint::from_shared(self.chain.prover_endpoint.clone())
                    .unwrap(),
            )
            .await
            .unwrap();

            let prove_res = prover_client
                .prove(ProveRequest {
                    vote: Some(protos::tendermint::types::CanonicalVote {
                        r#type: protos::tendermint::types::SignedMsgType::Precommit.into(),
                        height: commit.signed_header.commit.height.into(),
                        round: i64::from(u32::from(commit.signed_header.commit.round)),
                        block_id: Some(protos::tendermint::types::CanonicalBlockId {
                            hash: commit
                                .signed_header
                                .commit
                                .block_id
                                .hash
                                .as_bytes()
                                .to_vec(),
                            part_set_header: Some(
                                protos::tendermint::types::CanonicalPartSetHeader {
                                    total: commit
                                        .signed_header
                                        .commit
                                        .block_id
                                        .part_set_header
                                        .total,
                                    hash: commit
                                        .signed_header
                                        .commit
                                        .block_id
                                        .part_set_header
                                        .hash
                                        .as_bytes()
                                        .to_vec(),
                                },
                            ),
                        }),
                        chain_id: commit.signed_header.header.chain_id.clone().into(),
                    }),
                    trusted_commit: validators_trusted_commit,
                    untrusted_commit: validators_untrusted_commit,
                })
                .await
                .unwrap()
                .into_inner();

            #[derive(serde::Serialize)]
            struct ProofDump {
                #[serde(with = "hex")]
                untrusted_root: Vec<u8>,
                #[serde(with = "hex")]
                evm_zkp: Vec<u8>,
                #[serde(with = "hex")]
                gnark_zkp: Vec<u8>,
            }

            self.dumper.dump(
                format!(
                    "zk-{:06}-{:06}",
                    trusted_commit.signed_header.header.height.value(),
                    commit.signed_header.header.height.value()
                ),
                &ProofDump {
                    untrusted_root: prove_res.untrusted_validator_set_root.clone(),
                    evm_zkp: prove_res.proof.as_ref().unwrap().evm_proof.clone(),
                    gnark_zkp: prove_res.proof.as_ref().unwrap().compressed_content.clone(),
                },
            );

            let header_timestamp = tendermint_proto::google::protobuf::Timestamp::from(
                commit.signed_header.header.time,
            );

            let signed_header = SignedHeader {
                header: unionlabs::tendermint::types::header::Header {
                    version: unionlabs::tendermint::version::consensus::Consensus {
                        block: commit.signed_header.header.version.block,
                        app: commit.signed_header.header.version.app,
                    },
                    chain_id: commit.signed_header.header.chain_id.into(),
                    height: tendermint_height_to_bounded_i64(commit.signed_header.header.height),
                    time: Timestamp {
                        seconds: header_timestamp.seconds.try_into().unwrap(),
                        nanos: header_timestamp.nanos.try_into().unwrap(),
                    },
                    last_block_id: BlockId {
                        hash: tendermint_hash_to_h256(
                            commit.signed_header.header.last_block_id.unwrap().hash,
                        ),
                        part_set_header: PartSetHeader {
                            total: commit
                                .signed_header
                                .header
                                .last_block_id
                                .unwrap()
                                .part_set_header
                                .total,
                            hash: tendermint_hash_to_h256(
                                commit
                                    .signed_header
                                    .header
                                    .last_block_id
                                    .unwrap()
                                    .part_set_header
                                    .hash,
                            ),
                        },
                    },
                    last_commit_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.last_commit_hash.unwrap(),
                    ),
                    data_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.data_hash.unwrap(),
                    ),
                    validators_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.validators_hash,
                    ),
                    next_validators_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.next_validators_hash,
                    ),
                    consensus_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.consensus_hash,
                    ),
                    app_hash: commit
                        .signed_header
                        .header
                        .app_hash
                        .as_bytes()
                        .try_into()
                        .unwrap(),
                    last_results_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.last_results_hash.unwrap(),
                    ),
                    evidence_hash: tendermint_hash_to_h256(
                        commit.signed_header.header.evidence_hash.unwrap(),
                    ),
                    proposer_address: commit
                        .signed_header
                        .header
                        .proposer_address
                        .as_bytes()
                        .try_into()
                        .expect("value is a [u8; 20] internally, this should not fail; qed;"),
                },
                commit: Commit {
                    height: tendermint_height_to_bounded_i64(commit.signed_header.commit.height),
                    round: i32::from(commit.signed_header.commit.round)
                        .try_into()
                        .unwrap(),
                    block_id: BlockId {
                        hash: tendermint_hash_to_h256(commit.signed_header.commit.block_id.hash),
                        part_set_header: PartSetHeader {
                            total: commit.signed_header.commit.block_id.part_set_header.total,
                            hash: tendermint_hash_to_h256(
                                commit.signed_header.commit.block_id.part_set_header.hash,
                            ),
                        },
                    },
                    // NOTE: We don't need the signatures are they are part of the ZKP
                    signatures: vec![],
                },
            };

            let client_message = cometbls::header::Header {
                signed_header,
                untrusted_validator_set_root: prove_res
                    .untrusted_validator_set_root
                    .try_into()
                    .unwrap(),
                trusted_height: update_from,
                zero_knowledge_proof: prove_res.proof.unwrap().evm_proof,
            };

            tracing::debug!("Client message {:?}", client_message);

            tracing::debug!("Updating client...");

            counterparty
                .update_client(counterparty_client_id.clone(), client_message)
                .await;

            tracing::debug!("Updated client.");

            update_to
        }
    }
}

impl<C: ChainSpec> Ethereum<C> {
    async fn send_msg_and_read_event<
        TMsg: MsgIntoProto,
        TEvent: TryFrom<Event, Error = TryFromTendermintEventError>,
    >(
        &self,
        msg: TMsg,
    ) -> (Height, TEvent) {
        self.chain
            .broadcast_tx_commit([Any(msg).into_proto_with_signer(&self.chain.signer)])
            .map(|response| {
                (
                    self.chain.make_height(response.height.value()),
                    get_event_from_tx_response(response.deliver_tx.events).unwrap(),
                )
            })
            .await
    }
}

fn get_event_from_tx_response<T: TryFrom<Event, Error = TryFromTendermintEventError>>(
    events: Vec<tendermint::abci::Event>,
) -> Option<T> {
    events
        .into_iter()
        .find_map(|event| {
            let conversion_result = T::try_from(Event {
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
            });

            match conversion_result {
                Ok(ok) => Some(Ok(ok)),
                // this isn't fatal in this context
                Err(TryFromTendermintEventError::IncorrectType {
                    expected: _,
                    found: _,
                }) => None,
                Err(err) => Some(Err(err)),
            }
        })
        .transpose()
        // there were no errors parsing event if it was found
        .unwrap()
}

trait AbciStateRead<Counterparty>: IbcPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty>;
}

impl<Counterparty> AbciStateRead<Counterparty> for ClientStatePath
where
    Counterparty: Chain,
    ClientStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ClientStateOf<Counterparty>>: Debug,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty> {
        Self::Output::<Counterparty>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for ClientConsensusStatePath
where
    Counterparty: Chain,
    ConsensusStateOf<Counterparty>: TryFromProto,
    TryFromProtoErrorOf<ConsensusStateOf<Counterparty>>: Debug,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty> {
        Self::Output::<Counterparty>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for ConnectionPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty> {
        Self::Output::<Counterparty>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for ChannelEndPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty> {
        Self::Output::<Counterparty>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl<Counterparty> AbciStateRead<Counterparty> for CommitmentPath
where
    Counterparty: Chain,
{
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Counterparty> {
        bytes.try_into().unwrap()
    }
}

impl<Counterparty, P> IbcStateRead<Counterparty, P> for Union
where
    Counterparty: Chain,
    ClientStateOf<Counterparty>: TryFromProto,
    ConsensusStateOf<Counterparty>: TryFromProto,
    P: IbcPath + AbciStateRead<Counterparty> + 'static,
{
    fn state_proof(
        &self,
        path: P,
        at: Height,
    ) -> impl Future<Output = StateProof<P::Output<Counterparty>>> + '_ {
        async move {
            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    self.grpc_url.clone(),
                )
                .await
                .unwrap();

            let query_result = client
                .abci_query(AbciQueryRequest {
                    data: path.to_string().into_bytes(),
                    path: "store/ibc/key".to_string(),
                    height: at.revision_height.try_into().unwrap(),
                    prove: true,
                })
                .await
                .unwrap()
                .into_inner();

            StateProof {
                state: P::from_abci_bytes(query_result.value),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof_ops
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: self.make_height(query_result.height.try_into().unwrap()),
            }
        }
    }
}

fn tendermint_hash_to_h256(hash: tendermint::Hash) -> H256 {
    match hash {
        tendermint::Hash::Sha256(hash) => hash.into(),
        tendermint::Hash::None => panic!("empty hash???"),
    }
}

fn tendermint_height_to_bounded_i64(
    height: tendermint::block::Height,
) -> BoundedI64<0, { i64::MAX }> {
    i64::from(height).try_into().unwrap()
}
