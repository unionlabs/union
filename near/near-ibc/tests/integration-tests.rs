use near_units::parse_near;
use serde_json::json;
use workspaces::{network::Sandbox, prelude::*, sandbox, Account, Contract, Worker};

const WASM_FILEPATH: &str = "../../../../target/wasm32-unknown-unknown/release/near_ibc.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sandbox = sandbox().await?;
    let wasm = std::fs::read(WASM_FILEPATH)?;
    let contract = sandbox.dev_deploy(&wasm).await?;
    let lc = sandbox.dev_deploy(&wasm).await?;

    // create accounts
    let owner = sandbox.root_account();
    let user = owner
        .create_subaccount(&sandbox, "user")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    test_create_client(&user, &contract, &lc, &sandbox).await?;

    Ok(())
}

async fn test_create_client(
    user: &Account,
    contract: &Contract,
    lc: &Contract,
    sandbox: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    user.call(&sandbox, contract.id(), "register_client")
        .args_json(json!({ "client_type": "wasm", "account": lc.id() }))?
        .transact()
        .await?
        .json()?;
    println!("Increment ✅");
    Ok(())
}
