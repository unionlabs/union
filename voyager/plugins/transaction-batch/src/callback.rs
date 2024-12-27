use std::collections::VecDeque;

use enumorph::Enumorph;
use ibc_classic_spec::IbcClassic;
use ibc_union_spec::IbcUnion;
use itertools::Itertools;
use jsonrpsee::{core::RpcResult, types::ErrorObject};
use macros::model;
use tracing::{debug, instrument, warn};
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::WaitForTrustedHeight,
    core::{ChainId, ClientStateMeta, QueryHeight},
    data::{Data, IbcDatagram, OrderedClientUpdates, WithChainId},
    PluginMessage, RawClientId, VoyagerClient, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::{call, conc, data, noop, promise, seq, Op};

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
            .client_meta::<V>(
                module_server.chain_id.clone(),
                QueryHeight::Latest,
                self.client_id.clone(),
            )
            .await?;

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

#[instrument(
    skip_all,
    fields(
        chain_id = %module_server.chain_id,
        %client_id,
        has_updates = updates.is_some(),
        client_meta.height = %client_meta.counterparty_height,
        client_meta.chain_id = %client_meta.chain_id,
        %new_trusted_height
    )
)]
pub fn make_msgs<V: IbcSpecExt>(
    module_server: &Module,

    client_id: V::ClientId,
    batches: Vec<Vec<BatchableEvent<V>>>,

    updates: Option<OrderedClientUpdates>,

    client_meta: ClientStateMeta,
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

                let origin_chain_id = client_meta.chain_id.clone();
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
    pub updates: Option<OrderedClientUpdates>,
}

impl<V: IbcSpecExt> MakeBatchTransaction<V> {
    #[instrument(skip_all, fields(ibc_spec_id = %V::ID, %chain_id, datas_len = datas.len()))]
    pub fn call(self, chain_id: ChainId, datas: VecDeque<Data>) -> Op<VoyagerMessage> {
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

        match self.updates {
            Some(updates) => data(WithChainId {
                chain_id,
                message: updates
                    .updates
                    .into_iter()
                    .map(|(_, msg)| {
                        assert_eq!(msg.ibc_spec_id, V::ID);

                        V::update_client_datagram(
                            msg.client_id.decode_spec::<V>().unwrap(),
                            msg.client_message,
                        )
                    })
                    .chain(msgs)
                    .map(|e| IbcDatagram::new::<V>(e))
                    .collect::<Vec<_>>(),
            }),
            None => {
                if msgs.len() == 0 {
                    noop()
                } else {
                    // TODO: We can probably relax this in the future if we want to reuse this module to work with all IBC messages
                    // NOTE: We assume that all of the IBC messages were generated against the same consensus height
                    let required_consensus_height =
                        V::proof_height(msgs.peek().expect("msgs is non-empty; qed;"));

                    seq([
                        call(WaitForTrustedHeight {
                            chain_id: chain_id.clone(),
                            client_id: RawClientId::new(self.client_id.clone()),
                            ibc_spec_id: V::ID,
                            height: required_consensus_height,
                        }),
                        data(WithChainId {
                            chain_id,
                            message: msgs.map(IbcDatagram::new::<V>).collect::<Vec<_>>(),
                        }),
                    ])
                }
            }
        }
    }
}
