#![feature(result_option_inspect)]
#![feature(more_qualified_paths)]

use axum::{routing::get, Router};
use clap::Parser;
use hubble::datastore::{either::Either, hasura::HasuraDataStore, postgres::PostgresDatastore};
use reqwest::Client;
use sqlx::PgPool;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

mod cli;

mod eth;
mod healthz;
mod metrics;
mod tm;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install().unwrap();
    let args = crate::cli::Args::parse();
    tracing_subscriber::fmt().with_ansi(false).init();
    metrics::register_custom_metrics();

    let db = PgPool::connect_lazy(&args.database_url.unwrap())?;
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

    args.indexers.into_iter().for_each(|indexer| {
        let db = db.clone();
        set.spawn(async move {
            info!("starting indexer {:?}", indexer);
            // indexer should never return with Ok, thus we log the error.
            indexer.index(db).await.inspect_err(|err| {
                warn!("indexer {:?} exited with: {:?}", &indexer, err);
            })
        });
    });

    while let Some(res) = set.join_next().await {
        match res {
            Ok(Err(err)) => {
                error!(
                    "encountered error while indexing: {:?}. shutting down.",
                    err
                );
                healthz::set_unhealthy();
                set.shutdown().await;
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
