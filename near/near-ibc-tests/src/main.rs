mod msgs;
mod utils;

use std::{thread::sleep, time::Duration};

use ibc_vm_rs::{states::connection_handshake, DEFAULT_IBC_VERSION, DEFAULT_MERKLE_PREFIX};
use msgs::ChannelOpenTry;
use near_primitives_core::hash::CryptoHash;
use near_workspaces::{
    network::Sandbox,
    sandbox,
    types::{Gas, KeyType, NearToken, SecretKey},
    Account, AccountId, Contract, Worker,
};
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::{
        channel::{self, channel::Channel},
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection,
    },
    near::types::HeaderUpdate,
    validated::ValidateT,
};
use utils::convert_block_producers;

use crate::{
    msgs::{
        ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ClientState, ConnectionOpenAck,
        ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, ConsensusState, CreateClient,
        GetAccountId, GetCommitment, RegisterClient, UpdateClient,
    },
    utils::{
        chunk_proof, convert_block_header_inner, convert_light_client_block_view, state_proof,
    },
};

const IBC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_ibc.wasm";
const LC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_light_client.wasm";
const IBC_APP_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/dummy_ibc_app.wasm";

mod alice {
    pub const CLIENT_TYPE: &str = "near-alice";
}
mod bob {
    pub const CLIENT_TYPE: &str = "near-bob";
}

const INITIAL_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 100,
};

struct NearContract {
    account_id: AccountId,
    secret_key: SecretKey,
    contract: Contract,
}

pub async fn deploy_contract(
    sandbox: &Worker<Sandbox>,
    account_id: &str,
    wasm_path: &'static str,
) -> Contract {
    let wasm_blob = std::fs::read(wasm_path).unwrap();
    let account_id = account_id.to_string().try_into().unwrap();
    let secret_key = SecretKey::from_seed(KeyType::ED25519, "testificate");
    sandbox
        .create_tla_and_deploy(account_id, secret_key, &wasm_blob)
        .await
        .unwrap()
        .unwrap()
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let sandbox = sandbox().await.unwrap();

    let ibc_contract = deploy_contract(&sandbox, "ibc.test.near", IBC_WASM_FILEPATH).await;
    let alice_lc = deploy_contract(&sandbox, "light-client.test.near", LC_WASM_FILEPATH).await;
    let bob_lc = deploy_contract(
        &sandbox,
        "counterparty-light-client.test.near",
        LC_WASM_FILEPATH,
    )
    .await;
    let ibc_app_contract =
        deploy_contract(&sandbox, "ibc-app.test.near", IBC_APP_WASM_FILEPATH).await;

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

    register_client(
        &user,
        &ibc_contract,
        &alice_lc,
        alice::CLIENT_TYPE.to_string(),
    )
    .await;
    register_client(&user, &ibc_contract, &bob_lc, bob::CLIENT_TYPE.to_string()).await;

    create_client(
        &sandbox,
        &user,
        &ibc_contract,
        alice::CLIENT_TYPE.to_string(),
    )
    .await;
    create_client(&sandbox, &user, &ibc_contract, bob::CLIENT_TYPE.to_string()).await;

    connection_open(&sandbox, &user, &ibc_contract, &alice_lc, &bob_lc).await;
    channel_open(
        &sandbox,
        &user,
        &ibc_contract,
        &ibc_app_contract,
        &alice_lc,
        &bob_lc,
    )
    .await;

    initiate_ping(
        &sandbox,
        &user,
        &ibc_contract,
        &ibc_app_contract,
        "channel-1",
    )
    .await;
}

async fn connection_open_init(
    user: &Account,
    ibc_contract: &Contract,
    client_id: &str,
    counterparty_client_id: &str,
) {
    let open_init = ConnectionOpenInit {
        client_id: client_id.to_string(),
        counterparty: connection_handshake::Counterparty {
            client_id: counterparty_client_id.to_string().validate().unwrap(),
            connection_id: "".into(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        version: DEFAULT_IBC_VERSION[0].clone(),
        delay_period: 0,
    };

    println!("calling connection open init on alice");
    let res = user
        .call(ibc_contract.id(), "connection_open_init")
        .args_json(open_init)
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("connection open init res: {:?}", res);
}

async fn connection_open_try(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    client_id: &str,
    counterparty_client_id: &str,
    counterparty_connection_id: &str,
    proof_height: u64,
) {
    let connection_end_proof = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("connections/{counterparty_connection_id}"),
    )
    .await;

    let open_try = ConnectionOpenTry {
        client_id: client_id.to_string(),
        counterparty: connection_handshake::Counterparty {
            client_id: counterparty_client_id.to_string().validate().unwrap(),
            connection_id: counterparty_connection_id.to_string(),
            prefix: MerklePrefix {
                key_prefix: b"ibc".into(),
            },
        },
        counterparty_versions: DEFAULT_IBC_VERSION.clone(),
        connection_end_proof,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
        delay_period: 0,
    };

    println!("calling connection open try on bob");
    let res = user
        .call(ibc_contract.id(), "connection_open_try")
        .args_json(open_try)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("connection open try res: {:?}", res);
}

async fn connection_open_ack(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    connection_id: &str,
    counterparty_connection_id: &str,
    proof_height: u64,
) {
    let connection_end_proof = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("connections/{counterparty_connection_id}"),
    )
    .await;

    let open_ack = ConnectionOpenAck {
        connection_id: connection_id.to_string(),
        version: DEFAULT_IBC_VERSION[0].clone(),
        counterparty_connection_id: counterparty_connection_id.to_string(),
        connection_end_proof,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
    };

    println!("calling connection open ack on alice");
    let res = user
        .call(ibc_contract.id(), "connection_open_ack")
        .args_json(open_ack)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("connection open ack res: {:?}", res);
}

async fn connection_open_confirm(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    connection_id: &str,
    counterparty_connection_id: &str,
    proof_height: u64,
) {
    let connection_end_proof = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("connections/{counterparty_connection_id}"),
    )
    .await;

    let open_confirm = ConnectionOpenConfirm {
        connection_id: connection_id.to_string(),
        connection_end_proof,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
    };

    println!("calling connection open ack on alice");
    let res = user
        .call(ibc_contract.id(), "connection_open_confirm")
        .args_json(open_confirm)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("connection open confirm res: {:?}", res);
}

async fn wait_until_block_height(sandbox: &Worker<Sandbox>, height: u64) {
    loop {
        let current_height = sandbox.view_block().await.unwrap().height();
        if current_height >= height {
            break;
        }
        sleep(Duration::from_millis(100));
    }
}

async fn update_client(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    client: &Contract,
    client_id: &str,
) -> u64 {
    let latest_height: Height = serde_json::from_slice(
        &user
            .view(client.id(), "latest_height")
            .await
            .unwrap()
            .result,
    )
    .unwrap();

    let current_block = sandbox.view_block().await.unwrap();
    sleep(Duration::from_secs(2));
    // wait_until_block_height(sandbox, current_block.height() + 2).await;

    let light_client_block = sandbox
        .next_light_client_block(current_block.hash())
        .await
        .unwrap();

    let current_height = light_client_block.inner_lite.height;

    let (prev_state_root, prev_state_root_proof) = chunk_proof(sandbox, current_height).await;

    let update = UpdateClient {
        client_id: client_id.to_string(),
        client_msg: borsh::to_vec(&HeaderUpdate {
            new_state: convert_light_client_block_view(light_client_block),
            trusted_height: latest_height.revision_height,
            prev_state_root_proof,
            prev_state_root,
        })
        .unwrap(),
    };

    let res = user
        .call(ibc_contract.id(), "update_client")
        .args_json(update)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("Update result: {res:?}");
    current_height
}

async fn connection_open(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    alice_lc: &Contract,
    bob_lc: &Contract,
) {
    let alice_client_id = format!("{}-1", alice::CLIENT_TYPE);
    let bob_client_id = format!("{}-2", bob::CLIENT_TYPE);
    connection_open_init(user, ibc_contract, &alice_client_id, &bob_client_id).await;

    let current_height = update_client(sandbox, user, ibc_contract, bob_lc, &bob_client_id).await;

    connection_open_try(
        sandbox,
        user,
        ibc_contract,
        &bob_client_id,
        &alice_client_id,
        "connection-1",
        current_height - 1,
    )
    .await;

    let current_height =
        update_client(sandbox, user, ibc_contract, alice_lc, &alice_client_id).await;

    connection_open_ack(
        sandbox,
        user,
        ibc_contract,
        "connection-1",
        "connection-2",
        current_height - 1,
    )
    .await;

    let current_height = update_client(sandbox, user, ibc_contract, bob_lc, &bob_client_id).await;

    connection_open_confirm(
        sandbox,
        user,
        ibc_contract,
        "connection-2",
        "connection-1",
        current_height - 1,
    )
    .await;

    println!("[ + ] Connection opened between {alice_client_id} and {bob_client_id}");
}

async fn channel_open_init(
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    connection_id: &str,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();
    let channel_init = ChannelOpenInit {
        connection_hops: vec![connection_id.to_string().validate().unwrap()],
        port_id: port_id.clone(),
        counterparty: channel::counterparty::Counterparty {
            port_id,
            channel_id: "".into(),
        },
        version: "ucs01".into(),
    };

    println!("calling channel open init");
    let res = user
        .call(ibc_contract.id(), "channel_open_init")
        .gas(Gas::from_gas(300000000000000))
        .args_json(channel_init)
        .transact()
        .await
        .unwrap()
        .unwrap();
    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("channel open init res: {:?}", res);
}

async fn channel_open_try(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    connection_id: &str,
    counterparty_channel_id: &str,
    proof_height: u64,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();

    let proof_init = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("channelEnds/ports/{port_id}/channels/{counterparty_channel_id}"),
    )
    .await;

    let open_try = ChannelOpenTry {
        connection_hops: vec![connection_id.to_string().validate().unwrap()],
        port_id: port_id.clone(),
        counterparty: channel::counterparty::Counterparty {
            port_id,
            channel_id: counterparty_channel_id.to_string(),
        },
        counterparty_version: "ucs01".to_string(),
        version: "ucs01".to_string(),
        proof_init,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
    };

    println!("calling channel open try");
    let res = user
        .call(ibc_contract.id(), "channel_open_try")
        .gas(Gas::from_gas(300000000000000))
        .args_json(open_try)
        .transact()
        .await
        .unwrap()
        .unwrap();
    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("channel open try res: {:?}", res);
}

async fn channel_open_ack(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    channel_id: &str,
    counterparty_channel_id: &str,
    proof_height: u64,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();

    let proof_try = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("channelEnds/ports/{port_id}/channels/{counterparty_channel_id}"),
    )
    .await;

    let open_try = ChannelOpenAck {
        channel_id: channel_id.to_string().validate().unwrap(),
        port_id,
        counterparty_channel_id: counterparty_channel_id.to_string(),
        counterparty_version: "ucs01".to_string(),
        proof_try,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
    };

    println!("calling channel open ack");
    let res = user
        .call(ibc_contract.id(), "channel_open_ack")
        .gas(Gas::from_gas(300000000000000))
        .args_json(open_try)
        .transact()
        .await
        .unwrap()
        .unwrap();
    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("channel open ack res: {:?}", res);
}

async fn channel_open_confirm(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    channel_id: &str,
    counterparty_channel_id: &str,
    proof_height: u64,
) {
    let port_id = ibc_app.id().to_string().validate().unwrap();

    let proof_ack = state_proof(
        sandbox,
        ibc_contract.id(),
        proof_height,
        &format!("channelEnds/ports/{port_id}/channels/{counterparty_channel_id}"),
    )
    .await;

    let open_confirm = ChannelOpenConfirm {
        channel_id: channel_id.to_string().validate().unwrap(),
        port_id,
        proof_ack,
        proof_height: Height {
            revision_number: 0,
            revision_height: proof_height,
        },
    };

    println!("calling channel open confirm");
    let res = user
        .call(ibc_contract.id(), "channel_open_confirm")
        .gas(Gas::from_gas(300000000000000))
        .args_json(open_confirm)
        .transact()
        .await
        .unwrap()
        .unwrap();

    assert!(res.receipt_failures().is_empty() && res.failures().is_empty());
    println!("channel open confirm res: {:?}", res);
}

async fn channel_open(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    alice_lc: &Contract,
    bob_lc: &Contract,
) {
    let alice_client_id = format!("{}-1", alice::CLIENT_TYPE);
    let bob_client_id = format!("{}-2", bob::CLIENT_TYPE);
    channel_open_init(user, ibc_contract, ibc_app, "connection-1").await;

    let current_height = update_client(sandbox, user, ibc_contract, bob_lc, &bob_client_id).await;

    channel_open_try(
        sandbox,
        user,
        ibc_contract,
        ibc_app,
        "connection-2",
        "channel-1",
        current_height - 1,
    )
    .await;

    let current_height =
        update_client(sandbox, user, ibc_contract, alice_lc, &alice_client_id).await;

    channel_open_ack(
        sandbox,
        user,
        ibc_contract,
        ibc_app,
        "channel-1",
        "channel-2",
        current_height - 1,
    )
    .await;

    let current_height = update_client(sandbox, user, ibc_contract, bob_lc, &bob_client_id).await;

    channel_open_confirm(
        sandbox,
        user,
        ibc_contract,
        ibc_app,
        "channel-2",
        "channel-1",
        current_height - 1,
    )
    .await;

    println!("[ + ] - `channel_open`: Channel opened.");
}

/// Expectations:
/// 1. Light client's account id should be saved under the key `client_type`
async fn register_client(user: &Account, contract: &Contract, lc: &Contract, client_type: String) {
    let register = RegisterClient {
        client_type: client_type.clone(),
        account: lc.id().to_string(),
    };

    let res = user
        .call(contract.id(), "register_client")
        .args_json(register)
        .transact()
        .await
        .unwrap();
    println!("res: {:?}", res);

    let account_id: AccountId = serde_json::from_slice(
        user.view(contract.id(), "get_account_id")
            .args_json(GetAccountId { client_type })
            .await
            .unwrap()
            .result
            .as_slice(),
    )
    .unwrap();

    assert_eq!(&account_id, lc.id());
    println!("[ + ] `register_client`: Client successfully registered");
}

async fn create_client(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    client_type: String,
) {
    let current_block = sandbox.view_block().await.unwrap();
    sleep(Duration::from_secs(2));
    let lc_block = sandbox
        .next_light_client_block(current_block.hash())
        .await
        .unwrap();
    let height = lc_block.inner_lite.height;
    let create = CreateClient {
        client_type: client_type.clone(),
        client_state: borsh::to_vec(&ClientState {
            latest_height: height - 1,
            ibc_account_id: ibc_contract.id().clone(),
            // TODO(aeryz): this is only valid in this sandboxed environment where the validator set is not changing. For a real environment,
            // the relayer must read the block producers using another endpoint.
            initial_block_producers: lc_block.next_bps.map(convert_block_producers),
        })
        .unwrap(),
        consensus_state: borsh::to_vec(&ConsensusState {
            state: convert_block_header_inner(lc_block.inner_lite),
            chunk_prev_state_root: CryptoHash(
                sandbox
                    .view_block()
                    .block_height(height)
                    .await
                    .unwrap()
                    .chunks()[0]
                    .prev_state_root
                    .0,
            ),
        })
        .unwrap(),
    };
    let res = user
        .call(ibc_contract.id(), "create_client")
        .args_json(create)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap();

    assert!(res.receipt_failures().is_empty());
    println!("[ + ] `create_client`: Client of type {client_type} created successfully");
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Ping {
    ibc_addr: AccountId,
    source_channel: String,
}

pub async fn initiate_ping(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    ibc_contract: &Contract,
    ibc_app: &Contract,
    source_channel: &str,
) {
    let ping = Ping {
        ibc_addr: ibc_contract.id().clone(),
        source_channel: source_channel.to_string(),
    };

    let res = user
        .call(ibc_app.id(), "ping")
        .args_json(ping)
        .gas(Gas::from_gas(300000000000000))
        .transact()
        .await
        .unwrap()
        .unwrap();

    println!("{res:?}");
}
