/* Copyright 2018 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::error::Error;
use std::fmt;
use std::result;

#[derive(Debug, Clone)]
pub struct BinaryReaderError {
    // Wrap the actual error data in a `Box` so that the error is just one
    // word. This means that we can continue returning small `Result`s in
    // registers.
    pub(crate) inner: Box<BinaryReaderErrorInner>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinaryReaderErrorInner {
    pub(crate) message: String,
    pub(crate) offset: usize,
    pub(crate) needed_hint: Option<usize>,
}

pub type Result<T, E = BinaryReaderError> = result::Result<T, E>;

impl Error for BinaryReaderError {}

impl fmt::Display for BinaryReaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} (at offset {})",
            self.inner.message, self.inner.offset
        )
    }
}

impl BinaryReaderError {
    pub(crate) fn new(message: impl Into<String>, offset: usize) -> Self {
        let message = message.into();
        BinaryReaderError {
            inner: Box::new(BinaryReaderErrorInner {
                message,
                offset,
                needed_hint: None,
            }),
        }
    }

    pub(crate) fn eof(offset: usize, needed_hint: usize) -> Self {
        BinaryReaderError {
            inner: Box::new(BinaryReaderErrorInner {
                message: "Unexpected EOF".to_string(),
                offset,
                needed_hint: Some(needed_hint),
            }),
        }
    }

    /// Get this error's message.
    pub fn message(&self) -> &str {
        &self.inner.message
    }

    /// Get the offset within the Wasm binary where the error occured.
    pub fn offset(&self) -> usize {
        self.inner.offset
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CustomSectionKind {
    Unknown,
    Name,
    Producers,
    SourceMappingURL,
    Reloc,
    Linking,
}

/// Section code as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/binary/modules.html#sections
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SectionCode<'a> {
    Custom {
        name: &'a str,
        kind: CustomSectionKind,
    },
    Type,       // Function signature declarations
    Alias,      // Aliased indices from nested/parent modules
    Import,     // Import declarations
    Module,     // Module declarations
    Instance,   // Instance definitions
    Function,   // Function declarations
    Table,      // Indirect function table and other tables
    Memory,     // Memory attributes
    Global,     // Global declarations
    Export,     // Exports
    Start,      // Start function declaration
    Element,    // Elements section
    ModuleCode, // Module definitions
    Code,       // Function bodies (code)
    Data,       // Data segments
    DataCount,  // Count of passive data segments
    Tag,        // Tag declarations
}

/// Types as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/syntax/types.html#types
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
    ExnRef,
    Func,
    EmptyBlockType,
}

/// Either a value type or a function type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TypeOrFuncType {
    /// A value type.
    ///
    /// When used as the type for a block, this type is the optional result
    /// type: `[] -> [t?]`.
    Type(Type),

    /// A function type (referenced as an index into the types section).
    FuncType(u32),
}

/// External types as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/syntax/types.html#external-types
#[derive(Debug, Copy, Clone)]
pub enum ExternalKind {
    Function,
    Table,
    Memory,
    Tag,
    Global,
    Type,
    Module,
    Instance,
}

#[derive(Debug, Clone)]
pub enum TypeDef<'a> {
    Func(FuncType),
    Instance(InstanceType<'a>),
    Module(ModuleType<'a>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FuncType {
    pub params: Box<[Type]>,
    pub returns: Box<[Type]>,
}

#[derive(Debug, Clone)]
pub struct InstanceType<'a> {
    pub exports: Box<[ExportType<'a>]>,
}

#[derive(Debug, Clone)]
pub struct ModuleType<'a> {
    pub imports: Box<[crate::Import<'a>]>,
    pub exports: Box<[ExportType<'a>]>,
}

#[derive(Debug, Clone)]
pub struct ExportType<'a> {
    pub name: &'a str,
    pub ty: ImportSectionEntryType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TableType {
    pub element_type: Type,
    /// Initial size of this table, in elements.
    pub initial: u32,
    /// Optional maximum size of the table, in elements.
    pub maximum: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MemoryType {
    /// Whether or not this is a 64-bit memory, using i64 as an index. If this
    /// is false it's a 32-bit memory using i32 as an index.
    ///
    /// This is part of the memory64 proposal in WebAssembly.
    pub memory64: bool,

    /// Whether or not this is a "shared" memory, indicating that it should be
    /// send-able across threads and the `maximum` field is always present for
    /// valid types.
    ///
    /// This is part of the threads proposal in WebAssembly.
    pub shared: bool,

    /// Initial size of this memory, in wasm pages.
    ///
    /// For 32-bit memories (when `memory64` is `false`) this is guaranteed to
    /// be at most `u32::MAX` for valid types.
    pub initial: u64,

    /// Optional maximum size of this memory, in wasm pages.
    ///
    /// For 32-bit memories (when `memory64` is `false`) this is guaranteed to
    /// be at most `u32::MAX` for valid types. This field is always present for
    /// valid wasm memories when `shared` is `true`.
    pub maximum: Option<u64>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TagType {
    pub type_index: u32,
}

impl MemoryType {
    pub fn index_type(&self) -> Type {
        if self.memory64 {
            Type::I64
        } else {
            Type::I32
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GlobalType {
    pub content_type: Type,
    pub mutable: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum ImportSectionEntryType {
    Function(u32),
    Table(TableType),
    Memory(MemoryType),
    Tag(TagType),
    Global(GlobalType),
    Module(u32),
    Instance(u32),
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryImmediate {
    /// Alignment, stored as `n` where the actual alignment is `2^n`
    pub align: u8,

    /// A fixed byte-offset that this memory immediate specifies.
    ///
    /// Note that the memory64 proposal can specify a full 64-bit byte offset
    /// while otherwise only 32-bit offsets are allowed. Once validated
    /// memory immediates for 32-bit memories are guaranteed to be at most
    /// `u32::MAX` whereas 64-bit memories can use the full 64-bits.
    pub offset: u64,

    /// The index of the memory this immediate points to.
    ///
    /// Note that this points within the module's own memory index space, and
    /// is always zero unless the multi-memory proposal of WebAssembly is
    /// enabled.
    pub memory: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct Naming<'a> {
    pub index: u32,
    pub name: &'a str,
}

#[derive(Debug, Copy, Clone)]
pub enum NameType {
    Module,
    Function,
    Local,
    Label,
    Type,
    Table,
    Memory,
    Global,
    Element,
    Data,
    Unknown(u32),
}

#[derive(Debug, Copy, Clone)]
pub enum LinkingType {
    StackPointer(u32),
}

#[derive(Debug, Copy, Clone)]
pub enum RelocType {
    FunctionIndexLEB,
    TableIndexSLEB,
    TableIndexI32,
    GlobalAddrLEB,
    GlobalAddrSLEB,
    GlobalAddrI32,
    TypeIndexLEB,
    GlobalIndexLEB,
}

/// A br_table entries representation.
#[derive(Clone)]
pub struct BrTable<'a> {
    pub(crate) reader: crate::BinaryReader<'a>,
    pub(crate) cnt: u32,
    pub(crate) default: u32,
}

/// An IEEE binary32 immediate floating point value, represented as a u32
/// containing the bitpattern.
///
/// All bit patterns are allowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Ieee32(pub(crate) u32);

impl Ieee32 {
    pub fn bits(self) -> u32 {
        self.0
    }
}

/// An IEEE binary64 immediate floating point value, represented as a u64
/// containing the bitpattern.
///
/// All bit patterns are allowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Ieee64(pub(crate) u64);

impl Ieee64 {
    pub fn bits(self) -> u64 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct V128(pub(crate) [u8; 16]);

impl V128 {
    pub fn bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn i128(&self) -> i128 {
        i128::from_le_bytes(self.0)
    }
}

pub type SIMDLaneIndex = u8;

/// Instructions as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/binary/instructions.html
#[derive(Debug, Clone)]
pub enum Operator<'a> {
    Unreachable,
    Nop,
    Block {
        ty: TypeOrFuncType,
    },
    Loop {
        ty: TypeOrFuncType,
    },
    If {
        ty: TypeOrFuncType,
    },
    Else,
    Try {
        ty: TypeOrFuncType,
    },
    Catch {
        index: u32,
    },
    Throw {
        index: u32,
    },
    Rethrow {
        relative_depth: u32,
    },
    End,
    Br {
        relative_depth: u32,
    },
    BrIf {
        relative_depth: u32,
    },
    BrTable {
        table: BrTable<'a>,
    },
    Return,
    Call {
        function_index: u32,
    },
    CallIndirect {
        index: u32,
        table_index: u32,
    },
    ReturnCall {
        function_index: u32,
    },
    ReturnCallIndirect {
        index: u32,
        table_index: u32,
    },
    Delegate {
        relative_depth: u32,
    },
    CatchAll,
    Drop,
    Select,
    TypedSelect {
        ty: Type,
    },
    LocalGet {
        local_index: u32,
    },
    LocalSet {
        local_index: u32,
    },
    LocalTee {
        local_index: u32,
    },
    GlobalGet {
        global_index: u32,
    },
    GlobalSet {
        global_index: u32,
    },
    I32Load {
        memarg: MemoryImmediate,
    },
    I64Load {
        memarg: MemoryImmediate,
    },
    F32Load {
        memarg: MemoryImmediate,
    },
    F64Load {
        memarg: MemoryImmediate,
    },
    I32Load8S {
        memarg: MemoryImmediate,
    },
    I32Load8U {
        memarg: MemoryImmediate,
    },
    I32Load16S {
        memarg: MemoryImmediate,
    },
    I32Load16U {
        memarg: MemoryImmediate,
    },
    I64Load8S {
        memarg: MemoryImmediate,
    },
    I64Load8U {
        memarg: MemoryImmediate,
    },
    I64Load16S {
        memarg: MemoryImmediate,
    },
    I64Load16U {
        memarg: MemoryImmediate,
    },
    I64Load32S {
        memarg: MemoryImmediate,
    },
    I64Load32U {
        memarg: MemoryImmediate,
    },
    I32Store {
        memarg: MemoryImmediate,
    },
    I64Store {
        memarg: MemoryImmediate,
    },
    F32Store {
        memarg: MemoryImmediate,
    },
    F64Store {
        memarg: MemoryImmediate,
    },
    I32Store8 {
        memarg: MemoryImmediate,
    },
    I32Store16 {
        memarg: MemoryImmediate,
    },
    I64Store8 {
        memarg: MemoryImmediate,
    },
    I64Store16 {
        memarg: MemoryImmediate,
    },
    I64Store32 {
        memarg: MemoryImmediate,
    },
    MemorySize {
        mem: u32,
        mem_byte: u8,
    },
    MemoryGrow {
        mem: u32,
        mem_byte: u8,
    },
    I32Const {
        value: i32,
    },
    I64Const {
        value: i64,
    },
    F32Const {
        value: Ieee32,
    },
    F64Const {
        value: Ieee64,
    },
    RefNull {
        ty: Type,
    },
    RefIsNull,
    RefFunc {
        function_index: u32,
    },
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
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64ExtendI32S,
    I64ExtendI32U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F32DemoteF64,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    F64PromoteF32,
    I32ReinterpretF32,
    I64ReinterpretF64,
    F32ReinterpretI32,
    F64ReinterpretI64,
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    // 0xFC operators
    // Non-trapping Float-to-int Conversions
    I32TruncSatF32S,
    I32TruncSatF32U,
    I32TruncSatF64S,
    I32TruncSatF64U,
    I64TruncSatF32S,
    I64TruncSatF32U,
    I64TruncSatF64S,
    I64TruncSatF64U,

    // 0xFC operators
    // bulk memory https://github.com/WebAssembly/bulk-memory-operations/blob/master/proposals/bulk-memory-operations/Overview.md
    MemoryInit {
        segment: u32,
        mem: u32,
    },
    DataDrop {
        segment: u32,
    },
    MemoryCopy {
        src: u32,
        dst: u32,
    },
    MemoryFill {
        mem: u32,
    },
    TableInit {
        segment: u32,
        table: u32,
    },
    ElemDrop {
        segment: u32,
    },
    TableCopy {
        dst_table: u32,
        src_table: u32,
    },
    TableFill {
        table: u32,
    },
    TableGet {
        table: u32,
    },
    TableSet {
        table: u32,
    },
    TableGrow {
        table: u32,
    },
    TableSize {
        table: u32,
    },

    // 0xFE operators
    // https://github.com/WebAssembly/threads/blob/master/proposals/threads/Overview.md
    MemoryAtomicNotify {
        memarg: MemoryImmediate,
    },
    MemoryAtomicWait32 {
        memarg: MemoryImmediate,
    },
    MemoryAtomicWait64 {
        memarg: MemoryImmediate,
    },
    AtomicFence {
        flags: u8,
    },
    I32AtomicLoad {
        memarg: MemoryImmediate,
    },
    I64AtomicLoad {
        memarg: MemoryImmediate,
    },
    I32AtomicLoad8U {
        memarg: MemoryImmediate,
    },
    I32AtomicLoad16U {
        memarg: MemoryImmediate,
    },
    I64AtomicLoad8U {
        memarg: MemoryImmediate,
    },
    I64AtomicLoad16U {
        memarg: MemoryImmediate,
    },
    I64AtomicLoad32U {
        memarg: MemoryImmediate,
    },
    I32AtomicStore {
        memarg: MemoryImmediate,
    },
    I64AtomicStore {
        memarg: MemoryImmediate,
    },
    I32AtomicStore8 {
        memarg: MemoryImmediate,
    },
    I32AtomicStore16 {
        memarg: MemoryImmediate,
    },
    I64AtomicStore8 {
        memarg: MemoryImmediate,
    },
    I64AtomicStore16 {
        memarg: MemoryImmediate,
    },
    I64AtomicStore32 {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwAdd {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwAdd {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8AddU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16AddU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8AddU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16AddU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32AddU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwSub {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwSub {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8SubU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16SubU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8SubU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16SubU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32SubU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwAnd {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwAnd {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8AndU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16AndU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8AndU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16AndU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32AndU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwOr {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwOr {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8OrU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16OrU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8OrU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16OrU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32OrU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwXor {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwXor {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8XorU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16XorU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8XorU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16XorU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32XorU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwXchg {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwXchg {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8XchgU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16XchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8XchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16XchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32XchgU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmwCmpxchg {
        memarg: MemoryImmediate,
    },
    I64AtomicRmwCmpxchg {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw8CmpxchgU {
        memarg: MemoryImmediate,
    },
    I32AtomicRmw16CmpxchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw8CmpxchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw16CmpxchgU {
        memarg: MemoryImmediate,
    },
    I64AtomicRmw32CmpxchgU {
        memarg: MemoryImmediate,
    },

    // 0xFD operators
    // SIMD https://webassembly.github.io/simd/core/binary/instructions.html
    V128Load {
        memarg: MemoryImmediate,
    },
    V128Load8x8S {
        memarg: MemoryImmediate,
    },
    V128Load8x8U {
        memarg: MemoryImmediate,
    },
    V128Load16x4S {
        memarg: MemoryImmediate,
    },
    V128Load16x4U {
        memarg: MemoryImmediate,
    },
    V128Load32x2S {
        memarg: MemoryImmediate,
    },
    V128Load32x2U {
        memarg: MemoryImmediate,
    },
    V128Load8Splat {
        memarg: MemoryImmediate,
    },
    V128Load16Splat {
        memarg: MemoryImmediate,
    },
    V128Load32Splat {
        memarg: MemoryImmediate,
    },
    V128Load64Splat {
        memarg: MemoryImmediate,
    },
    V128Load32Zero {
        memarg: MemoryImmediate,
    },
    V128Load64Zero {
        memarg: MemoryImmediate,
    },
    V128Store {
        memarg: MemoryImmediate,
    },
    V128Load8Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Load16Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Load32Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Load64Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Store8Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Store16Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Store32Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Store64Lane {
        memarg: MemoryImmediate,
        lane: SIMDLaneIndex,
    },
    V128Const {
        value: V128,
    },
    I8x16Shuffle {
        lanes: [SIMDLaneIndex; 16],
    },
    I8x16ExtractLaneS {
        lane: SIMDLaneIndex,
    },
    I8x16ExtractLaneU {
        lane: SIMDLaneIndex,
    },
    I8x16ReplaceLane {
        lane: SIMDLaneIndex,
    },
    I16x8ExtractLaneS {
        lane: SIMDLaneIndex,
    },
    I16x8ExtractLaneU {
        lane: SIMDLaneIndex,
    },
    I16x8ReplaceLane {
        lane: SIMDLaneIndex,
    },
    I32x4ExtractLane {
        lane: SIMDLaneIndex,
    },
    I32x4ReplaceLane {
        lane: SIMDLaneIndex,
    },
    I64x2ExtractLane {
        lane: SIMDLaneIndex,
    },
    I64x2ReplaceLane {
        lane: SIMDLaneIndex,
    },
    F32x4ExtractLane {
        lane: SIMDLaneIndex,
    },
    F32x4ReplaceLane {
        lane: SIMDLaneIndex,
    },
    F64x2ExtractLane {
        lane: SIMDLaneIndex,
    },
    F64x2ReplaceLane {
        lane: SIMDLaneIndex,
    },
    I8x16Swizzle,
    I8x16Splat,
    I16x8Splat,
    I32x4Splat,
    I64x2Splat,
    F32x4Splat,
    F64x2Splat,
    I8x16Eq,
    I8x16Ne,
    I8x16LtS,
    I8x16LtU,
    I8x16GtS,
    I8x16GtU,
    I8x16LeS,
    I8x16LeU,
    I8x16GeS,
    I8x16GeU,
    I16x8Eq,
    I16x8Ne,
    I16x8LtS,
    I16x8LtU,
    I16x8GtS,
    I16x8GtU,
    I16x8LeS,
    I16x8LeU,
    I16x8GeS,
    I16x8GeU,
    I32x4Eq,
    I32x4Ne,
    I32x4LtS,
    I32x4LtU,
    I32x4GtS,
    I32x4GtU,
    I32x4LeS,
    I32x4LeU,
    I32x4GeS,
    I32x4GeU,
    I64x2Eq,
    I64x2Ne,
    I64x2LtS,
    I64x2GtS,
    I64x2LeS,
    I64x2GeS,
    F32x4Eq,
    F32x4Ne,
    F32x4Lt,
    F32x4Gt,
    F32x4Le,
    F32x4Ge,
    F64x2Eq,
    F64x2Ne,
    F64x2Lt,
    F64x2Gt,
    F64x2Le,
    F64x2Ge,
    V128Not,
    V128And,
    V128AndNot,
    V128Or,
    V128Xor,
    V128Bitselect,
    V128AnyTrue,
    I8x16Abs,
    I8x16Neg,
    I8x16Popcnt,
    I8x16AllTrue,
    I8x16Bitmask,
    I8x16NarrowI16x8S,
    I8x16NarrowI16x8U,
    I8x16Shl,
    I8x16ShrS,
    I8x16ShrU,
    I8x16Add,
    I8x16AddSatS,
    I8x16AddSatU,
    I8x16Sub,
    I8x16SubSatS,
    I8x16SubSatU,
    I8x16MinS,
    I8x16MinU,
    I8x16MaxS,
    I8x16MaxU,
    I8x16RoundingAverageU,
    I16x8ExtAddPairwiseI8x16S,
    I16x8ExtAddPairwiseI8x16U,
    I16x8Abs,
    I16x8Neg,
    I16x8Q15MulrSatS,
    I16x8AllTrue,
    I16x8Bitmask,
    I16x8NarrowI32x4S,
    I16x8NarrowI32x4U,
    I16x8ExtendLowI8x16S,
    I16x8ExtendHighI8x16S,
    I16x8ExtendLowI8x16U,
    I16x8ExtendHighI8x16U,
    I16x8Shl,
    I16x8ShrS,
    I16x8ShrU,
    I16x8Add,
    I16x8AddSatS,
    I16x8AddSatU,
    I16x8Sub,
    I16x8SubSatS,
    I16x8SubSatU,
    I16x8Mul,
    I16x8MinS,
    I16x8MinU,
    I16x8MaxS,
    I16x8MaxU,
    I16x8RoundingAverageU,
    I16x8ExtMulLowI8x16S,
    I16x8ExtMulHighI8x16S,
    I16x8ExtMulLowI8x16U,
    I16x8ExtMulHighI8x16U,
    I32x4ExtAddPairwiseI16x8S,
    I32x4ExtAddPairwiseI16x8U,
    I32x4Abs,
    I32x4Neg,
    I32x4AllTrue,
    I32x4Bitmask,
    I32x4ExtendLowI16x8S,
    I32x4ExtendHighI16x8S,
    I32x4ExtendLowI16x8U,
    I32x4ExtendHighI16x8U,
    I32x4Shl,
    I32x4ShrS,
    I32x4ShrU,
    I32x4Add,
    I32x4Sub,
    I32x4Mul,
    I32x4MinS,
    I32x4MinU,
    I32x4MaxS,
    I32x4MaxU,
    I32x4DotI16x8S,
    I32x4ExtMulLowI16x8S,
    I32x4ExtMulHighI16x8S,
    I32x4ExtMulLowI16x8U,
    I32x4ExtMulHighI16x8U,
    I64x2Abs,
    I64x2Neg,
    I64x2AllTrue,
    I64x2Bitmask,
    I64x2ExtendLowI32x4S,
    I64x2ExtendHighI32x4S,
    I64x2ExtendLowI32x4U,
    I64x2ExtendHighI32x4U,
    I64x2Shl,
    I64x2ShrS,
    I64x2ShrU,
    I64x2Add,
    I64x2Sub,
    I64x2Mul,
    I64x2ExtMulLowI32x4S,
    I64x2ExtMulHighI32x4S,
    I64x2ExtMulLowI32x4U,
    I64x2ExtMulHighI32x4U,
    F32x4Ceil,
    F32x4Floor,
    F32x4Trunc,
    F32x4Nearest,
    F32x4Abs,
    F32x4Neg,
    F32x4Sqrt,
    F32x4Add,
    F32x4Sub,
    F32x4Mul,
    F32x4Div,
    F32x4Min,
    F32x4Max,
    F32x4PMin,
    F32x4PMax,
    F64x2Ceil,
    F64x2Floor,
    F64x2Trunc,
    F64x2Nearest,
    F64x2Abs,
    F64x2Neg,
    F64x2Sqrt,
    F64x2Add,
    F64x2Sub,
    F64x2Mul,
    F64x2Div,
    F64x2Min,
    F64x2Max,
    F64x2PMin,
    F64x2PMax,
    I32x4TruncSatF32x4S,
    I32x4TruncSatF32x4U,
    F32x4ConvertI32x4S,
    F32x4ConvertI32x4U,
    I32x4TruncSatF64x2SZero,
    I32x4TruncSatF64x2UZero,
    F64x2ConvertLowI32x4S,
    F64x2ConvertLowI32x4U,
    F32x4DemoteF64x2Zero,
    F64x2PromoteLowF32x4,
    I8x16RelaxedSwizzle,
    I32x4RelaxedTruncSatF32x4S,
    I32x4RelaxedTruncSatF32x4U,
    I32x4RelaxedTruncSatF64x2SZero,
    I32x4RelaxedTruncSatF64x2UZero,
    F32x4Fma,
    F32x4Fms,
    F64x2Fma,
    F64x2Fms,
    I8x16LaneSelect,
    I16x8LaneSelect,
    I32x4LaneSelect,
    I64x2LaneSelect,
    F32x4RelaxedMin,
    F32x4RelaxedMax,
    F64x2RelaxedMin,
    F64x2RelaxedMax,
}
