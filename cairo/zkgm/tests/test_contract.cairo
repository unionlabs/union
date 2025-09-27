use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use zkgm::types::{Forward, Instruction, Opcode, Version, ZkgmPacket, ethabi_decode, ethabi_encode};

#[test]
fn test_instruction_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000004);
    encoded.append_u256(0x4141414100000000000000000000000000000000000000000000000000000000);

    let instruction: Instruction = ethabi_decode(encoded.clone()).unwrap();

    let i = Instruction { version: Version::V2, opcode: Opcode::TokenOrder, operand: "AAAAAAAA" };

    assert!(i == instruction);

    assert!(ethabi_encode(@instruction) == encoded);
}

#[test]
fn test_zkgm_packet_encode_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x4141414141414141414141414141414141414141414141414141414141414141);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000064);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000008);
    encoded.append_u256(0x4141414141414141000000000000000000000000000000000000000000000000);

    let packet: ZkgmPacket = ethabi_decode(encoded.clone()).unwrap();

    let p = ZkgmPacket {
        salt: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        path: 100,
        instruction: Instruction {
            version: Version::V2, opcode: Opcode::TokenOrder, operand: "AAAAAAAA",
        },
    };
    assert!(packet == p);

    assert!(encoded == ethabi_encode(@packet));
}

#[test]
fn test_forward_encode_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000064);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000000);
    encoded.append_u256(0x00000000000000000000000000000000000000000000000000000000000186a0);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000080);
    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000008);
    encoded.append_u256(0x4141414141414141000000000000000000000000000000000000000000000000);

    let forward: Forward = ethabi_decode(encoded.clone()).unwrap();

    let f = Forward {
        path: 100,
        timeout_height: 0,
        timeout_timestamp: 100000,
        instruction: Instruction {
            version: Version::V2, opcode: Opcode::TokenOrder, operand: "AAAAAAAA",
        },
    };
    assert!(forward == f);
    assert!(encoded == ethabi_encode(@forward));
}

#[test]
fn test_address_prediction_works() {}

