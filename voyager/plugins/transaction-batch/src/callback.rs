use std::collections::VecDeque;

use enumorph::Enumorph;
use frunk::hlist_pat;
use jsonrpsee::core::RpcResult;
use queue_msg::{
    aggregation::{HListTryFromIterator, SubsetOf},
    call, conc, data, promise, queue_msg, seq, Op,
};
use unionlabs::{id::ClientId, QueryHeight};
use voyager_message::{
    call::{
        MakeMsgAcknowledgement, MakeMsgChannelOpenAck, MakeMsgChannelOpenConfirm,
        MakeMsgChannelOpenTry, MakeMsgConnectionOpenAck, MakeMsgConnectionOpenConfirm,
        MakeMsgConnectionOpenTry, MakeMsgRecvPacket, WaitForTrustedHeight,
    },
    callback::Callback,
    data::{Data, IbcMessage, OrderedMsgUpdateClients, WithChainId},
    rpc::{json_rpc_error_to_rpc_error, VoyagerRpcClient},
    ChainId, VoyagerMessage,
};

use crate::{
    call::ModuleCall,
    data::{BatchableEvent, Event, ModuleData},
    Module,
};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeIbcMessagesFromUpdate(MakeIbcMessagesFromUpdate),
    MakeBatchTransaction(MakeBatchTransaction),
}

/// Given an [`OrderedMsgUpdateClients`], returns [`Op`]s that generate [`IbcMessage`]s with proofs at the highest height of the updates.
#[queue_msg]
pub struct MakeIbcMessagesFromUpdate {
    pub client_id: ClientId,
    pub batches: Vec<Vec<BatchableEvent>>,
}

impl MakeIbcMessagesFromUpdate {
    pub async fn call(
        self,
        module: &Module,
        datas: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        let Ok(
            hlist_pat![
                updates @ OrderedMsgUpdateClients { .. },
            ],
        ) = HListTryFromIterator::try_from_iter(datas)
        else {
            panic!("bad data")
        };

        let client_meta = module
            .client
            .client_meta(
                module.chain_id.clone(),
                QueryHeight::Latest,
                self.client_id.clone(),
            )
            .await
            .map_err(json_rpc_error_to_rpc_error)?;

        let new_trusted_height = updates
            .updates
            .last()
            .expect("must have at least one update")
            .0
            .height;

        Ok(conc(self.batches.into_iter().enumerate().map(
            |(i, batch)| {
                promise(
                    batch.into_iter().map(|batchable_event| {
                        assert!(batchable_event.provable_height <= new_trusted_height);

                        let origin_chain_id = client_meta.chain_id.clone();
                        let target_chain_id = module.chain_id.clone();

                        // in this context, we are the destination - the counterparty of the source is the destination
                        match batchable_event.event {
                            Event::ConnectionOpenInit(connection_open_init_event) => {
                                call(MakeMsgConnectionOpenTry {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_init_event,
                                })
                            }
                            Event::ConnectionOpenTry(connection_open_try_event) => {
                                call(MakeMsgConnectionOpenAck {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_try_event,
                                })
                            }
                            Event::ConnectionOpenAck(connection_open_ack_event) => {
                                call(MakeMsgConnectionOpenConfirm {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    connection_open_ack_event,
                                })
                            }
                            Event::ChannelOpenInit(channel_open_init_event) => {
                                call(MakeMsgChannelOpenTry {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_init_event,
                                })
                            }
                            Event::ChannelOpenTry(channel_open_try_event) => {
                                call(MakeMsgChannelOpenAck {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_try_event,
                                })
                            }
                            Event::ChannelOpenAck(channel_open_ack_event) => {
                                call(MakeMsgChannelOpenConfirm {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    channel_open_ack_event,
                                })
                            }
                            Event::SendPacket(send_packet_event) => call(MakeMsgRecvPacket {
                                origin_chain_id,
                                origin_chain_proof_height: new_trusted_height,
                                target_chain_id,
                                send_packet_event,
                            }),
                            Event::WriteAcknowledgement(write_acknowledgement_event) => {
                                call(MakeMsgAcknowledgement {
                                    origin_chain_id,
                                    origin_chain_proof_height: new_trusted_height,
                                    target_chain_id,
                                    write_acknowledgement_event,
                                })
                            }
                        }
                    }),
                    [],
                    Callback::plugin(
                        module.plugin_name(),
                        MakeBatchTransaction {
                            client_id: self.client_id.clone(),
                            updates: (i == 0).then(|| updates.clone()),
                        },
                    ),
                )
            },
        )))
    }
}

#[queue_msg]
pub struct MakeBatchTransaction {
    // NOTE: We could technically fetch this from the information in the callback data messages, but this is just so much easier
    pub client_id: ClientId,
    /// Updates to send before the messages in this message's callback data. If this is `None`, then that means the updates have been included in a previous batch, and this will instead be enqueued with a WaitForTrustedHeight in front of it.
    pub updates: Option<OrderedMsgUpdateClients>,
}

impl MakeBatchTransaction {
    pub fn call(
        self,
        chain_id: ChainId<'static>,
        datas: VecDeque<Data<ModuleData>>,
    ) -> Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>> {
        let msgs = datas
            .into_iter()
            .map(|d| IbcMessage::try_from_super(d).unwrap());

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
                    .map(|(_, msg)| IbcMessage::from(msg))
                    .chain(msgs)
                    .collect::<Vec<_>>(),
            }),
            None => {
                let msgs = msgs.collect::<Vec<_>>();

                // TODO: We can probably relax this in the future if we want to reuse this module to work with all IBC messages
                // NOTE: We assume that all of the IBC messages were generated against the same consensus height
                let required_consensus_height = msgs[0]
                    .proof_height()
                    .expect("all batchable messages have a proof height");

                seq([
                    call(WaitForTrustedHeight {
                        chain_id: chain_id.clone(),
                        client_id: self.client_id,
                        height: required_consensus_height,
                    }),
                    data(WithChainId {
                        chain_id,
                        message: msgs,
                    }),
                ])
            }
        }
    }
}

// #[derive(PartialEq, Eq)]
// pub struct OrderedIbcMessage(IbcMessage);

// impl PartialOrd for OrderedIbcMessage {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for OrderedIbcMessage {
//     fn cmp(&self, other: &Self) -> Ordering {
//         use IbcMessage::*;
//         use Ordering::*;

//         match (self.0, other.0) {
//             (ConnectionOpenTry(lhs), ConnectionOpenTry(rhs)) => lhs,
//             (ConnectionOpenTry(_), _) => Ordering::Less,
//             (ConnectionOpenAck(_), ConnectionOpenAck(_)) => Ordering::Equal,
//             (ConnectionOpenAck(_), ConnectionOpenConfirm(_)) => todo!(),
//             (ConnectionOpenAck(_), ChannelOpenTry(_)) => todo!(),
//             (ConnectionOpenAck(_), ChannelOpenAck(_)) => todo!(),
//             (ConnectionOpenAck(_), ChannelOpenConfirm(_)) => todo!(),
//             (ConnectionOpenAck(_), RecvPacket(_)) => todo!(),
//             (ConnectionOpenAck(_), AcknowledgePacket(_)) => todo!(),
//             (ConnectionOpenAck(_), TimeoutPacket(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ConnectionOpenTry(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ConnectionOpenAck(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ConnectionOpenConfirm(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ChannelOpenTry(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ChannelOpenAck(_)) => todo!(),
//             (ConnectionOpenConfirm(_), ChannelOpenConfirm(_)) => todo!(),
//             (ConnectionOpenConfirm(_), RecvPacket(_)) => todo!(),
//             (ConnectionOpenConfirm(_), AcknowledgePacket(_)) => todo!(),
//             (ConnectionOpenConfirm(_), TimeoutPacket(_)) => todo!(),
//             (ChannelOpenTry(_), ConnectionOpenTry(_)) => todo!(),
//             (ChannelOpenTry(_), ConnectionOpenAck(_)) => todo!(),
//             (ChannelOpenTry(_), ConnectionOpenConfirm(_)) => todo!(),
//             (ChannelOpenTry(_), ChannelOpenTry(_)) => todo!(),
//             (ChannelOpenTry(_), ChannelOpenAck(_)) => todo!(),
//             (ChannelOpenTry(_), ChannelOpenConfirm(_)) => todo!(),
//             (ChannelOpenTry(_), RecvPacket(_)) => todo!(),
//             (ChannelOpenTry(_), AcknowledgePacket(_)) => todo!(),
//             (ChannelOpenTry(_), TimeoutPacket(_)) => todo!(),
//             (ChannelOpenAck(_), ConnectionOpenTry(_)) => todo!(),
//             (ChannelOpenAck(_), ConnectionOpenAck(_)) => todo!(),
//             (ChannelOpenAck(_), ConnectionOpenConfirm(_)) => todo!(),
//             (ChannelOpenAck(_), ChannelOpenTry(_)) => todo!(),
//             (ChannelOpenAck(_), ChannelOpenAck(_)) => todo!(),
//             (ChannelOpenAck(_), ChannelOpenConfirm(_)) => todo!(),
//             (ChannelOpenAck(_), RecvPacket(_)) => todo!(),
//             (ChannelOpenAck(_), AcknowledgePacket(_)) => todo!(),
//             (ChannelOpenAck(_), TimeoutPacket(_)) => todo!(),
//             (ChannelOpenConfirm(_), ConnectionOpenTry(_)) => todo!(),
//             (ChannelOpenConfirm(_), ConnectionOpenAck(_)) => todo!(),
//             (ChannelOpenConfirm(_), ConnectionOpenConfirm(_)) => todo!(),
//             (ChannelOpenConfirm(_), ChannelOpenTry(_)) => todo!(),
//             (ChannelOpenConfirm(_), ChannelOpenAck(_)) => todo!(),
//             (ChannelOpenConfirm(_), ChannelOpenConfirm(_)) => todo!(),
//             (ChannelOpenConfirm(_), RecvPacket(_)) => todo!(),
//             (ChannelOpenConfirm(_), AcknowledgePacket(_)) => todo!(),
//             (ChannelOpenConfirm(_), TimeoutPacket(_)) => todo!(),
//             (RecvPacket(_), ConnectionOpenTry(_)) => todo!(),
//             (RecvPacket(_), ConnectionOpenAck(_)) => todo!(),
//             (RecvPacket(_), ConnectionOpenConfirm(_)) => todo!(),
//             (RecvPacket(_), ChannelOpenTry(_)) => todo!(),
//             (RecvPacket(_), ChannelOpenAck(_)) => todo!(),
//             (RecvPacket(_), ChannelOpenConfirm(_)) => todo!(),
//             (RecvPacket(_), RecvPacket(_)) => todo!(),
//             (RecvPacket(_), AcknowledgePacket(_)) => todo!(),
//             (RecvPacket(_), TimeoutPacket(_)) => todo!(),
//             (AcknowledgePacket(_), ConnectionOpenTry(_)) => todo!(),
//             (AcknowledgePacket(_), ConnectionOpenAck(_)) => todo!(),
//             (AcknowledgePacket(_), ConnectionOpenConfirm(_)) => todo!(),
//             (AcknowledgePacket(_), ChannelOpenTry(_)) => todo!(),
//             (AcknowledgePacket(_), ChannelOpenAck(_)) => todo!(),
//             (AcknowledgePacket(_), ChannelOpenConfirm(_)) => todo!(),
//             (AcknowledgePacket(_), RecvPacket(_)) => todo!(),
//             (AcknowledgePacket(_), AcknowledgePacket(_)) => todo!(),
//             (AcknowledgePacket(_), TimeoutPacket(_)) => todo!(),
//             (TimeoutPacket(_), ConnectionOpenTry(_)) => todo!(),
//             (TimeoutPacket(_), ConnectionOpenAck(_)) => todo!(),
//             (TimeoutPacket(_), ConnectionOpenConfirm(_)) => todo!(),
//             (TimeoutPacket(_), ChannelOpenTry(_)) => todo!(),
//             (TimeoutPacket(_), ChannelOpenAck(_)) => todo!(),
//             (TimeoutPacket(_), ChannelOpenConfirm(_)) => todo!(),
//             (TimeoutPacket(_), RecvPacket(_)) => todo!(),
//             (TimeoutPacket(_), AcknowledgePacket(_)) => todo!(),
//             (TimeoutPacket(_), TimeoutPacket(_)) => todo!(),
//         }
//     }
// }
