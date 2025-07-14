use std::{
    num::{NonZeroU32, NonZeroU8},
    panic::AssertUnwindSafe,
    time::Duration,
};

use cometbft_rpc::rpc_types::{Order, TxResponse};
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::{
    gas::{any, feemarket, fixed},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    BroadcastTxCommitError, TxClient,
};
use cosmos_sdk_event::CosmosSdkEvent;
use ibc_union_spec::{ChannelId, ClientId, ConnectionId, Timestamp};
use protos::{
    cosmos::base::v1beta1::Coin,
    cosmwasm::wasm::v1::{QuerySmartContractStateRequest, QuerySmartContractStateResponse},
};
use serde::{Deserialize, Serialize};
use ucs03_zkgm::msg::{PredictWrappedTokenResponse, QueryMsg};
use unionlabs::{
    self,
    bech32::Bech32,
    google::protobuf::any::mk_any,
    primitives::{encoding::HexUnprefixed, Bytes, H160, H256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow::{self, anyhow, bail, Context},
    jsonrpsee::tracing::info,
    primitives::ChainId,
    serde_json,
    vm::BoxDynError,
};

use crate::helpers;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: KeyringConfig,
    pub rpc_url: String,
    pub gas_config: GasFillerConfig,
    #[serde(default)]
    pub fee_recipient: Option<Bech32<Bytes>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "config")]
pub enum GasFillerConfig {
    Fixed(fixed::GasFiller),
    Feemarket(FeemarketConfig),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FeemarketConfig {
    pub max_gas: u64,
    pub gas_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[derive(Debug)]
pub struct Module {
    pub chain_id: ChainId,
    pub ibc_host_contract_address: Bech32<H256>,
    pub keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,
    pub rpc: Rpc,
    pub gas_config: any::GasFiller,
    pub bech32_prefix: String,
    pub fee_recipient: Option<Bech32<Bytes>>,
}

impl GasFillerConfig {
    async fn into_gas_filler(self, rpc_url: String) -> Result<any::GasFiller, BoxDynError> {
        Ok(match self {
            GasFillerConfig::Fixed(config) => any::GasFiller::Fixed(config),
            GasFillerConfig::Feemarket(config) => any::GasFiller::Feemarket(
                feemarket::GasFiller::new(feemarket::Config {
                    rpc_url,
                    max_gas: config.max_gas,
                    gas_multiplier: config.gas_multiplier,
                    denom: config.denom,
                })
                .await?,
            ),
        })
    }
}

impl Module {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let rpc = Rpc::new(config.rpc_url.clone()).await?;

        let chain_id = rpc.client().status().await?.node_info.network.to_string();

        if chain_id != config.chain_id.as_str() {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

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

        Ok(Self {
            ibc_host_contract_address: config.ibc_host_contract_address,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|entry| {
                    let signer =
                        LocalSigner::new(entry.value().try_into().unwrap(), bech32_prefix.clone());

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            rpc,
            chain_id: ChainId::new(chain_id),
            gas_config: config
                .gas_config
                .into_gas_filler(config.rpc_url.clone())
                .await
                .map_err(|e| anyhow!(e))?,
            bech32_prefix,
            fee_recipient: config.fee_recipient,
        })
    }

    async fn wait_for_event<T, F>(&self, mut filter_fn: F, max_wait: Duration) -> anyhow::Result<T>
    where
        F: FnMut(&ModuleEvent) -> Option<T> + Send + 'static,
        T: Send + 'static,
    {
        let client = self.rpc.client();

        let mut height = client.status().await?.sync_info.latest_block_height - 10;

        tokio::time::timeout(max_wait, async move {
            loop {
                let latest = client.status().await?.sync_info.latest_block_height;
                while height <= latest {
                    let mut page = NonZeroU32::new(1).unwrap();
                    let mut seen = 0;

                    loop {
                        let resp = client
                            .tx_search(
                                format!("tx.height={}", height),
                                false,
                                page,
                                NonZeroU8::new(100).unwrap(),
                                Order::Asc,
                            )
                            .await?;
                        seen += resp.txs.len();

                        for tx in resp.txs {
                            for raw_ev in tx.tx_result.events.into_iter() {
                                // println!("raw event: {raw_ev:?}");
                                let event = match CosmosSdkEvent::<ModuleEvent>::new(raw_ev) {
                                    Ok(event) => event,
                                    Err(cosmos_sdk_event::Error::Deserialize(_error)) => {
                                        // println!("unable to parse event: {error}");
                                        continue;
                                    }
                                    Err(_err) => {
                                        // println!("error parsing event: {}", ErrorReporter(err));
                                        continue;
                                    }
                                };
                                let ibc_evt = event.event;
                                if let Some(found) = filter_fn(&ibc_evt) {
                                    return Ok(found);
                                }
                            }
                        }

                        if seen >= resp.total_count as usize {
                            break;
                        }
                        page = page.checked_add(1).unwrap();
                    }

                    height += 1;
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        })
        .await
        .map_err(|_| anyhow::anyhow!("timed out after {:?}", max_wait))?
    }

    pub async fn wait_for_create_client_id(
        &self,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::CreateClientConfirm> {
        self.wait_for_event(
            |evt| {
                if let ModuleEvent::WasmCreateClient { client_id, .. } = evt {
                    Some(helpers::CreateClientConfirm {
                        client_id: client_id.raw().try_into().unwrap(),
                    })
                } else {
                    None
                }
            },
            max_wait,
        )
        .await
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::ChannelOpenConfirm> {
        self.wait_for_event(
            |evt| {
                if let ModuleEvent::WasmChannelOpenConfirm {
                    channel_id,
                    counterparty_channel_id,
                    ..
                } = evt
                {
                    Some(helpers::ChannelOpenConfirm {
                        channel_id: channel_id.raw().try_into().unwrap(),
                        counterparty_channel_id: counterparty_channel_id.raw().try_into().unwrap(),
                    })
                } else {
                    None
                }
            },
            max_wait,
        )
        .await
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        self.wait_for_event(
            |evt| {
                if let ModuleEvent::WasmConnectionOpenConfirm {
                    connection_id,
                    counterparty_connection_id,
                    ..
                } = evt
                {
                    Some(helpers::ConnectionConfirm {
                        connection_id: connection_id.raw().try_into().unwrap(),
                        counterparty_connection_id: counterparty_connection_id
                            .raw()
                            .try_into()
                            .unwrap(),
                    })
                } else {
                    None
                }
            },
            max_wait,
        )
        .await
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash_param: H256,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        println!("Waiting for packet recv event with hash: {packet_hash_param:?}");
        self.wait_for_event(
            move |evt| {
                if let ModuleEvent::WasmPacketRecv { packet_hash, .. } = evt {
                    println!("Packet recv event came with hash: {packet_hash:?}");
                    // if packet_hash.as_ref() == packet_hash_param.as_ref() {
                    return Some(helpers::PacketRecv {
                        packet_hash: *packet_hash,
                    });
                    // }
                    // None
                } else {
                    None
                }
            },
            max_wait,
        )
        .await
    }

    pub async fn wait_for_delegate(
        &self,
        validator_filter: String,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::Delegate> {
        self.wait_for_event(
            move |evt| {
                if let ModuleEvent::Delegate { validator, .. } = evt {
                    if validator == &validator_filter {
                        Some(helpers::Delegate {
                            validator: validator.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            max_wait,
        )
        .await
    }

    pub async fn predict_wrapped_token(
        &self,
        contract: Bech32<H256>,
        channel_id: ChannelId,
        token: Vec<u8>,
    ) -> anyhow::Result<String> {
        // build the query payload
        let msg = QueryMsg::PredictWrappedToken {
            path: "0".to_string(),
            channel_id,
            token: token.into(),
        };
        let req = QuerySmartContractStateRequest {
            address: contract.to_string(),
            query_data: serde_json::to_vec(&msg)
                .context("serializing PredictWrappedToken QueryMsg")?,
        };

        // execute the ABCI query
        let raw = self
            .rpc
            .client()
            .grpc_abci_query::<_, QuerySmartContractStateResponse>(
                "/cosmwasm.wasm.v1.Query/SmartContractState",
                &req,
                None,
                false,
            )
            .await?
            .into_result()?
            .unwrap()
            .data;

        let resp: PredictWrappedTokenResponse =
            serde_json::from_slice(&raw).context("deserializing PredictWrappedTokenResponse")?;

        // let addr: H160 = H160::from_str(&resp.wrapped_token)
        //     .context("parsing wrapped_token into H160")?;
        Ok(resp.wrapped_token)
    }

    // TODO(aeryz): return the digest
    pub async fn send_transaction(
        &self,
        contract: Bech32<H256>,
        msg: (Vec<u8>, Vec<Coin>),
    ) -> Option<Result<TxResponse, BroadcastTxCommitError>> {
        self.keyring
            .with(|signer| {
                let tx_client = TxClient::new(signer, &self.rpc, &self.gas_config);

                AssertUnwindSafe(async move {
                    match tx_client
                        .broadcast_tx_commit(
                            [msg]
                                .iter()
                                .map(|(x, funds)| {
                                    mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                                        sender: signer.address().to_string(),
                                        contract: contract.to_string(),
                                        msg: x.clone(),
                                        funds: funds.clone(),
                                    })
                                })
                                .collect::<Vec<_>>(),
                            "memo",
                            true,
                        )
                        .await
                    {
                        Ok(tx_response) => {
                            info!(
                                tx_hash = %tx_response.hash,
                                gas_used = %tx_response.tx_result.gas_used,
                                "submitted cosmos transaction"
                            );

                            Ok(tx_response)
                        }
                        Err(err) => {
                            info!(error = %ErrorReporter(&err), "cosmos tx failed");
                            Err(err)
                        }
                    }
                })
            })
            .await
    }

    pub async fn send_ibc_transaction(
        &self,
        contract: Bech32<H256>,
        msg: (Vec<u8>, Vec<Coin>),
    ) -> anyhow::Result<H256> {
        let result = self.send_transaction(contract, msg).await;

        let tx_result = result.ok_or_else(|| anyhow!("failed to send transaction"))??;

        let send_event = tx_result
            .tx_result
            .events
            .into_iter()
            .find_map(|e| {
                if e.ty == "wasm-packet_send" {
                    CosmosSdkEvent::<ModuleEvent>::new(e).ok().map(|e| e.event)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("wasm-packet_send event not found"))?;

        Ok(match send_event {
            ModuleEvent::WasmPacketSend { packet_hash, .. } => packet_hash,
            _ => bail!("unexpected event variant"),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum ModuleEvent {
    #[serde(rename = "delegate")]
    Delegate { validator: String, amount: String },

    #[serde(rename = "wasm-packet_send")]
    WasmPacketSend {
        #[serde(with = "serde_utils::string")]
        packet_source_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        packet_destination_channel_id: ChannelId,
        packet_data: Bytes,
        #[serde(with = "serde_utils::string")]
        packet_timeout_height: u64,
        #[serde(with = "serde_utils::string")]
        packet_timeout_timestamp: Timestamp,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
    },

    #[serde(rename = "wasm-create_client")]
    WasmCreateClient {
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        client_type: String,
    },

    #[serde(rename = "wasm-connection_open_confirm")]
    WasmConnectionOpenConfirm {
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
        #[serde(with = "serde_utils::string")]
        client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_client_id: ClientId,
        #[serde(with = "serde_utils::string")]
        counterparty_connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-channel_open_confirm")]
    WasmChannelOpenConfirm {
        port_id: Bech32<H256>,
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        counterparty_port_id: Bytes<HexUnprefixed>,
        #[serde(with = "serde_utils::string")]
        counterparty_channel_id: ChannelId,
        #[serde(with = "serde_utils::string")]
        connection_id: ConnectionId,
    },

    #[serde(rename = "wasm-packet_recv")]
    WasmPacketRecv {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        maker: Bech32<Bytes>,
        maker_msg: Bytes<HexUnprefixed>,
    },
}
