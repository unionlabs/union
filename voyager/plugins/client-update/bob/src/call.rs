use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::core::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
    FetchL2Update(FetchL2Update),
}

#[model]
pub struct FetchUpdate {
    pub from_height: Height,
    pub to_height: Height,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
}

#[model]
pub struct FetchL2Update {
    pub update_from: Height,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
}
