use serde::{Deserialize, Serialize};

use crate::{ibc::google::protobuf::any::Any, CosmosAccountId, IntoProto, MsgIntoProto, TypeUrl};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgUpdateClient<Header> {
    /// client unique identifier
    pub client_id: String,
    /// client message to update the light client
    pub client_message: Header,
}

impl TypeUrl for protos::ibc::core::client::v1::MsgUpdateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgUpdateClient";
}

impl<Header> MsgIntoProto for MsgUpdateClient<Header>
where
    Header: IntoProto,
{
    type Proto = protos::ibc::core::client::v1::MsgUpdateClient;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            client_id: self.client_id,
            client_message: Some(Any(self.client_message).into_proto()),
            signer: signer.to_string(),
        }
    }
}
