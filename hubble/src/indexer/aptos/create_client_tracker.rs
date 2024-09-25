use std::time::Duration;

use tokio::{task::JoinSet, time::interval};
use tracing::{info, info_span, Instrument};

use super::provider::Provider;
use crate::
    indexer::api::IndexerError
;

pub fn schedule_create_client_checker(
    _pg_pool: sqlx::PgPool,
    join_set: &mut JoinSet<Result<(), IndexerError>>,
    _provider: Provider,
    internal_chain_id: i32,
) {
    join_set.spawn(
        async move {
            let mut interval = interval(Duration::from_secs(10 * 60));

            loop {
                info!("{}: check", internal_chain_id);

                // TODO

                interval.tick().await;
            }
        }
        .instrument(info_span!("clients").or_current()),
    );
}
