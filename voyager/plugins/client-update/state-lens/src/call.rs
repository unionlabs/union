use enumorph::Enumorph;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::core::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
    FetchUpdateAfterL1Update(FetchUpdateAfterL1Update),
}

#[model]
pub struct FetchUpdate {
    pub counterparty_chain_id: ChainId,
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchUpdateAfterL1Update {
    pub counterparty_chain_id: ChainId,
    pub update_from: Height,
    pub update_to: Height,
}
