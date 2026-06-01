// #![warn(clippy::unwrap_used)]
#![feature(if_let_guard)]

use std::{
    collections::{BTreeMap, VecDeque},
    ops::Deref,
    panic::AssertUnwindSafe,
    sync::Arc,
};

use concurrent_keyring::{ConcurrentKeyring, KeyringConfig, KeyringEntry};
use gno_client::{
    BroadcastTxCommitError, TxClient,
    gas::fixed,
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
};
use gno_rpc::rpc_types::{MemFile, MemPackage, Msg, MsgRun, TxFee};
use ibc_union_spec::{Packet, datagram::Datagram};
use jsonrpsee::{Extensions, MethodsError, core::async_trait, proc_macros::rpc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use unionlabs::{
    self, ErrorReporter,
    never::Never,
    primitives::{Bech32, Bytes, H160, encoding::HexPrefixed},
};
use voyager_sdk::{
    DefaultCmd,
    anyhow::{self, bail},
    hook::SubmitTxHook,
    into_value,
    message::{PluginMessage, VoyagerMessage, data::Data},
    plugin::Plugin,
    primitives::ChainId,
    rpc::{PluginServer, RpcError, RpcResult, types::PluginInfo},
    vm::{Op, Visit, noop, pass::PassResult},
};

use crate::call::{IbcMessage, ModuleCall};

pub mod call;

pub mod client {}

#[tokio::main]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module(Arc<ModuleInner>);

#[derive(Debug)]
pub struct ModuleInner {
    pub chain_id: ChainId,
    pub ibc_core_realm: String,
    pub keyring: ConcurrentKeyring<Bech32<H160>, LocalSigner>,
    pub rpc: Rpc,
    pub fee: TxFee,
    // pub gas_config: any::GasFiller,
    pub fee_recipient: Option<Bech32<H160>>,
    pub max_tx_size: u32,
}

impl Deref for Module {
    type Target = ModuleInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub chain_id: ChainId,
    pub ibc_core_realm: String,
    pub keyring: KeyringConfig,
    pub rpc_url: String,
    pub fee: TxFee,
    // pub gas_config: GasFillerConfig,
    #[serde(default)]
    pub fee_recipient: Option<Bech32<H160>>,
    pub max_tx_size: u32,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        let rpc = Rpc::new(config.rpc_url.clone()).await?;

        let chain_id = rpc.client().status(None).await?.node_info.network;

        if chain_id != config.chain_id.as_str() {
            bail!(
                "incorrect chain id: expected `{}`, but found `{}`",
                config.chain_id,
                chain_id
            );
        }

        Ok(Self(Arc::new(ModuleInner {
            ibc_core_realm: config.ibc_core_realm,
            keyring: ConcurrentKeyring::new(
                config.keyring.name,
                config.keyring.keys.into_iter().map(|entry| {
                    let signer =
                        LocalSigner::new_from_private_key(entry.value().try_into().unwrap(), "g");

                    KeyringEntry {
                        address: signer.address(),
                        signer,
                    }
                }),
            ),
            rpc,
            fee: config.fee,
            chain_id: ChainId::new(chain_id),
            fee_recipient: config.fee_recipient,
            max_tx_size: config.max_tx_size,
        })))
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: plugin_name(&config.chain_id),
            interest_filter: SubmitTxHook::filter(&config.chain_id),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

// TODO: Currently duplicated between here and the ethereum tx plugin, deduplicate
#[rpc(server)]
trait TransactionPlugin {
    #[method(name = "signerAddresses")]
    async fn signer_addresses(&self) -> RpcResult<Vec<Bech32<H160>>>;

    #[method(name = "signerBalances")]
    async fn signer_balances(&self) -> RpcResult<BTreeMap<Bech32<H160>, String>>;
}

#[async_trait]
impl TransactionPluginServer for Module {
    async fn signer_addresses(&self) -> RpcResult<Vec<Bech32<H160>>> {
        Ok(self.keyring.keys().cloned().collect())
    }

    async fn signer_balances(&self) -> RpcResult<BTreeMap<Bech32<H160>, String>> {
        let mut out = BTreeMap::new();

        for address in self.keyring.keys() {
            let account = self
                .rpc
                .client()
                .account_info(address)
                .await
                .map_err(RpcError::retryable("error fetching account"))?
                .ok_or_else(|| {
                    RpcError::retryable_from_message("empty response when fetching account")
                })?;

            debug!("raw balance: {}", account.base_account.coins);

            let balance = account
                .base_account
                .coins
                .strip_suffix("ugnot")
                .ok_or_else(|| {
                    RpcError::retryable_from_message("invalid coins when fetching balance")
                })?;

            out.insert(address.clone(), balance.to_owned());
        }

        Ok(out)
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

    pub async fn do_send_transaction(
        &self,
        msgs: Vec<IbcMessage>,
    ) -> Option<Result<Option<Op<VoyagerMessage>>, BroadcastTxCommitError>> {
        self.keyring
            .with(|signer| {
                let msgs = msgs.clone();

                trace!(?msgs);

                // TODO: Figure out a way to thread this value through
                let memo = format!("Voyager {}", env!("CARGO_PKG_VERSION"));

                let ibc_core_realm = self.ibc_core_realm.clone();
                let msgs = process_msgs(
                    msgs,
                    signer,
                    ibc_core_realm,
                    // self.fee_recipient.as_ref(),
                    None,
                );

                let msgs = msgs
                    .into_iter()
                    .filter_map(|msg| match msg {
                        Ok(msg) => Some(msg),
                        Err(err) => {
                            error!("invalid msg: {err}");
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                let tx_client = TxClient::new(signer, &self.rpc, fixed::GasFiller::default());

                let batch_size = msgs.len();
                let msg_names = msgs.iter().map(|x| x.0.name()).collect::<Vec<_>>();

                // let approximate_size = msgs.iter().map(|x| x.1.encoded_len()).sum::<usize>();

                // info!(
                //     %approximate_size,
                //     max_tx_size = %self.max_tx_size,
                //     "approximate tx size"
                // );

                AssertUnwindSafe(async move {
                    if msgs.is_empty() {
                        info!("no msgs left to submit after filtering out invalid msgs");
                        return Ok(None);
                    }

                    // if approximate_size > self.max_tx_size as usize {
                    //     if msgs.len() == 1 {
                    //         error!(
                    //             %approximate_size,
                    //             max_tx_size = %self.max_tx_size,
                    //             msg = msgs.first().unwrap().0.name(),
                    //             "message is too large, dropping as it cannot be submitted"
                    //         );
                    //         return Ok(None);
                    //     } else {
                    //         warn!(
                    //             %approximate_size,
                    //             max_tx_size = %self.max_tx_size,
                    //             "tx is too large, splitting messages"
                    //         );

                    //         let mut msgs = msgs.into_iter().map(|x| x.0).collect::<Vec<_>>();

                    //         let new_msgs = msgs.split_off(msgs.len().div_ceil(2));

                    //         return Ok(Some(seq([
                    //             call(PluginMessage::new(
                    //                 self.plugin_name(),
                    //                 ModuleCall::SubmitTransaction(msgs),
                    //             )),
                    //             // ensure that the first half gets included
                    //             defer_relative(10),
                    //             call(PluginMessage::new(
                    //                 self.plugin_name(),
                    //                 ModuleCall::SubmitTransaction(new_msgs),
                    //             )),
                    //         ])));
                    //     }
                    // };

                    match tx_client
                        .broadcast_tx_commit(
                            msgs.iter().map(move |x| x.1.clone()).collect::<Vec<_>>(),
                            memo,
                            self.fee.clone(),
                        )
                        .await
                    {
                        Ok(tx_response) => {
                            info!(
                                height = tx_response.height,
                                tx_hash = %tx_response.hash,
                                gas_used = %tx_response.deliver_tx.gas_used,
                                gas_wanted = %tx_response.deliver_tx.gas_wanted,
                                batch.size = %batch_size,
                                "submitted gno transaction"
                            );

                            for msg in msg_names {
                                info!(
                                    tx_hash = %tx_response.hash,
                                    tx_hash_hex = %tx_response.hash.into_encoding::<HexPrefixed>(),
                                    %msg,
                                    "gno msg"
                                );
                            }

                            Ok(None)
                        }
                        Err(err) => {
                            info!(error = %ErrorReporter(&err), "gno tx failed");
                            Err(err)
                        }
                    }
                })
            })
            .await
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all)]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let mut hook = SubmitTxHook::new(&self.chain_id, |submit_tx| {
            PluginMessage::new(
                self.plugin_name(),
                ModuleCall::SubmitTransaction(
                    submit_tx
                        .datagrams
                        .clone()
                        .into_iter()
                        .map(IbcMessage::from_raw_datagram)
                        .collect::<Result<_, _>>()
                        .unwrap(),
                ),
            )
            .into()
        });

        debug!(msgs = msgs.len(), "optimizing messages");

        Ok(PassResult {
            optimize_further: vec![],
            ready: msgs
                .into_iter()
                .enumerate()
                .map(|(idx, mut op)| {
                    hook.visit_op(&mut op);

                    (vec![idx], op)
                })
                .collect(),
        })
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn call(&self, _: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        match msg {
            ModuleCall::SubmitTransaction(msgs) => {
                let batch_submission_result = self.do_send_transaction(msgs.clone()).await;

                match batch_submission_result {
                    None => Err(RpcError::retryable_from_message("no signers available")),
                    Some(Ok(None)) => {
                        for (idx, msg) in msgs.into_iter().enumerate() {
                            info!(
                                msg = msg.name(),
                                %idx,
                                data = %into_value(&msg),
                                "gno tx",
                            );
                        }
                        Ok(noop())
                    }
                    Some(Ok(Some(op))) => Ok(op),
                    Some(Err(err)) => match err {
                        _ if let Some(err) = err.as_json_rpc_error() => {
                            return Err(RpcError::retryable("jsonrpc error")(err));
                        }
                        BroadcastTxCommitError::TxFailed { error, log } => {
                            info!(%log, "error submitting gno tx: {}", ErrorReporter(error));

                            let _span = info_span!("gno msg failed").entered();
                            info!(%log, "tx log");

                            warn!("error submitting transaction: {log}");

                            // TODO: Add back more sophisticated checks
                            Err(RpcError::retryable_from_message(format!(
                                "error submitting tx, tx failed: {log}"
                            )))

                            // if msgs.len() == 1 {
                            //     warn!(msg = %into_value(msgs.pop().unwrap()), "gno msg failed");

                            //     Ok(noop())
                            // } else {
                            //     let failed_msg = msgs.remove(msg_idx);

                            //     if matches!(
                            //         failed_msg,
                            //         IbcMessage::IbcUnion(Datagram::UpdateClient(_))
                            //     ) {
                            //         warn!(
                            //             "update client failed, this may cause other messages to fail as well"
                            //         );
                            //     }

                            //     warn!(msg = %into_value(failed_msg), "dropping failed msg");

                            //     if msgs.is_empty() {
                            //         info!("no messages to submit after dropping failed messages");

                            //         Ok(noop())
                            //     } else {
                            //         Ok(call(PluginMessage::new(
                            //             self.plugin_name(),
                            //             ModuleCall::SubmitTransaction(msgs),
                            //         )))
                            //     }
                            // }
                        }
                        err => Err(RpcError::retryable("error submitting tx")(err)),
                    },
                }
            }
        }
    }

    #[instrument(skip_all)]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }

    #[instrument(skip_all, fiellds(chain_id = %self.chain_id))]
    async fn custom(&self, _: &Extensions, method: String, params: Vec<Value>) -> RpcResult<Value> {
        TransactionPluginServer::into_rpc(self.clone())
            .call::<Vec<Value>, Value>(&method, params)
            .await
            .map_err(|e| match e {
                MethodsError::Parse(e) => RpcError::fatal("error parsing args")(e),
                MethodsError::JsonRpc(error) => {
                    RpcError::from_parts(error.code(), error.message(), error.data())
                }
                MethodsError::InvalidSubscriptionId(_) => {
                    RpcError::fatal_from_message("subscriptions are not supported")
                }
            })
    }
}

fn process_msgs(
    msgs: Vec<IbcMessage>,
    signer: &LocalSigner,
    ibc_core_realm: String,
    fee_recipient: Option<&Bech32<Bytes>>,
) -> Vec<RpcResult<(IbcMessage, Msg)>> {
    msgs.into_iter()
        .map(|IbcMessage::IbcUnion(msg)| {
            let signer = signer.address().to_string();

            let relayer = fee_recipient.map_or(signer.clone(), |fr| fr.to_string());

            let body = match msg.clone() {
                Datagram::CreateClient(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	id := core.CreateClient(cross, core.MsgCreateClient{{
    	ClientType: "{}",
    	ClientStateBytes: {},
    	ConsensusStateBytes: {},
    	// Relayer: = {},
	}})
	println(id)
}}
                    "#,
                        msg.client_type,
                        gno_bytes(&msg.client_state_bytes),
                        gno_bytes(&msg.consensus_state_bytes),
                        relayer,
                    )
                }
                Datagram::UpdateClient(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	height := core.UpdateClient(cross, core.MsgUpdateClient{{
    	ClientId: core.ClientId({}),
    	ClientMessage: {},
    	// Relayer: {},
	}})
	println(height)
}}
                    "#,
                        msg.client_id,
                        gno_bytes(&msg.client_message),
                        relayer,
                    )
                }
                Datagram::ConnectionOpenInit(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	height := core.ConnectionOpenInit(cross, core.MsgConnectionOpenInit{{
    	ClientId: core.ClientId({}),
    	CounterpartyClientId: core.ClientId({}),
	}})
	println(height)
}}
                    "#,
                        msg.client_id, msg.counterparty_client_id,
                    )
                }
                Datagram::ConnectionOpenTry(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ConnectionOpenTry(cross, core.MsgConnectionOpenTry{{
    	ClientId: core.ClientId({}),
    	CounterpartyClientId: core.ClientId({}),
    	CounterpartyConnectionId: core.ConnectionId({}),
    	ProofInit: {},
    	ProofHeight: core.Height({}),
	}})
}}
                    "#,
                        msg.client_id,
                        msg.counterparty_client_id,
                        msg.counterparty_connection_id,
                        gno_bytes(&msg.proof_init),
                        msg.proof_height,
                    )
                }
                Datagram::ConnectionOpenAck(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ConnectionOpenAck(cross, core.MsgConnectionOpenAck{{
    	ConnectionId: core.ConnectionId({}),
    	CounterpartyConnectionId: core.ConnectionId({}),
    	ProofTry: {},
    	ProofHeight: core.Height({}),
	}})
}}
                    "#,
                        msg.connection_id,
                        msg.counterparty_connection_id,
                        gno_bytes(&msg.proof_try),
                        msg.proof_height,
                    )
                }
                Datagram::ConnectionOpenConfirm(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ConnectionOpenConfirm(cross, core.MsgConnectionOpenConfirm{{
    	ConnectionId: core.ConnectionId({}),
    	ProofAck: {},
    	ProofHeight: core.Height({}),
	}})
}}
                    "#,
                        msg.connection_id,
                        gno_bytes(&msg.proof_ack),
                        msg.proof_height,
                    )
                }
                Datagram::ChannelOpenInit(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ChannelOpenInit(cross, core.MsgChannelOpenInit{{
    	PortId: {},
    	CounterpartyPortId: {},
    	ConnectionId: {},
    	Version: "{}",
	}})
}}
                    "#,
                        gno_bytes(&msg.port_id),
                        gno_bytes(&msg.counterparty_port_id),
                        msg.connection_id,
                        msg.version,
                    )
                }
                Datagram::ChannelOpenTry(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ChannelOpenTry(cross, core.MsgChannelOpenTry{{
    	PortId: {},
    	Channel: core.Channel{{
        	State: {},
        	ConnectionId: core.ConnectionId({}),
        	CounterpartyChannelId: core.ChannelId({}),
        	CounterpartyPortId: {},
        	Version: "{}",
    	}},
    	CounterpartyVersion: "{}",
    	ProofInit: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        gno_bytes(&msg.port_id),
                        msg.channel.state as u8,
                        msg.channel.connection_id,
                        msg.channel
                            .counterparty_channel_id
                            .expect("must be present"),
                        gno_bytes(&msg.channel.counterparty_port_id),
                        msg.channel.version,
                        msg.counterparty_version,
                        gno_bytes(&msg.proof_init),
                        msg.proof_height,
                    )
                }
                Datagram::ChannelOpenAck(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ChannelOpenAck(cross, core.MsgChannelOpenAck{{
    	ChannelId: core.ChannelId({}),
    	CounterpartyVersion: "{}",
    	CounterpartyChannelId: core.ChannelId({}),
    	ProofTry: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        msg.channel_id,
                        msg.counterparty_version,
                        msg.counterparty_channel_id,
                        gno_bytes(&msg.proof_try),
                        msg.proof_height,
                    )
                }
                Datagram::ChannelOpenConfirm(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.ChannelOpenConfirm(cross, core.MsgChannelOpenConfirm{{
    	ChannelId: core.ChannelId({}),
    	ProofAck: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        msg.channel_id,
                        gno_bytes(&msg.proof_ack),
                        msg.proof_height,
                    )
                }
                Datagram::ChannelCloseInit(_msg) => todo!(),
                Datagram::ChannelCloseConfirm(_msg) => todo!(),
                Datagram::PacketRecv(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.PacketRecv(cross, core.MsgPacketRecv{{
    	Packets: []core.Packet{{ {} }},
    	RelayerMsgs: [][]byte{{ {} }},
    	Proof: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        msg.packets
                            .iter()
                            .map(gno_packet)
                            .collect::<Vec<_>>()
                            .join(","),
                        msg.relayer_msgs
                            .iter()
                            .map(gno_bytes)
                            .collect::<Vec<_>>()
                            .join(","),
                        gno_bytes(msg.proof),
                        msg.proof_height
                    )
                }
                Datagram::PacketAcknowledgement(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.PacketAcknowledgement(cross, core.MsgPacketAcknowledgement{{
    	Packets: []core.Packet{{ {} }},
    	Acknowledgements: [][]byte{{ {} }},
    	Proof: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        msg.packets
                            .iter()
                            .map(gno_packet)
                            .collect::<Vec<_>>()
                            .join(","),
                        msg.acknowledgements
                            .iter()
                            .map(gno_bytes)
                            .collect::<Vec<_>>()
                            .join(","),
                        gno_bytes(msg.proof),
                        msg.proof_height
                    )
                }
                Datagram::PacketTimeout(msg) => {
                    format!(
                        r#"
package main

import (
    core "{ibc_core_realm}"
)

func main() {{
	core.PacketTimeout(cross, core.MsgPacketTimeout{{
    	Packet: core.Packet{{
        	SourceChannelId: core.ChannelId({}),
        	DestinationChannelId: core.ChannelId({}),
        	Data: {},
        	TimeoutHeight: 0,
        	TimeoutTimestamp: core.Timestamp({}),
    	}},
    	Proof: {},
    	ProofHeight: {},
	}})
}}
                    "#,
                        msg.packet.source_channel_id,
                        msg.packet.destination_channel_id,
                        gno_bytes(&msg.packet.data),
                        msg.packet.timeout_timestamp,
                        gno_bytes(&msg.proof),
                        msg.proof_height,
                    )
                }
                Datagram::IntentPacketRecv(_msg) => todo!(),
                Datagram::BatchSend(_msg) => todo!(),
                Datagram::BatchAcks(_msg) => todo!(),
                Datagram::CommitMembershipProof(_msg) => {
                    return Err(RpcError::fatal_from_message(
                        "CommitMembershipProof is not supported on gno",
                    ));
                }
                Datagram::CommitNonMembershipProof(_msg) => {
                    return Err(RpcError::fatal_from_message(
                        "CommitNonMembershipProof is not supported on gno",
                    ));
                }
                Datagram::CommitPacketTimeout(_msg) => {
                    return Err(RpcError::fatal_from_message(
                        "CommitPacketTimeout is not supported on gno",
                    ));
                }
            };

            info!("MsgRun body: {body}");

            Ok((
                IbcMessage::IbcUnion(msg),
                Msg::Run(MsgRun {
                    caller: signer,
                    send: "".to_owned(),
                    max_deposit: "".to_owned(),
                    package: MemPackage {
                        name: "main".to_owned(),
                        path: "".to_owned(),
                        files: [MemFile {
                            name: "main.gno".to_owned(),
                            body,
                        }]
                        .to_vec(),
                        r#type: None,
                        info: None,
                    },
                }),
            ))
        })
        .collect()
}

fn gno_bytes(bz: impl AsRef<[u8]>) -> String {
    format!(
        "[]byte{{{}}}",
        bz.as_ref()
            .iter()
            .map(|b| b.to_string() + ", ")
            .collect::<String>()
    )
}

fn gno_packet(packet: &Packet) -> String {
    format!(
        "core.Packet{{ SourceChannelId: core.ChannelId({}), DestinationChannelId: core.ChannelId({}), Data: {}, TimeoutHeight: core.Height(0), TimeoutTimestamp: core.Timestamp({}) }}",
        packet.source_channel_id,
        packet.destination_channel_id,
        gno_bytes(&packet.data),
        packet.timeout_timestamp,
    )
}

#[cfg(test)]
mod tests {
    use concurrent_keyring::KeyringConfigEntry;
    use gno_rpc::rpc_types::TxFee;

    use super::*;

    #[test]
    fn config_parse() {
        let json = r#"{
            "chain_id": "chain_id",
            "ibc_core_realm": "oogabooga",
            "keyring": {
              "keys": [
                {
                  "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
                  "name": "name",
                  "type": "raw"
                }
              ],
              "name": "name"
            },
            "rpc_url": "rpc_url",
            "max_tx_size": 1000000,
            "fee": {
              "gas_wanted": "100",
              "gas_fee": "1ugnot"
            }
          }"#;

        let config = serde_json::from_str::<Config>(json).unwrap();

        assert_eq!(
            config,
            Config {
                chain_id: ChainId::new("chain_id"),
                ibc_core_realm: "oogabooga".to_owned(),
                keyring: KeyringConfig {
                    name: "name".to_string(),
                    keys: vec![KeyringConfigEntry::Raw {
                        name: "name".to_string(),
                        key: vec![0; 32],
                    }]
                },
                rpc_url: "rpc_url".to_string(),
                fee_recipient: None,
                max_tx_size: 1000000,
                fee: TxFee {
                    gas_wanted: 100,
                    gas_fee: "1ugnot".to_owned(),
                },
            }
        );
    }
}
