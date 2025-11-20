use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use zkgm::types::{Instruction, TestBro, ZkgmPacket, ZkgmPacketTrait, ethabi_decode};

#[test]
fn test_instruction_decode() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x000000000000000000000000000000000000000000000000000000000000000a);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000014);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000060);
    encoded.append_u256(0x4141414100000000000000000000000000000000000000000000000000000000);
    encoded.append_u256(0x4141414100000000000000000000000000000000000000000000000000000000);
    encoded.append_u256(0x4141414100000000000000000000000000000000000000000000000000000000);

    let instruction: Instruction = ethabi_decode(encoded.into()).unwrap();

    println!("instruction: {:?}", instruction);
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

    // let encoded = ZkgmPacket {
    //     salt: Default::default(),
    //     path: 100,
    //     instruction: Instruction { version: 1, opcode: 2, operand: Default::default() },
    // }
    //     .encode();

    let packet: ZkgmPacket = ethabi_decode(encoded.into()).unwrap();

    println!("packet: {:?}", packet);
}

#[test]
fn test_bro() {
    let mut encoded: ByteArray = Default::default();

    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000001);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000002);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000003);
    encoded.append_u256(0x0000000000000000000000000000000000000000000000000000000000000004);

    // let encoded = ZkgmPacket {
    //     salt: Default::default(),
    //     path: 100,
    //     instruction: Instruction { version: 1, opcode: 2, operand: Default::default() },
    // }
    //     .encode();

    let packet: TestBro = ethabi_decode(encoded.into()).unwrap();

    println!("packet: {:?}", packet);
}
