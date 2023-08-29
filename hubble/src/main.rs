use clap::Parser;
use tracing::info;
mod cli;
mod hasura;
mod tm;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = crate::cli::Args::parse();
    tracing_subscriber::fmt::init();

    for indexer in args.indexers.into_iter() {
        info!("starting indexer {:?}", indexer);
        let url = args.hasura.clone();
        let secret = args.secret.clone();
        tokio::task::spawn_local(async move { indexer.index(&url, &secret).await.unwrap() });
    }
}
