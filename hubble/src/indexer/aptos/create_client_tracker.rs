use std::{str::FromStr, time::Duration};

use aptos_rest_client::{
    aptos_api_types::{HexEncodedBytes, TransactionPayload},
    Transaction,
};
use serde_json::Value;
use tokio::task::JoinSet;
use tracing::{debug, info, warn, Instrument};
use unionlabs::encoding::{Bcs, DecodeAs};

use crate::indexer::{
    api::IndexerError,
    aptos::{postgres::unmapped_clients, provider::Provider},
};

pub fn schedule_create_client_checker(
    pg_pool: sqlx::PgPool,
    join_set: &mut JoinSet<Result<(), IndexerError>>,
    provider: Provider,
    internal_chain_id: i32,
) {
    join_set.spawn(
        async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));

            loop {
                match process_unmapped_clients(&pg_pool, &provider, internal_chain_id).await {
                    Ok(_) => info!("Client processing completed"),
                    Err(err) => warn!(error = ?err, "Error during client processing"),
                }

                interval.tick().await;
            }
        }
        .instrument(tracing::info_span!("clients").or_current()),
    );
}

async fn process_unmapped_clients(
    pg_pool: &sqlx::PgPool,
    provider: &Provider,
    internal_chain_id: i32,
) -> Result<(), IndexerError> {
    let unmapped_clients = unmapped_clients(pg_pool, internal_chain_id).await?;
    info!(
        internal_chain_id,
        unmapped_clients = unmapped_clients.len(),
        "Fetched unmapped clients"
    );

    for client in unmapped_clients {
        if let Err(err) = process_single_client(client, provider, pg_pool, internal_chain_id).await {
            warn!(error = ?err, "Error processing single client");
        }
    }

    Ok(())
}

async fn process_single_client(
    client: UnmappedClient,
    provider: &Provider,
    pg_pool: &sqlx::PgPool,
    internal_chain_id: i32,
) -> Result<(), IndexerError> {
    let height = client.height;
    let version = client.version;

    info!(height, version, "Processing client");

    let client_id = match &client.client_id {
        Some(id) => id,
        None => {
            debug!(height, version, "No client ID found, skipping");
            return Ok(());
        }
    };

    let tx = provider.get_transaction_by_version(version, None).await?.response.into_inner();
    let user_transaction = match tx {
        Transaction::UserTransaction(tx) => tx,
        _ => {
            warn!(height, version, "Unexpected transaction type, skipping");
            return Ok(());
        }
    };

    let payload = match user_transaction.request.payload {
        TransactionPayload::EntryFunctionPayload(payload) => payload,
        _ => {
            warn!(height, version, "Unexpected payload type, skipping");
            return Ok(());
        }
    };

    if payload.function.name.as_str() != "create_client" {
        warn!(height, version, function = %payload.function.name, "Unexpected function name, skipping");
        return Ok(());
    }

    let arguments = payload.arguments;
    if arguments.len() < 2 {
        warn!(height, version, arguments_count = arguments.len(), "Insufficient arguments, skipping");
        return Ok(());
    }

    let first_argument = &arguments[0];
    if *first_argument != Value::String("cometbls".to_string()) {
        warn!(height, version, first_argument = ?first_argument, "Unexpected first argument, skipping");
        return Ok(());
    }

    let argument_as_hex = match &arguments[1] {
        Value::String(hex) => hex,
        other => {
            warn!(height, version, second_argument = ?other, "Second argument is not a string, skipping");
            return Ok(());
        }
    };

    let argument_as_bytes = HexEncodedBytes::from_str(argument_as_hex).map_err(|err| {
        warn!(
            height,
            version,
            second_argument = %argument_as_hex,
            error = ?err,
            "Failed to decode second argument"
        );
        IndexerError::DecodeError(err.to_string())
    })?;

    let client_state = cometbls_light_client_types::client_state::ClientState::decode_as::<Bcs>(
        argument_as_bytes.inner(),
    )
    .map_err(|err| {
        warn!(
            height,
            version,
            second_argument = %argument_as_hex,
            error = ?err,
            "Failed to decode client state"
        );
        IndexerError::DecodeError(err.to_string())
    })?;

    insert_client(pg_pool, internal_chain_id, client_id, &client_state).await?;

    Ok(())
}

async fn insert_client(
    pg_pool: &sqlx::PgPool,
    chain_id: i32,
    client_id: &str,
    client_state: &cometbls_light_client_types::client_state::ClientState,
) -> Result<(), IndexerError> {
    sqlx::query!(
        r#"
        INSERT INTO hubble.clients (chain_id, client_id, counterparty_chain_id)
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING
        "#,
        chain_id,
        client_id,
        client_state.chain_id.to_string(),
    )
    .execute(pg_pool)
    .await?;

    Ok(())
}
