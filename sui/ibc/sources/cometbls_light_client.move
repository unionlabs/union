// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
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

module ibc::light_client {
    use std::option::{Self, Option};
    use std::string::{Self, String};
    use sui::table::{Self, Table};
    use sui::clock;
    use sui::object::{Self, UID};
    use sui::bcs::{Self, BCS};
    use ibc::ethabi;
    use ibc::height::{Self, Height};
    use ibc::create_lens_client_event::CreateLensClientEvent;
    use ibc::ics23;
    use ibc::groth16_verifier::{Self, ZKP};

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

    const E_HEIGHT_NOT_FOUND_ON_CONSENSUS_STATE: u64 = 0x99999;

    public struct Client has key, store {
        id: UID,
        client_state: ClientState,
        consensus_states: Table<u64, ConsensusState>,
    }

    public struct Timestamp has drop, copy {
        seconds: u64,
        nanos: u32
    }

    public struct LightHeader has drop, copy {
        height: u64,
        time: Timestamp,
        validators_hash: vector<u8>,
        next_validators_hash: vector<u8>,
        app_hash: vector<u8>
    }

    public struct Header has drop {
        signed_header: LightHeader,
        trusted_height: Height,
        zero_knowledge_proof: ZKP
    }

    public struct ConsensusState has copy, drop, store {
        timestamp: u64,
        app_hash: MerkleRoot,
        next_validators_hash: vector<u8>
    }

    public struct MerkleRoot has copy, drop, store {
        hash: vector<u8>
    }

    public struct ClientState has copy, drop, store {
        chain_id: string::String,
        trusting_period: u64,
        max_clock_drift: u64,
        frozen_height: Height,
        latest_height: Height,
        contract_address: vector<u8>
    }

    public struct PartialClientState has drop {
        chain_id: string::String,
        trusting_period: u64,
        max_clock_drift: u64,
        frozen_height: Height,
        latest_height: Height
    }


    public(package) fun create_client(
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>,
        ctx: &mut TxContext,
    ): (Client, vector<u8>, vector<u8>, String, Option<CreateLensClientEvent>) {
        let mut consensus_states = table::new(ctx);
        let client_state = decode_client_state(client_state_bytes);
        consensus_states.add(0, decode_consensus_state(consensus_state_bytes));
        (Client {
            id: object::new(ctx),
            client_state: decode_client_state(client_state_bytes),
            consensus_states: consensus_states
        },
        client_state_bytes,
        consensus_state_bytes,
        client_state.chain_id,
        option::none())
    }

    public(package) fun status(
        _client: &Client,
    ): u64 {
        0
    }

    public(package) fun check_for_misbehaviour(client: &Client, header: vector<u8>): bool {
        false
    }

    public(package) fun report_misbehaviour(
        client: &Client, misbehaviour: vector<u8>
    ){

    }

    public(package) fun get_timestamp_at_height(client: &Client, height: u64): u64  {
        0
    }

    public(package) fun verify_non_membership(
        _client: &Client,
        _height: u64,
        _proof: vector<u8>,
        _path: vector<u8>
    ): u64 {
        0
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let mut index = 0;
        let timestamp = ethabi::decode_uint(&buf, &mut index);
        let app_hash = ethabi::vector_slice(&buf, 32, 64);
        let next_validators_hash = ethabi::vector_slice(&buf, 64, 96);

        ConsensusState {
            timestamp: (timestamp as u64),
            app_hash: MerkleRoot { hash: app_hash },
            next_validators_hash: next_validators_hash
        }
    }

    fun decode_client_state(buf: vector<u8>): ClientState {
        let mut buf = bcs::new(buf);

        let chain_id = string::utf8(buf.peel_vec_u8());
        let trusting_period = buf.peel_u64();
        let max_clock_drift = buf.peel_u64();
        let frozen_height = height::decode_bcs(&mut buf); // TODO: Not sure if its correc;
        let latest_height = height::decode_bcs(&mut buf);
        let contract_address = buf.into_remainder_bytes();

        ClientState {
            chain_id,
            trusting_period,
            max_clock_drift,
            frozen_height,
            latest_height,
            contract_address
        }
    }

    #[test]
    fun test_decode_consensus() {
        let buf =
            x"0000000000000000000000000000000000000000000000001810cfdefbacb17df5631a5398a5443f5c858e3f8d4ffb2ddd5fa325d9f825572e1a0d302f7c9c092f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d";
        let consensus = decode_consensus_state(buf);
    }

    fun encode_consensus_state(cs: &ConsensusState): vector<u8> {
        let mut buf = vector::empty();

        ethabi::encode_uint<u64>(&mut buf, cs.timestamp);

        vector::append(&mut buf, cs.app_hash.hash);
        vector::append(&mut buf, cs.next_validators_hash);

        buf
    }


    fun encode_client_state(cs: &ClientState): vector<u8> {
        let mut buf = vector::empty();

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

    public(package) fun verify_header(
        client: &Client, clock: &clock::Clock, header: &Header, consensus_state: &ConsensusState
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

        let current_time = clock::timestamp_ms(clock) * 1_000_000;
        assert!(
            untrusted_timestamp < current_time + client.client_state.trusting_period,
            E_HEADER_EXCEEDED_TRUSTING_PERIOD
        );

        assert!(
            untrusted_timestamp < current_time + client.client_state.max_clock_drift,
            E_HEADER_EXCEEDED_MAX_CLOCK_DRIFT
        );

        if (untrusted_height_number == trusted_height_number + 1) {
            assert!(
                header.signed_header.validators_hash
                    == consensus_state.next_validators_hash,
                E_VALIDATORS_HASH_MISMATCH
            );
        };

        // assert!(
        //     groth16_verifier::verify_zkp(
        //         &state.client_state.chain_id,
        //         &consensus_state.next_validators_hash,
        //         light_header_as_input_hash(&header.signed_header),
        //         &header.zero_knowledge_proof
        //     ),
        //     E_INVALID_ZKP
        // );
    }


    public(package) fun update_client(
        client: &mut Client, clock: &clock::Clock, client_msg: vector<u8>
    ): (vector<u8>, vector<u8>, u64) {
        // TODO(aeryz): handle consensus state exist case
        let header = decode_header(client_msg);

        assert!(client.client_state.frozen_height.is_zero(), E_FROZEN_CLIENT);

        let consensus_state = client.consensus_states.borrow(height::get_revision_height(&header.trusted_height));

        client.verify_header(clock, &header, consensus_state);

        let untrusted_height_number = header.signed_header.height;
        let untrusted_timestamp =
            header.signed_header.time.seconds * 1_000_000_000
                + (header.signed_header.time.nanos as u64);

        if (untrusted_height_number
            > height::get_revision_height(&client.client_state.latest_height)) {
            client.client_state.latest_height = height::new(0, untrusted_height_number);
        };

        let new_height = height::get_revision_height(&client.client_state.latest_height);

        let new_consensus_state = ConsensusState {
            timestamp: untrusted_timestamp,
            app_hash: MerkleRoot { hash: header.signed_header.app_hash },
            next_validators_hash: header.signed_header.next_validators_hash
        };

        client.consensus_states.add(new_height, new_consensus_state);

        (
            encode_client_state(&client.client_state),
            encode_consensus_state(&new_consensus_state),
            new_height
        )

        // assert!(
        //     groth16_verifier::verify_zkp(
        //         &state.client_state.chain_id,
        //         &consensus_state.next_validators_hash,
        //         light_header_as_input_hash(&header.signed_header),
        //         &header.zero_knowledge_proof
        //     ),
        //     E_INVALID_ZKP
        // );

    }

    public(package) fun latest_height(
        client: &Client
    ): u64 {
        0
    }

    public(package) fun verify_membership(
        client: &Client,
        height: u64,
        proof: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ): u64 {
        let consensus_state = client.consensus_states.borrow(height);

        let mut path = vector<u8>[0x03];
        path.append(client.client_state.contract_address);
        path.append(key);

        ics23::verify_membership(
            ics23::decode_membership_proof(proof),
            consensus_state.app_hash.hash,
            b"wasm", // HARDCODED PREFIX
            path,
            value
        );

        0
    }

    public(package) fun get_client_state(
        client: &Client,
    ): vector<u8> {        
        encode_client_state(&client.client_state)
    }

    public(package) fun get_consensus_state(
        client: &Client,
        height: u64,
    ): vector<u8> {
        if (!client.consensus_states.contains(height)) {
            abort E_HEIGHT_NOT_FOUND_ON_CONSENSUS_STATE
        };
        let consensus_state = client.consensus_states.borrow(height);
        encode_consensus_state(consensus_state)
    }

    fun decode_header(buf: vector<u8>): Header {
        let mut buf = bcs::new(buf);
        peel_header(&mut buf)
    }

    fun peel_header(buf: &mut BCS): Header {
        let height = buf.peel_u64();

        let time = Timestamp {
            seconds: buf.peel_u64(),
            nanos: buf.peel_u32()
        };

        let signed_header = LightHeader {
            height,
            time,
            validators_hash: buf.peel_address().to_bytes(),
            next_validators_hash: buf.peel_address().to_bytes(),
            app_hash: buf.peel_address().to_bytes()
        };

        let trusted_height = height::decode_bcs(buf);

        let proof_bz = buf.peel_vec_u8();
        let zero_knowledge_proof = groth16_verifier::parse_zkp(proof_bz);

        Header { signed_header, trusted_height, zero_knowledge_proof }
    }
}
