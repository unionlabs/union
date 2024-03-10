use macros::model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::ibc::core::client::v1::MsgUpdateClient)))]
pub struct MsgUpdateClient<ClientId, Header> {
    pub client_id: ClientId,
    pub client_message: Header,
}
