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

    const E_PACKET_VERSION_LENGTH_EXCEEDS_MAX: u64 = 1;

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
        let buf = vector::empty<u8>();

        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, channel.state);
        ethabi::encode_uint<u32>(&mut buf, channel.connection_id);
        ethabi::encode_uint<u32>(&mut buf, channel.counterparty_channel_id);
        ethabi::encode_uint<u32>(&mut buf, 5 * 0x20);

        let version_offset = ((vector::length(&channel.counterparty_port_id) / 0x20) as u32);
        ethabi::encode_uint<u32>(&mut buf, (7 + version_offset) * 0x20);
        ethabi::encode_bytes(&mut buf, &channel.counterparty_port_id);

        let version_length = string::length(&channel.version);
        ethabi::encode_uint<u64>(&mut buf, version_length);
        let i = 32 - version_length;
        vector::append(&mut buf, *string::bytes(&channel.version));
        while (i > 0) {
            vector::push_back(&mut buf, 0);
            i = i - 1;
        };

        buf
    }

    // Constructor
    public fun new(
        state: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String
    ): Channel {
        assert!(string::length(&version) <= 32, E_PACKET_VERSION_LENGTH_EXCEEDS_MAX);

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
    public fun test_encode_channel() {
        let buf =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000044141414100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b75637330312d72656c6179000000000000000000000000000000000000000000";

        let channel = new(2, 1, 2, b"AAAA", string::utf8(b"ucs01-relay"));

        let encoded = encode(&channel);


        std::debug::print(&encoded);

        assert!(buf == encoded, 1);
    }
}
