use std::{
    fmt::Debug,
    ops::Div,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use beacon_api::client::BeaconApiClient;
use contracts::{
    glue::UnionIbcLightclientsCometblsV1HeaderData,
    ibc_handler::{
        self, GeneratedConnectionIdentifierFilter, IBCHandler, IBCHandlerEvents,
        IbcCoreChannelV1ChannelData, IbcCoreConnectionV1ConnectionEndData, SendPacketFilter,
    },
    ics20_bank::ICS20Bank,
};
use ethers::{
    abi::AbiEncode,
    prelude::{decode_logs, k256::ecdsa, parse_log, LogMeta, SignerMiddleware},
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer, Wallet},
    types::{H160, H256, U256, U64},
    utils::keccak256,
};
use futures::{Future, Stream, StreamExt};
use ibc_types::{
    ethereum::beacon::{LightClientBootstrap, LightClientFinalityUpdate},
    ethereum_consts_traits::ChainSpec,
    ibc::{
        core::{
            channel::{
                channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
                msg_channel_open_confirm::MsgChannelOpenConfirm,
                msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
                msg_recv_packet::MsgRecvPacket, packet::Packet,
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
                self,
                account_update::AccountUpdate,
                light_client_update::{LightClientUpdate, NextSyncCommitteeBranch},
                proof::Proof,
                sync_committee::SyncCommittee,
                trusted_sync_committee::TrustedSyncCommittee,
            },
            tendermint::fraction::Fraction,
            wasm,
        },
    },
    IntoProto,
};
use prost::Message;
use protos::{google, union::ibc::lightclients::ethereum::v1 as ethereum_v1};
use reqwest::Url;
use typenum::Unsigned;

use crate::chain::{cosmos::Ethereum, Connect, LightClient, StateProof};

pub const COMETBLS_CLIENT_TYPE: &str = "cometbls-new";

/// The solidity light client, tracking the state of the 08-wasm light client on union.
// TODO(benluelo): Generic over middleware?
pub struct Cometbls<C: ChainSpec> {
    pub ibc_handler: IBCHandler<SignerMiddleware<Provider<Ws>, Wallet<ecdsa::SigningKey>>>,
    pub ics20_bank: ICS20Bank<SignerMiddleware<Provider<Ws>, Wallet<ecdsa::SigningKey>>>,
    pub provider: Provider<Ws>,
    cometbls_client_address: H160,
    ics20_transfer_address: H160,
    wasm_code_id: H256,
    beacon_api_client: BeaconApiClient<C>,
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

impl<C: ChainSpec> LightClient for Cometbls<C> {
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

            tracing::info!(block_number = ?self.make_height(tx_rcp.block_number.unwrap().as_u64()));

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
            // tracing::info!(?self_height);
            // self.wait_for_beacon_block(self_height).await;
            let self_height = self.execution_height(self_height).await;
            self.wait_for_execution_block(self_height.revision_height.into())
                .await;

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

            let cometbls_client_state: Self::ClientState =
                google::protobuf::Any::decode(&*client_state_bytes)
                    .unwrap()
                    .try_into()
                    .unwrap();

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
            // tracing::info!(?self_height);
            // self.wait_for_beacon_block(self_height).await;
            let self_height = self.execution_height(self_height).await;
            self.wait_for_execution_block(self_height.revision_height.into())
                .await;

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
            // tracing::info!(?self_height);
            // self.wait_for_beacon_block(self_height).await;
            let self_height = self.execution_height(self_height).await;
            self.wait_for_execution_block(self_height.revision_height.into())
                .await;

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

    fn process_height_for_counterparty(&self, height: Height) -> impl Future<Output = Height> + '_ {
        self.execution_height(height)
    }

    fn packet_commitment_proof(
        &self,
        port_id: String,
        channel_id: String,
        sequence: u64,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Vec<u8>>> + '_ {
        async move {
            // tracing::info!(?self_height);
            // self.wait_for_beacon_block(self_height).await;
            let self_height = self.execution_height(self_height).await;
            self.wait_for_execution_block(self_height.revision_height.into())
                .await;

            self.get_proof(
                format!("commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}"),
                self_height,
                vec![],
                |_| vec![],
            )
            .await
        }
    }

    fn packet_stream(
        &self,
    ) -> impl Future<Output = impl Stream<Item = (Height, Packet)> + '_> + '_ {
        async move {
            self.provider
                .subscribe_logs(&self.ibc_handler.event::<SendPacketFilter>().filter)
                .await
                .unwrap()
                .then(move |log| async move {
                    let meta = LogMeta::from(&log);
                    let event: SendPacketFilter = parse_log(log).unwrap();

                    let (channel_data, _): (
                        contracts::ibc_handler::IbcCoreChannelV1ChannelData,
                        bool,
                    ) = self
                        .ibc_handler
                        .get_channel(event.source_port.clone(), event.source_channel.clone())
                        // .block(
                        //     self
                        //         .process_height_for_counterparty(updated_height)
                        //         .await
                        //         .revision_height,
                        // )
                        .await
                        .unwrap();

                    (
                        self.make_height(meta.block_number.0[0]),
                        Packet {
                            sequence: event.sequence,
                            source_port: event.source_port,
                            source_channel: event.source_channel,
                            destination_port: channel_data.counterparty.port_id,
                            destination_channel: channel_data.counterparty.channel_id,
                            data: event.data.to_vec(),
                            timeout_height: Height {
                                revision_number: event.timeout_height.revision_number,
                                revision_height: event.timeout_height.revision_height,
                            },
                            timeout_timestamp: event.timeout_timestamp,
                        },
                    )
                })

            // while let Some(Ok((event, meta))) = event_stream.next().await {
            //     let event: contracts::ibc_handler::SendPacketFilter = event;

            //     tracing::info!(event = ?event, "new event");
            //     println!("EVENT DATA: {:?}", event.data.to_vec());

            //     cometbls
            //         .wait_for_execution_block(meta.block_number.as_u64().into())
            //         .await;

            //     let latest_height = ethereum
            //         .query_client_state(ethereum_client_id.clone())
            //         .await
            //         .height();

            //     let updated_height = cometbls
            //         .update_counterparty_client(
            //             &ethereum,
            //             ethereum_client_id.clone(),
            //             latest_height,
            //             cometbls.query_latest_height().await,
            //         )
            //         .await;

            //     let commitment_proof = cometbls
            //         .packet_commitment_proof(
            //             event.source_port.clone(),
            //             event.source_channel.clone(),
            //             event.sequence,
            //             updated_height,
            //         )
            //         .await;

            //     let (channel_data, _): (contracts::ibc_handler::IbcCoreChannelV1ChannelData, bool) =
            //         cometbls
            //             .ibc_handler
            //             .get_channel(event.source_port.clone(), event.source_channel.clone())
            //             .block(
            //                 cometbls
            //                     .process_height_for_counterparty(updated_height)
            //                     .await
            //                     .revision_height,
            //             )
            //             .await
            //             .unwrap();

            //     let rcp = ethereum
            //         .recv_packet(MsgRecvPacket {
            //             packet: Packet {
            //                 sequence: event.sequence,
            //                 source_port: event.source_port,
            //                 source_channel: event.source_channel,
            //                 destination_port: channel_data.counterparty.port_id,
            //                 destination_channel: channel_data.counterparty.channel_id,
            //                 data: event.data.to_vec(),
            //                 timeout_height: Height::new(
            //                     event.timeout_height.revision_number,
            //                     event.timeout_height.revision_height,
            //                 ),
            //                 timeout_timestamp: event.timeout_timestamp,
            //             },
            //             proof_commitment: commitment_proof.proof,
            //             proof_height: commitment_proof.proof_height,
            //         })
            //         .await;

            //     tracing::info!(rcp = ?rcp, "received packet");
            // }
        }
    }
}

impl<C: ChainSpec> Connect<Ethereum<C>> for Cometbls<C> {
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     counterparty_state: <Ethereum as LightClient>::ClientState,
    // ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
    //     async move { todo!() }
    // }

    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_ {
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

            tracing::info!("in connection open init, waiting for execution block to be finalized");
            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                connection_id,
                self.make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<<Ethereum<C> as LightClient>::ClientState>,
    ) -> impl Future<Output = (String, Height)> + '_ {
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
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(connection_id) => {
                    Some(connection_id.0)
                }
                _ => None,
            })
            .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            (
                connection_id,
                self.make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<<Ethereum<C> as LightClient>::ClientState>,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            tracing::debug!(
                "Client state: {}",
                ethers::utils::hex::encode(msg.client_state.clone().into_proto().encode_to_vec())
            );

            let msg: contracts::ibc_handler::MsgConnectionOpenAck = msg.into();

            tracing::debug!(
                "Client state bytes {}",
                ethers::utils::hex::encode(&msg.client_state_bytes)
            );

            let tx_rcp = self
                .ibc_handler
                .connection_open_ack(msg)
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .connection_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn channel_open_init(
        &self,
        msg: MsgChannelOpenInit,
    ) -> impl Future<Output = (String, Height)> + '_ {
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

            (
                channel_id,
                self.make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn channel_open_try(
        &self,
        msg: MsgChannelOpenTry,
    ) -> impl Future<Output = (String, Height)> + '_ {
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

            (
                channel_id,
                self.make_height(tx_rcp.block_number.unwrap().as_u64()),
            )
        }
    }

    fn channel_open_ack(&self, msg: MsgChannelOpenAck) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .channel_open_ack(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn channel_open_confirm(
        &self,
        msg: MsgChannelOpenConfirm,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .channel_open_confirm(msg.into())
                .send()
                .await
                .unwrap()
                .await
                .unwrap()
                .unwrap();

            self.wait_for_execution_block(tx_rcp.block_number.unwrap())
                .await;

            self.make_height(tx_rcp.block_number.unwrap().as_u64())
        }
    }

    fn recv_packet(&self, packet: MsgRecvPacket) -> impl Future<Output = ()> + '_ {
        async move {
            let tx_rcp = self
                .ibc_handler
                .recv_packet(packet.into())
                .send()
                .await
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
    ) -> impl Future<Output = <Ethereum<C> as LightClient>::ClientState> + '_ {
        async move {
            let genesis = self.beacon_api_client.genesis().await.unwrap().data;

            let execution_height = self.execution_height(beacon_height).await;

            Any(wasm::client_state::ClientState {
                data: ethereum::client_state::ClientState {
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
    ) -> impl Future<Output = <Ethereum<C> as LightClient>::ConsensusState> + '_ {
        async move {
            let trusted_header = self
                .beacon_api_client
                .header(beacon_api::client::BlockId::Finalized)
                .await
                .unwrap()
                .data;

            let bootstrap = self
                .beacon_api_client
                .bootstrap(trusted_header.root)
                .await
                .unwrap()
                .data;

            let light_client_update = {
                let current_period = beacon_height.revision_height.div(C::PERIOD::U64);

                tracing::info!(%current_period);

                let light_client_updates = self
                    .beacon_api_client
                    .light_client_updates(current_period, 1)
                    .await
                    .unwrap();

                let [light_client_update] = &*light_client_updates.0 else { panic!() };

                light_client_update.data.clone()
            };

            Any(wasm::consensus_state::ConsensusState {
                data: ethereum::consensus_state::ConsensusState {
                    slot: bootstrap.header.beacon.slot,
                    storage_root: Default::default(),
                    timestamp: bootstrap.header.execution.timestamp,
                    current_sync_committee: bootstrap.current_sync_committee.aggregate_pubkey,
                    next_sync_committee: light_client_update
                        .next_sync_committee
                        .map(|nsc| nsc.aggregate_pubkey),
                },
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        }
    }

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Ethereum<C>,
        counterparty_client_id: String,
        mut trusted_slot: Height,
        target_slot: Height,
    ) -> impl Future<Output = Height> + 'a {
        async move {
            // We need to wait until the target slot is attested, because the update
            // won't be available otherwise.
            self.wait_for_beacon_block(target_slot).await;

            let finality_update = self.beacon_api_client.finality_update().await.unwrap();
            let target_period =
                self.sync_committee_period(finality_update.data.attested_header.beacon.slot);
            let trusted_period = self.sync_committee_period(trusted_slot.revision_height);

            assert!(trusted_period <= target_period, "Chain's current signature period cannot be behind of the saved state, something is wrong!");

            // Eth chain is more than 1 signature period ahead of us. We need to do sync committee
            // updates until we reach the `target_period - 1`.
            if trusted_period < target_period {
                tracing::debug!(
                    "Will update multiple sync committees from period {}, to {}",
                    trusted_period,
                    target_period
                );
                trusted_slot = self
                    .apply_sync_committee_updates(
                        counterparty,
                        &counterparty_client_id,
                        trusted_slot,
                        target_period,
                    )
                    .await;
            }

            if trusted_slot >= target_slot {
                return trusted_slot;
            }

            let execution_height = self.execution_height(trusted_slot).await;

            let updated_height = self.make_height(finality_update.data.attested_header.beacon.slot);
            let block_root = self
                .beacon_api_client
                .header(beacon_api::client::BlockId::Slot(
                    trusted_slot.revision_height,
                ))
                .await
                .unwrap()
                .data
                .root;
            let bootstrap = self.beacon_api_client.bootstrap(block_root).await.unwrap();

            let header = self
                .make_finality_update(
                    finality_update.data,
                    TrustedSyncCommittee {
                        trusted_height: execution_height,
                        sync_committee: bootstrap.data.current_sync_committee,
                        is_next: false,
                    },
                    None,
                    None,
                )
                .await;

            let header_json = serde_json::to_string(&header).unwrap();

            tracing::info!(%header_json, "submitting finality update");

            counterparty
                .update_client(counterparty_client_id, header)
                .await;

            updated_height
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

impl<C: ChainSpec> Cometbls<C> {
    async fn apply_sync_committee_updates(
        &self,
        counterparty: &Ethereum<C>,
        counterparty_client_id: &str,
        mut trusted_slot: Height,
        target_period: u64,
    ) -> Height {
        let trusted_period = self.sync_committee_period(trusted_slot.revision_height);

        let light_client_updates = loop {
            let updates = self
                .beacon_api_client
                .light_client_updates(trusted_period + 1, target_period - trusted_period)
                .await
                .unwrap();

            if updates
                .0
                .iter()
                .any(|update| update.data.finalized_header.beacon.slot == 0)
            {
                tracing::debug!("lightclient update not available yet; retrying in 3 seconds");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                continue;
            }

            break updates;
        };

        let mut trusted_block = self
            .beacon_api_client
            .header(beacon_api::client::BlockId::Slot(
                trusted_slot.revision_height,
            ))
            .await
            .unwrap()
            .data;

        for light_client_update in light_client_updates.0 {
            tracing::info!("applying light client update");

            // bootstrap contains the current sync committee for the given height
            let bootstrap = self
                .beacon_api_client
                .bootstrap(trusted_block.root.clone())
                .await
                .unwrap()
                .data;

            let header = self
                .make_sync_committee_update(bootstrap.clone(), light_client_update.clone().data)
                .await;

            tracing::debug!(
                message = "Checking if updated height > update from revision height",
                finalized_slot = header.data.consensus_update.finalized_header.beacon.slot,
                update_from = %trusted_slot
            );

            // If we update, we also need to advance `update_from`
            if header.data.consensus_update.attested_header.beacon.slot
                > trusted_slot.revision_height
            {
                trusted_block = self
                    .beacon_api_client
                    .header(beacon_api::client::BlockId::Slot(
                        light_client_update.data.attested_header.beacon.slot,
                    ))
                    .await
                    .unwrap()
                    .data;

                trusted_slot =
                    self.make_height(header.data.consensus_update.attested_header.beacon.slot);
            }

            counterparty
                .update_client(counterparty_client_id.into(), header)
                .await;
        }

        trusted_slot
    }

    pub async fn new(config: CometblsConfig) -> Self {
        let provider = Provider::new(Ws::connect(config.eth_rpc_api).await.unwrap());

        let chain_id = provider.get_chainid().await.unwrap();

        let wallet = config.wallet.with_chain_id(chain_id.as_u64());

        let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

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
            beacon_api_client: BeaconApiClient::new(config.eth_beacon_rpc_api).await,
        }
    }

    async fn get_proof<S: Clone + Debug>(
        &self,
        path: String,
        height: Height,
        state: S,
        _encode: impl FnOnce(S) -> Vec<u8>,
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

        // let found_value = U256::from(keccak256(encode(state.clone())));

        // assert_eq!(found_value, proof.value);

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
        let height = self
            .beacon_api_client
            .block(beacon_api::client::BlockId::Slot(
                beacon_height.revision_height,
            ))
            .await
            .unwrap()
            .data
            .message
            .body
            .execution_payload
            .block_number;

        self.make_height(height)
    }

    async fn make_sync_committee_update(
        &self,
        bootstrap: LightClientBootstrap<C>,
        update: LightClientUpdate<C>,
    ) -> wasm::header::Header<ethereum::header::Header<C>> {
        self.make_update(
            LightClientUpdate {
                attested_header: update.attested_header,
                next_sync_committee: update.next_sync_committee,
                next_sync_committee_branch: update.next_sync_committee_branch,
                finalized_header: update.finalized_header,
                finality_branch: update.finality_branch,
                sync_aggregate: update.sync_aggregate,
                signature_slot: update.signature_slot,
            },
            TrustedSyncCommittee {
                trusted_height: self
                    .execution_height(self.make_height(bootstrap.header.beacon.slot))
                    .await,
                sync_committee: bootstrap.current_sync_committee,
                is_next: true,
            },
        )
        .await
    }

    fn sync_committee_period<H: Into<u64>>(&self, height: H) -> u64 {
        height.into().div(C::PERIOD::U64)
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

    pub async fn wait_for_execution_block(&self, block_number: U64) {
        loop {
            let latest_finalized_block_number: u64 = self
                .beacon_api_client
                .finality_update()
                .await
                .unwrap()
                .data
                .attested_header
                .execution
                .block_number;

            tracing::debug!(
                %latest_finalized_block_number,
                waiting_for = %block_number,
                "waiting for block"
            );

            if latest_finalized_block_number >= block_number.as_u64() {
                break;
            }

            tracing::debug!("requested height not yet reached");
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    fn make_height(&self, height: impl Into<u64>) -> Height {
        Height::new(0, height.into())
    }

    async fn make_update(
        &self,
        light_client_update: LightClientUpdate<C>,
        trusted_sync_committee: TrustedSyncCommittee<C>,
    ) -> wasm::header::Header<ethereum::header::Header<C>> {
        let execution_block_number = light_client_update.attested_header.execution.block_number;
        let updated_height = self.make_height(execution_block_number);

        let account_update = self
            .provider
            .get_proof(
                self.ibc_handler.address(),
                vec![],
                // Proofs are get from the execution layer, so we use execution height, not beacon slot.
                Some(execution_block_number.into()),
            )
            .await
            .unwrap();

        wasm::header::Header {
            height: updated_height,
            data: ethereum::header::Header {
                consensus_update: light_client_update,
                trusted_sync_committee,
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
        }
    }

    async fn make_finality_update(
        &self,
        finality_update: LightClientFinalityUpdate<C>,
        trusted_sync_committee: TrustedSyncCommittee<C>,
        next_sync_committee: Option<SyncCommittee<C>>,
        next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    ) -> wasm::header::Header<ethereum::header::Header<C>> {
        self.make_update(
            LightClientUpdate {
                attested_header: finality_update.attested_header,
                next_sync_committee,
                next_sync_committee_branch,
                finalized_header: finality_update.finalized_header,
                finality_branch: finality_update.finality_branch,
                sync_aggregate: finality_update.sync_aggregate,
                signature_slot: finality_update.signature_slot,
            },
            trusted_sync_committee,
        )
        .await
    }
}

pub trait IntoEthAbi: Into<Self::EthAbi> {
    type EthAbi;

    fn into_eth_abi(self) -> Self::EthAbi {
        self.into()
    }
}
