#[starknet::contract]
mod MockClient {
    use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
    use core::keccak::compute_keccak_byte_array;
    use ibc::lightclient::ConsensusStateUpdate;
    use ibc::types::ClientId;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl MockClientImpl of ibc::lightclient::ILightClient<ContractState> {
        fn create_client(
            ref self: ContractState,
            caller: ContractAddress,
            client_id: ClientId,
            client_state_bytes: ByteArray,
            consensus_state_bytes: ByteArray,
            relayer: ContractAddress,
        ) -> (ConsensusStateUpdate, ByteArray) {
            (
                ConsensusStateUpdate {
                    client_state_commitment: compute_keccak_byte_array(@client_state_bytes),
                    consensus_state_commitment: compute_keccak_byte_array(@consensus_state_bytes),
                    height: 10,
                },
                "counterparty-chain",
            )
        }

        fn update_client(
            ref self: ContractState,
            caller: ContractAddress,
            client_id: ClientId,
            client_message: ByteArray,
            relayer: ContractAddress,
        ) -> ConsensusStateUpdate {
            let (offset, size) = client_message.read_u32(0);
            let (offset, client_state_bytes) = client_message.read_bytes(offset, size);
            let (offset, size) = client_message.read_u32(offset);
            let (_, consensus_state_bytes) = client_message.read_bytes(offset, size);

            ConsensusStateUpdate {
                client_state_commitment: compute_keccak_byte_array(@client_state_bytes),
                consensus_state_commitment: compute_keccak_byte_array(@consensus_state_bytes),
                height: 10,
            }
        }

        fn verify_membership(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: ByteArray,
            key: ByteArray,
            value: ByteArray,
        ) -> bool {
            true
        }
    }
}
