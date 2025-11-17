use starknet::ContractAddress;
use crate::types::{ClientId, ConnectionId};

#[derive(Drop, starknet::Event)]
pub struct RegisterClient {
    #[key]
    pub client_type: ByteArray,
    pub client_address: ContractAddress,
}

#[derive(Drop, starknet::Event)]
pub struct CreateClient {
    #[key]
    pub client_type: ByteArray,
    #[key]
    pub client_id: ClientId,
    #[key]
    pub counterparty_chain_id: ByteArray,
}

#[derive(Drop, starknet::Event)]
pub struct UpdateClient {
    #[key]
    pub client_id: ClientId,
    #[key]
    pub height: u64,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenInit {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenTry {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenAck {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}

#[derive(Drop, starknet::Event)]
pub struct ConnectionOpenConfirm {
    #[key]
    pub connection_id: ConnectionId,
    #[key]
    pub client_id: ClientId,
    pub counterparty_client_id: ClientId,
    pub counterparty_connection_id: ConnectionId,
}
