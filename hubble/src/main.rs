#![feature(return_position_impl_trait_in_trait)]

use clap::Parser;
use hasura::HasuraDataStore;
use reqwest::Client;
use tracing::{error, info};
mod cli;
mod hasura;
mod tm;

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();

    let args = crate::cli::Args::parse();
    tracing_subscriber::fmt::init();

    let url = args.url.clone();
    let secret = args.secret.clone();
    let client = Client::new();
    let db = HasuraDataStore::new(client, url, secret);

    let handles = args.indexers.into_iter().map(|indexer| {
        let db = db.clone();
        async move {
            info!("starting indexer {:?}", indexer);

            // indexer should never return with Ok, thus we log the error.
            let result = indexer.index(db).await;
            error!("indexer {:?} exited with: {:?}", &indexer, result);
        }
    });

    futures::future::join_all(handles).await;
}
