use std::collections::BTreeSet;

use enumorph::Enumorph;
use macros::model;
use unionlabs::{ibc::core::client::height::Height, primitives::H256};

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchBlocks(FetchBlocks),
    FetchBlock(FetchBlock),
    MakeChainEvent(MakeChainEvent),
}

/// Fetch a block at the specified height, requeuing a seq(wait(H+1), fetch(H+1)).
#[model]
pub struct FetchBlocks {
    pub height: Height,
}

#[model]
pub struct FetchBlock {
    /// If this is Some, then this message is "re-fetching" the events in this block, to ensure that no events were missed during the original fetch of this block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub already_seen_events: Option<BTreeSet<H256>>,
    pub height: Height,
}

#[model]
pub struct MakeChainEvent {
    pub height: Height,
    pub tx_hash: H256,
    pub event: crate::ibc_events::IbcEvent,
}
