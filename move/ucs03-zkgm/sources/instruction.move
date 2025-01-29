module zkgm::instruction {
    use zkgm::ethabi;

    use std::vector;

    struct Instruction has copy, drop, store {
        version: u8,
        opcode: u8,
        operand: vector<u8>
    }

    public fun new(version: u8, opcode: u8, operand: vector<u8>): Instruction {
        Instruction {
            version,
            opcode,
            operand
        }        
    }

    public fun version(instruction: &Instruction): u8 {
        instruction.version
    }

    public fun opcode(instruction: &Instruction): u8 {
        instruction.opcode
    }

    public fun operand(instruction: &Instruction): &vector<u8> {
        &instruction.operand
    }

    public fun encode(instruction: &Instruction): vector<u8> {
        let buf = vector::empty<u8>();

        ethabi::encode_uint<u8>(&mut buf, instruction.version);
        ethabi::encode_uint<u8>(&mut buf, instruction.opcode);
        ethabi::encode_uint<u8>(&mut buf, 0x60);
        ethabi::encode_bytes(&mut buf, &instruction.operand);

        buf
    }

    public fun decode(buf: &vector<u8>, index: &mut u64): Instruction {
        let version = (ethabi::decode_uint(buf, index) as u8);
        let opcode = (ethabi::decode_uint(buf, index) as u8);
        // skipping the pointer
        *index = *index + 0x20;
        let operand = ethabi::decode_bytes(buf, index);
        new(version, opcode, operand)
    }

    #[test]
    fun test_encode_instruction() {
        let encoded = x"000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000044141414100000000000000000000000000000000000000000000000000000000";
        let index = 0;
        let instruction = decode(&encoded, &mut index);
        let expected_instruction = new(10, 20, b"AAAA");
        
        assert!(instruction == expected_instruction, 1);
        assert!(encode(&instruction) == encoded, 1);
    }
}
