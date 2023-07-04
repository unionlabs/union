use bip32::XPrv;
use contracts::glue::{
    GoogleProtobufTimestampData, TendermintTypesBlockIDData, TendermintTypesCommitData,
    TendermintTypesHeaderData, TendermintTypesPartSetHeaderData, TendermintTypesSignedHeaderData,
    TendermintVersionConsensusData, UnionIbcLightclientsCometblsV1HeaderData,
};
use ethers::types::H256;
use futures::{Future, FutureExt};
use ibc_types::{
    core::{
        channel::{
            channel::Channel, msg_channel_open_ack::MsgChannelOpenAck,
            msg_channel_open_confirm::MsgChannelOpenConfirm,
            msg_channel_open_init::MsgChannelOpenInit, msg_channel_open_try::MsgChannelOpenTry,
            msg_recv_packet::MsgRecvPacket,
        },
        client::height::Height,
        commitment::merkle_root::MerkleRoot,
        connection::{
            connection_end::ConnectionEnd, msg_channel_open_ack::MsgConnectionOpenAck,
            msg_channel_open_confirm::MsgConnectionOpenConfirm,
            msg_channel_open_init::MsgConnectionOpenInit,
            msg_channel_open_try::MsgConnectionOpenTry,
        },
    },
    google::protobuf::{any::Any, duration::Duration},
    lightclients::{cometbls, ethereum, tendermint::fraction::Fraction, wasm},
    CosmosAccountId, IntoProto, MsgIntoProto,
};
use num_bigint::BigUint;
use prost::Message;
use protos::{
    cosmos::{
        auth,
        ics23::v1 as ics23_v1,
        staking::{self, v1beta1::BondStatus},
        tx,
    },
    google,
    ibc::core::{
        channel::v1 as channel_v1, client::v1 as client_v1, commitment::v1 as commitment_v1,
        connection::v1 as connection_v1,
    },
    union::prover::api::v1::{union_prover_api_client, ProveRequest},
};
use sha2::Digest;
use tendermint_rpc::{Client, WebSocketClient};
use tokio::task::JoinHandle;

use crate::chain::{evm::Cometbls, Connect, LightClient, StateProof};

/// The 08-wasm light client running on the union chain.
pub struct Ethereum {
    signer: CosmosAccountId,
    pub tm_client: WebSocketClient,
    pub driver_handle: JoinHandle<Result<(), tendermint_rpc::Error>>,
    wasm_code_id: H256,
    pub chain_id: String,
    pub chain_revision: u64,
}

impl Ethereum {
    pub async fn new(signer: XPrv, wasm_code_id: H256) -> Self {
        let (tm_client, driver) =
            WebSocketClient::builder("ws://127.0.0.1:26657/websocket".parse().unwrap())
                .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
                .build()
                .await
                .unwrap();

        let driver_handle = tokio::spawn(async move { driver.run().await });

        let chain_id = tm_client
            .status()
            .await
            .unwrap()
            .node_info
            .network
            .to_string();

        let chain_revision = chain_id.split('-').last().unwrap().parse().unwrap();

        Self {
            signer: CosmosAccountId::new(signer, "union".to_string()),
            tm_client,
            driver_handle,
            wasm_code_id,
            chain_id,
            chain_revision,
        }
    }

    async fn account_info_of_signer(signer: &CosmosAccountId) -> auth::v1beta1::BaseAccount {
        let account = auth::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
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

    pub async fn broadcast_tx_commit(
        &self,
        messages: impl IntoIterator<Item = google::protobuf::Any>,
    ) -> tendermint_rpc::endpoint::broadcast::tx_commit::Response {
        let account = Self::account_info_of_signer(&self.signer).await;

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
            .try_sign(sign_doc.encode_to_vec())
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

    fn make_height(&self, height: u64) -> Height {
        Height {
            revision_number: self.chain_revision,
            revision_height: height,
        }
    }
}

impl LightClient for Ethereum {
    type ClientState = Any<wasm::client_state::ClientState<ethereum::client_state::ClientState>>;
    type ConsensusState =
        Any<wasm::consensus_state::ConsensusState<ethereum::consensus_state::ConsensusState>>;
    type UpdateClientMessage = wasm::header::Header<ethereum::header::Header>;

    fn chain_id(&self) -> impl Future<Output = String> + '_ {
        async move { self.chain_id.clone() }
    }

    fn create_client(
        &self,
        client_state: Self::ClientState,
        consensus_state: Self::ConsensusState,
    ) -> impl futures::Future<Output = String> + '_ {
        async move {
            let msg = google::protobuf::Any {
                type_url: "/ibc.core.client.v1.MsgCreateClient".into(),
                value: client_v1::MsgCreateClient {
                    signer: self.signer.to_string(),
                    client_state: Some(client_state.into_proto()),
                    consensus_state: Some(consensus_state.into_proto()),
                }
                .encode_to_vec(),
            };

            self.broadcast_tx_commit([msg])
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
                .value
        }
    }

    fn update_client(
        &self,
        client_id: String,
        msg: Self::UpdateClientMessage,
    ) -> impl futures::Future<Output = ()> + '_ {
        async move {
            // let mut query_client =
            //     client_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            //         .await
            //         .unwrap();

            // dbg!(&msg.data.consensus_update.finalized_header.beacon.slot);

            // let cs_before: Self::ClientState = query_client
            //     .client_state(QueryClientStateRequest {
            //         client_id: client_id.clone(),
            //     })
            //     .await
            //     .unwrap()
            //     .into_inner()
            //     .client_state
            //     .unwrap()
            //     .try_into()
            //     .unwrap();

            self.broadcast_tx_commit([google::protobuf::Any {
                type_url: "/ibc.core.client.v1.MsgUpdateClient".into(),
                value: client_v1::MsgUpdateClient {
                    client_id: client_id.clone(),
                    client_message: Some(Any(msg).into_proto()),
                    signer: self.signer.to_string(),
                }
                .encode_to_vec(),
            }])
            .await;

            // let cs_after: Self::ClientState = query_client
            //     .client_state(QueryClientStateRequest {
            //         client_id: client_id.clone(),
            //     })
            //     .await
            //     .unwrap()
            //     .into_inner()
            //     .client_state
            //     .unwrap()
            //     .try_into()
            //     .unwrap();

            // dbg!(cs_before);
            // dbg!(cs_after);
        }
    }

    fn consensus_state_proof(
        &self,
        client_id: String,
        counterparty_height: Height,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ConsensusState>> + '_ {
        async move {
            let path = format!(
                "clients/{client_id}/consensusStates/{}-{}",
                counterparty_height.revision_number, counterparty_height.revision_height
            );

            let query_result = self
                .tm_client
                .abci_query(
                    Some("store/ibc/key".to_string()),
                    path,
                    Some(self_height.revision_height.try_into().unwrap()),
                    true,
                )
                .await
                .unwrap();

            StateProof {
                state: google::protobuf::Any::decode(&*query_result.value)
                    .unwrap()
                    .try_into()
                    .unwrap(),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: self.make_height(query_result.height.value()),
            }
        }
    }

    fn client_state_proof(
        &self,
        client_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Self::ClientState>> + '_ {
        async move {
            let path = format!("clients/{client_id}/clientState");

            let query_result = self
                .tm_client
                .abci_query(
                    Some("store/ibc/key".to_string()),
                    path,
                    Some(self_height.revision_height.try_into().unwrap()),
                    true,
                )
                .await
                .unwrap();

            tracing::debug!(
                "Client state serialized {:?}",
                String::from_utf8_lossy(&subtle_encoding::hex::encode(&query_result.value))
            );

            StateProof {
                state: google::protobuf::Any::decode(&*query_result.value)
                    .unwrap()
                    .try_into()
                    .unwrap(),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: self.make_height(query_result.height.value()),
            }
        }
    }

    fn connection_state_proof(
        &self,
        connection_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<ConnectionEnd>> + '_ {
        async move {
            let path = format!("connections/{connection_id}");

            let query_result = self
                .tm_client
                .abci_query(
                    Some("store/ibc/key".to_string()),
                    path,
                    Some(self_height.revision_height.try_into().unwrap()),
                    true,
                )
                .await
                .unwrap();

            let connection_end =
                connection_v1::ConnectionEnd::decode(&*query_result.value).unwrap();

            tracing::debug!("Connection ID: {:?}", connection_id);
            tracing::debug!("Connection End: {:?}", connection_end);

            let proof = StateProof {
                state: connection_end.try_into().unwrap(),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: self.make_height(query_result.height.value()),
            };

            tracing::debug!("Proof height {}", query_result.height.value());
            tracing::debug!(
                "Proof {}",
                String::from_utf8_lossy(&subtle_encoding::hex::encode(proof.proof.clone()))
            );

            proof
        }
    }

    fn channel_state_proof(
        &self,
        channel_id: String,
        port_id: String,
        self_height: Height,
    ) -> impl Future<Output = StateProof<Channel>> + '_ {
        async move {
            let path = format!("channelEnds/ports/{port_id}/channels/{channel_id}");

            let query_result = self
                .tm_client
                .abci_query(
                    Some("store/ibc/key".to_string()),
                    path,
                    Some(self_height.revision_height.try_into().unwrap()),
                    true,
                )
                .await
                .unwrap();

            let connection_end = channel_v1::Channel::decode(&*query_result.value).unwrap();

            tracing::debug!("{:?}", connection_end);

            StateProof {
                state: connection_end.try_into().unwrap(),
                proof: commitment_v1::MerkleProof {
                    proofs: query_result
                        .proof
                        .unwrap()
                        .ops
                        .into_iter()
                        .map(|op| ics23_v1::CommitmentProof::decode(op.data.as_slice()).unwrap())
                        .collect::<Vec<_>>(),
                }
                .encode_to_vec(),
                proof_height: self.make_height(query_result.height.value()),
            }
        }
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

            self.make_height(height)
        }
    }

    fn query_client_state(
        &self,
        client_id: String,
    ) -> impl Future<Output = Self::ClientState> + '_ {
        async move {
            client_v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
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

impl Connect<Cometbls> for Ethereum {
    // fn generate_counterparty_handshake_client_state(
    //     &self,
    //     counterparty_state: <Cometbls as LightClient>::ClientState,
    // ) -> impl Future<Output = Self::HandshakeClientState> + '_ {
    //     async move {
    //         super::msgs::wasm::ClientState {
    //             data: Any(super::msgs::cometbls::ClientState {
    //                 chain_id: todo!(),
    //                 trust_level: todo!(),
    //                 trusting_period: todo!(),
    //                 unbonding_period: todo!(),
    //                 max_clock_drift: todo!(),
    //                 frozen_height: todo!(),
    //             }),
    //             code_id: todo!(),
    //             latest_height: todo!(),
    //         }
    //     }
    // }

    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl futures::Future<Output = (String, Height)> + '_ {
        self.broadcast_tx_commit([google::protobuf::Any {
            type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".to_string(),
            value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
        }])
        .map(|response| {
            (
                response
                    .deliver_tx
                    .events
                    .into_iter()
                    .find(|event| event.kind == "connection_open_init")
                    .unwrap()
                    .attributes
                    .into_iter()
                    .find(|attr| attr.key == "connection_id")
                    .unwrap()
                    .value,
                self.make_height(response.height.value()),
            )
        })
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<<Cometbls as LightClient>::ClientState>,
    ) -> impl futures::Future<Output = (String, Height)> + '_ {
        self.broadcast_tx_commit([google::protobuf::Any {
            type_url: "/ibc.core.connection.v1.MsgConnectionOpenTry".to_string(),
            value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
        }])
        .map(|response| {
            (
                response
                    .deliver_tx
                    .events
                    .into_iter()
                    .find(|event| dbg!(event).kind == "connection_open_try")
                    .unwrap()
                    .attributes
                    .into_iter()
                    .find(|attr| attr.key == "connection_id")
                    .unwrap()
                    .value,
                self.make_height(response.height.value()),
            )
        })
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<<Cometbls as LightClient>::ClientState>,
    ) -> impl futures::Future<Output = Height> + '_ {
        async move {
            self.make_height(
                self.broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.connection.v1.MsgConnectionOpenAck".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await
                .height
                .value(),
            )
        }
    }

    fn connection_open_confirm(
        &self,
        msg: MsgConnectionOpenConfirm,
    ) -> impl futures::Future<Output = Height> + '_ {
        async move {
            self.make_height(
                self.broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.connection.v1.MsgConnectionOpenConfirm".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await
                .height
                .value(),
            )
        }
    }

    fn channel_open_init(
        &self,
        msg: MsgChannelOpenInit,
    ) -> impl futures::Future<Output = (String, Height)> + '_ {
        async move {
            let tx = self
                .broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.channel.v1.MsgChannelOpenInit".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await;
            (
                tx.deliver_tx
                    .events
                    .into_iter()
                    .find(|event| event.kind == "channel_open_init")
                    .unwrap()
                    .attributes
                    .into_iter()
                    .find(|attr| attr.key == "channel_id")
                    .unwrap()
                    .value,
                self.make_height(tx.height.value()),
            )
        }
    }

    fn channel_open_try(
        &self,
        msg: MsgChannelOpenTry,
    ) -> impl futures::Future<Output = (String, Height)> + '_ {
        async move {
            let tx = self
                .broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.channel.v1.MsgChannelOpenTry".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await;
            (
                tx.deliver_tx
                    .events
                    .into_iter()
                    .find(|event| event.kind == "channel_open_try")
                    .unwrap()
                    .attributes
                    .into_iter()
                    .find(|attr| attr.key == "channel_id")
                    .unwrap()
                    .value,
                self.make_height(tx.height.value()),
            )
        }
    }

    fn channel_open_ack(
        &self,
        msg: MsgChannelOpenAck,
    ) -> impl futures::Future<Output = Height> + '_ {
        async move {
            self.make_height(
                self.broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.channel.v1.MsgChannelOpenAck".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await
                .height
                .value(),
            )
        }
    }

    fn channel_open_confirm(
        &self,
        msg: MsgChannelOpenConfirm,
    ) -> impl futures::Future<Output = Height> + '_ {
        async move {
            self.make_height(
                self.broadcast_tx_commit([google::protobuf::Any {
                    type_url: "/ibc.core.channel.v1.MsgChannelOpenConfirm".to_string(),
                    value: msg.into_proto_with_signer(&self.signer).encode_to_vec(),
                }])
                .await
                .height
                .value(),
            )
        }
    }

    fn recv_packet(&self, _msg: MsgRecvPacket) -> impl futures::Future<Output = ()> + '_ {
        async move { todo!() }
    }

    fn generate_counterparty_client_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::ClientState> + '_ {
        async move {
            let params =
                staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
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
                    trusting_period: Duration {
                        seconds: (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                        nanos: (unbonding_period * 85 / 100)
                            .subsec_nanos()
                            .try_into()
                            .unwrap(),
                    },
                    unbonding_period: Duration {
                        seconds: unbonding_period.as_secs().try_into().unwrap(),
                        nanos: unbonding_period.subsec_nanos().try_into().unwrap(),
                    },
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    max_clock_drift: Duration {
                        seconds: 60 * 10,
                        nanos: 0,
                    },
                    frozen_height: Height {
                        revision_number: 0,
                        revision_height: 0,
                    },
                },
                code_id: self.wasm_code_id.to_fixed_bytes().to_vec(),
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

    fn generate_counterparty_consensus_state(
        &self,
        height: Height,
    ) -> impl Future<Output = <Cometbls as LightClient>::ConsensusState> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(tendermint::block::Height::try_from(height.revision_height).unwrap())
                .await
                .unwrap();

            let state = cometbls::consensus_state::ConsensusState {
                root: MerkleRoot {
                    hash: commit.signed_header.header.app_hash.as_bytes().to_vec(),
                },
                next_validators_hash: commit
                    .signed_header
                    .header
                    .next_validators_hash
                    .as_bytes()
                    .to_vec(),
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

    fn update_counterparty_client<'a>(
        &'a self,
        counterparty: &'a Cometbls,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + '_ {
        async move {
            let commit = self
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(update_to.revision_height)
                        .unwrap(),
                )
                .await
                .unwrap();

            tracing::debug!("New block {:?}", commit.signed_header.header.height);

            // todo: add to self
            let mut staking_client =
                staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                    .await
                    .unwrap();

            tracing::debug!("Query validators...");
            let mut validators = staking_client
                .validators(staking::v1beta1::QueryValidatorsRequest {
                    // How to use BondStatus???
                    status: BondStatus::Bonded.as_str_name().to_string(),
                    pagination: None,
                })
                .await
                .unwrap()
                .into_inner()
                .validators;

            // Validators must be sorted to match the root, by token then address
            validators.sort_by(|a, b| {
                let a_tokens = str::parse::<u128>(&a.tokens).unwrap();
                let b_tokens = str::parse::<u128>(&b.tokens).unwrap();
                if a_tokens == b_tokens {
                    let a_key = protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
                        &a.consensus_pubkey.clone().unwrap().value,
                    )
                    .unwrap()
                    .key;
                    let b_key = protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
                        &b.consensus_pubkey.clone().unwrap().value,
                    )
                    .unwrap()
                    .key;
                    // Tendermint address are sha256(pubkey)[0:20]
                    let a_address = sha2::Sha256::new()
                        .chain_update(a_key)
                        .finalize()
                        .into_iter()
                        .take(20)
                        .collect::<Vec<_>>();
                    let b_address = sha2::Sha256::new()
                        .chain_update(b_key)
                        .finalize()
                        .into_iter()
                        .take(20)
                        .collect::<Vec<_>>();
                    a_address.cmp(&b_address)
                } else {
                    a_tokens.cmp(&b_tokens)
                }
            });

            let simple_validators = validators
                .iter()
                .map(|v| {
                    protos::tendermint::types::SimpleValidator {
                        // Couldn't find a less ugly way
                        pub_key: v.consensus_pubkey.as_ref().map(|pk| {
                            protos::tendermint::crypto::PublicKey {
                                sum: Some(protos::tendermint::crypto::public_key::Sum::Bn254(
                                    protos::cosmos::crypto::bn254::PubKey::decode::<&[u8]>(
                                        &pk.value.clone(),
                                    )
                                    .unwrap()
                                    .key,
                                )),
                            }
                        }),
                        // Equivalent of sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction)
                        voting_power: (str::parse::<u128>(&v.tokens).unwrap() / 1_000_000_u128)
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
                    tendermint::block::CommitSig::BlockIdFlagAbsent => {}
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
                        tracing::debug!("Nul flag???");
                    }
                }
            }

            let trusted_commit = Some(protos::union::prover::api::v1::ValidatorSetCommit {
                validators: simple_validators,
                signatures,
                bitmap: bitmap.to_bytes_be(),
            });

            // The untrusted commit is the same as we only deal with adjacent verification for now.
            let untrusted_commit = trusted_commit.clone();

            tracing::debug!("Generate ZKP...");

            const PROVER_ENDPOINT: &str = "https://prover.cryptware.io:443";
            // const PROVER_ENDPOINT: &str = "http://localhost:8080";

            // .http2_keep_alive_interval(std::time::Duration::from_secs(10))
            // .keep_alive_while_idle(true),

            let mut prover_client = union_prover_api_client::UnionProverApiClient::connect(
                tonic::transport::Endpoint::from_static(PROVER_ENDPOINT),
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
                    trusted_commit,
                    untrusted_commit,
                })
                .await
                .unwrap()
                .into_inner();

            let header_timestamp = tendermint_proto::google::protobuf::Timestamp::from(
                commit.signed_header.header.time,
            );
            let client_message = UnionIbcLightclientsCometblsV1HeaderData {
                signed_header: TendermintTypesSignedHeaderData {
                    header: TendermintTypesHeaderData {
                        version: TendermintVersionConsensusData {
                            block: commit.signed_header.header.version.block,
                            app: commit.signed_header.header.version.app,
                        },
                        chain_id: commit.signed_header.header.chain_id.into(),
                        height: commit.signed_header.header.height.into(),
                        time: GoogleProtobufTimestampData {
                            secs: header_timestamp.seconds,
                            nanos: header_timestamp.nanos.into(),
                        },
                        last_block_id: TendermintTypesBlockIDData {
                            hash: commit
                                .signed_header
                                .header
                                .last_block_id
                                .unwrap()
                                .hash
                                .as_bytes()
                                .to_vec()
                                .into(),
                            part_set_header: TendermintTypesPartSetHeaderData {
                                total: commit
                                    .signed_header
                                    .header
                                    .last_block_id
                                    .unwrap()
                                    .part_set_header
                                    .total,
                                hash: commit
                                    .signed_header
                                    .header
                                    .last_block_id
                                    .unwrap()
                                    .part_set_header
                                    .hash
                                    .as_bytes()
                                    .to_vec()
                                    .into(),
                            },
                        },
                        last_commit_hash: commit
                            .signed_header
                            .header
                            .last_commit_hash
                            .unwrap()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        data_hash: commit
                            .signed_header
                            .header
                            .data_hash
                            .unwrap()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        validators_hash: commit
                            .signed_header
                            .header
                            .validators_hash
                            .as_bytes()
                            .to_vec()
                            .into(),
                        next_validators_hash: commit
                            .signed_header
                            .header
                            .next_validators_hash
                            .as_bytes()
                            .to_vec()
                            .into(),
                        consensus_hash: commit
                            .signed_header
                            .header
                            .consensus_hash
                            .as_bytes()
                            .to_vec()
                            .into(),
                        app_hash: commit
                            .signed_header
                            .header
                            .app_hash
                            .as_bytes()
                            .to_vec()
                            .into(),
                        last_results_hash: commit
                            .signed_header
                            .header
                            .last_results_hash
                            .unwrap()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        evidence_hash: commit
                            .signed_header
                            .header
                            .evidence_hash
                            .unwrap()
                            .as_bytes()
                            .to_vec()
                            .into(),
                        proposer_address: commit
                            .signed_header
                            .header
                            .proposer_address
                            .as_bytes()
                            .to_vec()
                            .into(),
                    },
                    commit: TendermintTypesCommitData {
                        height: commit.signed_header.commit.height.into(),
                        round: commit.signed_header.commit.round.into(),
                        block_id: TendermintTypesBlockIDData {
                            hash: commit
                                .signed_header
                                .commit
                                .block_id
                                .hash
                                .as_bytes()
                                .to_vec()
                                .into(),
                            part_set_header: TendermintTypesPartSetHeaderData {
                                total: commit.signed_header.commit.block_id.part_set_header.total,
                                hash: commit
                                    .signed_header
                                    .commit
                                    .block_id
                                    .part_set_header
                                    .hash
                                    .as_bytes()
                                    .to_vec()
                                    .into(),
                            },
                        },
                        // NOTE: We don't need the signatures are they are part of the ZKP
                        signatures: vec![],
                    },
                },
                untrusted_validator_set_root: prove_res.untrusted_validator_set_root.into(),
                trusted_height: update_from.into(),
                zero_knowledge_proof: prove_res.proof.unwrap().evm_proof.into(),
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
