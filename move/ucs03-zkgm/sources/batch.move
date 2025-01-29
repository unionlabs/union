module zkgm::batch {
    use zkgm::ethabi;
    use zkgm::instruction::{Instruction, Self};

    use std::vector;

    struct Batch has copy, drop, store {
        instructions: vector<Instruction>
    }

    public fun new(version: u8, opcode: u8, instructions: vector<Instruction>): Batch {
        Batch {
            instructions
        }        
    }

    public fun instructions(batch: &Batch): vector<Instruction> {
        batch.instructions
    }

    public fun encode(pack: &Batch): vector<u8> {
        let buf = vector::empty<u8>();

        ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&pack.instructions);
        ethabi::encode_uint<u64>(&mut buf, ack_arr_len);

        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                let instructions_encoded =
                    instruction::encode(vector::borrow(&pack.instructions, 0));
                vector::append(&mut buf, instructions_encoded);

                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let idx = 1;
        let prev_val = initial_stage;
        ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let curr_instruction = vector::borrow(&pack.instructions, idx - 1);
            let prev_length =
                ((
                    vector::length(instruction::operand(curr_instruction))
                        / 32
                ) as u32) + 1;
            ethabi::encode_uint<u32>(
                &mut buf,
                prev_val + (0x20 * 4) + ((prev_length * 0x20) as u32)
            );
            prev_val = prev_val + (4 * 0x20) + (((prev_length * 0x20) as u32));
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            let instructions_encoded =
                instruction::encode(vector::borrow(&pack.instructions, idx));
            vector::append(&mut buf, instructions_encoded);

            idx = idx + 1;
        };

        buf
    }

    public fun decode(buf: &vector<u8>, index: &mut u64): Batch {
        let main_arr_length = ethabi::decode_uint(buf, index);
        *index = *index + (0x20 * main_arr_length as u64);

        let idx = 0;
        let instructions = vector::empty();
        while (idx < main_arr_length) {
            let version = (ethabi::decode_uint(buf, index) as u8);
            let opcode = (ethabi::decode_uint(buf, index) as u8);
            *index = *index + 0x20;
            let operand = ethabi::decode_bytes(buf, index);

            let instruction = instruction::new((version as u8), (opcode as u8), operand);

            vector::push_back(&mut instructions, instruction);
            idx = idx + 1;
        };

        Batch { instructions: instructions }
    }

    #[test]
    fun test_encode_decode() {
        let decode_idx = 0x20;
        // ---------------- TEST 1 ----------------
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002668686820776f726c6468656c6c6f20777777776c6f20776f726c6468656c6c6f20776f726c64000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000370000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000086272726168686868000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();

        let instruction1 = instruction::new(111, 222, b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world");
        let instruction2 = instruction::new(1, 2, b"hhh worldhello wwwwlo worldhello world");
        let instruction3 = instruction::new(55, 66, b"brrahhhh");

        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        vector::push_back(&mut outer_arr, instruction3);
        let ack_data = Batch { instructions: outer_arr };
        let ack_bytes = encode(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode(&ack_bytes, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.instructions) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 2) == instruction3, 4);

        // ---------------- TEST 2 ----------------
        let decode_idx = 0x20;
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000162000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();

        let instruction1 = instruction::new(3, 5, b"b");
        let instruction2 = instruction::new(2, 4, b"");

        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        let ack_data2 = Batch { instructions: outer_arr };
        let ack_bytes2 = encode(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode(&ack_bytes2, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.instructions) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);

        // ---------------- TEST 3 ----------------
        let decode_idx = 0x20;
        let output3 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000000df000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000bd617764617764617764617764776164616161616161612061616161616161616161616161616161616161616120626262622064616477647720772077777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777000000";
        let outer_arr = vector::empty();

        let instruction1 = instruction::new(123, 223, b"awdawdawdawdwadaaaaaaa aaaaaaaaaaaaaaaaaaaaa bbbb dadwdw w wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww");

        vector::push_back(&mut outer_arr, instruction1);

        let ack_data3 = Batch { instructions: outer_arr };
        let ack_bytes3 = encode(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode(&ack_bytes3, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.instructions) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);

        // ---------------- TEST 4 ----------------
        let decode_idx = 0x20;
        let output4 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let outer_arr = vector::empty();

        let ack_data4 = Batch { instructions: outer_arr };
        let ack_bytes4 = encode(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode(&ack_bytes4, &mut decode_idx);
        assert!(vector::length(&ack_data_decoded.instructions) == 0, 1);

    }

}
