use macros::model;

use crate::{
    bytes::Bytes,
    ibc::core::{client::height::Height, connection::version::Version},
    id::ConnectionId,
};

#[model(proto(raw(protos::ibc::core::connection::v1::MsgConnectionOpenAck)))]
pub struct MsgConnectionOpenAck {
    pub connection_id: ConnectionId,
    pub counterparty_connection_id: ConnectionId,
    pub version: Version,
    pub client_state: Bytes,
    // TODO: Make this type generic
    pub proof_height: Height,
    pub proof_try: Bytes,
    pub proof_client: Bytes,
    pub proof_consensus: Bytes,
    // TODO: Make this type generic
    pub consensus_height: Height,
}
