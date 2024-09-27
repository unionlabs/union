use std::num::NonZeroU32;

use enumorph::Enumorph;
use queue_msg::queue_msg;
use unionlabs::{events::IbcEvent, hash::H256, ibc::core::client::height::Height};

#[queue_msg]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchBlocks(FetchBlock),
    FetchTransactions(FetchTransactions),
    MakeChainEvent(MakeChainEvent),
}

/// Fetch a block at the specified height, requeueing a seq(wait(H+1), fetch(H+1)).
#[queue_msg]
pub struct FetchBlock {
    pub height: Height,
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
