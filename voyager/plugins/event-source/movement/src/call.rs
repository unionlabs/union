use enumorph::Enumorph;
use macros::model;
use unionlabs::hash::H256;

#[model]
#[derive(Enumorph)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleCall {
    FetchTransactions(FetchTransactions),
    FetchBlocks(FetchBlocks),
    MakeEvent(MakeEvent),
}

#[model]
pub struct FetchBlocks {
    pub height: u64,
}

#[model]
pub struct FetchTransactions {
    pub height: u64,
}

#[model]
pub struct MakeEvent {
    pub event: crate::events::IbcEvent,
    pub tx_hash: H256,
    pub height: u64,
}
