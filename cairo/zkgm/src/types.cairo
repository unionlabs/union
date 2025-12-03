use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use alexandria_evm::decoder::AbiDecodeTrait;
use alexandria_evm::encoder::{AbiEncodeTrait, EVMCalldata};
use alexandria_evm::evm_enum::EVMTypes;
use alexandria_evm::evm_struct::EVMCalldata as StructEVMCalldata;

pub trait EthAbi2<T> {
    fn encode(self: @T) -> ByteArray;

    fn decode(val: ByteArray) -> Result<T, ()>;
}

pub fn ethabi_decode<T, +Serde<T>, +EthAbi<T>>(data: ByteArray) -> Option<T> {
    let mut calldata = StructEVMCalldata { relative_offset: 0, offset: 0, calldata: data };

    let mut decoded = calldata.decode(EthAbi::<T>::tokens().span());

    Serde::deserialize(ref decoded)
}

pub fn ethabi_encode<T, +Serde<T>, +EthAbi<T>>(data: @T) -> ByteArray {
    let mut encoder = EVMCalldata {
        calldata: Default::default(),
        offset: 0,
        dynamic_data: Default::default(),
        dynamic_offset: EthAbi::<T>::dynamic_offset(),
    };

    let mut bytes = Default::default();
    data.serialize(ref bytes);

    encoder.encode(EthAbi::<T>::tokens().span(), bytes.span())
}

pub trait EthAbi<T> {
    fn tokens() -> Array<EVMTypes>;

    fn dynamic_offset() -> u32 {
        0
    }
}

#[derive(Debug, Drop, Serde, PartialEq)]
pub struct ZkgmPacket {
    pub salt: ByteArray,
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
    fn dynamic_offset() -> u32 {
        3 * 0x20
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub enum Opcode {
    Forward,
    Call,
    Batch,
    TokenOrder,
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub enum Version {
    V0,
    V1,
    V2,
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Instruction {
    pub version: Version,
    pub opcode: Opcode,
    pub operand: ByteArray,
}

impl InstructionEthAbiImpl of EthAbi<Instruction> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Uint8, EVMTypes::Uint8, EVMTypes::Bytes]
    }

    fn dynamic_offset() -> u32 {
        3 * 0x20
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Forward {
    pub path: u256,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub instruction: Instruction,
}

impl ForwardEthAbiImpl of EthAbi<Forward> {
    fn tokens() -> Array<EVMTypes> {
        array![
            EVMTypes::Uint256, EVMTypes::Uint64, EVMTypes::Uint64,
            EVMTypes::Tuple(array![EVMTypes::Uint8, EVMTypes::Uint8, EVMTypes::Bytes].span()),
        ]
    }

    fn dynamic_offset() -> u32 {
        (4 + EthAbi::<Instruction>::dynamic_offset()) * 0x20
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Call {
    pub sender: ByteArray,
    pub eureka: bool,
    pub contract_address: ByteArray,
    pub contract_calldata: ByteArray,
}


impl CallEthAbiImpl of EthAbi<Call> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Bytes, EVMTypes::Bool, EVMTypes::Bytes, EVMTypes::Bytes]
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Batch {
    pub instructions: Array<Instruction>,
}

impl BatchEthAbiImpl of EthAbi<Batch> {
    fn tokens() -> Array<EVMTypes> {
        array![
            EVMTypes::Array(
                array![
                    EVMTypes::Tuple(
                        array![EVMTypes::Uint8, EVMTypes::Uint8, EVMTypes::Bytes].span(),
                    ),
                ]
                    .span(),
            ),
        ]
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct TokenOrderV2 {
    pub sender: ByteArray,
    pub receiver: ByteArray,
    pub base_token: ByteArray,
    pub base_amount: u256,
    // TODO(aeryz): short string?
    pub quote_token: ByteArray,
    pub quote_amount: u256,
    pub kind: u8,
    pub metadata: ByteArray,
}

impl TokenOrderV2EthAbiImpl of EthAbi<TokenOrderV2> {
    fn tokens() -> Array<EVMTypes> {
        array![
            EVMTypes::Bytes, EVMTypes::Bytes, EVMTypes::Bytes, EVMTypes::Uint256, EVMTypes::Bytes,
            EVMTypes::Uint256, EVMTypes::Uint8, EVMTypes::Bytes,
        ]
    }
}

#[derive(Clone, Drop, Debug, Serde, PartialEq, starknet::Store)]
pub struct TokenMetadata {
    pub implementation: ByteArray,
    pub initializer: ByteArray,
}

impl TokenMetadataEthAbiImpl of EthAbi<TokenMetadata> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Bytes, EVMTypes::Bytes]
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct SolverMetadata {
    pub solver_address: ByteArray,
    pub metadata: ByteArray,
}


impl SolverMetadataEthAbiImpl of EthAbi<SolverMetadata> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Bytes, EVMTypes::Bytes]
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Ack {
    pub tag: u256,
    pub inner_ack: ByteArray,
}

impl AckEthAbiImpl of EthAbi<Ack> {
    fn tokens() -> Array<EVMTypes> {
        array![EVMTypes::Uint256, EVMTypes::Bytes]
    }

    fn dynamic_offset() -> u32 {
        0x20
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct BatchAck {
    pub acknowledgements: Array<ByteArray>,
}

impl BatchAckEthAbiImpl of EthAbi2<BatchAck> {
    fn encode(self: @BatchAck) -> ByteArray {
        let mut out = Default::default();

        out.append_u256(0x20);
        out.append_u256(self.acknowledgements.len().into());

        let mut total_len = 0;
        let base_offset: u256 = self.acknowledgements.len().into() * 0x20;
        for ack in self.acknowledgements {
            out.append_u256(base_offset + total_len);
            let mut len = ack.len();
            len = len + (32 - (len % 32));

            total_len += 0x20 + len.into();
        }

        for ack in self.acknowledgements {
            out.append_u256(ack.len().into());
            out.append(ack);

            postfix_bytes(ack, ref out);
        }

        out
    }

    fn decode(val: ByteArray) -> Result<BatchAck, ()> {
        Err(())
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct TokenOrderAck {
    pub fill_type: u256,
    pub market_maker: ByteArray,
}

impl TokenOrderAckEthAbiImpl of EthAbi2<TokenOrderAck> {
    fn encode(self: @TokenOrderAck) -> ByteArray {
        let mut out = Default::default();

        out.append_u256(*self.fill_type);
        out.append_u256(0x40);
        out.append_u256(self.market_maker.len().into());
        out.append(self.market_maker);

        postfix_bytes(self.market_maker, ref out);

        out
    }

    fn decode(val: ByteArray) -> Result<TokenOrderAck, ()> {
        let (_, fill_type) = val.read_u256(0);

        let (_, len) = val.read_u256(32 * 2);
        let (_, market_maker) = val.read_bytes(32 * 3, len.try_into().unwrap());

        Ok(TokenOrderAck { fill_type, market_maker })
    }
}


/// Postfixes the encoded `bytes` with zeroes s.t. the encoded length is 32 * N
fn postfix_bytes(bytes: @ByteArray, ref buffer: ByteArray) {
    let mut len = 32 - (bytes.len() % 32);

    // the following reduces the number of appends to Log2(len) and removes the need
    // for a loop
    if len >= 16 {
        buffer.append_u128(0);
        len -= 16;
    }

    if len >= 8 {
        buffer.append_u64(0);
        len -= 8;
    }

    if len >= 4 {
        buffer.append_u32(0);
        len -= 4;
    }

    if len >= 2 {
        buffer.append_u16(0);
        len -= 2;
    }

    if len >= 1 {
        buffer.append_u8(0);
    }
}

