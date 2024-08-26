use std::collections::VecDeque;

use enumorph::Enumorph;
use frunk::{hlist_pat, HList};
use queue_msg::{
    aggregation::{SubsetOf, UseAggregate},
    call, queue_msg,
};
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{
    call::FetchUpdateHeaders,
    data::{Data, DecodedClientStateMeta, IbcMessage, OrderedMsgUpdateClients, WithChainId},
    ChainId, VoyagerMessage,
};

use crate::{
    data::{EventBatch, ModuleData},
    fetch::ModuleFetch,
};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleAggregate {
    MakeUpdateFromLatestHeightToAtLeastTargetHeight(
        MakeUpdateFromLatestHeightToAtLeastTargetHeight,
    ),
    MakeIbcMessagesFromUpdate(MakeIbcMessagesFromUpdate),
    MakeBatchTransaction(MakeBatchTransaction),
}

#[queue_msg]
pub struct MakeUpdateFromLatestHeightToAtLeastTargetHeight {
    pub target_height: Height,
}

impl UseAggregate<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>>
    for MakeUpdateFromLatestHeightToAtLeastTargetHeight
{
    type AggregatedData = HList![DecodedClientStateMeta];

    fn aggregate(
        Self { target_height }: Self,
        hlist_pat![client_meta]: Self::AggregatedData,
    ) -> queue_msg::Op<VoyagerMessage<ModuleData, ModuleFetch, ModuleAggregate>> {
        call(FetchUpdateHeaders {
            chain_id: client_meta.state.chain_id,
            update_from: client_meta.state.height,
            update_to: target_height,
        })
    }
}

/// Given an [`OrderedMsgUpdateClients`], returns [`Op`]s that generate [`IbcMessage`]s with proofs at the highest height of the updates.
#[queue_msg]
pub struct MakeIbcMessagesFromUpdate {
    pub batch: EventBatch,
}

#[queue_msg]
pub struct MakeBatchTransaction {
    pub updates: OrderedMsgUpdateClients,
}

impl MakeBatchTransaction {
    pub fn do_aggregate(
        self,
        chain_id: ChainId<'static>,
        datas: VecDeque<Data<ModuleData>>,
    ) -> Data<ModuleData> {
        let msgs = datas
            .into_iter()
            .map(|d| IbcMessage::try_from_super(d).unwrap())
            .collect::<Vec<_>>();

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

        Data::IdentifiedIbcMessageBatch(WithChainId {
            chain_id,
            message: self
                .updates
                .updates
                .into_iter()
                .map(|(_, msg)| IbcMessage::from(msg))
                .chain(msgs)
                .collect::<Vec<_>>(),
        })
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
