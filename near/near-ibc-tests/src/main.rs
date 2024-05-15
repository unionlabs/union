use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use ibc_vm_rs::{
    states::connection_handshake, IbcEvent, Status, DEFAULT_IBC_VERSION, DEFAULT_MERKLE_PREFIX,
};
use near_jsonrpc_client::methods::{self, RpcMethod};
use near_primitives::{
    hash::CryptoHash,
    merkle::{merklize, verify_path, MerklePath, MerklePathItem},
    sharding::{ChunkHash, ChunkHashHeight},
    trie_key::trie_key_parsers,
    types::{BlockReference, StateRoot},
    views::{QueryRequest, StateItem},
};
use near_store::{NibbleSlice, RawTrieNode};
use near_units::parse_near;
use near_workspaces::{
    error::RpcErrorCode,
    network::Sandbox,
    prelude::*,
    result::ValueOrReceiptId,
    rpc::query::ProcessQuery,
    sandbox,
    types::{ChunkHeader, Gas, KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, Worker,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::{
        channel::{self, channel::Channel},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{self, version::Version},
    },
    id::{ChannelId, ConnectionId, PortId},
    validated::ValidateT,
};

const WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_ibc.wasm";
const LC_WASM_FILEPATH: &str = "/home/aeryz/dev/union/union/near_light_client.wasm";
const IBC_APP_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/dummy_ibc_app.wasm";

const CLIENT_TYPE: &str = "cometbls";
const INITIAL_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 100,
};

#[tokio::main]
async fn main() {
    let sandbox = sandbox().await.unwrap();
    let wasm = std::fs::read(WASM_FILEPATH).unwrap();
    let lc_wasm = std::fs::read(LC_WASM_FILEPATH).unwrap();
    let ibc_app_wasm = std::fs::read(IBC_APP_WASM_FILEPATH).unwrap();

    let ibc_account_id: AccountId = String::from("ibc.test.near").try_into().unwrap();
    let ibc_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let contract = sandbox
        .create_tla_and_deploy(ibc_account_id.clone(), ibc_sk.clone(), &wasm)
        .await
        .unwrap()
        .unwrap();
    let lc_account_id: AccountId = String::from("light-client.test.near").try_into().unwrap();
    let lc_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let lc = sandbox
        .create_tla_and_deploy(lc_account_id.clone(), lc_sk.clone(), &lc_wasm)
        .await
        .unwrap()
        .unwrap();
    let counterparty_lc_account_id: AccountId = String::from("counterparty-light-client.test.near")
        .try_into()
        .unwrap();
    let counterparty_lc_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let counterparty_lc = sandbox
        .create_tla_and_deploy(
            counterparty_lc_account_id.clone(),
            counterparty_lc_sk.clone(),
            &lc_wasm,
        )
        .await
        .unwrap()
        .unwrap();
    let ibc_app_account_id: AccountId = String::from("ibc-app.test.near").try_into().unwrap();
    let ibc_app_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let ibc_app = sandbox
        .create_tla_and_deploy(
            ibc_app_account_id.clone(),
            ibc_app_sk.clone(),
            &ibc_app_wasm,
        )
        .await
        .unwrap()
        .unwrap();

    println!("contract id ({:?}), lc id ({:?})", contract.id(), lc.id());

    // create accounts
    let owner = sandbox.root_account().unwrap();
    let user = owner
        .create_subaccount("user")
        .initial_balance(NearToken::from_near(30))
        .transact()
        .await
        .unwrap()
        .into_result()
        .unwrap();

    println!("calling register");

    // let out: Vec<u8> = Sha256::new()
    //     .chain(b"account_idsm")
    //     .chain(borsh::to_vec("cometbls").unwrap())
    //     .finalize()
    //     .to_vec();
    // println!("{:?}", out);

    test_register_client(&user, &contract, &lc).await;
    test_create_client(&sandbox, &user, &contract, counterparty_lc.id()).await;
    let block = sandbox.view_block().await.unwrap();
    test_update_client(
        &user,
        &contract,
        &lc,
        &CryptoHash(block.chunks()[0].prev_state_root.0.clone()),
    )
    .await;

    // let height = block.height();

    // let nodes = proof
    //     .proof
    //     .into_iter()
    //     .map(|bytes| {
    //         let hash = CryptoHash::hash_bytes(&bytes);
    //         let node = near_store::RawTrieNodeWithSize::try_from_slice(&bytes).unwrap();
    //         (hash, node)
    //     })
    //     .collect::<HashMap<_, _>>();

    // println!("Nodes: {:?}", nodes);

    // let next_block = sandbox
    //     .view_block()
    //     .block_height(block.height() + 1)
    //     .await
    //     .unwrap();

    // let root = next_block.chunks()[0].prev_state_root.clone();

    // let (hash, merkle_path) = merklize_chunks(next_block.chunks());

    // println!(
    //     "merklized hash: {:?}, chunk root: {:?}",
    //     hash,
    //     next_block.header().chunk_headers_root()
    // );
    // println!("merkle path: {:?}", merkle_path);

    // println!(
    //     "verify merkle: {}",
    //     verify_path(
    //         hash,
    //         &merkle_path[0],
    //         ChunkHashHeight(
    //             ChunkHash(CryptoHash(next_block.chunks()[0].chunk_hash.0)),
    //             next_block.chunks()[0].height_included
    //         )
    //     )
    // );

    // let res = verify(
    //     nodes,
    //     &CryptoHash(root.0.clone()),
    //     contract.id(),
    //     &out,
    //     Some(&proof.data[&out]),
    // );
    // println!("Res: {}", res);

    // test_create_client(&user, &contract).await;
    // test_open_connection_starting_from_init(&user, &contract).await;
    // test_open_channel_starting_from_init(&user, &contract, &ibc_app).await;
}

fn merklize_chunks(chunks: &[ChunkHeader]) -> (CryptoHash, Vec<MerklePath>) {
    merklize(
        &chunks
            .into_iter()
            .map(|chunk| {
                ChunkHashHeight(
                    ChunkHash(CryptoHash(chunk.chunk_hash.0)),
                    chunk.height_included,
                )
            })
            .collect::<Vec<ChunkHashHeight>>(),
    )
}

fn verify(
    nodes: HashMap<CryptoHash, near_store::RawTrieNodeWithSize>,
    state_root: &StateRoot,
    account_id: &AccountId,
    key: &[u8],
    expected: Option<&[u8]>,
) -> bool {
    let query = trie_key_parsers::get_raw_prefix_for_contract_data(account_id, key);
    let mut key = NibbleSlice::new(&query);

    let mut expected_hash = state_root;
    while let Some(node) = nodes.get(expected_hash) {
        match &node.node {
            RawTrieNode::Leaf(node_key, value) => {
                let nib = &NibbleSlice::from_encoded(&node_key).0;
                return if &key != nib {
                    expected.is_none()
                } else {
                    expected.is_some_and(|expected| value == expected)
                };
            }
            RawTrieNode::Extension(node_key, child_hash) => {
                expected_hash = child_hash;

                // To avoid unnecessary copy
                let nib = NibbleSlice::from_encoded(&node_key).0;
                if !key.starts_with(&nib) {
                    return expected.is_none();
                }
                key = key.mid(nib.len());
            }
            RawTrieNode::BranchNoValue(children) => {
                if key.is_empty() {
                    return expected.is_none();
                }
                match children[key.at(0)] {
                    Some(ref child_hash) => {
                        key = key.mid(1);
                        expected_hash = child_hash;
                    }
                    None => return expected.is_none(),
                }
            }
            RawTrieNode::BranchWithValue(value, children) => {
                if key.is_empty() {
                    return expected.is_some_and(|exp| value == exp);
                }
                match children[key.at(0)] {
                    Some(ref child_hash) => {
                        key = key.mid(1);
                        expected_hash = child_hash;
                    }
                    None => return expected.is_none(),
                }
            }
        }
    }
    false
}

#[derive(serde::Serialize)]
struct RegisterClient {
    client_type: String,
    account: String,
}

#[derive(serde::Serialize)]
struct UpdateClient {
    client_id: String,
    client_msg: Vec<u8>,
}

#[derive(serde::Serialize)]
struct CreateClient {
    client_type: String,
    client_state: Vec<u8>,
    consensus_state: Vec<u8>,
}

#[derive(serde::Serialize)]
struct ConnectionOpenInit {
    client_id: String,
    counterparty: connection_handshake::Counterparty,
    version: Version,
    delay_period: u64,
}

#[derive(serde::Serialize)]
struct ConnectionOpenAck {
    connection_id: String,
    version: Version,
    counterparty_connection_id: String,
    connection_end_proof: Vec<u8>,
    proof_height: Height,
}

#[derive(serde::Serialize)]
struct ChannelOpenInit {
    connection_hops: Vec<ConnectionId>,
    port_id: PortId,
    counterparty: channel::counterparty::Counterparty,
    version: String,
}

#[derive(serde::Serialize)]
struct ChannelOpenAck {
    channel_id: ChannelId,
    port_id: PortId,
    counterparty_channel_id: String,
    counterparty_version: String,
    proof_try: Vec<u8>,
    proof_height: Height,
}

#[derive(serde::Serialize)]
struct GetCommitment {
    key: String,
}

#[derive(serde::Serialize)]
struct GetAccountId {
    client_type: String,
}

/// Expectations:
/// 1. Light client's account id should be saved under the key `client_type`
async fn test_register_client(user: &Account, contract: &Contract, lc: &Contract) {
    let register = RegisterClient {
        client_type: CLIENT_TYPE.into(),
        account: lc.id().to_string(),
    };

    let res = user
        .call(contract.id(), "register_client")
        .args_json(register)
        .transact()
        .await
        .unwrap();

    let account_id: AccountId = serde_json::from_slice(
        user.view(contract.id(), "get_account_id")
            .args_json(GetAccountId {
                client_type: CLIENT_TYPE.into(),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    assert_eq!(&account_id, lc.id());
    println!("[ + ] `test_register_client`: Client successfully registered");
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    latest_height: u64,
    ibc_account_id: AccountId,
}

async fn test_create_client(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    contract: &Contract,
    counterparty_client_id: &AccountId,
) {
    let block = sandbox.view_block().await.unwrap();
    let create = CreateClient {
        client_type: CLIENT_TYPE.into(),
        client_state: borsh::to_vec(&ClientState {
            latest_height: block.height() - 1,
            ibc_account_id: counterparty_client_id.clone(),
        })
        .unwrap(),
        consensus_state: borsh::to_vec(&ConsensusState {
            state_root: CryptoHash(block.chunks()[0].prev_state_root.0.clone()),
        })
        .unwrap(),
    };
    let res = user
        .call(contract.id(), "create_client")
        .args_json(create)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap();

    assert!(res.receipt_failures().is_empty());

    let outcomes = res.receipt_outcomes();

    // receipt for initializing the client
    assert!(matches!(
        outcomes[0].clone().into_result().unwrap(),
        ValueOrReceiptId::ReceiptId(..)
    ));
    // tx result for initializing the client
    assert!(matches!(
        outcomes[1].clone().into_result().unwrap(),
        ValueOrReceiptId::Value(..)
    ));

    // receipt for calling `client.status`
    assert!(matches!(
        outcomes[3].clone().into_result().unwrap(),
        ValueOrReceiptId::ReceiptId(..)
    ));
    // result of `client.status`
    match outcomes[4].clone().into_result().unwrap() {
        ValueOrReceiptId::Value(val) => assert_eq!(val.json::<Status>().unwrap(), Status::Active),
        ValueOrReceiptId::ReceiptId(_) => panic!("expected to get value"),
    }

    // receipt for calling `client.latest_height`
    assert!(matches!(
        outcomes[6].clone().into_result().unwrap(),
        ValueOrReceiptId::ReceiptId(..)
    ));
    // result of `client.latest_height`
    match outcomes[7].clone().into_result().unwrap() {
        ValueOrReceiptId::Value(val) => {
            assert_eq!(
                val.json::<Height>().unwrap().revision_height,
                block.height() - 1
            )
        }
        ValueOrReceiptId::ReceiptId(_) => panic!("expected to get value"),
    }

    assert_eq!(outcomes[9].logs.len(), 1);
    assert_eq!(
        IbcEvent::ClientCreated {
            client_id: format!("{CLIENT_TYPE}-1").validate().unwrap(),
            client_type: CLIENT_TYPE.into(),
            initial_height: block.height() - 1,
        },
        serde_json::from_str(&outcomes[9].logs[0]).unwrap()
    );

    println!("[ + ] `test_create_client`: Client {CLIENT_TYPE}-1 created successfully");
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ConsensusState {
    pub state_root: CryptoHash,
}

async fn test_update_client(
    user: &Account,
    contract: &Contract,
    lc: &Contract,
    state_root: &StateRoot,
) {
    let update = UpdateClient {
        client_id: "cometbls-1".into(),
        client_msg: borsh::to_vec(&ConsensusState {
            state_root: CryptoHash(state_root.0.clone()),
        })
        .unwrap(),
    };

    let res = user
        .call(contract.id(), "update_client")
        .args_json(update)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap();

    println!("Update res: {:?}", res);
}

async fn test_open_connection_starting_from_init(user: &Account, contract: &Contract) {
    let open_init = ConnectionOpenInit {
        client_id: "cometbls-1".into(),
        counterparty: connection_handshake::Counterparty {
            client_id: "08-wasm-0".to_string().validate().unwrap(),
            connection_id: "".into(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        version: DEFAULT_IBC_VERSION[0].clone(),
        delay_period: 0,
    };

    println!("calling connection open init");
    let res = user
        .call(contract.id(), "connection_open_init")
        .args_json(open_init)
        .transact()
        .await
        .unwrap();
    println!("connection open init res: {:?}", res);

    let connection_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: "connections/connection-1".into(),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let connection_end =
        connection_handshake::ConnectionEnd::decode_as::<Proto>(&connection_end_bytes).unwrap();

    assert_eq!(
        connection_handshake::ConnectionEnd {
            client_id: "cometbls-1".to_string().validate().unwrap(),
            versions: DEFAULT_IBC_VERSION.clone(),
            state: connection::state::State::Init,
            counterparty: connection_handshake::Counterparty {
                client_id: "08-wasm-0".to_string().validate().unwrap(),
                connection_id: "".into(),
                prefix: DEFAULT_MERKLE_PREFIX.clone()
            },
            delay_period: 0
        },
        connection_end
    );

    println!("Connection end: {connection_end:?}");

    let open_ack = ConnectionOpenAck {
        connection_id: "connection-1".to_string(),
        version: DEFAULT_IBC_VERSION[0].clone(),
        counterparty_connection_id: "connection-100".to_string(),
        connection_end_proof: vec![1, 2, 3],
        proof_height: Height {
            revision_number: 0,
            revision_height: 120,
        },
    };

    println!("calling connection open ack");
    let res = user
        .call(contract.id(), "connection_open_ack")
        .args_json(open_ack)
        .transact()
        .await
        .unwrap();

    println!("connectionopenack res: {res:?}");

    let connection_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: "connections/connection-1".into(),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let connection_end =
        connection_handshake::ConnectionEnd::decode_as::<Proto>(&connection_end_bytes).unwrap();
    assert_eq!(
        connection_handshake::ConnectionEnd {
            client_id: "cometbls-1".to_string().validate().unwrap(),
            versions: DEFAULT_IBC_VERSION.clone(),
            state: connection::state::State::Open,
            counterparty: connection_handshake::Counterparty {
                client_id: "08-wasm-0".to_string().validate().unwrap(),
                connection_id: "connection-100".into(),
                prefix: DEFAULT_MERKLE_PREFIX.clone()
            },
            delay_period: 0
        },
        connection_end
    );

    println!("[ + ] `test_open_connection_starting_from_init`: Connection opened.");
}

async fn test_open_channel_starting_from_init(
    user: &Account,
    contract: &Contract,
    ibc_app: &Contract,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();
    let channel_init = ChannelOpenInit {
        connection_hops: vec!["connection-1".to_string().validate().unwrap()],
        port_id: port_id.clone(),
        counterparty: channel::counterparty::Counterparty {
            port_id: "transfer".to_string().validate().unwrap(),
            channel_id: "".into(),
        },
        version: "ucs01".into(),
    };

    println!("calling channel open init");
    let res = user
        .call(contract.id(), "channel_open_init")
        .gas(Gas::from_gas(300000000000000))
        .args_json(channel_init)
        .transact()
        .await
        .unwrap();
    println!("channel open init res: {:?}", res);

    let channel_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: format!(
                    "channelEnds/ports/{}/channels/channel-1",
                    port_id.to_string()
                ),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let channel = Channel::decode_as::<Proto>(&channel_end_bytes).unwrap();

    assert_eq!(
        Channel {
            state: channel::state::State::Init,
            ordering: channel::order::Order::Unordered,
            counterparty: channel::counterparty::Counterparty {
                port_id: "transfer".to_string().validate().unwrap(),
                channel_id: "".into()
            },
            connection_hops: vec!["connection-1".to_string().validate().unwrap()],
            version: "ucs01".into()
        },
        channel
    );

    let channel_ack = ChannelOpenAck {
        channel_id: "channel-1".to_string().validate().unwrap(),
        port_id: port_id.clone(),
        counterparty_channel_id: "channel-100".into(),
        counterparty_version: "ucs01".into(),
        proof_try: vec![1, 2, 3],
        proof_height: Height {
            revision_number: 0,
            revision_height: 100,
        },
    };

    println!("calling channel open ack");
    let res = user
        .call(contract.id(), "channel_open_ack")
        .gas(Gas::from_gas(300000000000000))
        .args_json(channel_ack)
        .transact()
        .await
        .unwrap();
    println!("channel open ack res: {:?}", res);

    let channel_end_bytes: Vec<u8> = serde_json::from_slice(
        user.view(contract.id(), "get_commitment")
            .args_json(GetCommitment {
                key: format!(
                    "channelEnds/ports/{}/channels/channel-1",
                    port_id.to_string()
                ),
            })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    let channel = Channel::decode_as::<Proto>(&channel_end_bytes).unwrap();

    assert_eq!(
        Channel {
            state: channel::state::State::Open,
            ordering: channel::order::Order::Unordered,
            counterparty: channel::counterparty::Counterparty {
                port_id: "transfer".to_string().validate().unwrap(),
                channel_id: "channel-100".into()
            },
            connection_hops: vec!["connection-1".to_string().validate().unwrap()],
            version: "ucs01".into()
        },
        channel
    );

    println!("[ + ] - `test_open_channel_starting_from_init`: Channel opened.");
}
