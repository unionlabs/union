use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;

pub trait EthAbi<T> {
    fn encode(self: @T, ref buf: ByteArray);

    fn decode(val: ByteArray, offset: u32) -> Result<T, ()>;

    fn encoded_len(self: @T) -> u256 {
        0
    }
}

pub fn ethabi_decode<T, +EthAbi<T>>(data: ByteArray) -> Result<T, ()> {
    EthAbi::decode(data, 0)
}

pub fn ethabi_encode<T, +Serde<T>, +EthAbi<T>>(data: @T) -> ByteArray {
    let mut out = Default::default();
    data.encode(ref out);
    out
}

#[derive(Debug, Drop, Serde, PartialEq)]
pub struct ZkgmPacket {
    pub salt: ByteArray,
    pub path: u256,
    pub instruction: Instruction,
}

impl ZkgmPacketEthAbiImpl of EthAbi<ZkgmPacket> {
    fn encode(self: @ZkgmPacket, ref buf: ByteArray) {
        // this is actually bytes32, so append directly
        buf.append(self.salt);
        buf.append_u256(*self.path);
        buf.append_u256(0x20 * 3);
        self.instruction.encode(ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<ZkgmPacket, ()> {
        let (_, salt) = val.read_bytes(offset, 32);
        let (_, path) = val.read_u256(offset + 0x20);
        let instruction = EthAbi::decode(val, offset + 0x20 * 3)?;

        Ok(ZkgmPacket { salt, path, instruction })
    }

    fn encoded_len(self: @ZkgmPacket) -> u256 {
        32 * 3 + self.instruction.encoded_len()
    }
}

#[derive(Copy, Drop, Debug, Serde, PartialEq)]
pub enum Opcode {
    Forward,
    Call,
    Batch,
    TokenOrder,
}

impl OpcodeIntoU256Impl of Into<@Opcode, u256> {
    fn into(self: @Opcode) -> u256 {
        match self {
            Opcode::Forward => 0,
            Opcode::Call => 1,
            Opcode::Batch => 2,
            Opcode::TokenOrder => 3,
        }
    }
}

impl U256TryIntoOpcodeImpl of TryInto<u256, Opcode> {
    fn try_into(self: u256) -> Option<Opcode> {
        match self {
            0 => Some(Opcode::Forward),
            1 => Some(Opcode::Call),
            2 => Some(Opcode::Batch),
            3 => Some(Opcode::TokenOrder),
            _ => None,
        }
    }
}

#[derive(Copy, Drop, Debug, Serde, PartialEq)]
pub enum Version {
    V0,
    V1,
    V2,
}


impl VersionIntoU256Impl of Into<@Version, u256> {
    fn into(self: @Version) -> u256 {
        match self {
            Version::V0 => 0,
            Version::V1 => 1,
            Version::V2 => 2,
        }
    }
}

impl U256TryIntoVersionImpl of TryInto<u256, Version> {
    fn try_into(self: u256) -> Option<Version> {
        match self {
            0 => Some(Version::V0),
            1 => Some(Version::V1),
            2 => Some(Version::V2),
            _ => None,
        }
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Instruction {
    pub version: Version,
    pub opcode: Opcode,
    pub operand: ByteArray,
}

impl InstructionEthAbiImpl of EthAbi<Instruction> {
    fn encode(self: @Instruction, ref buf: ByteArray) {
        buf.append_u256(self.version.into());
        buf.append_u256(self.opcode.into());
        // `operand`'s offset
        buf.append_u256(0x20 * 3);
        buf.append_u256(self.operand.len().into());
        buf.append(self.operand);

        postfix_bytes(self.operand, ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<Instruction, ()> {
        let (_, version) = val.read_u256(offset);
        let (_, opcode) = val.read_u256(offset + 0x20);

        Ok(
            Instruction {
                version: version.try_into().ok_or(())?,
                opcode: opcode.try_into().ok_or(())?,
                operand: read_bytes(@val, offset, 2),
            },
        )
    }

    fn encoded_len(self: @Instruction) -> u256 {
        let operand_len: u256 = if self.operand.len() % 32 == 0 {
            self.operand.len().into()
        } else {
            (self.operand.len() / 32 + 32).into()
        };
        0x20 * 4 + operand_len
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
    fn encode(self: @Forward, ref buf: ByteArray) {
        buf.append_u256(*self.path);
        buf.append_u256((*self.timeout_height).into());
        buf.append_u256((*self.timeout_timestamp).into());
        buf.append_u256(0x20 * 4);
        self.instruction.encode(ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<Forward, ()> {
        let (_, path) = val.read_u256(offset);
        let (_, timeout_height) = val.read_u256(offset + 0x20);
        let (_, timeout_timestamp) = val.read_u256(offset + 0x40);

        let instruction = EthAbi::decode(val, offset + 0x80)?;

        Ok(
            Forward {
                path,
                timeout_height: timeout_height.try_into().unwrap(),
                timeout_timestamp: timeout_timestamp.try_into().unwrap(),
                instruction,
            },
        )
    }

    fn encoded_len(self: @Forward) -> u256 {
        0x20 * 4 + self.instruction.encoded_len()
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Call {
    pub sender: ByteArray,
    pub eureka: bool,
    pub contract_address: ByteArray,
    pub contract_calldata: ByteArray,
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Batch {
    pub instructions: Array<Instruction>,
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
    fn encode(self: @TokenOrderV2, ref buf: ByteArray) {
        let (sender_len, receiver_len, base_token_len, quote_token_len) = (
            encoded_bytes_len(self.sender),
            encoded_bytes_len(self.receiver),
            encoded_bytes_len(self.base_token),
            encoded_bytes_len(self.quote_token),
        );

        let mut cursor: u32 = 0x20 * 8;

        // sender offset
        buf.append_u256(cursor.into());
        cursor = cursor + 0x20 + sender_len;

        // receiver offset
        buf.append_u256(cursor.into());
        cursor = cursor + 0x20 + receiver_len;

        // base_token offset
        buf.append_u256(cursor.into());
        cursor = cursor + 0x20 + base_token_len;

        // base_amount
        buf.append_u256(*self.base_amount);

        // quote_token offset
        buf.append_u256(cursor.into());
        cursor = cursor + 0x20 + quote_token_len;

        // quote_amount
        buf.append_u256(*self.quote_amount);

        // kind
        buf.append_u256((*self.kind).into());

        // metadata offset
        buf.append_u256(cursor.into());

        // sender len
        buf.append_u256(self.sender.len().into());

        // sender
        buf.append(self.sender);
        postfix_bytes(self.sender, ref buf);

        // receiver len
        buf.append_u256(self.receiver.len().into());

        // receiver
        buf.append(self.receiver);
        postfix_bytes(self.receiver, ref buf);

        // base_token len
        buf.append_u256(self.base_token.len().into());

        // base_token
        buf.append(self.base_token);
        postfix_bytes(self.base_token, ref buf);

        // quote_token len
        buf.append_u256(self.quote_token.len().into());

        // quote_token
        buf.append(self.quote_token);
        postfix_bytes(self.quote_token, ref buf);

        // metadata len
        buf.append_u256(self.metadata.len().into());

        // metadata
        buf.append(self.metadata);
        postfix_bytes(self.metadata, ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<TokenOrderV2, ()> {
        let (_, base_amount) = val.read_u256(offset + 0x20 * 3);
        let (_, quote_amount) = val.read_u256(offset + 0x20 * 5);
        let (_, kind) = val.read_u256(offset + 0x20 * 6);

        Ok(
            TokenOrderV2 {
                sender: read_bytes(@val, offset, 0),
                receiver: read_bytes(@val, offset, 1),
                base_token: read_bytes(@val, offset, 2),
                base_amount,
                quote_token: read_bytes(@val, offset, 4),
                quote_amount,
                kind: kind.try_into().unwrap(),
                metadata: read_bytes(@val, offset, 7),
            },
        )
    }
}

/// implementation: ClassHash
/// initializer: Serde serialized calldata
#[derive(Clone, Drop, Debug, Serde, PartialEq, starknet::Store)]
pub struct TokenMetadata {
    pub implementation: ByteArray,
    pub initializer: ByteArray,
}

impl TokenMetadataEthAbiImpl of EthAbi<TokenMetadata> {
    fn encode(self: @TokenMetadata, ref buf: ByteArray) {}

    fn decode(val: ByteArray, offset: u32) -> Result<TokenMetadata, ()> {
        Err(())
    }
}

/// `solver_address`: Little-Endian felt252 which is encoded as LE u256
#[derive(Drop, Debug, Serde, PartialEq)]
pub struct SolverMetadata {
    pub solver_address: ByteArray,
    pub metadata: ByteArray,
}

impl SolverMetadataEthAbiImpl of EthAbi<SolverMetadata> {
    fn encode(self: @SolverMetadata, ref buf: ByteArray) {
        // solver_address offset
        buf.append_u256(0x20 * 2);

        // metadata offset
        buf
            .append_u256(
                ( // the solver_address bytes start here
                0x20 * 2
                    + // the length of the solver_address
                    0x20
                    + // the actual solver_address
                    encoded_bytes_len(self.solver_address))
                    .into(),
            );

        buf.append_u256(self.solver_address.len().into());
        buf.append(self.solver_address);
        postfix_bytes(self.solver_address, ref buf);

        buf.append_u256(self.metadata.len().into());
        buf.append(self.metadata);
        postfix_bytes(self.metadata, ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<SolverMetadata, ()> {
        Ok(
            SolverMetadata {
                solver_address: read_bytes(@val, offset, 0), metadata: read_bytes(@val, offset, 1),
            },
        )
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct Ack {
    pub tag: AckTag,
    pub inner_ack: ByteArray,
}


#[generate_trait]
pub impl AckImpl of AckTrait {
    fn new_failure() -> Ack {
        Ack { tag: AckTag::Failure, inner_ack: "" }
    }

    fn new_success(inner_ack: ByteArray) -> Ack {
        Ack { tag: AckTag::Success, inner_ack }
    }
}

impl AckEthAbiImpl of EthAbi<Ack> {
    fn encode(self: @Ack, ref buf: ByteArray) {}

    fn decode(val: ByteArray, offset: u32) -> Result<Ack, ()> {
        Err(())
    }
}

#[derive(Copy, Drop, Debug, Serde, PartialEq)]
pub enum AckTag {
    Failure,
    Success,
}

impl U256TryIntoAckTagImpl of TryInto<u256, AckTag> {
    fn try_into(self: u256) -> Option<AckTag> {
        match self {
            0 => Some(AckTag::Failure),
            1 => Some(AckTag::Success),
            _ => None,
        }
    }
}

impl AckTagIntoImpl of Into<@AckTag, u256> {
    fn into(self: @AckTag) -> u256 {
        match self {
            AckTag::Failure => 0,
            AckTag::Success => 1,
        }
    }
}


#[derive(Drop, Debug, Serde, PartialEq)]
pub struct BatchAck {
    pub acknowledgements: Array<ByteArray>,
}

impl BatchAckEthAbiImpl of EthAbi<BatchAck> {
    fn encode(self: @BatchAck, ref buf: ByteArray) {
        buf.append_u256(0x20);
        buf.append_u256(self.acknowledgements.len().into());

        let mut total_len = 0;
        let base_offset: u256 = self.acknowledgements.len().into() * 0x20;
        for ack in self.acknowledgements {
            buf.append_u256(base_offset + total_len);
            let mut len = ack.len();
            len = len + (32 - (len % 32));

            total_len += 0x20 + len.into();
        }

        for ack in self.acknowledgements {
            buf.append_u256(ack.len().into());
            buf.append(ack);

            postfix_bytes(ack, ref buf);
        }
    }

    fn decode(val: ByteArray, offset: u32) -> Result<BatchAck, ()> {
        let (_, holy_prefix) = val.read_u256(offset);

        if holy_prefix != 0x20 {
            return Err(());
        }

        let (_, len) = val.read_u256(offset + 0x20);
        let len: u32 = len.try_into().unwrap();

        let mut acknowledgements: Array<ByteArray> = Default::default();
        for i in 0..len {
            // read the offset where the i'th item is written to
            // we add `0x20 * 2` because it's where the array of bytes start
            let (_, mut bytes_offset) = val.read_u256(offset + 0x20 * (i + 2));
            bytes_offset = bytes_offset + (0x20 * 2);
            let bytes_offset = bytes_offset.try_into().unwrap();
            // read the length of the single `bytes` item
            let (_, len) = val.read_u256(offset + bytes_offset);
            // the full bytes are written right after the u256 `len`
            let (_, data) = val.read_bytes(offset + bytes_offset + 0x20, len.try_into().unwrap());
            acknowledgements.append(data);
        }

        Ok(BatchAck { acknowledgements })
    }
}

#[derive(Drop, Debug, Serde, PartialEq)]
pub struct TokenOrderAck {
    pub fill_type: u256,
    pub market_maker: ByteArray,
}

impl TokenOrderAckEthAbiImpl of EthAbi<TokenOrderAck> {
    fn encode(self: @TokenOrderAck, ref buf: ByteArray) {
        buf.append_u256(*self.fill_type);
        buf.append_u256(0x40);
        buf.append_u256(self.market_maker.len().into());
        buf.append(self.market_maker);

        postfix_bytes(self.market_maker, ref buf);
    }

    fn decode(val: ByteArray, offset: u32) -> Result<TokenOrderAck, ()> {
        let (_, fill_type) = val.read_u256(offset);

        let (_, len) = val.read_u256(offset + 32 * 2);
        let (_, market_maker) = val.read_bytes(offset + 32 * 3, len.try_into().unwrap());

        Ok(TokenOrderAck { fill_type, market_maker })
    }
}

/// Read a `bytes`
/// - `bytes`: The encoded bytes
/// - `base_offset`: The offset in the `bytes` where the data type that surrounds this `bytes`
/// - `bytes_order`: Starting from 0, the order of the desired `bytes` in the data type.
fn read_bytes(bytes: @ByteArray, base_offset: u32, bytes_order: u32) -> ByteArray {
    let (_, bytes_offset) = bytes.read_u256(base_offset + (0x20 * bytes_order));
    let bytes_offset = bytes_offset.try_into().unwrap();
    let (_, len) = bytes.read_u256(base_offset + bytes_offset);
    let (_, bytes) = bytes.read_bytes(base_offset + bytes_offset + 0x20, len.try_into().unwrap());

    bytes
}

/// Postfixes the encoded `bytes` with zeroes s.t. the encoded length is 32 * N
fn postfix_bytes(bytes: @ByteArray, ref buffer: ByteArray) {
    let len_mod_32 = bytes.len() % 32;

    if len_mod_32 == 0 {
        return;
    }

    let mut len = 32 - len_mod_32;

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

#[inline]
fn encoded_bytes_len(bytes: @ByteArray) -> u32 {
    if bytes.len() % 32 == 0 {
        bytes.len()
    } else {
        bytes.len() + (32 - (bytes.len() % 32))
    }
}

