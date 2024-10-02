use std::num::NonZeroU32;

use enumorph::Enumorph;
use macros::model;
use unionlabs::{events::IbcEvent, hash::H256, ibc::core::client::height::Height};

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchBlocks(FetchBlocks),
    FetchTransactions(FetchTransactions),
    MakeChainEvent(MakeChainEvent),
}

/// Fetch a block at the specified height, requeueing a seq(wait(H+1), fetch(H+1)).
#[model]
pub struct FetchBlocks {
    pub height: Height,
}

#[model]
pub struct FetchTransactions {
    pub height: Height,
    pub page: NonZeroU32,
}

#[model]
pub struct MakeChainEvent {
    pub height: Height,
    pub tx_hash: H256,
    pub event: IbcEvent,
}
