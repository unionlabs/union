module ibc::light_client {
    use std::string::String;
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};


    public struct Client has key, store {
        id: UID,
        client_state: vector<u8>,
        consensus_states: Table<u64, vector<u8>>,
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

    public(package) fun update_client(
        client: &Client, client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) {
        (vector::empty(), vector::empty(), vector::empty())
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
        path: vector<u8>,
        value: vector<u8>
    ): u64 {
        0
    }
}