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
            let n = vector::length(data);
            assert!(n > 0, 201);

            let kind = vector::borrow(data, 0);
            let kind_u8 = *kind;
            assert!(kind_u8 <= 0xA0, 202);

            if (kind_u8 < 0x80) {
                ((kind_u8 as u256), 1)
            } else {
                // Short string
                let short_len = (((kind_u8 as u64) - 0x80) as u8);

                assert!((short_len as u64) <= n, 203);

                assert!((short_len as u64) <= 32, 204);

                let val = 0;
                if (short_len == 32) {
                    let j: u8 = 0;
                    while (j < short_len-1) {
                        let idx = ((1 + j) as u64);
                        let b = (*vector::borrow(data, idx) as u256);
                        val = val | ((b << (8 * (31-j))) as u256);
                        j = j + 1;
                    }
                } else {
                    let j: u8 = 0;
                    if (short_len > 0){
                        while (j < short_len -1) {
                            let idx = ((1 + j) as u64);
                            let b = (*vector::borrow(data, idx) as u256);
                            val = val | (b << ((8 * (short_len - 1 - j))) as u256);
                            j = j + 1;
                        }
                    }
                };

                ((val as u256), (1 + short_len as u64) )
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
        let buf: vector<u8> = x"334455667733445566112233334455667733445566112233334455667733445544";

        let (result, size) = parse_uint(&buf);
        assert!(result == 51, 1);
        assert!(size == 1, 1);

        let buf: vector<u8> = x"4432";
        let (result, size) = parse_uint(&buf);
        assert!(result == 68, 1);
        assert!(size == 1, 1);

        let buf: vector<u8> = x"80";
        let (result, size) = parse_uint(&buf);
        assert!(result == 0, 1);
        assert!(size == 1, 1);

        let buf: vector<u8> = x"84848589";
        let (result, size) = parse_uint(&buf);
        assert!(result == 2223343872, 1);
        assert!(size == 5, 1);

        let buf: vector<u8> = x"85848589aa";
        let (result, size) = parse_uint(&buf);
        assert!(result == 569176074752, 1);
        assert!(size == 6, 1);

        let buf: vector<u8> = x"a012121212121212121212121212121212121212121212121212121212121212";
        let (result, size) = parse_uint(&buf);
        assert!(result == 8173559240281143206369716588848558201407293035221686873373476518205632680448, 1);
        assert!(size == 33, 1);
    }

}