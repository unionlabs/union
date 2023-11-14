use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{client::height::Height, connection::version::Version},
    id::ConnectionId,
    TypeUrl,
};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgConnectionOpenAck<ClientState> {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub version: Version,
    pub client_state: ClientState,
    pub proof_height: Height,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_try: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_client: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof_consensus: Vec<u8>,
    // TODO: Make this type generic
    pub consensus_height: Height,
}

impl<ClientState: Debug> Debug for MsgConnectionOpenAck<ClientState> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MsgConnectionOpenAck")
            .field("connection_id", &self.connection_id)
            .field("client_state", &self.client_state)
            .field(
                "counterparty_connection_id",
                &self.counterparty_connection_id,
            )
            .field("version", &self.version)
            .field("proof_height", &self.proof_height.to_string())
            .field("proof_try", &serde_utils::to_hex(&self.proof_try))
            .field("proof_client", &serde_utils::to_hex(&self.proof_client))
            .field(
                "proof_consensus",
                &serde_utils::to_hex(&self.proof_consensus),
            )
            .field("consensus_height", &self.consensus_height.to_string())
            .finish()
    }
}
impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenAck {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenAck";
}
