use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{CanonicalChainId, ClientId},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateLensClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub l1_client_id: ClientId,
    pub l2_client_id: ClientId,
    pub l2_chain_id: CanonicalChainId,
}
