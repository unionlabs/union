module IBC::LightClient {
    use std::vector;
    use std::from_bcs;
    use std::bcs;
    use std::string::{Self, String};
    use IBC::height::{Self, Height};
    use aptos_std::smart_table::{Self, SmartTable};
    use std::object;
    use std::timestamp;
    use IBC::ics23;
    use IBC::bcs_utils;
    use IBC::groth16_verifier::{Self, ZKP};

    struct State has key, store {
        client_state: ClientState,
        consensus_states: SmartTable<height::Height, ConsensusState>
    }

    struct Timestamp has drop, copy {
        seconds: u64,
        nanos: u32,
    }

    struct LightHeader has drop {
        height: u64,
        time: Timestamp,
        validators_hash: vector<u8>,
        next_validators_hash: vector<u8>,
        app_hash: vector<u8>,
    }

    struct Header has drop {
        signed_header: LightHeader,
        trusted_height: height::Height,
        zero_knowledge_proof: ZKP,
    }

    struct ClientState has copy, drop, store {
        chain_id: string::String,
        trusting_period: u64,
        unbonding_period: u64,
        max_clock_drift: u64,
        frozen_height: height::Height,
        latest_height: height::Height,
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
    public fun create_client(
        ibc_signer: &signer,
        client_id: String, 
        client_state_bytes: vector<u8>, 
        consensus_state_bytes: vector<u8>,
    ): (u64, vector<u8>, vector<u8>) {
        let client_state = decode_client_state(client_state_bytes);
        let consensus_state = decode_consensus_state(consensus_state_bytes);
        
        if (height::get_revision_height(&client_state.latest_height) == 0 || consensus_state.timestamp == 0) {
            return (1, vector::empty(), vector::empty())
        };

        if (string::length(&client_state.chain_id) > 31) {
            return (1, vector::empty(), vector::empty())
        };

        let consensus_states = smart_table::new<height::Height, ConsensusState>();
        smart_table::upsert<height::Height, ConsensusState>(&mut consensus_states, client_state.latest_height, consensus_state);

        let state = State {
            client_state: client_state,
            consensus_states: consensus_states
        };

        let store_constructor = object::create_named_object(ibc_signer, *string::bytes(&client_id));
        let client_signer = object::generate_signer(&store_constructor);

        move_to(&client_signer, state);
        
        (0, client_state_bytes, consensus_state_bytes)
    }

    public fun latest_height(
        client_id: String
    ): height::Height acquires State {
        // Return error code, 0 for success
        let state = borrow_global<State>(get_client_address(&client_id));
        state.client_state.latest_height
    }

    public fun verify_header(
        header: &Header,
        state: &State,
        consensus_state: &ConsensusState
    ): u64 {
        if (consensus_state.timestamp == 0) {
            return 1
        };

        let untrusted_height_number = header.signed_header.height;
        let trusted_height_number = height::get_revision_height(&header.trusted_height);

        if (untrusted_height_number <= trusted_height_number) {
            return 1
        };

        let trusted_timestamp = consensus_state.timestamp;
        let untrusted_timestamp = header.signed_header.time.seconds * 1_000_000_000 + (header.signed_header.time.nanos as u64);
        if (untrusted_timestamp <= trusted_timestamp) {
            return 1  
        };

        let current_time = timestamp::now_seconds() * 1_000_000_000;
        if (untrusted_timestamp> (current_time + state.client_state.trusting_period)) {
            return 1  
        };

        if (untrusted_timestamp >= current_time + state.client_state.max_clock_drift) {
            return 1  
        };

        if (untrusted_height_number == trusted_height_number + 1) {
            if (header.signed_header.validators_hash != consensus_state.next_validators_hash) {
                return 1
            };
        };

        if (groth16_verifier::verify_zkp(
            &state.client_state.chain_id,
            &consensus_state.next_validators_hash,
            light_header_as_input_hash(&header.signed_header),
            &header.zero_knowledge_proof,
        )) {
            return 1
        };

        0
    }

    public fun update_client(
        client_id: String,
        client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<height::Height>, u64) acquires State {
        let header = from_bcs::from_bytes<Header>(client_msg);

        let state = borrow_global_mut<State>(get_client_address(&client_id));

        if (height::is_zero(&state.client_state.frozen_height)) {
            return (vector::empty(), vector::empty(), vector::empty(), 1)
        };

        let consensus_state = smart_table::borrow<height::Height, ConsensusState>(&state.consensus_states, header.trusted_height);

        let err = verify_header(&header, state, consensus_state);
        if (err != 0) {
            return (vector::empty(), vector::empty(), vector::empty(), err)
        };

        let untrusted_height_number = header.signed_header.height;
        let untrusted_timestamp = header.signed_header.time.seconds * 1_000_000_000 + (header.signed_header.time.nanos as u64);

        if (untrusted_height_number > height::get_revision_height(&state.client_state.latest_height)) {
            height::set_revision_height(&mut state.client_state.latest_height, untrusted_height_number);
        };

        let new_height = height::new(height::get_revision_number(&state.client_state.latest_height), untrusted_height_number);

        let new_consensus_state = ConsensusState {
            timestamp: untrusted_timestamp,
            app_hash: MerkleRoot {
                hash: header.signed_header.app_hash,
            },
            next_validators_hash: header.signed_header.next_validators_hash
        };

        smart_table::upsert<height::Height, ConsensusState>(&mut state.consensus_states, new_height, new_consensus_state);

        (
            bcs::to_bytes(&state.client_state),
            vector<vector<u8>>[bcs::to_bytes(&new_consensus_state)],
            vector<height::Height>[
                new_height
            ],
            0
        )
    }

    public fun verify_membership(
        client_id: String,
        height: height::Height,
        proof: vector<u8>,
        prefix: vector<u8>,
        path: vector<u8>,
        value: vector<u8>, 
    ): u64 acquires State {
        let consensus_state = smart_table::borrow(&borrow_global<State>(get_client_address(&client_id)).consensus_states, height);

        ics23::verify_membership(
            from_bcs::from_bytes<ics23::MembershipProof>(proof),
            consensus_state.app_hash.hash,
            prefix,
            path,
            value
        )
    }

    public fun verify_non_membership(
        _client_id: String,
        _height: height::Height,
        _proof: vector<u8>,
        _prefix: vector<u8>,
        _path: vector<u8>,
    ): u64 {
        0
    }

    public fun status(
        _client_id: &String,
    ): u64 {
        // TODO(aeryz): fetch these status from proper exported consts
        0
    }

    fun get_client_address(client_id: &string::String): address {
        object::create_object_address(&@IBC, *string::bytes(client_id))
    }

    public fun new_client_state(
        chain_id: string::String,
        trusting_period: u64,
        unbonding_period: u64,
        max_clock_drift: u64,
        frozen_height: height::Height,
        latest_height: height::Height,
    ): ClientState {
        ClientState {            
            chain_id: chain_id,
            trusting_period: trusting_period,
            unbonding_period: unbonding_period,
            max_clock_drift: max_clock_drift,
            frozen_height: frozen_height,
            latest_height: latest_height,
        }
    }

    public fun new_consensus_state(
        timestamp: u64,
        app_hash: MerkleRoot,
        next_validators_hash: vector<u8>        
    ): ConsensusState {
        ConsensusState {
            timestamp: timestamp,
            app_hash: app_hash,
            next_validators_hash: next_validators_hash,
        }
    }


    public fun new_merkle_root(
        hash: vector<u8>    
    ): MerkleRoot {
        MerkleRoot {
            hash: hash
        }
    }


    public fun get_timestamp_at_height(
        _client_id: String,
        _height: height::Height
    ): u64 {
        1
    }

    public fun get_client_state(client_id: String): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(&client_id));
        bcs::to_bytes(&state.client_state)
    }

    public fun get_consensus_state(client_id: String, height: Height): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(&client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        bcs::to_bytes(consensus_state)
    }

    fun decode_client_state(buf: vector<u8>): ClientState {
        let buf = bcs_utils::new(buf);

        ClientState {
            chain_id: bcs_utils::peel_string(&mut buf),
            trusting_period: bcs_utils::peel_u64(&mut buf),
            unbonding_period: bcs_utils::peel_u64(&mut buf),
            max_clock_drift: bcs_utils::peel_u64(&mut buf),
            frozen_height: height::decode_bcs(&mut buf),
            latest_height: height::decode_bcs(&mut buf),
        }
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let buf = bcs_utils::new(buf);

        ConsensusState {
            timestamp: bcs_utils::peel_u64(&mut buf),
            app_hash: MerkleRoot {
                hash: bcs_utils::peel_bytes(&mut buf),
            },
            next_validators_hash: bcs_utils::peel_bytes(&mut buf),
        }
    }

    fun decode_header(buf: vector<u8>): Header {
        let buf = bcs_utils::new(buf);

        Header {
            signed_header: LightHeader {
                height: bcs_utils::peel_u64(&mut buf),
                time: Timestamp {
                    seconds: bcs_utils::peel_u64(&mut buf),
                    nanos: bcs_utils::peel_u32(&mut buf),
                },
                validators_hash: bcs_utils::peel_bytes(&mut buf),
                next_validators_hash: bcs_utils::peel_bytes(&mut buf),
                app_hash: bcs_utils::peel_bytes(&mut buf),
            },
            trusted_height: height::decode_bcs(&mut buf),
            zero_knowledge_proof: groth16_verifier::parse_zkp(bcs_utils::peel_bytes(&mut buf)),
        }
    }

    public fun light_header_as_input_hash(header: &LightHeader): vector<u8> {        
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
    fun parse_client_state() {
        let client_state = ClientState {
            chain_id: string::utf8(b"this-chain"),
            trusting_period: 9999999,
            unbonding_period: 12367,
            max_clock_drift: 0,
            frozen_height: height::new(11, 1273),
            latest_height: height::new(127638, 1000),
        };

        let cs = decode_client_state(bcs::to_bytes(&client_state));
        std::debug::print(&cs);
    }
    
    #[test]
    fun test_parse_zkp() {
        let zkp = x"1c911d332bca4aa85d3cea5099370b8f188326d3929436d809d5532bc24165089272d9494a6d75ae2389e07d5b6bab46d5ca923cebeb5c46e4d59233afc41115da87c1b5b63aefcc64580be04db609757dafc70c18302756c45c010bb18a1a2777cebaaa757fb71ced5efa731261a4da8dc3f1755e248927ebafdcde8030171559b4af7d1e2f29028c42ece0c7a65e2a814c536138e08f701727b12139b3ed06cc4013a258a88e083562242434d2d9236bb870503c9bc4294ef989da2462b6a9";

        parse_zkp(zkp);
    }

    #[test(ibc_signer = @IBC)]
    fun test_create_client(ibc_signer: &signer) acquires State {
        let client_state = ClientState {
            chain_id: string::utf8(b"this-chain"),
            trusting_period: 0,
            unbonding_period: 0,
            max_clock_drift: 0,
            frozen_height: height::new(0, 0),
            latest_height: height::new(0, 1000),
        };

        let consensus_state = ConsensusState {  
            timestamp: 10000,
            app_hash: MerkleRoot {
                hash: vector<u8>[]
            },
            next_validators_hash: vector<u8>[]
        };

        let (err, cs, cons) = create_client(ibc_signer, string::utf8(b"this_client"), bcs::to_bytes(&client_state), bcs::to_bytes(&consensus_state));
        assert!(err == 0 && cs == bcs::to_bytes(&client_state) && cons == bcs::to_bytes(&consensus_state), 1);


        let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client")));
        assert!(
            saved_state.client_state == client_state, 0
        );

        assert!(
            smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) == &consensus_state, 0
        );

        client_state.trusting_period = 2;
        consensus_state.timestamp = 20000;

        let (err, cs, cons) = create_client(ibc_signer, string::utf8(b"this_client-2"), bcs::to_bytes(&client_state), bcs::to_bytes(&consensus_state));
        assert!(err == 0 && cs == bcs::to_bytes(&client_state) && cons == bcs::to_bytes(&consensus_state), 1);

        // new client don't mess with this client's storage
        let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client")));
        assert!(
            saved_state.client_state != client_state, 0
        );

        assert!(
            smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) != &consensus_state, 0
        );

        let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client-2")));
        assert!(
            saved_state.client_state == client_state, 0
        );

        assert!(
            smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) == &consensus_state, 0
        );
    }
}
