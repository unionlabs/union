#![feature(more_qualified_paths)]
#![feature(try_blocks)]
#![allow(clippy::manual_async_fn, clippy::needless_lifetimes)]

use std::time::Duration;

use axum::{routing::get, Router};
use backon::ExponentialBuilder;
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

mod chain_id_query;
mod cli;
mod eth;
mod healthz;
mod logging;
mod metrics;
mod postgres;
mod tm;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install().unwrap();
    let args = crate::cli::Args::parse();

    crate::logging::init(args.log_format);
    metrics::register_custom_metrics();

    let db = PgPoolOptions::new()
        .max_connections(40)
        .connect(&args.database_url.unwrap())
        .await?;

    let mut set = JoinSet::new();

    if let Some(addr) = args.metrics_addr {
        set.spawn(async move {
            let app = Router::new()
                .route("/metrics", get(metrics::handler))
                .route("/healthz", get(healthz::handler));
            axum::Server::bind(&addr)
                .serve(app.into_make_service())
                .await
                .map_err(Into::into)
        });
    }

    args.indexers.clone().into_iter().for_each(|indexer| {
        let db: sqlx::Pool<sqlx::Postgres> = db.clone();
        set.spawn(async move {
            info!("starting indexer {:?}", indexer);
            // indexer should never return with Ok, thus we log the error.
            indexer.index(db).await.inspect_err(|err| {
                warn!("indexer exited with: {:?}", err);
            })
        });
    });

    let indexers = args.indexers.clone();

    let client_updates = async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));

        loop {
            info!("fetching new client counterparty_chain_ids");
            chain_id_query::tx(db.clone(), indexers.clone()).await;
            interval.tick().await;
        }
    };

    set.spawn(client_updates);

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Err(err)) => {
                error!(
                    "encountered error while indexing: {:?}. shutting down.",
                    err
                );
                info!("shutdown - setting unhealthy");
                healthz::set_unhealthy();
                info!("shutdown - shutting down");
                let shutdown_hook = set.shutdown();
                info!("shutdown - awaiting shutdown");
                shutdown_hook.await;
                info!("shutdown - returning");
                return Err(err);
            }
            Err(err) => return Err(err.into()),
            Ok(Ok(_)) => {
                info!("indexer exited gracefully");
            }
        }
    }
    Ok(())
}

/// Our ExponentialBackoff that we use everywhere.
pub fn expo_backoff() -> ExponentialBuilder {
    ExponentialBuilder::default()
        .with_min_delay(Duration::from_secs(2))
        .with_max_delay(Duration::from_secs(60))
        .with_max_times(60)
}
