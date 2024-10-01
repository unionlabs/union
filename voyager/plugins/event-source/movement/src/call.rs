use enumorph::Enumorph;
use macros::model;
use unionlabs::hash::H256;

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchBlock(FetchBlock),
    FetchBlocks(FetchBlocks),
    MakeEvent(MakeEvent),
}

#[model]
pub struct FetchBlocks {
    pub from_height: u64,
    pub to_height: u64,
}

#[model]
pub struct FetchBlock {
    pub height: u64,
}

#[model]
pub struct MakeEvent {
    pub event: crate::events::IbcEvent,
    pub tx_hash: H256,
    pub height: u64,
}
