use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use zkgm::types::{Instruction, InstructionTrait, ZkgmPacket, ethabi_decode, ethabi_encode};

#[test]
fn test_instruction_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000004);
    encoded.append_u256(0x4141414100000000000000000000000000000000000000000000000000000000);

    let instruction: Instruction = ethabi_decode(encoded.clone().into()).unwrap();

    let i = Instruction { version: 10, opcode: 20, operand: "AAAA" };

    assert!(i == instruction);

    assert!(ethabi_encode(@instruction) == encoded);
}

#[test]
fn test_zkgm_encode_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x4141414141414141414141414141414141414141414141414141414141414141);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000064);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000008);
    encoded.append_u256(0x4141414141414141000000000000000000000000000000000000000000000000);

    let packet: ZkgmPacket = ethabi_decode(encoded.clone().into()).unwrap();

    let p = ZkgmPacket {
        salt: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        path: 100,
        instruction: Instruction { version: 10, opcode: 20, operand: "AAAAAAAA" },
    };
    assert!(packet == p);

    assert!(encoded == ethabi_encode(@packet));
}
