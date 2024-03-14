use macros::model;
use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::connection::{counterparty::Counterparty, version::Version},
    traits::Id,
    EmptyString,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenInit)))]
#[serde(bound(
    serialize = "
        ClientId: Serialize,
        CounterpartyClientId: Serialize,
    ",
    deserialize = "
        ClientId: for<'d> Deserialize<'d>,
        CounterpartyClientId: for<'d> Deserialize<'d>,
    ",
))]
pub struct MsgConnectionOpenInit<ClientId: Id, CounterpartyClientId: Id> {
    pub client_id: ClientId,
    pub counterparty: Counterparty<CounterpartyClientId, EmptyString>,
    pub version: Version,
    pub delay_period: u64,
}
