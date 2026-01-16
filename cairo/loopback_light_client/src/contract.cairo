// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

#[starknet::contract]
mod StarknetLightClient {
    use alexandria_math::opt_math::OptBitShift;
    use core::keccak::compute_keccak_byte_array;
    use core::num::traits::Zero;
    use ibc::lightclient::ConsensusStateUpdate;
    use ibc::types::{ClientId, Timestamp, TimestampTrait};
    use starknet::storage::{Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePathEntry};
    use starknet::syscalls::get_block_hash_syscall;
    use starknet::{ContractAddress, SyscallResultTrait};
    use crate::types::{BlockTrait, ClientState, ConsensusState, ConsensusStateTrait, Header};

    pub mod Error {
        pub const CLIENT_STATE_NOT_FOUND: felt252 = 'CLIENT_STATE_NOT_FOUND';
        pub const CONSENSUS_STATE_NOT_FOUND: felt252 = 'CONSENSUS_STATE_NOT_FOUND';
        pub const INVALID_HEADER: felt252 = 'INVALID_HEADER';
        pub const INVALID_PROOF: felt252 = 'INVALID_PROOF';
        pub const BLOCK_HASH_MISMATCH: felt252 = 'BLOCK_HASH_MISMATCH';
    }

    #[storage]
    struct Storage {
        client_states: Map<ClientId, Option<ClientState>>,
        consensus_states: Map<ClientId, Map<u64, Option<ConsensusState>>>,
    }

    #[abi(embed_v0)]
    impl StarknetLightClientIbcImpl of ibc::lightclient::ILightClient<ContractState> {
        fn create_client(
            ref self: ContractState,
            caller: ContractAddress,
            client_id: ClientId,
            client_state_bytes: Array<felt252>,
            consensus_state_bytes: Array<felt252>,
            relayer: ContractAddress,
        ) -> (ConsensusStateUpdate, ByteArray) {
            let mut client_state_bytes = client_state_bytes.span();
            let client_state: ClientState = Serde::deserialize(ref client_state_bytes).unwrap();
            let mut consensus_state_bytes = consensus_state_bytes.span();
            let consensus_state: ConsensusState = Serde::deserialize(ref consensus_state_bytes)
                .unwrap();

            assert!(
                client_state.latest_height != 0 && consensus_state.timestamp.is_non_zero(),
                "invalid initial client/consensus state",
            );
            let counterparty_chain_id = chain_id_to_string(client_state.chain_id);
            let height = client_state.latest_height;

            self.client_states.write(client_id, Some(client_state));
            self.consensus_states.entry(client_id).write(height, Some(consensus_state));

            (
                ConsensusStateUpdate { consensus_state_commitment: Default::default(), height },
                counterparty_chain_id,
            )
        }

        fn update_client(
            ref self: ContractState,
            caller: ContractAddress,
            client_id: ClientId,
            client_message: Array<felt252>,
            relayer: ContractAddress,
        ) -> ConsensusStateUpdate {
            let mut client_message = client_message.span();
            let header: Header = Serde::deserialize(ref client_message)
                .expect(Error::INVALID_HEADER);

            let block_hash = get_block_hash_syscall(header.block.block_number).unwrap_syscall();

            assert(block_hash == header.block.hash(), Error::BLOCK_HASH_MISMATCH);

            let consensus_state = ConsensusState {
                contracts_trie_root: header.block.contracts_trie_root,
                classes_trie_root: header.block.classes_trie_root,
                timestamp: TimestampTrait::from_secs(header.block.block_timestamp),
            };

            let consensus_state_commitment = compute_keccak_byte_array(
                @consensus_state.ethabi_encode(),
            );

            self
                .consensus_states
                .entry(client_id)
                .write(header.block.block_number, Some(consensus_state));

            ConsensusStateUpdate { consensus_state_commitment, height: header.block.block_number }
        }

        /// Loopback client is only used for the trie root verification
        fn verify_membership(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: Array<felt252>,
            key: ByteArray,
            value: ByteArray,
        ) -> bool {
            false
        }

        /// Loopback client is only used for the trie root verification
        fn verify_non_membership(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: Array<felt252>,
            key: ByteArray,
        ) -> bool {
            false
        }

        fn get_timestamp_at_height(
            self: @ContractState, client_id: ClientId, height: u64,
        ) -> Timestamp {
            self
                .consensus_states
                .entry(client_id)
                .read(height)
                .expect(Error::CONSENSUS_STATE_NOT_FOUND)
                .timestamp
        }

        fn get_latest_height(self: @ContractState, client_id: ClientId) -> u64 {
            self.client_states.read(client_id).expect(Error::CLIENT_STATE_NOT_FOUND).latest_height
        }

        fn get_consensus_state(
            self: @ContractState, client_id: ClientId, height: u64,
        ) -> Array<felt252> {
            let mut buf = array![];

            self
                .consensus_states
                .entry(client_id)
                .read(height)
                .expect(Error::CONSENSUS_STATE_NOT_FOUND)
                .serialize(ref buf);

            buf
        }
    }

    pub fn chain_id_to_string(source: felt252) -> ByteArray {
        let mut source_u256: u256 = source.into();

        let mut chain_id = Default::default();

        let mut i = 0;
        while OptBitShift::shr(source_u256, i * 8) != 0 {
            i += 1;
        }

        chain_id.append_word(source, i.into());

        chain_id
    }
}

