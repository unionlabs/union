module ibc_api::ibc_api {
    use std::string::String;

    public entry fun create_client(
        _client_type: String,
        _client_state: vector<u8>,
        _consensus_state: vector<u8>,
    ) {}

    public entry fun connection_open_init(
        _client_id: String,
        _version_identifier: String,
        _version_features: vector<String>,
        _counterparty_client_id: String,
        _counterparty_connection_id: String,
        _counterparty_prefix: vector<u8>,
        _delay_period: u64,
    ) {}
    
    public entry fun connection_open_try(
        _counterparty_client_id: String,
        _counterparty_connection_id: String,
        _counterparty_prefix: vector<u8>,
        _delay_period: u64,
        _client_id: String,
        _client_state_bytes: vector<u8>,
        _counterparty_version_identifiers: vector<String>,
        _counterparty_version_features: vector<vector<String>>,
        _proof_init: vector<u8>,
        _proof_client: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun connection_open_ack(
        _connection_id: String,
        _client_state_bytes: vector<u8>,
        _version_identifier: String,
        _version_features: vector<String>,
        _proof_try: vector<u8>,
        _proof_client: vector<u8>,
        _counterparty_connection_id: String,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun connection_open_confirm(
        _connection_id: String,
        _proof_ack: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun channel_open_init(
        _connection_hops: vector<String>,
        _ordering: u8,
        _counterparty_port_id: String,
        _counterparty_channel_id: String,
        _version: String,
    ) {}

    public entry fun channel_open_try(
        _connection_hops: vector<String>,
        _ordering: u8,
        _counterparty_port_id: String,
        _counterparty_channel_id: String,
        _counterparty_version: String,
        _version: String,
        _proof_init: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun channel_open_ack(
        _channel_id: String,
        _counterparty_channel_id: String,
        _counterparty_version: String,
        _proof_try: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun channel_open_confirm(
        _channel_id: String,
        _proof_ack: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun recv_packet(
        _packet_sequence: u64,
        _packet_source_port: String,
        _packet_source_channel: String,
        _packet_destination_port: String,
        _packet_destination_channel: String,
        _packet_data: vector<u8>,
        _packet_timeout_revision_num: u64,
        _packet_timeout_revision_height: u64,
        _packet_timeout_timestamp: u64,
        _proof: vector<u8>,
        _proof_height_revision_num: u64,
        _proof_height_revision_height: u64,
    ) {}

    public entry fun update_client(_client_id: String, _client_message: vector<u8>) {}
}
