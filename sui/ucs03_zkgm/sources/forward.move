// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
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

module zkgm::forward {
    use zkgm::zkgm_ethabi;
    use zkgm::instruction::{Self, Instruction};

    public struct Forward has copy, drop, store {
        path: u256,
        timeout_height: u64,
        timeout_timestamp: u64,
        instruction: Instruction
    }

    public fun new(
        path: u256,
        timeout_height: u64,
        timeout_timestamp: u64,
        instruction: Instruction
    ): Forward {
        Forward {
            path,
            timeout_height,
            timeout_timestamp,
            instruction,
        }
    }

    public fun path(forward: &Forward): u256 {
        forward.path
    }

    public fun timeout_height(forward: &Forward): u64 {
        forward.timeout_height
    }

    public fun timeout_timestamp(forward: &Forward): u64 {
        forward.timeout_timestamp
    }

    public fun instruction(forward: &Forward): &Instruction {
        &forward.instruction
    }

    public fun encode(forward: &Forward): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u256>(&mut buf, forward.path);
        zkgm_ethabi::encode_uint<u64>(&mut buf, forward.timeout_height);
        zkgm_ethabi::encode_uint<u64>(&mut buf, forward.timeout_timestamp);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x80);
        let ins_buf = instruction::encode(&forward.instruction);
        vector::append(&mut buf, ins_buf);
        buf
    }

    public fun decode(buf: &vector<u8>): Forward {
        let mut index = 0;
        let path = zkgm_ethabi::decode_uint(buf, &mut index);
        let timeout_height = zkgm_ethabi::decode_uint(buf, &mut index);
        let timeout_timestamp = zkgm_ethabi::decode_uint(buf, &mut index);
        index = index + 0x20;

        Forward {
            path: (path as u256),
            timeout_height: (timeout_height as u64),
            timeout_timestamp: (timeout_timestamp as u64),
            instruction: instruction::decode(buf, &mut index)
        }
    }

    #[test]
    fun test_encode_decode_forward_packet() {
        let output =
            x"000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003700000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";

        let instruction =
            instruction::new(
                111,
                222,
                b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world"
            );

        let forward_data = Forward {
            path: 44,
            timeout_height: 55,
            timeout_timestamp: 66,
            instruction: instruction
        };

        let ack_bytes = encode(&forward_data);
        assert!(ack_bytes == output, 0);

        let forward_data_decoded = decode(&ack_bytes);
        assert!(forward_data_decoded.path == forward_data.path, 0);
        assert!(forward_data_decoded.timeout_height == forward_data.timeout_height, 1);
        assert!(
            forward_data_decoded.timeout_timestamp == forward_data.timeout_timestamp, 2
        );
        assert!(forward_data_decoded.instruction == forward_data.instruction, 3);
    }
}
