use macros::model;
use crate::primitives::Bytes;

use crate::{
    ibc::core::{
        client::height::Height,
        connection::{counterparty::Counterparty, version::Version},
    },
    id::ClientId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenTry)))]
pub struct MsgConnectionOpenTry {
    pub client_id: ClientId,
    pub counterparty: Counterparty,
    pub delay_period: u64,
    pub counterparty_versions: Vec<Version>,
    pub proof_height: Height,
    pub proof_init: Bytes,
}
