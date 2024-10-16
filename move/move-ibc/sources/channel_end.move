module ibc::channel {
    use std::option::{Self, Option};
    use std::string::{Self, String, utf8};
    use std::vector;
    use ibc::proto_utils;

    const CHAN_STATE_UNINITIALIZED: u8 = 0;
    const CHAN_STATE_INIT: u8 = 1;
    const CHAN_STATE_TRYOPEN: u8 = 2;
    const CHAN_STATE_OPEN: u8 = 3;
    const CHAN_STATE_CLOSED: u8 = 4;

    const CHAN_ORDERING_NONE: u8 = 0;
    const CHAN_ORDERING_UNORDERED: u8 = 1;
    const CHAN_ORDERING_ORDERED: u8 = 2;

    struct Channel has copy, drop, store {
        state: u8,
        ordering: u8,
        counterparty: Counterparty,
        connection_hops: vector<String>,
        version: String
    }

    struct Counterparty has copy, store, drop {
        port_id: String,
        channel_id: String
    }

    public fun state(channel: &Channel): u8 {
        channel.state
    }

    public fun ordering(channel: &Channel): u8 {
        channel.ordering
    }

    public fun chan_counterparty_port_id(channel: &Channel): &String {
        &channel.counterparty.port_id
    }

    public fun chan_counterparty_channel_id(channel: &Channel): &String {
        &channel.counterparty.channel_id
    }

    public fun connection_hops(channel: &Channel): &vector<String> {
        &channel.connection_hops
    }

    public fun version(channel: &Channel): &String {
        &channel.version
    }

    public fun counterparty_port_id(counterparty: &Counterparty): &String {
        &counterparty.port_id
    }

    public fun counterparty_channel_id(counterparty: &Counterparty): &String {
        &counterparty.channel_id
    }

    public fun set_state(channel: &mut Channel, state: u8) {
        channel.state = state;
    }

    public fun set_version(channel: &mut Channel, version: String) {
        channel.version = version;
    }

    public fun set_ordering(channel: &mut Channel, ordering: u8) {
        channel.ordering = ordering;
    }

    public fun set_chan_counterparty_channel_id(
        channel: &mut Channel, channel_id: String
    ) {
        channel.counterparty.channel_id = channel_id;
    }

    public fun new(
        state: u8,
        ordering: u8,
        counterparty: Counterparty,
        connection_hops: vector<String>,
        version: String
    ): Channel {
        Channel { state, ordering, counterparty, connection_hops, version }
    }

    public fun default(): Channel {
        Channel {
            state: CHAN_STATE_UNINITIALIZED,
            ordering: CHAN_ORDERING_NONE,
            counterparty: default_counterparty(),
            connection_hops: vector::empty(),
            version: utf8(b"")
        }
    }

    public fun new_counterparty(port_id: String, channel_id: String): Counterparty {
        Counterparty { port_id, channel_id }
    }

    public fun default_counterparty(): Counterparty {
        Counterparty {
            port_id: utf8(b""),
            channel_id: utf8(b"")
        }
    }

    public fun encode_proto(chan: Channel): vector<u8> {
        let buf = vector::empty();

        if (chan.state != 0) {
            vector::append(&mut buf, proto_utils::encode_u32(1, (chan.state as u32)));
        };

        if (chan.ordering != 0) {
            vector::append(&mut buf, proto_utils::encode_u32(2, (chan.ordering as u32)));
        };

        let counterparty = encode_proto_counterparty(chan.counterparty);
        if (!vector::is_empty(&counterparty)) {
            vector::append(&mut buf, proto_utils::encode_prefix(3, 2));
            vector::append(
                &mut buf, proto_utils::encode_varint(vector::length(&counterparty))
            );
            vector::append(&mut buf, counterparty);
        };

        if (!vector::is_empty(&chan.connection_hops)) {
            let i = 0;
            while (i < vector::length(&chan.connection_hops)) {
                vector::append(
                    &mut buf,
                    proto_utils::encode_string(
                        4, *vector::borrow(&chan.connection_hops, i)
                    )
                );
                i = i + 1;
            };
        };

        if (!string::is_empty(&chan.version)) {
            vector::append(&mut buf, proto_utils::encode_string(5, chan.version));
        };

        buf
    }

    public fun encode_proto_counterparty(value: Counterparty): vector<u8> {
        let buf = vector::empty();

        if (!string::is_empty(&value.port_id)) {
            vector::append(&mut buf, proto_utils::encode_string(1, value.port_id));
        };

        if (!string::is_empty(&value.channel_id)) {
            vector::append(&mut buf, proto_utils::encode_string(2, value.channel_id));
        };

        buf
    }

    public fun decode_proto(buf: vector<u8>): Option<Channel> {
        if (vector::is_empty(&buf)) {
            return option::none()
        };
        let cursor = 0;
        let channel = default();
        while (cursor < vector::length(&buf)) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(
                &buf, cursor
            );
            if (err != 0) {
                return option::none()
            };
            cursor = cursor + advance;
            let n_read =
                if (tag == 1) {
                    let (num, advance, err) =
                        proto_utils::decode_varint(wire_type, &buf, cursor);
                    if (err != 0) {
                        return option::none()
                    };
                    channel.state = (num as u8);
                    advance
                } else if (tag == 2) {
                    let (num, advance, err) =
                        proto_utils::decode_varint(wire_type, &buf, cursor);
                    if (err != 0) {
                        return option::none()
                    };
                    channel.ordering = (num as u8);
                    advance
                } else if (tag == 3) {
                    let (len, advance, err) =
                        proto_utils::decode_nested_len(wire_type, &buf, cursor);
                    if (err != 0) {
                        return option::none()
                    };
                    cursor = cursor + advance;
                    let (n_read, err) =
                        decode_counterparty(&buf, cursor, len, &mut channel.counterparty);
                    if (err != 0 || n_read != len) {
                        return option::none()
                    };
                    len
                } else if (tag == 4) {
                    let (str, advance) =
                        proto_utils::decode_string(wire_type, &buf, cursor);
                    if (option::is_none(&str)) {
                        return option::none()
                    };
                    vector::push_back(
                        &mut channel.connection_hops, option::extract(&mut str)
                    );
                    advance
                } else if (tag == 5) {
                    let (str, advance) =
                        proto_utils::decode_string(wire_type, &buf, cursor);
                    if (option::is_none(&str)) {
                        return option::none()
                    };
                    channel.version = option::extract(&mut str);
                    advance
                } else {
                    return option::none()
                };
            cursor = cursor + n_read;
        };

        option::some(channel)
    }

    fun decode_counterparty(
        buf: &vector<u8>,
        cursor: u64,
        len: u64,
        counterparty: &mut Counterparty
    ): (u64, u64) {
        let first_pos = cursor;
        while (cursor - first_pos < len) {
            let (tag, wire_type, advance, err) = proto_utils::decode_prefix(buf, cursor);
            if (err != 0) {
                return (0, err)
            };
            cursor = cursor + advance;
            let advance =
                if (tag == 1) {
                    let (str, advance) = proto_utils::decode_string(
                        wire_type, buf, cursor
                    );
                    if (option::is_none(&str)) {
                        return (0, 1)
                    };
                    counterparty.port_id = option::extract(&mut str);
                    advance

                } else if (tag == 2) {
                    let (str, advance) = proto_utils::decode_string(
                        wire_type, buf, cursor
                    );
                    if (option::is_none(&str)) {
                        return (0, 1)
                    };
                    counterparty.channel_id = option::extract(&mut str);
                    advance
                } else {
                    return (0, 1)
                };
            cursor = cursor + advance;
        };

        (cursor - first_pos, 0)
    }

    #[test]
    fun test_channel_proto() {
        let encoded_channel =
            x"080110011a130a06706f72742d3012096368616e6e656c2d30220c636f6e6e656374696f6e2d30220c636f6e6e656374696f6e2d312a0769637332302d31";

        let channel = Channel {
            state: CHAN_STATE_INIT,
            ordering: CHAN_ORDERING_UNORDERED,
            counterparty: Counterparty {
                port_id: utf8(b"port-0"),
                channel_id: utf8(b"channel-0")
            },
            connection_hops: vector[utf8(b"connection-0"), utf8(b"connection-1")],
            version: utf8(b"ics20-1")
        };

        let res = encode_proto(channel);

        assert!(encoded_channel == res, 0);

        let decoded = decode_proto(encoded_channel);

        assert!(option::extract(&mut decoded) == channel, 1);
    }

    #[test]
    fun test_channel_proto_with_defaults() {
        let encoded_channel =
            x"080110011a0d0a0012096368616e6e656c2d30220c636f6e6e656374696f6e2d30220c636f6e6e656374696f6e2d312a00";

        let channel = Channel {
            state: CHAN_STATE_INIT,
            ordering: CHAN_ORDERING_UNORDERED,
            counterparty: Counterparty {
                port_id: utf8(b""),
                channel_id: utf8(b"channel-0")
            },
            connection_hops: vector[utf8(b"connection-0"), utf8(b"connection-1")],
            version: utf8(b"")
        };

        let res = encode_proto(channel);

        assert!(encoded_channel == res, 0);

        let decoded = decode_proto(encoded_channel);

        assert!(option::extract(&mut decoded) == channel, 1);
    }
}
