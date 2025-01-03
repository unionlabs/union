module ibc::statelens_lc {
    use std::vector;

    public fun create_client(
        _ibc_signer: &signer,
        _client_id: u32,
        _client_state_bytes: vector<u8>,
        _consensus_state_bytes: vector<u8>
    ): (vector<u8>, vector<u8>) {
        (vector::empty(), vector::empty())
    }

    public fun latest_height(_client_id: u32): u64 {
        0
    }

    public fun update_client(
        _client_id: u32, _client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) {
        (vector::empty(), vector::empty(), vector::empty())
    }

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

    public fun get_client_state(_client_id: u32): vector<u8> {
        vector::empty()
    }

    public fun get_consensus_state(_client_id: u32, _height: u64): vector<u8> {
        vector::empty()
    }

    public fun check_for_misbehaviour(
        _client_id: u32, _header: vector<u8>
    ): bool {
        false
    }
}
