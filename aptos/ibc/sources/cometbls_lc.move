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

module ibc::cometbls_lc {
    use aptos_std::smart_table::{Self, SmartTable};

    use std::vector;
    use std::bcs;
    use std::string::{Self, String};
    use std::object;
    use std::timestamp;
    use std::option::{Self, Option};

    use ibc::ics23;
    use ibc::ethabi;
    use ibc::bcs_utils;
    use ibc::create_lens_client_event::CreateLensClientEvent;
    use ibc::groth16_verifier::{Self, ZKP};
    use ibc::height::{Self, Height};

    friend ibc::light_client;
    friend ibc::state_lens_ics23_ics23_lc;
    friend ibc::state_lens_ics23_mpt_lc;

    const E_INVALID_CLIENT_STATE: u64 = 35100;
    const E_CONSENSUS_STATE_TIMESTAMP_ZERO: u64 = 35101;
    const E_SIGNED_HEADER_HEIGHT_NOT_MORE_RECENT: u64 = 35102;
    const E_SIGNED_HEADER_TIMESTAMP_NOT_MORE_RECENT: u64 = 35103;
    const E_HEADER_EXCEEDED_TRUSTING_PERIOD: u64 = 35104;
    const E_HEADER_EXCEEDED_MAX_CLOCK_DRIFT: u64 = 35105;
    const E_VALIDATORS_HASH_MISMATCH: u64 = 35106;
    const E_INVALID_ZKP: u64 = 35107;
    const E_FROZEN_CLIENT: u64 = 35108;
    const E_INVALID_MISBEHAVIOUR: u64 = 35109;
    const E_UNIMPLEMENTED: u64 = 35199;

    struct State has key, store {
        client_state: ClientState,
        consensus_states: SmartTable<u64, ConsensusState>
    }

    struct Timestamp has drop, copy {
        seconds: u64,
        nanos: u32
    }

    struct LightHeader has drop, copy {
        height: u64,
        time: Timestamp,
        validators_hash: vector<u8>,
        next_validators_hash: vector<u8>,
        app_hash: vector<u8>
    }

    struct Header has drop {
        signed_header: LightHeader,
        trusted_height: Height,
        zero_knowledge_proof: ZKP
    }

    struct Misbehaviour has drop {
        header_a: Header,
        header_b: Header
    }

    struct ClientState has copy, drop, store {
        chain_id: String,
        trusting_period: u64,
        max_clock_drift: u64,
        frozen_height: Height,
        latest_height: Height,
        contract_address: vector<u8>
    }

    struct MerkleRoot has copy, drop, store {
        hash: vector<u8>
    }

    struct ConsensusState has copy, drop, store {
        timestamp: u64,
        app_hash: MerkleRoot,
        next_validators_hash: vector<u8>
    }

    // Function to mock the creation of a client
    public(friend) fun create_client(
        _: &signer,
        ibc_signer: &signer,
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>
    ): (vector<u8>, vector<u8>, String, Option<CreateLensClientEvent>) {
        let client_state = decode_client_state(client_state_bytes);
        let consensus_state = decode_consensus_state(consensus_state_bytes);

        assert!(
            !height::is_zero(&client_state.latest_height)
                && consensus_state.timestamp != 0,
            E_INVALID_CLIENT_STATE
        );

        assert!(string::length(&client_state.chain_id) <= 31, E_INVALID_CLIENT_STATE);

        let consensus_states = smart_table::new<u64, ConsensusState>();
        smart_table::upsert(
            &mut consensus_states,
            height::get_revision_height(&client_state.latest_height),
            consensus_state
        );

        let state = State { client_state: client_state, consensus_states: consensus_states };

        let store_constructor =
            object::create_named_object(ibc_signer, bcs::to_bytes<u32>(&client_id));
        let client_signer = object::generate_signer(&store_constructor);

        move_to(&client_signer, state);

        (client_state_bytes, consensus_state_bytes, client_state.chain_id, option::none())
    }

    public(friend) fun latest_height(client_id: u32): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        height::get_revision_height(&state.client_state.latest_height)
    }

    public(friend) fun verify_header(
        header: &Header, state: &State, consensus_state: &ConsensusState
    ) {
        assert!(consensus_state.timestamp != 0, E_CONSENSUS_STATE_TIMESTAMP_ZERO);

        let untrusted_height_number = header.signed_header.height;
        let trusted_height_number = height::get_revision_height(&header.trusted_height);

        assert!(
            untrusted_height_number > trusted_height_number,
            E_SIGNED_HEADER_HEIGHT_NOT_MORE_RECENT
        );

        let trusted_timestamp = consensus_state.timestamp;
        let untrusted_timestamp =
            header.signed_header.time.seconds * 1_000_000_000
                + (header.signed_header.time.nanos as u64);
        assert!(
            untrusted_timestamp > trusted_timestamp,
            E_SIGNED_HEADER_TIMESTAMP_NOT_MORE_RECENT
        );

        let current_time = timestamp::now_seconds() * 1_000_000_000;
        assert!(
            untrusted_timestamp < current_time + state.client_state.trusting_period,
            E_HEADER_EXCEEDED_TRUSTING_PERIOD
        );

        assert!(
            untrusted_timestamp < current_time + state.client_state.max_clock_drift,
            E_HEADER_EXCEEDED_MAX_CLOCK_DRIFT
        );

        if (untrusted_height_number == trusted_height_number + 1) {
            assert!(
                header.signed_header.validators_hash
                    == consensus_state.next_validators_hash,
                E_VALIDATORS_HASH_MISMATCH
            );
        };

        assert!(
            groth16_verifier::verify_zkp(
                &state.client_state.chain_id,
                &consensus_state.next_validators_hash,
                light_header_as_input_hash(&header.signed_header),
                &header.zero_knowledge_proof
            ),
            E_INVALID_ZKP
        );
    }

    public(friend) fun update_client(
        client_id: u32, client_msg: vector<u8>, _relayer: address
    ): (vector<u8>, vector<u8>, u64) acquires State {
        let header = decode_header(client_msg);

        let state = borrow_global_mut<State>(get_client_address(client_id));

        assert!(height::is_zero(&state.client_state.frozen_height), E_FROZEN_CLIENT);

        let consensus_state =
            smart_table::borrow<u64, ConsensusState>(
                &state.consensus_states,
                height::get_revision_height(&header.trusted_height)
            );

        verify_header(&header, state, consensus_state);

        let untrusted_height_number = header.signed_header.height;
        let untrusted_timestamp =
            header.signed_header.time.seconds * 1_000_000_000
                + (header.signed_header.time.nanos as u64);

        if (untrusted_height_number
            > height::get_revision_height(&state.client_state.latest_height)) {
            state.client_state.latest_height = height::new(0, untrusted_height_number);
        };

        let new_height = height::get_revision_height(&state.client_state.latest_height);

        let new_consensus_state = ConsensusState {
            timestamp: untrusted_timestamp,
            app_hash: MerkleRoot { hash: header.signed_header.app_hash },
            next_validators_hash: header.signed_header.next_validators_hash
        };

        smart_table::upsert(&mut state.consensus_states, new_height, new_consensus_state);

        (
            encode_client_state(&state.client_state),
            encode_consensus_state(&new_consensus_state),
            new_height
        )
    }

    // Checks whether `misbehaviour` is valid and freezes the client
    public(friend) fun misbehaviour(
        client_id: u32, misbehaviour: vector<u8>, _relayer: address
    ) acquires State {
        let Misbehaviour { header_a, header_b } = decode_misbehaviour(misbehaviour);

        assert!(
            header_a.signed_header.height >= header_b.signed_header.height,
            E_INVALID_MISBEHAVIOUR
        );

        let state = borrow_global_mut<State>(get_client_address(client_id));

        let consensus_state_a =
            smart_table::borrow(
                &state.consensus_states,
                height::get_revision_height(&header_a.trusted_height)
            );
        let consensus_state_b =
            smart_table::borrow(
                &state.consensus_states,
                height::get_revision_height(&header_b.trusted_height)
            );

        // verify both updates would have been accepted by the light client
        verify_header(&header_a, state, consensus_state_a);
        verify_header(&header_b, state, consensus_state_b);

        if (header_a.signed_header.height == header_b.signed_header.height) {
            // misbehaviour is only valid if
            assert!(
                header_a.signed_header == header_b.signed_header,
                E_INVALID_MISBEHAVIOUR
            );
        } else {
            assert!(
                consensus_state_a.timestamp > consensus_state_b.timestamp,
                E_INVALID_MISBEHAVIOUR
            );
        };

        height::set_revision_height(&mut state.client_state.frozen_height, 1);
    }

    public(friend) fun verify_membership(
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);

        let path = vector<u8>[0x03];
        vector::append(&mut path, state.client_state.contract_address);
        vector::append(&mut path, key);

        ics23::verify_membership(
            ics23::decode_membership_proof(proof),
            consensus_state.app_hash.hash,
            b"wasm", // HARDCODED PREFIX
            path,
            value
        );

        0
    }

    public(friend) fun verify_non_membership(
        _client_id: u32,
        _height: u64,
        _proof: vector<u8>,
        _path: vector<u8>
    ): u64 {
        0
    }

    public(friend) fun is_frozen(_client_id: u32): bool {
        // TODO: Implement this
        false
    }

    public(friend) fun status(_client_id: u32): u64 {
        // TODO(aeryz): fetch these status from proper exported consts
        0
    }

    fun get_client_address(client_id: u32): address {
        let vault_addr = object::create_object_address(&@ibc, b"IBC_VAULT_SEED");

        object::create_object_address(&vault_addr, bcs::to_bytes<u32>(&client_id))
    }

    public(friend) fun new_client_state(
        chain_id: string::String,
        trusting_period: u64,
        max_clock_drift: u64,
        frozen_height: Height,
        latest_height: Height,
        contract_address: vector<u8>
    ): ClientState {
        ClientState {
            chain_id,
            trusting_period,
            max_clock_drift,
            frozen_height,
            latest_height,
            contract_address
        }
    }

    public(friend) fun new_consensus_state(
        timestamp: u64, app_hash: MerkleRoot, next_validators_hash: vector<u8>
    ): ConsensusState {
        ConsensusState {
            timestamp: timestamp,
            app_hash: app_hash,
            next_validators_hash: next_validators_hash
        }
    }

    public(friend) fun new_merkle_root(hash: vector<u8>): MerkleRoot {
        MerkleRoot { hash: hash }
    }

    public(friend) fun get_timestamp_at_height(
        client_id: u32, height: u64
    ): u64 acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        consensus_state.timestamp
    }

    public(friend) fun get_client_state(client_id: u32): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        encode_client_state(&state.client_state)
    }

    public(friend) fun get_consensus_state(client_id: u32, height: u64): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        encode_consensus_state(consensus_state)
    }

    public(friend) fun mock_create_client(): (vector<u8>, vector<u8>) {
        let client_state = ClientState {
            chain_id: string::utf8(b"this-chain"),
            trusting_period: 0,
            max_clock_drift: 0,
            frozen_height: height::new(0, 0),
            latest_height: height::new(0, 1000),
            contract_address: vector::empty()
        };

        let consensus_state = ConsensusState {
            timestamp: 10000,
            app_hash: MerkleRoot {
                hash: x"0000000000000000000000000000000000000000000000000000000000000000"
            },
            next_validators_hash: x"0000000000000000000000000000000000000000000000000000000000000000"
        };

        let data1 = bcs::to_bytes(&client_state);
        let data2 = encode_consensus_state(&consensus_state);
        return (data1, data2)
    }

    public(friend) fun check_for_misbehaviour(
        client_id: u32, header: vector<u8>
    ): bool acquires State {
        let state = borrow_global_mut<State>(get_client_address(client_id));

        let header = decode_header(header);

        let expected_timestamp =
            header.signed_header.time.seconds * 1_000_000_000
                + (header.signed_header.time.nanos as u64);

        if (smart_table::contains(&state.consensus_states, header.signed_header.height)) {
            let ConsensusState {
                timestamp,
                app_hash: MerkleRoot { hash },
                next_validators_hash
            } = smart_table::borrow(
                &state.consensus_states, header.signed_header.height
            );

            if (timestamp != &expected_timestamp
                || hash != &header.signed_header.app_hash
                || next_validators_hash != &header.signed_header.next_validators_hash) {
                height::set_revision_height(&mut state.client_state.frozen_height, 1);
            };
        };

        // // TODO(aeryz): implement consensus state metadata tracking here
        false
    }

    fun decode_client_state(buf: vector<u8>): ClientState {
        let buf = bcs_utils::new(buf);

        let chain_id = bcs_utils::peel_string(&mut buf);
        let trusting_period = bcs_utils::peel_u64(&mut buf);
        let max_clock_drift = bcs_utils::peel_u64(&mut buf);
        let frozen_height = height::decode_bcs(&mut buf); // TODO: Not sure if its correc;
        let latest_height = height::decode_bcs(&mut buf);
        let contract_address = bcs_utils::peel_fixed_bytes(&mut buf, 32);

        ClientState {
            chain_id,
            trusting_period,
            max_clock_drift,
            frozen_height,
            latest_height,
            contract_address
        }
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let index = 0;
        let timestamp = ethabi::decode_uint(&buf, &mut index);
        let app_hash = vector::slice(&buf, 32, 64);
        let next_validators_hash = vector::slice(&buf, 64, 96);

        ConsensusState {
            timestamp: (timestamp as u64),
            app_hash: MerkleRoot { hash: app_hash },
            next_validators_hash: next_validators_hash
        }
    }

    #[test]
    fun test_decode_consensus() {
        let buf =
            x"0000000000000000000000000000000000000000000000001810cfdefbacb17df5631a5398a5443f5c858e3f8d4ffb2ddd5fa325d9f825572e1a0d302f7c9c092f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d";
        let _consensus = decode_consensus_state(buf);
    }

    fun encode_consensus_state(cs: &ConsensusState): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_uint<u64>(&mut buf, cs.timestamp);

        vector::append(&mut buf, cs.app_hash.hash);
        vector::append(&mut buf, cs.next_validators_hash);

        buf
    }

    struct PartialClientState has drop {
        chain_id: string::String,
        trusting_period: u64,
        max_clock_drift: u64,
        frozen_height: Height,
        latest_height: Height
    }

    fun encode_client_state(cs: &ClientState): vector<u8> {
        let buf = vector::empty();

        let partial = PartialClientState {
            chain_id: cs.chain_id,
            trusting_period: cs.trusting_period,
            max_clock_drift: cs.max_clock_drift,
            frozen_height: cs.frozen_height,
            latest_height: cs.latest_height
        };

        vector::append(&mut buf, bcs::to_bytes(&partial));
        vector::append(&mut buf, cs.contract_address);

        buf
    }

    fun decode_header(buf: vector<u8>): Header {
        let buf = bcs_utils::new(buf);
        peel_header(&mut buf)
    }

    fun peel_header(buf: &mut bcs_utils::BcsBuf): Header {
        let height = bcs_utils::peel_u64(buf);

        let time = Timestamp {
            seconds: bcs_utils::peel_u64(buf),
            nanos: bcs_utils::peel_u32(buf)
        };

        let signed_header = LightHeader {
            height,
            time,
            validators_hash: bcs_utils::peel_fixed_bytes(buf, 32),
            next_validators_hash: bcs_utils::peel_fixed_bytes(buf, 32),
            app_hash: bcs_utils::peel_fixed_bytes(buf, 32)
        };

        let trusted_height = height::decode_bcs(buf);

        let proof_bz = bcs_utils::peel_bytes(buf);
        let zero_knowledge_proof = groth16_verifier::parse_zkp(proof_bz);

        Header { signed_header, trusted_height, zero_knowledge_proof }
    }

    fun decode_misbehaviour(buf: vector<u8>): Misbehaviour {
        let buf = bcs_utils::new(buf);

        Misbehaviour {
            header_a: peel_header(&mut buf),
            header_b: peel_header(&mut buf)
        }
    }

    public(friend) fun light_header_as_input_hash(header: &LightHeader): vector<u8> {
        let inputs_hash = vector::empty();

        let height = bcs::to_bytes<u256>(&(header.height as u256));
        vector::reverse(&mut height);
        let seconds = bcs::to_bytes<u256>(&(header.time.seconds as u256));
        vector::reverse(&mut seconds);
        let nanos = bcs::to_bytes<u256>(&(header.time.nanos as u256));
        vector::reverse(&mut nanos);

        vector::append(&mut inputs_hash, height);
        vector::append(&mut inputs_hash, seconds);
        vector::append(&mut inputs_hash, nanos);
        vector::append(&mut inputs_hash, header.validators_hash);
        vector::append(&mut inputs_hash, header.next_validators_hash);
        vector::append(&mut inputs_hash, header.app_hash);

        inputs_hash
    }

    #[test]
    fun test_zkp_works() {
        let consensus =
            decode_consensus_state(
                x"00000000000000000000000000000000000000000000000018278e59cf08a67d3b1d403acd5f51abf9fc88024262e860935f014492dad5e7e69fb7212859a40912f1896da178c747a930cbc87d8e33b87c1d1fae377ab167523fa07d579fdb22"
            );
        let header =
            decode_header(
                x"6ee4480000000000432cbe67000000002114361112f1896da178c747a930cbc87d8e33b87c1d1fae377ab167523fa07d579fdb2212f1896da178c747a930cbc87d8e33b87c1d1fae377ab167523fa07d579fdb2221befefbd32576db7295e9ea22f75ca8111f3b983746c3cbaf9454dfddc7ef8e090000000000000058e4480000000000c0011b68961bb5f6bfb3b871f2ccaee38e9730d021121a356b9fd27ae2f1ccdca2a2930f540e8e5b8d162af95961be98d262c152902185bba758d76982e5fb39db112acea2d0916ead9ebc7cc0f9b242a55265804f65fff735f100556ec2a8a9f89a3bc27be5bb57ab0577c7d0a2f13fa221886e9309bf3b3ad8c59f319888ab4f971e752f83110ef218cad61d4b89a788c101cdc300b0ef7222ab8430d49f4d7c8a664182c3b959a176a52e9fa04a1fb20a0ee7bcf06066e19e171925c05bcc2c25"
            );

        let res =
            groth16_verifier::verify_zkp(
                &string::utf8(b"union-testnet-9"),
                &consensus.next_validators_hash,
                light_header_as_input_hash(&header.signed_header),
                &header.zero_knowledge_proof
            );

        assert!(res, 1);
    }
}
