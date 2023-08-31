use clap::Parser;
use tracing::{error, info};
mod cli;
mod hasura;
mod tm;

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();

    let args = crate::cli::Args::parse();
    tracing_subscriber::fmt::init();

    let mut handles = vec![];

    for indexer in args.indexers.into_iter() {
        info!("starting indexer {:?}", indexer);
        let url = args.url.clone();
        let secret = args.secret.clone();
        handles.push(tokio::task::spawn(async move {
            // indexer should never return with Ok, thus we log the error.
            let result = indexer.index(&url, &secret).await;
            error!("indexer {:?} exited with: {:?}", &indexer, result);
        }));
    }

    futures::future::join_all(handles).await;
}
