use ibc_vm_rs::{IbcEvent, Status, DEFAULT_IBC_VERSION};
use near_units::parse_near;
use serde_json::json;
use unionlabs::{
    encoding::{DecodeAs, Proto},
    ibc::core::{
        client::height::Height,
        commitment::merkle_prefix::MerklePrefix,
        connection::{connection_end::ConnectionEnd, counterparty::Counterparty, version::Version},
    },
};
use workspaces::{
    network::Sandbox,
    prelude::*,
    result::ValueOrReceiptId,
    sandbox,
    types::{KeyType, SecretKey},
    Account, AccountId, Contract, Worker,
};

const WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_ibc.wasm";
const LC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/dummy_light_client.wasm";

const CLIENT_TYPE: &str = "union";
const INITIAL_HEIGHT: Height = Height {
    revision_number: 0,
    revision_height: 100,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("herererere0??");
    let sandbox = sandbox().await?;
    println!("herererere1??");
    let wasm = std::fs::read(WASM_FILEPATH)?;
    let lc_wasm = std::fs::read(LC_WASM_FILEPATH)?;
    println!("herererere2??");

    let ibc_account_id: AccountId = String::from("ibc.test.near").try_into()?;
    let ibc_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let contract = sandbox
        .create_tla_and_deploy(ibc_account_id.clone(), ibc_sk.clone(), &wasm)
        .await?
        .unwrap();
    println!("herererere3??");
    let lc_account_id: AccountId = String::from("light-client.test.near").try_into()?;
    let lc_sk = SecretKey::from_seed(KeyType::ED25519, "testificate");
    let lc = sandbox
        .create_tla_and_deploy(lc_account_id.clone(), lc_sk.clone(), &lc_wasm)
        .await?
        .unwrap();
    println!("herererere4??");

    println!("contract id ({:?}), lc id ({:?})", contract.id(), lc.id());

    // create accounts
    let owner = sandbox.root_account().unwrap();
    let user = owner
        .create_subaccount("user")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    println!("calling register");

    test_register_client(&user, &contract, &lc).await;
    test_create_client(&user, &contract, &lc).await;

    // test_open_connection_starting_from_init(&sandbox, &user, &contract, &lc).await?;

    Ok(())
}

#[derive(serde::Serialize)]
struct RegisterClient {
    client_type: String,
    account: String,
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
    counterparty: Counterparty<String, String>,
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

    let _ = user
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

async fn test_create_client(user: &Account, contract: &Contract, lc: &Contract) {
    let create = CreateClient {
        client_type: CLIENT_TYPE.into(),
        client_state: vec![1, 2, 3],
        consensus_state: vec![4, 5, 6],
    };
    let res = user
        .call(contract.id(), "create_client")
        .args_json(create)
        .gas(300000000000000)
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
    // result of `client.latest_height`
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
        ValueOrReceiptId::Value(val) => assert_eq!(val.json::<Height>().unwrap(), INITIAL_HEIGHT),
        ValueOrReceiptId::ReceiptId(_) => panic!("expected to get value"),
    }

    assert_eq!(outcomes[9].logs.len(), 1);
    assert_eq!(
        IbcEvent::ClientCreated {
            client_id: format!("{CLIENT_TYPE}-1"),
            client_type: CLIENT_TYPE.into(),
            initial_height: INITIAL_HEIGHT.revision_height
        },
        serde_json::from_str(&outcomes[9].logs[0]).unwrap()
    );

    println!("[ + ] `test_create_client`: Client {CLIENT_TYPE}-1 created successfully");
}

async fn test_open_connection_starting_from_init(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    contract: &Contract,
    lc: &Contract,
) -> anyhow::Result<()> {
    let open_init = ConnectionOpenInit {
        client_id: "wasm-1".into(),
        counterparty: Counterparty {
            client_id: "cometbls-0".into(),
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
            .await?
            .result
            .as_slice(),
    )
    .unwrap();

    let connection_end =
        ConnectionEnd::<String, String, String>::decode_as::<Proto>(&connection_end_bytes).unwrap();

    println!("Connection end: {connection_end:?}");

    Ok(())
}
