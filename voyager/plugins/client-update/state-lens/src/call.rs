use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::primitives::ChainId;

use crate::StateLensClientState;

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
    FetchUpdateAfterL1Update(FetchUpdateAfterL1Update),
}

#[model]
pub struct FetchUpdate {
    pub chain_id: ChainId,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
    pub update_from: Height,
    pub update_to: Height,
}

#[model]
pub struct FetchUpdateAfterL1Update {
    pub counterparty_chain_id: ChainId,
    pub state_lens_client_state: StateLensClientState,
    pub client_id: ClientId,
    pub update_from: Height,
    pub update_to: Height,
}
