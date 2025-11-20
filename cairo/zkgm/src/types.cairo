use alexandria_evm::decoder::AbiDecodeTrait;
use alexandria_evm::encoder::{AbiEncodeTrait, EVMCalldata};
use alexandria_evm::evm_enum::EVMTypes;
use alexandria_evm::evm_struct::EVMCalldata as StructEVMCalldata;
use core::iter::Extend;

pub fn ethabi_decode<T, +Serde<T>, +EthAbi<T>>(data: ByteArray) -> Option<T> {
    let mut calldata = StructEVMCalldata { relative_offset: 0, offset: 0, calldata: data };

    println!("asdenaesdn");
    let mut decoded = calldata.decode(EthAbi::<T>::tokens().span());
    println!("asdenaesdn2");

    Serde::deserialize(ref decoded)
}

pub trait EthAbi<T> {
    fn tokens() -> Array<EVMTypes>;
}

#[derive(Debug, Drop, Serde)]
pub struct TestBro {
    pub a: ByteArray,
    pub b: ByteArray,
    pub c: ByteArray,
    pub d: ByteArray,
}

pub impl TestBroEthAbi of EthAbi<TestBro> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Bytes32, EVMTypes::Bytes32, EVMTypes::Bytes32, EVMTypes::Bytes32]
    }
}

#[derive(Debug, Drop, Serde)]
pub struct ZkgmPacket {
    // check if there is a fixed-sized type
    pub salt: u256,
    pub path: u256,
    pub instruction: Instruction,
}

impl ZkgmEthAbiDecodeImpl of EthAbi<ZkgmPacket> {
    fn tokens() -> Array<EVMTypes> {
        array![
            EVMTypes::Bytes32, EVMTypes::Uint256,
            EVMTypes::Tuple(array![EVMTypes::Uint8, EVMTypes::Uint8, EVMTypes::Bytes].span()),
        ]
    }
}

#[generate_trait]
pub impl ZkgmPacketImpl of ZkgmPacketTrait {
    fn encode(self: @ZkgmPacket) -> ByteArray {
        // let mut encoder = EVMCalldata {
        //     calldata: Default::default(),
        //     offset: 0,
        //     dynamic_data: Default::default(),
        //     dynamic_offset: 0,
        // };

        // let mut bz: Array<felt252> = array![
        //     self.salt.len().into(), (*self.path.low).into(), (*self.path.high).into(),
        // ];

        // bz.extend(self.instruction.values_to_be_encoded());

        // encoder.encode(EthAbi::<ZkgmPacket>::tokens().span(), bz.span())
        Default::default()
    }
}

#[derive(Drop, Debug, Serde)]
pub struct Instruction {
    pub version: u8,
    pub opcode: u8,
    pub operand: ByteArray,
}

impl InstructionEthAbiImpl of EthAbi<Instruction> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Uint8, EVMTypes::Uint8, EVMTypes::Bytes]
    }
}

#[generate_trait]
pub impl InstructionImpl of InstructionTrait {
    fn values_to_be_encoded(self: @Instruction) -> Array<felt252> {
        let mut bz: Array<felt252> = array![
            (*self.version).into(), (*self.opcode).into(), self.operand.len().into(),
        ];

        self.operand.serialize(ref bz);

        bz
    }
}

pub struct Forward {
    pub path: u256,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub instruction: Instruction,
}

pub struct Call {
    pub sender: ByteArray,
    pub eureka: bool,
    pub contract_address: ByteArray,
    pub contract_calldata: ByteArray,
}

pub struct Batch {
    pub instructions: Array<Instruction>,
}

pub struct TokenOrderV2 {
    pub sender: ByteArray,
    pub receiver: ByteArray,
    pub baseToken: ByteArray,
    pub baseAmount: u256,
    // TODO(aeryz): short string?
    pub quoteToken: ByteArray,
    pub quoteAmount: u256,
    pub kind: u8,
    pub metadata: ByteArray,
}

pub struct TokenMetadata {
    pub implementation: ByteArray,
    pub initializer: ByteArray,
}

pub struct SolverMetadata {
    pub solver_address: ByteArray,
    pub metadata: ByteArray,
}

pub struct Ack {
    pub tag: u256,
    pub inner_ack: ByteArray,
}

pub struct BatchAck {
    pub acknowledgements: Array<ByteArray>,
}

pub struct TokenOrderAck {
    pub fill_type: u256,
    pub market_maker: ByteArray,
}
