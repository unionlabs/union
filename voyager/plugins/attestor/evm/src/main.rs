#![doc = include_str!("../README.md")]

use std::{collections::VecDeque, ops::Deref, panic::AssertUnwindSafe, path::PathBuf, sync::Arc};

use alloy::{
    network::AnyNetwork,
    providers::{DynProvider, Provider, ProviderBuilder},
};
use attested_light_client::types::{Attestation, AttestationValue};
use clap::Subcommand;
use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use cosmos_client::{
    BroadcastTxCommitError, TxClient, TxError,
    gas::{any, feemarket, fixed, osmosis_eip1559_feemarket},
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
};
use ed25519_dalek::{SigningKey, ed25519::signature::SignerMut, pkcs8::DecodePrivateKey};
use ibc_union_spec::{
    IbcUnion, Timestamp,
    event::{
        ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit, ChannelOpenTry, ConnectionOpenAck,
        ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry, FullEvent,
    },
    path::{BatchPacketsPath, BatchReceiptsPath, ChannelPath, ConnectionPath},
};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info, info_span, instrument, warn};
use unionlabs::{
    ErrorReporter,
    cosmwasm::wasm::msg_execute_contract::MsgExecuteContract,
    encoding::{Bincode, EncodeAs},
    ethereum::ibc_commitment_key,
    never::Never,
    primitives::{Bech32, H160, H256},
};
use voyager_sdk::{
    anyhow::{self, bail},
    hook::simple_take_filter,
    message::{PluginMessage, VoyagerMessage, data::Data},
    plugin::Plugin,
    primitives::{ChainId, IbcSpec},
    rpc::{PluginServer, RpcError, RpcErrorExt, RpcResult, types::PluginInfo},
    vm::{Op, call, noop, pass::PassResult},
};

use crate::call::{ModuleCall, SubmitAttestation};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module(Arc<ModuleInner>);

impl Deref for Module {
    type Target = ModuleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct ModuleInner {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    pub provider: DynProvider<AnyNetwork>,

    pub attestation_key: SigningKey,

    pub keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,

    pub cosmos_client: cosmos_client::rpc::Rpc,

    pub gas_config: any::GasFiller,

    pub attestation_client_address: Bech32<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,

    /// The address of the `IBCHandler` smart contract.
    pub ibc_handler_address: H160,

    /// The RPC endpoint for the EVM chain being attested to.
    pub eth_rpc_url: String,

    /// The RPC endpoint for the cosmos chain to submit the attestations to.
    pub cosmos_rpc_url: String,

    pub gas_config: GasFillerConfig,

    pub attestation_client_address: Bech32<H256>,

    /// The path to the PKCS#8 encoded private key to sign the attestations with.
    pub attestation_key_path: PathBuf,

    pub keyring: KeyringConfig,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "config")]
pub enum GasFillerConfig {
    // fixed gas filler is it's own config
    Fixed(fixed::GasFiller),
    Feemarket(FeemarketConfig),
    OsmosisEip1559Feemarket(OsmosisEip1559FeemarketConfig),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FeemarketConfig {
    pub max_gas: u64,
    #[serde(with = "::serde_utils::string_opt")]
    pub gas_multiplier: Option<f64>,
    pub denom: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OsmosisEip1559FeemarketConfig {
    pub max_gas: u64,
    #[serde(with = "::serde_utils::string_opt")]
    pub gas_multiplier: Option<f64>,
    #[serde(with = "::serde_utils::string_opt")]
    pub base_fee_multiplier: Option<f64>,
    pub denom: Option<String>,
}

impl GasFillerConfig {
    async fn into_gas_filler(self, rpc_url: String) -> anyhow::Result<any::GasFiller> {
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
            GasFillerConfig::OsmosisEip1559Feemarket(config) => {
                any::GasFiller::OsmosisEip1559Feemarket(
                    osmosis_eip1559_feemarket::GasFiller::new(osmosis_eip1559_feemarket::Config {
                        rpc_url,
                        max_gas: config.max_gas,
                        gas_multiplier: config.gas_multiplier,
                        base_fee_multiplier: config.base_fee_multiplier,
                        denom: config.denom,
                    })
                    .await?,
                )
            }
        })
    }
}

#[derive(Subcommand)]
pub enum Cmd {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = Cmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let rpc = Rpc::new(config.cosmos_rpc_url.clone()).await?;

        let provider = DynProvider::new(
            ProviderBuilder::new()
                .network::<AnyNetwork>()
                .connect(&config.eth_rpc_url)
                .await?,
        );

        let raw_chain_id = provider.get_chain_id().await?;
        let chain_id = ChainId::new(raw_chain_id.to_string());

        if chain_id != config.chain_id {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        let attestation_key = SigningKey::read_pkcs8_pem_file(config.attestation_key_path)?;

        info!(attestation_key = %<H256>::new(attestation_key.verifying_key().to_bytes()));

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

        Ok(Self(Arc::new(ModuleInner {
            chain_id,
            ibc_handler_address: config.ibc_handler_address,
            provider,
            attestation_key,
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
            cosmos_client: rpc,
            attestation_client_address: config.attestation_client_address,
            gas_config: config
                .gas_config
                .into_gas_filler(config.cosmos_rpc_url)
                .await?,
        })))
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: simple_take_filter(format!(
                r#"
if ."@type" == "data" then
    ."@value" as $data |

    # pull all ibc events from this chain, they will be verified and attested to

    $data."@type" == "ibc_event" and $data."@value".chain_id == "{chain_id}" and $data."@value".ibc_spec_id == "{ibc_union_id}"
else
    false
end
"#,
                chain_id = config.chain_id,
                ibc_union_id = IbcUnion::ID,
            )),
        }
    }

    async fn cmd(_: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

fn plugin_name(chain_id: &ChainId) -> String {
    pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

    format!("{PLUGIN_NAME}/{}", chain_id)
}

impl Module {
    fn plugin_name(&self) -> String {
        plugin_name(&self.chain_id)
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, op)| {
                    let op = match op.into_data().unwrap() {
                        Data::IbcEvent(chain_event) => call(PluginMessage::new(
                            plugin_name(&self.chain_id),
                            ModuleCall::SubmitAttestation(SubmitAttestation {
                                event: chain_event.decode_event::<IbcUnion>().unwrap().unwrap(),
                                tx_hash: chain_event.tx_hash,
                                height: chain_event.provable_height.height().height(),
                            }),
                        )),
                        _ => todo!(),
                    };

                    (vec![idx], op)
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitAttestation(SubmitAttestation {
                event,
                tx_hash,
                height,
            }) => {
                self.submit_attestation(event, tx_hash, height).await?;

                Ok(noop())
            }
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    async fn submit_attestation(
        &self,
        event: FullEvent,
        tx_hash: H256,
        height: u64,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let key = match &event {
            FullEvent::ConnectionOpenInit(ConnectionOpenInit { connection_id, .. })
            | FullEvent::ConnectionOpenTry(ConnectionOpenTry { connection_id, .. })
            | FullEvent::ConnectionOpenAck(ConnectionOpenAck { connection_id, .. })
            | FullEvent::ConnectionOpenConfirm(ConnectionOpenConfirm { connection_id, .. }) => {
                ConnectionPath {
                    connection_id: *connection_id,
                }
                .key()
            }
            FullEvent::ChannelOpenInit(ChannelOpenInit { channel_id, .. })
            | FullEvent::ChannelOpenTry(ChannelOpenTry { channel_id, .. })
            | FullEvent::ChannelOpenAck(ChannelOpenAck { channel_id, .. })
            | FullEvent::ChannelOpenConfirm(ChannelOpenConfirm { channel_id, .. }) => ChannelPath {
                channel_id: *channel_id,
            }
            .key(),
            FullEvent::PacketSend(event) => BatchPacketsPath::from_packets(&[event.packet()]).key(),
            FullEvent::BatchSend(event) => BatchPacketsPath {
                batch_hash: event.batch_hash,
            }
            .key(),
            FullEvent::WriteAck(event) => BatchReceiptsPath::from_packets(&[event.packet()]).key(),
            _ => {
                info!(event = %event.name(), "nothing to attest for event");
                return Ok(noop());
            }
        };

        let value = self
            .provider
            .get_storage_at(
                self.ibc_handler_address.get().into(),
                ibc_commitment_key(key).into(),
            )
            .block_id(height.into())
            .await
            .map_err(RpcError::retryable("error fetching storage"))?;

        let tx = self
            .provider
            .get_transaction_by_hash(tx_hash.get().into())
            .await
            .map_err(RpcError::retryable("error fetching source transaction"))?
            .ok_or_else(|| RpcError::missing_state(format_args!("tx {tx_hash} not found")))?;

        let tx_height = tx.block_number.unwrap();

        if tx_height != height {
            return Err(RpcError::fatal_from_message(format_args!(
                "block number is inconsistent; event height \
                is {height} but tx height is {tx_height}"
            )));
        }

        let timestamp = self
            .provider
            .get_block_by_number(height.into())
            .await
            .map_err(RpcError::retryable("error fetching block"))
            .with_data(json!({
                "height": height
            }))?
            .ok_or_else(|| RpcError::missing_state(format_args!("block {height} not found")))?
            .header
            .timestamp;

        info_span!("attesting to state", %key, %value, %height, %timestamp)
            .in_scope(|| {
                self.keyring.with(|signer| {
                    let tx_client = TxClient::new(signer, &self.cosmos_client, &self.gas_config);

                    let attestation = Attestation {
                        chain_id: self.chain_id.to_string(),
                        height,
                        timestamp: Timestamp::from_secs(timestamp),
                        key: key.into(),
                        value: AttestationValue::Existence(value.to_be_bytes::<32>().into()),
                    };

                    let signature = self
                        .attestation_key.clone()
                        .sign(&(&attestation).encode_as::<Bincode>());

                    AssertUnwindSafe(async move {
                        let res = tx_client
                            .tx(
                                MsgExecuteContract {
                                    sender: tx_client.wallet().address().map_data(Into::into),
                                    contract: self.attestation_client_address.clone(),
                                    msg: serde_json::to_vec(
                                        &attested_light_client::msg::ExecuteMsg::Attest {
                                            attestation: attestation.clone(),
                                            attestor: self
                                                .attestation_key
                                                .verifying_key()
                                                .to_bytes()
                                                .into(),
                                            signature: signature.to_bytes().into(),
                                        },
                                    )
                                    .unwrap()
                                    .into(),
                                    funds: vec![],
                                },
                                "",
                                true,
                            )
                            .await;

                        match res {
                            Ok((tx_hash, _)) => {
                                info!(%tx_hash, "submitted attestation");
                                Ok(noop())
                            }
                            Err(TxError::BroadcastTxCommit(BroadcastTxCommitError::TxFailed {
                                codespace,
                                error_code,
                                log,
                            })) => {
                                error!(%codespace, %error_code, %log, "tx failed");
                                Ok(noop())
                            }
                            Err(err) => {
                                warn!(err = %ErrorReporter(&err), "error when submitting tx, will be retried");
                                Err(RpcError::retryable("error when submitting attestation")(err).with_data(json!(attestation)))
                            }
                        }
                    })
                })
            })
            .await
            .unwrap_or_else(|| {
                Ok(call(PluginMessage::new(
                    self.plugin_name(),
                    ModuleCall::SubmitAttestation(SubmitAttestation { event, tx_hash, height }),
                )))
            })
    }
}
