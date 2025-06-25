use std::{time::Duration, panic::AssertUnwindSafe};
use tokio::time::timeout;

use alloy::{
    contract::{Error, RawCallBuilder},
    transports::TransportError,
    providers::{
        fillers::RecommendedFillers, layers::CacheLayer, DynProvider, PendingTransactionError,
        Provider, ProviderBuilder,
    },
    network::{AnyNetwork, EthereumWallet}, primitives::TxHash, rpc::types::{self, AnyReceiptEnvelope, Filter, Log, TransactionReceipt}, signers::local::LocalSigner, sol_types::SolEventInterface    
};
use bip32::secp256k1::ecdsa::{self, SigningKey};
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use ibc_union_spec::{datagram::Datagram, IbcUnion};

use ibc_solidity::Ibc::{self, CreateClient, ChannelOpenConfirm, ConnectionOpenConfirm, IbcEvents, IbcErrors, PacketRecv};
use serde::{Deserialize, Serialize};
use unionlabs::{primitives::{H160, H256}, ErrorReporter};
use voyager_sdk::{
    anyhow::{self, anyhow, bail},
    into_value,
    primitives::ChainId};
    
use multicall::{Call3, Multicall, MulticallResult};
// use voyager_sdk::plugin::Plugin::
// use crate::multicall::{Call3, Multicall, MulticallResult};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    types::{ErrorObject, ErrorObjectOwned},
    Extensions, MethodsError,
};
use tracing::{error, info, info_span, instrument, trace, warn, Instrument};
use crate::helpers;

#[derive(Debug)]
pub struct Module {
    pub chain_id: ChainId,

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

    pub rpc_url: String,

    pub keyring: KeyringConfig,

    #[serde(default)]
    pub max_gas_price: Option<u128>,

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
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_event(|e| match e {
            IbcEvents::CreateClient(ev) => Some(helpers::CreateClientConfirm {
                client_id: ev.client_id,
            }),
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_event(|e| match e {
            IbcEvents::ConnectionOpenConfirm(ev) => {
                Some(helpers::ConnectionConfirm {
                    connection_id: ev.connection_id,
                    counterparty_connection_id: ev.counterparty_connection_id,
                })
            }
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        timeout: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm> {
        self.wait_for_event(|e| match e {
            IbcEvents::ChannelOpenConfirm(ev) => Some(helpers::ChannelOpenConfirm {
                channel_id: ev.channel_id,
                counterparty_channel_id: ev.counterparty_channel_id,
            }),
            _ => None,
        }, timeout)
        .await
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash: H256,
        timeout: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        self.wait_for_event(
            |e| match e {
                IbcEvents::PacketRecv(ev) if ev.packet_hash.as_slice() == packet_hash.as_ref() => Some(helpers::PacketRecv {
                    hash: ev.packet_hash,
                }),
                _ => None,
            },
            timeout,
        )
        .await
    }
    pub async fn send_ibc_transaction(
        &self,
        contract: H160,
        ibc_messages: Vec<(
            Datagram,
            RawCallBuilder<DynProvider<AnyNetwork>, AnyNetwork>,
        )>,
        packet_hash: H256,
        timeout: Duration,
    ) -> RpcResult<helpers::PacketRecv> {
        self.send_transaction(contract, ibc_messages).await?;

        let ev = self
            .wait_for_packet_recv(packet_hash, timeout)
            .await
            .map_err(|e| {
                ErrorObject::owned(
                    -1,
                    format!("timeout or RPC error waiting for PacketRecv: {}", e),
                    None::<()>,
                )
            })?;

        Ok(helpers::PacketRecv {
            hash: ev.hash,
        })
        // Ok(ev)
    }


    async fn submit_transaction(
        &self,
        wallet: &LocalSigner<SigningKey>,
        ibc_messages: Vec<(
            Datagram,
            RawCallBuilder<DynProvider<AnyNetwork>, AnyNetwork>,
        )>,
    ) -> Result<(), TxSubmitError> {
        let signer = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .filler(AnyNetwork::recommended_fillers())
                .wallet(EthereumWallet::new(wallet.clone()))
                .connect_provider(self.provider.clone()),
        );

        if let Some(max_gas_price) = self.max_gas_price {
            let gas_price = self
                .provider
                .get_gas_price()
                .await
                .expect("unable to fetch gas price");

            if gas_price > max_gas_price {
                warn!(%max_gas_price, %gas_price, "gas price is too high");

                return Err(TxSubmitError::GasPriceTooHigh {
                    max: self.max_gas_price.expect("max gas price is set"),
                    price: gas_price,
                });
            } else {
                info!(%gas_price, "gas price");
            }
        }

        let multicall = Multicall::new(self.multicall_address.into(), signer.clone());

        let ibc = Ibc::new(self.ibc_handler_address.into(), &self.provider);

        let msg_names = ibc_messages
            .iter()
            .map(|x| (x.0.clone(), x.0.name()))
            .collect::<Vec<_>>();

        let mut call = multicall.multicall(
            ibc_messages.clone()
                .into_iter()
                .map(|(_, call)| Call3 {
                    target: self.ibc_handler_address.into(),
                    allowFailure: true,
                    callData: call.calldata().clone(),
                })
                .collect(),
        );

        info!("submitting evm tx");
        let gas_estimate = call.estimate_gas().await.map_err(|e| {
            if ErrorReporter(&e).to_string().contains("gas required exceeds") {
                TxSubmitError::BatchTooLarge
            } else {
                TxSubmitError::Estimate(e)
            }
        })?;
        let gas_to_use = ((gas_estimate as f64) * self.gas_multiplier) as u64;
        info!(
            gas_multiplier = %self.gas_multiplier,
            %gas_estimate,
            %gas_to_use,
            "gas estimation successful"
        );

        if let Some(fixed) = self.fixed_gas_price {
            call = call.gas_price(fixed);
        }

        match call.gas(gas_to_use).send().await {
            Ok(ok) => {
                let tx_hash = <H256>::from(*ok.tx_hash());
                async move {
                    let receipt = ok.get_receipt().await?;
                    info!(%tx_hash, "tx included");

                    Ok(())
                }
                .instrument(info_span!("evm tx", %tx_hash))
                .await
            }

            Err(
            Error::PendingTransactionError(PendingTransactionError::TransportError(TransportError::ErrorResp(e)))
            | Error::TransportError(TransportError::ErrorResp(e)),
        ) if e.message.contains("insufficient funds for gas * price + value") => {
            error!("out of gas");
            return Err(TxSubmitError::OutOfGas);
        }

        Err(
            Error::PendingTransactionError(PendingTransactionError::TransportError(TransportError::ErrorResp(e)))
            | Error::TransportError(TransportError::ErrorResp(e)),
        ) if e.message.contains("oversized data")
           || e.message.contains("exceeds block gas limit")
           || e.message.contains("gas required exceeds") => 
        {
            if ibc_messages.len() == 1 {
                error!(error = %e.message, msg = ?ibc_messages[0], "message is too large");
                return Ok(());
            } else {
                warn!(error = %e.message, "batch is too large");
                return Err(TxSubmitError::BatchTooLarge);
            }
        }

        Err(err) => return Err(TxSubmitError::Error(err)),

        }
    }


    pub async fn send_transaction(
        &self,
        contract: H160,
        msg: Vec<(
            Datagram,
            RawCallBuilder<DynProvider<AnyNetwork>, AnyNetwork>,
        )>,
    ) -> RpcResult<alloy::primitives::FixedBytes<32>> {
        assert!(!msg.is_empty());
        let res = self.keyring
            .with({
                let msg = msg.clone();
                move |wallet| -> _ {
                    AssertUnwindSafe(self.submit_transaction(wallet, msg))
                    }
            }).await;
        
        match res {
            Some(Ok(())) => Ok(alloy::primitives::FixedBytes::<32>([0; 32])),
            Some(Err(e)) => Err(ErrorObject::owned(
                -1,
                format!("transaction submission failed: {:?}", e),
                None::<()>,
            )),
            None => Err(ErrorObject::owned(-1, "no signers available", None::<()>)),
        }
    }
}


pub mod multicall {
    alloy::sol! {
        #![sol(rpc)]

        struct Call3 {
            address target;
            bool allowFailure;
            bytes callData;
        }

        #[derive(Debug)]
        struct Result {
            bool success;
            bytes returnData;
        }

        #[derive(Debug)]
        event MulticallResult(Result[]);

        contract Multicall {
            function multicall(
                Call3[] calldata calls
            ) public payable returns (Result[] memory returnData);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TxSubmitError {
    #[error(transparent)]
    Error(#[from] Error),
    #[error("error estimating gas")]
    Estimate(#[source] Error),
    #[error("error waiting for transaction")]
    PendingTransactionError(#[from] PendingTransactionError),
    #[error("out of gas")]
    OutOfGas,
    #[error("0x revert")]
    EmptyRevert(Vec<Datagram>),
    #[error("gas price is too high: max {max}, price {price}")]
    GasPriceTooHigh { max: u128, price: u128 },
    #[error("rpc error (this is just the IbcDatagram conversion functions but i need to make those errors better)")]
    RpcError(#[from] ErrorObjectOwned),
    #[error("batch too large")]
    BatchTooLarge,
}