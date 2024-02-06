use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::connection::{counterparty::Counterparty, version::Version},
    EmptyString, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    bound(
        serialize = "
            ClientId: Serialize,
            CounterpartyClientId: Serialize,
        ",
        deserialize = "
            ClientId: for<'d> Deserialize<'d>,
            CounterpartyClientId: for<'d> Deserialize<'d>,
        ",
    ),
    deny_unknown_fields
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct MsgConnectionOpenInit<ClientId, CounterpartyClientId> {
    pub client_id: ClientId,
    pub counterparty: Counterparty<CounterpartyClientId, EmptyString>,
    pub version: Version,
    pub delay_period: u64,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenInit {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenInit";
}
