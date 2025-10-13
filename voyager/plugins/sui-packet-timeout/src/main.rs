use std::collections::VecDeque;

use ibc_union_spec::{
    IbcUnion,
    datagram::{Datagram, MsgCommitTimedOutPacket, MsgPacketTimeout},
    event::{FullEvent, PacketSend},
    path::{BatchPacketsPath, BatchReceiptsPath, TimedOutPacketPath},
};
use jsonrpsee::{
    Extensions,
    core::{RpcResult, async_trait},
    types::ErrorObject,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sui_sdk::{
    SuiClient, SuiClientBuilder,
    rpc_types::{SuiMoveValue, SuiParsedData, SuiTransactionBlockResponseOptions},
    types::{TypeTag, base_types::ObjectID, dynamic_field::DynamicFieldName},
};
use tracing::{debug, info, instrument, warn};
use unionlabs::{self, ErrorReporter, ibc::core::client::height::Height, never::Never};
use voyager_sdk::{
    DefaultCmd, ExtensionsExt, VoyagerClient,
    anyhow::{self, anyhow},
    message::{
        PluginMessage, VoyagerMessage,
        call::{SubmitTx, WaitForTrustedTimestamp},
        data::{Data, IbcDatagram},
    },
    plugin::Plugin,
    primitives::{ChainId, IbcSpec, QueryHeight},
    rpc::{FATAL_JSONRPC_ERROR_CODE, PluginServer, types::PluginInfo},
    types::{ProofType, RawClientId},
    vm::{Op, call, defer, defer_relative, noop, pass::PassResult, seq},
};

use crate::call::{
    MakeMsgTimeout, MakeMsgTimeoutCommitment, ModuleCall, WaitForTimeoutCommitment,
    WaitForTimeoutOrReceipt,
};

pub mod call;

#[tokio::main]
async fn main() {
    Module::run().await
}

pub struct Module {
    pub chain_id: ChainId,

    pub sui_client: SuiClient,

    pub ibc_store: ObjectID,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub ibc_store: ObjectID,
}

impl Plugin for Module {
    type Call = ModuleCall;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> anyhow::Result<Self> {
        Ok(Module::new(config).await?)
    }

    fn info(config: Self::Config) -> PluginInfo {
        PluginInfo {
            name: Module::plugin_name(),
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
                chain_id = config.chain_id
            ),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

impl Module {
    fn plugin_name() -> String {
        PLUGIN_NAME.to_string()
    }

    pub async fn new(
        Config {
            chain_id,
            rpc_url,
            ibc_store,
        }: Config,
    ) -> anyhow::Result<Self> {
        let sui_client = SuiClientBuilder::default().build(&rpc_url).await?;

        Ok(Self {
            chain_id,
            sui_client,
            ibc_store,
        })
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
                            Module::plugin_name(),
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
            ModuleCall::MakeMsgTimeoutCommitment(MakeMsgTimeoutCommitment { event, chain_id }) => {
                let client_meta = voyager_client
                    .client_state_meta::<IbcUnion>(
                        chain_id.clone(),
                        QueryHeight::Latest,
                        event.packet.source_channel.connection.client_id,
                    )
                    .await?;

                let proof_sent = voyager_client
                    .query_ibc_proof(
                        chain_id.clone(),
                        QueryHeight::Specific(client_meta.counterparty_height),
                        BatchPacketsPath::from_packets(&[event.packet()]),
                    )
                    .await?
                    .into_result()?;

                match proof_sent.proof_type {
                    ProofType::NonMembership => {
                        warn!(
                            packet_hash = %event.packet().hash(),
                            "packet is not even sent, aborting the timeout commitment"
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
                                proof_sent.proof,
                            )
                            .await?;

                        Ok(seq([
                            call(SubmitTx {
                                chain_id: chain_id.clone(),
                                datagrams: vec![IbcDatagram::new::<IbcUnion>(Datagram::from(
                                    MsgCommitTimedOutPacket {
                                        packet: event.packet(),
                                        proof: encoded_proof_commitment,
                                        proof_height: client_meta.counterparty_height.height(),
                                    },
                                ))],
                            }),
                            call(PluginMessage::new(
                                Module::plugin_name(),
                                ModuleCall::from(MakeMsgTimeout { event, chain_id }),
                            )),
                        ]))
                    }
                }
            }
            ModuleCall::WaitForTimeoutCommitment(WaitForTimeoutCommitment { event, chain_id }) => {
                match self
                    .wait_for_timeout_commitment(voyager_client, event.clone(), chain_id.clone())
                    .await
                {
                    Ok(op) => Ok(op),
                    Err(err) => {
                        warn!("{err}");

                        Ok(seq([
                            defer_relative(1),
                            call(PluginMessage::new(
                                Self::plugin_name(),
                                ModuleCall::WaitForTimeoutCommitment(WaitForTimeoutCommitment {
                                    event,
                                    chain_id,
                                }),
                            )),
                        ]))
                    }
                }
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
                                    proof: encoded_proof_commitment,
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
                        Self::plugin_name(),
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
                Self::plugin_name(),
                ModuleCall::from(MakeMsgTimeout {
                    event: event.clone(),
                    chain_id: chain_id.clone(),
                }),
            )),
            call(PluginMessage::new(
                Self::plugin_name(),
                ModuleCall::from(WaitForTimeoutCommitment { event, chain_id }),
            )),
        ])
    }

    async fn wait_for_timeout_commitment(
        &self,
        voyager_client: &VoyagerClient,
        event: PacketSend,
        chain_id: ChainId,
    ) -> anyhow::Result<Op<VoyagerMessage>> {
        let timeout_path = TimedOutPacketPath::from_packet(&event.packet());

        let SuiParsedData::MoveObject(object) = self
            .sui_client
            .read_api()
            .get_dynamic_field_object(
                self.ibc_store,
                DynamicFieldName {
                    type_: TypeTag::Vector(Box::new(TypeTag::U8)),
                    value: serde_json::to_value(timeout_path).expect("serde will work"),
                },
            )
            .await
            .map_err(|_| {
                anyhow!("could not get the dynamic field object, this might be an RPC issue")
            })?
            .data
            .ok_or(anyhow!("data does not exist"))?
            .content
            .ok_or(anyhow!("content does not exist"))?
        else {
            return Err(anyhow!(
                "data type is not `MoveObject`, this might be an RPC issue"
            ));
        };

        let SuiMoveValue::Vector(v) = object
            .fields
            .field_value("value")
            .expect("table has a value")
        else {
            return Err(anyhow!(
                "Returned data is not a `vector<u8>`. Either the data is not committed, or we are having an RPC issue."
            ));
        };

        let digest: Vec<u8> = v
            .into_iter()
            .map(|n| {
                let SuiMoveValue::Number(n) = n else {
                    // if there is already a commitment, we are 100% sure that it will be a `vector<u8>`
                    panic!("digest commitment is `vector<u8>`");
                };

                n as u8
            })
            .collect();

        let height = self
            .sui_client
            .read_api()
            .get_transaction_with_options(
                digest.try_into().expect("ibc saves tx digest"),
                SuiTransactionBlockResponseOptions::new(),
            )
            .await
            .map_err(|_| {
                anyhow!("The tx exists. But we might be having an RPC issue, so will retry.")
            })?
            .checkpoint
            .ok_or(
                anyhow!("The tx exists and it has checkpoint in it. But we might be having an RPC issue, so will retry.")
            )?;

        let proof_timeout = voyager_client
            .query_ibc_proof(
                self.chain_id.clone(),
                QueryHeight::Specific(Height::new(height)),
                TimedOutPacketPath::from_packet(&event.packet()),
            )
            .await?
            .into_result()?;

        match proof_timeout.proof_type {
            ProofType::NonMembership => Ok(seq([
                defer_relative(1),
                call(PluginMessage::new(
                    Self::plugin_name(),
                    ModuleCall::WaitForTimeoutOrReceipt(WaitForTimeoutOrReceipt {
                        event,
                        chain_id,
                    }),
                )),
            ])),
            ProofType::Membership => Ok(noop()),
        }
    }
}
