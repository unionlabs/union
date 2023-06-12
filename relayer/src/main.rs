// *almost* stable, more than safe enough to use imo https://github.com/rust-lang/rfcs/pull/3425
#![feature(return_position_impl_trait_in_trait)]
#![allow(clippy::manual_async_fn)]

use std::{collections::HashMap, str::FromStr, time::Duration};

use bip32::{DerivationPath, Language, XPrv};
use chain::{
    cosmos::Ethereum,
    evm::Cometbls,
    msgs::{
        self,
        connection::{
            MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
            MsgConnectionOpenTry,
        },
        MerklePrefix,
    },
    Connect, LightClient,
};
use clap::Parser;
use contracts::{
    glue::UnionIbcLightclientsCometblsV1ClientStateData,
    ibc_handler::{
        IBCHandler, IBCHandlerEvents, IbcCoreChannelV1ChannelData,
        IbcCoreChannelV1CounterpartyData, IbcCoreChannelV1PacketData,
        IbcCoreCommitmentV1MerklePrefixData, IbcCoreConnectionV1CounterpartyData,
        IbcCoreConnectionV1VersionData,
    },
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::{
    abi::AbiDecode,
    prelude::decode_logs,
    providers::Middleware,
    types::{Address, H256},
};
use futures::{FutureExt, StreamExt};
use prost::Message;
use protos::{
    cosmos::{
        self,
        auth::v1beta1::{BaseAccount, QueryAccountRequest},
        base::v1beta1::Coin,
        ics23::v1::{HashOp, InnerSpec, LeafOp, LengthOp, ProofSpec},
        staking,
    },
    google::protobuf::{self, Any},
    ibc::{
        applications::transfer::v1::MsgTransfer,
        core::{
            channel::{
                self,
                v1::{MsgChannelOpenAck, QueryChannelRequest, QueryPacketCommitmentRequest},
            },
            client::{
                self,
                v1::{
                    Height, QueryClientStateRequest, QueryConsensusStateHeightsRequest,
                    QueryConsensusStateRequest,
                },
            },
            connection::{self, v1::QueryConnectionRequest},
        },
        lightclients::{
            tendermint::{self, v1::Fraction},
            wasm,
        },
    },
};
use tendermint_rpc::{
    endpoint::commit, event::EventData, query::EventType, Client, SubscriptionClient,
    WebSocketClient, WebSocketClientUrl,
};

use crate::{
    cosmos_to_eth::{create_client, CHAIN_ID, COMETBLS_CLIENT_TYPE, PORT_ID},
    eth_to_cosmos::{broadcast_tx_commit, create_wasm_client, signer_from_pk},
};

pub mod chain;

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
mod cosmos_to_eth;
#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
mod eth_to_cosmos;

const ETH_BEACON_RPC_API: &str = "http://localhost:9596";

const ETH_RPC_API: &str = "http://localhost:8545";

const WASM_CLIENT_ID: &str = "08-wasm-0";

#[derive(Debug, Parser)]
pub struct Args {
    // nix run .#evm-devnet-deploy -L
    /// OwnableIBCHandler => address
    #[arg(long = "ibc-handler")]
    pub ibc_handler_address: Address,
    /// CometblsClient => address
    #[arg(long = "cometbls")]
    pub cometbls_client_address: Address,
    /// ICS20TransferBank => address
    #[arg(long = "ics20")]
    pub ics20_module_address: Address,

    #[arg(long = "code-id")]
    pub wasm_code_id: H256,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Registry::default()
    //     .with(
    //             .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env()),
    //     )
    //     .init();

    let args = Args::parse();

    // dbg!(get_wallet());

    // panic!();

    // cosmos::get_wasm_code().await

    // let mut sequence = 0;

    // eth_to_cosmos::create_wasm_client(sequence).await;

    // sequence += 1;

    // // dbg!(cosmos::query_for_wasm_light_client().await);

    // eth_to_cosmos::update_wasm_client(sequence).await;

    // cosmos_to_eth::update_contract().await;

    // let sequence = account_info_of_signer(&get_wallet()).await.sequence;

    // let mut sequence = 3;

    // let ibc_handler = create_ibc_handler_client(&args).await;

    // let bind_rcp: TransactionReceipt = ibc_handler
    //     .bind_port(PORT_ID.into(), args.ics20_module_address)
    //     .send()
    //     .await
    //     .unwrap()
    //     .await
    //     .unwrap()
    //     .unwrap();

    // let connection_id = handshake(ibc_handler.clone(), &args).await;

    // "connection-0".to_string()
    // channel_handshake(ibc_handler, connection_id).await;

    // relay_packets(ibc_handler).await;

    do_main(args).await
}

async fn do_main(args: Args) {
    // println!(
    //     "{}",
    //     wasm::v1::query_client::QueryClient::connect("tcp://0.0.0.0:9090")
    //         .await
    //         .unwrap()
    //         .code_ids(QueryCodeIdsRequest { pagination: None })
    //         .await
    //         .unwrap()
    //         .into_inner()
    //         .code_ids
    //         .first()
    //         .unwrap()
    // );

    // panic!();

    let cometbls = Cometbls::new(
        args.cometbls_client_address,
        args.ibc_handler_address,
        args.wasm_code_id,
    )
    .await;

    // let tx = cometbls.provider.get_block_with_txs(261).await.unwrap();

    // dbg!(tx);

    // panic!();

    let ethereum = Ethereum::new(get_wallet(), args.wasm_code_id).await;

    poignee_de_main(cometbls, ethereum).await;
}

async fn account_info_of_signer(signer: &XPrv) -> BaseAccount {
    let account = cosmos::auth::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
        .await
        .unwrap()
        .account(QueryAccountRequest {
            address: signer_from_pk(&signer.public_key().public_key().to_bytes().to_vec()),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

    assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

    BaseAccount::decode(&*account.value).unwrap()
}

// const API_URL: &str = "http://127.0.0.1:27444";

fn default_merkle_prefix() -> MerklePrefix {
    MerklePrefix {
        key_prefix: b"ibc".to_vec(),
    }
}

fn get_wallet() -> XPrv {
    const MNEMONIC: &str = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
    // const DERIVATION_PATH: &str = "m/44'/1337'/0'/0/0";
    const DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";
    const PASSWORD: &str = "";

    let mnemonic = bip32::Mnemonic::new(MNEMONIC, Language::English);

    let derivation_path = DerivationPath::from_str(DERIVATION_PATH).unwrap();

    let alice = XPrv::derive_from_path(
        mnemonic.unwrap().to_seed(PASSWORD).as_bytes(),
        &derivation_path,
    )
    .unwrap();

    alice
}

async fn poignee_de_main<Chain1, Chain2>(cometbls: Chain1, ethereum: Chain2)
where
    Chain1: LightClient + Connect<Chain2>,
    Chain2: LightClient + Connect<Chain1>,
    <Chain1 as LightClient>::ClientState: std::fmt::Debug,
{
    // let bytes = hex!("345656e1e0827561fa3ec35d3bc40493fd66ea4f52b68d38e915ecb403df2f6e");

    // let connection_end =
    //     <protos::ibc::core::connection::v1::ConnectionEnd as prost::Message>::decode(&bytes[..]);

    // dbg!(connection_end);

    // panic!();

    let cometbls_id = cometbls.chain_id().await;
    let ethereum_id = ethereum.chain_id().await;

    tracing::info!(cometbls_id, ethereum_id);

    let (cometbls_client_id, ethereum_latest_height) = {
        let latest_height = ethereum.query_latest_height().await;

        tracing::trace!("generating client state...");
        let client_state = ethereum
            .generate_counterparty_client_state(latest_height)
            .await;
        tracing::trace!("generating consensus state...");
        let consensus_state = ethereum
            .generate_counterparty_consensus_state(latest_height)
            .await;

        let client_id = cometbls.create_client(client_state, consensus_state).await;

        tracing::info!(chain_id = cometbls_id, client_id);

        (client_id, latest_height)
    };

    let (ethereum_client_id, cometbls_latest_height) = {
        let latest_height = cometbls.query_latest_height().await;

        tracing::trace!("generating client state...");
        let client_state = cometbls
            .generate_counterparty_client_state(latest_height)
            .await;
        tracing::trace!("generating consensus state...");
        let consensus_state = cometbls
            .generate_counterparty_consensus_state(latest_height)
            .await;

        let client_id = ethereum.create_client(client_state, consensus_state).await;

        tracing::info!(chain_id = ethereum_id, client_id);

        (client_id, latest_height)
    };

    tracing::info!(?cometbls_latest_height);
    tracing::info!(?ethereum_latest_height);

    let cometbls_connection_id = cometbls
        .connection_open_init(MsgConnectionOpenInit {
            client_id: cometbls_client_id.clone(),
            counterparty: msgs::connection::Counterparty {
                client_id: ethereum_client_id.clone(),
                // TODO(benluelo): Create a new struct with this field omitted as it's unused for open init
                connection_id: "".to_string(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            version: msgs::connection::Version {
                identifier: "1".into(),
                features: [
                    msgs::channel::Order::Unordered,
                    msgs::channel::Order::Ordered,
                ]
                .into_iter()
                .collect(),
            },
            delay_period: 6,
        })
        .await;

    let cometbls_latest_height = cometbls.query_latest_height().await;

    cometbls
        .generate_counterparty_update_client_message(cometbls_latest_height)
        .then(|update| ethereum.update_client(ethereum_client_id.clone(), update))
        .await;

    tracing::info!(
        chain_id = cometbls_id,
        connection_id = cometbls_connection_id
    );

    // generate state proofs

    let cometbls_client_state_proof = cometbls
        .client_state_proof(cometbls_client_id.clone(), cometbls_latest_height)
        .await;
    let cometbls_consensus_state_proof = cometbls
        .consensus_state_proof(
            cometbls_client_id.clone(),
            ethereum_latest_height,
            cometbls_latest_height,
        )
        .await;
    let cometbls_connection_state_proof = cometbls
        .connection_state_proof(cometbls_connection_id.clone(), cometbls_latest_height)
        .await;

    let ethereum_connection_id = ethereum
        .connection_open_try(MsgConnectionOpenTry {
            client_id: ethereum_client_id.clone(),
            counterparty: msgs::connection::Counterparty {
                client_id: cometbls_client_id.clone(),
                connection_id: cometbls_connection_id.clone(),
                prefix: MerklePrefix {
                    key_prefix: b"ibc".to_vec(),
                },
            },
            delay_period: 6,
            client_state: cometbls_client_state_proof.state,
            counterparty_versions: cometbls_connection_state_proof.state.versions,
            proof_height: cometbls_consensus_state_proof.proof_height,
            proof_init: cometbls_connection_state_proof.proof,
            proof_client: cometbls_client_state_proof.proof,
            proof_consensus: cometbls_consensus_state_proof.proof,
            consensus_height: ethereum_latest_height,
        })
        .await;

    let ethereum_connection_state_proof = ethereum
        .connection_state_proof(ethereum_connection_id.clone(), ethereum_latest_height)
        .await;

    let ethereum_client_state_proof = ethereum
        .client_state_proof(ethereum_client_id.clone(), ethereum_latest_height)
        .await;

    let ethereum_consensus_state_proof = ethereum
        .consensus_state_proof(
            ethereum_client_id.clone(),
            cometbls_latest_height,
            ethereum_latest_height.increment(),
        )
        .await;

    cometbls
        .connection_open_ack(MsgConnectionOpenAck {
            connection_id: cometbls_connection_id.clone(),
            counterparty_connection_id: ethereum_connection_id.clone(),
            version: msgs::connection::Version {
                identifier: "1".into(),
                features: [
                    msgs::channel::Order::Unordered,
                    msgs::channel::Order::Ordered,
                ]
                .into_iter()
                .collect(),
            },
            client_state: ethereum_client_state_proof.state,
            proof_height: ethereum_connection_state_proof.proof_height,
            proof_try: ethereum_connection_state_proof.proof,
            proof_client: ethereum_client_state_proof.proof,
            proof_consensus: ethereum_consensus_state_proof.proof,
            consensus_height: ethereum_consensus_state_proof.proof_height,
        })
        .await;

    let cometbls_connection_state_proof = cometbls
        .connection_state_proof(
            cometbls_connection_id.clone(),
            cometbls_latest_height.increment(),
        )
        .await;

    ethereum
        .connection_open_confirm(MsgConnectionOpenConfirm {
            connection_id: ethereum_connection_id,
            proof_ack: cometbls_connection_state_proof.proof,
            proof_height: cometbls_connection_state_proof.proof_height,
        })
        .await;
}

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
async fn handshake<M>(ibc_handler: IBCHandler<M>, args: &Args) -> String
where
    M: Middleware + 'static,
{
    const COMETBLS_CLIENT_ID: &str = "cometbls-0";

    let (tm_client, tm_driver) = WebSocketClient::builder(
        WebSocketClientUrl::from_str("ws://0.0.0.0:26657/websocket").unwrap(),
    )
    .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
    .build()
    .await
    .unwrap();

    // let (rx, tx) = tendermint_rpc::client::sync::unbounded();

    let _ = tokio::spawn(async move { tm_driver.run().await });

    let mut staking_client =
        staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let staking_params = staking_client
        .params(staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

    let commit: commit::Response = tm_client.latest_commit().await.unwrap();

    ibc_handler
        .register_client(COMETBLS_CLIENT_TYPE.into(), args.cometbls_client_address)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    println!("Creating client...");

    let eth_client_id = create_client(&ibc_handler, &commit, &staking_params).await;

    let create_wasm_client_response = create_wasm_client().await;

    dbg!(create_wasm_client_response);

    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".into(),
        value: connection::v1::MsgConnectionOpenInit {
            client_id: WASM_CLIENT_ID.to_string(),
            counterparty: Some(connection::v1::Counterparty {
                client_id: eth_client_id.clone(),
                connection_id: "".to_string(),
                prefix: Some(default_merkle_prefix().into()),
            }),
            version: Some(todo!()),
            delay_period: 0,
            signer: signer_from_pk(&alice_pk),
        }
        .encode_to_vec(),
    };

    let response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(&response);

    let connection_id = response
        .deliver_tx
        .events
        .into_iter()
        .find(|event| event.kind == "connection_open_init")
        .unwrap()
        .attributes
        .into_iter()
        .find(|attr| attr.key == "connection_id")
        .unwrap()
        .value;

    let mut connection_query_client =
        connection::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let connection_proof = connection_query_client
        .connection(QueryConnectionRequest {
            connection_id: connection_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    let mut client_query_client =
        client::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let client_state_proof = client_query_client
        .client_state(QueryClientStateRequest {
            client_id: WASM_CLIENT_ID.to_string(),
        })
        .await
        .unwrap()
        .into_inner();

    let consensus_state_proof = client_query_client
        .consensus_state(QueryConsensusStateRequest {
            client_id: WASM_CLIENT_ID.to_string(),
            revision_number: connection_proof
                .proof_height
                .clone()
                .unwrap()
                .revision_number,
            revision_height: 0,
            latest_height: true,
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(std::time::SystemTime::now());

    let try_response = ibc_handler
        .connection_open_try(contracts::ibc_handler::MsgConnectionOpenTry {
            counterparty: IbcCoreConnectionV1CounterpartyData {
                client_id: WASM_CLIENT_ID.to_string(),
                connection_id: connection_id.clone(),
                prefix: IbcCoreCommitmentV1MerklePrefixData {
                    key_prefix: default_merkle_prefix().key_prefix.into(),
                },
            },
            delay_period: 0,
            client_id: COMETBLS_CLIENT_ID.to_string(),
            // for membership verification, however it's stored in the store
            // i.e. ibc/clientStates/whatever
            // TYPE: proto(wasm<eth::v1::clientstate>)
            // WasmEth::ClientState (proto encoded)
            client_state_bytes: Default::default(),
            counterparty_versions: [IbcCoreConnectionV1VersionData {
                // identifier: default_connection_version().identifier,
                // features: default_connection_version().features,
                identifier: todo!(),
                features: todo!(),
            }]
            .to_vec(),
            proof_init: connection_proof.proof.into(),
            proof_client: client_state_proof.proof.into(),
            proof_consensus: consensus_state_proof.proof.into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: connection_proof
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_number,
                revision_height: connection_proof.proof_height.unwrap().revision_height,
            },
            consensus_height: IbcCoreClientV1HeightData {
                revision_number: consensus_state_proof
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_number,
                revision_height: consensus_state_proof
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_height,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    dbg!(std::time::SystemTime::now());

    dbg!(try_response);

    let (cometbls_client_state_bytes, is_found) = ibc_handler
        .get_client_state(COMETBLS_CLIENT_ID.to_string())
        .await
        .unwrap();

    assert!(is_found);

    let cometbls_client_state: UnionIbcLightclientsCometblsV1ClientStateData =
        AbiDecode::decode(cometbls_client_state_bytes).unwrap();

    dbg!(&cometbls_client_state);

    let wasm_client_state =
        wasm::v1::ClientState::decode(&*client_state_proof.client_state.unwrap().value).unwrap();

    dbg!(&wasm_client_state);

    #[allow(deprecated)]
    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.connection.v1.MsgConnectionOpenAck".into(),
        value: connection::v1::MsgConnectionOpenAck {
            connection_id: connection_id.clone(),
            counterparty_connection_id: connection_id.clone(),
            version: Some(todo!()),
            // version: Some(default_connection_version()),
            client_state: Some(protos::google::protobuf::Any {
                type_url: "/ibc.lightclients.wasm.v1.ClientState".to_string(),
                value: wasm::v1::ClientState {
                    data: protos::google::protobuf::Any {
                        type_url: "/ibc.lightclients.tendermint.v1.ClientState".to_string(),
                        value: tendermint::v1::ClientState {
                            chain_id: CHAIN_ID.to_string(),
                            trust_level: Some(Fraction {
                                // numerator: cometbls_client_state.trust_level.numerator,
                                // denominator: cometbls_client_state.trust_level.denominator,
                                numerator: 1,
                                denominator: 3,
                            }),
                            trusting_period: Some(protobuf::Duration {
                                // seconds: cometbls_client_state.trusting_period.seconds,
                                // nanos: cometbls_client_state.trusting_period.nanos,
                                seconds: 1814400,
                                nanos: 0,
                            }),
                            unbonding_period: Some(protobuf::Duration {
                                // seconds: cometbls_client_state.unbonding_period.seconds,
                                // nanos: cometbls_client_state.unbonding_period.nanos,
                                seconds: 1814400,
                                nanos: 0,
                            }),
                            max_clock_drift: Some(protobuf::Duration {
                                // seconds: cometbls_client_state.max_clock_drift.seconds,
                                // nanos: cometbls_client_state.max_clock_drift.nanos,
                                seconds: 40,
                                nanos: 0,
                            }),
                            frozen_height: Some(Height {
                                revision_number: cometbls_client_state
                                    .frozen_height
                                    .revision_number,
                                revision_height: cometbls_client_state
                                    .frozen_height
                                    .revision_height,
                            }),
                            latest_height: Some(Height {
                                // revision_number: cometbls_client_state
                                //     .latest_height
                                //     .revision_number,
                                revision_number: 1,
                                revision_height: todo!(),
                            }),
                            proof_specs: [
                                ProofSpec {
                                    leaf_spec: Some(LeafOp {
                                        hash: HashOp::Sha256 as _,
                                        prehash_key: HashOp::NoHash as _,
                                        prehash_value: HashOp::Sha256 as _,
                                        length: LengthOp::VarProto as _,
                                        prefix: [0].to_vec(),
                                    }),
                                    inner_spec: Some(InnerSpec {
                                        child_order: vec![0, 1],
                                        child_size: 33,
                                        min_prefix_length: 4,
                                        max_prefix_length: 12,
                                        empty_child: vec![],
                                        hash: HashOp::Sha256 as _,
                                    }),
                                    max_depth: 0,
                                    min_depth: 0,
                                },
                                ProofSpec {
                                    leaf_spec: Some(LeafOp {
                                        hash: HashOp::Sha256 as _,
                                        prehash_key: HashOp::NoHash as _,
                                        prehash_value: HashOp::Sha256 as _,
                                        length: LengthOp::VarProto as _,
                                        prefix: [0].to_vec(),
                                    }),
                                    inner_spec: Some(InnerSpec {
                                        child_order: vec![0, 1],
                                        child_size: 32,
                                        min_prefix_length: 1,
                                        max_prefix_length: 1,
                                        empty_child: vec![],
                                        hash: HashOp::Sha256 as _,
                                    }),
                                    max_depth: 0,
                                    min_depth: 0,
                                },
                            ]
                            .to_vec(),
                            upgrade_path: ["upgrade".to_string(), "upgradedIBCState".to_string()]
                                .to_vec(),
                            // TODO: figure out where to get these values from
                            allow_update_after_expiry: true,
                            allow_update_after_misbehaviour: true,
                        }
                        .encode_to_vec(),
                    }
                    .encode_to_vec(),
                    code_id: wasm_client_state.code_id,
                    latest_height: Some(Height {
                        revision_number: 1,
                        revision_height: wasm_client_state
                            .latest_height
                            .clone()
                            .unwrap()
                            .revision_height,
                    }),
                }
                .encode_to_vec(),
            }),
            proof_height: wasm_client_state.latest_height.clone(),
            proof_try: vec![1, 2, 3],
            proof_client: vec![1, 2, 3],
            proof_consensus: vec![1, 2, 3],
            consensus_height: consensus_state_proof.proof_height.clone(),
            signer: signer_from_pk(&alice_pk),
            host_consensus_state_proof: vec![],
        }
        .encode_to_vec(),
    };

    let ack_response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(ack_response);

    let connection_proof = connection_query_client
        .connection(QueryConnectionRequest {
            connection_id: connection_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(&connection_proof);

    ibc_handler
        .connection_open_confirm(contracts::ibc_handler::MsgConnectionOpenConfirm {
            connection_id: connection_id.clone(),
            proof_ack: connection_proof.proof.into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: connection_proof
                    .proof_height
                    .clone()
                    .unwrap()
                    .revision_number,
                revision_height: connection_proof.proof_height.unwrap().revision_height,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    connection_id
}

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
async fn channel_handshake<M>(ibc_handler: IBCHandler<M>, connection_id: String)
where
    M: Middleware + 'static,
{
    const CHANNEL_VERSION: &str = "ics20-1";
    const COMETBLS_CLIENT_ID: &str = "cometbls-0";

    // let wasm_client_update = client::v1::MsgUpdateClient {
    //     client_id: WASM_CLIENT_ID.to_string(),
    //     client_message: todo!(),
    //     signer: todo!(),
    // };

    // let a_end = connection::v1::ChannelEnd {
    //     client_id: CLIENT_A_ID.to_string(),
    //     versions: vec![default_connection_version()],
    //     state: connection::v1::State::Init.into(),
    //     counterparty: Some(connection::v1::Counterparty {
    //         client_id: CLIENT_B_ID.to_string(),
    //         connection_id: "connection-1".to_string(),
    //         prefix: Some(default_merkle_prefix()),
    //     }),
    //     delay_period: 0,
    // };

    let (_tm_client, tm_driver) = WebSocketClient::builder(
        WebSocketClientUrl::from_str("ws://0.0.0.0:26657/websocket").unwrap(),
    )
    .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
    .build()
    .await
    .unwrap();

    // let (rx, tx) = tendermint_rpc::client::sync::unbounded();

    let _ = tokio::spawn(async move { tm_driver.run().await });

    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.channel.v1.MsgChannelOpenInit".into(),
        value: channel::v1::MsgChannelOpenInit {
            signer: signer_from_pk(&alice_pk),
            port_id: PORT_ID.to_string(),
            channel: Some(channel::v1::Channel {
                state: channel::v1::State::Init as i32,
                ordering: channel::v1::Order::Unordered as i32,
                counterparty: Some(channel::v1::Counterparty {
                    port_id: PORT_ID.to_string(),
                    channel_id: "".to_string(),
                }),
                connection_hops: vec![connection_id.clone()],
                version: CHANNEL_VERSION.to_string(),
            }),
        }
        .encode_to_vec(),
    };

    let response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(&response);

    let cosmos_channel_id = response
        .deliver_tx
        .events
        .into_iter()
        .find(|event| event.kind == "channel_open_init")
        .unwrap()
        .attributes
        .into_iter()
        .find(|attr| attr.key == "channel_id")
        .unwrap()
        .value;

    let mut channel_query_client =
        channel::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let channel_proof = channel_query_client
        .channel(QueryChannelRequest {
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    let channel_open_try_receipt = ibc_handler
        .channel_open_try(contracts::ibc_handler::MsgChannelOpenTry {
            proof_init: channel_proof.proof.clone().into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
                revision_height: channel_proof.proof_height.clone().unwrap().revision_height,
            },
            port_id: PORT_ID.to_string(),
            channel: IbcCoreChannelV1ChannelData {
                state: channel::v1::State::Tryopen as u8,
                ordering: channel::v1::Order::Ordered as u8,
                counterparty: IbcCoreChannelV1CounterpartyData {
                    port_id: PORT_ID.to_string(),
                    channel_id: cosmos_channel_id.clone(),
                },
                connection_hops: vec![connection_id],
                version: CHANNEL_VERSION.to_string(),
            },
            counterparty_version: CHANNEL_VERSION.to_string(),
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let eth_channel_id = decode_logs::<IBCHandlerEvents>(
        channel_open_try_receipt
            .logs
            .into_iter()
            .map(|l| l.into())
            .collect::<Vec<_>>()
            .as_ref(),
    )
    .unwrap()
    .into_iter()
    .find_map(|l| match l {
        IBCHandlerEvents::GeneratedChannelIdentifierFilter(channel_id) => Some(channel_id.0),
        _ => None,
    })
    .unwrap();

    dbg!(&eth_channel_id);

    let (cometbls_client_state_bytes, is_found) = ibc_handler
        .get_client_state(COMETBLS_CLIENT_ID.to_string())
        .await
        .unwrap();

    assert!(is_found);

    let cometbls_client_state: UnionIbcLightclientsCometblsV1ClientStateData =
        AbiDecode::decode(cometbls_client_state_bytes).unwrap();

    dbg!(&cometbls_client_state);

    let mut client_query_client =
        client::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let consensus_state_proof = client_query_client
        .consensus_state(QueryConsensusStateRequest {
            client_id: WASM_CLIENT_ID.to_string(),
            revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
            revision_height: 0,
            latest_height: true,
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(&consensus_state_proof);

    let height = client_query_client
        .consensus_state_heights(QueryConsensusStateHeightsRequest {
            client_id: WASM_CLIENT_ID.to_string(),
            pagination: None,
        })
        .await
        .unwrap()
        .into_inner()
        .consensus_state_heights
        .into_iter()
        .max()
        .unwrap();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.channel.v1.MsgChannelOpenAck".into(),
        value: MsgChannelOpenAck {
            proof_height: Some(height),
            proof_try: vec![1, 2, 3],
            signer: signer_from_pk(&alice_pk),
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
            counterparty_channel_id: eth_channel_id.clone(),
            counterparty_version: CHANNEL_VERSION.to_string(),
        }
        .encode_to_vec(),
    };

    let ack_response = broadcast_tx_commit([msg].to_vec()).await;

    dbg!(ack_response);

    let channel_proof = channel_query_client
        .channel(QueryChannelRequest {
            port_id: PORT_ID.to_string(),
            channel_id: cosmos_channel_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();

    dbg!(&channel_proof);

    ibc_handler
        .channel_open_confirm(contracts::ibc_handler::MsgChannelOpenConfirm {
            port_id: PORT_ID.to_string(),
            channel_id: eth_channel_id.clone(),
            proof_ack: channel_proof.proof.into(),
            proof_height: IbcCoreClientV1HeightData {
                revision_number: channel_proof.proof_height.clone().unwrap().revision_number,
                revision_height: channel_proof.proof_height.unwrap().revision_height,
            },
        })
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    println!("successfully opened channel");
}

#[allow(
    dead_code,
    unused_variables,
    unreachable_code,
    clippy::diverging_sub_expression,
    clippy::let_underscore_future
)]
async fn relay_packets(ibc_handler: IBCHandler<impl Middleware + 'static>) {
    let listen_handle = tokio::spawn(async move {
        loop {
            let (client, driver) =
                WebSocketClient::builder("ws://127.0.0.1:26657/websocket".parse().unwrap())
                    .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
                    .build()
                    .await
                    .unwrap();

            let driver_handle = tokio::spawn(async move { driver.run().await });

            // Subscription functionality
            let mut subs = client.subscribe(EventType::Tx.into()).await.unwrap();

            while let Some(res) = subs.next().await {
                let ev = res.unwrap();

                // ibc_transfer { sender, reciever, amount, denom, memo? }

                println!("Got event: {:#?}", ev.events);

                match ev.data {
                    EventData::NewBlock {
                        block: _,
                        result_begin_block: _,
                        result_end_block: _,
                    } => {
                        // dbg!(result_begin_block, result_end_block);

                        // client.block(block.unwrap().header.height).await.unwrap();
                    }
                    EventData::Tx { tx_result } => {
                        let send_packet_event = tx_result
                            .result
                            .events
                            .into_iter()
                            .find_map(|e| {
                                (e.kind == "send_packet").then(|| {
                                    e.attributes
                                        .into_iter()
                                        .map(|attr| (attr.key, attr.value))
                                        .collect::<HashMap<_, _>>()
                                })
                            })
                            .unwrap();

                        let sequence = send_packet_event["packet_sequence"].parse().unwrap();

                        let packet_commitment =
                            channel::v1::query_client::QueryClient::connect("http://0.0.0.0:9090")
                                .await
                                .unwrap()
                                .packet_commitment(QueryPacketCommitmentRequest {
                                    port_id: PORT_ID.to_string(),
                                    channel_id: "channel-0".to_string(),
                                    sequence,
                                })
                                .await
                                .unwrap()
                                .into_inner();

                        let rcp = ibc_handler
                            .recv_packet(contracts::ibc_handler::MsgPacketRecv {
                                packet: IbcCoreChannelV1PacketData {
                                    sequence,
                                    source_port: send_packet_event["packet_src_port"].clone(),
                                    source_channel: send_packet_event["packet_src_channel"].clone(),
                                    destination_port: send_packet_event["packet_dst_port"].clone(),
                                    destination_channel: send_packet_event["packet_dst_channel"]
                                        .clone(),
                                    data: send_packet_event["packet_data"]
                                        .clone()
                                        .into_bytes()
                                        .into(),
                                    timeout_height: {
                                        let (revision, height) = send_packet_event
                                            ["packet_timeout_height"]
                                            .split_once('-')
                                            .unwrap();

                                        IbcCoreClientV1HeightData {
                                            revision_number: revision.parse().unwrap(),
                                            revision_height: height.parse().unwrap(),
                                        }
                                    },
                                    timeout_timestamp: send_packet_event
                                        ["packet_timeout_timestamp"]
                                        .parse()
                                        .unwrap(),
                                },
                                proof: packet_commitment.proof.into(),
                                proof_height: IbcCoreClientV1HeightData {
                                    revision_number: packet_commitment
                                        .proof_height
                                        .as_ref()
                                        .unwrap()
                                        .revision_number,
                                    revision_height: packet_commitment
                                        .proof_height
                                        .unwrap()
                                        .revision_height,
                                },
                            })
                            .send()
                            .await
                            .unwrap()
                            .await
                            .unwrap()
                            .unwrap();

                        dbg!(rcp);
                    }
                    EventData::GenericJsonEvent(_) => todo!(),
                };
            }

            println!("events finished");

            // Signal to the driver to terminate.
            client.close().unwrap();

            // Await the driver's termination to ensure proper connection closure.
            let _ = driver_handle.await.unwrap();
        }
    });

    let send_handle = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;

        let msg = MsgTransfer {
            source_port: PORT_ID.to_string(),
            source_channel: "channel-0".to_string(),
            token: Some(Coin {
                denom: "stake".to_string(),
                amount: "1".to_string(),
            }),
            sender: signer_from_pk(&get_wallet().public_key().public_key().to_bytes().to_vec()),
            receiver: "union1nrv37pqfcqul73v7d2e8y0jhjyeuhg57m3eqdt".to_string(),
            timeout_height: Some(Height {
                revision_number: 1,
                revision_height: 12_345_678_765,
            }),
            timeout_timestamp: Default::default(),
            memo: Default::default(),
        };

        broadcast_tx_commit(
            [Any {
                type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
                value: msg.encode_to_vec(),
            }]
            .to_vec(),
        )
        .await;
    });

    let (listen, send) = tokio::join!(listen_handle, send_handle);

    listen.unwrap();
    send.unwrap();
}
