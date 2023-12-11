use serde::{Deserialize, Serialize};

use crate::TypeUrl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgCreateClient<ClientState, ConsensusState> {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
}

impl TypeUrl for protos::ibc::core::client::v1::MsgCreateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgCreateClient";
}
