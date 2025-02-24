use std::collections::VecDeque;

use enumorph::Enumorph;
use futures::{stream::FuturesOrdered, TryFutureExt, TryStreamExt};
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use itertools::Itertools;
use jsonrpsee::core::RpcResult;
use macros::model;
use tracing::{debug, instrument, warn};
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::{SubmitTx, WaitForClientUpdate},
    core::{ChainId, ClientStateMeta, QueryHeight},
    data::{Data, IbcDatagram, OrderedHeaders},
    PluginMessage, RawClientId, VoyagerClient, VoyagerMessage,
};
use voyager_vm::{call, conc, noop, promise, seq, Op};

use crate::{
    call::{MakeMsg, ModuleCall},
    data::BatchableEvent,
    IbcSpecExt, Module,
};

#[model]
#[derive(Enumorph)]
pub enum ModuleCallback {
    MakeIbcMessagesFromUpdateV1(MakeIbcMessagesFromUpdate<IbcClassic>),
    MakeIbcMessagesFromUpdateUnion(MakeIbcMessagesFromUpdate<IbcUnion>),
    MakeBatchTransactionV1(MakeBatchTransaction<IbcClassic>),
    MakeBatchTransactionUnion(MakeBatchTransaction<IbcUnion>),
}

/// Given an [`OrderedMsgUpdateClients`], returns [`Op`]s that generate [`IbcMessage`]s with proofs at the highest height of the updates.
#[model]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct MakeIbcMessagesFromUpdate<V: IbcSpecExt> {
    pub client_id: V::ClientId,
    pub batches: Vec<Vec<BatchableEvent<V>>>,
}

impl<V: IbcSpecExt> MakeIbcMessagesFromUpdate<V>
where
    ModuleCall: From<MakeMsg<V>>,
    ModuleCallback: From<MakeBatchTransaction<V>>,
{
    pub async fn call(
        self,
        voyager_client: &VoyagerClient,
        module_server: &Module,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        let updates: Option<OrderedHeaders> = datas
            .into_iter()
            .exactly_one()
            .map_err(|found| serde_json::to_string(&found.collect::<Vec<_>>()).unwrap())
            .and_then(|d| {
                d.try_into()
                    .map_err(|found| serde_json::to_string(&found).unwrap())
            })
            .ok();

        let client_state_meta = voyager_client
            .client_state_meta::<V>(
                module_server.chain_id.clone(),
                QueryHeight::Latest,
                self.client_id.clone(),
            )
            .await?;

        let new_trusted_height = updates
            .as_ref()
            .map(|updates| {
                updates
                    .headers
                    .last()
                    .expect("must have at least one update")
                    .0
                    .height
            })
            .unwrap_or(client_state_meta.counterparty_height);

        make_msgs(
            module_server,
            self.client_id,
            self.batches,
            updates,
            client_state_meta,
            new_trusted_height,
        )
    }
}

#[instrument(
    skip_all,
    fields(
        chain_id = %module_server.chain_id,
        %client_id,
        has_updates = updates.is_some(),
        client_state_meta.counterparty_height = %client_state_meta.counterparty_height,
        client_state_meta.counterparty_chain_id = %client_state_meta.counterparty_chain_id,
        %new_trusted_height
    )
)]
pub fn make_msgs<V: IbcSpecExt>(
    module_server: &Module,

    client_id: V::ClientId,
    batches: Vec<Vec<BatchableEvent<V>>>,

    updates: Option<OrderedHeaders>,

    client_state_meta: ClientStateMeta,
    new_trusted_height: Height,
) -> RpcResult<Op<VoyagerMessage>>
where
    ModuleCall: From<MakeMsg<V>>,
    ModuleCallback: From<MakeBatchTransaction<V>>,
{
    Ok(conc(batches.into_iter().enumerate().map(|(i, batch)| {
        promise(
            batch.into_iter().map(|batchable_event| {
                assert!(
                    batchable_event.provable_height <= new_trusted_height,
                    "{} <= {}",
                    batchable_event.provable_height,
                    new_trusted_height
                );

                let origin_chain_id = client_state_meta.counterparty_chain_id.clone();
                let target_chain_id = module_server.chain_id.clone();

                debug!(
                    %origin_chain_id,
                    %target_chain_id,
                    event = V::event_name(&batchable_event.event),
                    provable_height = %batchable_event.provable_height,
                    first_seen_at = batchable_event.first_seen_at,
                    "batching event"
                );

                call(PluginMessage::new(
                    module_server.plugin_name(),
                    ModuleCall::from(MakeMsg::<V> {
                        origin_chain_id,
                        origin_chain_proof_height: new_trusted_height,
                        target_chain_id,
                        event: batchable_event.event,
                    }),
                ))
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
pub struct MakeBatchTransaction<V: IbcSpecExt> {
    // NOTE: We could technically fetch this from the information in the callback data messages, but this is just so much easier
    pub client_id: V::ClientId,
    /// Updates to send before the messages in this message's callback data. If this is `None`, then that means the updates have been included in a previous batch, and this will instead be enqueued with a WaitForTrustedHeight in front of it.
    pub updates: Option<OrderedHeaders>,
}

impl<V: IbcSpecExt> MakeBatchTransaction<V> {
    #[instrument(skip_all, fields(ibc_spec_id = %V::ID, %chain_id, datas_len = datas.len()))]
    pub async fn call(
        self,
        voyager_client: &VoyagerClient,
        chain_id: ChainId,
        datas: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        if datas.is_empty() {
            warn!("no IBC messages in queue! this likely means that all of the IBC messages that were queued to be sent were already sent to the destination chain");
        }

        let mut msgs = datas
            .into_iter()
            .map(|d| {
                IbcDatagram::try_from(d)
                    .unwrap()
                    .decode_datagram::<V>()
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

        let client_info = voyager_client
            .client_info::<V>(chain_id.clone(), self.client_id.clone())
            .await?;

        match self.updates {
            Some(updates) => Ok(call(SubmitTx {
                chain_id: chain_id.clone(),
                datagrams: updates
                    .headers
                    .into_iter()
                    .map(|(_, header)| {
                        voyager_client
                            .encode_header::<V>(
                                client_info.client_type.clone(),
                                client_info.ibc_interface.clone(),
                                header,
                            )
                            .map_ok(|encoded_header| {
                                V::update_client_datagram(self.client_id.clone(), encoded_header)
                            })
                    })
                    .collect::<FuturesOrdered<_>>()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .chain(msgs)
                    .map(|e| IbcDatagram::new::<V>(e))
                    .collect::<Vec<_>>(),
            })),
            None => {
                if msgs.len() == 0 {
                    Ok(noop())
                } else {
                    // TODO: We can probably relax this in the future if we want to reuse this module to work with all IBC messages
                    // NOTE: We assume that all of the IBC messages were generated against the same consensus height
                    let required_consensus_height =
                        V::proof_height(msgs.peek().expect("msgs is non-empty; qed;"));

                    Ok(seq([
                        call(WaitForClientUpdate {
                            chain_id: chain_id.clone(),
                            client_id: RawClientId::new(self.client_id.clone()),
                            ibc_spec_id: V::ID,
                            height: required_consensus_height,
                        }),
                        call(SubmitTx {
                            chain_id,
                            datagrams: msgs.map(IbcDatagram::new::<V>).collect::<Vec<_>>(),
                        }),
                    ]))
                }
            }
        }
    }
}
