module ibc::connection_end {
    use std::string::{String, utf8};
    use std::vector;
    use ibc::ethabi;

    struct ConnectionEnd has copy, store, drop, key {
        state: u64,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
    }

    // Getters
    public fun state(connection: &ConnectionEnd): u64 {
        connection.state
    }

    public fun client_id(connection: &ConnectionEnd): u32 {
        connection.client_id
    }

    public fun counterparty_client_id(connection: &ConnectionEnd): u32 {
        connection.counterparty_client_id
    }

    public fun counterparty_connection_id(connection: &ConnectionEnd): u32 {
        connection.counterparty_connection_id
    }

    // Setters
    public fun set_state(connection: &mut ConnectionEnd, new_state: u64) {
        connection.state = new_state;
    }

    public fun set_client_id(
        connection: &mut ConnectionEnd, new_client_id: u32
    ) {
        connection.client_id = new_client_id;
    }

    public fun set_counterparty_client_id(
        connection: &mut ConnectionEnd, new_id: u32
    ) {
        connection.counterparty_client_id = new_id;
    }

    public fun set_counterparty_connection_id(
        connection: &mut ConnectionEnd, new_id: u32
    ) {
        connection.counterparty_connection_id = new_id;
    }

    // Encode and decode functions (empty for now)
    public fun encode(connection: &ConnectionEnd): vector<u8> {
        let buf = vector::empty();

        ethabi::encode_uint(&mut buf, connection.state);
        ethabi::encode_uint(&mut buf, connection.client_id);
        ethabi::encode_uint(&mut buf, connection.counterparty_client_id);
        ethabi::encode_uint(&mut buf, connection.counterparty_connection_id);

        buf
    }

    // Constructor
    public fun new(
        state: u64,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
    ): ConnectionEnd {
        ConnectionEnd {
            state,
            client_id,
            counterparty_client_id,
            counterparty_connection_id,
        }
    }

    // Default function
    public fun default(): ConnectionEnd {
        new(0, 0, 0, 0)
    }

    #[test]
    fun test_encode_decode_connection() {
        /*
        ethabi given:
        0000000000000000000000000000000000000000000000000000000000000020
        0000000000000000000000000000000000000000000000000000000000000001 - state
        0000000000000000000000000000000000000000000000000000000000000064 - clientId
        00000000000000000000000000000000000000000000000000000000000000c8 - counterpartyClientId
        000000000000000000000000000000000000000000000000000000000000012c - counterpartyConnId
        00000000000000000000000000000000000000000000000000000000000000c0 -
        0000000000000000000000000000000000000000000000000000000000000100
        0000000000000000000000000000000000000000000000000000000000000008
        636f6d6574626c73000000000000000000000000000000000000000000000000
        0000000000000000000000000000000000000000000000000000000000000008
        6d6f76656d656e74000000000000000000000000000000000000000000000000
        */
        let buf =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c8000000000000000000000000000000000000000000000000000000000000012c00000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000008636f6d6574626c7300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000086d6f76656d656e74000000000000000000000000000000000000000000000000";
        let connection = ConnectionEnd {
            state: 1,
            client_id: 100,
            counterparty_client_id: 200,
            counterparty_connection_id: 300,
            client_type: utf8(b"cometbls"),
            counterparty_client_type: utf8(b"movement")
        };

        let encoded = encode(&connection);

        std::debug::print(&encoded);

        assert!(encoded == buf, 1);
    }
}
