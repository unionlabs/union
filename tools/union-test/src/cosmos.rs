use std::{
    num::{NonZeroU32, NonZeroU8},
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
    cosmos::bank::v1beta1::{
        QueryBalanceRequest, QueryBalanceResponse, QueryAllBalancesRequest, QueryAllBalancesResponse,
    },
};
use serde::{Deserialize, Serialize};
use ucs03_zkgm::msg::{PredictWrappedTokenResponse, QueryMsg};
use unionlabs::{
    self,
    google::protobuf::any::mk_any,
    primitives::{encoding::HexUnprefixed, Bech32, Bytes, H160, H256},
};
use voyager_sdk::{
    anyhow::{self, anyhow, bail, Context},
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
    pub privileged_acc_keyring: KeyringConfig,
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
    pub privileged_acc_keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,
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
            privileged_acc_keyring: ConcurrentKeyring::new(
                config.privileged_acc_keyring.name,
                config.privileged_acc_keyring.keys.into_iter().map(|entry| {
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

    pub async fn native_balance(&self, address: Bech32<H256>, token: &str) -> anyhow::Result<u128> {
        let balance: u128 = self
            .rpc
            .client()
            .grpc_abci_query::<_, protos::cosmos::bank::v1beta1::QueryBalanceResponse>(
                "/cosmos.bank.v1beta1.Query/Balance",
                &protos::cosmos::bank::v1beta1::QueryBalanceRequest {
                    address: address.to_string(),
                    denom: token.to_string(),
                },
                None,
                false,
            )
            .await?
            .into_result()?
            .unwrap()
            .balance
            .unwrap()
            .amount
            .parse()
            .unwrap();

        Ok(balance)
    }

    async fn wait_for_event<T, F>(
        &self,
        mut filter_fn: F,
        max_wait: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<T>>
    where
        F: FnMut(&ModuleEvent) -> Option<T> + Send + 'static,
        T: Send + 'static,
    {
        let client = self.rpc.client();

        let mut events = Vec::new();
        let mut height = client.status().await?.sync_info.latest_block_height - 10;

        tokio::time::timeout(max_wait, async move {
            loop {
                if events.len() >= expected_event_count {
                    return Ok(events);
                }
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
                                    events.push(found);
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
        Ok(self
            .wait_for_event(
                |evt| {
                    if let ModuleEvent::WasmCreateClient { client_id, .. } = evt {
                        Some(helpers::CreateClientConfirm {
                            client_id: client_id.raw(),
                        })
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_channel_open_confirm(
        &self,
        max_wait: Duration,
        expected_event_count: usize,
    ) -> anyhow::Result<Vec<helpers::ChannelOpenConfirm>> {
        self.wait_for_event(
            |evt| {
                if let ModuleEvent::WasmChannelOpenConfirm {
                    channel_id,
                    counterparty_channel_id,
                    ..
                } = evt
                {
                    Some(helpers::ChannelOpenConfirm {
                        channel_id: channel_id.raw(),
                        counterparty_channel_id: counterparty_channel_id.raw(),
                    })
                } else {
                    None
                }
            },
            max_wait,
            expected_event_count,
        )
        .await
    }

    pub async fn wait_for_connection_open_confirm(
        &self,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::ConnectionConfirm> {
        Ok(self
            .wait_for_event(
                |evt| {
                    if let ModuleEvent::WasmConnectionOpenConfirm {
                        connection_id,
                        counterparty_connection_id,
                        ..
                    } = evt
                    {
                        Some(helpers::ConnectionConfirm {
                            connection_id: connection_id.raw(),
                            counterparty_connection_id: counterparty_connection_id.raw(),
                        })
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_packet_recv(
        &self,
        packet_hash_param: H256,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::PacketRecv> {
        println!("Waiting for packet recv event with hash: {packet_hash_param:?}");
        Ok(self
            .wait_for_event(
                move |evt| {
                    if let ModuleEvent::WasmPacketRecv { packet_hash, .. } = evt {
                        println!("Packet recv event came with hash: {packet_hash:?}");
                        if packet_hash.as_ref() == packet_hash_param.as_ref() {
                            return Some(helpers::PacketRecv {
                                packet_hash: *packet_hash,
                            });
                        }
                        None
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_packet_timeout(
        &self,
        packet_hash_param: H256,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::PacketTimeout> {
        println!("Waiting for packet timeout event with hash: {packet_hash_param:?}");
        Ok(self
            .wait_for_event(
                move |evt| {
                    if let ModuleEvent::WasmPacketTimeout { packet_hash, .. } = evt {
                        println!("Packet timeout event came with hash: {packet_hash:?}");
                        if packet_hash.as_ref() == packet_hash_param.as_ref() {
                            return Some(helpers::PacketTimeout {
                                packet_hash: *packet_hash,
                            });
                        }
                        None
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_packet_ack(
        &self,
        packet_hash_param: H256,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::PacketAck> {
        println!("Waiting for packet ack event with hash: {packet_hash_param:?}");
        Ok(self
            .wait_for_event(
                move |evt| {
                    if let ModuleEvent::WasmPacketAck { packet_hash, .. } = evt {
                        if packet_hash.as_ref() == packet_hash_param.as_ref() {
                            return Some(helpers::PacketAck {
                                packet_hash: *packet_hash,
                            });
                        }
                        None
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_delegate(
        &self,
        validator_filter: String,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::Delegate> {
        Ok(self
            .wait_for_event(
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
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn wait_for_withdraw_rewards(
        &self,
        validator_filter: String,
        max_wait: Duration,
    ) -> anyhow::Result<helpers::WithdrawRewards> {
        Ok(self
            .wait_for_event(
                move |evt| {
                    println!("EVT is: {:?}", evt);
                    if let ModuleEvent::WithdrawRewards { validator, amount } = evt {
                        if validator == &validator_filter {
                            Some(helpers::WithdrawRewards {
                                validator: validator.clone(),
                                amount: amount.clone(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                max_wait,
                1,
            )
            .await?
            .pop()
            .unwrap())
    }

    pub async fn predict_wrapped_token(
        &self,
        contract: Bech32<H256>,
        channel_id: ChannelId,
        token: Vec<u8>,
    ) -> anyhow::Result<String> {
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

        Ok(resp.wrapped_token)
    }

    pub async fn get_signer(&self) -> (Bech32<H160>, &LocalSigner) {
        loop {
            if let Some(signer) = self.keyring.with(|s| async move { s }).await {
                let address = signer.address();
                return (address, signer);
            } else {
                // no signer yet, wait and retry
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
    }

    pub async fn get_balance(
        &self,
        address: impl Into<String>,
        denom: &str,
    ) -> anyhow::Result<protos::cosmos::base::v1beta1::Coin> {
        let req = QueryBalanceRequest {
            address: address.into(),
            denom: denom.to_string(),
        };
        let resp: QueryBalanceResponse = self
            .rpc
            .client()
            .grpc_abci_query(
                "/cosmos.bank.v1beta1.Query/Balance",
                &req,
                None,
                false,
            )
            .await?
            .into_result()?
            .unwrap();
        resp.balance
            .ok_or_else(|| anyhow::anyhow!("no balance for denom {}", denom))
    }
    
    pub async fn send_transaction_with_retry(
        &self,
        contract: Bech32<H256>,
        msg: (Vec<u8>, Vec<Coin>),
        signer: &LocalSigner,
    ) -> Option<Result<TxResponse, BroadcastTxCommitError>> {
        let max_retries = 5;
        for attempt in 1..=max_retries {
            let outcome = self
                .send_transaction(contract.clone(), msg.clone(), signer)
                .await;

            if let Some(Ok(_)) = &outcome {
                return outcome;
            }

            if let Some(Err(err)) = &outcome {
                if self.is_sequence_mismatch(err) && attempt < max_retries {
                    tracing::warn!(
                        "send_transaction attempt {}/{} failed with sequence mismatch, {:?}",
                        attempt,
                        max_retries,
                        err
                    );
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                } else {
                    return outcome;
                }
            }
        }

        None
    }

    // TODO(aeryz): return the digest
    pub async fn send_transaction(
        &self,
        contract: Bech32<H256>,
        msg: (Vec<u8>, Vec<Coin>),
        signer: &LocalSigner,
    ) -> Option<Result<TxResponse, BroadcastTxCommitError>> {
        let tx_client = TxClient::new(signer, &self.rpc, &self.gas_config);

        let signer_address = signer.address();
        let outcome = tx_client
            .broadcast_tx_commit(
                [msg]
                    .iter()
                    .map(|(x, funds)| {
                        mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                            sender: signer_address.to_string(),
                            contract: contract.to_string(),
                            msg: x.clone(),
                            funds: funds.clone(),
                        })
                    })
                    .collect::<Vec<_>>(),
                "memo",
                true,
            )
            .await;

        Some(outcome)
    }

    /// Helper to detect the ABCI “account sequence mismatch” error.
    fn is_sequence_mismatch(&self, err: &BroadcastTxCommitError) -> bool {
        match err {
            BroadcastTxCommitError::Query(grpc_err) => {
                grpc_err.log.contains("account sequence mismatch")
            }
            _ => false,
        }
    }

    pub async fn send_ibc_transaction(
        &self,
        contract: Bech32<H256>,
        msg: (Vec<u8>, Vec<Coin>),
        signer: &LocalSigner,
    ) -> anyhow::Result<H256> {
        let result = self.send_transaction(contract, msg, signer).await;

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

    #[serde(rename = "withdraw_rewards")]
    WithdrawRewards { validator: String, amount: String },

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
    #[serde(rename = "wasm-packet_timeout")]
    WasmPacketTimeout {
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

    #[serde(rename = "wasm-packet_ack")]
    WasmPacketAck {
        #[serde(with = "serde_utils::string")]
        channel_id: ChannelId,
        packet_hash: H256,
        acknowledgement: Bytes<HexUnprefixed>,
    },
}
