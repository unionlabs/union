use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::connection::{counterparty::Counterparty, version::Version},
    traits::Id,
    EmptyString,
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
#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenInit)))]
pub struct MsgConnectionOpenInit<ClientId: Id, CounterpartyClientId: Id> {
    pub client_id: ClientId,
    pub counterparty: Counterparty<CounterpartyClientId, EmptyString>,
    pub version: Version,
    pub delay_period: u64,
}
