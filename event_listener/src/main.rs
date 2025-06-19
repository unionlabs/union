use anyhow::Result;
use std::{num::NonZeroU8, num::NonZeroU32, time::Duration, sync::Arc, str::FromStr};

use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use tokio::runtime::Builder;
use anyhow::anyhow;
use alloy::{
    providers::{layers::CacheLayer, DynProvider, Provider, ProviderBuilder},
    rpc::types::Filter,
    sol_types::SolEventInterface,
};

use jsonrpsee::{
    core::{async_trait, RpcResult},
};
use ibc_union_msg::msg::{
    ExecuteMsg,
    MsgCreateClient,
    MsgConnectionOpenInit,
    MsgChannelOpenInit,
    MsgPacketRecv,
    MsgSendPacket,
};

use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use alloy::hex::FromHex;
use cosmos_client::{
    gas::{
        any,
        feemarket,
        fixed
    },
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    TxClient,
    BroadcastTxCommitError,
};
// use prost_types::Any;
use protos::cosmwasm::wasm::v1::MsgExecuteContract;
use protos::google::protobuf::Any as ProstAny;
use unionlabs::cosmos::base::coin::Coin;
use serde_json::to_vec;

use crate::call::IbcMessage; // your enum
use unionlabs::{
    primitives::{Bytes, H160, H256},
    google::protobuf::any::mk_any,
    bech32::Bech32,
};
pub mod call;

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
    }
}

// fn encode_ibc_message(
//     msg: IbcMessage,
//     signer: &LocalSigner,
//     ibc_host_contract_address: String
// ) -> anyhow::Result<protos::google::protobuf::Any> {
//     let signer = signer.address().to_string();
//     match msg {
//         IbcMessage::IbcUnion(datagram) => {
//             // pick only the 4 you need + SendPacket
//             let exec: ExecuteMsg = match datagram {
//                 ibc_union_spec::datagram::Datagram::CreateClient(m) => {
//                     ExecuteMsg::CreateClient(MsgCreateClient {
//                         client_type: m.client_type.to_string(),
//                         client_state_bytes: m.client_state_bytes,
//                         consensus_state_bytes: m.consensus_state_bytes,
//                         relayer: signer.to_string(),
//                     })
//                 }
//                 ibc_union_spec::datagram::Datagram::ConnectionOpenInit(m) => {
//                     ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
//                         client_id: m.client_id,
//                         counterparty_client_id: m.counterparty_client_id,
//                     })
//                 }
//                 ibc_union_spec::datagram::Datagram::ChannelOpenInit(m) => {
//                     ExecuteMsg::ChannelOpenInit(MsgChannelOpenInit {
//                         port_id: String::from_utf8(m.port_id.to_vec())?,
//                         connection_id: m.connection_id,
//                         counterparty_port_id: m.counterparty_port_id,
//                         version: m.version,
//                         relayer: signer.to_string(),
//                     })
//                 }
//                 ibc_union_spec::datagram::Datagram::PacketRecv(m) => {
//                     ExecuteMsg::PacketRecv(MsgPacketRecv {
//                         packets: m.packets.into_iter().collect(),
//                         relayer_msgs: m.relayer_msgs,
//                         proof: m.proof,
//                         proof_height: m.proof_height,
//                         relayer: signer.to_string(),
//                     })
//                 }
//                 other => {
//                     return Err(anyhow!(
//                         "encode_ibc_message: unsupported variant {:?}",
//                         other
//                     ));
//                 }
//             };

//             let bin = serde_json::to_vec(&exec)?;
//             Ok(mk_any(&MsgExecuteContract {
//                 sender:  signer.to_string(),
//                 contract: ibc_host_contract_address.to_string(),
//                 msg:     bin,
//                 funds:   vec![],
//             }))
//         }
//         other => Err(anyhow!(
//             "encode_ibc_message: expected IbcUnion, got {:?}",
//             other
//         )),
//     }
// }

fn encode_ibc_message(
    msg: IbcMessage,
    signer: &LocalSigner,
    ibc_host_contract_address: &str,
) -> anyhow::Result<protos::google::protobuf::Any> {
    // 1) bail if it’s not the union variant
    let datagram = match msg {
        IbcMessage::IbcUnion(d) => d,
        other => return Err(anyhow!("expected IbcUnion got {:?}", other)),
    };

    // 2) handle only ConnectionOpenInit for now
    let inner = match datagram {
        ibc_union_spec::datagram::Datagram::ConnectionOpenInit(m) => {
            // **Convert these fields to String!**  
            // If you leave them as their native type, the JSON
            // won’t have the right shape and the wasm contract
            // will reject it.
            let exec = ExecuteMsg::ConnectionOpenInit(MsgConnectionOpenInit {
                client_id:                m.client_id,
                counterparty_client_id:  m.counterparty_client_id,
            });

            // dump the JSON so you can verify it locally:
            let json = serde_json::to_string_pretty(&exec)?;
            eprintln!(">> payload JSON:\n{}", json);

            // now turn that JSON into the binary for the wasm message:
            let bin = serde_json::to_vec(&exec)?;

            MsgExecuteContract {
                sender:   signer.address().to_string(),
                contract: ibc_host_contract_address.to_string(),
                msg:      bin,
                funds:    vec![],
            }
        }

        other => {
            return Err(anyhow!(
                "encode_ibc_message: unsupported union variant {:?}",
                other
            ))
        }
    };

    Ok(mk_any(&inner))
}


/// send a batch of IBC messages in one Cosmos tx
async fn send_tx_msg(
    rpc_url: &str,
    signer: LocalSigner,
    ibc_host_contract_address: Bech32<H256>,
    gas_filler: any::GasFiller,
    msgs: Vec<IbcMessage>,
) -> Result<(), BroadcastTxCommitError> {
    // 1) connect to chain
    let rpc = Rpc::new(rpc_url.to_string()).await?;
    let tx_client = TxClient::new(&signer, &rpc, &gas_filler);

    let addr = signer.address().to_string();
    let mut anys: Vec<protos::google::protobuf::Any> = Vec::with_capacity(msgs.len());
    for msg in msgs {
        let any = encode_ibc_message(msg, &signer, ibc_host_contract_address.to_string().as_str())
            .map_err(|e| BroadcastTxCommitError::TxFailed {
                codespace:   "encode".to_string(),
                error_code:  NonZeroU32::new(1).unwrap(),
                log:         e.to_string(),
            })?;
        anys.push(any);

    }

    info!("encoded {:?} IBC messages", anys[0].value);

    let memo = "event_listener";
    let acct_addr = signer.address().to_string();
    info!(%acct_addr, "about to query for account");
    
    let res = tx_client
        .broadcast_tx_commit(anys, memo.to_string(), true)
        .await?;

    println!("✅ IBC tx committed: {}", res.hash);
    Ok(())
}


fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cosmos_rpc_url = "https://rpc.union-testnet-10.union.chain.cooking";
    let cosmos_start = 4520093;


    let evm_rpc = "https://rpc.bbn-test-5.babylon.chain.cooking/";
    let start_block_evm: u64 = 8548200;    // This block has packetrecv's
    let start_block_evm: u64 = 8139000; // This block has connection_open_confirm's and channel confirms
    let start_block_evm = 8087808;

    info!("starting Cosmos listener at block {}", cosmos_start);
    info!("starting  EVM listener at block {}", start_block_evm);

    let rt = Builder::new_current_thread()
        .enable_all()
        .build()?;

     // 4) run both listeners concurrently
     rt.block_on(async {
        let rpc_url: &'static str = "https://rpc.union-testnet-10.union.chain.cooking";
        let max_gas = 10000000;
        let gas_multiplier = 1.4;
        let denom = "muno";
        let ibc_host_contract_address = "union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf";

        let private_key_raw = "0xsomerandom"; 
        let key_bytes: [u8;32] = <[u8;32]>::from_hex(private_key_raw)?;

        let rpc = Rpc::new(rpc_url.to_string().clone()).await?;
        
        let bech32_prefix = rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
                "/cosmos.auth.v1beta1.Query/Bech32Prefix",
                &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
                None,
                false,
            )
            .await?
            .into_result()?
            .unwrap()
            .bech32_prefix;

        info!("bech32 prefix: {}", bech32_prefix);
    
        let signer = LocalSigner::new(key_bytes.into(), bech32_prefix.to_string());

        info!("signer address: {}", signer.address());

        // let gas_filler = any::GasFiller::Feemarket(
        //     feemarket::GasFiller::new(
        //         feemarket::Config {
        //             rpc_url: rpc_url.to_string(),
        //             max_gas: max_gas,
        //             gas_multiplier: Some(gas_multiplier),
        //             denom: Some(denom.to_string()),
        //         }
        //     ).await?
        // );

        // First, build the underlying fixed‐price gas filler:
        let fixed_config = fixed::GasFiller {
            gas_price:  0.003,
            gas_multiplier: gas_multiplier, 
            gas_denom:  denom.to_string(),
            max_gas,                       
            min_gas: max_gas/10,    
        };
        // Then wrap it in the `any::GasFiller::Fixed` enum:
        let gas_filler = any::GasFiller::Fixed(fixed_config);
        

        // Build your IBC message
        let init = ibc_union_spec::datagram::MsgConnectionOpenInit {
            client_id:          <_ as FromStr>::from_str("1")?,       // or "1".parse()?
            counterparty_client_id: <_ as FromStr>::from_str("1")?,
        };
        let msgs = vec![
            IbcMessage::IbcUnion(ibc_union_spec::datagram::Datagram::ConnectionOpenInit(init)),
        ];

        let ibc_host = ibc_host_contract_address.parse::<Bech32<H256>>()?;

        send_tx_msg(cosmos_rpc_url, signer, ibc_host, gas_filler, msgs).await?;

        // 4) now kick off your listeners
        let cosmos_start = 4_520_093u64;

        tokio::try_join!(

            listen_cosmos_events(&cosmos_rpc_url, cosmos_start),
            // listen_evm_events  (&evm_rpc,    ibc_handler, start_block_evm),
        )?;
        Ok(())
    })
}
