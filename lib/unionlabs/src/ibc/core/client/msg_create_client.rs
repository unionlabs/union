use serde::{Deserialize, Serialize};

use crate::{CosmosAccountId, IntoProto, MsgIntoProto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgCreateClient<ClientState, ConsensusState> {
    pub client_state: ClientState,
    pub consensus_state: ConsensusState,
}

impl<ClientState, ConsensusState> MsgIntoProto for MsgCreateClient<ClientState, ConsensusState>
where
    ClientState: IntoProto<Proto = protos::google::protobuf::Any>,
    ConsensusState: IntoProto<Proto = protos::google::protobuf::Any>,
{
    type Proto = protos::ibc::core::client::v1::MsgCreateClient;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            client_state: Some(self.client_state.into_proto()),
            consensus_state: Some(self.consensus_state.into_proto()),
            signer: signer.to_string(),
        }
    }
}

impl TypeUrl for protos::ibc::core::client::v1::MsgCreateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgCreateClient";
}

// #[cfg(feature = "ethabi")]
// impl<ClientState, ConsensusState> From<MsgCreateClient<ClientState, ConsensusState>>
//     for contracts::ibc_handler::MsgCreateClient
// {
//     fn from(msg: MsgCreateClient<ClientState, ConsensusState>) -> Self {
//         Self {
//             client_type: msg.client_type,
//             client_state_bytes: msg.client_state_bytes,
//             consensus_state_bytes: msg.consensus_state_bytes,
//         }
//     }
// }
