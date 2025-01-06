module ibc::mpt_verifier {

    use aptos_std::hash;
    use aptos_std::vector;
    use std::string::{Self};
    use std::bcs;
    use aptos_std::aptos_hash::keccak256;
    use std::from_bcs;

    const BIG_25: u256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    const BIG_32: u256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;

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
        assert!(n > 0, 201);

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
                assert!(n >= ((length_size as u64) + 1), 203);

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

    public fun encode_uint(value: u256): vector<u8> {
        if (value == 0) {
            return x"80";
        };

        if (value < 128) {
            let out = vector::empty<u8>();
            vector::push_back(&mut out, (value as u8));
            return out;
        };


        if (value > BIG_25) {
            let out = vector::empty<u8>();
            vector::push_back(&mut out, 0xa0);

            let i = 0;
            while (i < 32) {
                let shift_bits = (31 - i) * 8;
                let b_u256 = (value >> (shift_bits as u8)) & 0xff;
                let b_u8 = (b_u256 as u8);
                vector::push_back(&mut out, b_u8);
                i = i + 1;
            };
            return out;
        };


        if (value > BIG_32) {
            let out = vector::empty<u8>();
            vector::push_back(&mut out, 0x9f);

            let shifted = value << 8;
            let i = 0;
            while (i < 32) {
                let shift_bits = (31 - i) * 8;
                let b_u256 = (shifted >> (shift_bits as u8)) & 0xff;
                vector::push_back(&mut out, (b_u256 as u8));
                i = i + 1;
            };
            return out;
        };


        let n_bytes = num_bytes(value);
        let prefix = (0x80 + (n_bytes as u8)) & 0xff;

        let out = vector::empty<u8>();
        vector::push_back(&mut out, prefix);

        let i = 0;
        while (i < (n_bytes as u64)) {
            let shift_bits = ((n_bytes as u64) - 1 - i) * 8;
            let b_u256 = (value >> (shift_bits as u8)) & 0xff;
            vector::push_back(&mut out, (b_u256 as u8));
            i = i + 1;
        };
        out
    }

    fun num_bytes(value: u256): u8 {
        if (value == 0) {
            return 1;
        };

        let len = 0u8;
        let temp = value;
        while (temp != 0) {
            temp = temp >> 8;
            len = len + 1;
        };
        len
    }

    public fun parse_hash(data: &vector<u8>): (vector<u8>, u64) {
        let (val, consumed) = parse_uint(data);
        let hash_bytes = bcs::to_bytes<u256>(&val);
        vector::reverse(&mut hash_bytes);
        (hash_bytes, consumed)
    }

    public fun parse_nodes(input: &vector<u8>): vector<Node> {
        let nodes = vector::empty<Node>();

        let remaining = vector::empty<u8>();
        vector::append(&mut remaining, *input);

        while (vector::length(&remaining) > 0) {
            let (list_size, offset) = parse_list(&remaining);

            let total_len = (vector::length(&remaining) as u256);
            let consumed = offset + list_size;
            assert!(consumed <= total_len, 777);

            let node_data = vector::slice(&remaining, (offset as u64), (consumed as u64));
            let node_hash = keccak256(*input);

            let node = Node {
                data: node_data,
                hash: node_hash
            };
            vector::push_back(&mut nodes, node);

            let leftover = total_len - consumed;
            if (leftover == 0){
                remaining = vector::empty();
            } else{
                remaining = vector::slice(&remaining, (consumed as u64), (leftover as u64));
            };
        };

        nodes
    }

    fun subkeys_equal(
        key: u256,
        key_len: u64,
        test_bytes: &vector<u8>
    ): bool {
        let nibble_length = 2 * (vector::length(test_bytes));

        assert!(nibble_length <= key_len, 111 /* some error code */);

        let shift_amount: u256 = 256 - (4 * (nibble_length as u256));

        let test_raw = load_u256_big_endian(test_bytes);

        let test_value = test_raw >> (shift_amount as u8);
        let subkey     = key      >> (shift_amount as u8);

        test_value == subkey
    }


    fun load_u256_big_endian(data: &vector<u8>): u256 {
        let result = 0u256;
        let len = vector::length(data);

        let i = 0;
        while (i < 32) {
            result = result << 8;

            if (i < len) {
                let b = *vector::borrow(data, i);
                result = result | (b as u256);
            };

            i = i + 1;
        };

        result
    }

    #[test]
    public fun test_parse_nodes_small() {
        // A minimal RLP-encoded node: 
        //   0xc6 => short list of length 6
        //   0x85 => short string of length 5
        //   'h' 'e' 'l' 'l' 'o'
        let data: vector<u8> = x"c68568656c6c6f";

        // Call your parse_nodes(...) on this small example
        let nodes = parse_nodes(&data);

        // We expect exactly 1 node
        assert!(vector::length(&nodes) == 1, 9991);

        let node0 = *vector::borrow(&nodes, 0);
        // node0.data should be x"8568656c6c6f", i.e. 6 bytes
        assert!(vector::length(&node0.data) == 6, 9992);

        // For curiosity, let's print them
        std::debug::print(&string::utf8(b"node0.data: "));
        std::debug::print(&node0.data);
        std::debug::print(&string::utf8(b"node0.hash: "));
        std::debug::print(&node0.hash);

        // Optionally, you can confirm the hash matches keccak256 of x"c68568656c6c6f"
        let check = aptos_std::aptos_hash::keccak256(data);
        assert!(node0.hash == check, 9993);
    }


    #[test]
    public fun test_parse_hash() {
        let buf: vector<u8> = x"334455667733445566112233334455667733445566112233334455667733445544";

        let (result, offset) = parse_hash(&buf);
        std::debug::print(&string::utf8(b"result: "));
        std::debug::print(&result);
        assert!(result == x"0000000000000000000000000000000000000000000000000000000000000033", 1);
        assert!(offset == 1, 1);
    }

    #[test]
    public fun test_parse_nodes() {
        let buf: vector<u8> = x"f90211a0b51ceda38c7c0d96cee1d651d8c9001299aae0a56dd4778366faccf8c89802f0a011e1adf2007c6afdc9300271c03ad104cf9ed625a3cca7050416449175f7ef21a0e4187606d7baba63b37fd6978f264374e8d7289da084c4a56170ce1e438ff0f0a061869b1b76c51cc75983fc4792b3fc9c1c5e366a76149979920143afd2899770a0ae2ffd634be69d00ca955e55ad4bb4c1065d40938f82f56d678a87180087d2aba0dcfab65101c9968d7891a91ffc1d6c8bcda2773458d802feca923a7d938f7695a0c62fdc1d9731b77b5310a9a9e1bc9edb79976637f6f29c13ce49459ef7cdb7d5a0fce12c4968e940f0f7dbe888d359b81425bde60f261761608465fd74fa390828a04f77e522f007df2b5c6090006e531d113647900ef01ce8ddad6b6b908e786ce9a04beb43119c19f9f2b94738830b8ca07ce2cb40a2fc60e51567810deda9719527a05085bfa24339e17ba1305a8d7c93468ab8414fde3b1b0ce77ea3f196e16217eaa0071e1a46d2a544b7cc24d3153619887ab88606501aea6f30f03e084dab9da01aa0a27d98ca7583cd6f303c41747e5109978c3399cb632283a9a6d5300366bfc97ca0c38268688069ddd9ec101532ea6f0253025f9df93c6d5e916968221232f8da00a0654ec1fadfb6c2d7849b96c26a1373e111cc6fd30c408ee833e0e2a89c4828f7a04a1eba1371dffabf57cd6f2a1774d2d464968546390a9f4dd78a76444cfce53580";

        let result = parse_nodes(&buf);
        std::debug::print(&string::utf8(b"result: "));
        std::debug::print(&result);

        let test_me_hard = x"4700000000000000000000000000000211";
        vector::reverse(&mut test_me_hard);
        let res = keccak256(test_me_hard);
        std::debug::print(&string::utf8(b"res: "));
        std::debug::print(&res);
        // assert!(result == x"0000000000000000000000000000000000000000000000000000000000000033", 1);
        // assert!(offset == 1, 1);
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

        let buf: vector<u8> = x"f90211a0b51ceda38c7c0d96cee1d651d8c9001299aae0a56dd4778366faccf8c89802f0a011e1adf2007c6afdc9300271c03ad104cf9ed625a3cca7050416449175f7ef21a0e4187606d7baba63b37fd6978f264374e8d7289da084c4a56170ce1e438ff0f0a061869b1b76c51cc75983fc4792b3fc9c1c5e366a76149979920143afd2899770a0ae2ffd634be69d00ca955e55ad4bb4c1065d40938f82f56d678a87180087d2aba0dcfab65101c9968d7891a91ffc1d6c8bcda2773458d802feca923a7d938f7695a0c62fdc1d9731b77b5310a9a9e1bc9edb79976637f6f29c13ce49459ef7cdb7d5a0fce12c4968e940f0f7dbe888d359b81425bde60f261761608465fd74fa390828a04f77e522f007df2b5c6090006e531d113647900ef01ce8ddad6b6b908e786ce9a04beb43119c19f9f2b94738830b8ca07ce2cb40a2fc60e51567810deda9719527a05085bfa24339e17ba1305a8d7c93468ab8414fde3b1b0ce77ea3f196e16217eaa0071e1a46d2a544b7cc24d3153619887ab88606501aea6f30f03e084dab9da01aa0a27d98ca7583cd6f303c41747e5109978c3399cb632283a9a6d5300366bfc97ca0c38268688069ddd9ec101532ea6f0253025f9df93c6d5e916968221232f8da00a0654ec1fadfb6c2d7849b96c26a1373e111cc6fd30c408ee833e0e2a89c4828f7a04a1eba1371dffabf57cd6f2a1774d2d464968546390a9f4dd78a76444cfce53580";
        let (list_size, offset) = parse_list(&buf);
        assert!(list_size == 529, 1);
        assert!(offset == 3, 1);
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

    #[test]
    public fun test_encode_uint() {
        let result = encode_uint(1234455);
        assert!(result == x"8312d617", 1);

        let result = encode_uint(14);
        assert!(result == x"0e", 1);

        let result = encode_uint(1234455321122545);
        assert!(result == x"870462bb06e816f1", 1);
    }
}