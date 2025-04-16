#![allow(clippy::manual_async_fn, clippy::needless_lifetimes)]

use std::time::Duration;

use axum::{routing::get, Router};
use backon::{ConstantBuilder, ExponentialBuilder};
use clap::Parser;
use sqlx::postgres::PgPoolOptions;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

mod cli;
mod github_fetcher;
mod healthz;
mod indexer;
mod logging;
mod metrics;
mod postgres;
mod race_client;
mod token_fetcher;

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
            let label = indexer.label().to_owned();
            // indexer should never return with Ok, thus we log the error.
            indexer.index(db).await.inspect_err(|err| {
                warn!("indexer {label} exited with: {:?}", err);
            })
        });
    });

    let token_fetcher_db = db.clone();
    let token_fetcher = async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
        interval.tick().await;
        loop {
            info!("updating tokens");
            token_fetcher::update_tokens(&token_fetcher_db).await?;
            interval.tick().await;
        }
    };

    set.spawn(token_fetcher);

    let github_fetcher_db = db.clone();
    let github_fetcher = async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await;
        loop {
            info!("updating subscriptions");
            github_fetcher::update_subscriptions(&github_fetcher_db).await?;
            interval.tick().await;
        }
    };

    set.spawn(github_fetcher);

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
        .with_factor(1.25)
}

/// Our 'new block' backoff we use to check if a new block arrived.
pub fn new_block_backoff() -> ConstantBuilder {
    ConstantBuilder::default()
        .with_delay(Duration::from_millis(500))
        .with_max_times(60)
}
