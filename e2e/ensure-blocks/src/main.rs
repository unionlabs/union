use std::fmt::Debug;

use alloy::providers::{Provider, ProviderBuilder};
use clap::Parser;
use futures::{Stream, StreamExt};
use tendermint_rpc::{query::EventType, SubscriptionClient, WebSocketClient};
use tokio::join;
use tracing::{debug, error, info};

#[derive(Debug, Parser)]
struct Args {
    /// WebSocket URL for connection to the union chain
    #[clap(long)]
    union: String,

    /// WebSocket URL for connection to sepolia
    #[clap(long)]
    sepolia: String,

    /// Number of blocks to wait for
    #[clap(long, default_value_t = 10)]
    block_limit: usize,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if let Err(e) = do_main(args).await {
        error!(error = ?e, "An error occurred");
    }
    Ok(())
}

async fn do_main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let provider = initialize_provider(&args.sepolia).await?;
    let (tm_client, driver) = initialize_tendermint_client(&args.union).await?;

    // Spawn Tendermint driver
    tokio::spawn(async move {
        if let Err(e) = driver.run().await {
            error!(error = ?e, "Tendermint driver error");
        }
    });

    let block_limit = args.block_limit;
    let sepolia_blocks = process_blocks("sepolia", provider.subscribe_blocks().await?.into_stream(), block_limit);
    let union_blocks = process_blocks(
        "union",
        tm_client.subscribe(EventType::NewBlock.into()).await?,
        block_limit,
    );

    join!(union_blocks, sepolia_blocks);

    Ok(())
}

async fn initialize_provider(sepolia_url: &str) -> Result<Provider, Box<dyn std::error::Error>> {
    info!(url = %sepolia_url, "Initializing provider");
    let provider = ProviderBuilder::new()
        .on_builtin(sepolia_url)
        .await
        .map_err(|e| format!("Failed to initialize provider: {}", e))?;
    Ok(provider)
}

async fn initialize_tendermint_client(
    union_url: &str,
) -> Result<(WebSocketClient, WebSocketClient::Driver), Box<dyn std::error::Error>> {
    info!(url = %union_url, "Initializing Tendermint client");
    let url = union_url.parse()?;
    let (tm_client, driver) = WebSocketClient::builder(url)
        .compat_mode(tendermint_rpc::client::CompatMode::V0_37)
        .build()
        .await
        .map_err(|e| format!("Failed to initialize Tendermint client: {}", e))?;
    Ok((tm_client, driver))
}

async fn process_blocks<T>(
    chain: &'static str,
    blocks: impl Stream<Item = T>,
    block_limit: usize,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Debug,
{
    info!(chain, block_limit, "Processing blocks");
    let blocks: Vec<_> = blocks
        .take(block_limit)
        .enumerate()
        .map(|(n, block)| {
            info!(chain, block_number = n + 1, "Block received");
            block
        })
        .collect()
        .await;

    if blocks.len() != block_limit {
        error!(chain, "Did not receive expected number of blocks");
        return Err(format!("Expected {block_limit} blocks, got {}", blocks.len()).into());
    }

    info!(chain, "Completed block processing");
    Ok(())
}
