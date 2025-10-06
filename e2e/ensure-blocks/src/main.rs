use std::{fmt::Debug, time::Duration};

use alloy::providers::{Provider, ProviderBuilder};
use clap::Parser;
use futures::{stream::unfold, StreamExt};
use tokio::{join, time::sleep};

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

    let tm_client = cometbft_rpc::Client::new(args.union).await.unwrap();

    let sepolia_blocks = fetch_blocks(
        "sepolia",
        provider.subscribe_blocks().await.unwrap().into_stream(),
    );
    let union_blocks = fetch_blocks(
        "union",
        unfold((1, tm_client), |(block_number, tm_client)| async move {
            let latest_block = tm_client.block(None).await.unwrap();
            if latest_block.block.header.height.inner() > block_number {
                Some((Some(latest_block), (block_number + 1, tm_client)))
            } else {
                sleep(Duration::from_secs(1)).await;
                Some((None, (block_number, tm_client)))
            }
        }),
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
