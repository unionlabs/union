use std::ops::{Div, Rem};
use std::{
    fmt::Debug,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use contracts::{
    glue::{
        GoogleProtobufDurationData, GoogleProtobufTimestampData, IbcCoreCommitmentV1MerkleRootData,
        UnionIbcLightclientsCometblsV1ClientStateData,
        UnionIbcLightclientsCometblsV1ConsensusStateData,
        UnionIbcLightclientsCometblsV1FractionData, UnionIbcLightclientsCometblsV1HeaderData,
    },
    ibc_handler::{
        self, GeneratedConnectionIdentifierFilter, IBCHandler, IBCHandlerEvents,
        IbcCoreChannelV1ChannelData, IbcCoreChannelV1CounterpartyData, IbcCoreChannelV1PacketData,
        IbcCoreCommitmentV1MerklePrefixData, IbcCoreConnectionV1ConnectionEndData,
        IbcCoreConnectionV1CounterpartyData, IbcCoreConnectionV1VersionData,
    },
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::{
    abi::{AbiDecode, AbiEncode},
    prelude::{decode_logs, k256::ecdsa, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{H160, H256, U256},
    utils::keccak256,
};
use futures::Future;
use lodestar_rpc::types::{
    BeaconHeaderResponse, LightClientBootstrapResponse, LightClientUpdateData,
};
use prost::Message;
use protos::{google, union::ibc::lightclients::ethereum::v1 as ethereum_v1};
use strum::ParseError;

use crate::chain::{evm::ethereum::SyncAggregate, msgs::ethereum::Proof};
use crate::{
    chain::{
        cosmos::IntoProto,
        msgs::ethereum::{AccountUpdate, LightClientUpdate, SyncCommittee, TrustedSyncCommittee},
    },
    cosmos_to_eth::COMETBLS_CLIENT_TYPE,
    eth_to_cosmos::LCFUR,
    ETH_BEACON_RPC_API, ETH_RPC_API,
};

use super::{
    cosmos::{Any, Ethereum},
    msgs::{
        channel::{
            self, Channel, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit,
            MsgChannelOpenTry, MsgRecvPacket, Packet,
        },
        cometbls,
        connection::{
            self, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
            MsgConnectionOpenTry,
        },
        ethereum::{self, BeaconBlockHeader, ExecutionPayloadHeader},
        wasm, ConnectionEnd, Duration, Fraction, Height, MerklePrefix, MerkleRoot, StateProof,
        Timestamp, UnknownEnumVariant,
    },
    Connect, LightClient,
};

/// The solidity light client, tracking the state of the 08-wasm light client on union.
// TODO(benluelo): Generic over middleware?
pub struct Cometbls {
    ibc_handler: IBCHandler<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>>,
    pub provider: Provider<Http>,
    cometbls_client_address: H160,
    wasm_code_id: H256,
}

fn encode_dynamic_singleton_tuple(t: impl AbiEncode) -> Vec<u8> {
    U256::from(32)
        .encode()
        .into_iter()
        .chain(t.encode().into_iter())
        .collect::<Vec<_>>()
}

// TODO(benluelo): Return result instead of unwrap
// REVIEW(benluelo): The contract returns an Any, perhaps we need to decode to that type? It will
// need to be exposed through the glue contract (EDIT: The mock contract returns Any, but the
// actual cometbls client returns the bytes it recieves on creation, which is the dynamic singleton
// tuple encoded struct)
fn decode_dynamic_singleton_tuple<T: AbiDecode>(bs: &[u8]) -> T {
    let tuple_idx_bytes = U256::from(32).encode().len();

    T::decode(&bs[tuple_idx_bytes..]).unwrap()
}

impl LightClient for Cometbls {
    type ClientState = Any<wasm::ClientState<cometbls::ClientState>>;
    type ConsensusState = Any<wasm::ConsensusState<cometbls::ConsensusState>>;
    // TODO(benluelo): Wrap this in wasm?
    type UpdateClientMessage = UnionIbcLightclientsCometblsV1HeaderData;

    fn chain_id(&self) -> impl Future<Output = String> + '_ {
        async move { self.provider.get_chainid().await.unwrap().to_string() }
    }

    fn create_client(
        &self,
        client_state: Self::ClientState,
        consensus_state: Self::ConsensusState,
    ) -> impl Future<Output = String> + '_ {
        async {
            let register_client_result = self
                .ibc_handler
                .register_client(COMETBLS_CLIENT_TYPE.into(), self.cometbls_client_address);

            // TODO(benluelo): Better way to check if client type has already been registered?
            match register_client_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => eprintln!("{}", why.decode_revert::<String>().unwrap()),
            }

            let tx_rcp = self
                .ibc_handler
                .create_client(ibc_handler::MsgCreateClient {
                    client_type: COMETBLS_CLIENT_TYPE.to_string(),
                    client_state_bytes: client_state.into_proto().encode_to_vec().into(),
                    consensus_state_bytes: consensus_state.into_proto().encode_to_vec().into(),
                })
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let client_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(|l| l.into())
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedClientIdentifierFilter(client_id) => Some(client_id.0),
                _ => None,
            })
            .unwrap();

            let (consensus_state, is_found) = self
                .ibc_handler
                .get_consensus_state(
                    client_id.clone(),
                    IbcCoreClientV1HeightData {
                        revision_number: 0,
                        revision_height: tx_rcp.block_number.unwrap().as_u64(),
                    },
                )
                .call()
                .await
                .unwrap();

            assert!(is_found);

            // dbg!(consensus_state.to_string());

            client_id
        }
    }

    fn update_client(
        &self,
        client_id: String,
        msg: Self::UpdateClientMessage,
    ) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .update_client(ibc_handler::MsgUpdateClient {
                    client_id,
                    client_message: encode_dynamic_singleton_tuple(msg).into(),
                })
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn consensus_state_proof(
        &self,
        client_id: String,
        counterparty_height: Height,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ConsensusState>> + '_ {
        async move {
            tracing::info!(?self_height);
            self.wait_for_block(self_height).await;

            let (consensus_state_bytes, is_found) = self
                .ibc_handler
                .get_consensus_state(client_id.clone(), counterparty_height.into())
                .block(self_height.revision_height)
                .await
                .unwrap();

            assert!(is_found);

            dbg!(&consensus_state_bytes.to_string());

            // let optimized_consensus_state =
            //     OptimizedConsensusState::decode(&consensus_state_bytes).unwrap();

            let cometbls_consensus_state: Self::ConsensusState =
                google::protobuf::Any::decode(&*consensus_state_bytes)
                    .unwrap()
                    .try_into()
                    .unwrap();

            tracing::info!(?cometbls_consensus_state);

            self.get_proof(
                format!(
                    "clients/{client_id}/consensusStates/{}-{}",
                    counterparty_height.revision_number, counterparty_height.revision_height
                ),
                self_height,
                cometbls_consensus_state.clone(),
                // AbiEncode::encode,
                |x| x.into_proto().encode_to_vec(),
            )
            .await
        }
    }

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ClientState>> + '_ {
        async move {
            tracing::info!(?self_height);
            self.wait_for_block(self_height).await;

            let block_number = self.provider.get_block_number().await.unwrap();
            tracing::info!(?block_number);

            let (client_state_bytes, is_found) = self
                .ibc_handler
                .get_client_state(client_id.clone())
                .block(self_height.revision_height)
                .await
                .unwrap();

            assert!(is_found);

            dbg!(client_state_bytes.to_string());

            // let cometbls_client_state = decode_dynamic_singleton_tuple::<
            //     UnionIbcLightclientsCometblsV1ClientStateData,
            // >(&client_state_bytes);

            let cometbls_client_state: Self::ClientState =
                google::protobuf::Any::decode(&*client_state_bytes)
                    .unwrap()
                    .try_into()
                    .unwrap();

            // tracing::info!(?cometbls_client_state);

            // tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            let block_number = self.provider.get_block_number().await.unwrap();
            tracing::info!(?block_number);

            self.get_proof(
                format!("clients/{client_id}/clientState"),
                self_height,
                cometbls_client_state,
                |x| x.into_proto().encode_to_vec(),
            )
            .await
        }
    }

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<ConnectionEnd>> + '_ {
        async move {
            tracing::info!(?self_height);
            self.wait_for_block(self_height).await;

            let (connection_end, is_found): (IbcCoreConnectionV1ConnectionEndData, bool) = self
                .ibc_handler
                .get_connection(connection_id.clone())
                .block(self_height.revision_height)
                .await
                .unwrap();

            // let (connection_end_bytes, _is_found) = self
            //     .ibc_handler
            //     .get_connection_serialize(connection_id.clone())
            //     .block(self_height.revision_height)
            //     .await
            //     .unwrap();

            tracing::info!(?connection_end);

            assert!(is_found);

            let canonical_connection_end: ConnectionEnd = connection_end.try_into().unwrap();

            // assert_eq!(
            //     canonical_connection_end
            //         .clone()
            //         .into_proto()
            //         .encode_to_vec(),
            //     connection_end_bytes.to_vec()
            // );

            // let connection_end_from_contract =
            //     connection_v1::ConnectionEnd::decode(&*connection_end_bytes.to_vec()).unwrap();

            // dbg!(connection_end_from_contract);

            // super::msgs::StateProof {
            //     state: canonical_connection_end,
            //     proof: state_proof.proof,
            //     proof_height: state_proof.proof_height,
            // }

            self.get_proof(
                format!("connections/{connection_id}"),
                self_height,
                canonical_connection_end,
                |x| x.into_proto().encode_to_vec(),
            )
            .await
        }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            const API: &str = "eth/v2/debug/beacon/states";
            let height = reqwest::Client::new()
                .get(format!("{ETH_BEACON_RPC_API}/{API}/finalized"))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()
                .get("data")
                .unwrap()
                .get("slot")
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap();
            Height {
                revision_number: 0,
                revision_height: height,
            }

            // self.provider
            //     .get_block_number()
            //     .await
            //     .map(|height| Height {
            //         revision_number: 0,
            //         revision_height: height.as_u64(),
            //     })
            //     .unwrap()
        }
    }
}

impl Connect<Ethereum> for Cometbls {
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     counterparty_state: <Ethereum as LightClient>::ClientState,
    // ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
    //     async move { todo!() }
    // }

    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl Future<Output = String> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .connection_open_init(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let connection_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(|l| l.into())
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(
                    GeneratedConnectionIdentifierFilter(connection_id),
                ) => {
                    tracing::info!(connection_id, "created connection");

                    Some(connection_id)
                }
                _ => None,
            })
            .unwrap();

            self.wait_for_block(Height {
                revision_number: 0,
                revision_height: tx_rcp.block_number.unwrap().0[0],
            })
            .await;

            connection_id
        }
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<<Ethereum as LightClient>::ClientState>,
    ) -> impl Future<Output = String> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .connection_open_try(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.wait_for_block(Height {
                revision_number: 0,
                revision_height: tx_rcp.block_number.unwrap().0[0],
            })
            .await;

            decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(|l| l.into())
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(connection_id) => {
                    Some(connection_id.0)
                }
                _ => None,
            })
            .unwrap()
        }
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<<Ethereum as LightClient>::ClientState>,
    ) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .connection_open_ack(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .connection_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn channel_open_init(&self, msg: MsgChannelOpenInit) -> impl Future<Output = String> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .channel_open_init(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(|l| l.into())
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap()
            .into_iter()
            .find_map(|l| match l {
                IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => {
                    Some(channel_id.0)
                }
                _ => None,
            })
            .unwrap()
        }
    }

    fn channel_open_try(&self, msg: MsgChannelOpenTry) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .channel_open_try(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn channel_open_ack(&self, msg: MsgChannelOpenAck) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .channel_open_ack(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn channel_open_confirm(&self, msg: MsgChannelOpenConfirm) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .channel_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn recv_packet(&self, packet: MsgRecvPacket) -> impl Future<Output = ()> + '_ {
        async move {
            self.ibc_handler
                .recv_packet(packet.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();
        }
    }

    fn generate_counterparty_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ClientState> + '_ {
        async move {
            let genesis = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
                .get_genesis()
                .await
                .unwrap()
                .data;

            Any(wasm::ClientState {
                data: ethereum::ClientState {
                    genesis_validators_root: genesis.genesis_validators_root.as_bytes().to_vec(),
                    genesis_time: genesis.genesis_time.0,
                    fork_parameters: ethereum::ForkParameters {
                        genesis_fork_version: vec![0, 0, 0, 1],
                        genesis_slot: 0,
                        altair: ethereum::Fork {
                            version: vec![1, 0, 0, 1],
                            epoch: 0,
                        },
                        bellatrix: ethereum::Fork {
                            version: vec![2, 0, 0, 1],
                            epoch: 0,
                        },
                        capella: ethereum::Fork {
                            version: vec![3, 0, 0, 1],
                            epoch: 0,
                        },
                        eip4844: ethereum::Fork {
                            version: vec![4, 0, 0, 0],
                            epoch: u64::MAX,
                        },
                    },
                    seconds_per_slot: 6,
                    slots_per_epoch: 8,
                    epochs_per_sync_committee_period: 8,
                    trusting_period: 100000000,
                    latest_slot: height.revision_height,
                    min_sync_committee_participants: 0,
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    frozen_height: None,
                    counterparty_commitment_slot: 0,
                },
                code_id: self.wasm_code_id.to_fixed_bytes().to_vec(),
                latest_height: height,
            })
        }
    }

    fn generate_counterparty_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ConsensusState> + '_ {
        async move {
            let trusted_header = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
                .get_beacon_header_by_slot(ethereum_consensus::types::U64(height.revision_height))
                .await
                .unwrap()
                .data;

            let bootstrap = reqwest::get(dbg!(format!(
                "http://0.0.0.0:9596/eth/v1/beacon/light_client/bootstrap/0x{}",
                trusted_header.root
            )))
            .await
            .unwrap()
            .json::<LightClientBootstrapResponse<32, 256, 32>>()
            .await
            .unwrap()
            .data;

            let light_client_update = {
                let current_period = height.revision_height.div(64);

                let light_client_updates: lodestar_rpc::types::LightClientUpdatesResponse<32, 256, 32> =
                serde_json::from_value(
                    reqwest::get(format!(
                        "http://0.0.0.0:9596/eth/v1/beacon/light_client/updates?start_period={current_period}&count=1",
                    ))
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap(),
                )
                .unwrap();

                let [light_client_update] = &*light_client_updates.0 else { panic!() };

                light_client_update.data.clone()
            };

            Any(wasm::ConsensusState {
                data: ethereum::ConsensusState {
                    slot: bootstrap.header.beacon.slot.0,
                    storage_root: vec![1, 2, 3],
                    timestamp: bootstrap.header.execution.timestamp.0,
                    current_sync_committee: bootstrap
                        .current_sync_committee
                        .aggregate_pubkey
                        .to_vec(),
                    next_sync_committee: light_client_update
                        .next_sync_committee
                        .aggregate_pubkey
                        .iter()
                        .copied()
                        .collect(),
                },
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        }
    }

    /// Returns the actual height updated to (>= update_to)
    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Ethereum,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + 'a {
        async move {
            self.wait_for_block(update_to).await;

            const PERIOD: u64 = 64;

            // FIXME: what if update_to and update_from are in different sync_committee periods?

            let update_to_period = update_to.revision_height.div(PERIOD);

            let slot_in_period = update_from.revision_height.rem(PERIOD);

            let current_period = update_from.revision_height.div(PERIOD);

            let trusted_block = reqwest::get(format!(
                "http://0.0.0.0:9596/eth/v1/beacon/headers/{}",
                update_from.revision_height
            ))
            .await
            .unwrap()
            .json::<BeaconHeaderResponse>()
            .await
            .unwrap();

            // bootstrap contains the current sync committee for the given height
            let bootstrap: LightClientBootstrapResponse<32, 256, 32> = serde_json::from_value(
                reqwest::get(format!(
                    "http://0.0.0.0:9596/eth/v1/beacon/light_client/bootstrap/0x{}",
                    trusted_block.data.root
                ))
                .await
                .unwrap()
                .json::<serde_json::Value>()
                // .json::<LightClientBootstrapResponse<32, 256, 32>>()
                .await
                .unwrap(),
            )
            .unwrap();

            // contains the sync committee update for the current period
            let light_client_update = {
                let light_client_updates: lodestar_rpc::types::LightClientUpdatesResponse<32, 256, 32> =
                serde_json::from_value(
                    reqwest::get(format!(
                        "http://0.0.0.0:9596/eth/v1/beacon/light_client/updates?start_period={current_period}&count=1",
                    ))
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap(),
                )
                .unwrap();

                let [light_client_update] = &*light_client_updates.0 else { panic!() };

                light_client_update.clone()
            };

            // send the sync committee update if we are at the beginning of the period
            if slot_in_period == 0 || update_to_period > current_period {
                tracing::info!("at the beginning of the period, updating sync committee");

                let account_update = self
                    .provider
                    .get_proof(
                        self.ibc_handler.address(),
                        vec![],
                        Some(
                            light_client_update
                                .data
                                .finalized_header
                                .beacon
                                .slot
                                .0
                                .into(),
                        ),
                    )
                    .await
                    .unwrap();

                let header = wasm::Header {
                    data: ethereum::Header {
                        trusted_sync_committee: TrustedSyncCommittee {
                            trusted_height: Height {
                                revision_number: 0,
                                // NOTE: should be the same as trusted height passed in to this function
                                revision_height: bootstrap.data.header.beacon.slot.0,
                            },
                            sync_committee: SyncCommittee {
                                pubkeys: bootstrap
                                    .data
                                    .current_sync_committee
                                    .pubkeys
                                    .iter()
                                    .map(|x| x.0.iter().copied().collect())
                                    .collect(),
                                aggregate_pubkey: bootstrap
                                    .data
                                    .current_sync_committee
                                    .aggregate_pubkey
                                    .iter()
                                    .copied()
                                    .collect(),
                            },
                            is_next: false,
                        },
                        consensus_update: {
                            let LightClientUpdateData {
                                attested_header,
                                finalized_header,
                                finality_branch,
                                sync_aggregate,
                                signature_slot,
                                next_sync_committee,
                                next_sync_committee_branch,
                            } = light_client_update.clone().data;

                            LightClientUpdate {
                                attested_header: translate_header(attested_header),
                                next_sync_committee: SyncCommittee {
                                    pubkeys: next_sync_committee
                                        .pubkeys
                                        .iter()
                                        .map(|x| x.0.to_vec())
                                        .collect(),
                                    aggregate_pubkey: next_sync_committee.aggregate_pubkey.to_vec(),
                                },
                                next_sync_committee_branch: next_sync_committee_branch
                                    .to_vec()
                                    .iter()
                                    .map(|x| x.as_bytes().to_vec())
                                    .collect(),
                                finalized_header: translate_header(finalized_header),
                                finality_branch: finality_branch
                                    .iter()
                                    .map(|x| x.as_bytes().to_vec())
                                    .collect(),
                                sync_aggregate: SyncAggregate {
                                    sync_committee_bits: sync_aggregate
                                        .sync_committee_bits
                                        .as_bitslice()
                                        .to_bitvec()
                                        .into_vec(),
                                    sync_committee_signature: sync_aggregate
                                        .sync_committee_signature
                                        .iter()
                                        .copied()
                                        .collect(),
                                },
                                signature_slot: signature_slot.0,
                            }
                        },
                        account_update: AccountUpdate {
                            proofs: [Proof {
                                key: self.ibc_handler.address().as_bytes().to_vec(),
                                value: account_update.storage_hash.as_bytes().to_vec(),
                                proof: account_update
                                    .account_proof
                                    .into_iter()
                                    .map(|x| x.to_vec())
                                    .collect(),
                            }]
                            .to_vec(),
                        },
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                    height: update_to,
                };

                counterparty
                    .update_client(counterparty_client_id.clone(), header)
                    .await;
            }

            // wait until the execution height is >= the latest trusted height
            let finality_update = loop {
                let finality_update =
                    reqwest::get("http://0.0.0.0:9596/eth/v1/beacon/light_client/finality_update")
                        .await
                        .unwrap()
                        .json::<LCFUR>()
                        .await
                        .unwrap();

                let block_number = finality_update.data.finalized_header.execution.block_number;

                tracing::info!(
                    update_from = ?update_from,
                    update_to = ?update_to,
                    current = ?block_number
                );

                if block_number.0 >= update_to.revision_height {
                    break finality_update.data;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            };

            let actual_udpated_height = Height {
                revision_number: 0,
                revision_height: finality_update.finalized_header.execution.block_number.0,
            };

            // send the finality update

            // whether the sync committee signature is to be checked against the current or next sync committee
            let is_next = finality_update.finalized_header.beacon.slot.0 % PERIOD == 0;

            let trusted_sync_committee = if is_next {
                TrustedSyncCommittee {
                    trusted_height: update_from,
                    sync_committee: SyncCommittee {
                        pubkeys: light_client_update
                            .data
                            .next_sync_committee
                            .pubkeys
                            .iter()
                            .map(|x| x.0.iter().copied().collect())
                            .collect(),
                        aggregate_pubkey: light_client_update
                            .data
                            .next_sync_committee
                            .aggregate_pubkey
                            .iter()
                            .copied()
                            .collect(),
                    },
                    is_next,
                }
            } else {
                TrustedSyncCommittee {
                    trusted_height: update_from,
                    sync_committee: SyncCommittee {
                        pubkeys: bootstrap
                            .data
                            .current_sync_committee
                            .pubkeys
                            .iter()
                            .map(|x| x.0.iter().copied().collect())
                            .collect(),
                        aggregate_pubkey: bootstrap
                            .data
                            .current_sync_committee
                            .aggregate_pubkey
                            .iter()
                            .copied()
                            .collect(),
                    },
                    is_next,
                }
            };

            let account_update = self
                .provider
                .get_proof(
                    self.ibc_handler.address(),
                    vec![],
                    Some(finality_update.finalized_header.beacon.slot.0.into()),
                )
                .await
                .unwrap();

            let header = wasm::Header {
                height: actual_udpated_height,
                data: ethereum::Header {
                    trusted_sync_committee,
                    consensus_update: LightClientUpdate {
                        attested_header: translate_header(finality_update.attested_header),
                        // TODO(benluelo): make into Option
                        next_sync_committee: SyncCommittee::default(),
                        next_sync_committee_branch: Default::default(),
                        finalized_header: translate_header(finality_update.finalized_header),
                        finality_branch: finality_update
                            .finality_branch
                            .iter()
                            .map(|x| x.as_bytes().to_vec())
                            .collect(),
                        sync_aggregate: SyncAggregate {
                            sync_committee_bits: finality_update
                                .sync_aggregate
                                .sync_committee_bits
                                .as_bitslice()
                                .to_bitvec()
                                .into_vec(),
                            sync_committee_signature: finality_update
                                .sync_aggregate
                                .sync_committee_signature
                                .iter()
                                .copied()
                                .collect(),
                        },
                        signature_slot: finality_update.signature_slot.0,
                    },
                    account_update: AccountUpdate {
                        proofs: [Proof {
                            key: self.ibc_handler.address().as_bytes().to_vec(),
                            value: account_update.storage_hash.as_bytes().to_vec(),
                            proof: account_update
                                .account_proof
                                .into_iter()
                                .map(|x| x.to_vec())
                                .collect(),
                        }]
                        .to_vec(),
                    },
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            };

            tracing::info!("submitting finality update");
            counterparty
                .update_client(counterparty_client_id, header)
                .await;

            actual_udpated_height
        }
    }
}

impl Cometbls {
    pub async fn new(
        cometbls_client_address: H160,
        ibc_handler_address: H160,
        wasm_code_id: H256,
    ) -> Self {
        let provider = Provider::<Http>::try_from(ETH_RPC_API).unwrap();
        let chain_id = provider.get_chainid().await.unwrap();
        // TODO(benluelo): Pass this in as a parameter
        let wallet = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id.as_u64());

        let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        let ibc_handler = ibc_handler::IBCHandler::new(ibc_handler_address, signer_middleware);

        Self {
            ibc_handler,
            provider,
            cometbls_client_address,
            wasm_code_id,
        }
    }

    async fn get_proof<S: Clone + Debug>(
        &self,
        path: String,
        height: Height,
        state: S,
        encode: impl FnOnce(S) -> Vec<u8>,
    ) -> StateProof<S> {
        tracing::info!(path, ?height);

        let u256 = U256::from(0).encode();

        assert_eq!(u256.len(), 32);

        let path_keccak256 = keccak256(path.as_bytes());
        // tracing::info!(path_keccak256 = ?ethers::types::Bytes(path_keccak256.to_vec().into()));

        let path_keccak256_concat_slot = path_keccak256.into_iter().chain(u256).collect::<Vec<_>>();
        // tracing::info!(path_keccak256_concat_slot = ?ethers::types::Bytes(path_keccak256_concat_slot.to_vec().into()));

        let location = keccak256(path_keccak256_concat_slot);
        // tracing::info!(location = ?ethers::types::Bytes(location.to_vec().into()));

        let storage = self
            .provider
            .get_storage_at(
                self.ibc_handler.address(),
                location.into(),
                Some(height.revision_height.into()),
            )
            .await
            .unwrap();

        tracing::info!(?storage);

        let mut proof = self
            .provider
            // eth_getProof
            .get_proof(
                self.ibc_handler.address(),
                vec![location.into()],
                Some(height.revision_height.into()),
            )
            .await
            .unwrap();

        tracing::info!(?proof);

        assert_eq!(proof.storage_proof.len(), 1);

        let proof = proof.storage_proof.pop().unwrap();

        let found_value = U256::from(keccak256(encode(state.clone())));

        assert_eq!(found_value, proof.value);

        StateProof {
            state,
            proof: ethereum_v1::StorageProof {
                proofs: [ethereum_v1::Proof {
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
            .encode_to_vec(),
            proof_height: height,
        }
    }

    async fn wait_for_block(&self, requested_height: Height) {
        loop {
            // let current_block = self.provider.get_block_number().await.unwrap();

            let current_block = self.query_latest_height().await;

            tracing::debug!(?current_block, waiting_for = ?requested_height, "waiting for block");

            if current_block.revision_height >= requested_height.revision_height {
                break;
            } else {
                tracing::debug!("requested height not yet reached");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }
}

pub trait IntoEthAbi: Into<Self::EthAbi> {
    type EthAbi;

    fn into_eth_abi(self) -> Self::EthAbi {
        self.into()
    }
}

macro_rules! into_eth_abi {
    ($($from:ty => $to:ty),+) => {
        $(
            impl IntoEthAbi for $from {
                type EthAbi = $to;
            }
        )+
    };
}

into_eth_abi! {
    super::msgs::cometbls::ClientState => UnionIbcLightclientsCometblsV1ClientStateData,
    super::msgs::cometbls::ConsensusState => UnionIbcLightclientsCometblsV1ConsensusStateData
}

impl From<MsgConnectionOpenInit> for ibc_handler::MsgConnectionOpenInit {
    fn from(msg: MsgConnectionOpenInit) -> ibc_handler::MsgConnectionOpenInit {
        ibc_handler::MsgConnectionOpenInit {
            client_id: msg.client_id,
            counterparty: IbcCoreConnectionV1CounterpartyData {
                client_id: msg.counterparty.client_id,
                connection_id: msg.counterparty.connection_id,
                prefix: IbcCoreCommitmentV1MerklePrefixData {
                    key_prefix: msg.counterparty.prefix.key_prefix.into(),
                },
            },
            delay_period: msg.delay_period,
        }
    }
}

impl<ClientState> From<MsgConnectionOpenTry<ClientState>> for ibc_handler::MsgConnectionOpenTry {
    fn from(msg: MsgConnectionOpenTry<ClientState>) -> ibc_handler::MsgConnectionOpenTry {
        ibc_handler::MsgConnectionOpenTry {
            counterparty: msg.counterparty.into(),
            delay_period: msg.delay_period,
            client_id: msg.client_id,
            // client_state_bytes: msg.client_state.value.into(),
            // TODO(benluelo): Figure out what this is expected to be (i.e. eth abi or proto)
            client_state_bytes: Default::default(),
            counterparty_versions: msg
                .counterparty_versions
                .into_iter()
                .map(Into::into)
                .collect(),
            proof_init: msg.proof_init.into(),
            proof_client: msg.proof_client.into(),
            proof_consensus: msg.proof_consensus.into(),
            proof_height: msg.proof_height.into(),
            consensus_height: msg.consensus_height.into(),
        }
    }
}

impl<ClientState> From<MsgConnectionOpenAck<ClientState>> for ibc_handler::MsgConnectionOpenAck {
    fn from(msg: MsgConnectionOpenAck<ClientState>) -> ibc_handler::MsgConnectionOpenAck {
        ibc_handler::MsgConnectionOpenAck {
            connection_id: msg.connection_id,
            counterparty_connection_id: msg.counterparty_connection_id,
            version: msg.version.into(),
            // client_state_bytes: msg.client_state.value.into(),
            // TODO(benluelo): Figure out what this is expected to be (i.e. eth abi or proto)
            client_state_bytes: Default::default(),
            proof_height: msg.proof_height.into(),
            proof_try: msg.proof_try.into(),
            proof_client: msg.proof_client.into(),
            proof_consensus: msg.proof_consensus.into(),
            consensus_height: msg.consensus_height.into(),
        }
    }
}

impl From<MsgConnectionOpenConfirm> for ibc_handler::MsgConnectionOpenConfirm {
    fn from(msg: MsgConnectionOpenConfirm) -> ibc_handler::MsgConnectionOpenConfirm {
        ibc_handler::MsgConnectionOpenConfirm {
            connection_id: msg.connection_id,
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<MsgChannelOpenInit> for ibc_handler::MsgChannelOpenInit {
    fn from(msg: MsgChannelOpenInit) -> ibc_handler::MsgChannelOpenInit {
        ibc_handler::MsgChannelOpenInit {
            port_id: msg.port_id,
            channel: msg.channel.into(),
        }
    }
}

impl From<MsgChannelOpenTry> for ibc_handler::MsgChannelOpenTry {
    fn from(msg: MsgChannelOpenTry) -> ibc_handler::MsgChannelOpenTry {
        ibc_handler::MsgChannelOpenTry {
            port_id: msg.port_id,
            channel: msg.channel.into(),
            counterparty_version: msg.counterparty_version,
            proof_init: msg.proof_init.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<MsgChannelOpenAck> for ibc_handler::MsgChannelOpenAck {
    fn from(msg: MsgChannelOpenAck) -> ibc_handler::MsgChannelOpenAck {
        ibc_handler::MsgChannelOpenAck {
            port_id: msg.port_id,
            channel_id: msg.channel_id,
            counterparty_version: msg.counterparty_version,
            counterparty_channel_id: msg.counterparty_channel_id,
            proof_try: msg.proof_try.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<MsgChannelOpenConfirm> for ibc_handler::MsgChannelOpenConfirm {
    fn from(msg: MsgChannelOpenConfirm) -> ibc_handler::MsgChannelOpenConfirm {
        ibc_handler::MsgChannelOpenConfirm {
            port_id: msg.port_id,
            channel_id: msg.channel_id,
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<Height> for IbcCoreClientV1HeightData {
    fn from(value: Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<IbcCoreClientV1HeightData> for Height {
    fn from(value: IbcCoreClientV1HeightData) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<connection::Counterparty> for IbcCoreConnectionV1CounterpartyData {
    fn from(value: connection::Counterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}

impl From<IbcCoreConnectionV1CounterpartyData> for connection::Counterparty {
    fn from(value: IbcCoreConnectionV1CounterpartyData) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}

impl From<connection::Version> for IbcCoreConnectionV1VersionData {
    fn from(value: connection::Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|order| <&'static str>::from(order).to_string())
                .collect(),
        }
    }
}

impl TryFrom<IbcCoreConnectionV1VersionData> for connection::Version {
    type Error = ParseError;

    fn try_from(value: IbcCoreConnectionV1VersionData) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|order| order.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<MerklePrefix> for IbcCoreCommitmentV1MerklePrefixData {
    fn from(value: MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix.into(),
        }
    }
}

impl From<IbcCoreCommitmentV1MerklePrefixData> for MerklePrefix {
    fn from(value: IbcCoreCommitmentV1MerklePrefixData) -> Self {
        Self {
            key_prefix: value.key_prefix.to_vec(),
        }
    }
}

impl From<MsgRecvPacket> for ibc_handler::MsgPacketRecv {
    fn from(value: MsgRecvPacket) -> Self {
        Self {
            packet: value.packet.into(),
            proof: value.proof_commitment.into(),
            proof_height: value.proof_height.into(),
        }
    }
}

impl From<Packet> for IbcCoreChannelV1PacketData {
    fn from(value: Packet) -> Self {
        Self {
            sequence: value.sequence,
            source_port: value.source_port,
            source_channel: value.source_channel,
            destination_port: value.destination_port,
            destination_channel: value.destination_channel,
            data: value.data.into(),
            timeout_height: value.timeout_height.into(),
            timeout_timestamp: value.timeout_timestamp,
        }
    }
}

impl From<Channel> for IbcCoreChannelV1ChannelData {
    fn from(value: Channel) -> Self {
        Self {
            state: value.state as u8,
            ordering: value.ordering as u8,
            counterparty: value.counterparty.into(),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

impl From<channel::Counterparty> for IbcCoreChannelV1CounterpartyData {
    fn from(value: channel::Counterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

#[derive(Debug)]
pub enum TryFromConnnectionEndError {
    ParseError(ParseError),
    UnknownEnumVariant(UnknownEnumVariant<u8>),
}

impl TryFrom<IbcCoreConnectionV1ConnectionEndData> for ConnectionEnd {
    type Error = TryFromConnnectionEndError;

    fn try_from(val: IbcCoreConnectionV1ConnectionEndData) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: val.client_id,
            versions: val
                .versions
                .into_iter()
                .map(|x| x.try_into().map_err(TryFromConnnectionEndError::ParseError))
                .collect::<Result<_, _>>()?,
            state: val
                .state
                .try_into()
                .map_err(TryFromConnnectionEndError::UnknownEnumVariant)?,
            counterparty: val.counterparty.into(),
            delay_period: val.delay_period,
        })
    }
}

impl From<cometbls::ClientState> for UnionIbcLightclientsCometblsV1ClientStateData {
    fn from(value: cometbls::ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value.trusting_period.into(),
            unbonding_period: value.unbonding_period.into(),
            max_clock_drift: value.max_clock_drift.into(),
            frozen_height: value.frozen_height.into(),
        }
    }
}

impl From<UnionIbcLightclientsCometblsV1ClientStateData> for cometbls::ClientState {
    fn from(value: UnionIbcLightclientsCometblsV1ClientStateData) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value.trusting_period.into(),
            unbonding_period: value.unbonding_period.into(),
            max_clock_drift: value.max_clock_drift.into(),
            frozen_height: value.frozen_height.into(),
        }
    }
}

impl From<cometbls::ConsensusState> for UnionIbcLightclientsCometblsV1ConsensusStateData {
    fn from(value: cometbls::ConsensusState) -> Self {
        Self {
            root: value.root.into(),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}

impl From<Timestamp> for GoogleProtobufTimestampData {
    fn from(value: Timestamp) -> Self {
        Self {
            secs: value.seconds,
            // REVIEW(benluelo): Is this conversion *actually* fallible?
            nanos: value.nanos.try_into().unwrap(),
        }
    }
}

impl From<Duration> for GoogleProtobufDurationData {
    fn from(value: Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<GoogleProtobufDurationData> for Duration {
    fn from(value: GoogleProtobufDurationData) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<MerkleRoot> for IbcCoreCommitmentV1MerkleRootData {
    fn from(value: MerkleRoot) -> Self {
        Self {
            hash: value.hash.into(),
        }
    }
}

impl From<Fraction> for UnionIbcLightclientsCometblsV1FractionData {
    fn from(value: Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<UnionIbcLightclientsCometblsV1FractionData> for Fraction {
    fn from(value: UnionIbcLightclientsCometblsV1FractionData) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

pub fn translate_header(
    header: ethereum_consensus::capella::LightClientHeader<256, 32>,
) -> ethereum::LightClientHeader {
    ethereum::LightClientHeader {
        beacon: BeaconBlockHeader {
            slot: header.beacon.slot.0,
            proposer_index: header.beacon.proposer_index.0,
            parent_root: header.beacon.parent_root.as_bytes().to_vec(),
            state_root: header.beacon.state_root.as_bytes().to_vec(),
            body_root: header.beacon.body_root.as_bytes().to_vec(),
        },
        execution: ExecutionPayloadHeader {
            parent_hash: header.execution.parent_hash.as_bytes().to_vec(),
            fee_recipient: header.execution.fee_recipient.0.to_vec(),
            state_root: header.execution.state_root.as_bytes().to_vec(),
            receipts_root: header.execution.receipts_root.as_bytes().to_vec(),
            logs_bloom: header.execution.logs_bloom.iter().copied().collect(),
            prev_randao: header.execution.prev_randao.as_bytes().to_vec(),
            block_number: header.execution.block_number.0,
            gas_limit: header.execution.gas_limit.0,
            gas_used: header.execution.gas_used.0,
            timestamp: header.execution.timestamp.0,
            extra_data: header.execution.extra_data.iter().copied().collect(),
            base_fee_per_gas: header.execution.base_fee_per_gas.to_bytes_le(),
            block_hash: header.execution.block_hash.as_bytes().to_vec(),
            transactions_root: header.execution.transactions_root.as_bytes().to_vec(),
            withdrawals_root: header.execution.withdrawals_root.as_bytes().to_vec(),
        },
        execution_branch: header
            .execution_branch
            .iter()
            .map(|x| x.0.to_vec())
            .collect(),
    }
}

// Deploying IBCClient...
// IBCClient => 0x86D9aC0Bab011917f57B9E9607833b4340F9D4F8
// Deploying IBCConnection...
// IBCConnection => 0xD184c103F7acc340847eEE82a0B909E3358bc28d
// Deploying IBCChannelHandshake...
// IBCChannelHandshake => 0x992B9df075935E522EC7950F37eC8557e86f6fdb
// Deploying IBCPacket...
// IBCPacket => 0x2ffA5ecdBe006d30397c7636d3e015EEE251369F
// Deploying OwnableIBCHandler...
// OwnableIBCHandler => 0xFc97A6197dc90bef6bbEFD672742Ed75E9768553
// Deploying TestnetVerifier...
// TestnetVerifier => 0xEDa338E4dC46038493b885327842fD3E301CaB39
// Deploying CometblsClient...
// CometblsClient => 0x87d1f7fdfEe7f651FaBc8bFCB6E086C278b77A7d
// Deploying ICS20Bank...
// ICS20Bank => 0x774667629726ec1FaBEbCEc0D9139bD1C8f72a23
// Deploying ICS20TransferBank...
// ICS20TransferBank => 0x83428c7db9815f482a39a1715684dCF755021997

// 50216813883093446115790550889475408280769462667188799275008
// 50216813883093446115919139012454251028917341108522774102713
