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

pub mod ics23;
pub mod types;

#[starknet::contract]
mod CometblsLightClient {
    use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
    use alexandria_math::opt_math::OptBitShift;
    use core::num::traits::Zero;
    use ibc::lightclient::ConsensusStateUpdate;
    use ibc::types::{ClientId, Timestamp, TimestampTrait};
    use starknet::storage::{Map, StorageMapReadAccess, StorageMapWriteAccess, StoragePathEntry};
    use starknet::{ContractAddress, get_execution_info};
    use crate::ics23::{MembershipProof, MembershipProofTrait};
    use crate::types::{ClientState, ConsensusState, Header};

    pub const CLIENT_STATE_NOT_FOUND: felt252 = 'CLIENT_STATE_NOT_FOUND';
    pub const CONSENSUS_STATE_NOT_FOUND: felt252 = 'CONSENSUS_STATE_NOT_FOUND';
    pub const INVALID_HEADER: felt252 = 'INVALID_HEADER';
    pub const INVALID_PROOF: felt252 = 'INVALID_PROOF';

    #[storage]
    struct Storage {
        client_states: Map<ClientId, Option<ClientState>>,
        consensus_states: Map<ClientId, Map<u64, Option<ConsensusState>>>,
    }

    #[abi(embed_v0)]
    impl CometblsLightClientIbcImpl of ibc::lightclient::ILightClient<ContractState> {
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
            let mut client_state = self
                .client_states
                .read(client_id)
                .expect(CLIENT_STATE_NOT_FOUND);
            assert!(client_state.frozen_height == 0, "frozen client");

            let mut client_message = client_message.span();
            let header: Header = Serde::deserialize(ref client_message).expect(INVALID_HEADER);

            let mut consensus_state_entry = self.consensus_states.entry(client_id);
            let consensus_state = consensus_state_entry
                .read(header.trusted_height)
                .expect(CONSENSUS_STATE_NOT_FOUND);

            let (untrusted_height_number, untrusted_timestamp) = verify_header(
                @header, @consensus_state, @client_state,
            );

            if untrusted_height_number > client_state.latest_height {
                client_state.latest_height = untrusted_height_number;
                self.client_states.write(client_id, Some(client_state));
            }

            // TODO(aeryz): check for misbehaviour
            consensus_state_entry
                .write(
                    untrusted_height_number,
                    Some(
                        ConsensusState {
                            timestamp: TimestampTrait::from_nanos(untrusted_timestamp),
                            app_hash: header.signed_header.app_hash,
                            next_validators_hash: header.signed_header.next_validators_hash,
                        },
                    ),
                );

            // TODO(aeryz): fix the commitment
            ConsensusStateUpdate {
                consensus_state_commitment: Default::default(), height: untrusted_height_number,
            }
        }

        fn verify_membership(
            self: @ContractState,
            client_id: ClientId,
            height: u64,
            proof: Array<felt252>,
            key: ByteArray,
            value: ByteArray,
        ) -> bool {
            let client_state = self.client_states.read(client_id).expect(CLIENT_STATE_NOT_FOUND);

            assert!(client_state.frozen_height == 0, "frozen client");

            let consensus_state = self
                .consensus_states
                .entry(client_id)
                .read(height)
                .expect(CONSENSUS_STATE_NOT_FOUND);

            let mut proof = proof.span();
            let proof: MembershipProof = Serde::deserialize(ref proof).expect(INVALID_PROOF);

            let mut full_key = Default::default();

            // WASMD_CONTRACT_STORE_PREFIX
            full_key.append_u8(0x03);
            full_key.append_u256(client_state.contract_address);
            // IBC_UNION_COSMWASM_COMMITMENT_PREFIX
            full_key.append_u8(0x00);
            full_key.append(@key);

            let app_hash = u256_to_u32_array(consensus_state.app_hash);

            proof.verify(app_hash, "wasm", full_key, value).is_ok()
        }

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
                .expect(CONSENSUS_STATE_NOT_FOUND)
                .timestamp
        }

        fn get_latest_height(self: @ContractState, client_id: ClientId) -> u64 {
            self.client_states.read(client_id).expect(CLIENT_STATE_NOT_FOUND).latest_height
        }
    }

    fn verify_header(
        header: @Header, consensus_state: @ConsensusState, client_state: @ClientState,
    ) -> (u64, u64) {
        let untrusted_height_number = *header.signed_header.height;
        let trusted_height_number = *header.trusted_height;

        // revert CometblsClientLib.ErrUntrustedHeightLTETrustedHeight();
        assert!(untrusted_height_number > trusted_height_number, "untrusted height lte");

        let trusted_timestamp = *consensus_state.timestamp;
        // Normalize to nanosecond because ibc recvPacket expects nanos...
        let untrusted_timestamp = *header.signed_header.secs * 1_000_000_000
            + *header.signed_header.nanos;

        // revert CometblsClientLib.ErrUntrustedTimestampLTETrustedTimestamp();
        assert!(untrusted_timestamp > trusted_timestamp.nanos(), "err")

        let current_time = get_execution_info().block_info.block_timestamp * 1_000_000_000;

        assert!(
            is_expired(trusted_timestamp.nanos(), *client_state.trusting_period, current_time),
            "expired",
        );

        let max_clock_drift = current_time + *client_state.max_clock_drift;
        assert!(untrusted_timestamp < max_clock_drift, "max clock drift exceeded");

        // We want to verify that 1/3 of trusted valset & 2/3 of untrusted valset signed.
        // In adjacent verification, trusted vals = untrusted vals.
        // In non adjacent verification, untrusted vals are coming from the untrusted header.
        let trusted_validators_hash = consensus_state.next_validators_hash;
        if untrusted_height_number == trusted_height_number + 1 {
            assert!(
                header.signed_header.validators_hash == trusted_validators_hash,
                "invalid untrusted valhash",
            );
        }
        // TODO(aeryz): verify_zkp

        (untrusted_height_number, untrusted_timestamp)
    }

    fn is_expired(header_time: u64, trusting_period: u64, current_time: u64) -> bool {
        current_time > (header_time + trusting_period)
    }

    /// Convert a u256 into [u32; 8] (LE).
    fn u256_to_u32_array(n: u256) -> [u32; 8] {
        const MASK: u128 = 0xFFFFFFFF;
        [
            (n.low ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.low, 32) ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.low, 32 * 2) ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.low, 32 * 3) ^ MASK).try_into().unwrap(),
            (n.high ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.high, 32) ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.high, 32 * 2) ^ MASK).try_into().unwrap(),
            (OptBitShift::shr(n.high, 32 * 3) ^ MASK).try_into().unwrap(),
        ]
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

#[cfg(test)]
mod tests {
    use super::CometblsLightClient::*;
    #[test]
    fn test_chain_id_to_string() {
        assert_eq!(chain_id_to_string('cometbls'), "cometbls");
        assert_eq!(chain_id_to_string('cometbls-aseldfleasndf'), "cometbls-aseldfleasndf");
        assert_eq!(
            chain_id_to_string('3232323232323232323232323232323'),
            "3232323232323232323232323232323",
        );
        assert_eq!(chain_id_to_string('1'), "1");
        assert_eq!(chain_id_to_string(''), "");
    }
}
