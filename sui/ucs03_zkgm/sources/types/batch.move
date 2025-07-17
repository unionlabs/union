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

module zkgm::batch {
    use zkgm::zkgm_ethabi;
    use zkgm::instruction::{Instruction, Self};

    public struct Batch has copy, drop, store {
        instructions: vector<Instruction>
    }

    public fun new(instructions: vector<Instruction>): Batch {
        Batch { instructions }
    }

    public fun instructions(batch: &Batch): vector<Instruction> {
        batch.instructions
    }

    public fun encode(batch: &Batch): vector<u8> {
        let mut buf = vector::empty();
        zkgm_ethabi::encode_uint(&mut buf, 0x20);
        zkgm_ethabi::encode_dyn_array!(
            &mut buf,
            &batch.instructions,
            |b, instr| {
                b.append(instr.encode());
            }
        );

        buf
    }

    #[allow(unused_mut_ref)]
    public fun decode(buf: &vector<u8>): Batch {
        Batch {
            instructions: zkgm_ethabi::decode_dyn_array!(
                buf,
                &mut 0x20,
                |b| {
                    let mut i = 0;
                    instruction::decode(&b, &mut i)
                }
            )
        }
    }

    #[test]
    fun test_encode_decode() {
        // ---------------- TEST 1 ----------------
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002668686820776f726c6468656c6c6f20777777776c6f20776f726c6468656c6c6f20776f726c64000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000370000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000086272726168686868000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let instruction1 =
            instruction::new(
                111,
                222,
                b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world"
            );
        let instruction2 = instruction::new(
            1, 2, b"hhh worldhello wwwwlo worldhello world"
        );
        let instruction3 = instruction::new(55, 66, b"brrahhhh");

        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        vector::push_back(&mut outer_arr, instruction3);
        let ack_data = Batch { instructions: outer_arr };
        let ack_bytes = encode(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode(&ack_bytes);
        assert!(vector::length(&ack_data_decoded.instructions) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 2) == instruction3, 4);

        // ---------------- TEST 2 ----------------
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000162000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let instruction1 = instruction::new(3, 5, b"b");
        let instruction2 = instruction::new(2, 4, b"");

        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        let ack_data2 = Batch { instructions: outer_arr };
        let ack_bytes2 = encode(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode(&ack_bytes2);
        assert!(vector::length(&ack_data_decoded.instructions) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);

        // ---------------- TEST 3 ----------------
        let output3 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000000df000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000bd617764617764617764617764776164616161616161612061616161616161616161616161616161616161616120626262622064616477647720772077777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777000000";
        let mut outer_arr = vector::empty();

        let instruction1 =
            instruction::new(
                123,
                223,
                b"awdawdawdawdwadaaaaaaa aaaaaaaaaaaaaaaaaaaaa bbbb dadwdw w wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww"
            );

        vector::push_back(&mut outer_arr, instruction1);

        let ack_data3 = Batch { instructions: outer_arr };
        let ack_bytes3 = encode(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode(&ack_bytes3);
        assert!(vector::length(&ack_data_decoded.instructions) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);

        // ---------------- TEST 4 ----------------
        let output4 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();

        let ack_data4 = Batch { instructions: outer_arr };
        let ack_bytes4 = encode(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode(&ack_bytes4);
        assert!(vector::length(&ack_data_decoded.instructions) == 0, 1);

    }
}
