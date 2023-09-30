use serde::{Deserialize, Serialize};

use crate::{
    ibc::google::protobuf::any::Any, traits, CosmosAccountId, IntoProto, MsgIntoProto, Proto,
    TypeUrl,
};

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

impl<ClientId, Header> MsgIntoProto for MsgUpdateClient<ClientId, Header>
where
    ClientId: traits::Id,
    Header: IntoProto,
    <Header as Proto>::Proto: TypeUrl,
{
    type Proto = protos::ibc::core::client::v1::MsgUpdateClient;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            client_id: self.client_id.to_string(),
            client_message: Some(Any(self.client_message).into_proto()),
            signer: signer.to_string(),
        }
    }
}
