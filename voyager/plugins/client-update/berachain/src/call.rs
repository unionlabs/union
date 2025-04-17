use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::primitives::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[model]
pub struct FetchUpdate {
    pub counterparty_chain_id: ChainId,
    pub update_from: Height,
    pub update_to: Height,
    pub client_id: ClientId,
}
