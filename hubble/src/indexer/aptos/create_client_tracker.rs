use std::{str::FromStr, time::Duration};

use aptos_rest_client::{
    aptos_api_types::{HexEncodedBytes, TransactionPayload},
    Transaction,
};
use serde_json::Value;
use tokio::{task::JoinSet, time::interval};
use tracing::{debug, info, info_span, warn, Instrument};
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
            let mut interval = interval(Duration::from_secs(10 * 60));

            loop {
                info!("check");

                let unmapped_clients = unmapped_clients(&pg_pool, internal_chain_id).await?;
                info!("{}, check: unmapped clients: {}", internal_chain_id, unmapped_clients.len());

                for unmapped_client in unmapped_clients {
                    let height = unmapped_client.height;
                    let version = unmapped_client.version;

                    info!("{}-{}: checking", height, version);

                    let Some(client_id) = unmapped_client.client_id else {
                        debug!("{}-{}: no client id => skipping", height, version);
                        continue;
                    };

                    let tx = provider
                        .get_transaction_by_version(version, None)
                        .await?
                        .response
                        .into_inner();

                    let Transaction::UserTransaction(user_transaction) = tx else {
                        warn!("{}-{}: unexpected transaction type `{:?} => skipping", height, version, tx);
                        continue
                    };

                    let payload = user_transaction.request.payload;

                    let TransactionPayload::EntryFunctionPayload(entry_function_payload) = payload else {
                        warn!("{}-{}: unexpected payload type `{:?} => skipping", height, version, payload);
                        continue
                    };

                    if entry_function_payload.function.name.as_str() != "create_client" {
                        warn!("{}-{}: unexpected function name {} => skipping", height, version, entry_function_payload.function.name.as_str());
                        continue
                    };

                    let [first_argument, second_argument, ..] = &entry_function_payload.arguments[..] else {
                        warn!("{}-{}: expected at least two arguments {} => skipping", height, version, entry_function_payload.arguments.len());
                        continue
                    };

                    if Value::String("cometbls".to_string()) != *first_argument {
                        warn!("{}-{}: expected a 'cometbls' string as first argument {:?} => skipping", height, version, first_argument);
                        continue
                    }

                    let Value::String(argument_as_hex) = second_argument else {
                        warn!("{}-{}: expected a string as second argument {:?} => skipping", height, version, second_argument);
                        continue
                    };

                    let argument_as_bytes = match HexEncodedBytes::from_str(argument_as_hex.as_str()) {
                        Ok(argument_as_bytes) => argument_as_bytes,
                        Err(err) => {
                            warn!("{}-{}: expected hex encoded string as second argument {} => skipping ({:?})", height, version, argument_as_hex, err);
                            continue
                        }
                    };

                    let client_state = match cometbls_light_client_types::client_state::ClientState::decode_as::<Bcs>(argument_as_bytes.inner()) {
                        Ok(client_state) => client_state,
                        Err(err) => {
                            warn!("{}-{}: error decoding client state {}/{:?} => skipping ({:?})", height, version, argument_as_hex, argument_as_bytes, err);
                            continue
                        },
                    };

                    sqlx::query!(
                        r#"
                        INSERT INTO v0.clients (chain_id, client_id, counterparty_chain_id)
                        VALUES ($1, $2, $3)
                        ON CONFLICT DO NOTHING
                        "#,
                        internal_chain_id,
                        client_id,
                        client_state.chain_id.to_string(),
                    )
                    .execute(&pg_pool)
                    .await?;
                }

                interval.tick().await;
            }
        }
        .instrument(info_span!("clients").or_current()),
    );
}
