use enumorph::Enumorph;
use macros::model;
use unionlabs::primitives::H256;

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
    pub height: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,
}

#[model]
pub struct FetchBlock {
    pub height: u64,
}

#[model]
pub struct MakeChainEvent {
    pub height: u64,
    pub tx_hash: H256,
    // pub event: crate::ibc_events::IbcEvent,
}
