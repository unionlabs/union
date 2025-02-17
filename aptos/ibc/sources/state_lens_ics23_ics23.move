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

module ibc::state_lens_ics23_ics23_lc {
    use std::vector;
    use std::bcs;
    use std::string::String;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::object;
    // use std::timestamp;
    // use ibc::ics23;
    use ibc::ethabi;
    use ibc::bcs_utils;
    use ibc::height::{Self, Height};

    struct TendermintConsensusState has drop {
        timestamp: u64,
        app_hash: vector<u8>,
        next_validators_hash: vector<u8>
    }

    struct State has key, store {
        client_state: ClientState,
        consensus_states: SmartTable<u64, ConsensusState>
    }

    struct Header has drop {
        l1_height: Height,
        l2_height: Height,
        l2_consensus_state_proof: vector<u8>,
        l2_consensus_state: vector<u8>
    }

    struct Misbehaviour has drop {
        header_a: Header,
        header_b: Header
    }

    struct ClientState has copy, drop, store {
        l2_chain_id: String,
        l1_client_id: u32,
        l2_client_id: u32,
        l2_latest_height: u64,
        contract_address: vector<u8>
    }

    struct ConsensusState has copy, drop, store {
        timestamp: u64,
        app_hash: vector<u8>,
    }

    // Function to mock the creation of a client
    public fun create_client(
        ibc_signer: &signer,
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>
    ): (vector<u8>, vector<u8>) {
        let client_state = decode_client_state(client_state_bytes);
        let consensus_state = decode_consensus_state(consensus_state_bytes);

        // assert!(
        //     !height::is_zero(&client_state.latest_height)
        //         && consensus_state.timestamp != 0,
        //     E_INVALID_CLIENT_STATE
        // );

        // assert!(string::length(&client_state.chain_id) <= 31, E_INVALID_CLIENT_STATE);

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

        (client_state_bytes, consensus_state_bytes)
    }

    public fun latest_height(client_id: u32): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        state.client_state.l2_latest_height
    }

    public fun verify_header(
        _header: &Header, _state: &State, _consensus_state: &ConsensusState
    ) {
    }

    public fun update_client(
        client_id: u32, client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) acquires State {
        let state = borrow_global_mut<State>(get_client_address(client_id));

        let header = decode_header(client_msg);

        let l2_consensus_state = decode_tm_consensus_state(header.l2_consensus_state);

        let updated_height = height::get_revision_height(&header.l2_height);

        state.client_state.l2_latest_height = updated_height;

        let new_consensus_state = ConsensusState {
            timestamp: l2_consensus_state.timestamp,
            app_hash: l2_consensus_state.app_hash
        };

        smart_table::upsert(&mut state.consensus_states, updated_height, new_consensus_state);

        (
            encode_client_state(&state.client_state),
            vector[encode_consensus_state(&new_consensus_state)],
            vector[updated_height]
        )
    }

    // Checks whether `misbehaviour` is valid and freezes the client
    public fun report_misbehaviour(
        _client_id: u32, _misbehaviour: vector<u8>
    ) {
    }

    public fun verify_membership(
        _client_id: u32,
        _height: u64,
        _proof: vector<u8>,
        _key: vector<u8>,
        _value: vector<u8>
    ): u64 /* acquires State */ {
        // let state = borrow_global<State>(get_client_address(client_id));
        // let consensus_state = smart_table::borrow(&state.consensus_states, height);

        // let path = vector<u8>[0x03];
        // vector::append(&mut path, state.client_state.contract_address);
        // vector::append(&mut path, key);

        // ics23::verify_membership(
        //     ics23::decode_membership_proof(proof),
        //     consensus_state.app_hash.hash,
        //     b"wasm", // HARDCODED PREFIX
        //     path,
        //     value
        // );

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

    public fun is_frozen(_client_id: u32): bool {
        // TODO: Implement this
        false
    }

    public fun status(_client_id: u32): u64 {
        // TODO(aeryz): fetch these status from proper exported consts
        0
    }

    fun get_client_address(client_id: u32): address {
        let vault_addr = object::create_object_address(&@ibc, b"IBC_VAULT_SEED");

        object::create_object_address(&vault_addr, bcs::to_bytes<u32>(&client_id))
    }

    public fun get_timestamp_at_height(client_id: u32, height: u64): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        consensus_state.timestamp
    }

    public fun get_client_state(client_id: u32): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        encode_client_state(&state.client_state)
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

        ClientState {
            l2_chain_id: bcs_utils::peel_string(&mut buf),
            l1_client_id: bcs_utils::peel_u32(&mut buf),
            l2_client_id: bcs_utils::peel_u32(&mut buf),
            l2_latest_height: bcs_utils::peel_u64(&mut buf),
            contract_address: bcs_utils::peel_fixed_bytes(&mut buf, 32)
        }
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let index = 0;
        let timestamp = ethabi::decode_uint(&buf, &mut index);
        let app_hash = vector::slice(&buf, 32, 64);

        ConsensusState {
            timestamp: (timestamp as u64),
            app_hash
        }
    }

    fun decode_tm_consensus_state(buf: vector<u8>): TendermintConsensusState {
        let index = 0;
        let timestamp = ethabi::decode_uint(&buf, &mut index);
        let app_hash = vector::slice(&buf, 32, 64);
        let next_validators_hash = vector::slice(&buf, 64, 96);

        TendermintConsensusState {
            timestamp: (timestamp as u64),
            app_hash,
            next_validators_hash: next_validators_hash
        }
    }

    fun encode_consensus_state(cs: &ConsensusState): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_uint<u64>(&mut buf, cs.timestamp);

        vector::append(&mut buf, cs.app_hash);

        buf
    }

    struct PartialClientState has drop {
        l2_chain_id: String,
        l1_client_id: u32,
        l2_client_id: u32,
        l2_latest_height: u64,
    }

    fun encode_client_state(cs: &ClientState): vector<u8> {
        let buf = vector::empty();

        let partial = PartialClientState {
            l2_chain_id: cs.l2_chain_id,
            l1_client_id: cs.l1_client_id,
            l2_client_id: cs.l2_client_id,
            l2_latest_height: cs.l2_latest_height,
        };

        vector::append(&mut buf, bcs::to_bytes(&partial));
        vector::append(&mut buf, cs.contract_address);

        buf
    }

    fun decode_header(buf: vector<u8>): Header {
        let buf = bcs_utils::new(buf);

        Header {
            l1_height: height::decode_bcs(&mut buf),
            l2_height: height::decode_bcs(&mut buf),
            l2_consensus_state_proof: bcs_utils::peel_bytes(&mut buf),
            l2_consensus_state: bcs_utils::peel_bytes(&mut buf)
        }
    }

    #[test]
    fun see_client_state() {
        let cs = x"0a62626e2d746573742d3506000000150000003fca020000000000470680b0f0eb26976b5fc59c29d456f24312d7e1ceb5e6df9eebaa7e55ff96a5";
        std::debug::print(&decode_client_state(cs));
    }
}
