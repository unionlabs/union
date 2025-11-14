use starknet::ContractAddress;
use crate::types::ClientId;

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
