use std::collections::VecDeque;

use enumorph::Enumorph;
use itertools::Itertools;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use tracing::warn;
use unionlabs::{
    ibc::core::client::{height::Height, msg_update_client::MsgUpdateClient},
    id::ClientId,
};
use voyager_message::{
    call::WaitForTrustedHeight,
    core::{ChainId, ClientStateMeta, QueryHeight},
    data::{Data, IbcDatagram, OrderedClientUpdates, WithChainId},
    ibc_v1::{IbcMessage, IbcV1},
    rpc::{json_rpc_error_to_error_object, VoyagerRpcClient},
    IbcSpec, PluginMessage, RawClientId, VoyagerClient, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, noop, promise, seq, Op};

use crate::{
    call::{
        MakeMsgAcknowledgement, MakeMsgChannelOpenAck, MakeMsgChannelOpenConfirm,
        MakeMsgChannelOpenTry, MakeMsgConnectionOpenAck, MakeMsgConnectionOpenConfirm,
        MakeMsgConnectionOpenTry, MakeMsgRecvPacket, ModuleCall,
    },
    data::{BatchableEvent, Event},
    Module,
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeIbcMessagesFromUpdate(MakeIbcMessagesFromUpdate),
    MakeBatchTransaction(MakeBatchTransaction),
}

/// Given an [`OrderedMsgUpdateClients`], returns [`Op`]s that generate [`IbcMessage`]s with proofs at the highest height of the updates.
#[model]
pub struct MakeIbcMessagesFromUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}

impl MakeIbcMessagesFromUpdate {
    pub async fn call(
        self,
        voyager_client: &VoyagerClient,
        module_server: &Module,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let updates @ OrderedClientUpdates { .. } = datas
            .into_iter()
            .exactly_one()
            .map_err(|found| serde_json::to_string(&found.collect::<Vec<_>>()).unwrap())
            .and_then(|d| {
                d.try_into()
                    .map_err(|found| serde_json::to_string(&found).unwrap())
            })
            .map_err(|found| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!(
                        "OrderedHeaders not present in data queue for \
                        AggregateMsgUpdateClientsFromOrderedHeaders, \
                        found {found}",
                    ),
                    None::<()>,
                )
            })?;

        let client_meta = voyager_client
            .client_meta(
                module_server.chain_id.clone(),
                IbcV1::ID,
                QueryHeight::Latest,
                RawClientId::new(self.client_id.clone()),
            )
            .await
            .map_err(json_rpc_error_to_error_object)?;

        let new_trusted_height = updates
            .updates
            .last()
            .expect("must have at least one update")
            .0
            .height;

        make_msgs(
            module_server,
            self.client_id,
            self.batches,
            Some(updates),
            client_meta,
            new_trusted_height,
        )
    }
}

pub fn make_msgs(
    module_server: &Module,

    client_id: ClientId,
    batches: Vec<Vec<BatchableEvent>>,

    updates: Option<OrderedClientUpdates>,

    client_meta: ClientStateMeta,
    new_trusted_height: Height,
) -> RpcResult<Op<VoyagerMessage>> {
    Ok(conc(batches.into_iter().enumerate().map(|(i, batch)| {
        promise(
            batch.into_iter().map(|batchable_event| {
                assert!(batchable_event.provable_height <= new_trusted_height);

                let origin_chain_id = client_meta.chain_id.clone();
                let target_chain_id = module_server.chain_id.clone();

                // in this context, we are the destination - the counterparty of the source is the destination
                match batchable_event.event {
                    Event::ConnectionOpenInit(connection_open_init_event) => {
                        call(PluginMessage::new(
                            module_server.plugin_name(),
                            ModuleCall::from(MakeMsgConnectionOpenTry {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                connection_open_init_event,
                            }),
                        ))
                    }
                    Event::ConnectionOpenTry(connection_open_try_event) => {
                        call(PluginMessage::new(
                            module_server.plugin_name(),
                            ModuleCall::from(MakeMsgConnectionOpenAck {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                connection_open_try_event,
                            }),
                        ))
                    }
                    Event::ConnectionOpenAck(connection_open_ack_event) => {
                        call(PluginMessage::new(
                            module_server.plugin_name(),
                            ModuleCall::from(MakeMsgConnectionOpenConfirm {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                connection_open_ack_event,
                            }),
                        ))
                    }
                    Event::ChannelOpenInit(channel_open_init_event) => call(PluginMessage::new(
                        module_server.plugin_name(),
                        ModuleCall::from(MakeMsgChannelOpenTry {
                            origin_chain_id,
                            origin_chain_proof_height: new_trusted_height,
                            target_chain_id,
                            channel_open_init_event,
                        }),
                    )),
                    Event::ChannelOpenTry(channel_open_try_event) => call(PluginMessage::new(
                        module_server.plugin_name(),
                        ModuleCall::from(MakeMsgChannelOpenAck {
                            origin_chain_id,
                            origin_chain_proof_height: new_trusted_height,
                            target_chain_id,
                            channel_open_try_event,
                        }),
                    )),
                    Event::ChannelOpenAck(channel_open_ack_event) => call(PluginMessage::new(
                        module_server.plugin_name(),
                        ModuleCall::from(MakeMsgChannelOpenConfirm {
                            origin_chain_id,
                            origin_chain_proof_height: new_trusted_height,
                            target_chain_id,
                            channel_open_ack_event,
                        }),
                    )),
                    Event::SendPacket(send_packet_event) => call(PluginMessage::new(
                        module_server.plugin_name(),
                        ModuleCall::from(MakeMsgRecvPacket {
                            origin_chain_id,
                            origin_chain_proof_height: new_trusted_height,
                            target_chain_id,
                            send_packet_event,
                        }),
                    )),
                    Event::WriteAcknowledgement(write_acknowledgement_event) => {
                        call(PluginMessage::new(
                            module_server.plugin_name(),
                            ModuleCall::from(MakeMsgAcknowledgement {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                write_acknowledgement_event,
                            }),
                        ))
                    }
                }
            }),
            [],
            PluginMessage::new(
                module_server.plugin_name(),
                ModuleCallback::from(MakeBatchTransaction {
                    client_id: client_id.clone(),
                    // if updates are provided and this is the first batch using this update height, provide the updates along with the messages
                    updates: (i == 0).then(|| updates.clone()).flatten(),
                }),
            ),
        )
    })))
}

#[model]
pub struct MakeBatchTransaction {
    // NOTE: We could technically fetch this from the information in the callback data messages, but this is just so much easier
    pub client_id: ClientId,
    /// Updates to send before the messages in this message's callback data. If this is `None`, then that means the updates have been included in a previous batch, and this will instead be enqueued with a WaitForTrustedHeight in front of it.
    pub updates: Option<OrderedClientUpdates>,
}

impl MakeBatchTransaction {
    pub fn call(self, chain_id: ChainId, datas: VecDeque<Data>) -> Op<VoyagerMessage> {
        if datas.is_empty() {
            warn!("no IBC messages in queue! this likely means that all of the IBC messages that were queued to be sent were already sent to the destination chain");
        }

        let mut msgs = datas
            .into_iter()
            .map(|d| {
                IbcDatagram::try_from(d)
                    .unwrap()
                    .decode_datagram::<IbcV1>()
                    .unwrap()
                    .unwrap()
            })
            .peekable();

        // TODO: We may need to sort packet messages when we support ordered channels
        // msgs.sort_unstable_by(|a, b| match (a, b) {
        //     (IbcMessage::RecvPacket(_), IbcMessage::RecvPacket(_)) => todo!(),
        //     (IbcMessage::RecvPacket(_), IbcMessage::AcknowledgePacket(_)) => todo!(),
        //     (IbcMessage::RecvPacket(_), IbcMessage::TimeoutPacket(_)) => todo!(),
        //     (IbcMessage::AcknowledgePacket(_), IbcMessage::RecvPacket(_)) => todo!(),
        //     (IbcMessage::AcknowledgePacket(_), IbcMessage::AcknowledgePacket(_)) => todo!(),
        //     (IbcMessage::AcknowledgePacket(_), IbcMessage::TimeoutPacket(_)) => todo!(),
        //     (IbcMessage::TimeoutPacket(_), IbcMessage::RecvPacket(_)) => todo!(),
        //     (IbcMessage::TimeoutPacket(_), IbcMessage::AcknowledgePacket(_)) => todo!(),
        //     (IbcMessage::TimeoutPacket(_), IbcMessage::TimeoutPacket(_)) => todo!(),
        // });

        match self.updates {
            Some(updates) => data(WithChainId {
                chain_id,
                message: updates
                    .updates
                    .into_iter()
                    .map(|(_, msg)| {
                        assert_eq!(msg.ibc_version_id, IbcV1::ID);

                        IbcMessage::from(MsgUpdateClient {
                            client_id: msg.client_id.decode_spec::<IbcV1>().unwrap(),
                            client_message: msg.client_message,
                        })
                    })
                    .chain(msgs)
                    .map(|msg| IbcDatagram::new::<IbcV1>(msg))
                    .collect::<Vec<_>>(),
            }),
            None => {
                if msgs.len() == 0 {
                    noop()
                } else {
                    // TODO: We can probably relax this in the future if we want to reuse this module to work with all IBC messages
                    // NOTE: We assume that all of the IBC messages were generated against the same consensus height
                    let required_consensus_height = msgs
                        .peek()
                        .expect("msgs is non-empty; qed;")
                        .proof_height()
                        .expect("all batchable messages have a proof height");

                    seq([
                        call(WaitForTrustedHeight {
                            chain_id: chain_id.clone(),
                            client_id: RawClientId::new(self.client_id.clone()),
                            ibc_version_id: IbcV1::ID,
                            height: required_consensus_height,
                        }),
                        data(WithChainId {
                            chain_id,
                            message: msgs.map(IbcDatagram::new::<IbcV1>).collect::<Vec<_>>(),
                        }),
                    ])
                }
            }
        }
    }
}
