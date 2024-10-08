module ibc::cometbls_light_client {
    use std::string::String;
    use sui::table::{Self, Table};
    use sui::object::{Self, UID};

    use ibc::height::{Self, Height};

    public struct Client has key, store {
        id: UID,
        client_state: vector<u8>,
        consensus_states: Table<u64, vector<u8>>,
    }

    public(package) fun create_client(
        client_id: String,
        client_state: vector<u8>,
        consensus_state: vector<u8>,
        ctx: &mut TxContext,
    ): Client {
        let mut consensus_states = table::new(ctx);
        consensus_states.add(0, consensus_state);
        Client {
            id: object::new(ctx),
            client_state,
            consensus_states,
        }
    }

    public(package) fun status(
        _client: &Client,
    ): u64 {
        0
    }

    public(package) fun latest_height(
        _client: &Client
    ): Height {
        height::new(0, 0)
    }

    public(package) fun verify_membership(
        _client: &Client,
        height: height::Height,
        proof: vector<u8>,
        prefix: vector<u8>,
        path: vector<u8>,
        value: vector<u8>, 
    ): u64 {
        0
    }
}
