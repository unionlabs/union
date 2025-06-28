use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{BlockHeight, ClientId},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateClientEvent {
    #[serde(flatten)]
    pub header: Header,
    pub client_id: ClientId,
    pub counterparty_height: BlockHeight,
}
