use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cosmos_client::{
    TxClient,
    gas::{GasFillerT, feemarket},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
};
use futures::{
    StreamExt, TryFutureExt, TryStreamExt,
    future::join,
    stream::{self, try_unfold},
};
use lst::{
    msg::{BatchesResponse, ExecuteMsg, IdentifiedBatch, QueryMsg},
    types::{BatchId, PendingBatch, ReceivedBatch, SubmittedBatch},
};
use protos::{
    cosmos::auth::v1beta1::{Bech32PrefixRequest, Bech32PrefixResponse},
    cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::time::sleep;
use tracing::{info, instrument, warn};
use tracing_subscriber::EnvFilter;
use unionlabs::{
    ErrorReporter,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    google::protobuf::any::Any,
    never::Never,
    primitives::{Bech32, H256},
};

const VERSION: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
struct App {
    #[arg(global = true, default_value = "text", long)]
    log_format: LogFormat,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Version,
    Run {
        #[arg(long)]
        rpc_url: String,
        #[arg(long, env)]
        private_key: H256,
        #[arg(long)]
        lst_hub: Bech32<H256>,
    },
    #[command(visible_alias = "qp")]
    QueryPendingBatch {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        lst_hub: Bech32<H256>,
    },
    #[command(visible_alias = "qs")]
    QuerySubmittedBatches {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        lst_hub: Bech32<H256>,
    },
    #[command(visible_alias = "qr")]
    QueryReceivedBatches {
        #[arg(long)]
        rpc_url: String,
        #[arg(long)]
        lst_hub: Bech32<H256>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default, clap::ValueEnum)]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    do_main().await
}

async fn do_main() -> Result<()> {
    let app = App::parse();

    match app.log_format {
        LogFormat::Text => tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .json()
            .init(),
    }

    match app.cmd {
        Cmd::Version => println!("{VERSION}"),
        Cmd::Run {
            rpc_url,
            private_key,
            lst_hub,
        } => {
            let rpc = Rpc::new(rpc_url.clone()).await?;

            let bech32_prefix = rpc
                .client()
                .grpc_abci_query::<_, Bech32PrefixResponse>(
                    "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                    &Bech32PrefixRequest {},
                    None,
                    false,
                )
                .await
                .context("querying bech32 prefix")?
                .into_result()?
                .unwrap()
                .bech32_prefix;

            let client = TxClient::new(
                LocalSigner::new(private_key, bech32_prefix),
                rpc,
                feemarket::GasFiller::new(feemarket::Config {
                    rpc_url,
                    max_gas: 10000000,
                    gas_multiplier: Some(1.4),
                    denom: None,
                })
                .await?,
            );

            join(
                async {
                    loop {
                        match receive(&client, &lst_hub).await {
                            Err(why) => {
                                warn!("error receiving ready batches: {}", ErrorReporter(&*why))
                            }
                        };
                    }
                },
                async {
                    loop {
                        match submit(&client, &lst_hub).await {
                            Err(why) => {
                                warn!("error submitting pending batch: {}", ErrorReporter(&*why))
                            }
                        };
                    }
                },
            )
            .await;
        }
        Cmd::QueryPendingBatch { rpc_url, lst_hub } => {
            let client = cometbft_rpc::Client::new(rpc_url).await?;

            let batch = query_pending_batch(&client, &lst_hub).await?;

            print_json(&batch);
        }
        Cmd::QuerySubmittedBatches { rpc_url, lst_hub } => {
            let client = cometbft_rpc::Client::new(rpc_url).await?;

            let batches = query_batches::<SubmittedBatch>(&client, &lst_hub, |start_after| {
                QueryMsg::SubmittedBatches {
                    start_after,
                    limit: Some(10),
                }
            })
            .await?;

            print_json(&batches);
        }
        Cmd::QueryReceivedBatches { rpc_url, lst_hub } => {
            let client = cometbft_rpc::Client::new(rpc_url).await?;

            let batches = query_batches::<ReceivedBatch>(&client, &lst_hub, |start_after| {
                QueryMsg::ReceivedBatches {
                    start_after,
                    limit: Some(10),
                }
            })
            .await?;

            print_json(&batches);
        }
    }

    Ok(())
}

async fn receive(
    client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    lst_hub: &Bech32<H256>,
) -> Result<Never> {
    loop {
        let submitted_batches =
            query_batches::<SubmittedBatch>(client.rpc().client(), lst_hub, |start_after| {
                QueryMsg::SubmittedBatches {
                    start_after,
                    limit: Some(10),
                }
            })
            .await?;

        info!("found {} submitted batches", submitted_batches.len());

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be fine")
            .as_secs();

        let ready_batches = submitted_batches
            .into_iter()
            .filter(|b| b.batch.receive_time <= now)
            .collect::<Vec<_>>();

        if ready_batches.is_empty() {
            info!("no ready submitted batches");

            sleep(Duration::from_mins(5)).await;

            continue;
        } else {
            let batch_ids_str = ready_batches
                .iter()
                .map(|b| b.batch_id.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            info!(
                "found {} ready batches: {}",
                ready_batches.len(),
                batch_ids_str,
            );

            let tx_res = client
                .broadcast_tx_commit(
                    ready_batches.iter().map(|b| {
                        Any(MsgExecuteContract {
                            sender: client.wallet().address().map_data(Into::into),
                            contract: lst_hub.clone(),
                            msg: serde_json::to_vec(&ExecuteMsg::ReceiveBatch {
                                batch_id: b.batch_id,
                            })
                            .unwrap()
                            .into(),
                            funds: vec![],
                        })
                    }),
                    VERSION,
                    true,
                )
                .await?;

            info!(tx_hash = %tx_res.hash, "batches {batch_ids_str} received");

            sleep(Duration::from_mins(5)).await;
        }
    }
}

#[instrument(skip_all)]
async fn submit(
    client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    lst_hub: &Bech32<H256>,
) -> Result<Never> {
    loop {
        let PendingBatch {
            batch_id,
            total_lst_to_burn: _,
            unstake_requests_count,
            submit_time,
        } = query_pending_batch(client.rpc().client(), lst_hub).await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("should be fine")
            .as_secs();

        if submit_time <= now {
            if unstake_requests_count == 0 {
                info!("pending batch {batch_id} is ready, but there are no unstake requests");

                sleep(Duration::from_mins(5)).await;

                continue;
            } else {
                info!("pending batch {batch_id} is ready");

                let (tx_hash, _) = client
                    .tx(
                        MsgExecuteContract {
                            sender: client.wallet().address().map_data(Into::into),
                            contract: lst_hub.clone(),
                            msg: serde_json::to_vec(&ExecuteMsg::SubmitBatch {})
                                .unwrap()
                                .into(),
                            funds: vec![],
                        },
                        VERSION,
                        true,
                    )
                    .await?;

                info!(%tx_hash, "batch {batch_id} submitted");
            }
        } else {
            info!(submit_time, now, "pending batch {batch_id} is not ready");

            sleep(Duration::from_secs(submit_time - now)).await;
        }
    }
}

async fn query_batches<B: DeserializeOwned>(
    client: &cometbft_rpc::Client,
    lst_hub: &Bech32<H256>,
    query: fn(start_after: Option<BatchId>) -> QueryMsg,
) -> Result<Vec<IdentifiedBatch<B>>> {
    try_unfold(None, |start_after| {
        query_smart::<BatchesResponse<B>>(client, lst_hub, query(start_after)).map_ok(
            |batches_response| {
                batches_response
                    .batches
                    .last()
                    .map(|batch| batch.batch_id)
                    .map(|batch_id| (batches_response.batches, Some(batch_id)))
            },
        )
    })
    .map_ok(|batches| stream::iter(batches).map(anyhow::Ok))
    .try_flatten()
    .try_collect()
    .await
}

async fn query_pending_batch(
    client: &cometbft_rpc::Client,
    lst_hub: &Bech32<H256>,
) -> Result<PendingBatch> {
    query_smart::<PendingBatch>(client, lst_hub, &QueryMsg::PendingBatch {}).await
}

async fn query_smart<R: DeserializeOwned>(
    client: &cometbft_rpc::Client,
    contract: &Bech32<H256>,
    msg: impl Serialize,
) -> Result<R> {
    client
        .grpc_abci_query::<_, QuerySmartContractStateResponse>(
            "/cosmwasm.wasm.v1.Query/SmartContractState",
            &QuerySmartContractStateRequest {
                address: contract.to_string(),
                query_data: serde_json::to_string(&msg).unwrap().into_bytes(),
            },
            None,
            false,
        )
        .await?
        .into_result()?
        .context("no response?")
        .and_then(|res| serde_json::from_slice(&res.data).map_err(Into::into))
}

fn print_json<T: Serialize>(t: &T) {
    println!(
        "{}",
        serde_json::to_string(&t).expect("serialization is infallible; qed;")
    );
}
