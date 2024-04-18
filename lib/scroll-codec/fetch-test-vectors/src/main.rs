use std::{ops::RangeInclusive, path::PathBuf};

use clap::Parser;
use ethers::{
    abi::{AbiDecode, RawLog},
    contract::EthEvent,
    providers::{Http, Middleware, Provider},
};
use scroll_codec::{
    commit_batch, fetch_l1_message_hashes, CommitBatchCall, CommitBatchEvent, TestVector,
};
use url::Url;

#[derive(Parser)]
pub struct Args {
    #[arg(long)]
    pub eth_rpc_url: Url,
    #[arg(long)]
    pub scroll_api_url: String,
    #[arg(long)]
    pub out_dir: String,
    #[arg(long, value_parser(parse_range))]
    /// Format is `from,to`.
    pub range: RangeInclusive<u64>,
}

fn parse_range(s: &str) -> clap::error::Result<RangeInclusive<u64>> {
    s.split_once(',')
        .ok_or(clap::Error::new(clap::error::ErrorKind::InvalidValue))
        .and_then(|(from, to)| {
            let from = from
                .parse()
                .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidValue))?;
            let to = to
                .parse()
                .map_err(|_| clap::Error::new(clap::error::ErrorKind::InvalidValue))?;

            Ok(from..=to)
        })
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    fetch_test_vectors(args).await;
}

async fn fetch_test_vectors(args: Args) {
    let provider = Provider::new(Http::new(args.eth_rpc_url));
    let scroll_client = scroll_api::ScrollClient::new(args.scroll_api_url);

    for i in args.range {
        let provider = provider.clone();
        let scroll_client = scroll_client.clone();

        if PathBuf::from(format!("{}/{i}.json", args.out_dir)).exists() {
            println!("{i} exists, skipping");
            continue;
        }

        println!("fetching {i}");

        let batch = scroll_client.batch(i).await;

        let tx = provider
            .get_transaction(batch.batch.commit_tx_hash)
            .await
            .unwrap()
            .unwrap();

        println!("{i} fetched tx");

        let tx_rcp = provider
            .get_transaction_receipt(batch.batch.commit_tx_hash)
            .await
            .unwrap()
            .unwrap();

        println!("{i} fetched tx receipt");

        let call: CommitBatchCall = CommitBatchCall::decode(&tx.input).unwrap();
        let message_queue =
            fetch_l1_message_hashes(&provider, tx.block_number.unwrap().as_u64(), call.clone())
                .await
                .unwrap();

        println!("{i} fetched l1 message hashes");

        let [event] = tx_rcp
            .logs
            .into_iter()
            .filter_map(|x| <CommitBatchEvent as EthEvent>::decode_log(&RawLog::from(x)).ok())
            .collect::<Vec<CommitBatchEvent>>()
            .try_into()
            .expect("transactions only contain one commit batch");

        println!("{i} data fetched, writing file");

        let blob_versioned_hash = tx
            .blob_versioned_hashes
            .unwrap_or_default()
            .first()
            .map(|x| x.0.into());
        std::fs::write(
            format!("{}/{i}.json", args.out_dir),
            serde_json::to_string_pretty(&TestVector {
                tx_hash: batch.batch.commit_tx_hash,
                input: tx.input.to_vec(),
                call: call.clone(),
                blob_versioned_hash,
                message_queue: message_queue.clone(),
                expected_batch_hash: event.batch_hash,
            })
            .unwrap(),
        )
        .unwrap();

        println!("{i} file written, verifying");

        let hash = commit_batch(call, blob_versioned_hash, message_queue).unwrap();

        assert_eq!(hash, event.batch_hash);

        println!("{i} ok");
    }
}
