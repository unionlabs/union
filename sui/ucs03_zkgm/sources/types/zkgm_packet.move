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

module zkgm::zkgm_packet {
    use zkgm::zkgm_ethabi;
    use zkgm::instruction::{Self, Instruction};

    public struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u256,
        instruction: Instruction
    }

    public fun salt(zkgm_pack: &ZkgmPacket): vector<u8> {
        zkgm_pack.salt
    }

    public fun path(zkgm_pack: &ZkgmPacket): u256 {
        zkgm_pack.path
    }

    public fun instruction(zkgm_pack: &ZkgmPacket): Instruction {
        zkgm_pack.instruction
    }

    public fun new(salt: vector<u8>, path: u256, instruction: Instruction): ZkgmPacket {
        ZkgmPacket { salt, path, instruction }
    }

    public fun encode(packet: &ZkgmPacket): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_bytes32(&mut buf, &packet.salt);
        zkgm_ethabi::encode_uint<u256>(&mut buf, packet.path);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x60);

        vector::append(&mut buf, instruction::encode(&packet.instruction));

        buf
    }

    public fun decode(buf: &vector<u8>): ZkgmPacket {
        let mut index = 0;

        ZkgmPacket {
            salt: zkgm_ethabi::decode_bytes32(buf, &mut index),
            path: zkgm_ethabi::decode_uint(buf, &mut index),
            instruction: instruction::decode(buf, &mut (index + 0x20)),
        }
    }

    #[test]
    fun test_encode_decode() {
        let encoded =
            x"414141414141414141414141414141414141414141414141414141414141414100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000084141414141414141000000000000000000000000000000000000000000000000";
        let packet = decode(&encoded);
        let expected_packet =
            new(
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                100,
                instruction::new(10, 20, b"AAAAAAAA")
            );

        assert!(packet == expected_packet, 1);
        assert!(encode(&packet) == encoded, 1);
    }
}
