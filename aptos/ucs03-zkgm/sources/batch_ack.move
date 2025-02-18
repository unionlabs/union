// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module zkgm::batch_ack {
    use zkgm::zkgm_ethabi;

    use std::vector;

    struct BatchAck has copy, drop, store {
        acknowledgements: vector<vector<u8>>
    }

    public fun new(acknowledgements: vector<vector<u8>>): BatchAck {
        BatchAck { acknowledgements }
    }

    public fun acknowledgements(batch_ack: &BatchAck): vector<vector<u8>> {
        batch_ack.acknowledgements
    }

    public fun encode(ack: &BatchAck): vector<u8> {
        let buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.acknowledgements);
        zkgm_ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                zkgm_ethabi::encode_vector<u8>(
                    &mut buf,
                    vector::borrow(&ack.acknowledgements, 0),
                    |some_variable, data| {
                        zkgm_ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let idx = 1;
        let prev_val = initial_stage;
        zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(
                vector::borrow(&ack.acknowledgements, idx - 1)
            );
            zkgm_ethabi::encode_uint<u32>(
                &mut buf, prev_val + 0x20 * (prev_length + 1 as u32)
            );
            prev_val = prev_val + 0x20 * (prev_length + 1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            zkgm_ethabi::encode_vector<u8>(
                &mut buf,
                vector::borrow(&ack.acknowledgements, idx),
                |some_variable, data| {
                    zkgm_ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode(buf: &vector<u8>, index: &mut u64): BatchAck {
        let main_arr_length = zkgm_ethabi::decode_uint(buf, index);
        *index = *index + (0x20 * main_arr_length as u64);

        let idx = 0;
        let acknowledgements = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec =
                zkgm_ethabi::decode_vector<u8>(
                    buf,
                    index,
                    |buf, index| {
                        (zkgm_ethabi::decode_uint(buf, index) as u8)
                    }
                );
            vector::push_back(&mut acknowledgements, inner_vec);
            idx = idx + 1;
        };
        BatchAck { acknowledgements: acknowledgements }
    }

    #[test]
    fun test_encode_decode() {
        let decode_idx = 0x40;
        // ---------------- TEST 1 ----------------
        let output =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065";
        let outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        vector::push_back(&mut outer_arr, b"hehe");
        let ack_data = BatchAck { acknowledgements: outer_arr };
        let ack_bytes = encode(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode(&ack_bytes, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 2) == b"hehe", 4);

        // ---------------- TEST 2 ----------------
        let decode_idx = 0x40;
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000069";
        let outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        let ack_data2 = BatchAck { acknowledgements: outer_arr };
        let ack_bytes2 = encode(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode(&ack_bytes2, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);

        // ---------------- TEST 3 ----------------
        let decode_idx = 0x40;
        let output3 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000360000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000007300000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();
        let idx = 0;
        vector::push_back(&mut outer_arr, b"xdddd");
        vector::push_back(&mut outer_arr, b"test");
        while (idx < 10) {
            vector::push_back(&mut outer_arr, b"");
            idx = idx + 1;
        };

        let ack_data3 = BatchAck { acknowledgements: outer_arr };
        let ack_bytes3 = encode(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode(&ack_bytes3, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 12, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"test", 3);

        // ---------------- TEST 4 ----------------
        let decode_idx = 0x40;
        let output4 =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064";
        let outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"xdddd");

        let ack_data4 = BatchAck { acknowledgements: outer_arr };
        let ack_bytes4 = encode(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode(&ack_bytes4, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);

        // ---------------- TEST 5 ----------------
        let decode_idx = 0x40;
        let output5 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();

        let ack_data5 = BatchAck { acknowledgements: outer_arr };
        let ack_bytes5 = encode(&ack_data5);
        assert!(ack_bytes5 == output5, 0);
        let ack_data_decoded = decode(&ack_bytes5, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 0, 1);

    }
}
