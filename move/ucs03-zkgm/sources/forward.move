module zkgm::forward {
    use zkgm::ethabi;
    use zkgm::instruction::{Self, Instruction};

    use std::vector;
    use std::string::{Self, String};

    struct Forward has copy, drop, store {
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        instruction: Instruction
    }

    public fun channel_id(forward: &Forward): u32 {
        forward.channel_id
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
        let buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u32>(&mut buf, forward.channel_id);
        ethabi::encode_uint<u64>(&mut buf, forward.timeout_height);
        ethabi::encode_uint<u64>(&mut buf, forward.timeout_timestamp);
        ethabi::encode_uint<u8>(&mut buf, 0x80);
        let ins_buf = instruction::encode(&forward.instruction);
        vector::append(&mut buf, ins_buf);
        buf
    }

    public fun decode(buf: &vector<u8>, index: &mut u64): Forward {
        let channel_id = ethabi::decode_uint(buf, index);
        let timeout_height = ethabi::decode_uint(buf, index);
        let timeout_timestamp = ethabi::decode_uint(buf, index);
        *index = *index + 0x20;

        let version = (ethabi::decode_uint(buf, index) as u8);
        let opcode = (ethabi::decode_uint(buf, index) as u8);
        *index = *index + 0x20;
        let operand = ethabi::decode_bytes(buf, index);

        let instruction = instruction::new(version, opcode, operand);
        
        Forward {
            channel_id: (channel_id as u32),
            timeout_height: (timeout_height as u64),
            timeout_timestamp: (timeout_timestamp as u64),
            instruction: instruction
        }
    }

    #[test]
    fun test_encode_decode_forward_packet() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003700000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";

        let instruction = instruction::new(111, 222, b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world");

        let forward_data = Forward {
            channel_id: 44,
            timeout_height: 55,
            timeout_timestamp: 66,
            instruction: instruction
        };

        let ack_bytes = encode(&forward_data);
        std::debug::print(&string::utf8(b"ack bytes: "));
        std::debug::print(&ack_bytes);
        assert!(ack_bytes == output, 0);

        let decode_idx = 0x20;
        let forward_data_decoded = decode(&ack_bytes, &mut decode_idx);
        assert!(forward_data_decoded.channel_id == forward_data.channel_id, 0);
        assert!(forward_data_decoded.timeout_height == forward_data.timeout_height, 1);
        assert!(
            forward_data_decoded.timeout_timestamp == forward_data.timeout_timestamp, 2
        );
        assert!(forward_data_decoded.instruction == forward_data.instruction, 3);
    }
}
