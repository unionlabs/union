#![recursion_limit = "256"]
use serde::{Deserialize, Serialize};
use test_log::test;

use near_workspaces::network::{Sandbox, ValidatorKey};
use near_workspaces::{pick_unused_port, Worker};

const NFT_WASM_FILEPATH: &str = "../examples/res/non_fungible_token.wasm";
const EXPECTED_NFT_METADATA: &str = r#"{
  "spec": "nft-1.0.0",
  "name": "Example NEAR non-fungible token",
  "symbol": "EXAMPLE",
  "icon": "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E",
  "base_uri": null,
  "reference": null,
  "reference_hash": null
}"#;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct NftMetadata {
    spec: String,
    name: String,
    symbol: String,
    icon: String,
    base_uri: Option<String>,
    reference: Option<String>,
    reference_hash: Option<String>,
}

fn expected() -> NftMetadata {
    serde_json::from_str(EXPECTED_NFT_METADATA).unwrap()
}

async fn deploy_and_assert(worker: Worker<Sandbox>) -> anyhow::Result<()> {
    let wasm = std::fs::read(NFT_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await?;

    contract
        .call("new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": contract.id()
        }))
        .transact()
        .await?
        .into_result()?;

    let actual: NftMetadata = contract.view("nft_metadata").await?.json()?;

    assert_eq!(actual, expected());
    Ok(())
}

#[test(tokio::test)]
async fn test_dev_deploy() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    deploy_and_assert(worker).await?;
    Ok(())
}

#[test(tokio::test)]
async fn test_manually_spawned_deploy() -> anyhow::Result<()> {
    let rpc_port = pick_unused_port().await?;
    let net_port = pick_unused_port().await?;
    let mut home_dir = std::env::temp_dir();
    home_dir.push(format!("test-sandbox-{}", rpc_port));

    // initialize chain data with supplied home dir
    let output = near_sandbox_utils::init(&home_dir)?
        .wait_with_output()
        .await
        .unwrap();
    tracing::info!(target: "workspaces-test", "sandbox-init: {:?}", output);

    let mut child = near_sandbox_utils::run(&home_dir, rpc_port, net_port)?;

    // connect to local sandbox node
    let worker = near_workspaces::sandbox()
        .rpc_addr(&format!("http://localhost:{}", rpc_port))
        .validator_key(ValidatorKey::HomeDir(home_dir))
        .await?;
    deploy_and_assert(worker).await?;

    child.kill().await?;
    Ok(())
}
