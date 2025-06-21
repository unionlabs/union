use std::panic::AssertUnwindSafe;

use cometbft_rpc::rpc_types::TxResponse;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::{
    gas::{any, feemarket, fixed},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    BroadcastTxCommitError, TxClient,
};
use cosmos_sdk_event::CosmosSdkEvent;
use ibc_union_spec::{event::PacketSend, ChannelId, Timestamp};
use protos::cosmos::base::v1beta1::Coin;
use serde::{Deserialize, Serialize};
use unionlabs::{
    self,
    bech32::Bech32,
    encoding::{Encode, Json, Proto},
    google::protobuf::any::mk_any,
    primitives::{Bytes, H160, H256},
    ErrorReporter,
};
use voyager_sdk::{
    anyhow::{self, anyhow, bail},
    jsonrpsee::tracing::info,
    primitives::ChainId,
    vm::BoxDynError,
};

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
    // fixed gas filler is it's own config
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

    // TODO(aeryz): return the digest
    pub async fn send_transaction(
        &self,
        contract: Bech32<H256>,
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone>, Vec<Coin>)>,
    ) -> Option<Result<TxResponse, BroadcastTxCommitError>> {
        assert!(!funded_msgs.is_empty());
        self.keyring
            .with(|signer| {
                let tx_client = TxClient::new(signer, &self.rpc, &self.gas_config);

                let batch_size = funded_msgs.len();

                AssertUnwindSafe(async move {
                    match tx_client
                        .broadcast_tx_commit(
                            funded_msgs
                                .iter()
                                .map(|(x, funds)| {
                                    mk_any(&protos::cosmwasm::wasm::v1::MsgExecuteContract {
                                        sender: signer.address().to_string(),
                                        contract: contract.to_string(),
                                        msg: x.clone().encode(),
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
                                batch.size = %batch_size,
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
        funded_msgs: Vec<(Box<impl Encode<Json> + Clone>, Vec<Coin>)>,
    ) -> Option<Result<IbcEvent, BroadcastTxCommitError>> {
        let tx_result = match self.send_transaction(contract, funded_msgs).await? {
            Ok(tx_result) => tx_result,
            Err(e) => return Some(Err(e)),
        };

        println!("tx_result: {tx_result:?}");

        // TODO(aeryz): this should be an error
        let send_event = tx_result.tx_result.events.into_iter().find_map(|e| {
            if e.ty == "wasm-packet_send" {
                let event = CosmosSdkEvent::<IbcEvent>::new(e).ok()?.event;
                Some(event)
            } else {
                None
            }
        })?;

        Some(Ok(send_event))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "attributes")]
pub enum IbcEvent {
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
}
