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


    public fun parse_list(data: &vector<u8>): (u256, u256) {
        let n = vector::length(data);
        assert!(n > 0, 201); // Ensure we have at least one byte
        let kind = *vector::borrow(data, 0); // Get first byte

        assert!(kind >= 0xC0, 8002);

        if (kind < 0xF8) {
            let list_size = (kind as u256) - 0xC0;
            let offset = 1;
            return (list_size, offset);
        };
        let length_size = (kind as u64) - 0xF7;
        assert!(length_size <= 31, 202); // Length size must be <= 31
        assert!(n >= ((length_size as u64) + 1), 203); // Ensure buffer has enough bytes

        let length = 0;
        if (length_size > 0) {
            let j = 0;
            while(j < length_size) {
                let idx = (1 + (j as u64));
                let b = 0;
                if (idx < n) {
                    b = (*vector::borrow(data, idx) as u256);
                };
                length = (length << 8) | (b);
                j = j + 1;
            }
        };
        let list_size = (length as u256);
        let offset = (1 + (length_size as u256));
        (list_size, offset)

    }

    public fun next_size(data: &vector<u8>): u64 {
        let n = vector::length(data);
        assert!(n > 0, 201); // Ensure we have at least one byte

        let kind = vector::borrow(data, 0); // Get first byte
        let kind_u8 = *kind;

        if (kind_u8 < 0x80) {
            // Small single byte
            return 1;
        } else if (kind_u8 < 0xB8) {
            // Short string
            return 1 + (((kind_u8 as u256) - 0x80) as u64);
        } else if (kind_u8 < 0xC0) {
            // Long string
            let length_size = (((kind_u8 as u256) - 0xB7) as u8);

            // Ensure that we don't overflow and don't read out of bounds
            assert!(length_size <= 31, 202); // Length size must be <= 31
            assert!(n >= ((length_size as u64) + 1), 203); // Ensure buffer has enough bytes

            let length = 0;
            if (length_size > 0) {
                let j = 0;
                while(j < length_size) {
                    let idx = (1 + (j as u64));
                    let b = 0;
                    if (idx < n) {
                        b = (*vector::borrow(data, idx) as u256);
                    };
                    length = (length << 8) | (b);
                    j = j + 1;
                }
            };
            return (length as u64) + (1 + (length_size as u64));
        } else if (kind_u8 < 0xF8) {
            // Short list
            return 1 + (((kind_u8 as u256) - 0xC0) as u64);
        };
        // Long list
        let length_size = (((kind_u8 as u256) - 0xf7) as u8);

        // Ensure that we don't overflow and don't read out of bounds
        assert!(length_size <= 31, 202); // Length size must be <= 31
        assert!(n >= ((length_size as u64)), 203); // Ensure buffer has enough bytes

        let length = 0;
        if (length_size > 0) {
            let j = 0;
            while(j < length_size) {
                let idx = (1 + (j as u64));

                    let b = 0;
                    if (idx < n) {
                        b = (*vector::borrow(data, idx) as u256);
                    };
                length = (length << 8) | (b);
                j = j + 1;
            }
        };
        (length as u64) + (1 + (length_size as u64))
    }

//0xffab1234567891ba
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
                while (j < short_len) {
                    let idx = ((1 + j) as u64);
                    let b = 0;
                    if (idx < n) {
                        b = (*vector::borrow(data, idx) as u256);
                    };
                    val = val | ((b << (8 * (31-j))) as u256);
                    j = j + 1;
                }
            } else {
                let j: u8 = 0;
                if (short_len > 0){
                    while (j < short_len) {
                        let idx = ((1 + j) as u64);
                        let b = 0;
                        if (idx < n) {
                            b = (*vector::borrow(data, idx) as u256);
                        };
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
        assert!(n > 0, 201); // Ensure we have at least one byte

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
                assert!(offset + size >= n, 9001);
            } else {
                let length_size = (((kind as u256) - 0xB7) as u8);
                assert!(length_size <= 31, 9003);
                assert!(n >= ((length_size as u64) + 1), 203); // Ensure buffer has enough bytes

                let length = 0;
                if (length_size == 32) {
                    let j: u8 = 0;
                    while (j < length_size) {
                        let idx = ((1 + j) as u64);
                        let b = 0;
                        if (idx < n) {
                            b = (*vector::borrow(data, idx) as u256);
                        };
                        length = length | ((b << (8 * (31-j))) as u256);
                        j = j + 1;
                    }
                };
                size = (length as u64);
                offset = 1 + (length_size as u64);
                assert!(offset + size >= n, 9004);
            }

        };
        let end  = offset + size;
        assert!(end <= n, 9005);
        let result = vector::slice(data, offset, end);
        let rest = vector::slice(data, end, n);
        (result, rest)
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

    #[test]
    public fun test_next_size() {
        let buf: vector<u8> = x"ffab1234567891ba";
        let size = next_size(&buf);
        assert!(size == 12326972676061116937, 1);

        let buf: vector<u8> = x"75";
        let size = next_size(&buf);
        assert!(size == 1, 1);

        let buf: vector<u8> = x"85";
        let size = next_size(&buf);
        assert!(size == 6, 1);

        let buf: vector<u8> = x"b9b8b7";
        let size = next_size(&buf);
        assert!(size == 47290, 1);

        let buf: vector<u8> = x"b9b8b7";
        let size = next_size(&buf);
        assert!(size == 47290, 1);

        let buf: vector<u8> = x"babbaacc";
        let size = next_size(&buf);
        assert!(size == 12298960, 1);

        let buf: vector<u8> = x"c5c4";
        let size = next_size(&buf);
        assert!(size == 6, 1);
    }


    #[test]
    public fun test_parse_list() {
        let buf: vector<u8> = x"f9aabbcc";
        let (list_size, offset) = parse_list(&buf);
        assert!(list_size == 43707, 1);
        assert!(offset == 3, 1);

        let buf: vector<u8> = x"faaabbccdd";
        let (list_size, offset) = parse_list(&buf);
        assert!(list_size == 11189196, 1);
        assert!(offset == 4, 1);
    }

    #[test]
    public fun test_split_bytes() {
        let buf: vector<u8> = x"b90000ab12";
        let (result, res) = split_bytes(&buf);
        assert!(result == x"", 1);
        assert!(res == x"ab12", 1);

        let buf: vector<u8> = x"b800113344";
        let (result, res) = split_bytes(&buf);
        assert!(result == x"", 1);
        assert!(res == x"113344", 1);

        let buf: vector<u8> = x"b80011";
        let (result, res) = split_bytes(&buf);
        assert!(result == x"", 1);
        assert!(res == x"11", 1);
    }
}