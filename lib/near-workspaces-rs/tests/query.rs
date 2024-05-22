use near_workspaces::types::NearToken;
use near_workspaces::{network::Sandbox, Contract, Worker};

async fn init() -> anyhow::Result<(Worker<Sandbox>, Contract)> {
    let worker = near_workspaces::sandbox().await?;
    let status_msg = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;

    Ok((worker, status_msg))
}

#[tokio::test]
async fn test_invalid_query() -> anyhow::Result<()> {
    let (_worker, contract) = init().await?;

    // incorrect method name;
    let result = contract.view("view_status").args_json(("some_id",)).await;
    let error =
        result.expect_err("expected error while calling invalid method `status_msg.view_status`");
    assert!(format!("{error:?}").contains("MethodNotFound"));

    // incorrect args:
    let result = contract
        .view("get_status")
        .args_json(serde_json::json!({
            "account_id": 10,
        }))
        .await;
    let error =
        result.expect_err("expected error while passing invalid args to `status_msg.get_status`");
    assert!(format!("{error:?}").contains("Failed to deserialize input from JSON."));

    // deposit supplied when not required:
    let result = contract
        .call("set_status")
        .args_json(("some message",))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?
        .into_result();
    let error =
        result.expect_err("expected error while passing deposit to `status_msg.set_status`");
    assert!(format!("{error:?}").contains("Smart contract panicked: Method doesn't accept deposit"));

    Ok(())
}
