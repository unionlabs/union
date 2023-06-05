use std::{
    fmt::Debug,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use contracts::{
    glue::{
        GoogleProtobufDurationData, GoogleProtobufTimestampData, IbcCoreCommitmentV1MerkleRootData,
        OptimizedConsensusState, UnionIbcLightclientsCometblsV1ClientStateData,
        UnionIbcLightclientsCometblsV1ConsensusStateData,
        UnionIbcLightclientsCometblsV1FractionData, UnionIbcLightclientsCometblsV1HeaderData,
    },
    ibc_handler::{
        GeneratedConnectionIdentifierFilter, IBCHandler, IBCHandlerEvents,
        IbcCoreChannelV1ChannelData, IbcCoreChannelV1CounterpartyData, IbcCoreChannelV1PacketData,
        IbcCoreCommitmentV1MerklePrefixData, IbcCoreConnectionV1ConnectionEndData,
        IbcCoreConnectionV1CounterpartyData, IbcCoreConnectionV1VersionData, MsgChannelOpenAck,
        MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry, MsgConnectionOpenAck,
        MsgConnectionOpenConfirm, MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient,
        MsgPacketRecv, MsgUpdateClient,
    },
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::{
    abi::{AbiDecode, AbiEncode},
    prelude::{decode_logs, k256::ecdsa, SignerMiddleware},
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer, Wallet},
    types::{H160, U256},
    utils::keccak256,
};
use futures::Future;
use lodestar_rpc::types::LightClientBootstrapResponse;
use prost::Message;
use protos::{
    ibc::lightclients::wasm::{self, v1::QueryCodeIdsRequest},
    union::ibc::lightclients::ethereum::v1::{Proof, StorageProof},
};
use strum::ParseError;

use crate::{
    chain::cosmos::IntoProto, cosmos_to_eth::COMETBLS_CLIENT_TYPE, ETH_BEACON_RPC_API, ETH_RPC_API,
};

use super::{cosmos::Ethereum, Connect, LightClient};

/// The solidity light client, tracking the state of the 08-wasm light client on union.
// TODO(benluelo): Generic over middleware?
pub struct Cometbls {
    ibc_handler: IBCHandler<SignerMiddleware<Provider<Http>, Wallet<ecdsa::SigningKey>>>,
    pub provider: Provider<Http>,
    cometbls_client_address: H160,
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
    type ClientState = super::msgs::wasm::ClientState<super::msgs::cometbls::ClientState>;
    type ConsensusState = super::msgs::wasm::ConsensusState<super::msgs::cometbls::ConsensusState>;
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
                .create_client(MsgCreateClient {
                    client_type: COMETBLS_CLIENT_TYPE.to_string(),
                    client_state_bytes: encode_dynamic_singleton_tuple(client_state.into_eth_abi())
                        .into(),
                    consensus_state_bytes: encode_dynamic_singleton_tuple(
                        consensus_state.into_eth_abi(),
                    )
                    .into(),
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

            dbg!(consensus_state.to_string());

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
                .update_client(MsgUpdateClient {
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
        counterparty_height: super::msgs::Height,
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<Self::ConsensusState>> + '_ {
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

            let optimized_consensus_state =
                OptimizedConsensusState::decode(&consensus_state_bytes).unwrap();

            tracing::info!(?optimized_consensus_state);

            let consensus_state = super::msgs::cometbls::ConsensusState {
                timestamp: super::msgs::Timestamp {
                    seconds: optimized_consensus_state.timestamp.try_into().unwrap(),
                    nanos: 0,
                },
                root: super::msgs::MerkleRoot {
                    hash: optimized_consensus_state.root.into(),
                },
                next_validators_hash: optimized_consensus_state.next_validators_hash.into(),
            };

            // {
            //     let location = <[u8; 32]>::try_from(U256::from(12).encode())
            //         .unwrap()
            //         .into();
            //     // let encoded_differently = {
            //     //     let mut slice = [0u8; 32];
            //     //     U256::from(12).to_big_endian(&mut slice);
            //     //     keccak256(slice).into()
            //     // };

            //     // assert_eq!(location, encoded_differently);

            //     tracing::info!(?location, "sequence");

            //     let proof = self
            //         .provider
            //         // eth_getProof
            //         .get_proof(
            //             self.ibc_handler.address(),
            //             vec![location],
            //             Some(self_height.revision_height.into()),
            //         )
            //         .await
            //         .unwrap();

            //     tracing::info!(?proof, "sequence");

            //     for i in 0..=12 {
            //         dbg!(i);
            //         let storage = self
            //             .provider
            //             // eth_getProof
            //             .get_storage_at(
            //                 self.ibc_handler.address(),
            //                 {
            //                     let mut slice = [0u8; 32];
            //                     U256::from(i).to_big_endian(&mut slice);
            //                     slice.into()
            //                 },
            //                 None,
            //             )
            //             .await
            //             .unwrap();

            //         tracing::info!(?storage, "sequence");
            //     }
            // };

            // panic!();

            self.get_proof(
                format!(
                    "clients/{client_id}/consensusStates/{}-{}",
                    counterparty_height.revision_number, counterparty_height.revision_height
                ),
                self_height,
                consensus_state.clone(),
                // AbiEncode::encode,
                |_| optimized_consensus_state.encode(),
            )
            .await
        }
    }

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<Self::ClientState>> + '_ {
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

            let cometbls_client_state = decode_dynamic_singleton_tuple::<
                UnionIbcLightclientsCometblsV1ClientStateData,
            >(&client_state_bytes);

            tracing::info!(?cometbls_client_state);

            // tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            let block_number = self.provider.get_block_number().await.unwrap();
            tracing::info!(?block_number);

            self.get_proof(
                format!("clients/{client_id}/clientState"),
                self_height,
                cometbls_client_state.into(),
                |x: Self::ClientState| encode_dynamic_singleton_tuple(x.into_eth_abi()),
            )
            .await
        }
    }

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: super::msgs::Height,
    ) -> impl Future<Output = super::msgs::StateProof<super::msgs::ConnectionEnd>> + '_ {
        async move {
            tracing::info!(?self_height);
            self.wait_for_block(self_height).await;

            let (connection_end, is_found): (IbcCoreConnectionV1ConnectionEndData, bool) = self
                .ibc_handler
                .get_connection(connection_id.clone())
                .block(self_height.revision_height)
                .await
                .unwrap();

            tracing::info!(?connection_end);

            assert!(is_found);

            let canonical_connection_end: super::msgs::ConnectionEnd =
                connection_end.try_into().unwrap();

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

    fn query_latest_height(&self) -> impl Future<Output = super::msgs::Height> + '_ {
        async move {
            // const API: &str = "eth/v2/debug/beacon/states";

            // let height = reqwest::Client::new()
            //     .get(format!("{ETH_BEACON_RPC_API}/{API}/finalized"))
            //     .send()
            //     .await
            //     .unwrap()
            //     .json::<serde_json::Value>()
            //     .await
            //     .unwrap()
            //     .get("data")
            //     .unwrap()
            //     .get("slot")
            //     .unwrap()
            //     .as_str()
            //     .unwrap()
            //     .parse::<u64>()
            //     .unwrap();

            self.provider
                .get_block_number()
                .await
                .map(|height| super::msgs::Height {
                    revision_number: 0,
                    revision_height: height.as_u64(),
                })
                .unwrap()
        }
    }
}

impl Connect<Ethereum> for Cometbls {
    type HandshakeClientState = <Ethereum as LightClient>::ClientState;

    fn generate_counterparty_handshake_client_state(
        &self,
        counterparty_state: <Ethereum as LightClient>::ClientState,
    ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
        async move { todo!() }
    }

    fn connection_open_init(
        &self,
        msg: super::msgs::connection::MsgConnectionOpenInit,
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
                IBCHandlerEvents::GeneratedConnectionIdentifierFilter(
                    GeneratedConnectionIdentifierFilter(connection_id),
                ) => {
                    tracing::info!(connection_id, "created connection");

                    Some(connection_id)
                }
                _ => None,
            })
            .unwrap()
        }
    }

    fn connection_open_try(
        &self,
        msg: super::msgs::connection::MsgConnectionOpenTry<Self::HandshakeClientState>,
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
        msg: super::msgs::connection::MsgConnectionOpenAck<<Ethereum as LightClient>::ClientState>,
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
        msg: super::msgs::connection::MsgConnectionOpenConfirm,
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

    fn channel_open_init(
        &self,
        msg: super::msgs::channel::MsgChannelOpenInit,
    ) -> impl Future<Output = String> + '_ {
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

    fn channel_open_try(
        &self,
        msg: super::msgs::channel::MsgChannelOpenTry,
    ) -> impl Future<Output = ()> + '_ {
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

    fn channel_open_ack(
        &self,
        msg: super::msgs::channel::MsgChannelOpenAck,
    ) -> impl Future<Output = ()> + '_ {
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

    fn channel_open_confirm(
        &self,
        msg: super::msgs::channel::MsgChannelOpenConfirm,
    ) -> impl Future<Output = ()> + '_ {
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

    fn recv_packet(
        &self,
        packet: super::msgs::channel::MsgRecvPacket,
    ) -> impl Future<Output = ()> + '_ {
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
        height: super::msgs::Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ClientState> + '_ {
        async move {
            let genesis = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
                .get_genesis()
                .await
                .unwrap()
                .data;

            super::msgs::wasm::ClientState {
                data: super::msgs::ethereum::ClientState {
                    genesis_validators_root: genesis.genesis_validators_root.as_bytes().to_vec(),
                    genesis_time: genesis.genesis_time.0,
                    fork_parameters: super::msgs::ethereum::ForkParameters {
                        genesis_fork_version: vec![0, 0, 0, 1],
                        genesis_slot: 0,
                        altair: super::msgs::ethereum::Fork {
                            version: vec![1, 0, 0, 1],
                            epoch: 0,
                        },
                        bellatrix: super::msgs::ethereum::Fork {
                            version: vec![2, 0, 0, 1],
                            epoch: 0,
                        },
                        capella: super::msgs::ethereum::Fork {
                            version: vec![3, 0, 0, 1],
                            epoch: 0,
                        },
                        eip4844: super::msgs::ethereum::Fork {
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
                    trust_level: super::msgs::Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    frozen_height: super::msgs::Height {
                        revision_number: 0,
                        revision_height: 0,
                    },
                    counterparty_commitment_slot: 0,
                },
                code_id: ethers::utils::hex::decode(dbg!(
                    &wasm::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                        .await
                        .unwrap()
                        .code_ids(QueryCodeIdsRequest { pagination: None })
                        .await
                        .unwrap()
                        .into_inner()
                        .code_ids
                        .first()
                        .unwrap()[1..]
                ))
                .unwrap(),
                latest_height: height,
            }
        }
    }

    fn generate_counterparty_consensus_state(
        &self,
        height: super::msgs::Height,
    ) -> impl Future<Output = <Ethereum as LightClient>::ConsensusState> + '_ {
        async move {
            let trusted_header = lodestar_rpc::client::RPCClient::new(ETH_BEACON_RPC_API)
                .get_beacon_header_by_slot(dbg!(ethereum_consensus::types::U64(
                    height.revision_height
                )))
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

            super::msgs::wasm::ConsensusState {
                data: super::msgs::ethereum::ConsensusState {
                    slot: bootstrap.header.beacon.slot.0,
                    storage_root: vec![1, 2, 3],
                    timestamp: bootstrap.header.execution.timestamp.0,
                    current_sync_committee: bootstrap
                        .current_sync_committee
                        .aggregate_pubkey
                        .to_vec(),
                    next_sync_committee: vec![],
                },
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        }
    }

    fn generate_counterparty_update_client_message(
        &self,
    ) -> impl Future<Output = <Ethereum as LightClient>::UpdateClientMessage> + '_ {
        async move { todo!() }
    }
}

impl Cometbls {
    pub async fn new(cometbls_client_address: H160, ibc_handler_address: H160) -> Self {
        let provider = Provider::<Http>::try_from(ETH_RPC_API).unwrap();
        let chain_id = provider.get_chainid().await.unwrap();
        let wallet = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id.as_u64());

        let signer_middleware = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

        let ibc_handler =
            contracts::ibc_handler::IBCHandler::new(ibc_handler_address, signer_middleware);

        Self {
            ibc_handler,
            provider,
            cometbls_client_address,
        }
    }

    async fn get_proof<S: Clone + Debug>(
        &self,
        path: String,
        height: super::msgs::Height,
        state: S,
        encode: impl FnOnce(S) -> Vec<u8>,
    ) -> super::msgs::StateProof<S> {
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

        super::msgs::StateProof {
            state,
            proof: StorageProof {
                proof: [Proof {
                    key: proof.key.to_string(),
                    value: proof.value.to_string(),
                    proof: proof
                        .proof
                        .into_iter()
                        .map(|bytes| bytes.to_string())
                        .collect(),
                }]
                .to_vec(),
            }
            .encode_to_vec(),
            proof_height: height,
        }
    }

    async fn wait_for_block(&self, requested_height: super::msgs::Height) {
        loop {
            let current_block = self.provider.get_block_number().await.unwrap();

            tracing::debug!(?current_block, waiting_for = ?requested_height, "waiting for block");

            if current_block.0[0] >= requested_height.revision_height {
                break;
            } else {
                tracing::debug!("requested height not yet reached");
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
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

impl From<super::msgs::connection::MsgConnectionOpenInit> for MsgConnectionOpenInit {
    fn from(msg: super::msgs::connection::MsgConnectionOpenInit) -> MsgConnectionOpenInit {
        MsgConnectionOpenInit {
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

impl<ClientState> From<super::msgs::connection::MsgConnectionOpenTry<ClientState>>
    for MsgConnectionOpenTry
{
    fn from(
        msg: super::msgs::connection::MsgConnectionOpenTry<ClientState>,
    ) -> MsgConnectionOpenTry {
        MsgConnectionOpenTry {
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

impl<ClientState> From<super::msgs::connection::MsgConnectionOpenAck<ClientState>>
    for MsgConnectionOpenAck
{
    fn from(
        msg: super::msgs::connection::MsgConnectionOpenAck<ClientState>,
    ) -> MsgConnectionOpenAck {
        MsgConnectionOpenAck {
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

impl From<super::msgs::connection::MsgConnectionOpenConfirm> for MsgConnectionOpenConfirm {
    fn from(msg: super::msgs::connection::MsgConnectionOpenConfirm) -> MsgConnectionOpenConfirm {
        MsgConnectionOpenConfirm {
            connection_id: msg.connection_id,
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<super::msgs::channel::MsgChannelOpenInit> for MsgChannelOpenInit {
    fn from(msg: super::msgs::channel::MsgChannelOpenInit) -> MsgChannelOpenInit {
        MsgChannelOpenInit {
            port_id: msg.port_id,
            channel: msg.channel.into(),
        }
    }
}

impl From<super::msgs::channel::MsgChannelOpenTry> for MsgChannelOpenTry {
    fn from(msg: super::msgs::channel::MsgChannelOpenTry) -> MsgChannelOpenTry {
        MsgChannelOpenTry {
            port_id: msg.port_id,
            channel: msg.channel.into(),
            counterparty_version: msg.counterparty_version,
            proof_init: msg.proof_init.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<super::msgs::channel::MsgChannelOpenAck> for MsgChannelOpenAck {
    fn from(msg: super::msgs::channel::MsgChannelOpenAck) -> MsgChannelOpenAck {
        MsgChannelOpenAck {
            port_id: msg.port_id,
            channel_id: msg.channel_id,
            counterparty_version: msg.counterparty_version,
            counterparty_channel_id: msg.counterparty_channel_id,
            proof_try: msg.proof_try.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<super::msgs::channel::MsgChannelOpenConfirm> for MsgChannelOpenConfirm {
    fn from(msg: super::msgs::channel::MsgChannelOpenConfirm) -> MsgChannelOpenConfirm {
        MsgChannelOpenConfirm {
            port_id: msg.port_id,
            channel_id: msg.channel_id,
            proof_ack: msg.proof_ack.into(),
            proof_height: msg.proof_height.into(),
        }
    }
}

impl From<super::msgs::Height> for IbcCoreClientV1HeightData {
    fn from(value: super::msgs::Height) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<IbcCoreClientV1HeightData> for super::msgs::Height {
    fn from(value: IbcCoreClientV1HeightData) -> Self {
        Self {
            revision_number: value.revision_number,
            revision_height: value.revision_height,
        }
    }
}

impl From<super::msgs::connection::Counterparty> for IbcCoreConnectionV1CounterpartyData {
    fn from(value: super::msgs::connection::Counterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}

impl From<IbcCoreConnectionV1CounterpartyData> for super::msgs::connection::Counterparty {
    fn from(value: IbcCoreConnectionV1CounterpartyData) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}

impl From<super::msgs::connection::Version> for IbcCoreConnectionV1VersionData {
    fn from(value: super::msgs::connection::Version) -> Self {
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

impl TryFrom<IbcCoreConnectionV1VersionData> for super::msgs::connection::Version {
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

impl From<super::msgs::MerklePrefix> for IbcCoreCommitmentV1MerklePrefixData {
    fn from(value: super::msgs::MerklePrefix) -> Self {
        Self {
            key_prefix: value.key_prefix.into(),
        }
    }
}

impl From<IbcCoreCommitmentV1MerklePrefixData> for super::msgs::MerklePrefix {
    fn from(value: IbcCoreCommitmentV1MerklePrefixData) -> Self {
        Self {
            key_prefix: value.key_prefix.to_vec(),
        }
    }
}

impl From<super::msgs::channel::MsgRecvPacket> for MsgPacketRecv {
    fn from(value: super::msgs::channel::MsgRecvPacket) -> Self {
        Self {
            packet: value.packet.into(),
            proof: value.proof_commitment.into(),
            proof_height: value.proof_height.into(),
        }
    }
}

impl From<super::msgs::channel::Packet> for IbcCoreChannelV1PacketData {
    fn from(value: super::msgs::channel::Packet) -> Self {
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

impl From<super::msgs::channel::Channel> for IbcCoreChannelV1ChannelData {
    fn from(value: super::msgs::channel::Channel) -> Self {
        Self {
            state: value.state as u8,
            ordering: value.ordering as u8,
            counterparty: value.counterparty.into(),
            connection_hops: value.connection_hops,
            version: value.version,
        }
    }
}

impl From<super::msgs::channel::Counterparty> for IbcCoreChannelV1CounterpartyData {
    fn from(value: super::msgs::channel::Counterparty) -> Self {
        Self {
            port_id: value.port_id,
            channel_id: value.channel_id,
        }
    }
}

#[derive(Debug)]
pub enum TryFromConnnectionEndError {
    ParseError(ParseError),
    UnknownEnumVariant(super::msgs::UnknownEnumVariant<u8>),
}

impl TryFrom<IbcCoreConnectionV1ConnectionEndData> for super::msgs::ConnectionEnd {
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

impl From<super::msgs::cometbls::ClientState> for UnionIbcLightclientsCometblsV1ClientStateData {
    fn from(value: super::msgs::cometbls::ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value.trusting_period.into(),
            unbonding_period: value.unbonding_period.into(),
            max_clock_drift: value.max_clock_drift.into(),
            frozen_height: value.frozen_height.into(),
            latest_height: value.latest_height.into(),
        }
    }
}

impl From<UnionIbcLightclientsCometblsV1ClientStateData> for super::msgs::cometbls::ClientState {
    fn from(value: UnionIbcLightclientsCometblsV1ClientStateData) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value.trusting_period.into(),
            unbonding_period: value.unbonding_period.into(),
            max_clock_drift: value.max_clock_drift.into(),
            frozen_height: value.frozen_height.into(),
            latest_height: value.latest_height.into(),
        }
    }
}

impl From<super::msgs::cometbls::ConsensusState>
    for UnionIbcLightclientsCometblsV1ConsensusStateData
{
    fn from(value: super::msgs::cometbls::ConsensusState) -> Self {
        Self {
            timestamp: value.timestamp.into(),
            root: value.root.into(),
            next_validators_hash: value.next_validators_hash.into(),
        }
    }
}

impl From<super::msgs::Timestamp> for GoogleProtobufTimestampData {
    fn from(value: super::msgs::Timestamp) -> Self {
        Self {
            secs: value.seconds,
            // REVIEW(benluelo): Is this conversion *actually* fallible?
            nanos: value.nanos.try_into().unwrap(),
        }
    }
}

impl From<super::msgs::Duration> for GoogleProtobufDurationData {
    fn from(value: super::msgs::Duration) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<GoogleProtobufDurationData> for super::msgs::Duration {
    fn from(value: GoogleProtobufDurationData) -> Self {
        Self {
            seconds: value.seconds,
            nanos: value.nanos,
        }
    }
}

impl From<super::msgs::MerkleRoot> for IbcCoreCommitmentV1MerkleRootData {
    fn from(value: super::msgs::MerkleRoot) -> Self {
        Self {
            hash: value.hash.into(),
        }
    }
}

impl From<super::msgs::Fraction> for UnionIbcLightclientsCometblsV1FractionData {
    fn from(value: super::msgs::Fraction) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}

impl From<UnionIbcLightclientsCometblsV1FractionData> for super::msgs::Fraction {
    fn from(value: UnionIbcLightclientsCometblsV1FractionData) -> Self {
        Self {
            numerator: value.numerator,
            denominator: value.denominator,
        }
    }
}
