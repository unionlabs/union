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

module zkgm::acknowledgement {
    use zkgm::zkgm_ethabi;

    use std::vector;

    struct Acknowledgement has copy, drop, store {
        tag: u256,
        inner_ack: vector<u8>
    }

    public fun new(tag: u256, inner_ack: vector<u8>): Acknowledgement {
        Acknowledgement { tag, inner_ack }
    }

    public fun tag(ack: &Acknowledgement): u256 {
        ack.tag
    }

    public fun inner_ack(ack: &Acknowledgement): &vector<u8> {
        &ack.inner_ack
    }

    public fun encode(ack: &Acknowledgement): vector<u8> {
        let buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u256>(&mut buf, ack.tag);

        let version_offset = 0x40;
        zkgm_ethabi::encode_uint<u32>(&mut buf, version_offset);
        zkgm_ethabi::encode_bytes(&mut buf, &ack.inner_ack);

        buf
    }

    public fun decode(buf: &vector<u8>): Acknowledgement {
        let index = 0x0;
        let tag = zkgm_ethabi::decode_uint(buf, &mut index);
        index = index + 0x20;
        let inner_ack = zkgm_ethabi::decode_bytes(buf, &mut index);
        Acknowledgement { tag: tag, inner_ack: inner_ack }
    }

    #[test]
    fun test_encode_decode_ack() {
        let output =
            x"000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000768656c6c6c6f6f00000000000000000000000000000000000000000000000000";
        let ack_data = Acknowledgement { tag: 7788909223344, inner_ack: b"hellloo" };

        let ack_bytes = encode(&ack_data);
        std::debug::print(&ack_bytes);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode(&ack_bytes);
        assert!(ack_data_decoded.tag == 7788909223344, 1);
        assert!(ack_data_decoded.inner_ack == b"hellloo", 3);
    }

    #[test]
    fun test_decode_ack() {
        let output =
            x"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000";

        let ack_data_decoded = decode(&output);
        std::debug::print(&ack_data_decoded);
    }
}
