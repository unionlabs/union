module ibc::proto_utils {
    use std::string::{Self, String};
    use std::vector;
    use std::option::{Self, Option};

    public fun encode_string(field: u8, value: String): vector<u8> {
        let prefix = encode_varint(((((field << 3) as u8) | 2) as u64));
        vector::append(&mut prefix, encode_varint(string::length(&value)));
        vector::append(&mut prefix, *string::bytes(&value));
        prefix
    }

    public fun encode_bytes(field: u8, value: vector<u8>): vector<u8> {
        let prefix = encode_prefix(field, 2);
        vector::append(&mut prefix, encode_varint(vector::length(&value)));
        vector::append(&mut prefix, value);
        prefix
    }

    public fun encode_varint(value: u64): vector<u8> {
        let buf: vector<u8> = vector::empty();
        let i = 0;
        while (i < 10) {
            if (value < 0x80) {
                vector::push_back(&mut buf, (value as u8));
                break
            } else {
                vector::push_back(&mut buf, (((value & 0x7F) | 0x80) as u8));
                value = value >> 7;
            };
            i = i + 1;
        };
        buf
    }

    public fun decode_string(
        wire_type: u64, buf: &vector<u8>, cursor: u64
    ): (Option<String>, u64) {
        if (wire_type != 2) {
            return (option::none(), 0)
        };

        decode_untagged_string(buf, cursor)
    }

    public fun decode_bytes(wire_type: u64, buf: &vector<u8>, cursor: u64):
        (Option<vector<u8>>, u64) {
        if (wire_type != 2) {
            return (option::none(), 0)
        };

        decode_untagged_bytes(buf, cursor)
    }

    public fun decode_untagged_bytes(buf: &vector<u8>, cursor: u64): (Option<vector<u8>>, u64) {
        let (bytes_len, advance, err) = decode_varint_raw(buf, cursor);
        cursor = cursor + advance;
        if (err != 0) {
            return (option::none(), 0)
        };
        (
            option::some(vector::slice(buf, cursor, cursor + bytes_len)),
            advance + bytes_len
        )
    }

    public fun decode_untagged_string(buf: &vector<u8>, cursor: u64): (Option<String>, u64) {
        let (strlen, advance, err) = decode_varint_raw(buf, cursor);
        cursor = cursor + advance;
        if (err != 0) {
            return (option::none(), 0)
        };
        (string::try_utf8(vector::slice(buf, cursor, cursor + strlen)), advance
            + strlen)
    }

    public fun decode_prefix(buf: &vector<u8>, cursor: u64): (u64, u64, u64, u64) {
        let (key, advance, err) = decode_varint_raw(buf, cursor);
        if (err != 0) {
            return (0, 0, 0, err)
        };

        // TODO(aeryz): check if key > u32::MAX
        let wire_type = key & 0x07;
        let tag = (key as u32) >> 3;

        if (tag < 1) {
            return (0, 0, 0, err)
        };

        ((tag as u64), wire_type, advance, 0)

    }

    public fun decode_nested_len(
        wire_type: u64, buf: &vector<u8>, cursor: u64
    ): (u64, u64, u64) {
        if (wire_type != 2) {
            return (0, 0, 1)
        };

        decode_varint_raw(buf, cursor)
    }

    public fun decode_varint(
        wire_type: u64, buf: &vector<u8>, cursor: u64
    ): (u64, u64, u64) {
        if (wire_type != 0) {
            return (0, 0, 1)
        };

        decode_varint_raw(buf, cursor)
    }

    public fun decode_varint_raw(buf: &vector<u8>, cursor: u64): (u64, u64, u64) {
        let len = vector::length(buf) - cursor;
        if (len == 0) {
            return (0, 0, 1)
        };

        let byte = *vector::borrow(buf, cursor);
        if (byte < 0x80) {
            ((byte as u64), 1, 0)
        } else if (len > 10 || *vector::borrow(buf, cursor + len - 1) < 0x80) {
            decode_varint_raw_slice(buf, cursor)
        } else {
            decode_varint_raw_slow(buf, cursor)
        }
    }

    fun decode_varint_raw_slice(buf: &vector<u8>, cursor: u64): (u64, u64, u64) {
        let len = vector::length(buf) - cursor;
        if (len <= 10 && *vector::borrow(buf, cursor + len - 1) >= 0x80) {
            return (0, 0, 1)
        };

        let b = *vector::borrow(buf, cursor);
        let part0 = (b as u32);
        if (b < 0x80) {
            return ((part0 as u64), 1, 0)
        };

        part0 = part0 - 0x80;
        let b = *vector::borrow(buf, cursor + 1);
        part0 = part0 + ((b as u32) << 7);
        if (b < 0x80) {
            return ((part0 as u64), 2, 0)
        };

        part0 = part0 - (0x80 << 7);
        let b = *vector::borrow(buf, cursor + 2);
        part0 = part0 + ((b as u32) << 14);
        if (b < 0x80) {
            return ((part0 as u64), 3, 0)
        };

        part0 = part0 - (0x80 << 14);
        let b = *vector::borrow(buf, cursor + 3);
        part0 = part0 + ((b as u32) << 21);
        if (b < 0x80) {
            return ((part0 as u64), 4, 0)
        };

        part0 = part0 - (0x80 << 21);
        let value = (part0 as u64);

        let b = *vector::borrow(buf, cursor + 4);
        let part1 = (b as u32);
        if (b < 0x80) {
            return (value + ((part1 as u64) << 28), 5, 0)
        };

        part1 = part1 - 0x80;
        let b = *vector::borrow(buf, cursor + 5);
        let part1 = part1 + ((b as u32) << 7);
        if (b < 0x80) {
            return (value + ((part1 as u64) << 28), 6, 0)
        };

        part1 = part1 - (0x80 << 7);
        let b = *vector::borrow(buf, cursor + 6);
        let part1 = part1 + ((b as u32) << 14);
        if (b < 0x80) {
            return (value + ((part1 as u64) << 28), 7, 0)
        };

        part1 = part1 - (0x80 << 14);
        let b = *vector::borrow(buf, cursor + 7);
        let part1 = part1 + ((b as u32) << 21);
        if (b < 0x80) {
            return (value + ((part1 as u64) << 28), 8, 0)
        };

        part1 = part1 - (0x80 << 21);
        let value = value + ((part1 as u64) << 28);

        let b = *vector::borrow(buf, cursor + 8);
        let part2 = (b as u32);
        if (b < 0x80) {
            return (value + ((part2 as u64) << 56), 9, 0)
        };
        part2 = part2 - 0x80;
        let b = *vector::borrow(buf, cursor + 9);
        let part2 = part2 + ((b as u32) << 7);
        if (b < 0x02) {
            return (value + ((part2 as u64) << 56), 10, 0)
        };

        (0, 0, 1)
    }

    fun decode_varint_raw_slow(buf: &vector<u8>, cursor: u64): (u64, u64, u64) {
        let value = 0u64;
        let rem = vector::length(buf) - cursor;
        if (rem > 10) {
            rem = 10;
        };

        let count = 0u8;
        while ((count as u64) < rem) {
            let byte = *vector::borrow(buf, cursor);
            value = value | ((byte & 0x7F) as u64) << (count * 7);
            if (byte <= 0x7F) {
                if (count == 9 && byte >= 0x02) {
                    return (0, 0, 1)
                } else {
                    return (value, (count as u64), 0)
                }
            };
            count = count + 1;
        };

        (0, 0, 1)
    }

    public fun encode_prefix(field: u8, wire_type: u8): vector<u8> {
        encode_varint(((((field << 3) as u8) | wire_type) as u64))
    }

    public fun encode_u64(field: u8, value: u64): vector<u8> {
        let prefix = encode_prefix(field, 0);
        vector::append(&mut prefix, encode_varint(value));
        prefix
    }

    public fun encode_u32(field: u8, value: u32): vector<u8> {
        let prefix = encode_prefix(field, 0);
        vector::append(&mut prefix, encode_varint((value as u64)));
        prefix
    }

    #[test]
    public fun test_varint() {
        let exp = vector<u64>[
            100,
            10000,
            100000,
            100000000,
            10000000000,
            1000000000000,
            10000000000000,
            1000000000000000,
            100000000000000000,
            10000000000000000000
        ];

        let i = 0;
        while (i < vector::length(&exp)) {
            let exp = *vector::borrow(&exp, i);
            let enc_num = encode_varint(exp);
            let (num, _, _) = decode_varint_raw(&enc_num, 0);
            assert!(num == exp, 0);
            i = i + 1;
        };
    }

    #[test]
    public fun test_str() {
        let exp = vector[
            string::utf8(b"h"),
            string::utf8(b"hello world"),
            string::utf8(b"hello world hello again")
        ];

        let i = 0;
        while (i < vector::length(&exp)) {
            let exp = *vector::borrow(&exp, i);
            let enc = encode_string(1, exp);
            std::debug::print(&enc);
            let (tag, wire_type, advance, err) = decode_prefix(&enc, 0);
            assert!(tag == 1, 0);
            assert!(wire_type == 2, 0);
            assert!(err == 0, 0);

            let (dec, _) = decode_untagged_string(&enc, advance);
            assert!(option::extract<String>(&mut dec) == exp, 0);

            i = i + 1;
        };
    }

    #[test]
    public fun encode_prefix_test() {
        std::debug::print(&encode_prefix(4, 2));
    }
}
