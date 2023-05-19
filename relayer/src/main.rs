// use std::{str::FromStr, time::Duration};

// use bip32::Language;
// use cosmrs::{
//     bip32::DerivationPath,
//     crypto::secp256k1::{self, SigningKey},
//     proto::traits::Message,
//     AccountId,
// };
// use ethers::{
//     providers::{Http, Middleware, Provider},
//     types::{BlockId, BlockNumber, EIP1186ProofResponse, H160, H256},
//     utils::hex,
// };
// use hex_literal::hex;
// use protos::{
//     cosmos::{
//         base::v1beta1::Coin,
//         ics23::v1::{HashOp, InnerSpec, LeafOp, LengthOp, ProofSpec},
//         tx::v1beta1::{AuthInfo, Fee, SignerInfo, Tx, TxBody},
//     },
//     google::protobuf::Any,
//     ibc::{
//         core::{
//             client::v1::{Height, MsgUpdateClient},
//             commitment::v1::MerklePrefix,
//             connection::{
//                 self,
//                 v1::{
//                     ConnectionEnd, Counterparty, MsgConnectionOpenInit, MsgConnectionOpenTry,
//                     State, Version,
//                 },
//             },
//         },
//         lightclients::{
//             tendermint::{self, v1::Fraction},
//             wasm::{self, v1::QueryCodeIdsRequest},
//         },
//     },
// };

use std::str::FromStr;

use bip32::{DerivationPath, Language, XPrv};
// use cosmrs::crypto::secp256k1::SigningKey;

mod cosmos;

#[tokio::main]
async fn main() {
    // dbg!(get_wallet());

    // panic!();

    // cosmos::get_wasm_code().await

    let mut sequence = 0;

    cosmos::create_wasm_client(sequence).await;

    sequence += 1;

    // dbg!(cosmos::query_for_wasm_light_client().await);

    cosmos::update_wasm_client(sequence).await;
}

// const API_URL: &str = "http://127.0.0.1:27444";

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

// fn default_merkle_prefix() -> MerklePrefix {
//     MerklePrefix {
//         key_prefix: b"ibc".to_vec(),
//     }
// }

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

// fn default_connection_version() -> Version {
//     let default_connection_version = Version {
//         identifier: "1".into(),
//         features: ["ORDER_ORDERED".into(), "ORDER_UNORDERED".into()].to_vec(),
//     };
//     default_connection_version
// }

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

//     // connect from chain A to chain B

//     // MsgUpdateClient updates the client on the initializing chain A with the latest consensus state of chain B
//     let client_a_udpate = MsgUpdateClient {
//         client_id: CLIENT_B_ID.to_string(),
//         client_message: todo!(),
//         signer: todo!(),
//     };

//     let a_end = ConnectionEnd {
//         client_id: CLIENT_A_ID.to_string(),
//         versions: vec![default_connection_version()],
//         state: State::Init.into(),
//         counterparty: Some(Counterparty {
//             client_id: CLIENT_B_ID.to_string(),
//             connection_id: "connection-1".to_string(),
//             prefix: Some(default_merkle_prefix()),
//         }),
//         delay_period: 0,
//     };

//     let a_to_b_msg_client =
//         protos::ibc::core::connection::v1::msg_client::MsgClient::connect("")
//             .await
//             .unwrap();

//     let b_to_a_msg_client =
//         protos::ibc::core::connection::v1::msg_client::MsgClient::connect("")
//             .await
//             .unwrap();

//     b_to_a_msg_client
//         .connection_open_init(MsgConnectionOpenInit {
//             client_id: todo!(),
//             counterparty: todo!(),
//             version: todo!(),
//             delay_period: todo!(),
//             signer: todo!(),
//         })
//         .await
//         .unwrap();
// }
