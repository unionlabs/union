use serde::{Deserialize, Serialize};

use crate::indexer::event::{
    header::Header,
    types::{ClientId, ConnectionId},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectionOpenInitEvent {
    #[serde(flatten)]
    pub header: Header,
    pub connection_id: ConnectionId,
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}
