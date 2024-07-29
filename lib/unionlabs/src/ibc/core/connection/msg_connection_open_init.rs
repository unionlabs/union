use macros::model;

use crate::{
    ibc::core::connection::{counterparty::Counterparty, version::Version},
    id::ClientId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenInit)))]
pub struct MsgConnectionOpenInit {
    pub client_id: ClientId,
    pub counterparty: Counterparty,
    pub version: Version,
    pub delay_period: u64,
}
