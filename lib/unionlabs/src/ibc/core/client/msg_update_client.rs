use serde::{Deserialize, Serialize};

use crate::TypeUrl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgUpdateClient<ClientId, Header> {
    /// client unique identifier
    pub client_id: ClientId,
    /// client message to update the light client
    pub client_message: Header,
}

impl TypeUrl for protos::ibc::core::client::v1::MsgUpdateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgUpdateClient";
}
