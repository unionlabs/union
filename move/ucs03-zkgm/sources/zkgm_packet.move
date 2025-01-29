module zkgm::zkgm_packet {
    use zkgm::ethabi;
    use zkgm::instruction::{Self, Instruction};

    use std::vector;

    struct ZkgmPacket has copy, drop, store {
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
        ZkgmPacket {
            salt,
            path,
            instruction
        }
    }

    public fun encode(packet: &ZkgmPacket): vector<u8> {
        let buf = vector::empty<u8>();
        ethabi::encode_bytes32(&mut buf, &packet.salt);
        ethabi::encode_uint<u256>(&mut buf, packet.path);
        ethabi::encode_uint<u8>(&mut buf, 0x60);

        let ins_buf = instruction::encode(&packet.instruction);
        vector::append(&mut buf, ins_buf);

        buf
    }

    public fun decode(buf: &vector<u8>): ZkgmPacket {
        let index = 0;
        let salt = ethabi::decode_bytes32(buf, &mut index);
        let path = ethabi::decode_uint(buf, &mut index);
        // skipping the pointer
        index = index + 0x20;
        let instruction = instruction::decode(buf, &mut index);

        ZkgmPacket { salt: salt, path: path, instruction: instruction }
    }

    #[test]
    fun test_encode_zkgm_packet() {
        let encoded = x"414141414141414141414141414141414141414141414141414141414141414100000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000084141414141414141000000000000000000000000000000000000000000000000";
        let packet = decode(&encoded);
        let expected_packet = new(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", 100, instruction::new(10, 20, b"AAAAAAAA"));

        assert!(packet == expected_packet, 1);
        assert!(encode(&packet) == encoded, 1);
    }
}
