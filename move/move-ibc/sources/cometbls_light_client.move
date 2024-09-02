module IBC::LightClient {
    use IBC::proto_utils;
    use std::option;
    use std::vector;
    use std::from_bcs;
    use std::bcs;
    use std::hash;
    use std::any::Any;
    use std::string::{Self, String, utf8};
    use std::bn254_algebra::{Fr, FormatFrMsb, FormatFrLsb, G1, FormatG1Uncompr, G2, Gt, FormatG2Compr};
    use std::crypto_algebra::{deserialize, serialize, zero, add, scalar_mul, multi_pairing, Element, eq};
    use std::aptos_hash;
    use IBC::height::{Self, Height};
    use aptos_std::smart_table::{Self, SmartTable};
    use std::object;
    use std::timestamp;
    use IBC::ics23;

    const ALPHA_G1: vector<u8> = x"99a818c167016f7f6d02d84005a5ed1f7c6c19c4ddf15733b67acc0129076709ff810d9d3374808069c1ea1e5d263a90cf8181b98b415805797176357acec708";
    const BETA_G2: vector<u8> = x"742884ea18a00ef31874d5fc5511b18fa9391dc69b971b898a2dbfc644033f15656dc92f1f94dc170026cd80212e5160d2539e7e8b40885d1d60b770d25f3599";
    const GAMMA_G2: vector<u8> = x"19b6719e42c42ed1df46fa08c870c5241a52913b65d9b43679e089c2e0bb1622cf3a489ca7927f4f81400a2ebd739a935bceb3224264eff8e248311ae96be7a0";
    const DELTA_G2: vector<u8> = x"eb044ddb951e9b28eda7da93aba341ef2c96a4d6182ca785a32018c9c803d405fcb9f04a31c988a2f5a64710ffafe101831d6147259b54e45d47e0d1184c5e29";
    const PEDERSEN_G: vector<u8> = x"5ae56dc014a8137712f4584658ba6f7e390cc39892f97e56ca859887d8d8f0138719bd9ffa2bba963951da2e08ba92ffc1049ba2f1fd7d7f03b02c13f8f67d25";
    const PEDERSEN_G_ROOT_SIGMA_NEG: vector<u8> = x"af5b4e30123a344339321dd621b5fdf9cd9870625928fa07235f011cdf04a1026863cae2f2b0c0ce457e81ad25a068fb1cb86026096be8e3f75c55a741e1bfaf";
    const GAMMA_ABC_G1: vector<vector<u8>> = vector[x"81925330941d53d8cec1c44210f6c882fee82c4ae97cb64b4f864327e54318270624cb7325a89fea7ad2cbde478a7ba38eca18bba1f024f672b1f89cc6423325" ,
    x"ca4b125d5e1a2ec0e22672434fbe9ca0e3ca15b0c20e16e9020ed6f471be0d0b0ce070b6a8b95f687014d83de09f9efe33caaf16aa92e5ec888376d3eb9a0b13"
    , x"c790c4a1918ab12e7e3c36005b2f5cbcf5408ced98033571760c7cf4d5939e02d9f1ee6a9c13b6ebbe2e11dab23f5600040fcb833bb5798faecf9d451005f12c"];
    const HMAC_O: vector<u8> = x"1F333139281E100F5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C5C";
    const HMAC_I: vector<u8> = x"75595B5342747A653636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636363636";
    const PRIME_R_MINUS_ONE: vector<u8> = x"000000f093f5e1439170b97948e833285d588181b64550b829a031e1724e6430";

    struct State has key, store {
        client_state: ClientState,
        consensus_states: SmartTable<height::Height, ConsensusState>
    }

    struct Proof has drop {
        a: Element<G1>,
        b: Element<G2>,
        c: Element<G1>,
    }

    struct ZKP has drop {
        proof: Proof,
        proof_commitment: Element<G1>,
        proof_commitment_pok: Element<G1>,
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

    fun verify_header(
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

        if (verify_zkp(
            &state.client_state.chain_id,
            &consensus_state.next_validators_hash,
            &header.signed_header,
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
        let header = decode_header(client_msg);

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
            encode_client_state(state.client_state),
            vector<vector<u8>>[encode_consensus_state(new_consensus_state)],
            vector<height::Height>[
                new_height
            ],
            0
        )
    }

    public fun verify_membership(
        _client_id: String,
        _height: height::Height,
        _proof: vector<u8>,
        _prefix: vector<u8>,
        _path: vector<u8>,
        _value: vector<u8>, 
    ): u64 {
        // let consensus_state = smart_table::borrow(&borrow_global<State>(get_client_address(&client_id)).consensus_states, height);
        // let proof = any::unpack<ics23::MembershipProof>(proof);

        // ics23::verify_membership(
        //     proof,
        //     consensus_state.app_hash.hash,
        //     prefix,
        //     path,
        //     value
        // )
        0
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

    fun get_client_address(client_id: &string::String): address {
        object::create_object_address(&@IBC, *string::bytes(client_id))
    }

   fun hmac_keccak(message: &vector<u8>): vector<u8> {
        let inner = HMAC_I;
        vector::append(&mut inner, *message);
        let outer = HMAC_O;
        vector::append(&mut outer, aptos_hash::keccak256(inner));

        aptos_hash::keccak256(outer)
    }

    fun hash_commitment(proof_commitment: &Element<G1>): u256 {
        let buffer = serialize<G1, FormatG1Uncompr>(proof_commitment);
        vector::reverse_slice(&mut buffer, 0, 32);
        vector::reverse_slice(&mut buffer, 32, 64);
        let hmac = hmac_keccak(&buffer);
        vector::reverse(&mut hmac);

        let prime_r_minus_one = from_bcs::to_u256(PRIME_R_MINUS_ONE);
        let hmac = from_bcs::to_u256(hmac);

        (hmac % prime_r_minus_one) + 1
    }
    
    fun verify_zkp(chain_id: &String, trusted_validators_hash: &vector<u8>, header: &LightHeader, zkp: &ZKP): bool {
        let inputs_hash: vector<u8> = vector::empty();
        let i = 0;
        while (i < 32 - string::length(chain_id)) {
            vector::push_back(&mut inputs_hash, 0);
            i = i + 1;
        };
        vector::append(&mut inputs_hash, *string::bytes(chain_id));

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
        vector::append(&mut inputs_hash, *trusted_validators_hash);

        let inputs_hash = hash::sha2_256(inputs_hash);

        let x = vector::borrow_mut(&mut inputs_hash, 0);
        *x = 0;

        let inputs_hash = std::option::extract(&mut deserialize<Fr, FormatFrMsb>(&inputs_hash));
        let commitment_hash = hash_commitment(&zkp.proof_commitment);
        let hmac = bcs::to_bytes(&commitment_hash);
        let commitment_hash = std::option::extract(&mut deserialize<Fr, FormatFrLsb>(&hmac));

        let alpha_g1 = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(&ALPHA_G1));

        let beta_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&BETA_G2));

        let gamma_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&GAMMA_G2));
        let delta_g2 = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&DELTA_G2));
        let pedersen_g = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&PEDERSEN_G));
        let pedersen_g_root_sigma_neg = std::option::extract(&mut deserialize<G2, FormatG2Compr>(&PEDERSEN_G_ROOT_SIGMA_NEG));

        let gamma_abc_1 = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(vector::borrow(&mut GAMMA_ABC_G1, 0)));
        let gamma_abc_2 = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(vector::borrow(&mut GAMMA_ABC_G1, 1)));
        let gamma_abc_3 = std::option::extract(&mut deserialize<G1, FormatG1Uncompr>(vector::borrow(&mut GAMMA_ABC_G1, 2)));

        // TODO(aeryz): why is this unused?
        let _res = serialize<Fr, FormatFrLsb>(&commitment_hash);

        let msm_inner = add(&add<G1>(&gamma_abc_1, &zkp.proof_commitment), &scalar_mul<G1, Fr>(&gamma_abc_2, &inputs_hash));
        let public_inputs_msm = add<G1>(&msm_inner, &scalar_mul<G1, Fr>(&gamma_abc_3, &commitment_hash));
        let res = serialize<G1, FormatG1Uncompr>(&public_inputs_msm);
        vector::reverse_slice(&mut res, 0, 32);
        vector::reverse_slice(&mut res, 32, 64);

        let res = multi_pairing<G1, G2, Gt>(
            &vector<Element<G1>>[
                zkp.proof.a, public_inputs_msm, zkp.proof.c, alpha_g1
            ],
            &vector<Element<G2>>[
                zkp.proof.b, gamma_g2, delta_g2, beta_g2
            ]
        );

        if (!eq<Gt>(&res, &zero<Gt>())) {
            return false
        };

        let res = multi_pairing<G1, G2, Gt>(
            &vector<Element<G1>>[
                zkp.proof_commitment, zkp.proof_commitment_pok
            ],
            &vector<Element<G2>>[
                pedersen_g, pedersen_g_root_sigma_neg
            ]
        );

        eq<Gt>(&res, &zero<Gt>())        
    }

    // #[test(ibc_signer = @IBC)]
    // fun test_create_client(ibc_signer: &signer) acquires State {
    //     let client_state = ClientState {
    //         chain_id: string::utf8(b"this-chain"),
    //         trusting_period: 0,
    //         unbonding_period: 0,
    //         max_clock_drift: 0,
    //         frozen_height: height::new(0, 0),
    //         latest_height: height::new(0, 1000),
    //     };

    //     let consensus_state = ConsensusState {  
    //         timestamp: 10000,
    //         app_hash: MerkleRoot {
    //             hash: vector<u8>[]
    //         },
    //         next_validators_hash: vector<u8>[]
    //     };

    //     assert!(create_client(ibc_signer, string::utf8(b"this_client"), std::any::pack<ClientState>(client_state), std::any::pack<ConsensusState>(consensus_state)) == 0, 1);

    //     let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client")));
    //     assert!(
    //         saved_state.client_state == client_state, 0
    //     );

    //     assert!(
    //         smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) == &consensus_state, 0
    //     );

    //     client_state.trusting_period = 2;
    //     consensus_state.timestamp = 20000;

    //     assert!(create_client(ibc_signer, string::utf8(b"this_client-2"), std::any::pack<ClientState>(client_state), std::any::pack<ConsensusState>(consensus_state)) == 0, 1);

    //     // new client don't mess with this client's storage
    //     let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client")));
    //     assert!(
    //         saved_state.client_state != client_state, 0
    //     );

    //     assert!(
    //         smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) != &consensus_state, 0
    //     );

    //     let saved_state = borrow_global<State>(get_client_address(&string::utf8(b"this_client-2")));
    //     assert!(
    //         saved_state.client_state == client_state, 0
    //     );

    //     assert!(
    //         smart_table::borrow<height::Height, ConsensusState>(&saved_state.consensus_states, client_state.latest_height) == &consensus_state, 0
    //     );
    // }

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
        encode_client_state(state.client_state)
    }

    public fun get_consensus_state(client_id: String, height: Height): vector<u8> acquires State {
        let state = borrow_global<State>(get_client_address(&client_id));
        let consensus_state = smart_table::borrow(&state.consensus_states, height);
        encode_consensus_state(*consensus_state)
    }

    fun default_client_state(): ClientState {
        ClientState {
            chain_id: utf8(b""),
            trusting_period: 0,
            unbonding_period: 0,
            max_clock_drift: 0,
            frozen_height: height::default(),
            latest_height: height::default(),
        }
    }

    fun decode_client_state(buf: vector<u8>): ClientState {
        let cursor = 0;
        let client_state = default_client_state();
        if (vector::is_empty(&buf)) {
            return client_state
        };
        while (cursor < vector::length(&buf)) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(&buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let n_read = if (tag == 1) {
                let (str, advance) = proto_utils::decode_string(wire_type, &buf, cursor);
                client_state.chain_id = option::extract(&mut str);
                advance
            } else if (tag == 2) {                
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                client_state.trusting_period = num;
                advance
            } else if (tag == 3) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                client_state.unbonding_period = num;
                advance
            } else if (tag == 4) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                client_state.max_clock_drift = num;
                advance
            } else if (tag == 5) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let (n_read, err) = height::decode_proto(&buf, cursor, len, &mut client_state.frozen_height);
                assert!(err == 0 && n_read == len, 1);
                len
            } else if (tag == 6) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let (n_read, err) = height::decode_proto(&buf, cursor, len, &mut client_state.latest_height);
                assert!(err == 0 && n_read == len, 1);
                len
            } else {
                abort 1
            };
            cursor = cursor + n_read;
        };

        client_state
    }

    fun encode_client_state(client_state: ClientState): vector<u8> {
        let buf = vector::empty();

        if (!string::is_empty(&client_state.chain_id)) {
            vector::append(&mut buf, proto_utils::encode_string(1, client_state.chain_id));
        };

        if (client_state.trusting_period != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(2, client_state.trusting_period));
        };

        if (client_state.unbonding_period != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(3, client_state.unbonding_period));
        };

        if (client_state.max_clock_drift!= 0) {
            vector::append(&mut buf, proto_utils::encode_u64(4, client_state.max_clock_drift));
        };

        let frozen_height = height::encode_proto(client_state.frozen_height);
        if (!vector::is_empty(&frozen_height)) {
            vector::append(&mut buf, proto_utils::encode_prefix(5, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&frozen_height)));
            vector::append(&mut buf, frozen_height);
        };

        let latest_height = height::encode_proto(client_state.latest_height);
        if (!vector::is_empty(&latest_height)) {
            vector::append(&mut buf, proto_utils::encode_prefix(6, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&latest_height)));
            vector::append(&mut buf, latest_height);
        };

        buf
    }

    fun default_consensus_state(): ConsensusState {
        ConsensusState {
            timestamp: 0,
            app_hash: MerkleRoot {
                hash: vector::empty(),
            },
            next_validators_hash: vector::empty(),
        }
    }

    fun decode_consensus_state(buf: vector<u8>): ConsensusState {
        let cursor = 0;
        let consensus_state = default_consensus_state();
        if (vector::is_empty(&buf)) {
            return consensus_state
        };
        while (cursor < vector::length(&buf)) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(&buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let n_read = if (tag == 1) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                consensus_state.timestamp = num;
                advance
            } else if (tag == 2) {                
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let n_read = decode_merkle_root(&buf, cursor, len, &mut consensus_state.app_hash);
                assert!(n_read == len, 1);
                len
            } else if (tag == 3) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, &buf, cursor);
                consensus_state.next_validators_hash = option::extract(&mut bytes);
                advance
            } else {
                abort 1
            };
            
            cursor = cursor + n_read;
        };

        consensus_state
    }

    fun encode_consensus_state(consensus_state: ConsensusState): vector<u8> {
        let buf = vector::empty();

        if (consensus_state.timestamp != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(1, consensus_state.timestamp));
        };

        let app_hash = encode_merkle_root(consensus_state.app_hash);
        if (!vector::is_empty(&app_hash)) {
            vector::append(&mut buf, proto_utils::encode_prefix(1, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&app_hash)));
            vector::append(&mut buf, app_hash);
        };

        if (!vector::is_empty(&consensus_state.next_validators_hash)) {
            vector::append(&mut buf, proto_utils::encode_bytes(3, consensus_state.next_validators_hash));
        };

        buf
    }

    fun decode_merkle_root(buf: &vector<u8>, cursor: u64, len: u64, merkle_root: &mut MerkleRoot): u64 {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, buf, cursor);
                merkle_root.hash = option::extract(&mut bytes);
                advance
            } else {
                abort 1
            };
            cursor = cursor + advance;
        };

        // nullable = false
        if (vector::is_empty(&merkle_root.hash)) {
            abort 1
        };

        cursor - first_pos
    }

    public fun encode_merkle_root(merkle_root: MerkleRoot): vector<u8> {
        let buf = vector::empty();

        // nullable = false
        vector::append(&mut buf, proto_utils::encode_bytes(1, merkle_root.hash));

        buf
    }

    public fun default_zkp(): ZKP {
        ZKP {
            proof: Proof {
                a: zero<G1>(),
                b: zero<G2>(),
                c: zero<G1>(),
            },
            proof_commitment: zero<G1>(),
            proof_commitment_pok: zero<G1>(),
        }
    }

    public fun default_header(): Header {
        Header {
            signed_header: default_light_header(),
            trusted_height: height::default(),
            zero_knowledge_proof: default_zkp(),
        }
    }

    fun decode_header(buf: vector<u8>): Header {
        let cursor = 0;
        let header = default_header();
        if (vector::is_empty(&buf)) {
            return header
        };
        while (cursor < vector::length(&buf)) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(&buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let n_read = if (tag == 1) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let n_read = decode_light_header(&buf, cursor, len, &mut header.signed_header);
                assert!(n_read == len, 1);
                len
            } else if (tag == 2) {                
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, &buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let (n_read, err) = height::decode_proto(&buf, cursor, len, &mut header.trusted_height);
                assert!(err == 0 && n_read == len, 1);
                len
            } else if (tag == 3) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, &buf, cursor);
                let _zkp = option::extract(&mut bytes);
                // TODO(aeryz): parse zkp here
                advance
            } else {
                abort 1
            };
            
            cursor = cursor + n_read;
        };

        header
    }


    public fun default_light_header(): LightHeader {
        LightHeader {
            height: 0,
            time: Timestamp {
                nanos: 0,
                seconds: 0,
            },
            validators_hash: vector::empty(),
            next_validators_hash: vector::empty(),
            app_hash: vector::empty(),
        }
    }

    public fun encode_light_header(header: LightHeader): vector<u8> {
        let buf = vector::empty();

        if (header.height != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(1, header.height));
        };

        let time = encode_timestamp(header.time);
        if (!vector::is_empty(&time)) {
            vector::append(&mut buf, proto_utils::encode_prefix(1, 2));
            vector::append(&mut buf, proto_utils::encode_varint(vector::length(&time)));
            vector::append(&mut buf, time);
        };

        if (!vector::is_empty(&header.validators_hash)) {
            vector::append(&mut buf, proto_utils::encode_bytes(3, header.validators_hash));
        };

        if (!vector::is_empty(&header.next_validators_hash)) {
            vector::append(&mut buf, proto_utils::encode_bytes(4, header.next_validators_hash));
        };

        if (!vector::is_empty(&header.app_hash)) {
            vector::append(&mut buf, proto_utils::encode_bytes(5, header.app_hash));
        };

        buf
    }
    
    public fun decode_light_header(buf: &vector<u8>, cursor: u64, len: u64, light_header: &mut LightHeader): u64 {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, buf, cursor);
                assert!(err == 0, 1);
                light_header.height = num;
                advance
            } else if (tag == 2) {
                let (len, advance, err) = proto_utils::decode_nested_len(wire_type, buf, cursor);
                assert!(err == 0, 1);
                cursor = cursor + advance;
                let n_read = decode_timestamp(buf, cursor, len, &mut light_header.time);
                assert!(n_read == len, 1);
                len
            } else if (tag == 3) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, buf, cursor);
                light_header.validators_hash = option::extract(&mut bytes);
                advance
            } else if (tag == 4) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, buf, cursor);
                light_header.validators_hash = option::extract(&mut bytes);
                advance
            } else if (tag == 5) {
                let (bytes, advance) = proto_utils::decode_bytes(wire_type, buf, cursor);
                light_header.validators_hash = option::extract(&mut bytes);
                advance
            }
            
            else {
                abort 1
            };
            cursor = cursor + advance;
        };

        cursor - first_pos
    }

    public fun encode_timestamp(timestamp: Timestamp): vector<u8> {
        let buf = vector::empty();

        if (timestamp.seconds != 0) {
            vector::append(&mut buf, proto_utils::encode_u64(1, timestamp.seconds));
        };

        if (timestamp.nanos != 0) {
            vector::append(&mut buf, proto_utils::encode_u32(2, timestamp.nanos));
        };

        buf
    }

    fun decode_timestamp(buf: &vector<u8>, cursor: u64, len: u64, timestamp: &mut Timestamp): u64 {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            assert!(err == 0, 1);
            cursor = cursor + advance;
            let advance = if (tag == 1) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, buf, cursor);
                assert!(err == 0, 1);
                timestamp.seconds = num;
                advance
            } else if (tag == 2) {
                let (num, advance, err) = proto_utils::decode_varint(wire_type, buf, cursor);
                assert!(err == 0, 1);
                timestamp.nanos = (num as u32);
                advance
            } else {
                abort 1
            };
            cursor = cursor + advance;
        };

        cursor - first_pos
    }
}
