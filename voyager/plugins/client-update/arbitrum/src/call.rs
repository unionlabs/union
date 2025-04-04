use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use macros::model;
use unionlabs::ibc::core::client::height::Height;
use voyager_message::{core::ChainId, RawClientId};

#[model]
#[derive(Enumorph)]
pub enum ModuleCall {
    /// Fetch both the L1 and L2 update for an arbitrum light client.
    FetchUpdate(FetchUpdate),
    /// Fetch the L2 update for an arbitrum client. This assumes that the L1 client is updated to the specified `l1_height`.
    FetchL2Update(FetchL2Update),
}

#[model]
pub struct FetchUpdate {
    pub from_height: Height,
    pub to_height: Height,
    pub counterparty_chain_id: ChainId,
    pub client_id: RawClientId,
}

#[model]
pub struct FetchL2Update {
    pub update_from: Height,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
}
