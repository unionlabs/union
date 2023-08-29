use std::fmt::Debug;

#[cfg(feature = "ethabi")]
use prost::Message;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{client::height::Height, connection::version::Version},
    id::ConnectionId,
    CosmosAccountId, IntoProto, MsgIntoProto, TypeUrl,
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

impl<ClientState> MsgIntoProto for MsgConnectionOpenAck<ClientState>
where
    ClientState: IntoProto<Proto = protos::google::protobuf::Any>,
{
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenAck;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            connection_id: self.connection_id.to_string(),
            counterparty_connection_id: self.counterparty_connection_id.to_string(),
            version: Some(self.version.into()),
            client_state: Some(self.client_state.into_proto()),
            proof_height: Some(self.proof_height.into()),
            proof_try: self.proof_try,
            proof_client: self.proof_client,
            proof_consensus: self.proof_consensus,
            consensus_height: Some(self.consensus_height.into()),
            signer: signer.to_string(),
            host_consensus_state_proof: vec![],
        }
    }
}

#[cfg(feature = "ethabi")]
impl<ClientState> From<MsgConnectionOpenAck<ClientState>>
    for contracts::ibc_handler::MsgConnectionOpenAck
where
    ClientState: IntoProto<Proto = protos::google::protobuf::Any>,
{
    fn from(msg: MsgConnectionOpenAck<ClientState>) -> Self {
        Self {
            connection_id: msg.connection_id.to_string(),
            counterparty_connection_id: msg.counterparty_connection_id.to_string(),
            version: msg.version.into(),
            // client_state_bytes: msg.client_state.value.into(),
            client_state_bytes: msg.client_state.into_proto().encode_to_vec().into(),
            proof_height: msg.proof_height.into(),
            proof_try: msg.proof_try.into(),
            proof_client: msg.proof_client.into(),
            proof_consensus: msg.proof_consensus.into(),
            consensus_height: msg.consensus_height.into(),
        }
    }
}
