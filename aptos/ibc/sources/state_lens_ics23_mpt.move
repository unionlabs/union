// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
// 

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module ibc::state_lens_ics23_mpt_lc {
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{String, Self};

    use aptos_std::smart_table::{Self, SmartTable};
    use aptos_std::aptos_hash::keccak256;
    use aptos_std::from_bcs;
    use aptos_std::bcs;
    use aptos_std::object;

    use ibc::height::{Self, Height};
    use ibc::ethabi;
    use ibc::bcs_utils;
    use ibc::commitment;
    use ibc::cometbls_lc;
    use ibc::create_lens_client_event::{Self, CreateLensClientEvent};

    const E_INVALID_CLIENT_STATE: u64 = 35200;
    const E_CONSENSUS_STATE_TIMESTAMP_ZERO: u64 = 35201;
    const E_SIGNED_HEADER_HEIGHT_NOT_MORE_RECENT: u64 = 35202;
    const E_SIGNED_HEADER_TIMESTAMP_NOT_MORE_RECENT: u64 = 35203;
    const E_HEADER_EXCEEDED_TRUSTING_PERIOD: u64 = 35204;
    const E_HEADER_EXCEEDED_MAX_CLOCK_DRIFT: u64 = 35205;
    const E_VALIDATORS_HASH_MISMATCH: u64 = 35206;
    const E_INVALID_ZKP: u64 = 35207;
    const E_FROZEN_CLIENT: u64 = 35208;
    const E_INVALID_MISBEHAVIOUR: u64 = 35209;
    const E_L2_CONSENSUS_STATE_PROOF_VERIFICATION: u64 = 35210;
    const E_UNIMPLEMENTED: u64 = 35299;

    struct State has key, store {
        client_state: ClientState,
        consensus_states: SmartTable<u64, ConsensusState>
    }

    struct ClientState has copy, drop, store {
        /// L2 chain ID. This is the same as the ID of the chain being tracked by `self.l2_client_id`.
        ///
        /// ("C")
        l2_chain_id: String,

        /// L1 client ID. This is the ID of the L1 client running on A that is used to check the L2
        /// inclusion proof against.
        ///
        /// ("B" on "A")
        l1_client_id: u32,

        /// L2 client ID. This is the ID of the L2 client running on B (L1) tracking the C (L2).
        ///
        /// ("C" on "B")
        l2_client_id: u32,

        /// L2 latest height
        l2_latest_height: u64,

        /// the offset at which we extract the u64 timestamp from the l2 consensus state
        /// timestamp = consensus_state[timestamp_offset:timestamp_offset+8]
        timestamp_offset: u16,
        /// the offset at which we extract the bytes32 state root from the l2 consensus state
        /// state_root = consensus_state[state_root_offset:state_root_offset+32]
        state_root_offset: u16,
        /// the offset at which we extract the bytes32 storage root (of the ibc contract on the l2) from the l2 consensus state
        /// storage_root = consensus_state[storage_root_offset:storage_root_offset+32]
        storage_root_offset: u16
    }

    struct ConsensusState has copy, drop, store {
        /// Timestamp of the execution layer.
        timestamp: u64,
        /// State root of the execution layer.
        state_root: vector<u8>,
        /// Storage root of the ibc contract extracted from the state root.
        storage_root: vector<u8>
    }

    struct Header has copy, drop {
        l1_height: Height,
        l2_height: Height,
        /// Proof of the L2 consensus state as stored in the state of the L1.
        l2_consensus_state_proof: vector<u8>,
        l2_consensus_state: vector<u8>
    }

    public fun create_client(
        ibc_signer: &signer,
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>
    ): (vector<u8>, vector<u8>, String, Option<CreateLensClientEvent>) {
        let client_state = decode_client_state(client_state_bytes);
        let consensus_state = decode_consensus_state(consensus_state_bytes);

        assert!(
            client_state.l2_latest_height != 0 && consensus_state.timestamp != 0,
            E_INVALID_CLIENT_STATE
        );

        let consensus_states = smart_table::new<u64, ConsensusState>();
        smart_table::upsert(
            &mut consensus_states,
            client_state.l2_latest_height,
            consensus_state
        );

        let state = State { client_state: client_state, consensus_states: consensus_states };

        let store_constructor =
            object::create_named_object(ibc_signer, bcs::to_bytes<u32>(&client_id));
        let client_signer = object::generate_signer(&store_constructor);

        move_to(&client_signer, state);

        let lens_client_event = create_lens_client_event::new(
            client_id,
            client_state.l2_chain_id,
            client_state.l1_client_id,
            client_state.l2_client_id,
        );

        (client_state_bytes, consensus_state_bytes, client_state.l2_chain_id, option::some(lens_client_event))
    }

    public fun latest_height(client_id: u32): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        state.client_state.l2_latest_height
    }

    public fun update_client(
        client_id: u32, client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) acquires State {
        let header = decode_header(client_msg);

        let state = borrow_global_mut<State>(get_client_address(client_id));

        assert!(
            cometbls_lc::verify_membership(
                state.client_state.l1_client_id,
                height::get_revision_height(&header.l1_height),
                header.l2_consensus_state_proof,
                commitment::consensus_state_commitment_key(
                    state.client_state.l2_client_id,
                    height::get_revision_height(&header.l2_height)
                ),
                keccak256(header.l2_consensus_state)
            ) == 0,
            E_L2_CONSENSUS_STATE_PROOF_VERIFICATION
        );

        let l2_timestamp =
            extract_uint64(
                vector::slice(
                    &header.l2_consensus_state,
                    (state.client_state.timestamp_offset as u64),
                    ((state.client_state.timestamp_offset + 8) as u64)
                )
            );
        let l2_state_root = vector::slice(
            &header.l2_consensus_state,
            (state.client_state.state_root_offset as u64),
            ((state.client_state.state_root_offset + 32) as u64)
        );
        let l2_storage_root = vector::slice(
            &header.l2_consensus_state,
            (state.client_state.storage_root_offset as u64),
            ((state.client_state.storage_root_offset + 32) as u64)
        );

        let new_height = height::get_revision_height(&header.l2_height);

        if ((state.client_state.l2_latest_height as u64) < new_height) {
            state.client_state.l2_latest_height = new_height;
        };

        let consensus_state = ConsensusState {
            timestamp: l2_timestamp,
            state_root: l2_state_root,
            storage_root: l2_storage_root
        };

        smart_table::upsert(&mut state.consensus_states, new_height, consensus_state);

        (
            bcs::to_bytes(&state.client_state),
            vector[encode_consensus_state(&consensus_state)],
            vector[new_height]
        )
    }

    public fun report_misbehaviour(
        _client_id: u32, _misbehaviour: vector<u8>
    ) {}

    public fun verify_membership(
        _client_id: u32,
        _height: u64,
        _proof: vector<u8>,
        _key: vector<u8>,
        _value: vector<u8>
    ): u64 {
        0
    }

    public fun verify_non_membership(
        _client_id: u32,
        _height: u64,
        _proof: vector<u8>,
        _path: vector<u8>
    ): u64 {
        0
    }

    public fun status(_client_id: u32): u64 {
        0
    }

    public fun get_timestamp_at_height(_client_id: u32, _height: u64): u64 {
        0
    }

    public fun get_client_state(client_id: u32): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        bcs::to_bytes(&state.client_state)
    }

    public fun get_consensus_state(client_id: u32, height: u64): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        encode_consensus_state(consensus_state)
    }

    public fun check_for_misbehaviour(
        _client_id: u32, _header: vector<u8>
    ): bool {
        false
    }

    fun decode_client_state(buf: vector<u8>): ClientState {
        let buf = bcs_utils::new(buf);

        let l2_chain_id = bcs_utils::peel_string(&mut buf);
        let l1_client_id = bcs_utils::peel_u32(&mut buf);
        let l2_client_id = bcs_utils::peel_u32(&mut buf);
        let l2_latest_height = bcs_utils::peel_u64(&mut buf);
        let timestamp_offset = bcs_utils::peel_u16(&mut buf);
        let state_root_offset = bcs_utils::peel_u16(&mut buf);
        let storage_root_offset = bcs_utils::peel_u16(&mut buf);

        ClientState {
            l2_chain_id,
            l1_client_id,
            l2_client_id,
            l2_latest_height,
            timestamp_offset,
            state_root_offset,
            storage_root_offset
        }
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let index = 0;
        let timestamp = (ethabi::decode_uint(&buf, &mut index) as u64);
        let state_root = vector::slice(&buf, 32, 64);
        let storage_root = vector::slice(&buf, 64, 96);

        ConsensusState { timestamp, state_root, storage_root }
    }

    fun decode_header(buf: vector<u8>): Header {
        let buf = bcs_utils::new(buf);

        let l1_height = height::decode_bcs(&mut buf);
        let l2_height = height::decode_bcs(&mut buf);
        let l2_consensus_state_proof = bcs_utils::peel_bytes(&mut buf);
        let l2_consensus_state = bcs_utils::peel_bytes(&mut buf);

        Header {
            l1_height,
            l2_height,
            l2_consensus_state_proof,
            l2_consensus_state
        }
    }

    fun encode_consensus_state(cs: &ConsensusState): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_uint<u64>(&mut buf, cs.timestamp);

        vector::append(&mut buf, cs.state_root);
        vector::append(&mut buf, cs.storage_root);

        buf
    }

    fun extract_uint64(buf: vector<u8>): u64 {
        vector::reverse(&mut buf);
        from_bcs::to_u64(buf)
    }

    fun get_client_address(client_id: u32): address {
        let vault_addr = object::create_object_address(&@ibc, b"IBC_VAULT_SEED");

        object::create_object_address(&vault_addr, bcs::to_bytes<u32>(&client_id))
    }
}
