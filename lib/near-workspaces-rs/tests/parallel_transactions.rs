use std::{collections::VecDeque, task::Poll};

use serde_json::json;

const STATUS_MSG_CONTRACT: &[u8] = include_bytes!("../../examples/res/status_message.wasm");

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_parallel() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    let parallel_tasks = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
        .iter()
        .map(|msg| {
            let id = contract.id().clone();
            let account = account.clone();

            tokio::spawn(async move {
                account
                    .call(&id, "set_status")
                    .args_json(json!({
                        "message": msg.to_string(),
                    }))
                    .transact()
                    .await?
                    .into_result()?;
                anyhow::Result::<()>::Ok(())
            })
        });
    futures::future::join_all(parallel_tasks).await;

    // Check the final set message. This should be random each time this test function is called:
    let final_set_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;
    println!("Final set message: {:?}", final_set_msg);

    Ok(())
}

#[tokio::test]
async fn test_parallel_async() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(STATUS_MSG_CONTRACT).await?;
    let account = worker.dev_create_account().await?;

    // nonce of access key before any transactions occurred.
    let nonce_start = worker
        .view_access_key(account.id(), &account.secret_key().public_key())
        .await?
        .nonce;

    // Create a queue statuses we can check the status of later.
    let mut statuses = VecDeque::new();
    let messages = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
    for msg in messages {
        let status = account
            .call(contract.id(), "set_status")
            .args_json(json!({
                "message": msg,
            }))
            .transact_async()
            .await?;
        statuses.push_back(status);
    }

    // Retry checking the statuses of all transactions until the queue is empty
    // with all transactions completed.
    while let Some(status) = statuses.pop_front() {
        match status.status().await? {
            Poll::Ready(_) => (),
            Poll::Pending => statuses.push_back(status),
        }
    }

    // Check the final set message. This should be "j" due to the ordering of the queue.
    let final_set_msg = account
        .call(contract.id(), "get_status")
        .args_json(json!({ "account_id": account.id() }))
        .view()
        .await?
        .json::<String>()?;
    assert_eq!(final_set_msg, "j");

    let nonce_end = worker
        .view_access_key(account.id(), &account.secret_key().public_key())
        .await?
        .nonce;

    // The amount of transactions should equal the increase in nonce:
    assert!(nonce_end - nonce_start == messages.len() as u64);

    Ok(())
}
