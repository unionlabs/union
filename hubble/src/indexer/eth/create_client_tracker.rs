use std::{str::FromStr, time::Duration};

use alloy::primitives::FixedBytes;
use tokio::{task::JoinSet, time::interval};
use tracing::{debug, info, info_span, warn, Instrument};
use unionlabs::{
    encoding::{DecodeAs, EthAbi},
    ibc::lightclients::cometbls::client_state::ClientState,
};

use crate::{
    chain_id_query::IbcHandler,
    indexer::{
        api::IndexerError,
        eth::{postgres::unmapped_clients, provider::Provider},
    },
};

pub fn schedule_create_client_checker(
    pg_pool: sqlx::PgPool,
    join_set: &mut JoinSet<Result<(), IndexerError>>,
    provider: Provider,
    internal_chain_id: i32,
) {
    join_set.spawn(async move {
        let mut interval = interval(Duration::from_secs(10 * 60));

        loop {
            info!("check");

            let unmapped_clients = unmapped_clients(&pg_pool, internal_chain_id).await?;
            info!("{}, check: unmapped clients: {}", internal_chain_id, unmapped_clients.len());

            for unmapped_client in unmapped_clients {
                let height = unmapped_client.height;
                let transaction_hash = unmapped_client.transaction_hash;

                info!("{}-{}: checking", height, transaction_hash);

                let Some(client_id) = unmapped_client.client_id else {
                    debug!("{}-{}: no client id => skipping", height, transaction_hash);
                    continue;
                };

                let tx = provider
                    .get_transaction_by_hash(FixedBytes::from_str(&transaction_hash).expect("valid transaction hash"), None)
                    .await?
                    .response
                    .expect("transaction");

                let msg = match <IbcHandler::CreateClientCall as alloy::sol_types::SolCall>::abi_decode(&tx.input,true) {
                    Ok(msg) => msg,
                    Err(err) => {
                        warn!("{}-{}: cannot decode, most likely due to ABI change: {} => skipping", height, transaction_hash, err);
                        continue
                    }
                };

                match &*msg._0.client_type {
                    "cometbls" => {
                        let cs = ClientState::decode_as::<EthAbi>(&msg._0.client_state_bytes).unwrap();

                        sqlx::query!(
                            r#"
                            INSERT INTO v0.clients (chain_id, client_id, counterparty_chain_id)
                            VALUES ($1, $2, $3)
                            ON CONFLICT DO NOTHING
                            "#,
                            internal_chain_id,
                            client_id,
                            cs.chain_id.to_string(),
                        )
                        .execute(&pg_pool)
                        .await?;
                    }
                    ty => {
                        warn!("{}-{}: unknown evm client type `{} => skipping", height, transaction_hash, ty);
                        continue
                    }
                }
            }

            interval.tick().await;
        }
    }.instrument(info_span!("clients")));
}
