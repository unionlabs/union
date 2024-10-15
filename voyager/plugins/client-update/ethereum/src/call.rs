use enumorph::Enumorph;
use voyager_message::macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::core::ChainId;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[model]
pub struct FetchUpdate {
    pub from_height: Height,
    pub to_height: Height,
    pub counterparty_chain_id: ChainId<'static>,
}
