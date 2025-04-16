use std::fmt::Debug;

use alloy::providers::{Provider, ProviderBuilder};
use clap::Parser;
use futures::StreamExt;
use tendermint_rpc::{query::EventType, SubscriptionClient, WebSocketClient};
use tokio::join;

const BLOCKS_TO_WAIT_FOR: usize = 10;

#[derive(Debug, Parser)]
struct Args {
    union: String,
    /// Websocket URL for connection to sepolia
    sepolia: String,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    do_main(args).await;
}

async fn do_main(args: Args) {
    let provider = ProviderBuilder::new().connect(&args.sepolia).await.unwrap();

    let (tm_client, driver) = WebSocketClient::builder(args.union.parse().unwrap())
        .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
        .build()
        .await
        .unwrap();
    tokio::spawn(async move { driver.run().await });

    let sepolia_blocks = fetch_blocks(
        "sepolia",
        provider.subscribe_blocks().await.unwrap().into_stream(),
    );
    let union_blocks = fetch_blocks(
        "union",
        tm_client
            .subscribe(EventType::NewBlock.into())
            .await
            .unwrap(),
    );

    join!(union_blocks, sepolia_blocks);
}

async fn fetch_blocks<T: Debug>(chain: &'static str, blocks: impl StreamExt<Item = T>) {
    let blocks = blocks
        .take(BLOCKS_TO_WAIT_FOR)
        .enumerate()
        .map(|(n, block)| {
            println!("{chain}: block {}", n + 1);
            block
        })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(blocks.len(), BLOCKS_TO_WAIT_FOR, "{chain}");
}
