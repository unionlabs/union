use super::{
	BlockType, CountedList, CountedListWriter, Deserialize, Error, Serialize, Uint32, Uint64,
	Uint8, VarInt32, VarInt64, VarUint32,
};
use crate::io;
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

/// List of instructions (usually inside a block section).
#[derive(Debug, Clone, PartialEq)]
pub struct Instructions(Vec<Instruction>);

impl Instructions {
	/// New list of instructions from vector of instructions.
	pub fn new(elements: Vec<Instruction>) -> Self {
		Instructions(elements)
	}

	/// Empty expression with only `Instruction::End` instruction.
	pub fn empty() -> Self {
		Instructions(vec![Instruction::End])
	}

	/// List of individual instructions.
	pub fn elements(&self) -> &[Instruction] {
		&self.0
	}

	/// Individual instructions, mutable.
	pub fn elements_mut(&mut self) -> &mut Vec<Instruction> {
		&mut self.0
	}
}

impl Deserialize for Instructions {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut instructions = Vec::new();
		let mut block_count = 1usize;

		loop {
			let instruction = Instruction::deserialize(reader)?;
			if instruction.is_terminal() {
				block_count -= 1;
			} else if instruction.is_block() {
				block_count =
					block_count.checked_add(1).ok_or(Error::Other("too many instructions"))?;
			}

			instructions.push(instruction);
			if block_count == 0 {
				break
			}
		}

		Ok(Instructions(instructions))
	}
}

/// Initialization expression.
#[derive(Debug, Clone, PartialEq)]
pub struct InitExpr(Vec<Instruction>);

impl InitExpr {
	/// New initialization expression from instruction list.
	///
	/// `code` must end with the `Instruction::End` instruction!
	pub fn new(code: Vec<Instruction>) -> Self {
		InitExpr(code)
	}

	/// Empty expression with only `Instruction::End` instruction.
	pub fn empty() -> Self {
		InitExpr(vec![Instruction::End])
	}

	/// List of instructions used in the expression.
	pub fn code(&self) -> &[Instruction] {
		&self.0
	}

	/// List of instructions used in the expression.
	pub fn code_mut(&mut self) -> &mut Vec<Instruction> {
		&mut self.0
	}
}

impl Deserialize for InitExpr {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let mut instructions = Vec::new();

		loop {
			let instruction = Instruction::deserialize(reader)?;
			let is_terminal = instruction.is_terminal();
			instructions.push(instruction);
			if is_terminal {
				break
			}
		}

		Ok(InitExpr(instructions))
	}
}

/// Instruction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Instruction {
	Unreachable,
	Nop,
	Block(BlockType),
	Loop(BlockType),
	If(BlockType),
	Else,
	End,
	Br(u32),
	BrIf(u32),
	BrTable(Box<BrTableData>),
	Return,

	Call(u32),
	CallIndirect(u32, u8),

	Drop,
	Select,

	GetLocal(u32),
	SetLocal(u32),
	TeeLocal(u32),
	GetGlobal(u32),
	SetGlobal(u32),

	// All store/load instructions operate with 'memory immediates'
	// which represented here as (flag, offset) tuple
	I32Load(u32, u32),
	I64Load(u32, u32),
	F32Load(u32, u32),
	F64Load(u32, u32),
	I32Load8S(u32, u32),
	I32Load8U(u32, u32),
	I32Load16S(u32, u32),
	I32Load16U(u32, u32),
	I64Load8S(u32, u32),
	I64Load8U(u32, u32),
	I64Load16S(u32, u32),
	I64Load16U(u32, u32),
	I64Load32S(u32, u32),
	I64Load32U(u32, u32),
	I32Store(u32, u32),
	I64Store(u32, u32),
	F32Store(u32, u32),
	F64Store(u32, u32),
	I32Store8(u32, u32),
	I32Store16(u32, u32),
	I64Store8(u32, u32),
	I64Store16(u32, u32),
	I64Store32(u32, u32),

	CurrentMemory(u8),
	GrowMemory(u8),

	I32Const(i32),
	I64Const(i64),
	F32Const(u32),
	F64Const(u64),

	I32Eqz,
	I32Eq,
	I32Ne,
	I32LtS,
	I32LtU,
	I32GtS,
	I32GtU,
	I32LeS,
	I32LeU,
	I32GeS,
	I32GeU,

	I64Eqz,
	I64Eq,
	I64Ne,
	I64LtS,
	I64LtU,
	I64GtS,
	I64GtU,
	I64LeS,
	I64LeU,
	I64GeS,
	I64GeU,

	F32Eq,
	F32Ne,
	F32Lt,
	F32Gt,
	F32Le,
	F32Ge,

	F64Eq,
	F64Ne,
	F64Lt,
	F64Gt,
	F64Le,
	F64Ge,

	I32Clz,
	I32Ctz,
	I32Popcnt,
	I32Add,
	I32Sub,
	I32Mul,
	I32DivS,
	I32DivU,
	I32RemS,
	I32RemU,
	I32And,
	I32Or,
	I32Xor,
	I32Shl,
	I32ShrS,
	I32ShrU,
	I32Rotl,
	I32Rotr,

	I64Clz,
	I64Ctz,
	I64Popcnt,
	I64Add,
	I64Sub,
	I64Mul,
	I64DivS,
	I64DivU,
	I64RemS,
	I64RemU,
	I64And,
	I64Or,
	I64Xor,
	I64Shl,
	I64ShrS,
	I64ShrU,
	I64Rotl,
	I64Rotr,
	F32Abs,
	F32Neg,
	F32Ceil,
	F32Floor,
	F32Trunc,
	F32Nearest,
	F32Sqrt,
	F32Add,
	F32Sub,
	F32Mul,
	F32Div,
	F32Min,
	F32Max,
	F32Copysign,
	F64Abs,
	F64Neg,
	F64Ceil,
	F64Floor,
	F64Trunc,
	F64Nearest,
	F64Sqrt,
	F64Add,
	F64Sub,
	F64Mul,
	F64Div,
	F64Min,
	F64Max,
	F64Copysign,

	I32WrapI64,
	I32TruncSF32,
	I32TruncUF32,
	I32TruncSF64,
	I32TruncUF64,
	I64ExtendSI32,
	I64ExtendUI32,
	I64TruncSF32,
	I64TruncUF32,
	I64TruncSF64,
	I64TruncUF64,
	F32ConvertSI32,
	F32ConvertUI32,
	F32ConvertSI64,
	F32ConvertUI64,
	F32DemoteF64,
	F64ConvertSI32,
	F64ConvertUI32,
	F64ConvertSI64,
	F64ConvertUI64,
	F64PromoteF32,

	I32ReinterpretF32,
	I64ReinterpretF64,
	F32ReinterpretI32,
	F64ReinterpretI64,

	#[cfg(feature = "atomics")]
	Atomics(AtomicsInstruction),

	#[cfg(feature = "simd")]
	Simd(SimdInstruction),

	#[cfg(feature = "sign_ext")]
	SignExt(SignExtInstruction),

	#[cfg(feature = "bulk")]
	Bulk(BulkInstruction),
}

#[allow(missing_docs)]
#[cfg(feature = "atomics")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AtomicsInstruction {
	AtomicWake(MemArg),
	I32AtomicWait(MemArg),
	I64AtomicWait(MemArg),

	I32AtomicLoad(MemArg),
	I64AtomicLoad(MemArg),
	I32AtomicLoad8u(MemArg),
	I32AtomicLoad16u(MemArg),
	I64AtomicLoad8u(MemArg),
	I64AtomicLoad16u(MemArg),
	I64AtomicLoad32u(MemArg),
	I32AtomicStore(MemArg),
	I64AtomicStore(MemArg),
	I32AtomicStore8u(MemArg),
	I32AtomicStore16u(MemArg),
	I64AtomicStore8u(MemArg),
	I64AtomicStore16u(MemArg),
	I64AtomicStore32u(MemArg),

	I32AtomicRmwAdd(MemArg),
	I64AtomicRmwAdd(MemArg),
	I32AtomicRmwAdd8u(MemArg),
	I32AtomicRmwAdd16u(MemArg),
	I64AtomicRmwAdd8u(MemArg),
	I64AtomicRmwAdd16u(MemArg),
	I64AtomicRmwAdd32u(MemArg),

	I32AtomicRmwSub(MemArg),
	I64AtomicRmwSub(MemArg),
	I32AtomicRmwSub8u(MemArg),
	I32AtomicRmwSub16u(MemArg),
	I64AtomicRmwSub8u(MemArg),
	I64AtomicRmwSub16u(MemArg),
	I64AtomicRmwSub32u(MemArg),

	I32AtomicRmwAnd(MemArg),
	I64AtomicRmwAnd(MemArg),
	I32AtomicRmwAnd8u(MemArg),
	I32AtomicRmwAnd16u(MemArg),
	I64AtomicRmwAnd8u(MemArg),
	I64AtomicRmwAnd16u(MemArg),
	I64AtomicRmwAnd32u(MemArg),

	I32AtomicRmwOr(MemArg),
	I64AtomicRmwOr(MemArg),
	I32AtomicRmwOr8u(MemArg),
	I32AtomicRmwOr16u(MemArg),
	I64AtomicRmwOr8u(MemArg),
	I64AtomicRmwOr16u(MemArg),
	I64AtomicRmwOr32u(MemArg),

	I32AtomicRmwXor(MemArg),
	I64AtomicRmwXor(MemArg),
	I32AtomicRmwXor8u(MemArg),
	I32AtomicRmwXor16u(MemArg),
	I64AtomicRmwXor8u(MemArg),
	I64AtomicRmwXor16u(MemArg),
	I64AtomicRmwXor32u(MemArg),

	I32AtomicRmwXchg(MemArg),
	I64AtomicRmwXchg(MemArg),
	I32AtomicRmwXchg8u(MemArg),
	I32AtomicRmwXchg16u(MemArg),
	I64AtomicRmwXchg8u(MemArg),
	I64AtomicRmwXchg16u(MemArg),
	I64AtomicRmwXchg32u(MemArg),

	I32AtomicRmwCmpxchg(MemArg),
	I64AtomicRmwCmpxchg(MemArg),
	I32AtomicRmwCmpxchg8u(MemArg),
	I32AtomicRmwCmpxchg16u(MemArg),
	I64AtomicRmwCmpxchg8u(MemArg),
	I64AtomicRmwCmpxchg16u(MemArg),
	I64AtomicRmwCmpxchg32u(MemArg),
}

#[allow(missing_docs)]
#[cfg(feature = "simd")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SimdInstruction {
	V128Const(Box<[u8; 16]>),
	V128Load(MemArg),
	V128Store(MemArg),
	I8x16Splat,
	I16x8Splat,
	I32x4Splat,
	I64x2Splat,
	F32x4Splat,
	F64x2Splat,
	I8x16ExtractLaneS(u8),
	I8x16ExtractLaneU(u8),
	I16x8ExtractLaneS(u8),
	I16x8ExtractLaneU(u8),
	I32x4ExtractLane(u8),
	I64x2ExtractLane(u8),
	F32x4ExtractLane(u8),
	F64x2ExtractLane(u8),
	I8x16ReplaceLane(u8),
	I16x8ReplaceLane(u8),
	I32x4ReplaceLane(u8),
	I64x2ReplaceLane(u8),
	F32x4ReplaceLane(u8),
	F64x2ReplaceLane(u8),
	V8x16Shuffle(Box<[u8; 16]>),
	I8x16Add,
	I16x8Add,
	I32x4Add,
	I64x2Add,
	I8x16Sub,
	I16x8Sub,
	I32x4Sub,
	I64x2Sub,
	I8x16Mul,
	I16x8Mul,
	I32x4Mul,
	// I64x2Mul,
	I8x16Neg,
	I16x8Neg,
	I32x4Neg,
	I64x2Neg,
	I8x16AddSaturateS,
	I8x16AddSaturateU,
	I16x8AddSaturateS,
	I16x8AddSaturateU,
	I8x16SubSaturateS,
	I8x16SubSaturateU,
	I16x8SubSaturateS,
	I16x8SubSaturateU,
	I8x16Shl,
	I16x8Shl,
	I32x4Shl,
	I64x2Shl,
	I8x16ShrS,
	I8x16ShrU,
	I16x8ShrS,
	I16x8ShrU,
	I32x4ShrS,
	I32x4ShrU,
	I64x2ShrS,
	I64x2ShrU,
	V128And,
	V128Or,
	V128Xor,
	V128Not,
	V128Bitselect,
	I8x16AnyTrue,
	I16x8AnyTrue,
	I32x4AnyTrue,
	I64x2AnyTrue,
	I8x16AllTrue,
	I16x8AllTrue,
	I32x4AllTrue,
	I64x2AllTrue,
	I8x16Eq,
	I16x8Eq,
	I32x4Eq,
	// I64x2Eq,
	F32x4Eq,
	F64x2Eq,
	I8x16Ne,
	I16x8Ne,
	I32x4Ne,
	// I64x2Ne,
	F32x4Ne,
	F64x2Ne,
	I8x16LtS,
	I8x16LtU,
	I16x8LtS,
	I16x8LtU,
	I32x4LtS,
	I32x4LtU,
	// I64x2LtS,
	// I64x2LtU,
	F32x4Lt,
	F64x2Lt,
	I8x16LeS,
	I8x16LeU,
	I16x8LeS,
	I16x8LeU,
	I32x4LeS,
	I32x4LeU,
	// I64x2LeS,
	// I64x2LeU,
	F32x4Le,
	F64x2Le,
	I8x16GtS,
	I8x16GtU,
	I16x8GtS,
	I16x8GtU,
	I32x4GtS,
	I32x4GtU,
	// I64x2GtS,
	// I64x2GtU,
	F32x4Gt,
	F64x2Gt,
	I8x16GeS,
	I8x16GeU,
	I16x8GeS,
	I16x8GeU,
	I32x4GeS,
	I32x4GeU,
	// I64x2GeS,
	// I64x2GeU,
	F32x4Ge,
	F64x2Ge,
	F32x4Neg,
	F64x2Neg,
	F32x4Abs,
	F64x2Abs,
	F32x4Min,
	F64x2Min,
	F32x4Max,
	F64x2Max,
	F32x4Add,
	F64x2Add,
	F32x4Sub,
	F64x2Sub,
	F32x4Div,
	F64x2Div,
	F32x4Mul,
	F64x2Mul,
	F32x4Sqrt,
	F64x2Sqrt,
	F32x4ConvertSI32x4,
	F32x4ConvertUI32x4,
	F64x2ConvertSI64x2,
	F64x2ConvertUI64x2,
	I32x4TruncSF32x4Sat,
	I32x4TruncUF32x4Sat,
	I64x2TruncSF64x2Sat,
	I64x2TruncUF64x2Sat,
}

#[allow(missing_docs)]
#[cfg(feature = "sign_ext")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SignExtInstruction {
	I32Extend8S,
	I32Extend16S,
	I64Extend8S,
	I64Extend16S,
	I64Extend32S,
}

#[allow(missing_docs)]
#[cfg(feature = "bulk")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BulkInstruction {
	MemoryInit(u32),
	MemoryDrop(u32),
	MemoryCopy,
	MemoryFill,
	TableInit(u32),
	TableDrop(u32),
	TableCopy,
}

#[cfg(any(feature = "simd", feature = "atomics"))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct MemArg {
	pub align: u8,
	pub offset: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub struct BrTableData {
	pub table: Box<[u32]>,
	pub default: u32,
}

impl Instruction {
	/// Is this instruction starts the new block (which should end with terminal instruction).
	pub fn is_block(&self) -> bool {
		matches!(self, &Instruction::Block(_) | &Instruction::Loop(_) | &Instruction::If(_))
	}

	/// Is this instruction determines the termination of instruction sequence?
	///
	/// `true` for `Instruction::End`
	pub fn is_terminal(&self) -> bool {
		matches!(self, &Instruction::End)
	}
}

#[allow(missing_docs)]
pub mod opcodes {
	pub const UNREACHABLE: u8 = 0x00;
	pub const NOP: u8 = 0x01;
	pub const BLOCK: u8 = 0x02;
	pub const LOOP: u8 = 0x03;
	pub const IF: u8 = 0x04;
	pub const ELSE: u8 = 0x05;
	pub const END: u8 = 0x0b;
	pub const BR: u8 = 0x0c;
	pub const BRIF: u8 = 0x0d;
	pub const BRTABLE: u8 = 0x0e;
	pub const RETURN: u8 = 0x0f;
	pub const CALL: u8 = 0x10;
	pub const CALLINDIRECT: u8 = 0x11;
	pub const DROP: u8 = 0x1a;
	pub const SELECT: u8 = 0x1b;
	pub const GETLOCAL: u8 = 0x20;
	pub const SETLOCAL: u8 = 0x21;
	pub const TEELOCAL: u8 = 0x22;
	pub const GETGLOBAL: u8 = 0x23;
	pub const SETGLOBAL: u8 = 0x24;
	pub const I32LOAD: u8 = 0x28;
	pub const I64LOAD: u8 = 0x29;
	pub const F32LOAD: u8 = 0x2a;
	pub const F64LOAD: u8 = 0x2b;
	pub const I32LOAD8S: u8 = 0x2c;
	pub const I32LOAD8U: u8 = 0x2d;
	pub const I32LOAD16S: u8 = 0x2e;
	pub const I32LOAD16U: u8 = 0x2f;
	pub const I64LOAD8S: u8 = 0x30;
	pub const I64LOAD8U: u8 = 0x31;
	pub const I64LOAD16S: u8 = 0x32;
	pub const I64LOAD16U: u8 = 0x33;
	pub const I64LOAD32S: u8 = 0x34;
	pub const I64LOAD32U: u8 = 0x35;
	pub const I32STORE: u8 = 0x36;
	pub const I64STORE: u8 = 0x37;
	pub const F32STORE: u8 = 0x38;
	pub const F64STORE: u8 = 0x39;
	pub const I32STORE8: u8 = 0x3a;
	pub const I32STORE16: u8 = 0x3b;
	pub const I64STORE8: u8 = 0x3c;
	pub const I64STORE16: u8 = 0x3d;
	pub const I64STORE32: u8 = 0x3e;
	pub const CURRENTMEMORY: u8 = 0x3f;
	pub const GROWMEMORY: u8 = 0x40;
	pub const I32CONST: u8 = 0x41;
	pub const I64CONST: u8 = 0x42;
	pub const F32CONST: u8 = 0x43;
	pub const F64CONST: u8 = 0x44;
	pub const I32EQZ: u8 = 0x45;
	pub const I32EQ: u8 = 0x46;
	pub const I32NE: u8 = 0x47;
	pub const I32LTS: u8 = 0x48;
	pub const I32LTU: u8 = 0x49;
	pub const I32GTS: u8 = 0x4a;
	pub const I32GTU: u8 = 0x4b;
	pub const I32LES: u8 = 0x4c;
	pub const I32LEU: u8 = 0x4d;
	pub const I32GES: u8 = 0x4e;
	pub const I32GEU: u8 = 0x4f;
	pub const I64EQZ: u8 = 0x50;
	pub const I64EQ: u8 = 0x51;
	pub const I64NE: u8 = 0x52;
	pub const I64LTS: u8 = 0x53;
	pub const I64LTU: u8 = 0x54;
	pub const I64GTS: u8 = 0x55;
	pub const I64GTU: u8 = 0x56;
	pub const I64LES: u8 = 0x57;
	pub const I64LEU: u8 = 0x58;
	pub const I64GES: u8 = 0x59;
	pub const I64GEU: u8 = 0x5a;

	pub const F32EQ: u8 = 0x5b;
	pub const F32NE: u8 = 0x5c;
	pub const F32LT: u8 = 0x5d;
	pub const F32GT: u8 = 0x5e;
	pub const F32LE: u8 = 0x5f;
	pub const F32GE: u8 = 0x60;

	pub const F64EQ: u8 = 0x61;
	pub const F64NE: u8 = 0x62;
	pub const F64LT: u8 = 0x63;
	pub const F64GT: u8 = 0x64;
	pub const F64LE: u8 = 0x65;
	pub const F64GE: u8 = 0x66;

	pub const I32CLZ: u8 = 0x67;
	pub const I32CTZ: u8 = 0x68;
	pub const I32POPCNT: u8 = 0x69;
	pub const I32ADD: u8 = 0x6a;
	pub const I32SUB: u8 = 0x6b;
	pub const I32MUL: u8 = 0x6c;
	pub const I32DIVS: u8 = 0x6d;
	pub const I32DIVU: u8 = 0x6e;
	pub const I32REMS: u8 = 0x6f;
	pub const I32REMU: u8 = 0x70;
	pub const I32AND: u8 = 0x71;
	pub const I32OR: u8 = 0x72;
	pub const I32XOR: u8 = 0x73;
	pub const I32SHL: u8 = 0x74;
	pub const I32SHRS: u8 = 0x75;
	pub const I32SHRU: u8 = 0x76;
	pub const I32ROTL: u8 = 0x77;
	pub const I32ROTR: u8 = 0x78;

	pub const I64CLZ: u8 = 0x79;
	pub const I64CTZ: u8 = 0x7a;
	pub const I64POPCNT: u8 = 0x7b;
	pub const I64ADD: u8 = 0x7c;
	pub const I64SUB: u8 = 0x7d;
	pub const I64MUL: u8 = 0x7e;
	pub const I64DIVS: u8 = 0x7f;
	pub const I64DIVU: u8 = 0x80;
	pub const I64REMS: u8 = 0x81;
	pub const I64REMU: u8 = 0x82;
	pub const I64AND: u8 = 0x83;
	pub const I64OR: u8 = 0x84;
	pub const I64XOR: u8 = 0x85;
	pub const I64SHL: u8 = 0x86;
	pub const I64SHRS: u8 = 0x87;
	pub const I64SHRU: u8 = 0x88;
	pub const I64ROTL: u8 = 0x89;
	pub const I64ROTR: u8 = 0x8a;
	pub const F32ABS: u8 = 0x8b;
	pub const F32NEG: u8 = 0x8c;
	pub const F32CEIL: u8 = 0x8d;
	pub const F32FLOOR: u8 = 0x8e;
	pub const F32TRUNC: u8 = 0x8f;
	pub const F32NEAREST: u8 = 0x90;
	pub const F32SQRT: u8 = 0x91;
	pub const F32ADD: u8 = 0x92;
	pub const F32SUB: u8 = 0x93;
	pub const F32MUL: u8 = 0x94;
	pub const F32DIV: u8 = 0x95;
	pub const F32MIN: u8 = 0x96;
	pub const F32MAX: u8 = 0x97;
	pub const F32COPYSIGN: u8 = 0x98;
	pub const F64ABS: u8 = 0x99;
	pub const F64NEG: u8 = 0x9a;
	pub const F64CEIL: u8 = 0x9b;
	pub const F64FLOOR: u8 = 0x9c;
	pub const F64TRUNC: u8 = 0x9d;
	pub const F64NEAREST: u8 = 0x9e;
	pub const F64SQRT: u8 = 0x9f;
	pub const F64ADD: u8 = 0xa0;
	pub const F64SUB: u8 = 0xa1;
	pub const F64MUL: u8 = 0xa2;
	pub const F64DIV: u8 = 0xa3;
	pub const F64MIN: u8 = 0xa4;
	pub const F64MAX: u8 = 0xa5;
	pub const F64COPYSIGN: u8 = 0xa6;

	pub const I32WRAPI64: u8 = 0xa7;
	pub const I32TRUNCSF32: u8 = 0xa8;
	pub const I32TRUNCUF32: u8 = 0xa9;
	pub const I32TRUNCSF64: u8 = 0xaa;
	pub const I32TRUNCUF64: u8 = 0xab;
	pub const I64EXTENDSI32: u8 = 0xac;
	pub const I64EXTENDUI32: u8 = 0xad;
	pub const I64TRUNCSF32: u8 = 0xae;
	pub const I64TRUNCUF32: u8 = 0xaf;
	pub const I64TRUNCSF64: u8 = 0xb0;
	pub const I64TRUNCUF64: u8 = 0xb1;
	pub const F32CONVERTSI32: u8 = 0xb2;
	pub const F32CONVERTUI32: u8 = 0xb3;
	pub const F32CONVERTSI64: u8 = 0xb4;
	pub const F32CONVERTUI64: u8 = 0xb5;
	pub const F32DEMOTEF64: u8 = 0xb6;
	pub const F64CONVERTSI32: u8 = 0xb7;
	pub const F64CONVERTUI32: u8 = 0xb8;
	pub const F64CONVERTSI64: u8 = 0xb9;
	pub const F64CONVERTUI64: u8 = 0xba;
	pub const F64PROMOTEF32: u8 = 0xbb;

	pub const I32REINTERPRETF32: u8 = 0xbc;
	pub const I64REINTERPRETF64: u8 = 0xbd;
	pub const F32REINTERPRETI32: u8 = 0xbe;
	pub const F64REINTERPRETI64: u8 = 0xbf;

	#[cfg(feature = "sign_ext")]
	pub mod sign_ext {
		pub const I32_EXTEND8_S: u8 = 0xc0;
		pub const I32_EXTEND16_S: u8 = 0xc1;
		pub const I64_EXTEND8_S: u8 = 0xc2;
		pub const I64_EXTEND16_S: u8 = 0xc3;
		pub const I64_EXTEND32_S: u8 = 0xc4;
	}

	#[cfg(feature = "atomics")]
	pub mod atomics {
		pub const ATOMIC_PREFIX: u8 = 0xfe;
		pub const ATOMIC_WAKE: u8 = 0x00;
		pub const I32_ATOMIC_WAIT: u8 = 0x01;
		pub const I64_ATOMIC_WAIT: u8 = 0x02;

		pub const I32_ATOMIC_LOAD: u8 = 0x10;
		pub const I64_ATOMIC_LOAD: u8 = 0x11;
		pub const I32_ATOMIC_LOAD8U: u8 = 0x12;
		pub const I32_ATOMIC_LOAD16U: u8 = 0x13;
		pub const I64_ATOMIC_LOAD8U: u8 = 0x14;
		pub const I64_ATOMIC_LOAD16U: u8 = 0x15;
		pub const I64_ATOMIC_LOAD32U: u8 = 0x16;
		pub const I32_ATOMIC_STORE: u8 = 0x17;
		pub const I64_ATOMIC_STORE: u8 = 0x18;
		pub const I32_ATOMIC_STORE8U: u8 = 0x19;
		pub const I32_ATOMIC_STORE16U: u8 = 0x1a;
		pub const I64_ATOMIC_STORE8U: u8 = 0x1b;
		pub const I64_ATOMIC_STORE16U: u8 = 0x1c;
		pub const I64_ATOMIC_STORE32U: u8 = 0x1d;

		pub const I32_ATOMIC_RMW_ADD: u8 = 0x1e;
		pub const I64_ATOMIC_RMW_ADD: u8 = 0x1f;
		pub const I32_ATOMIC_RMW_ADD8U: u8 = 0x20;
		pub const I32_ATOMIC_RMW_ADD16U: u8 = 0x21;
		pub const I64_ATOMIC_RMW_ADD8U: u8 = 0x22;
		pub const I64_ATOMIC_RMW_ADD16U: u8 = 0x23;
		pub const I64_ATOMIC_RMW_ADD32U: u8 = 0x24;

		pub const I32_ATOMIC_RMW_SUB: u8 = 0x25;
		pub const I64_ATOMIC_RMW_SUB: u8 = 0x26;
		pub const I32_ATOMIC_RMW_SUB8U: u8 = 0x27;
		pub const I32_ATOMIC_RMW_SUB16U: u8 = 0x28;
		pub const I64_ATOMIC_RMW_SUB8U: u8 = 0x29;
		pub const I64_ATOMIC_RMW_SUB16U: u8 = 0x2a;
		pub const I64_ATOMIC_RMW_SUB32U: u8 = 0x2b;

		pub const I32_ATOMIC_RMW_AND: u8 = 0x2c;
		pub const I64_ATOMIC_RMW_AND: u8 = 0x2d;
		pub const I32_ATOMIC_RMW_AND8U: u8 = 0x2e;
		pub const I32_ATOMIC_RMW_AND16U: u8 = 0x2f;
		pub const I64_ATOMIC_RMW_AND8U: u8 = 0x30;
		pub const I64_ATOMIC_RMW_AND16U: u8 = 0x31;
		pub const I64_ATOMIC_RMW_AND32U: u8 = 0x32;

		pub const I32_ATOMIC_RMW_OR: u8 = 0x33;
		pub const I64_ATOMIC_RMW_OR: u8 = 0x34;
		pub const I32_ATOMIC_RMW_OR8U: u8 = 0x35;
		pub const I32_ATOMIC_RMW_OR16U: u8 = 0x36;
		pub const I64_ATOMIC_RMW_OR8U: u8 = 0x37;
		pub const I64_ATOMIC_RMW_OR16U: u8 = 0x38;
		pub const I64_ATOMIC_RMW_OR32U: u8 = 0x39;

		pub const I32_ATOMIC_RMW_XOR: u8 = 0x3a;
		pub const I64_ATOMIC_RMW_XOR: u8 = 0x3b;
		pub const I32_ATOMIC_RMW_XOR8U: u8 = 0x3c;
		pub const I32_ATOMIC_RMW_XOR16U: u8 = 0x3d;
		pub const I64_ATOMIC_RMW_XOR8U: u8 = 0x3e;
		pub const I64_ATOMIC_RMW_XOR16U: u8 = 0x3f;
		pub const I64_ATOMIC_RMW_XOR32U: u8 = 0x40;

		pub const I32_ATOMIC_RMW_XCHG: u8 = 0x41;
		pub const I64_ATOMIC_RMW_XCHG: u8 = 0x42;
		pub const I32_ATOMIC_RMW_XCHG8U: u8 = 0x43;
		pub const I32_ATOMIC_RMW_XCHG16U: u8 = 0x44;
		pub const I64_ATOMIC_RMW_XCHG8U: u8 = 0x45;
		pub const I64_ATOMIC_RMW_XCHG16U: u8 = 0x46;
		pub const I64_ATOMIC_RMW_XCHG32U: u8 = 0x47;

		pub const I32_ATOMIC_RMW_CMPXCHG: u8 = 0x48;
		pub const I64_ATOMIC_RMW_CMPXCHG: u8 = 0x49;
		pub const I32_ATOMIC_RMW_CMPXCHG8U: u8 = 0x4a;
		pub const I32_ATOMIC_RMW_CMPXCHG16U: u8 = 0x4b;
		pub const I64_ATOMIC_RMW_CMPXCHG8U: u8 = 0x4c;
		pub const I64_ATOMIC_RMW_CMPXCHG16U: u8 = 0x4d;
		pub const I64_ATOMIC_RMW_CMPXCHG32U: u8 = 0x4e;
	}

	#[cfg(feature = "simd")]
	pub mod simd {
		// https://github.com/WebAssembly/simd/blob/master/proposals/simd/BinarySIMD.md
		pub const SIMD_PREFIX: u8 = 0xfd;

		pub const V128_LOAD: u32 = 0x00;
		pub const V128_STORE: u32 = 0x01;
		pub const V128_CONST: u32 = 0x02;
		pub const V8X16_SHUFFLE: u32 = 0x03;

		pub const I8X16_SPLAT: u32 = 0x04;
		pub const I8X16_EXTRACT_LANE_S: u32 = 0x05;
		pub const I8X16_EXTRACT_LANE_U: u32 = 0x06;
		pub const I8X16_REPLACE_LANE: u32 = 0x07;
		pub const I16X8_SPLAT: u32 = 0x08;
		pub const I16X8_EXTRACT_LANE_S: u32 = 0x09;
		pub const I16X8_EXTRACT_LANE_U: u32 = 0xa;
		pub const I16X8_REPLACE_LANE: u32 = 0x0b;
		pub const I32X4_SPLAT: u32 = 0x0c;
		pub const I32X4_EXTRACT_LANE: u32 = 0x0d;
		pub const I32X4_REPLACE_LANE: u32 = 0x0e;
		pub const I64X2_SPLAT: u32 = 0x0f;
		pub const I64X2_EXTRACT_LANE: u32 = 0x10;
		pub const I64X2_REPLACE_LANE: u32 = 0x11;
		pub const F32X4_SPLAT: u32 = 0x12;
		pub const F32X4_EXTRACT_LANE: u32 = 0x13;
		pub const F32X4_REPLACE_LANE: u32 = 0x14;
		pub const F64X2_SPLAT: u32 = 0x15;
		pub const F64X2_EXTRACT_LANE: u32 = 0x16;
		pub const F64X2_REPLACE_LANE: u32 = 0x17;

		pub const I8X16_EQ: u32 = 0x18;
		pub const I8X16_NE: u32 = 0x19;
		pub const I8X16_LT_S: u32 = 0x1a;
		pub const I8X16_LT_U: u32 = 0x1b;
		pub const I8X16_GT_S: u32 = 0x1c;
		pub const I8X16_GT_U: u32 = 0x1d;
		pub const I8X16_LE_S: u32 = 0x1e;
		pub const I8X16_LE_U: u32 = 0x1f;
		pub const I8X16_GE_S: u32 = 0x20;
		pub const I8X16_GE_U: u32 = 0x21;

		pub const I16X8_EQ: u32 = 0x22;
		pub const I16X8_NE: u32 = 0x23;
		pub const I16X8_LT_S: u32 = 0x24;
		pub const I16X8_LT_U: u32 = 0x25;
		pub const I16X8_GT_S: u32 = 0x26;
		pub const I16X8_GT_U: u32 = 0x27;
		pub const I16X8_LE_S: u32 = 0x28;
		pub const I16X8_LE_U: u32 = 0x29;
		pub const I16X8_GE_S: u32 = 0x2a;
		pub const I16X8_GE_U: u32 = 0x2b;

		pub const I32X4_EQ: u32 = 0x2c;
		pub const I32X4_NE: u32 = 0x2d;
		pub const I32X4_LT_S: u32 = 0x2e;
		pub const I32X4_LT_U: u32 = 0x2f;
		pub const I32X4_GT_S: u32 = 0x30;
		pub const I32X4_GT_U: u32 = 0x31;
		pub const I32X4_LE_S: u32 = 0x32;
		pub const I32X4_LE_U: u32 = 0x33;
		pub const I32X4_GE_S: u32 = 0x34;
		pub const I32X4_GE_U: u32 = 0x35;

		pub const F32X4_EQ: u32 = 0x40;
		pub const F32X4_NE: u32 = 0x41;
		pub const F32X4_LT: u32 = 0x42;
		pub const F32X4_GT: u32 = 0x43;
		pub const F32X4_LE: u32 = 0x44;
		pub const F32X4_GE: u32 = 0x45;

		pub const F64X2_EQ: u32 = 0x46;
		pub const F64X2_NE: u32 = 0x47;
		pub const F64X2_LT: u32 = 0x48;
		pub const F64X2_GT: u32 = 0x49;
		pub const F64X2_LE: u32 = 0x4a;
		pub const F64X2_GE: u32 = 0x4b;

		pub const V128_NOT: u32 = 0x4c;
		pub const V128_AND: u32 = 0x4d;
		pub const V128_OR: u32 = 0x4e;
		pub const V128_XOR: u32 = 0x4f;
		pub const V128_BITSELECT: u32 = 0x50;

		pub const I8X16_NEG: u32 = 0x51;
		pub const I8X16_ANY_TRUE: u32 = 0x52;
		pub const I8X16_ALL_TRUE: u32 = 0x53;
		pub const I8X16_SHL: u32 = 0x54;
		pub const I8X16_SHR_S: u32 = 0x55;
		pub const I8X16_SHR_U: u32 = 0x56;
		pub const I8X16_ADD: u32 = 0x57;
		pub const I8X16_ADD_SATURATE_S: u32 = 0x58;
		pub const I8X16_ADD_SATURATE_U: u32 = 0x59;
		pub const I8X16_SUB: u32 = 0x5a;
		pub const I8X16_SUB_SATURATE_S: u32 = 0x5b;
		pub const I8X16_SUB_SATURATE_U: u32 = 0x5c;
		pub const I8X16_MUL: u32 = 0x5d;

		pub const I16X8_NEG: u32 = 0x62;
		pub const I16X8_ANY_TRUE: u32 = 0x63;
		pub const I16X8_ALL_TRUE: u32 = 0x64;
		pub const I16X8_SHL: u32 = 0x65;
		pub const I16X8_SHR_S: u32 = 0x66;
		pub const I16X8_SHR_U: u32 = 0x67;
		pub const I16X8_ADD: u32 = 0x68;
		pub const I16X8_ADD_SATURATE_S: u32 = 0x69;
		pub const I16X8_ADD_SATURATE_U: u32 = 0x6a;
		pub const I16X8_SUB: u32 = 0x6b;
		pub const I16X8_SUB_SATURATE_S: u32 = 0x6c;
		pub const I16X8_SUB_SATURATE_U: u32 = 0x6d;
		pub const I16X8_MUL: u32 = 0x6e;

		pub const I32X4_NEG: u32 = 0x73;
		pub const I32X4_ANY_TRUE: u32 = 0x74;
		pub const I32X4_ALL_TRUE: u32 = 0x75;
		pub const I32X4_SHL: u32 = 0x76;
		pub const I32X4_SHR_S: u32 = 0x77;
		pub const I32X4_SHR_U: u32 = 0x78;
		pub const I32X4_ADD: u32 = 0x79;
		pub const I32X4_ADD_SATURATE_S: u32 = 0x7a;
		pub const I32X4_ADD_SATURATE_U: u32 = 0x7b;
		pub const I32X4_SUB: u32 = 0x7c;
		pub const I32X4_SUB_SATURATE_S: u32 = 0x7d;
		pub const I32X4_SUB_SATURATE_U: u32 = 0x7e;
		pub const I32X4_MUL: u32 = 0x7f;

		pub const I64X2_NEG: u32 = 0x84;
		pub const I64X2_ANY_TRUE: u32 = 0x85;
		pub const I64X2_ALL_TRUE: u32 = 0x86;
		pub const I64X2_SHL: u32 = 0x87;
		pub const I64X2_SHR_S: u32 = 0x88;
		pub const I64X2_SHR_U: u32 = 0x89;
		pub const I64X2_ADD: u32 = 0x8a;
		pub const I64X2_SUB: u32 = 0x8d;

		pub const F32X4_ABS: u32 = 0x95;
		pub const F32X4_NEG: u32 = 0x96;
		pub const F32X4_SQRT: u32 = 0x97;
		pub const F32X4_ADD: u32 = 0x9a;
		pub const F32X4_SUB: u32 = 0x9b;
		pub const F32X4_MUL: u32 = 0x9c;
		pub const F32X4_DIV: u32 = 0x9d;
		pub const F32X4_MIN: u32 = 0x9e;
		pub const F32X4_MAX: u32 = 0x9f;

		pub const F64X2_ABS: u32 = 0xa0;
		pub const F64X2_NEG: u32 = 0xa1;
		pub const F64X2_SQRT: u32 = 0xa2;
		pub const F64X2_ADD: u32 = 0xa5;
		pub const F64X2_SUB: u32 = 0xa6;
		pub const F64X2_MUL: u32 = 0xa7;
		pub const F64X2_DIV: u32 = 0xa8;
		pub const F64X2_MIN: u32 = 0xa9;
		pub const F64X2_MAX: u32 = 0xaa;

		pub const I32X4_TRUNC_S_F32X4_SAT: u32 = 0xab;
		pub const I32X4_TRUNC_U_F32X4_SAT: u32 = 0xac;
		pub const I64X2_TRUNC_S_F64X2_SAT: u32 = 0xad;
		pub const I64X2_TRUNC_U_F64X2_SAT: u32 = 0xae;

		pub const F32X4_CONVERT_S_I32X4: u32 = 0xaf;
		pub const F32X4_CONVERT_U_I32X4: u32 = 0xb0;
		pub const F64X2_CONVERT_S_I64X2: u32 = 0xb1;
		pub const F64X2_CONVERT_U_I64X2: u32 = 0xb2;
	}

	#[cfg(feature = "bulk")]
	pub mod bulk {
		pub const BULK_PREFIX: u8 = 0xfc;
		pub const MEMORY_INIT: u8 = 0x08;
		pub const MEMORY_DROP: u8 = 0x09;
		pub const MEMORY_COPY: u8 = 0x0a;
		pub const MEMORY_FILL: u8 = 0x0b;
		pub const TABLE_INIT: u8 = 0x0c;
		pub const TABLE_DROP: u8 = 0x0d;
		pub const TABLE_COPY: u8 = 0x0e;
	}
}

impl Deserialize for Instruction {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		use self::{opcodes::*, Instruction::*};

		#[cfg(feature = "sign_ext")]
		use self::opcodes::sign_ext::*;

		let val: u8 = Uint8::deserialize(reader)?.into();

		Ok(match val {
			UNREACHABLE => Unreachable,
			NOP => Nop,
			BLOCK => Block(BlockType::deserialize(reader)?),
			LOOP => Loop(BlockType::deserialize(reader)?),
			IF => If(BlockType::deserialize(reader)?),
			ELSE => Else,
			END => End,

			BR => Br(VarUint32::deserialize(reader)?.into()),
			BRIF => BrIf(VarUint32::deserialize(reader)?.into()),
			BRTABLE => {
				let t1: Vec<u32> = CountedList::<VarUint32>::deserialize(reader)?
					.into_inner()
					.into_iter()
					.map(Into::into)
					.collect();

				BrTable(Box::new(BrTableData {
					table: t1.into_boxed_slice(),
					default: VarUint32::deserialize(reader)?.into(),
				}))
			},
			RETURN => Return,
			CALL => Call(VarUint32::deserialize(reader)?.into()),
			CALLINDIRECT => {
				let signature: u32 = VarUint32::deserialize(reader)?.into();
				let table_ref: u8 = Uint8::deserialize(reader)?.into();
				if table_ref != 0 {
					return Err(Error::InvalidTableReference(table_ref))
				}

				CallIndirect(signature, table_ref)
			},
			DROP => Drop,
			SELECT => Select,

			GETLOCAL => GetLocal(VarUint32::deserialize(reader)?.into()),
			SETLOCAL => SetLocal(VarUint32::deserialize(reader)?.into()),
			TEELOCAL => TeeLocal(VarUint32::deserialize(reader)?.into()),
			GETGLOBAL => GetGlobal(VarUint32::deserialize(reader)?.into()),
			SETGLOBAL => SetGlobal(VarUint32::deserialize(reader)?.into()),

			I32LOAD => I32Load(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD => I64Load(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			F32LOAD => F32Load(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			F64LOAD => F64Load(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32LOAD8S => I32Load8S(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32LOAD8U => I32Load8U(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32LOAD16S => I32Load16S(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32LOAD16U => I32Load16U(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD8S => I64Load8S(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD8U => I64Load8U(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD16S => I64Load16S(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD16U => I64Load16U(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD32S => I64Load32S(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64LOAD32U => I64Load32U(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32STORE => I32Store(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64STORE => I64Store(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			F32STORE => F32Store(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			F64STORE => F64Store(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32STORE8 => I32Store8(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I32STORE16 => I32Store16(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64STORE8 => I64Store8(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64STORE16 => I64Store16(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			I64STORE32 => I64Store32(
				VarUint32::deserialize(reader)?.into(),
				VarUint32::deserialize(reader)?.into(),
			),

			CURRENTMEMORY => {
				let mem_ref: u8 = Uint8::deserialize(reader)?.into();
				if mem_ref != 0 {
					return Err(Error::InvalidMemoryReference(mem_ref))
				}
				CurrentMemory(mem_ref)
			},
			GROWMEMORY => {
				let mem_ref: u8 = Uint8::deserialize(reader)?.into();
				if mem_ref != 0 {
					return Err(Error::InvalidMemoryReference(mem_ref))
				}
				GrowMemory(mem_ref)
			},

			I32CONST => I32Const(VarInt32::deserialize(reader)?.into()),
			I64CONST => I64Const(VarInt64::deserialize(reader)?.into()),
			F32CONST => F32Const(Uint32::deserialize(reader)?.into()),
			F64CONST => F64Const(Uint64::deserialize(reader)?.into()),
			I32EQZ => I32Eqz,
			I32EQ => I32Eq,
			I32NE => I32Ne,
			I32LTS => I32LtS,
			I32LTU => I32LtU,
			I32GTS => I32GtS,
			I32GTU => I32GtU,
			I32LES => I32LeS,
			I32LEU => I32LeU,
			I32GES => I32GeS,
			I32GEU => I32GeU,

			I64EQZ => I64Eqz,
			I64EQ => I64Eq,
			I64NE => I64Ne,
			I64LTS => I64LtS,
			I64LTU => I64LtU,
			I64GTS => I64GtS,
			I64GTU => I64GtU,
			I64LES => I64LeS,
			I64LEU => I64LeU,
			I64GES => I64GeS,
			I64GEU => I64GeU,

			F32EQ => F32Eq,
			F32NE => F32Ne,
			F32LT => F32Lt,
			F32GT => F32Gt,
			F32LE => F32Le,
			F32GE => F32Ge,

			F64EQ => F64Eq,
			F64NE => F64Ne,
			F64LT => F64Lt,
			F64GT => F64Gt,
			F64LE => F64Le,
			F64GE => F64Ge,

			I32CLZ => I32Clz,
			I32CTZ => I32Ctz,
			I32POPCNT => I32Popcnt,
			I32ADD => I32Add,
			I32SUB => I32Sub,
			I32MUL => I32Mul,
			I32DIVS => I32DivS,
			I32DIVU => I32DivU,
			I32REMS => I32RemS,
			I32REMU => I32RemU,
			I32AND => I32And,
			I32OR => I32Or,
			I32XOR => I32Xor,
			I32SHL => I32Shl,
			I32SHRS => I32ShrS,
			I32SHRU => I32ShrU,
			I32ROTL => I32Rotl,
			I32ROTR => I32Rotr,

			I64CLZ => I64Clz,
			I64CTZ => I64Ctz,
			I64POPCNT => I64Popcnt,
			I64ADD => I64Add,
			I64SUB => I64Sub,
			I64MUL => I64Mul,
			I64DIVS => I64DivS,
			I64DIVU => I64DivU,
			I64REMS => I64RemS,
			I64REMU => I64RemU,
			I64AND => I64And,
			I64OR => I64Or,
			I64XOR => I64Xor,
			I64SHL => I64Shl,
			I64SHRS => I64ShrS,
			I64SHRU => I64ShrU,
			I64ROTL => I64Rotl,
			I64ROTR => I64Rotr,
			F32ABS => F32Abs,
			F32NEG => F32Neg,
			F32CEIL => F32Ceil,
			F32FLOOR => F32Floor,
			F32TRUNC => F32Trunc,
			F32NEAREST => F32Nearest,
			F32SQRT => F32Sqrt,
			F32ADD => F32Add,
			F32SUB => F32Sub,
			F32MUL => F32Mul,
			F32DIV => F32Div,
			F32MIN => F32Min,
			F32MAX => F32Max,
			F32COPYSIGN => F32Copysign,
			F64ABS => F64Abs,
			F64NEG => F64Neg,
			F64CEIL => F64Ceil,
			F64FLOOR => F64Floor,
			F64TRUNC => F64Trunc,
			F64NEAREST => F64Nearest,
			F64SQRT => F64Sqrt,
			F64ADD => F64Add,
			F64SUB => F64Sub,
			F64MUL => F64Mul,
			F64DIV => F64Div,
			F64MIN => F64Min,
			F64MAX => F64Max,
			F64COPYSIGN => F64Copysign,

			I32WRAPI64 => I32WrapI64,
			I32TRUNCSF32 => I32TruncSF32,
			I32TRUNCUF32 => I32TruncUF32,
			I32TRUNCSF64 => I32TruncSF64,
			I32TRUNCUF64 => I32TruncUF64,
			I64EXTENDSI32 => I64ExtendSI32,
			I64EXTENDUI32 => I64ExtendUI32,
			I64TRUNCSF32 => I64TruncSF32,
			I64TRUNCUF32 => I64TruncUF32,
			I64TRUNCSF64 => I64TruncSF64,
			I64TRUNCUF64 => I64TruncUF64,
			F32CONVERTSI32 => F32ConvertSI32,
			F32CONVERTUI32 => F32ConvertUI32,
			F32CONVERTSI64 => F32ConvertSI64,
			F32CONVERTUI64 => F32ConvertUI64,
			F32DEMOTEF64 => F32DemoteF64,
			F64CONVERTSI32 => F64ConvertSI32,
			F64CONVERTUI32 => F64ConvertUI32,
			F64CONVERTSI64 => F64ConvertSI64,
			F64CONVERTUI64 => F64ConvertUI64,
			F64PROMOTEF32 => F64PromoteF32,

			I32REINTERPRETF32 => I32ReinterpretF32,
			I64REINTERPRETF64 => I64ReinterpretF64,
			F32REINTERPRETI32 => F32ReinterpretI32,
			F64REINTERPRETI64 => F64ReinterpretI64,

			#[cfg(feature = "sign_ext")]
			I32_EXTEND8_S | I32_EXTEND16_S | I64_EXTEND8_S | I64_EXTEND16_S | I64_EXTEND32_S => match val {
				I32_EXTEND8_S => SignExt(SignExtInstruction::I32Extend8S),
				I32_EXTEND16_S => SignExt(SignExtInstruction::I32Extend16S),
				I64_EXTEND8_S => SignExt(SignExtInstruction::I64Extend8S),
				I64_EXTEND16_S => SignExt(SignExtInstruction::I64Extend16S),
				I64_EXTEND32_S => SignExt(SignExtInstruction::I64Extend32S),
				_ => return Err(Error::UnknownOpcode(val)),
			},

			#[cfg(feature = "atomics")]
			atomics::ATOMIC_PREFIX => return deserialize_atomic(reader),

			#[cfg(feature = "simd")]
			simd::SIMD_PREFIX => return deserialize_simd(reader),

			#[cfg(feature = "bulk")]
			bulk::BULK_PREFIX => return deserialize_bulk(reader),

			_ => return Err(Error::UnknownOpcode(val)),
		})
	}
}

#[cfg(feature = "atomics")]
fn deserialize_atomic<R: io::Read>(reader: &mut R) -> Result<Instruction, Error> {
	use self::{opcodes::atomics::*, AtomicsInstruction::*};

	let val: u8 = Uint8::deserialize(reader)?.into();
	let mem = MemArg::deserialize(reader)?;
	Ok(Instruction::Atomics(match val {
		ATOMIC_WAKE => AtomicWake(mem),
		I32_ATOMIC_WAIT => I32AtomicWait(mem),
		I64_ATOMIC_WAIT => I64AtomicWait(mem),

		I32_ATOMIC_LOAD => I32AtomicLoad(mem),
		I64_ATOMIC_LOAD => I64AtomicLoad(mem),
		I32_ATOMIC_LOAD8U => I32AtomicLoad8u(mem),
		I32_ATOMIC_LOAD16U => I32AtomicLoad16u(mem),
		I64_ATOMIC_LOAD8U => I64AtomicLoad8u(mem),
		I64_ATOMIC_LOAD16U => I64AtomicLoad16u(mem),
		I64_ATOMIC_LOAD32U => I64AtomicLoad32u(mem),
		I32_ATOMIC_STORE => I32AtomicStore(mem),
		I64_ATOMIC_STORE => I64AtomicStore(mem),
		I32_ATOMIC_STORE8U => I32AtomicStore8u(mem),
		I32_ATOMIC_STORE16U => I32AtomicStore16u(mem),
		I64_ATOMIC_STORE8U => I64AtomicStore8u(mem),
		I64_ATOMIC_STORE16U => I64AtomicStore16u(mem),
		I64_ATOMIC_STORE32U => I64AtomicStore32u(mem),

		I32_ATOMIC_RMW_ADD => I32AtomicRmwAdd(mem),
		I64_ATOMIC_RMW_ADD => I64AtomicRmwAdd(mem),
		I32_ATOMIC_RMW_ADD8U => I32AtomicRmwAdd8u(mem),
		I32_ATOMIC_RMW_ADD16U => I32AtomicRmwAdd16u(mem),
		I64_ATOMIC_RMW_ADD8U => I64AtomicRmwAdd8u(mem),
		I64_ATOMIC_RMW_ADD16U => I64AtomicRmwAdd16u(mem),
		I64_ATOMIC_RMW_ADD32U => I64AtomicRmwAdd32u(mem),

		I32_ATOMIC_RMW_SUB => I32AtomicRmwSub(mem),
		I64_ATOMIC_RMW_SUB => I64AtomicRmwSub(mem),
		I32_ATOMIC_RMW_SUB8U => I32AtomicRmwSub8u(mem),
		I32_ATOMIC_RMW_SUB16U => I32AtomicRmwSub16u(mem),
		I64_ATOMIC_RMW_SUB8U => I64AtomicRmwSub8u(mem),
		I64_ATOMIC_RMW_SUB16U => I64AtomicRmwSub16u(mem),
		I64_ATOMIC_RMW_SUB32U => I64AtomicRmwSub32u(mem),

		I32_ATOMIC_RMW_AND => I32AtomicRmwAnd(mem),
		I64_ATOMIC_RMW_AND => I64AtomicRmwAnd(mem),
		I32_ATOMIC_RMW_AND8U => I32AtomicRmwAnd8u(mem),
		I32_ATOMIC_RMW_AND16U => I32AtomicRmwAnd16u(mem),
		I64_ATOMIC_RMW_AND8U => I64AtomicRmwAnd8u(mem),
		I64_ATOMIC_RMW_AND16U => I64AtomicRmwAnd16u(mem),
		I64_ATOMIC_RMW_AND32U => I64AtomicRmwAnd32u(mem),

		I32_ATOMIC_RMW_OR => I32AtomicRmwOr(mem),
		I64_ATOMIC_RMW_OR => I64AtomicRmwOr(mem),
		I32_ATOMIC_RMW_OR8U => I32AtomicRmwOr8u(mem),
		I32_ATOMIC_RMW_OR16U => I32AtomicRmwOr16u(mem),
		I64_ATOMIC_RMW_OR8U => I64AtomicRmwOr8u(mem),
		I64_ATOMIC_RMW_OR16U => I64AtomicRmwOr16u(mem),
		I64_ATOMIC_RMW_OR32U => I64AtomicRmwOr32u(mem),

		I32_ATOMIC_RMW_XOR => I32AtomicRmwXor(mem),
		I64_ATOMIC_RMW_XOR => I64AtomicRmwXor(mem),
		I32_ATOMIC_RMW_XOR8U => I32AtomicRmwXor8u(mem),
		I32_ATOMIC_RMW_XOR16U => I32AtomicRmwXor16u(mem),
		I64_ATOMIC_RMW_XOR8U => I64AtomicRmwXor8u(mem),
		I64_ATOMIC_RMW_XOR16U => I64AtomicRmwXor16u(mem),
		I64_ATOMIC_RMW_XOR32U => I64AtomicRmwXor32u(mem),

		I32_ATOMIC_RMW_XCHG => I32AtomicRmwXchg(mem),
		I64_ATOMIC_RMW_XCHG => I64AtomicRmwXchg(mem),
		I32_ATOMIC_RMW_XCHG8U => I32AtomicRmwXchg8u(mem),
		I32_ATOMIC_RMW_XCHG16U => I32AtomicRmwXchg16u(mem),
		I64_ATOMIC_RMW_XCHG8U => I64AtomicRmwXchg8u(mem),
		I64_ATOMIC_RMW_XCHG16U => I64AtomicRmwXchg16u(mem),
		I64_ATOMIC_RMW_XCHG32U => I64AtomicRmwXchg32u(mem),

		I32_ATOMIC_RMW_CMPXCHG => I32AtomicRmwCmpxchg(mem),
		I64_ATOMIC_RMW_CMPXCHG => I64AtomicRmwCmpxchg(mem),
		I32_ATOMIC_RMW_CMPXCHG8U => I32AtomicRmwCmpxchg8u(mem),
		I32_ATOMIC_RMW_CMPXCHG16U => I32AtomicRmwCmpxchg16u(mem),
		I64_ATOMIC_RMW_CMPXCHG8U => I64AtomicRmwCmpxchg8u(mem),
		I64_ATOMIC_RMW_CMPXCHG16U => I64AtomicRmwCmpxchg16u(mem),
		I64_ATOMIC_RMW_CMPXCHG32U => I64AtomicRmwCmpxchg32u(mem),

		_ => return Err(Error::UnknownOpcode(val)),
	}))
}

#[cfg(feature = "simd")]
fn deserialize_simd<R: io::Read>(reader: &mut R) -> Result<Instruction, Error> {
	use self::{opcodes::simd::*, SimdInstruction::*};

	let val = VarUint32::deserialize(reader)?.into();
	Ok(Instruction::Simd(match val {
		V128_CONST => {
			let mut buf = [0; 16];
			reader.read(&mut buf)?;
			V128Const(Box::new(buf))
		},
		V128_LOAD => V128Load(MemArg::deserialize(reader)?),
		V128_STORE => V128Store(MemArg::deserialize(reader)?),
		I8X16_SPLAT => I8x16Splat,
		I16X8_SPLAT => I16x8Splat,
		I32X4_SPLAT => I32x4Splat,
		I64X2_SPLAT => I64x2Splat,
		F32X4_SPLAT => F32x4Splat,
		F64X2_SPLAT => F64x2Splat,
		I8X16_EXTRACT_LANE_S => I8x16ExtractLaneS(Uint8::deserialize(reader)?.into()),
		I8X16_EXTRACT_LANE_U => I8x16ExtractLaneU(Uint8::deserialize(reader)?.into()),
		I16X8_EXTRACT_LANE_S => I16x8ExtractLaneS(Uint8::deserialize(reader)?.into()),
		I16X8_EXTRACT_LANE_U => I16x8ExtractLaneU(Uint8::deserialize(reader)?.into()),
		I32X4_EXTRACT_LANE => I32x4ExtractLane(Uint8::deserialize(reader)?.into()),
		I64X2_EXTRACT_LANE => I64x2ExtractLane(Uint8::deserialize(reader)?.into()),
		F32X4_EXTRACT_LANE => F32x4ExtractLane(Uint8::deserialize(reader)?.into()),
		F64X2_EXTRACT_LANE => F64x2ExtractLane(Uint8::deserialize(reader)?.into()),
		I8X16_REPLACE_LANE => I8x16ReplaceLane(Uint8::deserialize(reader)?.into()),
		I16X8_REPLACE_LANE => I16x8ReplaceLane(Uint8::deserialize(reader)?.into()),
		I32X4_REPLACE_LANE => I32x4ReplaceLane(Uint8::deserialize(reader)?.into()),
		I64X2_REPLACE_LANE => I64x2ReplaceLane(Uint8::deserialize(reader)?.into()),
		F32X4_REPLACE_LANE => F32x4ReplaceLane(Uint8::deserialize(reader)?.into()),
		F64X2_REPLACE_LANE => F64x2ReplaceLane(Uint8::deserialize(reader)?.into()),
		V8X16_SHUFFLE => {
			let mut buf = [0; 16];
			reader.read(&mut buf)?;
			V8x16Shuffle(Box::new(buf))
		},
		I8X16_ADD => I8x16Add,
		I16X8_ADD => I16x8Add,
		I32X4_ADD => I32x4Add,
		I64X2_ADD => I64x2Add,
		I8X16_SUB => I8x16Sub,
		I16X8_SUB => I16x8Sub,
		I32X4_SUB => I32x4Sub,
		I64X2_SUB => I64x2Sub,
		I8X16_MUL => I8x16Mul,
		I16X8_MUL => I16x8Mul,
		I32X4_MUL => I32x4Mul,
		// I64X2_MUL => I64x2Mul,
		I8X16_NEG => I8x16Neg,
		I16X8_NEG => I16x8Neg,
		I32X4_NEG => I32x4Neg,
		I64X2_NEG => I64x2Neg,

		I8X16_ADD_SATURATE_S => I8x16AddSaturateS,
		I8X16_ADD_SATURATE_U => I8x16AddSaturateU,
		I16X8_ADD_SATURATE_S => I16x8AddSaturateS,
		I16X8_ADD_SATURATE_U => I16x8AddSaturateU,
		I8X16_SUB_SATURATE_S => I8x16SubSaturateS,
		I8X16_SUB_SATURATE_U => I8x16SubSaturateU,
		I16X8_SUB_SATURATE_S => I16x8SubSaturateS,
		I16X8_SUB_SATURATE_U => I16x8SubSaturateU,
		I8X16_SHL => I8x16Shl,
		I16X8_SHL => I16x8Shl,
		I32X4_SHL => I32x4Shl,
		I64X2_SHL => I64x2Shl,
		I8X16_SHR_S => I8x16ShrS,
		I8X16_SHR_U => I8x16ShrU,
		I16X8_SHR_S => I16x8ShrS,
		I16X8_SHR_U => I16x8ShrU,
		I32X4_SHR_S => I32x4ShrS,
		I32X4_SHR_U => I32x4ShrU,
		I64X2_SHR_S => I64x2ShrS,
		I64X2_SHR_U => I64x2ShrU,
		V128_AND => V128And,
		V128_OR => V128Or,
		V128_XOR => V128Xor,
		V128_NOT => V128Not,
		V128_BITSELECT => V128Bitselect,
		I8X16_ANY_TRUE => I8x16AnyTrue,
		I16X8_ANY_TRUE => I16x8AnyTrue,
		I32X4_ANY_TRUE => I32x4AnyTrue,
		I64X2_ANY_TRUE => I64x2AnyTrue,
		I8X16_ALL_TRUE => I8x16AllTrue,
		I16X8_ALL_TRUE => I16x8AllTrue,
		I32X4_ALL_TRUE => I32x4AllTrue,
		I64X2_ALL_TRUE => I64x2AllTrue,
		I8X16_EQ => I8x16Eq,
		I16X8_EQ => I16x8Eq,
		I32X4_EQ => I32x4Eq,
		// I64X2_EQ => I64x2Eq,
		F32X4_EQ => F32x4Eq,
		F64X2_EQ => F64x2Eq,
		I8X16_NE => I8x16Ne,
		I16X8_NE => I16x8Ne,
		I32X4_NE => I32x4Ne,
		// I64X2_NE => I64x2Ne,
		F32X4_NE => F32x4Ne,
		F64X2_NE => F64x2Ne,
		I8X16_LT_S => I8x16LtS,
		I8X16_LT_U => I8x16LtU,
		I16X8_LT_S => I16x8LtS,
		I16X8_LT_U => I16x8LtU,
		I32X4_LT_S => I32x4LtS,
		I32X4_LT_U => I32x4LtU,
		// I64X2_LT_S => I64x2LtS,
		// I64X2_LT_U => I64x2LtU,
		F32X4_LT => F32x4Lt,
		F64X2_LT => F64x2Lt,
		I8X16_LE_S => I8x16LeS,
		I8X16_LE_U => I8x16LeU,
		I16X8_LE_S => I16x8LeS,
		I16X8_LE_U => I16x8LeU,
		I32X4_LE_S => I32x4LeS,
		I32X4_LE_U => I32x4LeU,
		// I64X2_LE_S => I64x2LeS,
		// I64X2_LE_U => I64x2LeU,
		F32X4_LE => F32x4Le,
		F64X2_LE => F64x2Le,
		I8X16_GT_S => I8x16GtS,
		I8X16_GT_U => I8x16GtU,
		I16X8_GT_S => I16x8GtS,
		I16X8_GT_U => I16x8GtU,
		I32X4_GT_S => I32x4GtS,
		I32X4_GT_U => I32x4GtU,
		// I64X2_GT_S => I64x2GtS,
		// I64X2_GT_U => I64x2GtU,
		F32X4_GT => F32x4Gt,
		F64X2_GT => F64x2Gt,
		I8X16_GE_S => I8x16GeS,
		I8X16_GE_U => I8x16GeU,
		I16X8_GE_S => I16x8GeS,
		I16X8_GE_U => I16x8GeU,
		I32X4_GE_S => I32x4GeS,
		I32X4_GE_U => I32x4GeU,
		// I64X2_GE_S => I64x2GeS,
		// I64X2_GE_U => I64x2GeU,
		F32X4_GE => F32x4Ge,
		F64X2_GE => F64x2Ge,
		F32X4_NEG => F32x4Neg,
		F64X2_NEG => F64x2Neg,
		F32X4_ABS => F32x4Abs,
		F64X2_ABS => F64x2Abs,
		F32X4_MIN => F32x4Min,
		F64X2_MIN => F64x2Min,
		F32X4_MAX => F32x4Max,
		F64X2_MAX => F64x2Max,
		F32X4_ADD => F32x4Add,
		F64X2_ADD => F64x2Add,
		F32X4_SUB => F32x4Sub,
		F64X2_SUB => F64x2Sub,
		F32X4_DIV => F32x4Div,
		F64X2_DIV => F64x2Div,
		F32X4_MUL => F32x4Mul,
		F64X2_MUL => F64x2Mul,
		F32X4_SQRT => F32x4Sqrt,
		F64X2_SQRT => F64x2Sqrt,
		F32X4_CONVERT_S_I32X4 => F32x4ConvertSI32x4,
		F32X4_CONVERT_U_I32X4 => F32x4ConvertUI32x4,
		F64X2_CONVERT_S_I64X2 => F64x2ConvertSI64x2,
		F64X2_CONVERT_U_I64X2 => F64x2ConvertUI64x2,
		I32X4_TRUNC_S_F32X4_SAT => I32x4TruncSF32x4Sat,
		I32X4_TRUNC_U_F32X4_SAT => I32x4TruncUF32x4Sat,
		I64X2_TRUNC_S_F64X2_SAT => I64x2TruncSF64x2Sat,
		I64X2_TRUNC_U_F64X2_SAT => I64x2TruncUF64x2Sat,

		_ => return Err(Error::UnknownSimdOpcode(val)),
	}))
}

#[cfg(feature = "bulk")]
fn deserialize_bulk<R: io::Read>(reader: &mut R) -> Result<Instruction, Error> {
	use self::{opcodes::bulk::*, BulkInstruction::*};

	let val: u8 = Uint8::deserialize(reader)?.into();
	Ok(Instruction::Bulk(match val {
		MEMORY_INIT => {
			if u8::from(Uint8::deserialize(reader)?) != 0 {
				return Err(Error::UnknownOpcode(val))
			}
			MemoryInit(VarUint32::deserialize(reader)?.into())
		},
		MEMORY_DROP => MemoryDrop(VarUint32::deserialize(reader)?.into()),
		MEMORY_FILL => {
			if u8::from(Uint8::deserialize(reader)?) != 0 {
				return Err(Error::UnknownOpcode(val))
			}
			MemoryFill
		},
		MEMORY_COPY => {
			if u8::from(Uint8::deserialize(reader)?) != 0 {
				return Err(Error::UnknownOpcode(val))
			}
			MemoryCopy
		},

		TABLE_INIT => {
			if u8::from(Uint8::deserialize(reader)?) != 0 {
				return Err(Error::UnknownOpcode(val))
			}
			TableInit(VarUint32::deserialize(reader)?.into())
		},
		TABLE_DROP => TableDrop(VarUint32::deserialize(reader)?.into()),
		TABLE_COPY => {
			if u8::from(Uint8::deserialize(reader)?) != 0 {
				return Err(Error::UnknownOpcode(val))
			}
			TableCopy
		},

		_ => return Err(Error::UnknownOpcode(val)),
	}))
}

#[cfg(any(feature = "simd", feature = "atomics"))]
impl Deserialize for MemArg {
	type Error = Error;

	fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
		let align = Uint8::deserialize(reader)?;
		let offset = VarUint32::deserialize(reader)?;
		Ok(MemArg { align: align.into(), offset: offset.into() })
	}
}

macro_rules! op {
	($writer: expr, $byte: expr) => {{
		let b: u8 = $byte;
		$writer.write(&[b])?;
	}};
	($writer: expr, $byte: expr, $s: block) => {{
		op!($writer, $byte);
		$s;
	}};
}

#[cfg(feature = "atomics")]
macro_rules! atomic {
	($writer: expr, $byte: expr, $mem:expr) => {{
		$writer.write(&[ATOMIC_PREFIX, $byte])?;
		MemArg::serialize($mem, $writer)?;
	}};
}

#[cfg(feature = "simd")]
macro_rules! simd {
	($writer: expr, $byte: expr, $other:expr) => {{
		$writer.write(&[SIMD_PREFIX])?;
		VarUint32::from($byte).serialize($writer)?;
		$other;
	}};
}

#[cfg(feature = "bulk")]
macro_rules! bulk {
	($writer: expr, $byte: expr) => {{
		$writer.write(&[BULK_PREFIX, $byte])?;
	}};
	($writer: expr, $byte: expr, $remaining:expr) => {{
		bulk!($writer, $byte);
		$remaining;
	}};
}

impl Serialize for Instruction {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::{opcodes::*, Instruction::*};

		match self {
			Unreachable => op!(writer, UNREACHABLE),
			Nop => op!(writer, NOP),
			Block(block_type) => op!(writer, BLOCK, {
				block_type.serialize(writer)?;
			}),
			Loop(block_type) => op!(writer, LOOP, {
				block_type.serialize(writer)?;
			}),
			If(block_type) => op!(writer, IF, {
				block_type.serialize(writer)?;
			}),
			Else => op!(writer, ELSE),
			End => op!(writer, END),
			Br(idx) => op!(writer, BR, {
				VarUint32::from(idx).serialize(writer)?;
			}),
			BrIf(idx) => op!(writer, BRIF, {
				VarUint32::from(idx).serialize(writer)?;
			}),
			BrTable(ref table) => op!(writer, BRTABLE, {
				let list_writer = CountedListWriter::<VarUint32, _>(
					table.table.len(),
					table.table.iter().map(|x| VarUint32::from(*x)),
				);
				list_writer.serialize(writer)?;
				VarUint32::from(table.default).serialize(writer)?;
			}),
			Return => op!(writer, RETURN),
			Call(index) => op!(writer, CALL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			CallIndirect(index, reserved) => op!(writer, CALLINDIRECT, {
				VarUint32::from(index).serialize(writer)?;
				Uint8::from(reserved).serialize(writer)?;
			}),
			Drop => op!(writer, DROP),
			Select => op!(writer, SELECT),
			GetLocal(index) => op!(writer, GETLOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			SetLocal(index) => op!(writer, SETLOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			TeeLocal(index) => op!(writer, TEELOCAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			GetGlobal(index) => op!(writer, GETGLOBAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			SetGlobal(index) => op!(writer, SETGLOBAL, {
				VarUint32::from(index).serialize(writer)?;
			}),
			I32Load(flags, offset) => op!(writer, I32LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load(flags, offset) => op!(writer, I64LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F32Load(flags, offset) => op!(writer, F32LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F64Load(flags, offset) => op!(writer, F64LOAD, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load8S(flags, offset) => op!(writer, I32LOAD8S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load8U(flags, offset) => op!(writer, I32LOAD8U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load16S(flags, offset) => op!(writer, I32LOAD16S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Load16U(flags, offset) => op!(writer, I32LOAD16U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load8S(flags, offset) => op!(writer, I64LOAD8S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load8U(flags, offset) => op!(writer, I64LOAD8U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load16S(flags, offset) => op!(writer, I64LOAD16S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load16U(flags, offset) => op!(writer, I64LOAD16U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load32S(flags, offset) => op!(writer, I64LOAD32S, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Load32U(flags, offset) => op!(writer, I64LOAD32U, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store(flags, offset) => op!(writer, I32STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store(flags, offset) => op!(writer, I64STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F32Store(flags, offset) => op!(writer, F32STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			F64Store(flags, offset) => op!(writer, F64STORE, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store8(flags, offset) => op!(writer, I32STORE8, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I32Store16(flags, offset) => op!(writer, I32STORE16, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store8(flags, offset) => op!(writer, I64STORE8, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store16(flags, offset) => op!(writer, I64STORE16, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			I64Store32(flags, offset) => op!(writer, I64STORE32, {
				VarUint32::from(flags).serialize(writer)?;
				VarUint32::from(offset).serialize(writer)?;
			}),
			CurrentMemory(flag) => op!(writer, CURRENTMEMORY, {
				Uint8::from(flag).serialize(writer)?;
			}),
			GrowMemory(flag) => op!(writer, GROWMEMORY, {
				Uint8::from(flag).serialize(writer)?;
			}),
			I32Const(def) => op!(writer, I32CONST, {
				VarInt32::from(def).serialize(writer)?;
			}),
			I64Const(def) => op!(writer, I64CONST, {
				VarInt64::from(def).serialize(writer)?;
			}),
			F32Const(def) => op!(writer, F32CONST, {
				Uint32::from(def).serialize(writer)?;
			}),
			F64Const(def) => op!(writer, F64CONST, {
				Uint64::from(def).serialize(writer)?;
			}),
			I32Eqz => op!(writer, I32EQZ),
			I32Eq => op!(writer, I32EQ),
			I32Ne => op!(writer, I32NE),
			I32LtS => op!(writer, I32LTS),
			I32LtU => op!(writer, I32LTU),
			I32GtS => op!(writer, I32GTS),
			I32GtU => op!(writer, I32GTU),
			I32LeS => op!(writer, I32LES),
			I32LeU => op!(writer, I32LEU),
			I32GeS => op!(writer, I32GES),
			I32GeU => op!(writer, I32GEU),

			I64Eqz => op!(writer, I64EQZ),
			I64Eq => op!(writer, I64EQ),
			I64Ne => op!(writer, I64NE),
			I64LtS => op!(writer, I64LTS),
			I64LtU => op!(writer, I64LTU),
			I64GtS => op!(writer, I64GTS),
			I64GtU => op!(writer, I64GTU),
			I64LeS => op!(writer, I64LES),
			I64LeU => op!(writer, I64LEU),
			I64GeS => op!(writer, I64GES),
			I64GeU => op!(writer, I64GEU),

			F32Eq => op!(writer, F32EQ),
			F32Ne => op!(writer, F32NE),
			F32Lt => op!(writer, F32LT),
			F32Gt => op!(writer, F32GT),
			F32Le => op!(writer, F32LE),
			F32Ge => op!(writer, F32GE),

			F64Eq => op!(writer, F64EQ),
			F64Ne => op!(writer, F64NE),
			F64Lt => op!(writer, F64LT),
			F64Gt => op!(writer, F64GT),
			F64Le => op!(writer, F64LE),
			F64Ge => op!(writer, F64GE),

			I32Clz => op!(writer, I32CLZ),
			I32Ctz => op!(writer, I32CTZ),
			I32Popcnt => op!(writer, I32POPCNT),
			I32Add => op!(writer, I32ADD),
			I32Sub => op!(writer, I32SUB),
			I32Mul => op!(writer, I32MUL),
			I32DivS => op!(writer, I32DIVS),
			I32DivU => op!(writer, I32DIVU),
			I32RemS => op!(writer, I32REMS),
			I32RemU => op!(writer, I32REMU),
			I32And => op!(writer, I32AND),
			I32Or => op!(writer, I32OR),
			I32Xor => op!(writer, I32XOR),
			I32Shl => op!(writer, I32SHL),
			I32ShrS => op!(writer, I32SHRS),
			I32ShrU => op!(writer, I32SHRU),
			I32Rotl => op!(writer, I32ROTL),
			I32Rotr => op!(writer, I32ROTR),

			I64Clz => op!(writer, I64CLZ),
			I64Ctz => op!(writer, I64CTZ),
			I64Popcnt => op!(writer, I64POPCNT),
			I64Add => op!(writer, I64ADD),
			I64Sub => op!(writer, I64SUB),
			I64Mul => op!(writer, I64MUL),
			I64DivS => op!(writer, I64DIVS),
			I64DivU => op!(writer, I64DIVU),
			I64RemS => op!(writer, I64REMS),
			I64RemU => op!(writer, I64REMU),
			I64And => op!(writer, I64AND),
			I64Or => op!(writer, I64OR),
			I64Xor => op!(writer, I64XOR),
			I64Shl => op!(writer, I64SHL),
			I64ShrS => op!(writer, I64SHRS),
			I64ShrU => op!(writer, I64SHRU),
			I64Rotl => op!(writer, I64ROTL),
			I64Rotr => op!(writer, I64ROTR),
			F32Abs => op!(writer, F32ABS),
			F32Neg => op!(writer, F32NEG),
			F32Ceil => op!(writer, F32CEIL),
			F32Floor => op!(writer, F32FLOOR),
			F32Trunc => op!(writer, F32TRUNC),
			F32Nearest => op!(writer, F32NEAREST),
			F32Sqrt => op!(writer, F32SQRT),
			F32Add => op!(writer, F32ADD),
			F32Sub => op!(writer, F32SUB),
			F32Mul => op!(writer, F32MUL),
			F32Div => op!(writer, F32DIV),
			F32Min => op!(writer, F32MIN),
			F32Max => op!(writer, F32MAX),
			F32Copysign => op!(writer, F32COPYSIGN),
			F64Abs => op!(writer, F64ABS),
			F64Neg => op!(writer, F64NEG),
			F64Ceil => op!(writer, F64CEIL),
			F64Floor => op!(writer, F64FLOOR),
			F64Trunc => op!(writer, F64TRUNC),
			F64Nearest => op!(writer, F64NEAREST),
			F64Sqrt => op!(writer, F64SQRT),
			F64Add => op!(writer, F64ADD),
			F64Sub => op!(writer, F64SUB),
			F64Mul => op!(writer, F64MUL),
			F64Div => op!(writer, F64DIV),
			F64Min => op!(writer, F64MIN),
			F64Max => op!(writer, F64MAX),
			F64Copysign => op!(writer, F64COPYSIGN),

			I32WrapI64 => op!(writer, I32WRAPI64),
			I32TruncSF32 => op!(writer, I32TRUNCSF32),
			I32TruncUF32 => op!(writer, I32TRUNCUF32),
			I32TruncSF64 => op!(writer, I32TRUNCSF64),
			I32TruncUF64 => op!(writer, I32TRUNCUF64),
			I64ExtendSI32 => op!(writer, I64EXTENDSI32),
			I64ExtendUI32 => op!(writer, I64EXTENDUI32),
			I64TruncSF32 => op!(writer, I64TRUNCSF32),
			I64TruncUF32 => op!(writer, I64TRUNCUF32),
			I64TruncSF64 => op!(writer, I64TRUNCSF64),
			I64TruncUF64 => op!(writer, I64TRUNCUF64),
			F32ConvertSI32 => op!(writer, F32CONVERTSI32),
			F32ConvertUI32 => op!(writer, F32CONVERTUI32),
			F32ConvertSI64 => op!(writer, F32CONVERTSI64),
			F32ConvertUI64 => op!(writer, F32CONVERTUI64),
			F32DemoteF64 => op!(writer, F32DEMOTEF64),
			F64ConvertSI32 => op!(writer, F64CONVERTSI32),
			F64ConvertUI32 => op!(writer, F64CONVERTUI32),
			F64ConvertSI64 => op!(writer, F64CONVERTSI64),
			F64ConvertUI64 => op!(writer, F64CONVERTUI64),
			F64PromoteF32 => op!(writer, F64PROMOTEF32),

			I32ReinterpretF32 => op!(writer, I32REINTERPRETF32),
			I64ReinterpretF64 => op!(writer, I64REINTERPRETF64),
			F32ReinterpretI32 => op!(writer, F32REINTERPRETI32),
			F64ReinterpretI64 => op!(writer, F64REINTERPRETI64),

			#[cfg(feature = "sign_ext")]
			SignExt(ref a) => match *a {
				SignExtInstruction::I32Extend8S => op!(writer, sign_ext::I32_EXTEND8_S),
				SignExtInstruction::I32Extend16S => op!(writer, sign_ext::I32_EXTEND16_S),
				SignExtInstruction::I64Extend8S => op!(writer, sign_ext::I64_EXTEND8_S),
				SignExtInstruction::I64Extend16S => op!(writer, sign_ext::I64_EXTEND16_S),
				SignExtInstruction::I64Extend32S => op!(writer, sign_ext::I64_EXTEND32_S),
			},

			#[cfg(feature = "atomics")]
			Atomics(a) => return a.serialize(writer),

			#[cfg(feature = "simd")]
			Simd(a) => return a.serialize(writer),

			#[cfg(feature = "bulk")]
			Bulk(a) => return a.serialize(writer),
		}

		Ok(())
	}
}

#[cfg(feature = "atomics")]
impl Serialize for AtomicsInstruction {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::{opcodes::atomics::*, AtomicsInstruction::*};

		match self {
			AtomicWake(m) => atomic!(writer, ATOMIC_WAKE, m),
			I32AtomicWait(m) => atomic!(writer, I32_ATOMIC_WAIT, m),
			I64AtomicWait(m) => atomic!(writer, I64_ATOMIC_WAIT, m),

			I32AtomicLoad(m) => atomic!(writer, I32_ATOMIC_LOAD, m),
			I64AtomicLoad(m) => atomic!(writer, I64_ATOMIC_LOAD, m),
			I32AtomicLoad8u(m) => atomic!(writer, I32_ATOMIC_LOAD8U, m),
			I32AtomicLoad16u(m) => atomic!(writer, I32_ATOMIC_LOAD16U, m),
			I64AtomicLoad8u(m) => atomic!(writer, I64_ATOMIC_LOAD8U, m),
			I64AtomicLoad16u(m) => atomic!(writer, I64_ATOMIC_LOAD16U, m),
			I64AtomicLoad32u(m) => atomic!(writer, I64_ATOMIC_LOAD32U, m),
			I32AtomicStore(m) => atomic!(writer, I32_ATOMIC_STORE, m),
			I64AtomicStore(m) => atomic!(writer, I64_ATOMIC_STORE, m),
			I32AtomicStore8u(m) => atomic!(writer, I32_ATOMIC_STORE8U, m),
			I32AtomicStore16u(m) => atomic!(writer, I32_ATOMIC_STORE16U, m),
			I64AtomicStore8u(m) => atomic!(writer, I64_ATOMIC_STORE8U, m),
			I64AtomicStore16u(m) => atomic!(writer, I64_ATOMIC_STORE16U, m),
			I64AtomicStore32u(m) => atomic!(writer, I64_ATOMIC_STORE32U, m),

			I32AtomicRmwAdd(m) => atomic!(writer, I32_ATOMIC_RMW_ADD, m),
			I64AtomicRmwAdd(m) => atomic!(writer, I64_ATOMIC_RMW_ADD, m),
			I32AtomicRmwAdd8u(m) => atomic!(writer, I32_ATOMIC_RMW_ADD8U, m),
			I32AtomicRmwAdd16u(m) => atomic!(writer, I32_ATOMIC_RMW_ADD16U, m),
			I64AtomicRmwAdd8u(m) => atomic!(writer, I64_ATOMIC_RMW_ADD8U, m),
			I64AtomicRmwAdd16u(m) => atomic!(writer, I64_ATOMIC_RMW_ADD16U, m),
			I64AtomicRmwAdd32u(m) => atomic!(writer, I64_ATOMIC_RMW_ADD32U, m),

			I32AtomicRmwSub(m) => atomic!(writer, I32_ATOMIC_RMW_SUB, m),
			I64AtomicRmwSub(m) => atomic!(writer, I64_ATOMIC_RMW_SUB, m),
			I32AtomicRmwSub8u(m) => atomic!(writer, I32_ATOMIC_RMW_SUB8U, m),
			I32AtomicRmwSub16u(m) => atomic!(writer, I32_ATOMIC_RMW_SUB16U, m),
			I64AtomicRmwSub8u(m) => atomic!(writer, I64_ATOMIC_RMW_SUB8U, m),
			I64AtomicRmwSub16u(m) => atomic!(writer, I64_ATOMIC_RMW_SUB16U, m),
			I64AtomicRmwSub32u(m) => atomic!(writer, I64_ATOMIC_RMW_SUB32U, m),

			I32AtomicRmwAnd(m) => atomic!(writer, I32_ATOMIC_RMW_AND, m),
			I64AtomicRmwAnd(m) => atomic!(writer, I64_ATOMIC_RMW_AND, m),
			I32AtomicRmwAnd8u(m) => atomic!(writer, I32_ATOMIC_RMW_AND8U, m),
			I32AtomicRmwAnd16u(m) => atomic!(writer, I32_ATOMIC_RMW_AND16U, m),
			I64AtomicRmwAnd8u(m) => atomic!(writer, I64_ATOMIC_RMW_AND8U, m),
			I64AtomicRmwAnd16u(m) => atomic!(writer, I64_ATOMIC_RMW_AND16U, m),
			I64AtomicRmwAnd32u(m) => atomic!(writer, I64_ATOMIC_RMW_AND32U, m),

			I32AtomicRmwOr(m) => atomic!(writer, I32_ATOMIC_RMW_OR, m),
			I64AtomicRmwOr(m) => atomic!(writer, I64_ATOMIC_RMW_OR, m),
			I32AtomicRmwOr8u(m) => atomic!(writer, I32_ATOMIC_RMW_OR8U, m),
			I32AtomicRmwOr16u(m) => atomic!(writer, I32_ATOMIC_RMW_OR16U, m),
			I64AtomicRmwOr8u(m) => atomic!(writer, I64_ATOMIC_RMW_OR8U, m),
			I64AtomicRmwOr16u(m) => atomic!(writer, I64_ATOMIC_RMW_OR16U, m),
			I64AtomicRmwOr32u(m) => atomic!(writer, I64_ATOMIC_RMW_OR32U, m),

			I32AtomicRmwXor(m) => atomic!(writer, I32_ATOMIC_RMW_XOR, m),
			I64AtomicRmwXor(m) => atomic!(writer, I64_ATOMIC_RMW_XOR, m),
			I32AtomicRmwXor8u(m) => atomic!(writer, I32_ATOMIC_RMW_XOR8U, m),
			I32AtomicRmwXor16u(m) => atomic!(writer, I32_ATOMIC_RMW_XOR16U, m),
			I64AtomicRmwXor8u(m) => atomic!(writer, I64_ATOMIC_RMW_XOR8U, m),
			I64AtomicRmwXor16u(m) => atomic!(writer, I64_ATOMIC_RMW_XOR16U, m),
			I64AtomicRmwXor32u(m) => atomic!(writer, I64_ATOMIC_RMW_XOR32U, m),

			I32AtomicRmwXchg(m) => atomic!(writer, I32_ATOMIC_RMW_XCHG, m),
			I64AtomicRmwXchg(m) => atomic!(writer, I64_ATOMIC_RMW_XCHG, m),
			I32AtomicRmwXchg8u(m) => atomic!(writer, I32_ATOMIC_RMW_XCHG8U, m),
			I32AtomicRmwXchg16u(m) => atomic!(writer, I32_ATOMIC_RMW_XCHG16U, m),
			I64AtomicRmwXchg8u(m) => atomic!(writer, I64_ATOMIC_RMW_XCHG8U, m),
			I64AtomicRmwXchg16u(m) => atomic!(writer, I64_ATOMIC_RMW_XCHG16U, m),
			I64AtomicRmwXchg32u(m) => atomic!(writer, I64_ATOMIC_RMW_XCHG32U, m),

			I32AtomicRmwCmpxchg(m) => atomic!(writer, I32_ATOMIC_RMW_CMPXCHG, m),
			I64AtomicRmwCmpxchg(m) => atomic!(writer, I64_ATOMIC_RMW_CMPXCHG, m),
			I32AtomicRmwCmpxchg8u(m) => atomic!(writer, I32_ATOMIC_RMW_CMPXCHG8U, m),
			I32AtomicRmwCmpxchg16u(m) => atomic!(writer, I32_ATOMIC_RMW_CMPXCHG16U, m),
			I64AtomicRmwCmpxchg8u(m) => atomic!(writer, I64_ATOMIC_RMW_CMPXCHG8U, m),
			I64AtomicRmwCmpxchg16u(m) => atomic!(writer, I64_ATOMIC_RMW_CMPXCHG16U, m),
			I64AtomicRmwCmpxchg32u(m) => atomic!(writer, I64_ATOMIC_RMW_CMPXCHG32U, m),
		}

		Ok(())
	}
}

#[cfg(feature = "simd")]
impl Serialize for SimdInstruction {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::{opcodes::simd::*, SimdInstruction::*};

		match self {
			V128Const(ref c) => simd!(writer, V128_CONST, writer.write(&c[..])?),
			V128Load(m) => simd!(writer, V128_LOAD, MemArg::serialize(m, writer)?),
			V128Store(m) => simd!(writer, V128_STORE, MemArg::serialize(m, writer)?),
			I8x16Splat => simd!(writer, I8X16_SPLAT, {}),
			I16x8Splat => simd!(writer, I16X8_SPLAT, {}),
			I32x4Splat => simd!(writer, I32X4_SPLAT, {}),
			I64x2Splat => simd!(writer, I64X2_SPLAT, {}),
			F32x4Splat => simd!(writer, F32X4_SPLAT, {}),
			F64x2Splat => simd!(writer, F64X2_SPLAT, {}),
			I8x16ExtractLaneS(i) => simd!(writer, I8X16_EXTRACT_LANE_S, writer.write(&[i])?),
			I8x16ExtractLaneU(i) => simd!(writer, I8X16_EXTRACT_LANE_U, writer.write(&[i])?),
			I16x8ExtractLaneS(i) => simd!(writer, I16X8_EXTRACT_LANE_S, writer.write(&[i])?),
			I16x8ExtractLaneU(i) => simd!(writer, I16X8_EXTRACT_LANE_U, writer.write(&[i])?),
			I32x4ExtractLane(i) => simd!(writer, I32X4_EXTRACT_LANE, writer.write(&[i])?),
			I64x2ExtractLane(i) => simd!(writer, I64X2_EXTRACT_LANE, writer.write(&[i])?),
			F32x4ExtractLane(i) => simd!(writer, F32X4_EXTRACT_LANE, writer.write(&[i])?),
			F64x2ExtractLane(i) => simd!(writer, F64X2_EXTRACT_LANE, writer.write(&[i])?),
			I8x16ReplaceLane(i) => simd!(writer, I8X16_REPLACE_LANE, writer.write(&[i])?),
			I16x8ReplaceLane(i) => simd!(writer, I16X8_REPLACE_LANE, writer.write(&[i])?),
			I32x4ReplaceLane(i) => simd!(writer, I32X4_REPLACE_LANE, writer.write(&[i])?),
			I64x2ReplaceLane(i) => simd!(writer, I64X2_REPLACE_LANE, writer.write(&[i])?),
			F32x4ReplaceLane(i) => simd!(writer, F32X4_REPLACE_LANE, writer.write(&[i])?),
			F64x2ReplaceLane(i) => simd!(writer, F64X2_REPLACE_LANE, writer.write(&[i])?),
			V8x16Shuffle(ref i) => simd!(writer, V8X16_SHUFFLE, writer.write(&i[..])?),
			I8x16Add => simd!(writer, I8X16_ADD, {}),
			I16x8Add => simd!(writer, I16X8_ADD, {}),
			I32x4Add => simd!(writer, I32X4_ADD, {}),
			I64x2Add => simd!(writer, I64X2_ADD, {}),
			I8x16Sub => simd!(writer, I8X16_SUB, {}),
			I16x8Sub => simd!(writer, I16X8_SUB, {}),
			I32x4Sub => simd!(writer, I32X4_SUB, {}),
			I64x2Sub => simd!(writer, I64X2_SUB, {}),
			I8x16Mul => simd!(writer, I8X16_MUL, {}),
			I16x8Mul => simd!(writer, I16X8_MUL, {}),
			I32x4Mul => simd!(writer, I32X4_MUL, {}),
			// I64x2Mul => simd!(writer, I64X2_MUL, {}),
			I8x16Neg => simd!(writer, I8X16_NEG, {}),
			I16x8Neg => simd!(writer, I16X8_NEG, {}),
			I32x4Neg => simd!(writer, I32X4_NEG, {}),
			I64x2Neg => simd!(writer, I64X2_NEG, {}),
			I8x16AddSaturateS => simd!(writer, I8X16_ADD_SATURATE_S, {}),
			I8x16AddSaturateU => simd!(writer, I8X16_ADD_SATURATE_U, {}),
			I16x8AddSaturateS => simd!(writer, I16X8_ADD_SATURATE_S, {}),
			I16x8AddSaturateU => simd!(writer, I16X8_ADD_SATURATE_U, {}),
			I8x16SubSaturateS => simd!(writer, I8X16_SUB_SATURATE_S, {}),
			I8x16SubSaturateU => simd!(writer, I8X16_SUB_SATURATE_U, {}),
			I16x8SubSaturateS => simd!(writer, I16X8_SUB_SATURATE_S, {}),
			I16x8SubSaturateU => simd!(writer, I16X8_SUB_SATURATE_U, {}),
			I8x16Shl => simd!(writer, I8X16_SHL, {}),
			I16x8Shl => simd!(writer, I16X8_SHL, {}),
			I32x4Shl => simd!(writer, I32X4_SHL, {}),
			I64x2Shl => simd!(writer, I64X2_SHL, {}),
			I8x16ShrS => simd!(writer, I8X16_SHR_S, {}),
			I8x16ShrU => simd!(writer, I8X16_SHR_U, {}),
			I16x8ShrS => simd!(writer, I16X8_SHR_S, {}),
			I16x8ShrU => simd!(writer, I16X8_SHR_U, {}),
			I32x4ShrU => simd!(writer, I32X4_SHR_U, {}),
			I32x4ShrS => simd!(writer, I32X4_SHR_S, {}),
			I64x2ShrU => simd!(writer, I64X2_SHR_U, {}),
			I64x2ShrS => simd!(writer, I64X2_SHR_S, {}),
			V128And => simd!(writer, V128_AND, {}),
			V128Or => simd!(writer, V128_OR, {}),
			V128Xor => simd!(writer, V128_XOR, {}),
			V128Not => simd!(writer, V128_NOT, {}),
			V128Bitselect => simd!(writer, V128_BITSELECT, {}),
			I8x16AnyTrue => simd!(writer, I8X16_ANY_TRUE, {}),
			I16x8AnyTrue => simd!(writer, I16X8_ANY_TRUE, {}),
			I32x4AnyTrue => simd!(writer, I32X4_ANY_TRUE, {}),
			I64x2AnyTrue => simd!(writer, I64X2_ANY_TRUE, {}),
			I8x16AllTrue => simd!(writer, I8X16_ALL_TRUE, {}),
			I16x8AllTrue => simd!(writer, I16X8_ALL_TRUE, {}),
			I32x4AllTrue => simd!(writer, I32X4_ALL_TRUE, {}),
			I64x2AllTrue => simd!(writer, I64X2_ALL_TRUE, {}),
			I8x16Eq => simd!(writer, I8X16_EQ, {}),
			I16x8Eq => simd!(writer, I16X8_EQ, {}),
			I32x4Eq => simd!(writer, I32X4_EQ, {}),
			// I64x2Eq => simd!(writer, I64X2_EQ, {}),
			F32x4Eq => simd!(writer, F32X4_EQ, {}),
			F64x2Eq => simd!(writer, F64X2_EQ, {}),
			I8x16Ne => simd!(writer, I8X16_NE, {}),
			I16x8Ne => simd!(writer, I16X8_NE, {}),
			I32x4Ne => simd!(writer, I32X4_NE, {}),
			// I64x2Ne => simd!(writer, I64X2_NE, {}),
			F32x4Ne => simd!(writer, F32X4_NE, {}),
			F64x2Ne => simd!(writer, F64X2_NE, {}),
			I8x16LtS => simd!(writer, I8X16_LT_S, {}),
			I8x16LtU => simd!(writer, I8X16_LT_U, {}),
			I16x8LtS => simd!(writer, I16X8_LT_S, {}),
			I16x8LtU => simd!(writer, I16X8_LT_U, {}),
			I32x4LtS => simd!(writer, I32X4_LT_S, {}),
			I32x4LtU => simd!(writer, I32X4_LT_U, {}),
			// I64x2LtS => simd!(writer, I64X2_LT_S, {}),
			// I64x2LtU => simd!(writer, I64X2_LT_U, {}),
			F32x4Lt => simd!(writer, F32X4_LT, {}),
			F64x2Lt => simd!(writer, F64X2_LT, {}),
			I8x16LeS => simd!(writer, I8X16_LE_S, {}),
			I8x16LeU => simd!(writer, I8X16_LE_U, {}),
			I16x8LeS => simd!(writer, I16X8_LE_S, {}),
			I16x8LeU => simd!(writer, I16X8_LE_U, {}),
			I32x4LeS => simd!(writer, I32X4_LE_S, {}),
			I32x4LeU => simd!(writer, I32X4_LE_U, {}),
			// I64x2LeS => simd!(writer, I64X2_LE_S, {}),
			// I64x2LeU => simd!(writer, I64X2_LE_U, {}),
			F32x4Le => simd!(writer, F32X4_LE, {}),
			F64x2Le => simd!(writer, F64X2_LE, {}),
			I8x16GtS => simd!(writer, I8X16_GT_S, {}),
			I8x16GtU => simd!(writer, I8X16_GT_U, {}),
			I16x8GtS => simd!(writer, I16X8_GT_S, {}),
			I16x8GtU => simd!(writer, I16X8_GT_U, {}),
			I32x4GtS => simd!(writer, I32X4_GT_S, {}),
			I32x4GtU => simd!(writer, I32X4_GT_U, {}),
			// I64x2GtS => simd!(writer, I64X2_GT_S, {}),
			// I64x2GtU => simd!(writer, I64X2_GT_U, {}),
			F32x4Gt => simd!(writer, F32X4_GT, {}),
			F64x2Gt => simd!(writer, F64X2_GT, {}),
			I8x16GeS => simd!(writer, I8X16_GE_S, {}),
			I8x16GeU => simd!(writer, I8X16_GE_U, {}),
			I16x8GeS => simd!(writer, I16X8_GE_S, {}),
			I16x8GeU => simd!(writer, I16X8_GE_U, {}),
			I32x4GeS => simd!(writer, I32X4_GE_S, {}),
			I32x4GeU => simd!(writer, I32X4_GE_U, {}),
			// I64x2GeS => simd!(writer, I64X2_GE_S, {}),
			// I64x2GeU => simd!(writer, I64X2_GE_U, {}),
			F32x4Ge => simd!(writer, F32X4_GE, {}),
			F64x2Ge => simd!(writer, F64X2_GE, {}),
			F32x4Neg => simd!(writer, F32X4_NEG, {}),
			F64x2Neg => simd!(writer, F64X2_NEG, {}),
			F32x4Abs => simd!(writer, F32X4_ABS, {}),
			F64x2Abs => simd!(writer, F64X2_ABS, {}),
			F32x4Min => simd!(writer, F32X4_MIN, {}),
			F64x2Min => simd!(writer, F64X2_MIN, {}),
			F32x4Max => simd!(writer, F32X4_MAX, {}),
			F64x2Max => simd!(writer, F64X2_MAX, {}),
			F32x4Add => simd!(writer, F32X4_ADD, {}),
			F64x2Add => simd!(writer, F64X2_ADD, {}),
			F32x4Sub => simd!(writer, F32X4_SUB, {}),
			F64x2Sub => simd!(writer, F64X2_SUB, {}),
			F32x4Div => simd!(writer, F32X4_DIV, {}),
			F64x2Div => simd!(writer, F64X2_DIV, {}),
			F32x4Mul => simd!(writer, F32X4_MUL, {}),
			F64x2Mul => simd!(writer, F64X2_MUL, {}),
			F32x4Sqrt => simd!(writer, F32X4_SQRT, {}),
			F64x2Sqrt => simd!(writer, F64X2_SQRT, {}),
			F32x4ConvertSI32x4 => simd!(writer, F32X4_CONVERT_S_I32X4, {}),
			F32x4ConvertUI32x4 => simd!(writer, F32X4_CONVERT_U_I32X4, {}),
			F64x2ConvertSI64x2 => simd!(writer, F64X2_CONVERT_S_I64X2, {}),
			F64x2ConvertUI64x2 => simd!(writer, F64X2_CONVERT_U_I64X2, {}),
			I32x4TruncSF32x4Sat => simd!(writer, I32X4_TRUNC_S_F32X4_SAT, {}),
			I32x4TruncUF32x4Sat => simd!(writer, I32X4_TRUNC_U_F32X4_SAT, {}),
			I64x2TruncSF64x2Sat => simd!(writer, I64X2_TRUNC_S_F64X2_SAT, {}),
			I64x2TruncUF64x2Sat => simd!(writer, I64X2_TRUNC_U_F64X2_SAT, {}),
		}

		Ok(())
	}
}

#[cfg(feature = "bulk")]
impl Serialize for BulkInstruction {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		use self::{opcodes::bulk::*, BulkInstruction::*};

		match self {
			MemoryInit(seg) => bulk!(writer, MEMORY_INIT, {
				Uint8::from(0).serialize(writer)?;
				VarUint32::from(seg).serialize(writer)?;
			}),
			MemoryDrop(seg) => bulk!(writer, MEMORY_DROP, VarUint32::from(seg).serialize(writer)?),
			MemoryFill => bulk!(writer, MEMORY_FILL, Uint8::from(0).serialize(writer)?),
			MemoryCopy => bulk!(writer, MEMORY_COPY, Uint8::from(0).serialize(writer)?),
			TableInit(seg) => bulk!(writer, TABLE_INIT, {
				Uint8::from(0).serialize(writer)?;
				VarUint32::from(seg).serialize(writer)?;
			}),
			TableDrop(seg) => bulk!(writer, TABLE_DROP, VarUint32::from(seg).serialize(writer)?),
			TableCopy => bulk!(writer, TABLE_COPY, Uint8::from(0).serialize(writer)?),
		}

		Ok(())
	}
}

#[cfg(any(feature = "simd", feature = "atomics"))]
impl Serialize for MemArg {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		Uint8::from(self.align).serialize(writer)?;
		VarUint32::from(self.offset).serialize(writer)?;
		Ok(())
	}
}

macro_rules! fmt_op {
	($f: expr, $mnemonic: expr) => {{
		write!($f, "{}", $mnemonic)
	}};
	($f: expr, $mnemonic: expr, $immediate: expr) => {{
		write!($f, "{} {}", $mnemonic, $immediate)
	}};
	($f: expr, $mnemonic: expr, $immediate1: expr, $immediate2: expr) => {{
		write!($f, "{} {} {}", $mnemonic, $immediate1, $immediate2)
	}};
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Instruction::*;

		match *self {
			Unreachable => fmt_op!(f, "unreachable"),
			Nop => fmt_op!(f, "nop"),
			Block(BlockType::NoResult) => fmt_op!(f, "block"),
			Block(BlockType::Value(value_type)) => fmt_op!(f, "block", value_type),
			#[cfg(feature = "multi_value")]
			Block(BlockType::TypeIndex(idx)) => write!(f, "block type_idx={}", idx),
			Loop(BlockType::NoResult) => fmt_op!(f, "loop"),
			Loop(BlockType::Value(value_type)) => fmt_op!(f, "loop", value_type),
			#[cfg(feature = "multi_value")]
			Loop(BlockType::TypeIndex(idx)) => write!(f, "loop type_idx={}", idx),
			If(BlockType::NoResult) => fmt_op!(f, "if"),
			If(BlockType::Value(value_type)) => fmt_op!(f, "if", value_type),
			#[cfg(feature = "multi_value")]
			If(BlockType::TypeIndex(idx)) => write!(f, "if type_idx={}", idx),
			Else => fmt_op!(f, "else"),
			End => fmt_op!(f, "end"),
			Br(idx) => fmt_op!(f, "br", idx),
			BrIf(idx) => fmt_op!(f, "br_if", idx),
			BrTable(ref table) => fmt_op!(f, "br_table", table.default),
			Return => fmt_op!(f, "return"),
			Call(index) => fmt_op!(f, "call", index),
			CallIndirect(index, _) => fmt_op!(f, "call_indirect", index),
			Drop => fmt_op!(f, "drop"),
			Select => fmt_op!(f, "select"),
			GetLocal(index) => fmt_op!(f, "get_local", index),
			SetLocal(index) => fmt_op!(f, "set_local", index),
			TeeLocal(index) => fmt_op!(f, "tee_local", index),
			GetGlobal(index) => fmt_op!(f, "get_global", index),
			SetGlobal(index) => fmt_op!(f, "set_global", index),

			I32Load(_, 0) => write!(f, "i32.load"),
			I32Load(_, offset) => write!(f, "i32.load offset={}", offset),

			I64Load(_, 0) => write!(f, "i64.load"),
			I64Load(_, offset) => write!(f, "i64.load offset={}", offset),

			F32Load(_, 0) => write!(f, "f32.load"),
			F32Load(_, offset) => write!(f, "f32.load offset={}", offset),

			F64Load(_, 0) => write!(f, "f64.load"),
			F64Load(_, offset) => write!(f, "f64.load offset={}", offset),

			I32Load8S(_, 0) => write!(f, "i32.load8_s"),
			I32Load8S(_, offset) => write!(f, "i32.load8_s offset={}", offset),

			I32Load8U(_, 0) => write!(f, "i32.load8_u"),
			I32Load8U(_, offset) => write!(f, "i32.load8_u offset={}", offset),

			I32Load16S(_, 0) => write!(f, "i32.load16_s"),
			I32Load16S(_, offset) => write!(f, "i32.load16_s offset={}", offset),

			I32Load16U(_, 0) => write!(f, "i32.load16_u"),
			I32Load16U(_, offset) => write!(f, "i32.load16_u offset={}", offset),

			I64Load8S(_, 0) => write!(f, "i64.load8_s"),
			I64Load8S(_, offset) => write!(f, "i64.load8_s offset={}", offset),

			I64Load8U(_, 0) => write!(f, "i64.load8_u"),
			I64Load8U(_, offset) => write!(f, "i64.load8_u offset={}", offset),

			I64Load16S(_, 0) => write!(f, "i64.load16_s"),
			I64Load16S(_, offset) => write!(f, "i64.load16_s offset={}", offset),

			I64Load16U(_, 0) => write!(f, "i64.load16_u"),
			I64Load16U(_, offset) => write!(f, "i64.load16_u offset={}", offset),

			I64Load32S(_, 0) => write!(f, "i64.load32_s"),
			I64Load32S(_, offset) => write!(f, "i64.load32_s offset={}", offset),

			I64Load32U(_, 0) => write!(f, "i64.load32_u"),
			I64Load32U(_, offset) => write!(f, "i64.load32_u offset={}", offset),

			I32Store(_, 0) => write!(f, "i32.store"),
			I32Store(_, offset) => write!(f, "i32.store offset={}", offset),

			I64Store(_, 0) => write!(f, "i64.store"),
			I64Store(_, offset) => write!(f, "i64.store offset={}", offset),

			F32Store(_, 0) => write!(f, "f32.store"),
			F32Store(_, offset) => write!(f, "f32.store offset={}", offset),

			F64Store(_, 0) => write!(f, "f64.store"),
			F64Store(_, offset) => write!(f, "f64.store offset={}", offset),

			I32Store8(_, 0) => write!(f, "i32.store8"),
			I32Store8(_, offset) => write!(f, "i32.store8 offset={}", offset),

			I32Store16(_, 0) => write!(f, "i32.store16"),
			I32Store16(_, offset) => write!(f, "i32.store16 offset={}", offset),

			I64Store8(_, 0) => write!(f, "i64.store8"),
			I64Store8(_, offset) => write!(f, "i64.store8 offset={}", offset),

			I64Store16(_, 0) => write!(f, "i64.store16"),
			I64Store16(_, offset) => write!(f, "i64.store16 offset={}", offset),

			I64Store32(_, 0) => write!(f, "i64.store32"),
			I64Store32(_, offset) => write!(f, "i64.store32 offset={}", offset),

			CurrentMemory(_) => fmt_op!(f, "current_memory"),
			GrowMemory(_) => fmt_op!(f, "grow_memory"),

			I32Const(def) => fmt_op!(f, "i32.const", def),
			I64Const(def) => fmt_op!(f, "i64.const", def),
			F32Const(def) => fmt_op!(f, "f32.const", def),
			F64Const(def) => fmt_op!(f, "f64.const", def),

			I32Eq => write!(f, "i32.eq"),
			I32Eqz => write!(f, "i32.eqz"),
			I32Ne => write!(f, "i32.ne"),
			I32LtS => write!(f, "i32.lt_s"),
			I32LtU => write!(f, "i32.lt_u"),
			I32GtS => write!(f, "i32.gt_s"),
			I32GtU => write!(f, "i32.gt_u"),
			I32LeS => write!(f, "i32.le_s"),
			I32LeU => write!(f, "i32.le_u"),
			I32GeS => write!(f, "i32.ge_s"),
			I32GeU => write!(f, "i32.ge_u"),

			I64Eq => write!(f, "i64.eq"),
			I64Eqz => write!(f, "i64.eqz"),
			I64Ne => write!(f, "i64.ne"),
			I64LtS => write!(f, "i64.lt_s"),
			I64LtU => write!(f, "i64.lt_u"),
			I64GtS => write!(f, "i64.gt_s"),
			I64GtU => write!(f, "i64.gt_u"),
			I64LeS => write!(f, "i64.le_s"),
			I64LeU => write!(f, "i64.le_u"),
			I64GeS => write!(f, "i64.ge_s"),
			I64GeU => write!(f, "i64.ge_u"),

			F32Eq => write!(f, "f32.eq"),
			F32Ne => write!(f, "f32.ne"),
			F32Lt => write!(f, "f32.lt"),
			F32Gt => write!(f, "f32.gt"),
			F32Le => write!(f, "f32.le"),
			F32Ge => write!(f, "f32.ge"),

			F64Eq => write!(f, "f64.eq"),
			F64Ne => write!(f, "f64.ne"),
			F64Lt => write!(f, "f64.lt"),
			F64Gt => write!(f, "f64.gt"),
			F64Le => write!(f, "f64.le"),
			F64Ge => write!(f, "f64.ge"),

			I32Clz => write!(f, "i32.clz"),
			I32Ctz => write!(f, "i32.ctz"),
			I32Popcnt => write!(f, "i32.popcnt"),
			I32Add => write!(f, "i32.add"),
			I32Sub => write!(f, "i32.sub"),
			I32Mul => write!(f, "i32.mul"),
			I32DivS => write!(f, "i32.div_s"),
			I32DivU => write!(f, "i32.div_u"),
			I32RemS => write!(f, "i32.rem_s"),
			I32RemU => write!(f, "i32.rem_u"),
			I32And => write!(f, "i32.and"),
			I32Or => write!(f, "i32.or"),
			I32Xor => write!(f, "i32.xor"),
			I32Shl => write!(f, "i32.shl"),
			I32ShrS => write!(f, "i32.shr_s"),
			I32ShrU => write!(f, "i32.shr_u"),
			I32Rotl => write!(f, "i32.rotl"),
			I32Rotr => write!(f, "i32.rotr"),

			I64Clz => write!(f, "i64.clz"),
			I64Ctz => write!(f, "i64.ctz"),
			I64Popcnt => write!(f, "i64.popcnt"),
			I64Add => write!(f, "i64.add"),
			I64Sub => write!(f, "i64.sub"),
			I64Mul => write!(f, "i64.mul"),
			I64DivS => write!(f, "i64.div_s"),
			I64DivU => write!(f, "i64.div_u"),
			I64RemS => write!(f, "i64.rem_s"),
			I64RemU => write!(f, "i64.rem_u"),
			I64And => write!(f, "i64.and"),
			I64Or => write!(f, "i64.or"),
			I64Xor => write!(f, "i64.xor"),
			I64Shl => write!(f, "i64.shl"),
			I64ShrS => write!(f, "i64.shr_s"),
			I64ShrU => write!(f, "i64.shr_u"),
			I64Rotl => write!(f, "i64.rotl"),
			I64Rotr => write!(f, "i64.rotr"),

			F32Abs => write!(f, "f32.abs"),
			F32Neg => write!(f, "f32.neg"),
			F32Ceil => write!(f, "f32.ceil"),
			F32Floor => write!(f, "f32.floor"),
			F32Trunc => write!(f, "f32.trunc"),
			F32Nearest => write!(f, "f32.nearest"),
			F32Sqrt => write!(f, "f32.sqrt"),
			F32Add => write!(f, "f32.add"),
			F32Sub => write!(f, "f32.sub"),
			F32Mul => write!(f, "f32.mul"),
			F32Div => write!(f, "f32.div"),
			F32Min => write!(f, "f32.min"),
			F32Max => write!(f, "f32.max"),
			F32Copysign => write!(f, "f32.copysign"),

			F64Abs => write!(f, "f64.abs"),
			F64Neg => write!(f, "f64.neg"),
			F64Ceil => write!(f, "f64.ceil"),
			F64Floor => write!(f, "f64.floor"),
			F64Trunc => write!(f, "f64.trunc"),
			F64Nearest => write!(f, "f64.nearest"),
			F64Sqrt => write!(f, "f64.sqrt"),
			F64Add => write!(f, "f64.add"),
			F64Sub => write!(f, "f64.sub"),
			F64Mul => write!(f, "f64.mul"),
			F64Div => write!(f, "f64.div"),
			F64Min => write!(f, "f64.min"),
			F64Max => write!(f, "f64.max"),
			F64Copysign => write!(f, "f64.copysign"),

			I32WrapI64 => write!(f, "i32.wrap/i64"),
			I32TruncSF32 => write!(f, "i32.trunc_s/f32"),
			I32TruncUF32 => write!(f, "i32.trunc_u/f32"),
			I32TruncSF64 => write!(f, "i32.trunc_s/f64"),
			I32TruncUF64 => write!(f, "i32.trunc_u/f64"),

			I64ExtendSI32 => write!(f, "i64.extend_s/i32"),
			I64ExtendUI32 => write!(f, "i64.extend_u/i32"),

			I64TruncSF32 => write!(f, "i64.trunc_s/f32"),
			I64TruncUF32 => write!(f, "i64.trunc_u/f32"),
			I64TruncSF64 => write!(f, "i64.trunc_s/f64"),
			I64TruncUF64 => write!(f, "i64.trunc_u/f64"),

			F32ConvertSI32 => write!(f, "f32.convert_s/i32"),
			F32ConvertUI32 => write!(f, "f32.convert_u/i32"),
			F32ConvertSI64 => write!(f, "f32.convert_s/i64"),
			F32ConvertUI64 => write!(f, "f32.convert_u/i64"),
			F32DemoteF64 => write!(f, "f32.demote/f64"),

			F64ConvertSI32 => write!(f, "f64.convert_s/i32"),
			F64ConvertUI32 => write!(f, "f64.convert_u/i32"),
			F64ConvertSI64 => write!(f, "f64.convert_s/i64"),
			F64ConvertUI64 => write!(f, "f64.convert_u/i64"),
			F64PromoteF32 => write!(f, "f64.promote/f32"),

			I32ReinterpretF32 => write!(f, "i32.reinterpret/f32"),
			I64ReinterpretF64 => write!(f, "i64.reinterpret/f64"),
			F32ReinterpretI32 => write!(f, "f32.reinterpret/i32"),
			F64ReinterpretI64 => write!(f, "f64.reinterpret/i64"),

			#[cfg(feature = "sign_ext")]
			SignExt(ref i) => match i {
				SignExtInstruction::I32Extend8S => write!(f, "i32.extend8_s"),
				SignExtInstruction::I32Extend16S => write!(f, "i32.extend16_s"),
				SignExtInstruction::I64Extend8S => write!(f, "i64.extend8_s"),
				SignExtInstruction::I64Extend16S => write!(f, "i64.extend16_s"),
				SignExtInstruction::I64Extend32S => write!(f, "i64.extend32_s"),
			},

			#[cfg(feature = "atomics")]
			Atomics(ref i) => i.fmt(f),

			#[cfg(feature = "simd")]
			Simd(ref i) => i.fmt(f),

			#[cfg(feature = "bulk")]
			Bulk(ref i) => i.fmt(f),
		}
	}
}

#[cfg(feature = "atomics")]
impl fmt::Display for AtomicsInstruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::AtomicsInstruction::*;

		match *self {
			AtomicWake(_) => write!(f, "atomic.wake"),
			I32AtomicWait(_) => write!(f, "i32.atomic.wait"),
			I64AtomicWait(_) => write!(f, "i64.atomic.wait"),

			I32AtomicLoad(_) => write!(f, "i32.atomic.load"),
			I64AtomicLoad(_) => write!(f, "i64.atomic.load"),
			I32AtomicLoad8u(_) => write!(f, "i32.atomic.load8_u"),
			I32AtomicLoad16u(_) => write!(f, "i32.atomic.load16_u"),
			I64AtomicLoad8u(_) => write!(f, "i64.atomic.load8_u"),
			I64AtomicLoad16u(_) => write!(f, "i64.atomic.load16_u"),
			I64AtomicLoad32u(_) => write!(f, "i64.atomic.load32_u"),
			I32AtomicStore(_) => write!(f, "i32.atomic.store"),
			I64AtomicStore(_) => write!(f, "i64.atomic.store"),
			I32AtomicStore8u(_) => write!(f, "i32.atomic.store8_u"),
			I32AtomicStore16u(_) => write!(f, "i32.atomic.store16_u"),
			I64AtomicStore8u(_) => write!(f, "i64.atomic.store8_u"),
			I64AtomicStore16u(_) => write!(f, "i64.atomic.store16_u"),
			I64AtomicStore32u(_) => write!(f, "i64.atomic.store32_u"),

			I32AtomicRmwAdd(_) => write!(f, "i32.atomic.rmw.add"),
			I64AtomicRmwAdd(_) => write!(f, "i64.atomic.rmw.add"),
			I32AtomicRmwAdd8u(_) => write!(f, "i32.atomic.rmw8_u.add"),
			I32AtomicRmwAdd16u(_) => write!(f, "i32.atomic.rmw16_u.add"),
			I64AtomicRmwAdd8u(_) => write!(f, "i64.atomic.rmw8_u.add"),
			I64AtomicRmwAdd16u(_) => write!(f, "i64.atomic.rmw16_u.add"),
			I64AtomicRmwAdd32u(_) => write!(f, "i64.atomic.rmw32_u.add"),

			I32AtomicRmwSub(_) => write!(f, "i32.atomic.rmw.sub"),
			I64AtomicRmwSub(_) => write!(f, "i64.atomic.rmw.sub"),
			I32AtomicRmwSub8u(_) => write!(f, "i32.atomic.rmw8_u.sub"),
			I32AtomicRmwSub16u(_) => write!(f, "i32.atomic.rmw16_u.sub"),
			I64AtomicRmwSub8u(_) => write!(f, "i64.atomic.rmw8_u.sub"),
			I64AtomicRmwSub16u(_) => write!(f, "i64.atomic.rmw16_u.sub"),
			I64AtomicRmwSub32u(_) => write!(f, "i64.atomic.rmw32_u.sub"),

			I32AtomicRmwAnd(_) => write!(f, "i32.atomic.rmw.and"),
			I64AtomicRmwAnd(_) => write!(f, "i64.atomic.rmw.and"),
			I32AtomicRmwAnd8u(_) => write!(f, "i32.atomic.rmw8_u.and"),
			I32AtomicRmwAnd16u(_) => write!(f, "i32.atomic.rmw16_u.and"),
			I64AtomicRmwAnd8u(_) => write!(f, "i64.atomic.rmw8_u.and"),
			I64AtomicRmwAnd16u(_) => write!(f, "i64.atomic.rmw16_u.and"),
			I64AtomicRmwAnd32u(_) => write!(f, "i64.atomic.rmw32_u.and"),

			I32AtomicRmwOr(_) => write!(f, "i32.atomic.rmw.or"),
			I64AtomicRmwOr(_) => write!(f, "i64.atomic.rmw.or"),
			I32AtomicRmwOr8u(_) => write!(f, "i32.atomic.rmw8_u.or"),
			I32AtomicRmwOr16u(_) => write!(f, "i32.atomic.rmw16_u.or"),
			I64AtomicRmwOr8u(_) => write!(f, "i64.atomic.rmw8_u.or"),
			I64AtomicRmwOr16u(_) => write!(f, "i64.atomic.rmw16_u.or"),
			I64AtomicRmwOr32u(_) => write!(f, "i64.atomic.rmw32_u.or"),

			I32AtomicRmwXor(_) => write!(f, "i32.atomic.rmw.xor"),
			I64AtomicRmwXor(_) => write!(f, "i64.atomic.rmw.xor"),
			I32AtomicRmwXor8u(_) => write!(f, "i32.atomic.rmw8_u.xor"),
			I32AtomicRmwXor16u(_) => write!(f, "i32.atomic.rmw16_u.xor"),
			I64AtomicRmwXor8u(_) => write!(f, "i64.atomic.rmw8_u.xor"),
			I64AtomicRmwXor16u(_) => write!(f, "i64.atomic.rmw16_u.xor"),
			I64AtomicRmwXor32u(_) => write!(f, "i64.atomic.rmw32_u.xor"),

			I32AtomicRmwXchg(_) => write!(f, "i32.atomic.rmw.xchg"),
			I64AtomicRmwXchg(_) => write!(f, "i64.atomic.rmw.xchg"),
			I32AtomicRmwXchg8u(_) => write!(f, "i32.atomic.rmw8_u.xchg"),
			I32AtomicRmwXchg16u(_) => write!(f, "i32.atomic.rmw16_u.xchg"),
			I64AtomicRmwXchg8u(_) => write!(f, "i64.atomic.rmw8_u.xchg"),
			I64AtomicRmwXchg16u(_) => write!(f, "i64.atomic.rmw16_u.xchg"),
			I64AtomicRmwXchg32u(_) => write!(f, "i64.atomic.rmw32_u.xchg"),

			I32AtomicRmwCmpxchg(_) => write!(f, "i32.atomic.rmw.cmpxchg"),
			I64AtomicRmwCmpxchg(_) => write!(f, "i64.atomic.rmw.cmpxchg"),
			I32AtomicRmwCmpxchg8u(_) => write!(f, "i32.atomic.rmw8_u.cmpxchg"),
			I32AtomicRmwCmpxchg16u(_) => write!(f, "i32.atomic.rmw16_u.cmpxchg"),
			I64AtomicRmwCmpxchg8u(_) => write!(f, "i64.atomic.rmw8_u.cmpxchg"),
			I64AtomicRmwCmpxchg16u(_) => write!(f, "i64.atomic.rmw16_u.cmpxchg"),
			I64AtomicRmwCmpxchg32u(_) => write!(f, "i64.atomic.rmw32_u.cmpxchg"),
		}
	}
}

#[cfg(feature = "simd")]
impl fmt::Display for SimdInstruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::SimdInstruction::*;

		match *self {
			V128Const(_) => write!(f, "v128.const"),
			V128Load(_) => write!(f, "v128.load"),
			V128Store(_) => write!(f, "v128.store"),
			I8x16Splat => write!(f, "i8x16.splat"),
			I16x8Splat => write!(f, "i16x8.splat"),
			I32x4Splat => write!(f, "i32x4.splat"),
			I64x2Splat => write!(f, "i64x2.splat"),
			F32x4Splat => write!(f, "f32x4.splat"),
			F64x2Splat => write!(f, "f64x2.splat"),
			I8x16ExtractLaneS(_) => write!(f, "i8x16.extract_lane_s"),
			I8x16ExtractLaneU(_) => write!(f, "i8x16.extract_lane_u"),
			I16x8ExtractLaneS(_) => write!(f, "i16x8.extract_lane_s"),
			I16x8ExtractLaneU(_) => write!(f, "i16x8.extract_lane_u"),
			I32x4ExtractLane(_) => write!(f, "i32x4.extract_lane"),
			I64x2ExtractLane(_) => write!(f, "i64x2.extract_lane"),
			F32x4ExtractLane(_) => write!(f, "f32x4.extract_lane"),
			F64x2ExtractLane(_) => write!(f, "f64x2.extract_lane"),
			I8x16ReplaceLane(_) => write!(f, "i8x16.replace_lane"),
			I16x8ReplaceLane(_) => write!(f, "i16x8.replace_lane"),
			I32x4ReplaceLane(_) => write!(f, "i32x4.replace_lane"),
			I64x2ReplaceLane(_) => write!(f, "i64x2.replace_lane"),
			F32x4ReplaceLane(_) => write!(f, "f32x4.replace_lane"),
			F64x2ReplaceLane(_) => write!(f, "f64x2.replace_lane"),
			V8x16Shuffle(_) => write!(f, "v8x16.shuffle"),
			I8x16Add => write!(f, "i8x16.add"),
			I16x8Add => write!(f, "i16x8.add"),
			I32x4Add => write!(f, "i32x4.add"),
			I64x2Add => write!(f, "i64x2.add"),
			I8x16Sub => write!(f, "i8x16.sub"),
			I16x8Sub => write!(f, "i16x8.sub"),
			I32x4Sub => write!(f, "i32x4.sub"),
			I64x2Sub => write!(f, "i64x2.sub"),
			I8x16Mul => write!(f, "i8x16.mul"),
			I16x8Mul => write!(f, "i16x8.mul"),
			I32x4Mul => write!(f, "i32x4.mul"),
			// I64x2Mul => write!(f, "i64x2.mul"),
			I8x16Neg => write!(f, "i8x16.neg"),
			I16x8Neg => write!(f, "i16x8.neg"),
			I32x4Neg => write!(f, "i32x4.neg"),
			I64x2Neg => write!(f, "i64x2.neg"),
			I8x16AddSaturateS => write!(f, "i8x16.add_saturate_s"),
			I8x16AddSaturateU => write!(f, "i8x16.add_saturate_u"),
			I16x8AddSaturateS => write!(f, "i16x8.add_saturate_S"),
			I16x8AddSaturateU => write!(f, "i16x8.add_saturate_u"),
			I8x16SubSaturateS => write!(f, "i8x16.sub_saturate_S"),
			I8x16SubSaturateU => write!(f, "i8x16.sub_saturate_u"),
			I16x8SubSaturateS => write!(f, "i16x8.sub_saturate_S"),
			I16x8SubSaturateU => write!(f, "i16x8.sub_saturate_u"),
			I8x16Shl => write!(f, "i8x16.shl"),
			I16x8Shl => write!(f, "i16x8.shl"),
			I32x4Shl => write!(f, "i32x4.shl"),
			I64x2Shl => write!(f, "i64x2.shl"),
			I8x16ShrS => write!(f, "i8x16.shr_s"),
			I8x16ShrU => write!(f, "i8x16.shr_u"),
			I16x8ShrS => write!(f, "i16x8.shr_s"),
			I16x8ShrU => write!(f, "i16x8.shr_u"),
			I32x4ShrS => write!(f, "i32x4.shr_s"),
			I32x4ShrU => write!(f, "i32x4.shr_u"),
			I64x2ShrS => write!(f, "i64x2.shr_s"),
			I64x2ShrU => write!(f, "i64x2.shr_u"),
			V128And => write!(f, "v128.and"),
			V128Or => write!(f, "v128.or"),
			V128Xor => write!(f, "v128.xor"),
			V128Not => write!(f, "v128.not"),
			V128Bitselect => write!(f, "v128.bitselect"),
			I8x16AnyTrue => write!(f, "i8x16.any_true"),
			I16x8AnyTrue => write!(f, "i16x8.any_true"),
			I32x4AnyTrue => write!(f, "i32x4.any_true"),
			I64x2AnyTrue => write!(f, "i64x2.any_true"),
			I8x16AllTrue => write!(f, "i8x16.all_true"),
			I16x8AllTrue => write!(f, "i16x8.all_true"),
			I32x4AllTrue => write!(f, "i32x4.all_true"),
			I64x2AllTrue => write!(f, "i64x2.all_true"),
			I8x16Eq => write!(f, "i8x16.eq"),
			I16x8Eq => write!(f, "i16x8.eq"),
			I32x4Eq => write!(f, "i32x4.eq"),
			// I64x2Eq => write!(f, "i64x2.eq"),
			F32x4Eq => write!(f, "f32x4.eq"),
			F64x2Eq => write!(f, "f64x2.eq"),
			I8x16Ne => write!(f, "i8x16.ne"),
			I16x8Ne => write!(f, "i16x8.ne"),
			I32x4Ne => write!(f, "i32x4.ne"),
			// I64x2Ne => write!(f, "i64x2.ne"),
			F32x4Ne => write!(f, "f32x4.ne"),
			F64x2Ne => write!(f, "f64x2.ne"),
			I8x16LtS => write!(f, "i8x16.lt_s"),
			I8x16LtU => write!(f, "i8x16.lt_u"),
			I16x8LtS => write!(f, "i16x8.lt_s"),
			I16x8LtU => write!(f, "i16x8.lt_u"),
			I32x4LtS => write!(f, "i32x4.lt_s"),
			I32x4LtU => write!(f, "i32x4.lt_u"),
			// I64x2LtS => write!(f, "// I64x2.lt_s"),
			// I64x2LtU => write!(f, "// I64x2.lt_u"),
			F32x4Lt => write!(f, "f32x4.lt"),
			F64x2Lt => write!(f, "f64x2.lt"),
			I8x16LeS => write!(f, "i8x16.le_s"),
			I8x16LeU => write!(f, "i8x16.le_u"),
			I16x8LeS => write!(f, "i16x8.le_s"),
			I16x8LeU => write!(f, "i16x8.le_u"),
			I32x4LeS => write!(f, "i32x4.le_s"),
			I32x4LeU => write!(f, "i32x4.le_u"),
			// I64x2LeS => write!(f, "// I64x2.le_s"),
			// I64x2LeU => write!(f, "// I64x2.le_u"),
			F32x4Le => write!(f, "f32x4.le"),
			F64x2Le => write!(f, "f64x2.le"),
			I8x16GtS => write!(f, "i8x16.gt_s"),
			I8x16GtU => write!(f, "i8x16.gt_u"),
			I16x8GtS => write!(f, "i16x8.gt_s"),
			I16x8GtU => write!(f, "i16x8.gt_u"),
			I32x4GtS => write!(f, "i32x4.gt_s"),
			I32x4GtU => write!(f, "i32x4.gt_u"),
			// I64x2GtS => write!(f, "// I64x2.gt_s"),
			// I64x2GtU => write!(f, "// I64x2.gt_u"),
			F32x4Gt => write!(f, "f32x4.gt"),
			F64x2Gt => write!(f, "f64x2.gt"),
			I8x16GeS => write!(f, "i8x16.ge_s"),
			I8x16GeU => write!(f, "i8x16.ge_u"),
			I16x8GeS => write!(f, "i16x8.ge_s"),
			I16x8GeU => write!(f, "i16x8.ge_u"),
			I32x4GeS => write!(f, "i32x4.ge_s"),
			I32x4GeU => write!(f, "i32x4.ge_u"),
			// I64x2GeS => write!(f, "// I64x2.ge_s"),
			// I64x2GeU => write!(f, "// I64x2.ge_u"),
			F32x4Ge => write!(f, "f32x4.ge"),
			F64x2Ge => write!(f, "f64x2.ge"),
			F32x4Neg => write!(f, "f32x4.neg"),
			F64x2Neg => write!(f, "f64x2.neg"),
			F32x4Abs => write!(f, "f32x4.abs"),
			F64x2Abs => write!(f, "f64x2.abs"),
			F32x4Min => write!(f, "f32x4.min"),
			F64x2Min => write!(f, "f64x2.min"),
			F32x4Max => write!(f, "f32x4.max"),
			F64x2Max => write!(f, "f64x2.max"),
			F32x4Add => write!(f, "f32x4.add"),
			F64x2Add => write!(f, "f64x2.add"),
			F32x4Sub => write!(f, "f32x4.sub"),
			F64x2Sub => write!(f, "f64x2.sub"),
			F32x4Div => write!(f, "f32x4.div"),
			F64x2Div => write!(f, "f64x2.div"),
			F32x4Mul => write!(f, "f32x4.mul"),
			F64x2Mul => write!(f, "f64x2.mul"),
			F32x4Sqrt => write!(f, "f32x4.sqrt"),
			F64x2Sqrt => write!(f, "f64x2.sqrt"),
			F32x4ConvertSI32x4 => write!(f, "f32x4.convert_s/i32x4"),
			F32x4ConvertUI32x4 => write!(f, "f32x4.convert_u/i32x4"),
			F64x2ConvertSI64x2 => write!(f, "f64x2.convert_s/i64x2"),
			F64x2ConvertUI64x2 => write!(f, "f64x2.convert_u/i64x2"),
			I32x4TruncSF32x4Sat => write!(f, "i32x4.trunc_s/f32x4:sat"),
			I32x4TruncUF32x4Sat => write!(f, "i32x4.trunc_u/f32x4:sat"),
			I64x2TruncSF64x2Sat => write!(f, "i64x2.trunc_s/f64x2:sat"),
			I64x2TruncUF64x2Sat => write!(f, "i64x2.trunc_u/f64x2:sat"),
		}
	}
}

#[cfg(feature = "bulk")]
impl fmt::Display for BulkInstruction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::BulkInstruction::*;

		match *self {
			MemoryInit(_) => write!(f, "memory.init"),
			MemoryDrop(_) => write!(f, "memory.drop"),
			MemoryFill => write!(f, "memory.fill"),
			MemoryCopy => write!(f, "memory.copy"),
			TableInit(_) => write!(f, "table.init"),
			TableDrop(_) => write!(f, "table.drop"),
			TableCopy => write!(f, "table.copy"),
		}
	}
}

impl Serialize for Instructions {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		for op in self.0.into_iter() {
			op.serialize(writer)?;
		}

		Ok(())
	}
}

impl Serialize for InitExpr {
	type Error = Error;

	fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
		for op in self.0.into_iter() {
			op.serialize(writer)?;
		}

		Ok(())
	}
}

#[test]
fn ifelse() {
	// see if-else.wast/if-else.wasm
	let instruction_list = super::deserialize_buffer::<Instructions>(&[
		0x04, 0x7F, 0x41, 0x05, 0x05, 0x41, 0x07, 0x0B, 0x0B,
	])
	.expect("valid hex of if instruction");
	let instructions = instruction_list.elements();
	match instructions[0] {
		Instruction::If(_) => (),
		_ => panic!("Should be deserialized as if instruction"),
	}
	let before_else = instructions
		.iter()
		.skip(1)
		.take_while(|op| !matches!(**op, Instruction::Else))
		.count();
	let after_else = instructions
		.iter()
		.skip(1)
		.skip_while(|op| !matches!(**op, Instruction::Else))
		.take_while(|op| !matches!(**op, Instruction::End))
		.count() - 1; // minus Instruction::Else itself
	assert_eq!(before_else, after_else);
}

#[test]
fn display() {
	let instruction = Instruction::GetLocal(0);
	assert_eq!("get_local 0", format!("{}", instruction));

	let instruction = Instruction::F64Store(0, 24);
	assert_eq!("f64.store offset=24", format!("{}", instruction));

	let instruction = Instruction::I64Store(0, 0);
	assert_eq!("i64.store", format!("{}", instruction));
}

#[test]
fn size_off() {
	assert!(::std::mem::size_of::<Instruction>() <= 24);
}

#[test]
fn instructions_hashset() {
	use self::Instruction::{Block, Call, Drop};
	use super::types::{BlockType::Value, ValueType};

	let set: std::collections::HashSet<Instruction> =
		vec![Call(1), Block(Value(ValueType::I32)), Drop].into_iter().collect();
	assert!(set.contains(&Drop));
}
