use std::str::FromStr;

use bip32::{DerivationPath, Language, XPrv};
use contracts::{
    glue::UnionIbcLightclientsCometblsV1ClientStateData,
    ibc_handler::{
        IBCHandler, IBCHandlerEvents, IbcCoreChannelV1ChannelData,
        IbcCoreChannelV1CounterpartyData, IbcCoreCommitmentV1MerklePrefixData,
        IbcCoreConnectionV1CounterpartyData, IbcCoreConnectionV1VersionData,
    },
    shared_types::IbcCoreClientV1HeightData,
};
use cosmos_to_eth::create_ibc_handler_client;
use ethers::{
    abi::AbiDecode, prelude::decode_logs, providers::Middleware, types::TransactionReceipt,
};
use prost::Message;
use protos::{
    cosmos::{
        self,
        auth::v1beta1::{BaseAccount, QueryAccountRequest},
        ics23::v1::{HashOp, InnerSpec, LeafOp, LengthOp, ProofSpec},
        staking,
    },
    google::protobuf,
    ibc::{
        core::{
            channel::{
                self,
                v1::{MsgChannelOpenAck, MsgChannelOpenInit, Order, QueryChannelRequest},
            },
            client::{
                self,
                v1::{
                    Height, QueryClientStateRequest, QueryConsensusStateHeightsRequest,
                    QueryConsensusStateRequest,
                },
            },
            commitment::v1::MerklePrefix,
            connection::{
                self,
                v1::{MsgConnectionOpenAck, MsgConnectionOpenInit, QueryConnectionRequest},
            },
        },
        lightclients::{
            tendermint::{self, v1::Fraction},
            wasm,
        },
    },
};
use tendermint_rpc::{endpoint::commit, Client, WebSocketClient, WebSocketClientUrl};

use crate::{
    cosmos_to_eth::{create_client, CHAIN_ID, COMETBLS_CLIENT_TYPE, PORT_ID},
    eth_to_cosmos::{broadcast_tx_commit, create_wasm_client, signer_from_pk},
};
// use cosmrs::crypto::secp256k1::SigningKey;

mod cosmos_to_eth;
mod eth_to_cosmos;

const ETH_BEACON_RPC_API: &str = "http://localhost:9596";

const ETH_RPC_API: &str = "http://localhost:8545";

const WASM_CLIENT_ID: &str = "08-wasm-0";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // dbg!(get_wallet());

    // panic!();

    // cosmos::get_wasm_code().await

    // let mut sequence = 0;

    // eth_to_cosmos::create_wasm_client(sequence).await;

    // sequence += 1;

    // // dbg!(cosmos::query_for_wasm_light_client().await);

    // eth_to_cosmos::update_wasm_client(sequence).await;

    // cosmos_to_eth::update_contract().await;

    let sequence = get_sequence().await;

    // let mut sequence = 3;

    let ibc_handler = create_ibc_handler_client().await;

    let bind_rcp: TransactionReceipt = ibc_handler
        .bind_port(PORT_ID.into(), ICS20_MODULE_ADDRESS.parse().unwrap())
        .send()
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();

    let connection_id = handshake(ibc_handler.clone()).await;

    // "connection-0".to_string()
    channel_handshake(ibc_handler, connection_id).await;
}

async fn get_sequence() -> u64 {
    let account = cosmos::auth::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
        .await
        .unwrap()
        .account(QueryAccountRequest {
            address: signer_from_pk(&get_wallet().public_key().public_key().to_bytes().to_vec()),
        })
        .await
        .unwrap()
        .into_inner()
        .account
        .unwrap();

    assert!(account.type_url == "/cosmos.auth.v1beta1.BaseAccount");

    let sequence = BaseAccount::decode(&*account.value).unwrap().sequence;

    println!("sequence is {sequence}");

    sequence
}

// nix run .#evm-devnet-deploy -L
// OwnableIBCHandler => address
const IBC_HANDLER_ADDRESS: &str = "0x0baE8645095583bc7fFC9a806743C3bA1B9ea0ec";
// CometblsClient => address
const COMETBLS_CLIENT_ADDRESS: &str = "0xD583F49C022ccc912A907C892F494fd04A1A9935";
// ICS20TransferBank => address
const ICS20_MODULE_ADDRESS: &str = "0x341aA165C8bC3719c28Fb8c6A150E95a1342D081";

const API_URL: &str = "http://127.0.0.1:27444";

// const TM_CLIENT_ID: &str = "08-wasm-0";
// const ETH_CLIENT_ID: &str = "10-eth-0";

// const CONNECTION_ID: &str = "connection-0";

// fn main() {
//     println!("Hello, world!");

//     let counterparty_of_eth = Counterparty {
//         client_id: TM_CLIENT_ID.to_owned(),
//         connection_id: CONNECTION_ID.to_owned(),
//         prefix: Some(default_merkle_prefix()),
//     };

//     let eth_connection_end = ConnectionEnd {
//         // versions: vec![Version {
//         //     identifier: todo!(),
//         //     features: todo!(),
//         // }],
//         versions: vec![],
//         state: State::Init as _,
//         counterparty: Some(counterparty_of_eth),
//         delay_period: 0,
//         client_id: ETH_CLIENT_ID.to_owned(),
//     };
// }

fn default_merkle_prefix() -> MerklePrefix {
    MerklePrefix {
        key_prefix: b"ibc".to_vec(),
    }
}

// async fn eth_get_proof() -> EIP1186ProofResponse {
//     let contract_address: H160 = std::fs::read_to_string("address.txt")
//         .unwrap()
//         .parse()
//         .unwrap();

//     dbg!(contract_address);

//     let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();

//     let block = provider
//         .get_block(BlockNumber::Safe)
//         .await
//         .unwrap()
//         .unwrap();

//     let block_number = block.number.unwrap();

//     dbg!(&block_number);

//     provider
//         .get_proof(
//             H160::from(contract_address),
//             vec![
//                 H256(hex!(
//                     "0000000000000000000000000000000000000000000000000000000000000000"
//                 )),
//                 H256(hex!(
//                     "0000000000000000000000000000000000000000000000000000000000000001"
//                 )),
//                 H256(hex!(
//                     "0000000000000000000000000000000000000000000000000000000000000002"
//                 )),
//             ],
//             Some(BlockId::Number(BlockNumber::Number(block_number))),
//         )
//         .await
//         .unwrap()
// }

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

    // cosmrs::AccountId::new("union", &alice.public_key().to_bytes()).unwrap()

    // subtle_encoding::bech32::encode("cosmos", public_key.to_bytes())
}

// async fn msg_connection_open_init() {
//     let wallet = get_wallet();
//     let alice = wallet;

//     let default_connection_version = default_connection_version();

//     let msg_connection_open_init = MsgConnectionOpenInit {
//         client_id: TM_CLIENT_ID.into(),
//         counterparty: Some(Counterparty {
//             client_id: "07-tendermint-0".into(),
//             // TODO(benluelo): cosmjs leaves this undefined, figure out what to put here
//             connection_id: "".into(),
//             prefix: Some(MerklePrefix {
//                 key_prefix: b"ibc".to_vec(),
//             }),
//         }),
//         version: Some(default_connection_version),
//         delay_period: 0,
//         signer: AccountId::new("cosmos", &alice.public_key().to_bytes())
//             .unwrap()
//             .to_string(),
//     };

//     let any = Any {
//         type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".into(),
//         value: msg_connection_open_init.encode_to_vec(),
//     };

//     let tx = Tx {
//         body: {
//             TxBody {
//                 messages: [any].to_vec(),
//                 memo: "".into(),
//                 timeout_height: 0_u8.into(),
//                 extension_options: Default::default(),
//                 non_critical_extension_options: Default::default(),
//             }
//         }
//         .into(),
//         auth_info: Some(AuthInfo {
//             signer_infos: [SignerInfo {
//                 public_key: None,
//                 // public_key: Some(SignerPublicKey::Single(alice.public_key())),
//                 mode_info: None,
//                 // mode_info: ModeInfo::single(SignMode::Direct),
//                 sequence: 0,
//             }]
//             .to_vec(),
//             fee: Some(Fee {
//                 amount: [Coin {
//                     denom: "uno".into(),
//                     amount: "123123123".into(),
//                 }]
//                 .to_vec(),
//                 gas_limit: todo!(),
//                 payer: "".into(),
//                 granter: "".into(),
//             }),
//             tip: None,
//         }),
//         signatures: [alice
//             .sign(&msg_connection_open_init.encode_to_vec())
//             .unwrap()
//             .to_vec()]
//         .to_vec(),
//     };

//     let response = connection::v1::msg_client::MsgClient::connect(API_URL)
//         .await
//         .unwrap()
//         .connection_open_init(msg_connection_open_init)
//         .await
//         .unwrap()
//         .into_inner();
// }

fn default_connection_version() -> connection::v1::Version {
    let default_connection_version = connection::v1::Version {
        identifier: "1".into(),
        features: [
            Order::Unordered.as_str_name().into(),
            Order::Ordered.as_str_name().into(),
        ]
        .to_vec(),
    };
    default_connection_version
}

// async fn msg_connection_open_try() {
//     let alice = get_wallet();

//     let any = Any {
//         type_url: "/ibc.lightclients.tendermint.v1.ClientState".into(),
//         value: get_tm_client_state().encode_to_vec(),
//     };

//     let wasm_client_state = wasm::v1::ClientState {
//         data: any.encode_to_vec(),
//         code_id: get_last_wasm_client().await,
//         latest_height: Some(Height {
//             revision_number: 0,
//             revision_height: 6,
//         }),
//     };

//     let msg = MsgConnectionOpenTry {
//         client_id: "08-wasm-0".into(),
//         previous_connection_id: "".into(),
//         client_state: Some(Any {
//             type_url: "/ibc.core.connection.v1.MsgConnectionOpenTry".into(),
//             value: wasm_client_state.encode_to_vec(),
//         }),
//         counterparty: Some(Counterparty {
//             client_id: "07-tendermint-0".into(),
//             connection_id: "connection-0".into(),
//             prefix: Some(default_merkle_prefix()),
//         }),
//         delay_period: 0,
//         counterparty_versions: vec![default_connection_version()],
//         proof_height: Some(Height {
//             revision_number: 0,
//             revision_height: 0,
//         }),
//         proof_init: vec![1, 2],
//         proof_client: vec![1, 2],
//         proof_consensus: vec![1, 2],
//         consensus_height: Some(Height {
//             revision_number: 0,
//             revision_height: 1,
//         }),
//         signer: AccountId::new("cosmos", &alice.public_key().to_bytes())
//             .unwrap()
//             .to_string(),
//         host_consensus_state_proof: vec![],
//     };
// }

// async fn get_last_wasm_client() -> Vec<u8> {
//     let mut wasm_client =
//         wasm::v1::query_client::QueryClient::connect(tonic::transport::Endpoint::from_static(""))
//             .await
//             .unwrap();

//     let code_ids = wasm_client
//         .code_ids(QueryCodeIdsRequest { pagination: None })
//         .await
//         .unwrap();

//     let code_id = code_ids.into_inner().code_ids.pop().unwrap();

//     hex::decode(code_id).unwrap()
// }

// fn get_tm_client_state() -> tendermint::v1::ClientState {
//     tendermint::v1::ClientState {
//         chain_id: "ibc-0".into(),
//         trust_level: Some(Fraction {
//             numerator: 1,
//             denominator: 3,
//         }),
//         trusting_period: Some(Duration::from_secs(1814400).into()),
//         unbonding_period: Some(Duration::from_secs(1814400).into()),
//         max_clock_drift: Some(Duration::from_secs(40).into()),
//         frozen_height: Some(Height {
//             revision_number: 0,
//             revision_height: 0,
//         }),
//         latest_height: Some(Height {
//             revision_number: 0,
//             revision_height: 5,
//         }),
//         proof_specs: [
//             ProofSpec {
//                 leaf_spec: Some(LeafOp {
//                     hash: HashOp::Sha256 as _,
//                     prehash_key: HashOp::NoHash as _,
//                     prehash_value: HashOp::Sha256 as _,
//                     length: LengthOp::VarProto as _,
//                     prefix: b"AA==".to_vec(),
//                 }),
//                 inner_spec: Some(InnerSpec {
//                     child_order: vec![0, 1],
//                     child_size: 33,
//                     min_prefix_length: 4,
//                     max_prefix_length: 12,
//                     empty_child: vec![],
//                     hash: HashOp::Sha256 as _,
//                 }),
//                 max_depth: 0,
//                 min_depth: 0,
//             },
//             ProofSpec {
//                 leaf_spec: Some(LeafOp {
//                     hash: HashOp::Sha256 as _,
//                     prehash_key: HashOp::NoHash as _,
//                     prehash_value: HashOp::Sha256 as _,
//                     length: LengthOp::VarProto as _,
//                     prefix: b"AA==".to_vec(),
//                 }),
//                 inner_spec: Some(InnerSpec {
//                     child_order: vec![0, 1],
//                     child_size: 32,
//                     min_prefix_length: 1,
//                     max_prefix_length: 1,
//                     empty_child: vec![],
//                     hash: HashOp::Sha256 as _,
//                 }),
//                 max_depth: 0,
//                 min_depth: 0,
//             },
//         ]
//         .to_vec(),
//         upgrade_path: ["upgrade".into(), "upgradedIBCState".into()].to_vec(),
//         allow_update_after_expiry: true,
//         allow_update_after_misbehaviour: true,
//     }
// }

// trait IbcEndpoint {
//     fn send_message();
// }

// async fn handshake() {
//     const CLIENT_A_ID: &str = "client-a";
//     const CLIENT_B_ID: &str = "client-b";

//     // }

async fn handshake<M>(ibc_handler: IBCHandler<M>) -> String
where
    M: Middleware + 'static,
{
    const COMETBLS_CLIENT_ID: &str = "cometbls-0";

    // let wasm_client_update = client::v1::MsgUpdateClient {
    //     client_id: WASM_CLIENT_ID.to_string(),
    //     client_message: todo!(),
    //     signer: todo!(),
    // };

    // let a_end = connection::v1::ConnectionEnd {
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

    let handler = create_ibc_handler_client().await;

    handler
        .register_client(
            COMETBLS_CLIENT_TYPE.into(),
            COMETBLS_CLIENT_ADDRESS.parse().unwrap(),
        )
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

    println!("Creating client...");

    let eth_client_id = create_client(&handler, &commit, &staking_params).await;

    let create_wasm_client_response = create_wasm_client(get_sequence().await).await;

    dbg!(create_wasm_client_response);

    let alice = get_wallet();
    let alice_pk = alice.public_key().public_key().to_bytes().to_vec();

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.connection.v1.MsgConnectionOpenInit".into(),
        value: MsgConnectionOpenInit {
            client_id: WASM_CLIENT_ID.to_string(),
            counterparty: Some(connection::v1::Counterparty {
                client_id: eth_client_id.clone(),
                connection_id: "".to_string(),
                prefix: Some(default_merkle_prefix()),
            }),
            version: Some(default_connection_version()),
            delay_period: 0,
            signer: signer_from_pk(&alice_pk),
        }
        .encode_to_vec(),
    };

    let response = broadcast_tx_commit(
        [msg].to_vec(),
        alice_pk.clone(),
        get_wallet(),
        get_sequence().await,
    )
    .await;

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
            client_state_bytes: Default::default(),
            counterparty_versions: [IbcCoreConnectionV1VersionData {
                identifier: default_connection_version().identifier,
                features: default_connection_version().features,
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

    let msg = protos::google::protobuf::Any {
        type_url: "/ibc.core.connection.v1.MsgConnectionOpenAck".into(),
        value: MsgConnectionOpenAck {
            connection_id: connection_id.clone(),
            counterparty_connection_id: connection_id.clone(),
            version: Some(default_connection_version()),
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
                                revision_height: cometbls_client_state
                                    .latest_height
                                    .revision_height,
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

    let ack_response =
        broadcast_tx_commit([msg].to_vec(), alice_pk, get_wallet(), get_sequence().await).await;

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

    let (tm_client, tm_driver) = WebSocketClient::builder(
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
        value: MsgChannelOpenInit {
            signer: signer_from_pk(&alice_pk),
            port_id: PORT_ID.to_string(),
            channel: Some(channel::v1::Channel {
                state: channel::v1::State::Init as i32,
                ordering: Order::Unordered as i32,
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

    let response = broadcast_tx_commit(
        [msg].to_vec(),
        alice_pk.clone(),
        get_wallet(),
        get_sequence().await,
    )
    .await;

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

    let ack_response =
        broadcast_tx_commit([msg].to_vec(), alice_pk, get_wallet(), get_sequence().await).await;

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
