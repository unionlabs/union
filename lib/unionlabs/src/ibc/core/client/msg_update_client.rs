use serde::{Deserialize, Serialize};

use crate::TypeUrl;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct MsgUpdateClient<ClientId, Header> {
    pub client_id: ClientId,
    pub client_message: Header,
}

impl TypeUrl for protos::ibc::core::client::v1::MsgUpdateClient {
    const TYPE_URL: &'static str = "/ibc.core.client.v1.MsgUpdateClient";
}
