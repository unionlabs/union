#![feature(return_position_impl_trait_in_trait)]
#![feature(result_option_inspect)]

use clap::Parser;
use hasura::HasuraDataStore;
use reqwest::Client;
use tokio::task::JoinSet;
use tracing::{error, info, warn};
use tracing_subscriber::util::SubscriberInitExt;

mod cli;
mod hasura;
mod tm;

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install().unwrap();

    let args = crate::cli::Args::parse();
    tracing_subscriber::fmt::fmt()
        .with_ansi(false)
        .finish()
        .init();

    let url = args.url.clone();
    let secret = args.secret.clone();
    let client = Client::new();
    let db = HasuraDataStore::new(client, url, secret);
    let mut set = JoinSet::new();

    args.indexers.into_iter().for_each(|indexer| {
        let db = db.clone();
        set.spawn(async move {
            info!("starting indexer {:?}", indexer);
            // indexer should never return with Ok, thus we log the error.
            let result = indexer.index(db).await.inspect_err(|err| {
                warn!("indexer {:?} exited with: {:?}", &indexer, err);
            });
            result
        });
    });

    while let Some(res) = set.join_next().await {
        if let Err(err) = res {
            error!(
                "encountered error while indexing: {:?}. shutting down.",
                err
            );
            set.shutdown().await;
            return Err(err.into());
        }
        info!("indexer exited gracefully");
    }
    Ok(())
}
