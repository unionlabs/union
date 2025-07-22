use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use parlia_light_client_types::Header;
use serde::{Deserialize, Serialize};
use unionlabs::ibc::core::client::height::Height;
use voyager_sdk::primitives::ChainId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Enumorph)]
pub enum ModuleCall {
    FetchUpdate(FetchUpdate),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchUpdate {
    pub from_height: Height,
    pub to_height: Height,
    pub counterparty_chain_id: ChainId,
    pub client_id: ClientId,
    pub already_fetched_updates: Vec<Header>,
}
