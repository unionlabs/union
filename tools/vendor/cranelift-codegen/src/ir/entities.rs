//! Cranelift IR entity references.
//!
//! Instructions in Cranelift IR need to reference other entities in the function. This can be other
//! parts of the function like basic blocks or stack slots, or it can be external entities
//! that are declared in the function preamble in the text format.
//!
//! These entity references in instruction operands are not implemented as Rust references both
//! because Rust's ownership and mutability rules make it difficult, and because 64-bit pointers
//! take up a lot of space, and we want a compact in-memory representation. Instead, entity
//! references are structs wrapping a `u32` index into a table in the `Function` main data
//! structure. There is a separate index type for each entity type, so we don't lose type safety.
//!
//! The `entities` module defines public types for the entity references along with constants
//! representing an invalid reference. We prefer to use `Option<EntityRef>` whenever possible, but
//! unfortunately that type is twice as large as the 32-bit index type on its own. Thus, compact
//! data structures use the `PackedOption<EntityRef>` representation, while function arguments and
//! return values prefer the more Rust-like `Option<EntityRef>` variant.
//!
//! The entity references all implement the `Display` trait in a way that matches the textual IR
//! format.

use crate::entity::entity_impl;
use core::fmt;
use core::u32;
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// An opaque reference to a [basic block](https://en.wikipedia.org/wiki/Basic_block) in a
/// [`Function`](super::function::Function).
///
/// You can get a `Block` using
/// [`FunctionBuilder::create_block`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.create_block)
///
/// While the order is stable, it is arbitrary and does not necessarily resemble the layout order.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Block(u32);
entity_impl!(Block, "block");

impl Block {
    /// Create a new block reference from its number. This corresponds to the `blockNN` representation.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to an SSA value.
///
/// You can get a constant `Value` from the following
/// [`InstBuilder`](super::InstBuilder) instructions:
///
/// - [`iconst`](super::InstBuilder::iconst) for integer constants
/// - [`f32const`](super::InstBuilder::f32const) for 32-bit float constants
/// - [`f64const`](super::InstBuilder::f64const) for 64-bit float constants
/// - [`bconst`](super::InstBuilder::bconst) for boolean constants
/// - [`vconst`](super::InstBuilder::vconst) for vector constants
/// - [`null`](super::InstBuilder::null) for null reference constants
///
/// Any `InstBuilder` instruction that has an output will also return a `Value`.
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Value(u32);
entity_impl!(Value, "v");

impl Value {
    /// Create a value from its number representation.
    /// This is the number in the `vNN` notation.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX / 2 {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to an instruction in a [`Function`](super::Function).
///
/// Most usage of `Inst` is internal. `Inst`ructions are returned by
/// [`InstBuilder`](super::InstBuilder) instructions that do not return a
/// [`Value`], such as control flow and trap instructions.
///
/// If you look around the API, you can find many inventive uses for `Inst`,
/// such as [annotating specific instructions with a comment][inst_comment]
/// or [performing reflection at compile time](super::DataFlowGraph::analyze_branch)
/// on the type of instruction.
///
/// [inst_comment]: https://github.com/bjorn3/rustc_codegen_cranelift/blob/0f8814fd6da3d436a90549d4bb19b94034f2b19c/src/pretty_clif.rs
///
/// While the order is stable, it is arbitrary and does not necessarily resemble the layout order.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Inst(u32);
entity_impl!(Inst, "inst");

/// An opaque reference to a stack slot.
///
/// Stack slots represent an address on the
/// [call stack](https://en.wikipedia.org/wiki/Call_stack).
///
/// `StackSlot`s can be created with
/// [`FunctionBuilder::create_stackslot`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.create_stack_slot).
///
/// `StackSlot`s are most often used with
/// [`stack_addr`](super::InstBuilder::stack_addr),
/// [`stack_load`](super::InstBuilder::stack_load), and
/// [`stack_store`](super::InstBuilder::stack_store).
///
/// While the order is stable, it is arbitrary and does not necessarily resemble the stack order.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct StackSlot(u32);
entity_impl!(StackSlot, "ss");

impl StackSlot {
    /// Create a new stack slot reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a global value.
///
/// A `GlobalValue` is a [`Value`](Value) that will be live across the entire
/// function lifetime. It can be preloaded from other global values.
///
/// You can create a `GlobalValue` in the following ways:
///
/// - When compiling to WASM, you can use it to load values from a
/// [`VmContext`](super::GlobalValueData::VMContext) using
/// [`FuncEnvironment::make_global`](https://docs.rs/cranelift-wasm/*/cranelift_wasm/trait.FuncEnvironment.html#tymethod.make_global).
/// - When compiling to native code, you can use it for objects in static memory with
/// [`Module::declare_data_in_func`](https://docs.rs/cranelift-module/*/cranelift_module/trait.Module.html#method.declare_data_in_func).
/// - For any compilation target, it can be registered with
/// [`FunctionBuilder::create_global_value`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.create_global_value).
///
/// `GlobalValue`s can be retrieved with
/// [`InstBuilder:global_value`](super::InstBuilder::global_value).
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct GlobalValue(u32);
entity_impl!(GlobalValue, "gv");

impl GlobalValue {
    /// Create a new global value reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a constant.
///
/// You can store [`ConstantData`](super::ConstantData) in a
/// [`ConstantPool`](super::ConstantPool) for efficient storage and retrieval.
/// See [`ConstantPool::insert`](super::ConstantPool::insert).
///
/// While the order is stable, it is arbitrary and does not necessarily resemble the order in which
/// the constants are written in the constant pool.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Constant(u32);
entity_impl!(Constant, "const");

impl Constant {
    /// Create a const reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to an immediate.
///
/// Some immediates (e.g. SIMD shuffle masks) are too large to store in the
/// [`InstructionData`](super::instructions::InstructionData) struct and therefore must be
/// tracked separately in [`DataFlowGraph::immediates`](super::dfg::DataFlowGraph). `Immediate`
/// provides a way to reference values stored there.
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Immediate(u32);
entity_impl!(Immediate, "imm");

impl Immediate {
    /// Create an immediate reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a [jump table](https://en.wikipedia.org/wiki/Branch_table).
///
/// `JumpTable`s are used for indirect branching and are specialized for dense,
/// 0-based jump offsets. If you want a jump table which doesn't start at 0,
/// or is not contiguous, consider using a [`Switch`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.Switch.html) instead.
///
/// `JumpTable` are used with [`br_table`](super::InstBuilder::br_table).
///
/// `JumpTable`s can be created with
/// [`create_jump_table`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.create_jump_table).
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct JumpTable(u32);
entity_impl!(JumpTable, "jt");

impl JumpTable {
    /// Create a new jump table reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to another [`Function`](super::Function).
///
/// `FuncRef`s are used for [direct](super::InstBuilder::call) function calls
/// and by [`func_addr`](super::InstBuilder::func_addr) for use in
/// [indirect](super::InstBuilder::call_indirect) function calls.
///
/// `FuncRef`s can be created with
///
/// - [`FunctionBuilder::import_function`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.import_function)
/// for external functions
/// - [`Module::declare_func_in_func`](https://docs.rs/cranelift-module/*/cranelift_module/trait.Module.html#method.declare_func_in_func)
/// for functions declared elsewhere in the same native
/// [`Module`](https://docs.rs/cranelift-module/*/cranelift_module/trait.Module.html)
/// - [`FuncEnvironment::make_direct_func`](https://docs.rs/cranelift-wasm/*/cranelift_wasm/trait.FuncEnvironment.html#tymethod.make_direct_func)
/// for functions declared in the same WebAssembly
/// [`FuncEnvironment`](https://docs.rs/cranelift-wasm/*/cranelift_wasm/trait.FuncEnvironment.html#tymethod.make_direct_func)
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct FuncRef(u32);
entity_impl!(FuncRef, "fn");

impl FuncRef {
    /// Create a new external function reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a function [`Signature`](super::Signature).
///
/// `SigRef`s are used to declare a function with
/// [`FunctionBuiler::import_function`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.import_function)
/// as well as to make an [indirect function call](super::InstBuilder::call_indirect).
///
/// `SigRef`s can be created with
/// [`FunctionBuilder::import_signature`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.import_signature).
///
/// You can retrieve the [`Signature`](super::Signature) that was used to create a `SigRef` with
/// [`FunctionBuilder::signature`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.signature) or
/// [`func.dfg.signatures`](super::dfg::DataFlowGraph::signatures).
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct SigRef(u32);
entity_impl!(SigRef, "sig");

impl SigRef {
    /// Create a new function signature reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a [heap](https://en.wikipedia.org/wiki/Memory_management#DYNAMIC).
///
/// Heaps are used to access dynamically allocated memory through
/// [`heap_addr`](super::InstBuilder::heap_addr).
///
/// To create a heap, use [`FunctionBuilder::create_heap`](https://docs.rs/cranelift-frontend/*/cranelift_frontend/struct.FunctionBuilder.html#method.create_heap).
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Heap(u32);
entity_impl!(Heap, "heap");

impl Heap {
    /// Create a new heap reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to a [WebAssembly
/// table](https://developer.mozilla.org/en-US/docs/WebAssembly/Understanding_the_text_format#WebAssembly_tables).
///
/// `Table`s are used to store a list of function references.
/// They can be created with [`FuncEnvironment::make_table`](https://docs.rs/cranelift-wasm/*/cranelift_wasm/trait.FuncEnvironment.html#tymethod.make_table).
/// They can be used with
/// [`FuncEnvironment::translate_call_indirect`](https://docs.rs/cranelift-wasm/*/cranelift_wasm/trait.FuncEnvironment.html#tymethod.translate_call_indirect).
///
/// While the order is stable, it is arbitrary.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Table(u32);
entity_impl!(Table, "table");

impl Table {
    /// Create a new table reference from its number.
    ///
    /// This method is for use by the parser.
    pub fn with_number(n: u32) -> Option<Self> {
        if n < u32::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// An opaque reference to any of the entities defined in this module that can appear in CLIF IR.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum AnyEntity {
    /// The whole function.
    Function,
    /// a basic block.
    Block(Block),
    /// An instruction.
    Inst(Inst),
    /// An SSA value.
    Value(Value),
    /// A stack slot.
    StackSlot(StackSlot),
    /// A Global value.
    GlobalValue(GlobalValue),
    /// A jump table.
    JumpTable(JumpTable),
    /// A constant.
    Constant(Constant),
    /// An external function.
    FuncRef(FuncRef),
    /// A function call signature.
    SigRef(SigRef),
    /// A heap.
    Heap(Heap),
    /// A table.
    Table(Table),
    /// A function's stack limit
    StackLimit,
}

impl fmt::Display for AnyEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Function => write!(f, "function"),
            Self::Block(r) => r.fmt(f),
            Self::Inst(r) => r.fmt(f),
            Self::Value(r) => r.fmt(f),
            Self::StackSlot(r) => r.fmt(f),
            Self::GlobalValue(r) => r.fmt(f),
            Self::JumpTable(r) => r.fmt(f),
            Self::Constant(r) => r.fmt(f),
            Self::FuncRef(r) => r.fmt(f),
            Self::SigRef(r) => r.fmt(f),
            Self::Heap(r) => r.fmt(f),
            Self::Table(r) => r.fmt(f),
            Self::StackLimit => write!(f, "stack_limit"),
        }
    }
}

impl fmt::Debug for AnyEntity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self as &dyn fmt::Display).fmt(f)
    }
}

impl From<Block> for AnyEntity {
    fn from(r: Block) -> Self {
        Self::Block(r)
    }
}

impl From<Inst> for AnyEntity {
    fn from(r: Inst) -> Self {
        Self::Inst(r)
    }
}

impl From<Value> for AnyEntity {
    fn from(r: Value) -> Self {
        Self::Value(r)
    }
}

impl From<StackSlot> for AnyEntity {
    fn from(r: StackSlot) -> Self {
        Self::StackSlot(r)
    }
}

impl From<GlobalValue> for AnyEntity {
    fn from(r: GlobalValue) -> Self {
        Self::GlobalValue(r)
    }
}

impl From<JumpTable> for AnyEntity {
    fn from(r: JumpTable) -> Self {
        Self::JumpTable(r)
    }
}

impl From<Constant> for AnyEntity {
    fn from(r: Constant) -> Self {
        Self::Constant(r)
    }
}

impl From<FuncRef> for AnyEntity {
    fn from(r: FuncRef) -> Self {
        Self::FuncRef(r)
    }
}

impl From<SigRef> for AnyEntity {
    fn from(r: SigRef) -> Self {
        Self::SigRef(r)
    }
}

impl From<Heap> for AnyEntity {
    fn from(r: Heap) -> Self {
        Self::Heap(r)
    }
}

impl From<Table> for AnyEntity {
    fn from(r: Table) -> Self {
        Self::Table(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use core::u32;

    #[test]
    fn value_with_number() {
        assert_eq!(Value::with_number(0).unwrap().to_string(), "v0");
        assert_eq!(Value::with_number(1).unwrap().to_string(), "v1");

        assert_eq!(Value::with_number(u32::MAX / 2), None);
        assert!(Value::with_number(u32::MAX / 2 - 1).is_some());
    }

    #[test]
    fn memory() {
        use crate::packed_option::PackedOption;
        use core::mem;
        // This is the whole point of `PackedOption`.
        assert_eq!(
            mem::size_of::<Value>(),
            mem::size_of::<PackedOption<Value>>()
        );
    }

    #[test]
    fn constant_with_number() {
        assert_eq!(Constant::with_number(0).unwrap().to_string(), "const0");
        assert_eq!(Constant::with_number(1).unwrap().to_string(), "const1");
    }
}
