module ibc::mpt_verifier {

    use aptos_std::hash;
    use aptos_std::vector;
    use std::string::{Self};
    use std::bcs;
    use aptos_std::aptos_hash::keccak256;
    use std::from_bcs;

    const BIG_25: u256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    const BIG_32: u256 = 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff;
    const ODD_LENGTH: u8 = 1;
    const LEAF: u8       = 2;
    const MAX_PREFIX: u8 = 3;



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
                assert!(n >= offset + size , 9001);
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
                assert!(n >= offset + size , 9001);
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
            // std::debug::print(&string::utf8(b"consumed: "));
            // std::debug::print(&consumed);
            // std::debug::print(&string::utf8(b"offset: "));
            // std::debug::print(&offset);
            // std::debug::print(&string::utf8(b"total_len: "));
            // std::debug::print(&total_len);
            // std::debug::print(&string::utf8(b"input: "));
            // std::debug::print(&input);
            assert!(consumed <= total_len, 777);

            let node_data = vector::slice(&remaining, 0, (consumed as u64));
            let node_hash = keccak256(node_data);
            let node_data = vector::slice(&remaining, (offset as u64), (consumed as u64));
            // std::debug::print(&string::utf8(b"node_hash: "));
            // std::debug::print(&node_hash);
            // std::debug::print(&string::utf8(b"node_data: "));
            // std::debug::print(&node_data);

            let node = Node {
                data: node_data,
                hash: node_hash
            };
            vector::push_back(&mut nodes, node);

            let leftover = total_len - consumed;
            if (leftover == 0){
                remaining = vector::empty();
            } else{
                remaining = vector::slice(&remaining, (consumed as u64), (total_len as u64));
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

    public fun suffix_bytes(
        data: &vector<u8>,
        start: u64
    ): vector<u8> {
        let len_data = vector::length(data);
        assert!(start <= len_data, 1007);
        vector::slice(data, start, len_data)
    }

    public fun verify_trie_value_with_nodes(
        nodes: &vector<Node>,
        key_u256: u256,
        key_len_bytes: u64,
        expected_hash: vector<u8>
    ): (bool, vector<u8>) {
        if (vector::length(nodes) == 0) {
            let empty_trie = x"80";
            let empty_hash = aptos_std::aptos_hash::keccak256(empty_trie);
            if (empty_hash != expected_hash) {
                // root hash incorrect
                std::debug::print(&string::utf8(b"breaking here: "));
                return (false, vector::empty<u8>());
            };
        };

        let key_len_nibbles = key_len_bytes * 2;

        let is_exists = true;
        let value = vector::empty<u8>();

        let i = 0;
        let nodes_count = vector::length(nodes);

        while (true) {
            if (i >= nodes_count) {
                std::debug::print(&string::utf8(b"breaking here: "));
                return (false, vector::empty<u8>());
            };

            let node_struct = *vector::borrow(nodes, i);
            i = i + 1;

            let node_hash = &node_struct.hash;
            // if (node_struct.hash != expected_hash) {
            //     std::debug::print(&string::utf8(b"breaking here2: "));
            //     return (false, vector::empty<u8>());
            // };

            let node_data = node_struct.data;
            let node_len = vector::length(&node_data);

            let size1 = next_size(&node_data);
            std::debug::print(&string::utf8(b"size1: "));
            std::debug::print(&size1);
            std::debug::print(&string::utf8(b"node_len: "));
            std::debug::print(&node_len);
            std::debug::print(&string::utf8(b"node_data: "));
            std::debug::print(&node_data);

            let suffix_1 = suffix_bytes(&node_data, size1);
            let size2 = next_size(&suffix_1);
            let sum_size = size1 + size2;
            std::debug::print(&string::utf8(b"size2: "));
            std::debug::print(&size2);

            if (sum_size == node_len) {
                std::debug::print(&string::utf8(b"node_data split bytes: "));
                std::debug::print(&node_data);
                let (encoded_path, node_remainder) = split_bytes(&node_data);
                node_data = node_remainder;
                std::debug::print(&string::utf8(b"encoded_path: "));
                std::debug::print(&encoded_path);

                let kind = *vector::borrow(&encoded_path, 0);
                let prefix = (kind >> 4);
                assert!(prefix <= MAX_PREFIX, 9008);

                let keys_match = false;
                std::debug::print(&string::utf8(b"prefix: "));
                std::debug::print(&prefix);
                if ((prefix & ODD_LENGTH) == 0) {
                    assert!((kind & 0x0f) == 0, 9009);
                    keys_match = true;
                } else {
                    let secondNibble = (kind & 0x0f);
                    let topNibble = ((key_u256 >> 252) & 0x0f);
                    keys_match = ((secondNibble as u256) == topNibble);

                    key_u256 = key_u256 << 4;
                    key_len_nibbles = key_len_nibbles - 1;
                };

                let encoded_path_rest = suffix_bytes(&encoded_path, 1);
                if (keys_match) {
                    let subkeys_ok = subkeys_equal(
                        key_u256,
                        key_len_nibbles,
                        &encoded_path_rest
                    );
                    keys_match = keys_match && subkeys_ok;
                };

                let path_len = vector::length(&encoded_path_rest);
                key_u256 = key_u256 << ( (path_len * 8) as u8 );
                key_len_nibbles = key_len_nibbles - (2 * path_len);

                if ((prefix & LEAF) == 0) {
                    assert!(keys_match, 9010);

                    let (hash_bytes, _) = parse_hash(&node_data);
                    expected_hash = hash_bytes;
                } else {
                    assert!(key_len_nibbles == 0, 9011);

                    if (keys_match) {
                        let (valBytes, _) = split_bytes(&node_data);
                        value = valBytes;
                        break;
                    } else {
                        std::debug::print(&string::utf8(b"breaking here3: "));
                        is_exists = false;
                        break;
                    }
                };

            } else {
                let key_nibble = (key_u256 >> 252);

                let node_branch = suffix_bytes(&node_data, sum_size);

                let i2 = 0;
                while (i2 < key_nibble) {
                    node_branch = skip(&node_branch);
                    i2 = i2 + 1;
                };
                std::debug::print(&string::utf8(b"key_nibble: "));
                std::debug::print(&key_nibble);
                std::debug::print(&string::utf8(b"node_branch: "));
                std::debug::print(&node_branch);
                std::debug::print(&string::utf8(b"i2: "));
                std::debug::print(&i2);

                let (hash_bytes, _) = parse_hash(&node_branch);
                std::debug::print(&string::utf8(b"hash_bytes: "));
                std::debug::print(&hash_bytes);

                if (vector::length(&hash_bytes) == 0) {
                    is_exists = false;
                    break;
                };
                expected_hash = hash_bytes;

                key_u256 = key_u256 << 4;
                key_len_nibbles = key_len_nibbles - 1;
            };
        };
        (is_exists, value)
    }

    public fun verify_trie_value(
        proof: &vector<u8>,
        key: &vector<u8>,
        root_hash: vector<u8>
    ): (bool, vector<u8>) {
        let nodes = parse_nodes(proof);
        let key_u256 = load_u256_big_endian(key);
        verify_trie_value_with_nodes(&nodes, key_u256, 32, root_hash)
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

        // Optionally, you can confirm the hash matches keccak256 of x"c68568656c6c6f"
        let check = aptos_std::aptos_hash::keccak256(data);
        assert!(node0.hash == check, 9993);
    }

    #[test]
    public fun test_non_existance() {
        let storage_root = x"195170ca4e76873504de92ee3651ba91e339555d9d008c5995e51c2c3ada74eb";
        let proof0 = x"f90211a0b51ceda38c7c0d96cee1d651d8c9001299aae0a56dd4778366faccf8c89802f0a011e1adf2007c6afdc9300271c03ad104cf9ed625a3cca7050416449175f7ef21a0e4187606d7baba63b37fd6978f264374e8d7289da084c4a56170ce1e438ff0f0a061869b1b76c51cc75983fc4792b3fc9c1c5e366a76149979920143afd2899770a0ae2ffd634be69d00ca955e55ad4bb4c1065d40938f82f56d678a87180087d2aba0dcfab65101c9968d7891a91ffc1d6c8bcda2773458d802feca923a7d938f7695a0c62fdc1d9731b77b5310a9a9e1bc9edb79976637f6f29c13ce49459ef7cdb7d5a0fce12c4968e940f0f7dbe888d359b81425bde60f261761608465fd74fa390828a04f77e522f007df2b5c6090006e531d113647900ef01ce8ddad6b6b908e786ce9a04beb43119c19f9f2b94738830b8ca07ce2cb40a2fc60e51567810deda9719527a05085bfa24339e17ba1305a8d7c93468ab8414fde3b1b0ce77ea3f196e16217eaa0071e1a46d2a544b7cc24d3153619887ab88606501aea6f30f03e084dab9da01aa0a27d98ca7583cd6f303c41747e5109978c3399cb632283a9a6d5300366bfc97ca0c38268688069ddd9ec101532ea6f0253025f9df93c6d5e916968221232f8da00a0654ec1fadfb6c2d7849b96c26a1373e111cc6fd30c408ee833e0e2a89c4828f7a04a1eba1371dffabf57cd6f2a1774d2d464968546390a9f4dd78a76444cfce53580";
        let proof1 = x"f90211a0d2ad4e2cda383901cb101634058b0c28584f168817c8447237daeb3c9faf6a57a02aded885b60f7182faa84f45d253edfab7a4038ee77c0175a8a20c9ecdd5f7c1a0438af8d7678818cc1e6f8bd83fc257651d70271cb471aa44b8ae9ca1aaa39786a0461f00dce5db7e9aa3ea7809371894a533df231c19a7c560bc7dd8ed2da011bba0c7a35d543c85562f030230c6da9e7b14009763aede970ced5a7aa386efcc62aaa09ed4100cc66c040897a2f0ea8aad36b9e40a0a2d4c60ecd7e6f2fa1d0e9fe707a007dadece03c60c19d74e6b837b77016ec48bb2c03ee427f87a12fa2302414a70a099ea06c8a1ac2f55e4d92ee67d611246e4a5decbfacfb401280d36fdd7ed72f0a04314144712573463dd20917ec938c54c19d033345f4eefaa8176d5cc0947cd1ba0b186d1d4416d902040cd48bf76d66b9c07178a295eedd0b27dcdb9a1c62eaf3ea0725af34d18fa40a2bceb40044cc53887f6142ce85a117594880a961d7338ff5da0a5e885ec57fe93a8ea19d0a7d59978504369d1d70cc6725be2c3ce8269e8e43da03a9da344b108b810acdbb908a5126884db431ab6f92c1ae4ec8588878c9755e0a060725a3a4909300cb466cead03f96631829328cad205178b209075f93e3fba2aa088932d16240dba2d1e610ba21f8ae39eec93c7ea2b7f24cfaf5382b9473a6df7a07c8ef77d3c8bf3eba9410f21bd181cd5e3ab748853591bcb4590b369b51f8b6c80";
        let proof2 = x"f90211a0c3f832d8e98835a14ea9a03c00a01cbf29276bfb67651eac13718367a4ad0e76a0db6816cae273414a9a5df1737a56881974aed8c1d9b04cf1f507046a61cc7b1ba04063f66644dda617fe33e8d5ea79a099060be0b795b1bcae23f3385368176e0fa08dbda17ff3ac3ac7119738929d5b3e7b3693a838fdc619dfaa1c3edf56d2d471a0cf3b6bca6c7f17ba1850a73298fd39ee664da3477fec632db21b4dcd606e000aa026cee290ba68be291699ccfdf9a36b9c71095823fdd7a2728c3c7c4de7a5a5a7a078edcc7bd5823abc0d061a6161f08b8570a0ea1e18c37bd45daa2c8f92e66c79a05aa01ed340d6660dfbba7d83ed8f3f7c67051eacb094539f4c6e66f5380ac811a0607e85089c3c93110100f72589316c5ee05a294170b9c7c237ba3e6d7b43fe24a08f62d04a9f3f53bbc0eedbd6977ab1b310f95dfac18c474e18276d10e841eaf8a016faef66b158b706e9ef82cbce0af7633a07a692b4efb0c190a7b512bdd60d53a0cf3b7eec574b244719ec6e6839d77ae6729b601f08ed95005604a8db1ab00a09a0cd3c3095df97c9704ca69e39f5447e025dddde3a88dd627d6b53498e752b1222a0633ba144a3457f62fee4d360651e9bf74b47e586a0ea728f007c5384782d40bba05fa650d9a8e984e22122a9e0f784c05eaaf715a6c47b4e25ef11f5c76eaa4a7da0ee4d4150dc5bcb72e56b064379612bf9f98f737669ddb2dee0fc37e106b6f07480";
        let proof3 = x"f90211a05c977fc66d9a243988617d8342a5b16e5c699dad0d0fb41b9e2338eeb50d3243a0a384c741370fc093dcfd685ef64e17cd7f60620e3c5667944875c711e866f04fa0f1b3c8885165550d9c6210b68c06f403deec85c80ce416622704f9c4f2fb0d67a046f967f97da2767a0441a3b6e781d50a2d4c0bcc8486712f3fb2fbedaa33b656a0bb65d399adb76fdcf6b65a191886fb1371bcf24508f75a7cce35bd89a35117a5a0464f1500e7d54aa4947a16d0f6853db12cca3fb0d3e75b05b44d64ea3af6fcfca02fd9eca689c827f6583dadca743f4fb215b2098fd1fceb0ce2641d25191058d3a03d3507f62fc570fb59a2a6af53a0902ab6c09f7ae8553f81ff408d03fef60258a0cf027ae79de4be99b553c4107000c7dbbe0f86a8feceb0e84cf54e47e00676c1a0759bc87c6eebad2fa9ce8d3ab17926bae7140dc63e92ac9eff66b3e7c0bd0688a02b3a89578a034492fd59e93d0db0112ca9a3291ddcc631b7786edb10859d8015a0ca725677841d29a9acc1a1b61232d2d8332649b33ddd2160294901e20dac5e96a08f494c65b6346ca006ffbc13fce5089beb3f3e60b85091e598baef14abccead1a0dec06af2f069a106a419338744178e5b240302c17db4dc61a88c78ac6763f1eba0713ef37453d2d3c1fbe0a897c9f0757a3d1fdb53a20504aee155da26df70c117a0c4c286a1d37c2bb0a776d0895391e5dd89b1d902ac1526dedead2859faf8137d80";
        let proof4 = x"f89180a0b0ed89cc0fc540cb9cf910b35e5a017a0f171ef740c046716e23c35af3e668b380a05fff94dc1b65c002a713803772dc0ea531377d4e35f723163b8ee85a9747dc47a0f818819b52ccd3f4617889ab8480540cf7f2d591dade8b04a843d5ab3bb3781980808080808080a0e826c12966bfaf8019ae0e008dd2a972b92255bb7bab9a9283d6803f1de1a19d80808080";
        let proof5 = x"";
        let proofChain = vector::empty<u8>();

        vector::append(&mut proofChain, proof0);
        vector::append(&mut proofChain, proof1);
        vector::append(&mut proofChain, proof2);
        vector::append(&mut proofChain, proof3);
        vector::append(&mut proofChain, proof4);
        vector::append(&mut proofChain, proof5);

        let slot = x"0000000000000000000000000000000000000000000000000000000000000001";
        let key = keccak256(slot);


        let (is_exists, value) = verify_trie_value(&proofChain, &key, storage_root);
        assert(!is_exists, 1001);
    }

    #[test]
    public fun test_verify_okk() {
        let storage_root = x"195170ca4e76873504de92ee3651ba91e339555d9d008c5995e51c2c3ada74eb";
        let proof0 = x"f90211a0b51ceda38c7c0d96cee1d651d8c9001299aae0a56dd4778366faccf8c89802f0a011e1adf2007c6afdc9300271c03ad104cf9ed625a3cca7050416449175f7ef21a0e4187606d7baba63b37fd6978f264374e8d7289da084c4a56170ce1e438ff0f0a061869b1b76c51cc75983fc4792b3fc9c1c5e366a76149979920143afd2899770a0ae2ffd634be69d00ca955e55ad4bb4c1065d40938f82f56d678a87180087d2aba0dcfab65101c9968d7891a91ffc1d6c8bcda2773458d802feca923a7d938f7695a0c62fdc1d9731b77b5310a9a9e1bc9edb79976637f6f29c13ce49459ef7cdb7d5a0fce12c4968e940f0f7dbe888d359b81425bde60f261761608465fd74fa390828a04f77e522f007df2b5c6090006e531d113647900ef01ce8ddad6b6b908e786ce9a04beb43119c19f9f2b94738830b8ca07ce2cb40a2fc60e51567810deda9719527a05085bfa24339e17ba1305a8d7c93468ab8414fde3b1b0ce77ea3f196e16217eaa0071e1a46d2a544b7cc24d3153619887ab88606501aea6f30f03e084dab9da01aa0a27d98ca7583cd6f303c41747e5109978c3399cb632283a9a6d5300366bfc97ca0c38268688069ddd9ec101532ea6f0253025f9df93c6d5e916968221232f8da00a0654ec1fadfb6c2d7849b96c26a1373e111cc6fd30c408ee833e0e2a89c4828f7a04a1eba1371dffabf57cd6f2a1774d2d464968546390a9f4dd78a76444cfce53580";
        let proof1 = x"f90211a0e06a0657d0607ed2e2c32e879f439169a7fc4af77b35d9932dbeb2dfebe695c7a0a36e146bac35dbfac21f392c4030f374d6a749fbb09a17f61763374b758850f7a06984b50415a207367532fa5a6191f819b7ac6ef29164bc545b55d49397e2651fa022cc8e966d7c342d94abbe77dbcfb0a52b123f8117d78aa50463b8355acaadaaa0fcd4ae819a2addc899ddc0dda500f51bf61e2f20b3835c9ebd1011d62f28c934a0db9ed7b2486bb67a7971ae8c29683266ad9add781a1825de4e36c890e0f3cdf6a03b12d6c5b2b7211fbe8be70283bfdb3a382f85ec3db7f4e40733037b0d74dd7fa0b47f4e1076af0e9fe906587dd314896bcf4e496660b1f48a7fffd23f82b2ab5da098b00edd42f648defdd793c58f1d7f62ffa20f0b2b49073901496b9735e70e39a03adf726421bddaf8624147ca2ec8abc017e40ad77eada3157da77078ea9adf22a0e9ae74e8967516c78db4ac8aa7de5d80f02cd78b63213c3ba3357410df0a2e04a0cf56383af5dbdf2f6a1faae0681f81d00235ed137d5ac60e9ec0ff6ec8c37617a03826b3b060923bb30be9247355a3a7f570798bd1bdffeaf8c9100a1148f2071ca0a26f8f831f9939d92f85544dd55113f789a2042c5bf6adbf2f1fa261ad0d1266a0d07681f008065226a1ca369667ca08e1b6f76e92e0943d8075a25cb44c000814a01bcb25ed256b6742464e7229c04009279be5064d99ff5beb73361c5e79e6a55480";
        let proof2 = x"f90211a09dd4cacf70185dbb577b5c33f042e6f9c549c52331097b52258f214247891619a022dbeaa0beca42f9149bed094e43e8917ae33fb12f4f0ef73918174726a51145a0dc8b421a2dba882b708f4f919a5d150d5ac23098758b51e6582d89e32749d25ba0791a4ba7cd6cbbbe430be28fb6274d96493387c57701f7e93032db5fd149f45ca04d7fb99c8c354be9261724a4fa71c5be06132c600179725e2e5c86c60251da64a042356ecfc50f7b34d72e0dd7317f259fbbbe3f34db86baa21dcf35020f033ae5a01baf1a9894e8a1d9421e89c5b35f9cb1a97d85de2eeaca8cb2887e8acb41339ea0e81d4a406243c4093b54586b4ff46013065260f9e5ef4c00f582a813fc403ae9a0299b27a27116a228dfa8c54475ff83b5f6cb0b220193d3e725d7384c2540f697a0bc67389552138a30be44b93506db12b3c744a49d0cd305284ade2147f1a6d24ba0b15bbf046f439753f42359ecd4c0a9b9bbfc7839db828a683d64d0f1e0eb2609a0b145b111161cfe54b14a06c0d9166303c3531ce1b28fabda37bb65716fc0f0b5a081da4b16a391ec0c44c1008368daeb742f023e953cdb4c22cb125cf67c16c3b8a002ce122b0ece08a93c64c24462663e5671c832091aea0a549b8d97cfe17f5c6aa03a507cd55ea9be74639d78ade7135a6816f677c85e29d9e575ee44b3b470bbe2a05049a431f1ce759fb5fe0053c1ceaa64fc3c3b69a4e936502946080a452e0bbc80";
        let proof3 = x"f90211a03600c8c217ed73076745dc695be6dd82dae247c6f8a1ab5a54c138a05430847ea0f1c001ada8d78660bd4e76f7e826d851c5cc8286f86d625be87b34b4f751be29a09a0cca8adf25996197bac969a6b51ff5c4277ef98d63a104c1d36a78c71f1e34a0ff0f8508adb6649051f05bee1918e4e4369ad69ddf75ee8bf85617cd5092ffdda01af98a2f12fb227c56da4fde86d2728154774385f9d4a3439c31b4c58e7fe69fa04303c4a3a0718a2d0802aba9c2f79d60b3da50ae58a66ca96d19d12ee1dad10fa066c67418b17b99c11716aa93dd2f37948f335f5028467deb592c5a760f406513a0b879342a5e6e0f0fcd9782bede64a229619485c1f00f7358fb4fb5a09277bc8ea0d3af6303bae36da3f7c9a77027b3cdbb752efe0ece4df34f12bc95f8b2a2c982a0685aea1196bd14056d1fe4d1ab6376239a2765865bec5e6a58a8ed3cf3687beba0f60c3da6c4bbc68bc02ae62ed14a53c6b456fb39f9a6a29192a3280eba80ca83a07eb4d7ee61fac4cd7d6b60ebd40e39ab67d149f92092fb4544886fcb2c129d98a089165c6c4ec484255323338862b5b4f84c1a425a0544a6197a44de209545163aa0969c546e88a081325e4e94790d16a42b737fcd4449f3ab5cf0357a369e50b1cfa0d4ef7c6ba9e64d60179b5a7cb84b678c9374b232d60edd14b42bcc856950726ca0d0a3de051337fee25685748612adde9bebc1acff896d2142eef25cd85e69b5c380";
        let proof4 = x"f8b180808080a0595f19b886413b654f6ec0fc17933985ec962dc71f526bbb58111ce8a6169f5d80808080a00803d929cf7dd0abafcc85912206b2f29b3c3d39b1042def65b0891c3891ef0d80a0a1793298225c34075209c538c84c18c39760df1b851cccb42b039a83e612469ca0b63459eca6ea6705c6b4067115a73202bba37f32db40d5df642138f46f98195980a08c14912091a7a7a1b45434498d91c48461dc50aff8895b0402f7c950b2c4f2178080";
        let proof5 = x"f8518080808080a03073e62a78a3f9f4d405308b3da3311763019d81f315ea983561f9cebd783b18808080808080a099d0f432e9ce1cb35f07c66e4936a06134dc48b4d43d4c058fe17dc0ecb281ff80808080";
        let proof6 = x"e09e20d9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e56301";
        let proofChain = vector::empty<u8>();

        vector::append(&mut proofChain, proof0);
        vector::append(&mut proofChain, proof1);
        vector::append(&mut proofChain, proof2);
        vector::append(&mut proofChain, proof3);
        vector::append(&mut proofChain, proof4);
        vector::append(&mut proofChain, proof5);
        vector::append(&mut proofChain, proof6);

        let slot = x"0000000000000000000000000000000000000000000000000000000000000000";
        let key = keccak256(slot);

        let (is_exists, value) = verify_trie_value(&proofChain, &key, storage_root);
        assert(is_exists, 1001);
        assert(value == x"01", 1002);
    }

    #[test]
    public fun test_verify_ok2() {
        let storage_root = x"64bec87c43e402ed2648ae3e10c9ba5d980ac30ae0dfcc4c90a47856380ce76a";
        let proof0 = x"f90211a0290ff9c2465abdc3e521b0e22d434ca9965d9294f984c4af27b62defa7aa0404a0681afeef44df0f0f3ff44a1fc6b6b1c1b5b3ddf1df4b8334184ac69e06d663cea0e7d87d908639d88cccb5e82139e6969ef7a60ef15f2c1b92a42721c00a684534a0cb0d69ffacac2472aba8113fabe43ae0fb1ec1adc0ba524b4d77a4ae1b9f1834a0d8c1a0faa0ee7b3d651997d9bed61cd1a38fdd1d5811d0f6f35135d505772271a055d3a97b39c767db94b3a1ec2cd527ecee17b2c48b05e478c846f74e8c4b0770a07f84fae77d495ad51e1754ca932a17967af94e0ab56da206569bf581b86ff1a3a0444fdf31592bedd27a9525245ce36aa23bb53767574d2ddf7db0c8ed649b7d08a02724a8992048374ba00a4381a1a0c44a10a3863647977f61e668de5532ae10eda0ceff06700cbb9dc8b2a95604aa18ec3863877f64b0101fdeba6fff45aa220e98a02a3280086775de99c51785b0281459bce312b7231bcd59a030dc3c277bb29854a03506c7687acc02c53b15bb4c15cea1cb1065b7a24a1a931200bd117d88dee84ea055759451409a66a368f9ee9bce914924080954448b54ee7e849fabdd7d5d4124a0fdfe38b6023fb6e7a4728a07872f191a0b173ca2b2f2c9dac4e46b1478594903a03bdaa97bd901df14cad52a8369a832f766f1da1877c72c26dff87e4f8eedf73fa02d69cf9241410b8bd737972cbf4675a8bfdce0f1ce923a7a5c8209579ba0b55180";
        let proof1 = x"f901518080a0fe0849c9829308dfeebb656b80c84fd25cddad6195e55da1759fb534872d0565a0492b4ed07e3463e8a3f2280e664efe6dcd914e8b6f96dd457b2fb1514fa4dcf3a0136bc83176214e5c162c0d0ee80ec5c99bb74b612de9d31651547629a6d3bff680a0b03da2ced67fe1e95e2d166e4faeac40f4deb62242e768646e20302e989ab6eb80a06507648f5ee64cf12436e03298b41203bf3bb7344ef267853802bf97fd9b48cba093f776423813a1ca4b75baa37b23e49a02898a68961e357d9e8064663d5bb20380a0450922b2b63e417a5397d6f4346828778a5d58bdf292948e1c542a1ed0319ee480a0d76251a116716185de4f499c9934deb89fb9bfecbd85d4e6d8fa268958fd4eb8a0cc6759e97e0d6b2947385d706a65b69f15512add39d2163b3cbcfd427307df87a036cee042c4ab7e473bfb673d9a8f99eb6d86ad267803005fdeb57eb636121ad180";
        let proof2 = x"f843a020d71926b1d4cc00b9747141c15cf96296e56262d843136a42daf00aca967037a1a0faadeddd9e83b87f941ff7ac6c1ff3a55a976f082f579d64ca49253295321ca6";
        let proofChain = vector::empty<u8>();

        let expected_valude = encode_uint(113385518376749189221566347534743733501213541687712268135309701180845563452582);
        vector::append(&mut proofChain, proof0);
        vector::append(&mut proofChain, proof1);
        vector::append(&mut proofChain, proof2);

        let slot = x"91da3fd0782e51c6b3986e9e672fd566868e71f3dbc2d6c2cd6fbb3e361af2a70000000000000000000000000000000000000000000000000000000000000000";
        let key = keccak256(slot);
        let key = keccak256(key);

        let (is_exists, value) = verify_trie_value(&proofChain, &key, storage_root);
    }


    #[test]
    public fun test_parse_hash() {
        let buf: vector<u8> = x"334455667733445566112233334455667733445566112233334455667733445544";

        let (result, offset) = parse_hash(&buf);
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

        let buf: vector<u8> = x"9e20d9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e56301";
        let (result, res) = split_bytes(&buf);
        std::debug::print(&string::utf8(b"result: "));
        std::debug::print(&result);
        assert!(result == x"20d9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563", 1);
        assert!(res == x"01", 1);
    }

    #[test]
    public fun test_encode_uint() {
        let result = encode_uint(1234455);
        assert!(result == x"8312d617", 1);

        let result = encode_uint(14);
        assert!(result == x"0e", 1);

        let result = encode_uint(1234455321122545);
        assert!(result == x"870462bb06e816f1", 1);

        let result = encode_uint(113385518376749189221566347534743733501213541687712268135309701180845563452582);
        assert!(result == x"a0faadeddd9e83b87f941ff7ac6c1ff3a55a976f082f579d64ca49253295321ca6", 1);
    }

}