use std::time::Duration;
use tokio::time::timeout;

use alloy::{
    network::{AnyNetwork, EthereumWallet},
    providers::{fillers::RecommendedFillers, DynProvider, Provider, ProviderBuilder},
    rpc::types::Filter,
    signers::local::LocalSigner,
    sol_types::SolEventInterface,
};
use bip32::secp256k1::ecdsa::{self, SigningKey};
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use ibc_solidity::Ibc::{self, CreateClient, ChannelOpenConfirm, ConnectionOpenConfirm, IbcEvents, PacketRecv};
use serde::{Deserialize, Serialize};
use unionlabs::primitives::{H160, H256};
use voyager_sdk::{anyhow, primitives::ChainId};

#[derive(Debug)]
pub struct Module {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    pub provider: DynProvider<AnyNetwork>,

    pub keyring: ConcurrentKeyring<alloy::primitives::Address, LocalSigner<SigningKey>>,

    pub max_gas_price: Option<u128>,

    pub fixed_gas_price: Option<u128>,

    pub gas_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,
    pub multicall_address: H160,

    /// The RPC endpoint for the execution chain.
    pub rpc_url: String,

    pub keyring: KeyringConfig,

    #[serde(default)]
    pub max_gas_price: Option<u128>,

    /// Temporary fix for 0g until they fix their eth_feeHistory endpoint
    #[serde(default)]
    pub fixed_gas_price: Option<u128>,

    pub gas_multiplier: f64,
}

impl Module {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.rpc_url)
                .await?,
        );

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            panic!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id, chain_id
            );
        }

        Ok(Self {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            multicall_address: config.multicall_address,
            provider,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|config| {
                    let signing_key = <ecdsa::SigningKey as bip32::PrivateKey>::from_bytes(
                        &config.value().as_slice().try_into().unwrap(),
                    )
                    .unwrap();

                    let signer = LocalSigner::from_signing_key(signing_key);

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            max_gas_price: config.max_gas_price,
            fixed_gas_price: config.fixed_gas_price,
            gas_multiplier: config.gas_multiplier,
        })
    }

    // TODO(aeryz): timeout brotha
    async fn wait_for_event<T, F: Fn(IbcEvents) -> Option<T>>(
        &self,
        filter_fn: F,
        timeout: Duration,
    ) -> anyhow::Result<T> {
        tokio::time::timeout(timeout, async {

            let mut prev_latest = self.provider.get_block_number().await?;
            loop {
                let latest = self.provider.get_block_number().await?;

                if prev_latest >= latest {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }

                while prev_latest <= latest {
                    let filter = Filter::new()
                        .address(alloy::primitives::Address::from(self.ibc_handler_address))
                        .from_block(prev_latest)
                        .to_block(prev_latest);

                    // 4) query logs; get_logs returns empty Vec if none
                    let logs = match self.provider.get_logs(&filter).await {
                        Ok(logs) => logs,
                        Err(e) => {
                            return Err(anyhow::anyhow!("get_logs RPC error: {}", e));
                        }
                    };

                    for log in logs {
                        if let Ok(ibc_event) = IbcEvents::decode_log(&log.inner) {
                            if let Some(event) = filter_fn(ibc_event.data) {
                                return Ok(event);
                            }
                        }
                    }

                    prev_latest += 1u64;
                }

                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }).await
        .map_err(|_| anyhow::anyhow!("timed out after {:?}", timeout))?
    }
    
    pub async fn wait_for_create_client(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<CreateClient> {
        self.wait_for_event(|e| match e {
            IbcEvents::CreateClient(ev) => Some(ev),
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<ConnectionOpenConfirm> {
        self.wait_for_event(|e| match e {
            IbcEvents::ConnectionOpenConfirm(ev) => Some(ev),
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<ChannelOpenConfirm> {
        self.wait_for_event(|e| match e {
            IbcEvents::ChannelOpenConfirm(ev) => Some(ev),
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<PacketRecv> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::PacketRecv(ev) if ev.packet_hash.as_slice() == packet_hash.as_ref() => Some(ev),
                _ => None,
            },
            timeout,
        )
        .await
    }


    // async fn submit_transaction(
    //     &self,
    //     wallet: &LocalSigner<SigningKey>,
    //     ibc_messages: Vec<Datagram>,
    // ) -> anyhow::Result<()> {
    //     let signer = DynProvider::new(
    //         ProviderBuilder::new()
    //             .network::<AnyNetwork>()
    //             .filler(AnyNetwork::recommended_fillers())
    //             // .filler(<NonceFiller>::default())
    //             // .filler(ChainIdFiller::default())
    //             .wallet(EthereumWallet::new(wallet.clone()))
    //             .connect_provider(self.provider.clone()),
    //     );

    //     if let Some(max_gas_price) = self.max_gas_price {
    //         let gas_price = self
    //             .provider
    //             .get_gas_price()
    //             .await
    //             .expect("unable to fetch gas price");

    //         if gas_price > max_gas_price {
    //             panic!("gas price is too high");
    //         } else {
    //             println!("gas price {}", gas_price);
    //         }
    //     }

    //     let multicall = Multicall::new(self.multicall_address.into(), signer.clone());

    //     let ibc = Ibc::new(self.ibc_handler_address.into(), &self.provider);

    //     let msgs = process_msgs(
    //         &ibc,
    //         ibc_messages,
    //         self.fee_recipient.unwrap_or(wallet.address()).into(),
    //     )?;

    //     trace!(?msgs);

    //     let msg_names = msgs
    //         .iter()
    //         // .map(|x| (x.0.clone(), x.1.function.name.clone()))
    //         .map(|x| (x.0.clone(), x.0.name()))
    //         .collect::<Vec<_>>();

    //     let mut call = multicall.multicall(
    //         msgs.clone()
    //             .into_iter()
    //             .map(|(_, call)| Call3 {
    //                 target: self.ibc_handler_address.into(),
    //                 allowFailure: true,
    //                 callData: call.calldata().clone(),
    //             })
    //             .collect(),
    //     );

    //     info!("submitting evm tx");

    //     let gas_estimate = call.estimate_gas().await.map_err(|e| {
    //         if ErrorReporter(&e)
    //             .to_string()
    //             .contains("gas required exceeds")
    //         {
    //             TxSubmitError::BatchTooLarge
    //         } else {
    //             TxSubmitError::Estimate(e)
    //         }
    //     })?;

    //     let gas_to_use = ((gas_estimate as f64) * self.gas_multiplier) as u64;

    //     info!(
    //         gas_multiplier = %self.gas_multiplier,
    //         %gas_estimate,
    //         %gas_to_use,
    //         "gas estimatation successful"
    //     );

    //     if let Some(fixed_gas_price) = self.fixed_gas_price {
    //         call = call.gas_price(fixed_gas_price);
    //     }

    //     match call.gas(gas_to_use).send().await {
    //         Ok(ok) => {
    //             let tx_hash = <H256>::from(*ok.tx_hash());
    //             async move {
    //                 let receipt = ok.get_receipt().await?;

    //                 info!(%tx_hash, "tx included");

    //                 let result = MulticallResult::decode_log_data(
    //                     receipt
    //                         .inner
    //                         .inner
    //                         .logs()
    //                         .last()
    //                         .expect("multicall event should be last log")
    //                         .data(),
    //                 )
    //                 .expect("unable to decode multicall result log");

    //                 info!(
    //                     gas_used = %receipt.gas_used,
    //                     batch.size = msg_names.len(),
    //                     "submitted batched evm messages"
    //                 );

    //                 for (idx, (result, (msg, msg_name))) in
    //                     result._0.into_iter().zip(msg_names).enumerate()
    //                 {
    //                     if result.success {
    //                         info!(
    //                             msg = msg_name,
    //                             %idx,
    //                             data = %into_value(&msg),
    //                             "evm tx",
    //                         );
    //                     } else if let Ok(known_revert) =
    //                         IbcErrors::abi_decode_validate(&result.returnData)
    //                     {
    //                         error!(
    //                             msg = %msg_name,
    //                             %idx,
    //                             revert = ?known_revert,
    //                             well_known = true,
    //                             data = %into_value(&msg),
    //                             "evm message failed",
    //                         );
    //                     } else if result.returnData.is_empty() {
    //                         error!(
    //                             msg = %msg_name,
    //                             %idx,
    //                             revert = %result.returnData,
    //                             well_known = false,
    //                             data = %into_value(&msg),
    //                             "evm message failed with 0x revert, likely an ABI issue",
    //                         );
    //                     } else {
    //                         error!(
    //                             msg = %msg_name,
    //                             %idx,
    //                             revert = %result.returnData,
    //                             well_known = false,
    //                             data = %into_value(&msg),
    //                             "evm message failed",
    //                         );
    //                     }
    //                 }

    //                 Ok(())
    //             }
    //             .instrument(info_span!(
    //                 "evm tx",
    //                 %tx_hash,
    //             ))
    //             .await
    //         }
    //         Err(
    //             Error::PendingTransactionError(PendingTransactionError::TransportError(
    //                 TransportError::ErrorResp(e),
    //             ))
    //             | Error::TransportError(TransportError::ErrorResp(e)),
    //         ) if e
    //             .message
    //             .contains("insufficient funds for gas * price + value") =>
    //         {
    //             error!("out of gas");
    //             Err(TxSubmitError::OutOfGas)
    //         }
    //         Err(
    //             Error::PendingTransactionError(PendingTransactionError::TransportError(
    //                 TransportError::ErrorResp(e),
    //             ))
    //             | Error::TransportError(TransportError::ErrorResp(e)),
    //         ) if e.message.contains("oversized data")
    //             || e.message.contains("exceeds block gas limit")
    //             || e.message.contains("gas required exceeds") =>
    //         {
    //             if msgs.len() == 1 {
    //                 error!(error = %e.message, msg = ?msgs[0], "message is too large");
    //                 Ok(()) // drop the message
    //             } else {
    //                 warn!(error = %e.message, "batch is too large");
    //                 Err(TxSubmitError::BatchTooLarge)
    //             }
    //         }
    //         Err(err) => Err(TxSubmitError::Error(err)),
    //     }
    // }
}
