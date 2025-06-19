use anyhow::Result;
use std::{num::NonZeroU8, num::NonZeroU32, time::Duration};

use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use tokio::runtime::Builder;
use anyhow::anyhow;
use alloy::{
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::Filter,
    sol_types::SolEventInterface,
};

use unionlabs::{
    primitives::{H160},
};

use ibc_solidity::Ibc;

use cometbft_rpc::{Client as TendermintClient, rpc_types::Order};

async fn listen_cosmos_events(
    rpc_url: &str,
    mut height: u64,
) -> Result<()> {
    let client = TendermintClient::new(rpc_url).await?;

    loop {
        let status = client.status().await?;
        let latest = status.sync_info.latest_block_height;
        info!(
            "latest block height: {} (current: {})",
            latest,
            height
        );
        while height <= latest {
            let mut page = NonZeroU32::new(1).unwrap();
            let mut seen = 0;

            if height % 100 == 0 {
                info!("processing block {}", height);
            }


            loop {
                let resp = match client
                    .tx_search(
                        format!("tx.height={}", height),
                        false,
                        page,
                        NonZeroU8::new(100).unwrap(),
                        Order::Asc,
                    )
                    .await
                {
                    Ok(r) => r,
                    Err(err) => {
                        let serr = err.to_string();

                        if serr.contains("page should be within") {
                            warn!("page {} for block {} is out of range -- breaking loop.", page, height);
                            break;
                        } else {
                            // some other RPC error: bail out
                            return Err(anyhow!("tx_search error: {}", err));
                        }
                    }
                };

                seen += resp.txs.len();
                info!("found {} txs in block {} page {}", resp.txs.len(), height, page);
                for tx in &resp.txs {
                    for ev in &tx.tx_result.events {

                        match ev.ty.as_str() {
                            "wasm-channel_open_confirm"
                          | "wasm-connection_open_confirm" => {
                        //   | "wasm-packet_recv" => {
                              info!(
                                  "[block={}][tx={}] {} → {:?}",
                                  height,
                                  tx.hash,
                                  ev.ty,
                                  ev.attributes,
                              );
                          }
                            _ => {}
                        }
                    }
                }

                if seen >= (resp.total_count as usize) {
                    break;
                } else {
                    page = page
                    .checked_add(1)
                    .expect("how many events does this block have???");
                }
            }

            height += 1;
        }

        // wait before polling again
        // tokio::time::sleep(Duration::from_secs(5)).await;
    }
}


async fn listen_evm_events(
    rpc_url: &str,
    contract: H160,
    mut block: u64,
) -> Result<()> {
    let provider = DynProvider::new(
        ProviderBuilder::new().
        connect(rpc_url)
        .await?
    );
    loop {
        let latest = provider.get_block_number().await?;

        while block <= latest {
            if block % 100 == 0 {
                info!("processing EVM block {}", block);
            }

            let filter = Filter::new()
                .address(alloy::primitives::Address::from(contract))
                .from_block(block)
                .to_block(block);

            // 4) query logs; get_logs returns empty Vec if none
            let logs = match provider.get_logs(&filter).await {
                Ok(logs) => logs,
                Err(e) => {
                    return Err(anyhow::anyhow!("get_logs RPC error: {}", e));
                }
            };

            for log in logs {
                if let Ok(ibc_event) = Ibc::IbcEvents::decode_log(&log.inner) {
                    match ibc_event.data {
                        Ibc::IbcEvents::ConnectionOpenConfirm(evt) => {
                            info!(
                                "[evm-block={}][tx={:?}] ConnectionOpenConfirm → {:?}",
                                block,
                                log.transaction_hash,
                                evt
                            );
                        }
                        Ibc::IbcEvents::ChannelOpenConfirm(evt) => {
                            info!(
                                "[evm-block={}][tx={:?}] ChannelOpenConfirm → {:?}",
                                block,
                                log.transaction_hash,
                                evt
                            );
                        }
                        Ibc::IbcEvents::PacketRecv(evt) => {
                            info!(
                                "[evm-block={}][tx={:?}] PacketRecv → {:?}",
                                block,
                                log.transaction_hash,
                                evt
                            );
                        }
                        _ => {}
                    }
                }
            }

            block += 1u64;
        }

        // 6) wait before polling again
        // tokio::time::sleep(Duration::from_secs(5)).await;
    }
}


fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cosmos_rpc_url = "https://rpc.xion-testnet-2.xion.chain.cooking";
    let cosmos_start = 4520093;


    let evm_rpc = "https://rpc.bbn-test-5.babylon.chain.cooking/";
    let start_block_evm: u64 = 8548200;    // This block has packetrecv's
    let start_block_evm: u64 = 8139000; // This block has connection_open_confirm's and channel confirms
    let start_block_evm = 8087808;

    let ibc_handler = "0xee4ea8d358473f0fcebf0329feed95d56e8c04d7".parse::<H160>()?;

    info!("starting Cosmos listener at block {}", cosmos_start);
    info!("starting  EVM listener at block {}", start_block_evm);

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()?;

     // 4) run both listeners concurrently
     rt.block_on(async {
        tokio::try_join!(
            listen_cosmos_events(&cosmos_rpc_url, cosmos_start),
            // listen_evm_events  (&evm_rpc,    ibc_handler, start_block_evm),
        )?;
        Ok(())
    })
}
