use ibc_vm_rs::DEFAULT_IBC_VERSION;
use near_units::parse_near;
use serde_json::json;
use unionlabs::ibc::core::{
    client::height::Height,
    commitment::merkle_prefix::MerklePrefix,
    connection::{counterparty::Counterparty, version::Version},
};
use workspaces::{
    network::Sandbox,
    prelude::*,
    sandbox,
    types::{KeyType, SecretKey},
    Account, AccountId, Contract, Worker,
};

const WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/near_ibc.wasm";
const LC_WASM_FILEPATH: &str =
    "/home/aeryz/dev/union/union/target/wasm32-unknown-unknown/release/dummy_light_client.wasm";

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
    test_create_client(&sandbox, &user, &contract, &lc).await?;
    test_open_connection_starting_from_init(&sandbox, &user, &contract, &lc).await?;

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

async fn test_create_client(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    contract: &Contract,
    lc: &Contract,
) -> anyhow::Result<()> {
    let register = RegisterClient {
        client_type: "wasm".into(),
        account: lc.id().to_string(),
    };
    let res = user
        .call(contract.id(), "register_client")
        .args_json(register)
        .transact()
        .await
        .unwrap();

    println!("Register result {res:?}");
    println!("moving a block");
    sandbox.fast_forward(1).await.unwrap();
    println!("moved");

    let create = CreateClient {
        client_type: "wasm".into(),
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
    println!("Create result {res:?}");

    Ok(())
}

async fn test_open_connection_starting_from_init(
    sandbox: &Worker<Sandbox>,
    user: &Account,
    contract: &Contract,
    lc: &Contract,
) -> anyhow::Result<()> {
    let open_init = ConnectionOpenInit {
        client_id: "wasm-0".into(),
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
    let res = user
        .call(contract.id(), "connection_open_init")
        .args_json(open_init)
        .transact()
        .await
        .unwrap();

    println!("Register result {res:?}");
    println!("moving a block");
    sandbox.fast_forward(1).await.unwrap();
    println!("moved");

    let create = CreateClient {
        client_type: "wasm".into(),
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
    println!("Create result {res:?}");

    Ok(())
}
