use std::collections::VecDeque;

use ibc_union_spec::{
    IbcUnion,
    datagram::{Datagram, MsgPacketTimeout},
    event::FullEvent,
    path::BatchReceiptsPath,
};
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument};
use unionlabs::{self, never::Never};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient, anyhow,
    message::{
        PluginMessage, VoyagerMessage,
        call::{FetchUpdateHeaders, SubmitTx},
        callback::AggregateSubmitTxFromOrderedHeaders,
        data::{Data, IbcDatagram},
    },
    plugin::Plugin,
    primitives::{IbcSpec, QueryHeight},
    rpc::{PluginServer, RpcError, RpcErrorExt, RpcResult, types::PluginInfo},
    types::{ProofType, RawClientId},
    vm::{Op, call, defer, defer_relative, noop, pass::PassResult, promise, seq},
};

use crate::call::{
    MakeMsgTimeout, MakeMsgTimeoutFromTrustedHeight, ModuleCall, UpdateClientToHeightTimestamp,
    WaitForTimeoutOrReceipt,
};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

pub struct Module {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".event."@type" == "packet_send"
then
    false # interest, but only copy
else
    null
end
"#,
                ibc_union_id = IbcUnion::ID,
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name(&self) -> String {
        PLUGIN_NAME.to_string()
    }

    pub fn new(Config {}: Config) -> Self {
        Self {}
    }
}

#[async_trait]
impl PluginServer<ModuleCall, Never> for Module {
    #[instrument(skip_all, fields())]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        let ready = msgs
            .into_iter()
            .enumerate()
            .map(|(idx, msg)| match msg {
                Op::Data(Data::IbcEvent(ref chain_event)) => match chain_event
                    .decode_event::<IbcUnion>()
                    .ok_or_else(|| {
                        RpcError::fatal_from_message("unexpected data message in queue").with_data(
                            json!({
                                "msg": msg.clone(),
                            }),
                        )
                    })?
                    .map_err(RpcError::fatal("unable to parse ibc datagram"))
                    .with_data(json!({
                        "msg": msg.clone(),
                    }))? {
                    FullEvent::PacketSend(packet_send) => Ok((
                        vec![idx],
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                                event: packet_send,
                                chain_id: chain_event.chain_id.clone(),
                                counterparty_chain_id: chain_event.counterparty_chain_id.clone(),
                            }),
                        )),
                    )),
                    datagram => Err(RpcError::fatal_from_message(format!(
                        "unexpected ibc datagram {}",
                        datagram.name()
                    ))
                    .with_data(json!({
                        "msg": msg,
                    }))),
                },
                _ => Err(
                    RpcError::fatal_from_message("unexpected message in queue").with_data(json!({
                        "msg": msg,
                    })),
                ),
            })
            .collect::<RpcResult<Vec<_>>>()?;

        Ok(PassResult {
            optimize_further: vec![],
            ready,
        })
    }

    #[instrument(skip_all, fields())]
    async fn call(&self, e: &Extensions, msg: ModuleCall) -> RpcResult<Op<VoyagerMessage>> {
        let voyager_client = e.voyager_client()?;

        match msg {
            ModuleCall::WaitForTimeoutOrReceipt(call) => {
                self.wait_for_timeout_or_receipt(voyager_client, call).await
            }
            ModuleCall::MakeMsgTimeout(MakeMsgTimeout {
                event,
                chain_id,
                counterparty_chain_id,
            }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let proof_unreceived = voyager_client
                    .query_ibc_proof(
                        counterparty_chain_id.clone(),
                        QueryHeight::Specific(client_meta.counterparty_height),
                        BatchReceiptsPath::from_packets(&[event.packet().clone()]),
                    )
                    .await?
                    .into_result()?;

                match proof_unreceived.proof_type {
                    ProofType::NonMembership => {
                        Ok(seq([
                            // wait for the counterparty to finalize the timeout timestamp
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(UpdateClientToHeightTimestamp {
                                    chain_id: chain_id.clone(),
                                    counterparty_chain_id: counterparty_chain_id.clone(),
                                    client_id: event.packet.source_channel.connection.client_id,
                                    timestamp: event.packet.timeout_timestamp,
                                }),
                            )),
                            // build the timeout tx once the client is updated
                            call(PluginMessage::new(
                                self.plugin_name(),
                                ModuleCall::from(MakeMsgTimeoutFromTrustedHeight {
                                    event,
                                    chain_id,
                                    counterparty_chain_id,
                                }),
                            )),
                        ]))
                    }
                    ProofType::Membership => {
                        info!(
                            packet_hash = %event.packet().hash(),
                            "packet already received",
                        );

                        Ok(noop())
                    }
                }
            }
            ModuleCall::UpdateClientToHeightTimestamp(UpdateClientToHeightTimestamp {
                chain_id,
                counterparty_chain_id,
                client_id,
                timestamp,
            }) => {
                let latest_finalized_timestamp = voyager_client
                    .query_latest_timestamp(chain_id.clone(), true)
                    .await?;

                if latest_finalized_timestamp < timestamp {
                    Ok(seq([
                        // if the latest finalized timestamp isn't high enough yet, wait a bit and try again
                        defer_relative(60),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(UpdateClientToHeightTimestamp {
                                chain_id,
                                counterparty_chain_id,
                                client_id,
                                timestamp,
                            }),
                        )),
                    ]))
                } else {
                    let latest_finalized_height = voyager_client
                        .query_latest_height(counterparty_chain_id.clone(), true)
                        .await?;

                    let client_info = voyager_client
                        .client_info::<IbcUnion>(chain_id.clone(), client_id)
                        .await?;

                    let client_meta = voyager_client
                        .client_state_meta::<IbcUnion>(
                            chain_id.clone(),
                            QueryHeight::Finalized,
                            client_id,
                        )
                        .await?;

                    // update the counterparty client
                    Ok(promise(
                        [call(FetchUpdateHeaders {
                            client_type: client_info.client_type,
                            chain_id: counterparty_chain_id.clone(),
                            counterparty_chain_id: chain_id.clone(),
                            client_id: RawClientId::new(client_id),
                            update_from: client_meta.counterparty_height,
                            update_to: latest_finalized_height,
                        })],
                        [],
                        AggregateSubmitTxFromOrderedHeaders {
                            ibc_spec_id: IbcUnion::ID,
                            chain_id: chain_id.clone(),
                            client_id: RawClientId::new(client_id),
                        },
                    ))
                }
            }
            ModuleCall::MakeMsgTimeoutFromTrustedHeight(MakeMsgTimeoutFromTrustedHeight {
                event,
                chain_id,
                counterparty_chain_id,
            }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Finalized,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let consensus_state_meta = voyager_client
                    .consensus_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Finalized,
                        event.packet.source_channel.connection.client_id,
                        client_meta.counterparty_height,
                    )
                    .await?;

                if consensus_state_meta.timestamp >= event.packet.timeout_timestamp {
                    let proof_unreceived = voyager_client
                        .query_ibc_proof(
                            counterparty_chain_id,
                            QueryHeight::Specific(client_meta.counterparty_height),
                            BatchReceiptsPath::from_packets(&[event.packet().clone()]),
                        )
                        .await?
                        .into_result()?;

                    match proof_unreceived.proof_type {
                        ProofType::NonMembership => {
                            let client_info = voyager_client
                                .client_info::<IbcUnion>(
                                    chain_id.clone(),
                                    event.packet.source_channel.connection.client_id,
                                )
                                .await?;

                            let encoded_proof_commitment = voyager_client
                                .encode_proof::<IbcUnion>(
                                    client_info.client_type,
                                    client_info.ibc_interface,
                                    proof_unreceived.proof,
                                )
                                .await?;

                            Ok(call(SubmitTx {
                                chain_id,
                                datagrams: vec![IbcDatagram::new::<IbcUnion>(Datagram::from(
                                    MsgPacketTimeout {
                                        packet: event.packet(),
                                        proof: encoded_proof_commitment,
                                        proof_height: client_meta.counterparty_height.height(),
                                    },
                                ))],
                            }))
                        }
                        ProofType::Membership => {
                            info!(
                                packet_hash = %event.packet().hash(),
                                "packet already received",
                            );

                            Ok(noop())
                        }
                    }
                } else {
                    Ok(seq([
                        // if the latest trusted timestamp isn't high enough yet, wait a bit and try again
                        defer_relative(10),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(UpdateClientToHeightTimestamp {
                                chain_id: chain_id.clone(),
                                counterparty_chain_id: counterparty_chain_id.clone(),
                                client_id: event.packet.source_channel.connection.client_id,
                                timestamp: event.packet.timeout_timestamp,
                            }),
                        )),
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeMsgTimeoutFromTrustedHeight {
                                event,
                                chain_id,
                                counterparty_chain_id,
                            }),
                        )),
                    ]))
                }
            }
        }
    }

    #[instrument(skip_all, fields())]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}

impl Module {
    #[instrument(
        skip_all,
        fields(
            %chain_id,
            %counterparty_chain_id,
            packet_hash = %event.packet().hash()
        )
    )]
    async fn wait_for_timeout_or_receipt(
        &self,
        voyager_client: &VoyagerClient,
        WaitForTimeoutOrReceipt {
            event,
            chain_id,
            counterparty_chain_id,
        }: WaitForTimeoutOrReceipt,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let receipt = voyager_client
            .maybe_query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Latest,
                BatchReceiptsPath::from_packets(&[event.packet()]),
            )
            .await?;

        info!(
            height = receipt.height.height(),
            "counterparty latest height"
        );

        match receipt.state {
            Some(receipt) => {
                info!(%receipt, "packet received");
                Ok(noop())
            }
            None => {
                debug!("packet not received yet");

                if event.packet.timeout_timestamp.is_zero() {
                    Err(RpcError::fatal_from_message(
                        "packet has no timeout timestamp - should be impossible",
                    ))
                } else {
                    let counterparty_timestamp = voyager_client
                        .query_latest_timestamp(counterparty_chain_id.clone(), false)
                        .await?;

                    if event.packet.timeout_timestamp <= counterparty_timestamp {
                        info!(
                            "packet timed out (timestamp): {} <= {}",
                            event.packet.timeout_timestamp, counterparty_timestamp
                        );
                    }

                    Ok(seq([
                        // wait until the timestamp is hit
                        defer(event.packet.timeout_timestamp.as_secs()),
                        // then attempt to make the timeout message
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::from(MakeMsgTimeout {
                                event,
                                chain_id,
                                counterparty_chain_id,
                            }),
                        )),
                    ]))
                }
            }
        }
    }
}
