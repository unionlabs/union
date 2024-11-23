use std::num::NonZeroU32;

use enumorph::Enumorph;
use macros::model;
use unionlabs::{hash::H256, ibc::core::client::height::Height};

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchBlocks(FetchBlocks),
    FetchTransactions(FetchTransactions),
    MakeChainEvent(MakeChainEvent),
}

/// Fetch a block at the specified height, requeuing a seq(wait(H+1), fetch(H+1)).
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
    pub event: RawEvent,
}

#[model]
pub enum RawEvent {
    IbcV1(ibc_events::IbcEvent),
    IbcUnion(ibc_events::union_ibc::IbcEvent),
}

impl RawEvent {
    pub fn name(&self) -> &'static str {
        match self {
            RawEvent::IbcV1(ibc_event) => ibc_event.name(),
            RawEvent::IbcUnion(ibc_event) => ibc_event.name(),
        }
    }
}
