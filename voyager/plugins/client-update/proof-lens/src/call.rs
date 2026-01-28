use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use proof_lens_light_client_types::ClientState;
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::primitives::ChainId;

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
    pub update_to: Height,
}

#[model]
pub struct FetchUpdateAfterL1Update {
    pub counterparty_chain_id: ChainId,
    pub proof_lens_client_state: ClientState,
    pub client_id: ClientId,
    pub update_to: Height,
}
