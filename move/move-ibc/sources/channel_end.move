module ibc::channel {
    use std::option::{Self, Option};
    use std::vector;
    use std::string::{Self, String};
    use ibc::ethabi;

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    struct Channel has copy, store, drop, key {
        state: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String
    }

    // Getters
    public fun state(channel: &Channel): u8 {
        channel.state
    }

    public fun counterparty_port_id(channel: &Channel): &vector<u8> {
        &channel.counterparty_port_id
    }

    public fun connection_id(channel: &Channel): u32 {
        channel.connection_id
    }

    public fun counterparty_channel_id(channel: &Channel): u32 {
        channel.counterparty_channel_id
    }

    public fun version(channel: &Channel): &String {
        &channel.version
    }

    // Setters
    public fun set_state(channel: &mut Channel, new_state: u8) {
        channel.state = new_state;
    }

    public fun set_counterparty_port_id(
        channel: &mut Channel, new_counterparty_port_id: vector<u8>
    ) {
        channel.counterparty_port_id = new_counterparty_port_id;
    }

    public fun set_connection_id(
        channel: &mut Channel, new_connection_id: u32
    ) {
        channel.connection_id = new_connection_id;
    }

    public fun set_counterparty_channel_id(
        channel: &mut Channel, new_id: u32
    ) {
        channel.counterparty_channel_id = new_id;
    }

    public fun set_version(channel: &mut Channel, new_version: String) {
        channel.version = new_version;
    }

    // Encode and decode functions (empty for now)
    public fun encode(channel: &Channel): vector<u8> {
        // TODO(aeryz): fix this
        let buf = vector::empty<u8>();

        // offset of version ????
        ethabi::encode_uint<u64>(&mut buf, 32 * 1);

        ethabi::encode_uint<u8>(&mut buf, channel.state);
        ethabi::encode_uint<u32>(&mut buf, channel.connection_id);
        ethabi::encode_uint<u32>(&mut buf, channel.counterparty_channel_id);

        // offset of counterparty_port_id?
        ethabi::encode_uint<u64>(&mut buf, 32 * 5);

        // no idea what is this
        ethabi::encode_uint<u64>(&mut buf, 32 * 11);

        ethabi::encode_vector<u8>(
            &mut buf,
            &channel.counterparty_port_id,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        // something is wrong in the encode_string i don't know what
        // ethabi::encode_string(&mut buf, &channel.version);

        // let i = 32 - string::length(&channel.version);
        
        // vector::append(&mut buf, *string::bytes(&channel.version));
        // while (i > 0) {
        //     vector::push_back(&mut buf, 0);
        //     i = i - 1;
        // };

        buf
    }

    // FIXME(aeryz):
    public fun decode(buf: vector<u8>): Option<Channel> {
        let index = 0;

        let state = (ethabi::decode_uint(&buf, &mut index) as u8);
        let connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);
        let counterparty_connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);

        let i = index;
        std::debug::print(&i);
        while (i < index + 32) {
            let char = *vector::borrow(&buf, i);

            if (char == 0) { break };

            i = i + 1;
        };
        let counterparty_port_id = vector::slice(&buf, index, i);


        let i = index;
        std::debug::print(&i);
        while (i < index + 32) {
            let char = *vector::borrow(&buf, i);

            if (char == 0) { break };

            i = i + 1;
        };
        let version = string::utf8(vector::slice(&buf, index, i));


        option::some(
            new(
                state,
                connection_id,
                counterparty_connection_id,
                counterparty_port_id,
                version
            )
        )
    }

    // Constructor
    public fun new(
        state: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String
    ): Channel {
        Channel {
            state,
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version
        }
    }

    // Default function
    public fun default(): Channel {
        new(0, 0, 0, vector::empty(), string::utf8(b""))
    }

    #[test]
    public fun test_encode_decode_channel() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f0000000000000000000000000000000000000000000000000000000000000005312e302e30000000000000000000000000000000000000000000000000000000";

        let channel = new(
            1,
            1,
            100,
            b"hello",
            string::utf8(b"1.0.0")
        );

        let encoded = encode(&channel);
        std::debug::print(&encoded);
        assert!(buf == encoded, 1);
        assert!(decode(encoded) == option::some(channel), 1);
    }
}