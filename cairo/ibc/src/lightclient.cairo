use starknet::ContractAddress;
use crate::types::ClientId;

#[derive(Drop, Serde)]
pub struct ConsensusStateUpdate {
    pub client_state_commitment: u256,
    pub consensus_state_commitment: u256,
    pub height: u64,
}

#[starknet::interface]
pub trait ILightClient<TContractState> {
    fn create_client(
        ref self: TContractState,
        caller: ContractAddress,
        client_id: ClientId,
        client_state_bytes: ByteArray,
        consensus_state_bytes: ByteArray,
        relayer: ContractAddress,
    ) -> (ConsensusStateUpdate, ByteArray);

    fn update_client(
        ref self: TContractState,
        caller: ContractAddress,
        client_id: ClientId,
        client_message: ByteArray,
        relayer: ContractAddress,
    ) -> ConsensusStateUpdate;

    fn verify_membership(
        self: @TContractState,
        client_id: ClientId,
        height: u64,
        proof: ByteArray,
        key: ByteArray,
        value: ByteArray,
    ) -> bool;
}
