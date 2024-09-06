use std::num::NonZeroU32;

use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::{events::IbcEvent, hash::H256, ibc::core::client::height::Height};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchBlocks(FetchBlocks),
    FetchTransactions(FetchTransactions),
    MakeChainEvent(MakeChainEvent),
}

#[queue_msg]
pub struct FetchBlocks {
    pub from_height: Height,
    pub to_height: Height,
}

#[queue_msg]
pub struct FetchTransactions {
    pub height: Height,
    pub page: NonZeroU32,
}

#[queue_msg]
pub struct MakeChainEvent {
    pub height: Height,
    pub tx_hash: H256,
    pub event: IbcEvent,
}
