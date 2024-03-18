use std::fmt::Display;

use macros::apply;
use queue_msg::{data, msg_struct, HandleData, QueueError, QueueMsg, QueueMsgTypes};
use serde::{Deserialize, Serialize};
use unionlabs::{events::IbcEvent, hash::H256, ClientType};

use crate::{any_enum, AnyChainIdentified, BlockPollingTypes, ChainExt};

#[apply(any_enum)]
#[any = AnyData]
pub enum Data<C: ChainExt> {
    IbcEvent(ChainEvent<C>),
    LatestHeight(LatestHeight<C>),

    #[serde(untagged)]
    ChainSpecific(ChainSpecificData<C>),
}

// Passthrough since we don't want to handle any top-level data, just bubble it up to the top level.
impl HandleData<BlockPollingTypes> for AnyChainIdentified<AnyData> {
    fn handle(
        self,
        _store: &<BlockPollingTypes as QueueMsgTypes>::Store,
    ) -> Result<QueueMsg<BlockPollingTypes>, QueueError> {
        Ok(data(self))
    }
}

impl<C: ChainExt> Display for Data<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::IbcEvent(event) => write!(f, "IbcEvent({})", event.event.name()),
            Data::LatestHeight(lh) => write!(f, "LatestHeight({})", lh.0),
            Data::ChainSpecific(cs) => write!(f, "{}", cs.0),
        }
    }
}

#[apply(msg_struct)]
pub struct ChainSpecificData<C: ChainExt>(pub C::Data);

#[apply(msg_struct)]
pub struct ChainEvent<C: ChainExt> {
    pub client_type: ClientType,
    pub tx_hash: H256,
    // the 'provable height' of the event
    pub height: C::Height,
    pub event: IbcEvent<C::ClientId, C::ClientType, String>,
}

#[apply(msg_struct)]
pub struct LatestHeight<C: ChainExt>(pub C::Height);
