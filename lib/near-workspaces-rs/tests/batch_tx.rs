use near_workspaces::operations::Function;
use near_workspaces::types::NearToken;
use serde_json::json;
use test_log::test;

#[test(tokio::test)]
async fn test_batch_tx() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let contract = worker
        .dev_deploy(include_bytes!("../../examples/res/status_message.wasm"))
        .await?;

    // Batch transaction with two `call`s into `set_status`. The second one
    // should override the first one.
    contract
        .batch()
        .call(
            Function::new("set_status")
                .args_json(json!({
                    "message": "hello_world",
                }))
                .deposit(NearToken::from_near(0)),
        )
        .call(Function::new("set_status").args_json(json!({
            "message": "world_hello",
        })))
        .transact()
        .await?
        .into_result()?;

    let status_msg: String = contract
        .call("get_status")
        .args_json(serde_json::json!({
            "account_id": contract.id(),
        }))
        .view()
        .await?
        .json()?;

    assert_eq!(status_msg, "world_hello");
    Ok(())
}
