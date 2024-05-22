use near_workspaces::types::NearToken;
use serde_json::json;
use test_log::test;

use near_workspaces::operations::Function;
use near_workspaces::types::GasMeter;

#[test(tokio::test)]
async fn test_gas_meter_with_single_transaction() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_deploy(include_bytes!("*.wasm")).await?;
    let status_msg = {
        let (id, sk) = worker.dev_generate().await;
        let contract = worker
            .create_tla_and_deploy(
                id.clone(),
                sk,
                include_bytes!("../../examples/res/status_message.wasm"),
            )
            .await?;
        total_gas += contract.details.total_gas_burnt.as_gas();

        contract.into_result()?
    };

    // analogous to: worker.dev_create_account().await?;
    let account = {
        let (id, sk) = worker.dev_generate().await;
        let account = worker.create_tla(id.clone(), sk).await?;
        total_gas += account.details.total_gas_burnt.as_gas();

        account.into_result()?
    };

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_deploy(include_bytes!("*.wasm")).await?;
    let status_msg = {
        let (id, sk) = worker.dev_generate().await;
        let contract = worker
            .create_tla_and_deploy(
                id.clone(),
                sk,
                include_bytes!("../../examples/res/status_message.wasm"),
            )
            .await?;
        total_gas += contract.details.total_gas_burnt.as_gas();

        contract.into_result()?
    };

    // analogous to: worker.dev_create_account().await?;
    let account = {
        let (id, sk) = worker.dev_generate().await;
        let account = worker.create_tla(id.clone(), sk).await?;
        total_gas += account.details.total_gas_burnt.as_gas();

        account.into_result()?
    };

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_parallel_transactions() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_deploy(include_bytes!("*.wasm")).await?;
    let status_msg = {
        let (id, sk) = worker.dev_generate().await;
        let contract = worker
            .create_tla_and_deploy(
                id.clone(),
                sk,
                include_bytes!("../../examples/res/status_message.wasm"),
            )
            .await?;
        total_gas += contract.details.total_gas_burnt.as_gas();

        contract.into_result()?
    };

    // analogous to: worker.dev_create_account().await?;
    let account = {
        let (id, sk) = worker.dev_generate().await;
        let account = worker.create_tla(id.clone(), sk).await?;
        total_gas += account.details.total_gas_burnt.as_gas();

        account.into_result()?
    };

    let mut tasks = Vec::new();

    for _ in 0..10 {
        let account = account.clone();
        let status_msg = status_msg.clone();

        tasks.push(tokio::spawn(async move {
            let txn = account
                .call(status_msg.id(), "set_status")
                .args_json(serde_json::json!({
                    "message": "hello world",
                }))
                .transact()
                .await?;

            Ok::<_, anyhow::Error>(txn.total_gas_burnt)
        }));
    }

    for task in tasks {
        total_gas += task.await??.as_gas();
    }

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_with_multiple_transactions_and_view() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_deploy(include_bytes!("*.wasm")).await?;
    let status_msg = {
        let (id, sk) = worker.dev_generate().await;
        let contract = worker
            .create_tla_and_deploy(
                id.clone(),
                sk,
                include_bytes!("../../examples/res/status_message.wasm"),
            )
            .await?;
        total_gas += contract.details.total_gas_burnt.as_gas();

        contract.into_result()?
    };

    // analogous to: worker.dev_create_account().await?;
    let account = {
        let (id, sk) = worker.dev_generate().await;
        let account = worker.create_tla(id.clone(), sk).await?;
        total_gas += account.details.total_gas_burnt.as_gas();

        account.into_result()?
    };

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    let txn = account
        .call(status_msg.id(), "set_status")
        .args_json(serde_json::json!({
            "message": "hello world",
        }))
        .transact()
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    let _ = account
        .call(status_msg.id(), "get_status")
        .args_json(serde_json::json!({
            "account_id": account.id(),
        }))
        .view()
        .await?;

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_batch_tx() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_deploy(include_bytes!("*.wasm")).await?;
    let contract = {
        let (id, sk) = worker.dev_generate().await;
        let contract = worker
            .create_tla_and_deploy(
                id.clone(),
                sk,
                include_bytes!("../../examples/res/status_message.wasm"),
            )
            .await?;
        total_gas += contract.details.total_gas_burnt.as_gas();

        contract.into_result()?
    };

    let txn = contract
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
        .await?;
    total_gas += txn.total_gas_burnt.as_gas();

    let txn = contract
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
        .await?;

    total_gas += txn.total_gas_burnt.as_gas();

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_gas_meter_create_account_transaction() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    let mut total_gas = 0;

    // analogous to: worker.dev_create_account().await?;
    let account = {
        let (id, sk) = worker.dev_generate().await;
        let account = worker.create_tla(id.clone(), sk).await?;
        total_gas += account.details.total_gas_burnt.as_gas();

        account.into_result()?
    };

    let sub = account.create_subaccount("subaccount").transact().await?;
    total_gas += sub.details.total_gas_burnt.as_gas();

    assert_eq!(total_gas, gas_meter.elapsed().unwrap().as_gas());

    Ok(())
}

#[test(tokio::test)]
async fn test_dropped_gas_meter() -> anyhow::Result<()> {
    let mut worker = near_workspaces::sandbox().await?;
    let gas_meter = GasMeter::now(&mut worker);
    drop(gas_meter);

    worker.dev_create_account().await?;

    Ok(())
}
