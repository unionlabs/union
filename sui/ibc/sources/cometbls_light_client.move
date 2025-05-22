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
    use std::string::{Self, String};
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use sui::bcs;
    use ibc::ethabi;
    use ibc::height::{Self, Height};

    const E_HEIGHT_NOT_FOUND_ON_CONSENSUS_STATE: u64 = 0x99999;

    public struct Client has key, store {
        id: UID,
        client_state: ClientState,
        consensus_states: Table<u64, ConsensusState>,
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
    ): (Client, vector<u8>, vector<u8>, String) {
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
        client_state.chain_id)
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

    public(package) fun update_client(
        client: &Client, client_msg: vector<u8>
    ): (vector<u8>, vector<u8>, u64) {

        let consensus_state = ConsensusState{
            timestamp: 0,
            app_hash: MerkleRoot{hash: vector::empty()},
            next_validators_hash: vector::empty()
        };
        let client_state = ClientState {
            chain_id: string::utf8(b"this-chain"),
            trusting_period: 0,
            max_clock_drift: 0,
            frozen_height: height::default(),
            latest_height: height::new(0, 1000),
            contract_address: vector::empty()
        };

        (encode_client_state(&client_state),
            encode_consensus_state(&consensus_state),
            0)
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
}
