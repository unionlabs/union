use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{CanonicalChainId, ClientId, ClientType},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub counterparty_chain_id: CanonicalChainId,
}
