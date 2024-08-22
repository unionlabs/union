// use macros::model;

use crate::{hash::H160, ibc::core::client::height::Height};

// #[model(proto(
//     raw(protos::union::ibc::lightclients::ethereum::v1::ClientState),
//     into,
//     from
// ))]

pub struct ClientState {
    pub l1_contract_address: H160,
    pub l2_contract_address: H160,
    pub table_handle: Vec<u8>, // TODO(aeryz): AccountAddress
    pub frozen_height: Height,
}
