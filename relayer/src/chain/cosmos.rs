use std::marker::PhantomData;

use clap::Args;
use contracts::devnet_ownable_ibc_handler::GetExpectedTimePerBlockReturn;
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
use tendermint_proto::types::ValidatorSet;
use tendermint_rpc::{
    event::EventData, query::EventType, Client, SubscriptionClient, WebSocketClient,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use unionlabs::{
    bounded_int::BoundedI64,
    cosmos::{staking::query_validators_response::QueryValidatorsResponse, ics23::{proof_spec::ProofSpec, leaf_op::LeafOp, inner_spec::InnerSpec}},
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
    CosmosAccountId, MsgIntoProto, TryFromProto,
};

use super::{events::TryFromTendermintEventError, union::Union};
use crate::{
    chain::{
        dumper::Dumper,
        events::{
            ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck,
            ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, SendPacket, UpdateClient,
        },
        proof::{
            ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
            ConnectionPath, IbcPath, StateProof,
        },
        union::{
            get_event_from_tx_response, tendermint_hash_to_h256, tendermint_height_to_bounded_i64,
            Tendermint,
        },
        Chain, ChainConnection, ClientStateOf, Connect, CreateClient, IbcStateRead, LightClient,
    },
    config::UnionChainConfig,
};

/// The light client on Tendermint based Cosmos chains, that track CometBLS based chains
pub struct Cometbls {
    chain: <Self as LightClient>::HostChain,
    dumper: Dumper,
}

/// Cosmos chain
#[derive(Debug, Clone)]
pub struct Cosmos {
    pub signer: CosmosAccountId,
    tm_client: WebSocketClient,
    chain_id: String,
    chain_revision: u64,
    wasm_code_id: H256,
    prover_endpoint: String,
    grpc_url: String,
    dump_path: String,
}

impl Cosmos {   
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
            // TODO(aeryz): this prefix should be configurable
            signer: CosmosAccountId::new(config.signer.value(), "wasm".to_string()),
            wasm_code_id: config.wasm_code_id,
            tm_client,
            chain_id,
            chain_revision,
            prover_endpoint: config.prover_endpoint,
            dump_path: config.dump_path,
            grpc_url: config.grpc_url,
        }
    }

}

impl ChainConnection<Union> for Cosmos {
    type LightClient = Cometbls;

    fn light_client(&self) -> Self::LightClient {
        Cometbls {
            chain: self.clone(),
            dumper: Dumper::new(self.dump_path.clone()),
        }
    }
}

impl CreateClient<Cometbls> for Cosmos {
    fn create_client(
        &self,
        _: <Cometbls as LightClient>::Config,
        counterparty_chain: <Cometbls as LightClient>::CounterpartyChain,
    ) -> impl Future<Output = (String, Cometbls)> + '_ {
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
                Cometbls {
                    chain: self.clone(),
                    dumper: Dumper::new(self.dump_path.clone()),
                },
            )
        }
    }
}

impl LightClient for Cometbls {
    type UpdateClientMessage = wasm::header::Header<cometbls::header::Header>;

    type IbcStateRead = CosmosStateRead;

    type HostChain = Cosmos;

    type CounterpartyChain = Union;

    type Config = ();

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

impl Cometbls {
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

impl Connect<Tendermint> for Cometbls {
    fn connection_open_init(
        &self,
        msg: MsgConnectionOpenInit,
    ) -> impl Future<Output = (Height, ConnectionOpenInit)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn connection_open_try(
        &self,
        msg: MsgConnectionOpenTry<ClientStateOf<<Tendermint as LightClient>::CounterpartyChain>>,
    ) -> impl Future<Output = (Height, ConnectionOpenTry)> + '_ {
        self.send_msg_and_read_event(msg)
    }

    fn connection_open_ack(
        &self,
        msg: MsgConnectionOpenAck<ClientStateOf<<Tendermint as LightClient>::CounterpartyChain>>,
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

    fn recv_packet(&self, msg: MsgRecvPacket) -> impl Future<Output = ()> + Send + '_ {
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
        counterparty: &'a Tendermint,
        counterparty_client_id: String,
        update_from: Height,
        update_to: Height,
    ) -> impl Future<Output = Height> + 'a {
        async move {
            let trusted_commit = self
                .chain
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(update_from.revision_height + 1)
                        .unwrap(),
                )
                .await
                .unwrap();

            self.dumper.dump(
                format!("commit-{:06}", trusted_commit.signed_header.header.height.value()),
                &trusted_commit,
            );

            let trusted_valset = {
                let validators_response = self
                    .chain
                    .tm_client
                    .validators(
                        TryInto::<tendermint::block::Height>::try_into(update_from.revision_height + 1)
                            .unwrap(),
                        tendermint_rpc::Paging::All,
                    )
                    .await
                    .unwrap();

                let mut proposer = None;
                for val in &validators_response.validators {
                    if val.address == trusted_commit.signed_header.header.proposer_address {
                        proposer = Some(val.clone());
                        break;
                    }
                }

                let total_voting_power = validators_response
                    .validators
                    .iter()
                    .map(|v| v.power.value())
                    .sum::<u64>()
                    .try_into()
                    .unwrap();

                let proposer = proposer.unwrap();

                unionlabs::tendermint::types::validator_set::ValidatorSet {
                    validators: validators_response
                        .validators
                        .into_iter()
                        .map(|val| unionlabs::tendermint::types::validator::Validator {
                            address: val.address.try_into().unwrap(),
                            pub_key: if let tendermint::PublicKey::Ed25519(pkey) = val.pub_key {
                                PublicKey::Ed25519(pkey.as_bytes().to_vec())
                            } else {
                                panic!("validator pubkey cannnot be other than ed25519")
                            },
                            voting_power: val.power.into(),
                            proposer_priority: val.proposer_priority.into(),
                        })
                        .collect(),
                    proposer: unionlabs::tendermint::types::validator::Validator {
                        address: proposer.address.try_into().unwrap(),
                        pub_key: if let tendermint::PublicKey::Ed25519(pkey) = proposer.pub_key {
                            PublicKey::Ed25519(pkey.as_bytes().to_vec())
                        } else {
                            panic!("validator pubkey cannnot be other than ed25519")
                        },
                        voting_power: proposer.power.into(),
                        proposer_priority: proposer.proposer_priority.into(),
                    },
                    total_voting_power,
                }
            };

            let commit = self
                .chain
                .tm_client
                .commit(
                    TryInto::<tendermint::block::Height>::try_into(update_to.revision_height)
                        .unwrap(),
                )
                .await
                .unwrap();
            

            let untrusted_valset = {
                let validators_response = self
                    .chain
                    .tm_client
                    .validators(
                        TryInto::<tendermint::block::Height>::try_into(update_to.revision_height)
                            .unwrap(),
                        tendermint_rpc::Paging::All,
                    )
                    .await
                    .unwrap();

                let mut proposer = None;
                for val in &validators_response.validators {
                    if val.address == commit.signed_header.header.proposer_address {
                        proposer = Some(val.clone());
                        break;
                    }
                }

                let total_voting_power = validators_response
                    .validators
                    .iter()
                    .map(|v| v.power.value())
                    .sum::<u64>()
                    .try_into()
                    .unwrap();

                let proposer = proposer.unwrap();

                unionlabs::tendermint::types::validator_set::ValidatorSet {
                    validators: validators_response
                        .validators
                        .into_iter()
                        .map(|val| unionlabs::tendermint::types::validator::Validator {
                            address: val.address.try_into().unwrap(),
                            pub_key: if let tendermint::PublicKey::Ed25519(pkey) = val.pub_key {
                                PublicKey::Ed25519(pkey.as_bytes().to_vec())
                            } else {
                                panic!("validator pubkey cannnot be other than ed25519")
                            },
                            voting_power: val.power.into(),
                            proposer_priority: val.proposer_priority.into(),
                        })
                        .collect(),
                    proposer: unionlabs::tendermint::types::validator::Validator {
                        address: proposer.address.try_into().unwrap(),
                        pub_key: if let tendermint::PublicKey::Ed25519(pkey) = proposer.pub_key {
                            PublicKey::Ed25519(pkey.as_bytes().to_vec())
                        } else {
                            panic!("validator pubkey cannnot be other than ed25519")
                        },
                        voting_power: proposer.power.into(),
                        proposer_priority: proposer.proposer_priority.into(),
                    },
                    total_voting_power,
                }
            };

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
                    signatures: commit
                        .signed_header
                        .commit
                        .signatures
                        .into_iter()
                        .filter(|sig| !sig.is_absent())
                        .map(|sig| {
                            let (validator_address, timestamp, signature) = match sig {
                                tendermint::block::CommitSig::BlockIdFlagCommit { validator_address, timestamp, signature } => (validator_address, timestamp, signature),
                                tendermint::block::CommitSig::BlockIdFlagNil { validator_address, timestamp, signature } => (validator_address, timestamp, signature),
                                _ => panic!("Already filtered, impossible.")
                            };

                                unionlabs::tendermint::types::commit_sig::CommitSig {
                            block_id_flag:
                                unionlabs::tendermint::types::block_id_flag::BlockIdFlag::Commit,
                            validator_address: 
                                validator_address
                                .as_bytes()
                                .try_into()
                                .unwrap(),
                            timestamp: {
                        let timestamp: tendermint_proto::google::protobuf::Timestamp = timestamp.into();
                                Timestamp {
                                    seconds: timestamp.seconds.try_into().unwrap(),
                                    nanos: timestamp.nanos.try_into().unwrap(),
                                }
                            },
                            signature: signature.unwrap().into_bytes(),
                                    }
                        })
                        .collect(),
                },
            };

            let client_message = unionlabs::ibc::lightclients::tendermint::header::Header {
                signed_header,
                validator_set: untrusted_valset,
                trusted_height: Height {
                    revision_number: update_from.revision_number,
                    revision_height: update_from.revision_height,
                },
                trusted_validators: trusted_valset,
            };

            tracing::debug!("Client message {:?}", client_message);

            tracing::debug!("Updating client...");

            counterparty
                .update_client(counterparty_client_id, client_message)
                .await;

            tracing::debug!("Updated client.");

            update_to
        }
    }
}

impl Chain for Cosmos {
    type SelfClientState = Any<unionlabs::ibc::lightclients::tendermint::client_state::ClientState>;

    type SelfConsensusState =
        Any<unionlabs::ibc::lightclients::tendermint::consensus_state::ConsensusState>;

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

            Any(
                unionlabs::ibc::lightclients::tendermint::client_state::ClientState {
                    chain_id: self.chain_id().await,
                    // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
                    trust_level: Fraction {
                        numerator: 1,
                        denominator: 3,
                    },
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    trusting_period: Duration {
                        seconds: ((unbonding_period * 85 / 100).as_secs() as i64)
                            .try_into()
                            .unwrap(),
                        nanos: ((unbonding_period * 85 / 100).subsec_nanos() as i32)
                            .try_into()
                            .unwrap(),
                    },
                    unbonding_period: Duration {
                        seconds: (unbonding_period.as_secs() as i64).try_into().unwrap(),
                        nanos: (unbonding_period.subsec_nanos() as i32).try_into().unwrap(),
                    },
                    // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                    max_clock_drift: Duration {
                        seconds: (60 * 10).try_into().unwrap(),
                        nanos: 0.try_into().unwrap(),
                    },
                    frozen_height: None,
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
                    proof_specs: vec![
                        ProofSpec {
                        leaf_spec: LeafOp { 
                            hash: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256, 
                            prehash_key: unionlabs::cosmos::ics23::hash_op::HashOp::NoHash, 
                            prehash_value: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256, 
                            length: unionlabs::cosmos::ics23::length_op::LengthOp::VarProto, 
                            prefix: vec![0]
                        },
                        inner_spec: InnerSpec { 
                            child_order: vec![0, 1], 
                            child_size: 33, 
                            min_prefix_length: 4, 
                            max_prefix_length: 12, 
                            empty_child: vec![], 
                            hash: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256
                        },
                        max_depth: 0,
                        min_depth: 0,
                    },
                        ProofSpec {
                        leaf_spec: LeafOp { 
                            hash: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256, 
                            prehash_key: unionlabs::cosmos::ics23::hash_op::HashOp::NoHash, 
                            prehash_value: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256, 
                            length: unionlabs::cosmos::ics23::length_op::LengthOp::VarProto, 
                            prefix: vec![0]
                        },
                        inner_spec: InnerSpec { 
                            child_order: vec![0, 1], 
                            child_size: 32, 
                            min_prefix_length: 1, 
                            max_prefix_length: 1, 
                            empty_child: vec![], 
                            hash: unionlabs::cosmos::ics23::hash_op::HashOp::Sha256
                        },
                        max_depth: 0,
                        min_depth: 0,
                    }
                    ],
                    upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
                    allow_update_after_expiry: false,
                    allow_update_after_misbehavior: false,
                },
            )
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

            Any(
                unionlabs::ibc::lightclients::tendermint::consensus_state::ConsensusState {
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
                        let timestamp: tendermint_proto::google::protobuf::Timestamp =
                            commit.signed_header.header.time.into();
                        Timestamp {
                            seconds: timestamp.seconds.try_into().unwrap(),
                            nanos: timestamp.nanos.try_into().unwrap(),
                        }
                    },
                },
            )
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
                                    sequence: send_packet.packet_sequence.parse().unwrap(),
                                    source_port: send_packet.packet_src_port,
                                    source_channel: send_packet.packet_src_channel,
                                    destination_port: send_packet.packet_dst_port,
                                    destination_channel: send_packet.packet_dst_channel,
                                    data: hex::decode(send_packet.packet_data_hex).unwrap(),
                                    timeout_height: send_packet
                                        .packet_timeout_height
                                        .parse()
                                        .unwrap(),
                                    timeout_timestamp: send_packet
                                        .packet_timeout_timestamp
                                        .parse()
                                        .unwrap(),
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

            let (_missed_events_tx, missed_events_rx) = tokio::sync::mpsc::unbounded_channel();

            UnboundedReceiverStream::new(missed_events_rx)
                .chain(UnboundedReceiverStream::new(events_from_now_rx))
        }
    }
}

trait AbciStateRead: IbcPath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls>;
}

impl AbciStateRead for ClientStatePath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls> {
        Self::Output::<Cometbls>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl AbciStateRead for ClientConsensusStatePath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls> {
        Self::Output::<Cometbls>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl AbciStateRead for ConnectionPath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls> {
        Self::Output::<Cometbls>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl AbciStateRead for ChannelEndPath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls> {
        Self::Output::<Cometbls>::try_from_proto_bytes(&bytes).unwrap()
    }
}

impl AbciStateRead for CommitmentPath {
    fn from_abci_bytes(bytes: Vec<u8>) -> Self::Output<Cometbls> {
        bytes.try_into().unwrap()
    }
}

pub struct CosmosStateRead;

impl<P> IbcStateRead<Cometbls, P> for CosmosStateRead
where
    P: IbcPath + AbciStateRead + 'static,
{
    fn state_proof(
        light_client: &Cometbls,
        path: P,
        at: Height,
    ) -> impl Future<Output = StateProof<P::Output<Cometbls>>> + '_ {
        async move {
            let mut client =
                protos::cosmos::base::tendermint::v1beta1::service_client::ServiceClient::connect(
                    light_client.chain.grpc_url.clone(),
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
                proof_height: light_client
                    .chain
                    .make_height(query_result.height.try_into().unwrap()),
            }
        }
    }
}

impl Cosmos {
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
}
