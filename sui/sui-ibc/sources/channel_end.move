module ibc::channel {
    use std::option::{Self, Option};
    use std::vector;
    use ibc::ethabi;

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    public struct Channel has copy, store, drop {
        state: u8,
        ordering: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        version: vector<u8>
    }

    // Getters
    public fun state(channel: &Channel): u8 {
        channel.state
    }

    public fun ordering(channel: &Channel): u8 {
        channel.ordering
    }

    public fun connection_id(channel: &Channel): u32 {
        channel.connection_id
    }

    public fun counterparty_channel_id(channel: &Channel): u32 {
        channel.counterparty_channel_id
    }

    public fun version(channel: &Channel): &vector<u8> {
        &channel.version
    }

    // Setters
    public fun set_state(channel: &mut Channel, new_state: u8) {
        channel.state = new_state;
    }

    public fun set_ordering(channel: &mut Channel, new_ordering: u8) {
        channel.ordering = new_ordering;
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

    public fun set_version(channel: &mut Channel, new_version: vector<u8>) {
        channel.version = new_version;
    }

    // Encode and decode functions (empty for now)
    public fun encode(channel: &Channel): vector<u8> {
        let mut buf = vector::empty<u8>();

        ethabi::encode_uint<u8>(&mut buf, channel.state);
        ethabi::encode_uint<u8>(&mut buf, channel.ordering);
        ethabi::encode_uint<u32>(&mut buf, channel.connection_id);
        ethabi::encode_uint<u32>(&mut buf, channel.counterparty_channel_id);

        let mut i = 32 - vector::length(&channel.version);
        vector::append(&mut buf, channel.version);
        while (i > 0) {
            vector::push_back(&mut buf, 0);
            i = i - 1;
        };

        buf
    }

    public fun decode(buf: vector<u8>): Option<Channel> {
        let mut index = 0;

        let state = (ethabi::decode_uint(&buf, &mut index) as u8);
        let ordering = (ethabi::decode_uint(&buf, &mut index) as u8);
        let connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);
        let counterparty_connection_id = (ethabi::decode_uint(&buf, &mut index) as u32);

        let mut i = index;
        while (i < index + 32) {
            let char = *vector::borrow(&buf, i);

            if (char == 0) { break };

            i = i + 1;
        };

        let version = ethabi::vector_slice(&buf, index, i);

        option::some(
            new(
                state,
                ordering,
                connection_id,
                counterparty_connection_id,
                version
            )
        )
    }

    // Constructor
    public fun new(
        state: u8,
        ordering: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        version: vector<u8>
    ): Channel {
        Channel { state, ordering, connection_id, counterparty_channel_id, version }
    }

    // Default function
    public fun default(): Channel {
        new(0, 0, 0, 0, vector::empty())
    }

    #[test]
    public fun test_encode_decode_channel() {
        let buf =
            x"00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000c868656c6c6f000000000000000000000000000000000000000000000000000000";
        let channel = new(1, 1, 100, 200, b"hello");

        let encoded = encode(&channel);

        assert!(buf == encoded, 1);
        assert!(decode(encoded) == option::some(channel), 1);
    }
}
