use std::collections::VecDeque;

use ibc_union_spec::{
    IbcUnion,
    datagram::{Datagram, MsgPacketTimeout},
    event::{FullEvent, PacketSend},
    path::{BatchReceiptsPath, TimedOutPacketPath},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument, warn};
use unionlabs::{self, ErrorReporter, never::Never};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient, anyhow,
    message::{
        PluginMessage, VoyagerMessage,
        call::{SubmitTx, WaitForTrustedTimestamp},
        data::{Data, IbcDatagram},
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec, QueryHeight},
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, types::PluginInfo},
    types::{ProofType, RawClientId},
    vm::{Op, call, defer, noop, pass::PassResult, seq},
};

use crate::call::{MakeMsgTimeout, ModuleCall, WaitForTimeoutOrReceipt};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

pub struct Module {
    chain_id: ChainId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    chain_id: ChainId,
}

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
            // TODO: Support IBC classic
            interest_filter: format!(
                r#"
if ."@type" == "data"
    and ."@value"."@type" == "ibc_event"
    and ."@value"."@value".ibc_spec_id == "{ibc_union_id}"
    and ."@value"."@value".event."@type" == "packet_send"
    and ."@value"."@value".counterparty_chain_id == "{chain_id}"
then
    false # interest, but only copy
else
    null
end
"#,
                ibc_union_id = IbcUnion::ID,
                chain_id = module.chain_id
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

    pub fn new(Config { chain_id }: Config) -> Self {
        Self { chain_id }
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
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "unexpected data message in queue",
                            Some(json!({
                                "msg": msg.clone(),
                            })),
                        )
                    })?
                    .map_err(|err| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "unable to parse ibc datagram",
                            Some(json!({
                                "err": ErrorReporter(err).to_string(),
                                "msg": msg,
                            })),
                        )
                    })? {
                    FullEvent::PacketSend(packet_send) => Ok((
                        vec![idx],
                        call(PluginMessage::new(
                            self.plugin_name(),
                            ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                                event: packet_send,
                                chain_id: chain_event.chain_id.clone(),
                            }),
                        )),
                    )),
                    datagram => Err(ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unexpected ibc datagram {}", datagram.name()),
                        Some(json!({
                            "msg": msg,
                        })),
                    )),
                },
                _ => Err(ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    "unexpected message in queue",
                    Some(json!({
                        "msg": msg,
                    })),
                )),
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
            ModuleCall::MakeMsgTimeout(MakeMsgTimeout { event, chain_id }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let proof_unreceived = voyager_client
                    .query_ibc_proof(
                        self.chain_id.clone(),
                        QueryHeight::Specific(client_meta.counterparty_height),
                        TimedOutPacketPath::from_packet(&event.packet()),
                    )
                    .await?
                    .into_result()?;

                match proof_unreceived.proof_type {
                    ProofType::NonMembership => {
                        warn!(
                            packet_hash = %event.packet().hash(),
                            "packet timed out, but the time out commitment is not made yet"
                        );

                        Ok(noop())
                    }
                    ProofType::Membership => {
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
                                    proof: encoded_proof_commtment,
                                    proof_height: client_meta.counterparty_height.height(),
                                },
                            ))],
                        }))
                    }
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
            packet_hash = %event.packet().hash()
        )
    )]
    async fn wait_for_timeout_or_receipt(
        &self,
        voyager_client: &VoyagerClient,
        WaitForTimeoutOrReceipt { event, chain_id }: WaitForTimeoutOrReceipt,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let counterparty_latest_height = voyager_client
            .query_latest_height(self.chain_id.clone(), false)
            .await?;

        info!("counterparty latest height: {counterparty_latest_height}");

        let receipt = voyager_client
            .maybe_query_ibc_state(
                self.chain_id.clone(),
                QueryHeight::Specific(counterparty_latest_height),
                BatchReceiptsPath::from_packets(&[event.packet()]),
            )
            .await?;

        match receipt.state {
            Some(receipt) => {
                info!(%receipt, "packet received");
                Ok(noop())
            }
            None => {
                debug!("packet not received yet");

                if !event.packet.timeout_timestamp.is_zero() {
                    let counterparty_timestamp = voyager_client
                        .query_latest_timestamp(self.chain_id.clone(), false)
                        .await?;

                    if event.packet.timeout_timestamp <= counterparty_timestamp {
                        info!(
                            "packet timed out (timestamp): {} <= {}",
                            event.packet.timeout_timestamp, counterparty_timestamp
                        );
                    }

                    Ok(self.mk_wait(chain_id, event))
                } else {
                    Ok(call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                            event,
                            chain_id,
                        }),
                    )))
                }
            }
        }
    }

    fn mk_wait(&self, chain_id: ChainId, event: PacketSend) -> Op<VoyagerMessage> {
        seq([
            // wait until the timestamp is hit
            defer(event.packet.timeout_timestamp.as_secs()),
            // then wait for the counterparty client to be updated to a block >= the timestamp
            call(WaitForTrustedTimestamp {
                chain_id: chain_id.clone(),
                ibc_spec_id: IbcUnion::ID,
                client_id: RawClientId::new(event.packet.source_channel.connection.client_id),
                timestamp: event.packet.timeout_timestamp,
                finalized: false,
            }),
            // then make the timeout message
            call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(MakeMsgTimeout { event, chain_id }),
            )),
        ])
    }
}
