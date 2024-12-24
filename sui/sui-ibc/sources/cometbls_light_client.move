module ibc::light_client {
    use std::string::{Self, String};
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};
    use sui::bcs;
    use ibc::ethabi;
    use ibc::height::{Self, Height};

    public struct Client has key, store {
        id: UID,
        client_state: vector<u8>,
        consensus_states: Table<u64, vector<u8>>,
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
    ): Client {
        let mut consensus_states = table::new(ctx);
        consensus_states.add(0, consensus_state_bytes);
        Client {
            id: object::new(ctx),
            client_state: client_state_bytes,
            consensus_states: consensus_states
        }
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
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) {

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
            vector[encode_consensus_state(&consensus_state)],
            vector[0])
        // (vector::empty(), vector::empty(), vector::empty())
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
}