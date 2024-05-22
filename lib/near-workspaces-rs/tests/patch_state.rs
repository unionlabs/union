// Required since `test_log` adds more recursion than the standard recursion limit of 128
#![recursion_limit = "256"]

use near_primitives::borsh::{self, BorshDeserialize, BorshSerialize};
use near_token::NearToken;
use serde_json::json;
use test_log::test;

use near_workspaces::types::{KeyType, SecretKey};
use near_workspaces::{AccessKey, AccountDetailsPatch, AccountId, Contract, DevNetwork, Worker};

const STATUS_MSG_WASM_FILEPATH: &str = "../examples/res/status_message.wasm";

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_primitives::borsh")]
struct Record {
    k: String,
    v: String,
}

#[derive(Clone, Eq, PartialEq, Debug, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_primitives::borsh")]
struct StatusMessage {
    records: Vec<Record>,
}

async fn view_status_state(
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<(AccountId, StatusMessage)> {
    let wasm = std::fs::read(STATUS_MSG_WASM_FILEPATH)?;
    let contract = worker.dev_deploy(&wasm).await.unwrap();

    contract
        .call("set_status")
        .args_json(json!({
            "message": "hello",
        }))
        .transact()
        .await?
        .into_result()?;

    let mut state_items = contract.view_state().await?;
    let state = state_items
        .remove(b"STATE".as_slice())
        .ok_or_else(|| anyhow::anyhow!("Could not retrieve STATE"))?;
    let status_msg: StatusMessage = StatusMessage::try_from_slice(&state)?;

    Ok((contract.id().clone(), status_msg))
}

#[test(tokio::test)]
async fn test_view_state() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (contract_id, status_msg) = view_status_state(&worker).await?;

    assert_eq!(
        status_msg,
        StatusMessage {
            records: vec![Record {
                k: contract_id.to_string(),
                v: "hello".to_string(),
            }]
        }
    );

    Ok(())
}

#[test(tokio::test)]
async fn test_patch_state() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (contract_id, mut status_msg) = view_status_state(&worker).await?;
    status_msg.records.push(Record {
        k: "alice.near".to_string(),
        v: "hello world".to_string(),
    });

    worker
        .patch_state(&contract_id, b"STATE", &borsh::to_vec(&status_msg)?)
        .await?;

    let status: String = worker
        .view(&contract_id, "get_status")
        .args_json(json!({
            "account_id": "alice.near",
        }))
        .await?
        .json()?;

    assert_eq!(status, "hello world".to_string());

    Ok(())
}

#[test(tokio::test)]
async fn test_patch() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (contract_id, mut status_msg) = view_status_state(&worker).await?;
    status_msg.records.push(Record {
        k: "alice.near".to_string(),
        v: "hello world".to_string(),
    });

    worker
        .patch(&contract_id)
        .state(b"STATE", &borsh::to_vec(&status_msg)?)
        .transact()
        .await?;

    let status: String = worker
        .view(&contract_id, "get_status")
        .args_json(json!({
            "account_id": "alice.near",
        }))
        .await?
        .json()?;

    assert_eq!(status, "hello world".to_string());

    Ok(())
}

#[test(tokio::test)]
async fn test_patch_full() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (contract_id, status_msg) = view_status_state(&worker).await?;
    let status_msg_acc = worker.view_account(&contract_id).await?;
    let status_msg_code = worker.view_code(&contract_id).await?;

    let bob_id: AccountId = "bob.test.near".parse()?;
    let sk = SecretKey::from_seed(KeyType::ED25519, "bob's test key");

    // Equivalent to worker.import_contract()
    worker
        .patch(&bob_id)
        .account(
            AccountDetailsPatch::default()
                .balance(NearToken::from_near(100))
                .locked(status_msg_acc.locked)
                .code_hash(status_msg_acc.code_hash)
                .storage_usage(status_msg_acc.storage_usage),
        )
        .access_key(sk.public_key(), AccessKey::full_access())
        .code(&status_msg_code)
        .state(b"STATE", &borsh::to_vec(&status_msg)?)
        .transact()
        .await?;

    let bob_status_msg_acc = Contract::from_secret_key(bob_id, sk, &worker);
    let msg: String = bob_status_msg_acc
        .view("get_status")
        .args_json(json!({
            "account_id": contract_id,
        }))
        .await?
        .json()?;

    // Check that a complete contract got imported over correctly. This should be return "hello"
    // from the original contract `set_status("hello")`
    assert_eq!(msg, "hello".to_string());

    Ok(())
}

#[tokio::test]
async fn test_patch_code_hash() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let (contract_id, _) = view_status_state(&worker).await?;
    let status_msg_acc = worker.view_account(&contract_id).await?;
    let status_msg_code = worker.view_code(&contract_id).await?;

    let bob = worker.dev_create_account().await?;

    // Patching code bytes should also set the code hash, otherwise the node will crash
    // when we try to do anything with the contract.
    worker
        .patch(bob.id())
        .code(&status_msg_code)
        .transact()
        .await?;

    let code_hash = worker.view_account(bob.id()).await?.code_hash;
    assert_eq!(status_msg_acc.code_hash, code_hash);

    Ok(())
}

// account_from_current
#[tokio::test]
async fn test_patch_account_from_current() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;

    let bob = worker.dev_create_account().await?;

    const NEW_BALANCE: NearToken = NearToken::from_yoctonear(10_u128.pow(16));

    let f = |mut acc: near_workspaces::types::AccountDetails| {
        acc.balance = NEW_BALANCE;
        AccountDetailsPatch::from(acc)
    };
    worker
        .patch(bob.id())
        .account_from_current(f)
        .transact()
        .await?;

    let bob_acc = worker.view_account(bob.id()).await?;

    assert_eq!(bob_acc.balance, NEW_BALANCE);

    Ok(())
}
