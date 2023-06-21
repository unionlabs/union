use std::{
    fmt::Debug,
    ops::Div,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use contracts::{
    glue::UnionIbcLightclientsCometblsV1HeaderData,
    ibc_handler::{
        self, GeneratedConnectionIdentifierFilter, IBCHandler, IBCHandlerEvents,
        IbcCoreChannelV1ChannelData, IbcCoreConnectionV1ConnectionEndData,
    },
    ics20_bank::ICS20Bank,
};
use ethereum_verifier::{
    BYTES_PER_LOGS_BLOOM, EPOCHS_PER_SYNC_COMMITTEE_PERIOD, MAX_EXTRA_DATA_BYTES, SLOTS_PER_EPOCH,
};
use ethers::{
    abi::AbiEncode,
    prelude::{decode_logs, k256::ecdsa, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{BlockNumber, H160, H256, U256, U64},
    utils::keccak256,
};
use futures::Future;
use ibc_types::{
    core::{
        channel::{
            channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket,
        },
        client::height::Height,
        connection::{
            connection_end::ConnectionEnd, msg_channel_open_ack::MsgConnectionOpenAck,
            msg_channel_open_confirm::MsgConnectionOpenConfirm,
            msg_channel_open_init::MsgConnectionOpenInit,
            msg_channel_open_try::MsgConnectionOpenTry,
        },
    },
    google::protobuf::any::Any,
    lightclients::{
        cometbls,
        ethereum::{
            self, account_update::AccountUpdate, beacon_block_header::BeaconBlockHeader,
            execution_payload_header::ExecutionPayloadHeader, fork::Fork,
            fork_parameters::ForkParameters, light_client_header::LightClientHeader,
            light_client_update::LightClientUpdate, proof::Proof, sync_aggregate::SyncAggregate,
            sync_committee::SyncCommittee, trusted_sync_committee::TrustedSyncCommittee,
        },
        tendermint::fraction::Fraction,
        wasm,
    },
    IntoProto,
};
use lodestar_rpc::types::BeaconHeaderResponse;
use prost::Message;
use protos::{google, union::ibc::lightclients::ethereum::v1 as ethereum_v1};

use crate::{
    chain::{cosmos::Ethereum, Connect, LightClient, StateProof},
    ETH_BEACON_RPC_API, ETH_RPC_API,
};

pub type LightClientFinalityUpdateResponse = lodestar_rpc::types::LightClientFinalityUpdateResponse<
    { ethereum_verifier::SYNC_COMMITTEE_SIZE },
    { ethereum_verifier::BYTES_PER_LOGS_BLOOM },
    { ethereum_verifier::MAX_EXTRA_DATA_BYTES },
>;

pub type LightClientBootstrapResponse = lodestar_rpc::types::LightClientBootstrapResponse<
    { ethereum_verifier::SYNC_COMMITTEE_SIZE },
    { ethereum_verifier::BYTES_PER_LOGS_BLOOM },
    { ethereum_verifier::MAX_EXTRA_DATA_BYTES },
>;

pub type LightClientUpdatesResponse = lodestar_rpc::types::LightClientUpdatesResponse<
    { ethereum_verifier::SYNC_COMMITTEE_SIZE },
    { ethereum_verifier::BYTES_PER_LOGS_BLOOM },
    { ethereum_verifier::MAX_EXTRA_DATA_BYTES },
>;

pub type LightClientUpdateData = lodestar_rpc::types::LightClientUpdateData<
    { ethereum_verifier::SYNC_COMMITTEE_SIZE },
    { ethereum_verifier::BYTES_PER_LOGS_BLOOM },
    { ethereum_verifier::MAX_EXTRA_DATA_BYTES },
>;

// TODO(benluelo): Either pass this in or calculate it somehow
const PERIOD: u64 = EPOCHS_PER_SYNC_COMMITTEE_PERIOD * SLOTS_PER_EPOCH;

pub const COMETBLS_CLIENT_TYPE: &str = "cometbls-new";

/// The solidity light client, tracking the state of the 08-wasm light client on union.
// TODO(benluelo): Generic over middleware?
pub struct Cometbls {
    pub ibc_handler: IBCHandler<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>>,
    pub ics20_bank: ICS20Bank<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>>,
    pub provider: Provider<Http>,
    cometbls_client_address: H160,
    ics20_transfer_address: H160,
    wasm_code_id: H256,
    eth_beacon_rpc_api: String,
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
// actual cometbls client returns the bytes it receives on creation, which is the dynamic singleton
// tuple encoded struct)
// fn decode_dynamic_singleton_tuple<T: AbiDecode>(bs: &[u8]) -> T {
//     let tuple_idx_bytes = U256::from(32).encode().len();

//     T::decode(&bs[tuple_idx_bytes..]).unwrap()
// }

impl LightClient for Cometbls {
    type ClientState = Any<wasm::client_state::ClientState<cometbls::client_state::ClientState>>;
    type ConsensusState =
        Any<wasm::consensus_state::ConsensusState<cometbls::consensus_state::ConsensusState>>;
    // TODO(benluelo): Better type for this
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

            tracing::info!(ibc_handler_address = ?self.ibc_handler.address());

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
                    .map(Into::into)
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

            tracing::info!(block_number = ?tx_rcp.block_number);

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            // let (consensus_state, is_found) = self
            //     .ibc_handler
            //     .get_consensus_state(
            //         client_id.clone(),
            //         IbcCoreClientV1HeightData {
            //             revision_number: 0,
            //             revision_height: tx_rcp.block_number.unwrap().as_u64(),
            //         },
            //     )
            //     .call()
            //     .await
            //     .unwrap();

            // assert!(is_found);

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
            self.wait_for_beacon_block(self_height).await;

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
            // tracing::info!(?self_height);
            // self.wait_for_beacon_block(self_height).await;
            let self_height = self.execution_height(self_height).await;
            self.wait_for_execution_block(self_height.revision_height.into())
                .await;

            let block_number = self.provider.get_block_number().await.unwrap();
            tracing::info!(?block_number, ?self_height);

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
            self.wait_for_beacon_block(self_height).await;

            let (connection_end, is_found): (IbcCoreConnectionV1ConnectionEndData, bool) = self
                .ibc_handler
                .get_connection(connection_id.clone())
                .block(self_height.revision_height)
                .await
                .unwrap();

            tracing::info!(?connection_end);

            assert!(is_found);

            let canonical_connection_end: ConnectionEnd = connection_end.try_into().unwrap();

            self.get_proof(
                format!("connections/{connection_id}"),
                self_height,
                canonical_connection_end,
                |x| x.into_proto().encode_to_vec(),
            )
            .await
        }
    }

    fn channel_state_proof(
        &self,
        channel_id: String,
        port_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Channel>> + '_ {
        async move {
            tracing::info!(?self_height);
            self.wait_for_beacon_block(self_height).await;

            let (channel, is_found): (IbcCoreChannelV1ChannelData, bool) = self
                .ibc_handler
                .get_channel(port_id.clone(), channel_id.clone())
                .block(self_height.revision_height)
                .await
                .unwrap();

            tracing::info!(?channel);

            assert!(is_found);

            let canonical_channel: Channel = channel.try_into().unwrap();

            self.get_proof(
                format!("channelEnds/ports/{port_id}/channels/{channel_id}"),
                self_height,
                canonical_channel,
                |x| x.into_proto().encode_to_vec(),
            )
            .await
        }
    }

    fn query_latest_height(&self) -> impl Future<Output = Height> + '_ {
        async move {
            let height = reqwest::get(format!(
                "{eth_beacon_rpc_api}/eth/v2/debug/beacon/states/finalized",
                eth_beacon_rpc_api = self.eth_beacon_rpc_api
            ))
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap()["data"]["slot"]
                .as_str()
                .unwrap()
                .parse()
                .unwrap();

            Height {
                revision_number: 0,
                revision_height: height,
            }
        }
    }

    fn query_client_state(
        &self,
        client_id: String,
    ) -> impl Future<Output = Self::ClientState> + '_ {
        async move {
            let (client_state_bytes, is_found) = self
                .ibc_handler
                .get_client_state(client_id.clone())
                .await
                .unwrap();

            assert!(is_found);

            google::protobuf::Any::decode(&*client_state_bytes)
                .unwrap()
                .try_into()
                .unwrap()
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

            // TODO(benluelo): Better way to get logs
            let connection_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
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

            tracing::info!("in conninit, waiting for execution block to be finalized");
            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
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

            // self.wait_for_beacon_block(Height {
            //     revision_number: 0,
            //     revision_height: tx_rcp.block_number.unwrap().0[0],
            // })
            // .await;

            decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
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
            // TODO: Make sure this is done in both init and try
            let bind_port_result = self
                .ibc_handler
                .bind_port("transfer".to_string(), self.ics20_transfer_address);

            match bind_port_result.send().await {
                Ok(ok) => {
                    ok.await.unwrap().unwrap();
                }
                Err(why) => tracing::info!(why = ?why.decode_revert::<String>()),
            }

            let tx_rcp = self
                .ibc_handler
                .channel_open_init(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let channel_id = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
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
            .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            channel_id
        }
    }

    fn channel_open_try(&self, msg: MsgChannelOpenTry) -> impl Future<Output = String> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .channel_open_try(msg.into())
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
                    .map(Into::into)
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
            let tx_rcp = self
                .ibc_handler
                .recv_packet(packet.into())
                .send()
                .await
                // .map_err(|err| err.decode_revert::<String>())
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            let events = decode_logs::<IBCHandlerEvents>(
                tx_rcp
                    .logs
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .as_ref(),
            )
            .unwrap();

            dbg!(events);
        }
    }

    fn generate_counterparty_client_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ClientState> + '_ {
        async move {
            let genesis = lodestar_rpc::client::RPCClient::new(self.eth_beacon_rpc_api.clone())
                .get_genesis()
                .await
                .unwrap()
                .data;

            let execution_height = self.execution_height(beacon_height).await;

            Any(wasm::client_state::ClientState {
                data: ethereum::client_state::ClientState {
                    genesis_validators_root: genesis.genesis_validators_root.as_bytes().to_vec(),
                    genesis_time: genesis.genesis_time.0,
                    fork_parameters: ForkParameters {
                        genesis_fork_version: vec![0, 0, 0, 1],
                        genesis_slot: 0,
                        altair: Fork {
                            version: vec![1, 0, 0, 1],
                            epoch: 0,
                        },
                        bellatrix: Fork {
                            version: vec![2, 0, 0, 1],
                            epoch: 0,
                        },
                        capella: Fork {
                            version: vec![3, 0, 0, 1],
                            epoch: 0,
                        },
                        eip4844: Fork {
                            version: vec![4, 0, 0, 0],
                            epoch: u64::MAX,
                        },
                    },
                    seconds_per_slot: 6,
                    slots_per_epoch: 8,
                    epochs_per_sync_committee_period: 8,
                    trusting_period: 100_000_000,
                    latest_slot: beacon_height.revision_height,
                    min_sync_committee_participants: 0,
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    frozen_height: None,
                    counterparty_commitment_slot: 0,
                },
                code_id: self.wasm_code_id.to_fixed_bytes().to_vec(),
                latest_height: execution_height,
            })
        }
    }

    fn generate_counterparty_consensus_state(
        &self,
        beacon_height: Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ConsensusState> + '_ {
        async move {
            let trusted_header =
                lodestar_rpc::client::RPCClient::new(self.eth_beacon_rpc_api.as_str())
                    .get_block_header("finalized".to_string())
                    .await
                    .unwrap()
                    .data;

            let bootstrap = reqwest::get(dbg!(format!(
                "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/bootstrap/0x{root}",
                eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                root = trusted_header.root
            )))
            .await
            .unwrap()
            .json::<LightClientBootstrapResponse>()
            .await
            .unwrap()
            .data;

            let light_client_update = {
                let current_period = beacon_height.revision_height.div(PERIOD);

                tracing::info!(%current_period);

                let light_client_updates = reqwest::get(dbg!(format!(
                    "{}/eth/v1/beacon/light_client/updates?start_period={current_period}&count=1",
                    self.eth_beacon_rpc_api
                )))
                .await
                .unwrap()
                .json::<LightClientUpdatesResponse>()
                .await
                .unwrap();

                let [light_client_update] = &*light_client_updates.0 else { panic!() };

                light_client_update.data.clone()
            };

            Any(wasm::consensus_state::ConsensusState {
                data: ethereum::consensus_state::ConsensusState {
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

    /// Returns the actual height updated to (>= `update_to`)
    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Ethereum,
        counterparty_client_id: String,
        mut update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + 'a {
        async move {
            assert!(
                update_to >= update_from,
                "cannot update to block in the past: {update_to} >= {update_from}"
            );

            self.wait_for_beacon_block(update_to).await;

            let trusted_header =
                lodestar_rpc::client::RPCClient::new(self.eth_beacon_rpc_api.clone())
                    .get_block_header("finalized".to_string())
                    .await
                    .unwrap()
                    .data;

            let beacon_slot = trusted_header.header.message.slot.0;

            let update_to_period = beacon_slot.div(PERIOD);
            let current_period = update_from.revision_height.div(PERIOD);

            tracing::info!(%current_period);

            let mut trusted_block = reqwest::get(format!(
                "{eth_beacon_rpc_api}/eth/v1/beacon/headers/{height}",
                eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                height = update_from.revision_height
            ))
            .await
            .unwrap()
            .json::<BeaconHeaderResponse>()
            .await
            .unwrap()
            .data;

            // +1 here because we want to update to the `update_to_period`'s period.
            let periods_to_update = update_to_period - current_period + 1;

            tracing::debug!(update_to_period, current_period);

            // We are looping here because some of the updates might not be available yet. We understand that when
            // we see the finalized header's slot as 0.
            let light_client_updates = loop {
                let updates =
                    reqwest::get(format!(
                        "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/updates?start_period={current_period}&count={count}",
                        eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                        count = periods_to_update,
                    ))
                    .await
                    .unwrap()
                    .json::<LightClientUpdatesResponse>()
                    .await
                    .unwrap()
                    .0;

                if updates
                    .iter()
                    .any(|update| update.data.finalized_header.beacon.slot.0 == 0)
                {
                    tracing::debug!("lightclient update not available yet; retrying in 3 seconds");

                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    continue;
                }

                break updates;
            };

            // REVIEW(benluelo): Is there a way to make this not have to be mutable?
            let mut light_client_update = light_client_updates.last().unwrap().clone();

            // don't do sync committee updates if the sync committee period is the same
            if update_to_period != current_period {
                // send the sync committee update if we are at the beginning of the period
                for light_client_update in &light_client_updates[1..] {
                    tracing::info!("applying light client update");

                    // bootstrap contains the current sync committee for the given height
                    let bootstrap = reqwest::get(format!(
                        "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/bootstrap/0x{root}",
                        eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                        root = trusted_block.root,
                    ))
                    .await
                    .unwrap()
                    .json::<LightClientBootstrapResponse>()
                    .await
                    .unwrap();

                    let account_update = self
                        .provider
                        .get_proof(
                            self.ibc_handler.address(),
                            vec![],
                            Some(
                                light_client_update
                                    .data
                                    .finalized_header
                                    // REVIEW: Do we want execution.block_number here?
                                    .beacon
                                    .slot
                                    .0
                                    .into(),
                            ),
                        )
                        .await
                        .unwrap();

                    let execution_height = self
                        .execution_height(Height {
                            revision_number: 0,
                            revision_height: bootstrap.data.header.beacon.slot.0,
                        })
                        .await;

                    let header = wasm::header::Header {
                        data: ethereum::header::Header {
                            trusted_sync_committee: TrustedSyncCommittee {
                                trusted_height:
                                    // NOTE: should be the same as trusted height passed in to this function
                                    execution_height,
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
                                is_next: true,
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
                                        aggregate_pubkey: next_sync_committee
                                            .aggregate_pubkey
                                            .to_vec(),
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

                    tracing::debug!(
                        message = "Checking if updated height > update from revision height",
                        finalized_slot = header.data.consensus_update.finalized_header.beacon.slot,
                        update_from = %update_from
                    );

                    // If we update, we also need to advance `update_from`
                    if header.data.consensus_update.finalized_header.beacon.slot
                        > update_from.revision_height
                    {
                        trusted_block = reqwest::get(format!(
                            "{eth_beacon_rpc_api}/eth/v1/beacon/headers/{slot}",
                            eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                            slot = light_client_update.data.finalized_header.beacon.slot.0
                        ))
                        .await
                        .unwrap()
                        .json::<BeaconHeaderResponse>()
                        .await
                        .unwrap()
                        .data;

                        update_from = Height {
                            revision_number: 0,
                            revision_height: header
                                .data
                                .consensus_update
                                .finalized_header
                                .beacon
                                .slot,
                        };
                    }

                    counterparty
                        .update_client(counterparty_client_id.clone(), header)
                        .await;
                }
            }

            // We might be already updated to the height that we want, no need to proceed.
            if update_to == update_from {
                tracing::info!(%update_from, "requested height {update_to} already reached");

                return update_to;
            }

            // wait until the beacon (execution?) height is >= the latest trusted height
            let finality_update = loop {
                let finality_update = reqwest::get(format!(
                    "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/finality_update",
                    eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                ))
                .await
                .unwrap()
                .json::<LightClientFinalityUpdateResponse>()
                .await
                .unwrap();

                // REVIEW: Do we want execution.block_number here?
                let current_slot = finality_update.data.finalized_header.beacon.slot.0;

                tracing::info!(
                    update_from = %update_from,
                    update_to = %update_to,
                    current = %current_slot
                );

                if current_slot >= update_to.revision_height {
                    break finality_update.data;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            };

            let actual_updated_height = Height {
                revision_number: 0,
                revision_height: finality_update.finalized_header.beacon.slot.0,
            };

            // send the finality update

            // whether the sync committee signature is to be checked against the current or next sync committee
            let is_next = finality_update.finalized_header.beacon.slot.0 % PERIOD == 0;

            let bootstrap = reqwest::get(format!(
                "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/bootstrap/0x{root}",
                eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                root = trusted_block.root,
            ))
            .await
            .unwrap()
            .json::<LightClientBootstrapResponse>()
            .await
            .unwrap();

            let execution_height = self.execution_height(update_from).await;

            let trusted_sync_committee = if is_next {
                TrustedSyncCommittee {
                    trusted_height: execution_height,
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
                    trusted_height: execution_height,
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
                    Some(
                        // REVIEW: Do we want finalized_header.beacon.slot here?
                        finality_update
                            .finalized_header
                            .execution
                            .block_number
                            .0
                            .into(),
                    ),
                )
                .await
                .unwrap();

            // Even we make sure that we update until the latest period with sync committee updates, there is still a
            // chance that the store period can increment while we wait for `update_to` to be finalized. This happens
            // when `update_to` is very close to the next period. When that's the case, the block that we'll update to
            // will be in the next period, so we combine the sync committee update with finality update.
            if is_next {
                tracing::info!("will try finality update with is_next=true");
                let update_period = finality_update.finalized_header.beacon.slot.0.div(PERIOD);
                let light_client_updates = reqwest::get(format!(
                    "{eth_beacon_rpc_api}/eth/v1/beacon/light_client/updates?start_period={update_period}&count={count}",
                    eth_beacon_rpc_api = self.eth_beacon_rpc_api,
                    count = 1,
                ))
                .await
                .unwrap()
                .json::<LightClientUpdatesResponse>()
                .await
                .unwrap()
                .0;

                light_client_update = light_client_updates.last().unwrap().clone();
            }

            let header = wasm::header::Header {
                height: actual_updated_height,
                data: ethereum::header::Header {
                    trusted_sync_committee,
                    consensus_update: LightClientUpdate {
                        attested_header: translate_header(finality_update.attested_header),
                        // TODO(benluelo): make into Option
                        next_sync_committee: if is_next {
                            SyncCommittee {
                                pubkeys: light_client_update
                                    .data
                                    .next_sync_committee
                                    .pubkeys
                                    .iter()
                                    .map(|x| x.0.to_vec())
                                    .collect(),
                                aggregate_pubkey: light_client_update
                                    .data
                                    .next_sync_committee
                                    .aggregate_pubkey
                                    .to_vec(),
                            }
                        } else {
                            Default::default()
                        },
                        next_sync_committee_branch: if is_next {
                            light_client_update
                                .data
                                .next_sync_committee_branch
                                .to_vec()
                                .iter()
                                .map(|x| x.as_bytes().to_vec())
                                .collect()
                        } else {
                            Default::default()
                        },
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

            actual_updated_height
        }
    }
}

pub struct CometblsConfig {
    pub cometbls_client_address: H160,
    pub ibc_handler_address: H160,
    pub ics20_transfer_address: H160,
    pub ics20_bank_address: H160,

    pub wasm_code_id: H256,

    pub wallet: LocalWallet,

    pub eth_rpc_api: Url,
    // TODO(benluelo): Make this into a `Url`
    pub eth_beacon_rpc_api: String,
}

impl Cometbls {
    pub async fn new(config: CometblsConfig) -> Self {
        let provider = Provider::new(Http::new(config.eth_rpc_api));

        let chain_id = provider.get_chainid().await.unwrap();

        let wallet = config.wallet.with_chain_id(chain_id.as_u64());

        let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet.clone()));

        let ibc_handler =
            ibc_handler::IBCHandler::new(config.ibc_handler_address, signer_middleware.clone());

        let ics20_bank = ICS20Bank::new(config.ics20_bank_address, signer_middleware);

        // ics20_bank
        //     .set_operator(config.ics20_transfer_address)
        //     .send()
        //     .await
        //     .unwrap()
        //     .await
        //     .unwrap()
        //     .unwrap();

        Self {
            ibc_handler,
            ics20_bank,
            provider,
            cometbls_client_address: config.cometbls_client_address,
            ics20_transfer_address: config.ics20_transfer_address,
            wasm_code_id: config.wasm_code_id,
            eth_beacon_rpc_api: config.eth_beacon_rpc_api,
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

        let location = keccak256(
            keccak256(path.as_bytes())
                .into_iter()
                .chain(u256)
                .collect::<Vec<_>>(),
        );

        let proof = self
            .provider
            .get_proof(
                self.ibc_handler.address(),
                vec![location.into()],
                Some(height.revision_height.into()),
            )
            .await
            .unwrap();

        tracing::info!(?proof);

        let proof = match <[_; 1]>::try_from(proof.storage_proof) {
            Ok([proof]) => proof,
            Err(invalid) => {
                panic!("received invalid response from eth_getProof, expected length of 1 but got {invalid:#?}");
            }
        };

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

    async fn execution_height(&self, beacon_height: Height) -> Height {
        let height = reqwest::get(format!(
            "{eth_beacon_rpc_api}/eth/v2/debug/beacon/states/{slot}",
            eth_beacon_rpc_api = self.eth_beacon_rpc_api,
            slot = beacon_height.revision_height
        ))
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["data"]["latest_execution_payload_header"]["block_number"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();

        Height {
            revision_number: 0,
            revision_height: height,
        }
    }

    async fn wait_for_beacon_block(&self, requested_height: Height) {
        loop {
            let current_block = self.query_latest_height().await;

            tracing::debug!(?current_block, waiting_for = ?requested_height, "waiting for block");

            if current_block.revision_height >= requested_height.revision_height {
                break;
            }

            tracing::debug!("requested height not yet reached");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    async fn wait_for_execution_block(&self, block_number: U64) {
        loop {
            let latest_finalized_block_number = self
                .provider
                .get_block(BlockNumber::Finalized)
                .await
                .unwrap()
                .unwrap()
                .number
                .unwrap();

            tracing::debug!(
                %latest_finalized_block_number,
                waiting_for = %block_number,
                "waiting for block"
            );

            if latest_finalized_block_number >= block_number {
                break;
            }

            tracing::debug!("requested height not yet reached");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

pub trait IntoEthAbi: Into<Self::EthAbi> {
    type EthAbi;

    fn into_eth_abi(self) -> Self::EthAbi {
        self.into()
    }
}

pub fn translate_header(
    header: ethereum_consensus::capella::LightClientHeader<
        { BYTES_PER_LOGS_BLOOM },
        { MAX_EXTRA_DATA_BYTES },
    >,
) -> LightClientHeader {
    LightClientHeader {
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
