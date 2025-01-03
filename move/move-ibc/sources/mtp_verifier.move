module ibc::mpt_verifier {

    use aptos_std::hash;
    use aptos_std::vector;
    use std::string::{Self};
    use aptos_std::aptos_hash::keccak256;
    use std::from_bcs;

    /// A lightweight struct containing the node bytes + node hash
    struct Node has copy, drop, store {
        data: vector<u8>,  // entire node RLP data
        hash: vector<u8>,  // keccak256(data)
    }


    public fun parse_list(data: &vector<u8>): (u64, u64) {
        let start_idx = 0;
        let n = vector::length(data);
        assert!(start_idx < n, 8001);

        let kind = *vector::borrow(data, start_idx);

        assert!(kind >= 0xC0, 8002);

        if (kind < 0xF8) {
            let list_size = (kind as u64) - 0xC0;
            let offset = 1;
            (list_size, offset)

        } else {
            let length_size = (kind as u64) - 0xF7;
            assert!(length_size <= 31, 8003);
            assert!((start_idx + 1 + length_size) <= n, 8004);

            let list_size = 0u64;
            let i = 0u64;
            while (i < length_size) {
                let b = *vector::borrow(data, start_idx + 1 + i);
                list_size = (list_size << 8) | (b as u64);
                i = i + 1;
            };
            let offset = length_size + 1;
            (list_size, offset)
        }
    }

    public fun next_size(data: &vector<u8>): u64 {
        let start_idx = 0;
        let n = vector::length(data);
        assert!(start_idx < n, 101);

        // TODO: not %100 sure about this
        let kind = *vector::borrow(data, start_idx);

        if (kind < 0x80) {
            1
        } else if (kind < 0xB8) {
            (1 + (kind as u64) - 0x80)
        } else if (kind < 0xC0) {
            let length_size = (kind as u64) - 0xB7;
            assert!(length_size <= 31, 102);
            assert!((start_idx + 1 + length_size) <= n, 103);

            let length: u64 = 0;
            let i = 0;
            while (i < length_size) {
                let b = *vector::borrow(data, start_idx + 1 + i);
                length = (length << 8) | (b as u64);
                i = i + 1;
            };
            (1 + length_size + length)

        } else if (kind < 0xF8) {
            (1 + (kind as u64) - 0xC0)

        } else {
            let length_size = (kind as u64) - 0xF7;
            assert!(length_size <= 31, 104);
            assert!((start_idx + 1 + length_size) <= n, 105);

            let length: u64 = 0;
            let i = 0;
            while (i < length_size) {
                let b = *vector::borrow(data, start_idx + 1 + i);
                length = (length << 8) | (b as u64);
                i = i + 1;
            };
            (1 + length_size + length)
        }
    }

    public fun skip(data: &vector<u8>): vector<u8> {
        let start_idx = 0;
        let size_to_skip = next_size(data);
        let length_data = vector::length(data);

        let new_start = start_idx + size_to_skip;
        assert!(new_start <= length_data, 1101);

        let remainder = vector::empty<u8>();
        let i = new_start;
        while (i < length_data) {
            let b = *vector::borrow(data, i);
            vector::push_back(&mut remainder, b);
            i = i + 1;
        };
        remainder
    }


    public fun parse_uint(data: &vector<u8>): (u256, u64) {
        let start_idx = 0;
        // TODO: All of these are wrong, need to investigate
        // calldataload mechanism in solidity assembly not sure how it works.
        // all below also wrong, need to be fixed.
        let n = vector::length(data);
        assert!(start_idx < n, 201);

        // let kind = *vector::borrow(data, start_idx);
        let val = vector::empty();
        let i = 0;
        while (i < 32) {
            vector::push_back(&mut val, *vector::borrow(data, start_idx+i));
            i = i + 1;
        };
        let kind = from_bcs::to_u256(val);

        assert!(kind <= 0xA0, 202);

        if (kind < 0x80) {
            std::debug::print(&string::utf8(b"returning under if: "));
            (kind, 1)
        } else {
            let short_len = (kind as u64) - 0x80;

            assert!((start_idx + 1 + short_len) <= n, 203);

            assert!(short_len <= 32, 204);

            let val = vector::empty<u8>();
            let i = 0u64;
            while (i < (32 - short_len)) {
                vector::push_back(&mut val, 0);
                i = i + 1;
            };

            let j = 0u64;
            while (j < short_len) {
                let b = *vector::borrow(data, start_idx + 1 + j);
                vector::push_back(&mut val, b);
                j = j + 1;
            };
            std::debug::print(&string::utf8(b"returning under else: "));

            (kind, 1 + short_len)
        }
    }

    public fun split_bytes(data: &vector<u8>): (vector<u8>, vector<u8>) {
        let n = vector::length(data);
        assert!(n > 0, 9001);

        let kind = *vector::borrow(data, 0);

        assert!(kind <= 0xBF, 9002);

        let offset = 0u64;
        let size = 0u64;

        if (kind < 0x80) {
            offset = 0;
            size = 1;
        } else {
            if (kind < 0xB8) {
                offset = 1;
                size = (kind as u64) - 0x80;
            } else {
                let length_size = (kind as u64) - 0xB7;
                assert!(length_size <= 31, 9003);
                assert!((1 + length_size) <= n, 9004);

                let tmp_len = 0u64;
                let i = 0u64;
                while (i < length_size) {
                    let b = *vector::borrow(data, 1 + i);
                    tmp_len = (tmp_len << 8) | (b as u64);
                    i = i + 1;
                };

                size = tmp_len;
                offset = 1 + length_size;
            }
        };

        let end_of_result = offset + size;
        assert!(end_of_result <= n, 9005);

        let splitted = vector::slice(data, offset, size);

        let remainder_len = n - end_of_result;
        let remainder = vector::slice(data, end_of_result, remainder_len);

        (splitted, remainder)
    }


    #[test]
    public fun test_parse_uint() {
        let buf = x"15";

        let (result, size) = parse_uint(&buf);
        std::debug::print(&string::utf8(b"result is: "));
        std::debug::print(&result);
        std::debug::print(&string::utf8(b"size is: "));
        std::debug::print(&size);
        assert!(result == 21, 1);
        assert!(size == 1, 1);
    }

}