use std::collections::VecDeque;

use ibc_union_spec::{
    datagram::{Datagram, MsgPacketTimeout},
    event::{FullEvent, PacketSend},
    path::BatchReceiptsPath,
    IbcUnion,
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, instrument, warn};
use unionlabs::{self, ibc::core::client::height::Height, traits::Member, ErrorReporter};
use voyager_message::{
    call::{SubmitTx, WaitForTrustedHeight, WaitForTrustedTimestamp},
    data::{Data, IbcDatagram},
    module::{PluginInfo, PluginServer},
    primitives::{ChainId, IbcSpec, QueryHeight},
    rpc::ProofType,
    DefaultCmd, ExtensionsExt, Plugin, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
    FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, noop, pass::PassResult, seq, BoxDynError, Op};

use crate::{
    call::{MakeMsgTimeout, ModuleCall, WaitForTimeoutOrReceipt},
    callback::ModuleCallback,
};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

pub struct Module {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = ModuleCallback;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
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
impl PluginServer<ModuleCall, ModuleCallback> for Module {
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
                                counterparty_chain_id: chain_event.counterparty_chain_id.clone(),
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
        let voyager_client = e.try_get::<VoyagerClient>()?;

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
                        counterparty_chain_id,
                        QueryHeight::Specific(client_meta.counterparty_height),
                        BatchReceiptsPath::from_packets(&[event.packet().clone()]),
                    )
                    .await?;

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
                        warn!(
                            packet_hash = %event.packet().hash(),
                            "packet timed out, but it was already received on the counterparty"
                        );

                        Ok(noop())
                    }
                }
            }
        }
    }

    #[instrument(skip_all, fields())]
    async fn callback(
        &self,
        _: &Extensions,
        cb: ModuleCallback,
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
        let counterparty_latest_height = voyager_client
            .query_latest_height(counterparty_chain_id.clone(), false)
            .await?;

        info!("counterparty latest height: {counterparty_latest_height}");

        let receipt = voyager_client
            .maybe_query_ibc_state(
                counterparty_chain_id.clone(),
                QueryHeight::Specific(counterparty_latest_height),
                BatchReceiptsPath::from_packets(&[event.packet()]),
            )
            .await?;

        match receipt.state {
            Some(receipt) => {
                info!("packet received (receipt: {receipt})");
                Ok(noop())
            }
            None => {
                debug!("packet not received yet");

                if event.packet.timeout_height != 0
                    && event.packet.timeout_height > counterparty_latest_height.height()
                {
                    info!(
                        "packet timed out (height): {} <= {}",
                        event.packet.timeout_height, counterparty_latest_height
                    );

                    Ok(self.mk_wait(chain_id, counterparty_chain_id, event))
                } else if !event.packet.timeout_timestamp.is_zero() {
                    let counterparty_timestamp = voyager_client
                        .query_latest_timestamp(counterparty_chain_id.clone(), false)
                        .await?;

                    if event.packet.timeout_timestamp <= counterparty_timestamp {
                        info!(
                            "packet timed out (timestamp): {} <= {}",
                            event.packet.timeout_timestamp, counterparty_timestamp
                        );
                    }

                    Ok(self.mk_wait(chain_id, counterparty_chain_id, event))
                } else {
                    Ok(call(PluginMessage::new(
                        self.plugin_name(),
                        ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                            event,
                            chain_id,
                            counterparty_chain_id,
                        }),
                    )))
                }
            }
        }
    }

    fn mk_wait(
        &self,
        chain_id: ChainId,
        counterparty_chain_id: ChainId,
        event: PacketSend,
    ) -> Op<VoyagerMessage> {
        seq([
            conc(
                [
                    (event.packet.timeout_height != 0).then_some(call(WaitForTrustedHeight {
                        chain_id: chain_id.clone(),
                        ibc_spec_id: IbcUnion::ID,
                        client_id: RawClientId::new(
                            event.packet.source_channel.connection.client_id,
                        ),
                        height: Height::new(event.packet.timeout_height),
                        finalized: false,
                    })),
                    (event.packet.timeout_timestamp.as_nanos() != 0).then_some(call(
                        WaitForTrustedTimestamp {
                            chain_id: chain_id.clone(),
                            ibc_spec_id: IbcUnion::ID,
                            client_id: RawClientId::new(
                                event.packet.source_channel.connection.client_id,
                            ),
                            timestamp: event.packet.timeout_timestamp,
                            finalized: false,
                        },
                    )),
                ]
                .into_iter()
                .flatten(),
            ),
            call(PluginMessage::new(
                self.plugin_name(),
                ModuleCall::from(MakeMsgTimeout {
                    event,
                    chain_id,
                    counterparty_chain_id,
                }),
            )),
        ])
    }
}
