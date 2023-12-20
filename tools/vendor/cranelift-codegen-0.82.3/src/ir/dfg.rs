//! Data flow graph tracking Instructions, Values, and blocks.

use crate::entity::{self, PrimaryMap, SecondaryMap};
use crate::ir;
use crate::ir::builder::ReplaceBuilder;
use crate::ir::extfunc::ExtFuncData;
use crate::ir::instructions::{BranchInfo, CallInfo, InstructionData};
use crate::ir::{types, ConstantData, ConstantPool, Immediate};
use crate::ir::{
    Block, FuncRef, Inst, SigRef, Signature, SourceLoc, Type, Value, ValueLabelAssignments,
    ValueList, ValueListPool,
};
use crate::packed_option::ReservedValue;
use crate::write::write_operands;
use crate::HashMap;
use alloc::vec::Vec;
use core::fmt;
use core::iter;
use core::mem;
use core::ops::{Index, IndexMut};
use core::u16;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// A data flow graph defines all instructions and basic blocks in a function as well as
/// the data flow dependencies between them. The DFG also tracks values which can be either
/// instruction results or block parameters.
///
/// The layout of blocks in the function and of instructions in each block is recorded by the
/// `Layout` data structure which forms the other half of the function representation.
///
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct DataFlowGraph {
    /// Data about all of the instructions in the function, including opcodes and operands.
    /// The instructions in this map are not in program order. That is tracked by `Layout`, along
    /// with the block containing each instruction.
    insts: PrimaryMap<Inst, InstructionData>,

    /// List of result values for each instruction.
    ///
    /// This map gets resized automatically by `make_inst()` so it is always in sync with the
    /// primary `insts` map.
    results: SecondaryMap<Inst, ValueList>,

    /// basic blocks in the function and their parameters.
    ///
    /// This map is not in program order. That is handled by `Layout`, and so is the sequence of
    /// instructions contained in each block.
    blocks: PrimaryMap<Block, BlockData>,

    /// Memory pool of value lists.
    ///
    /// The `ValueList` references into this pool appear in many places:
    ///
    /// - Instructions in `insts` that don't have room for their entire argument list inline.
    /// - Instruction result values in `results`.
    /// - block parameters in `blocks`.
    pub value_lists: ValueListPool,

    /// Primary value table with entries for all values.
    values: PrimaryMap<Value, ValueData>,

    /// Function signature table. These signatures are referenced by indirect call instructions as
    /// well as the external function references.
    pub signatures: PrimaryMap<SigRef, Signature>,

    /// The pre-legalization signature for each entry in `signatures`, if any.
    pub old_signatures: SecondaryMap<SigRef, Option<Signature>>,

    /// External function references. These are functions that can be called directly.
    pub ext_funcs: PrimaryMap<FuncRef, ExtFuncData>,

    /// Saves Value labels.
    pub values_labels: Option<HashMap<Value, ValueLabelAssignments>>,

    /// Constants used within the function
    pub constants: ConstantPool,

    /// Stores large immediates that otherwise will not fit on InstructionData
    pub immediates: PrimaryMap<Immediate, ConstantData>,
}

impl DataFlowGraph {
    /// Create a new empty `DataFlowGraph`.
    pub fn new() -> Self {
        Self {
            insts: PrimaryMap::new(),
            results: SecondaryMap::new(),
            blocks: PrimaryMap::new(),
            value_lists: ValueListPool::new(),
            values: PrimaryMap::new(),
            signatures: PrimaryMap::new(),
            old_signatures: SecondaryMap::new(),
            ext_funcs: PrimaryMap::new(),
            values_labels: None,
            constants: ConstantPool::new(),
            immediates: PrimaryMap::new(),
        }
    }

    /// Clear everything.
    pub fn clear(&mut self) {
        self.insts.clear();
        self.results.clear();
        self.blocks.clear();
        self.value_lists.clear();
        self.values.clear();
        self.signatures.clear();
        self.old_signatures.clear();
        self.ext_funcs.clear();
        self.values_labels = None;
        self.constants.clear();
        self.immediates.clear();
    }

    /// Get the total number of instructions created in this function, whether they are currently
    /// inserted in the layout or not.
    ///
    /// This is intended for use with `SecondaryMap::with_capacity`.
    pub fn num_insts(&self) -> usize {
        self.insts.len()
    }

    /// Returns `true` if the given instruction reference is valid.
    pub fn inst_is_valid(&self, inst: Inst) -> bool {
        self.insts.is_valid(inst)
    }

    /// Get the total number of basic blocks created in this function, whether they are
    /// currently inserted in the layout or not.
    ///
    /// This is intended for use with `SecondaryMap::with_capacity`.
    pub fn num_blocks(&self) -> usize {
        self.blocks.len()
    }

    /// Returns `true` if the given block reference is valid.
    pub fn block_is_valid(&self, block: Block) -> bool {
        self.blocks.is_valid(block)
    }

    /// Get the total number of values.
    pub fn num_values(&self) -> usize {
        self.values.len()
    }

    /// Starts collection of debug information.
    pub fn collect_debug_info(&mut self) {
        if self.values_labels.is_none() {
            self.values_labels = Some(HashMap::new());
        }
    }

    /// Inserts a `ValueLabelAssignments::Alias` for `to_alias` if debug info
    /// collection is enabled.
    pub fn add_value_label_alias(&mut self, to_alias: Value, from: SourceLoc, value: Value) {
        if let Some(values_labels) = self.values_labels.as_mut() {
            values_labels.insert(to_alias, ir::ValueLabelAssignments::Alias { from, value });
        }
    }
}

/// Resolve value aliases.
///
/// Find the original SSA value that `value` aliases, or None if an
/// alias cycle is detected.
fn maybe_resolve_aliases(values: &PrimaryMap<Value, ValueData>, value: Value) -> Option<Value> {
    let mut v = value;

    // Note that values may be empty here.
    for _ in 0..=values.len() {
        if let ValueData::Alias { original, .. } = values[v] {
            v = original;
        } else {
            return Some(v);
        }
    }

    None
}

/// Resolve value aliases.
///
/// Find the original SSA value that `value` aliases.
fn resolve_aliases(values: &PrimaryMap<Value, ValueData>, value: Value) -> Value {
    if let Some(v) = maybe_resolve_aliases(values, value) {
        v
    } else {
        panic!("Value alias loop detected for {}", value);
    }
}

/// Iterator over all Values in a DFG
pub struct Values<'a> {
    inner: entity::Iter<'a, Value, ValueData>,
}

/// Check for non-values
fn valid_valuedata(data: &ValueData) -> bool {
    if let ValueData::Alias {
        ty: types::INVALID,
        original,
    } = *data
    {
        if original == Value::reserved_value() {
            return false;
        }
    }
    true
}

impl<'a> Iterator for Values<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .by_ref()
            .find(|kv| valid_valuedata(kv.1))
            .map(|kv| kv.0)
    }
}

/// Handling values.
///
/// Values are either block parameters or instruction results.
impl DataFlowGraph {
    /// Allocate an extended value entry.
    fn make_value(&mut self, data: ValueData) -> Value {
        self.values.push(data)
    }

    /// Get an iterator over all values.
    pub fn values<'a>(&'a self) -> Values {
        Values {
            inner: self.values.iter(),
        }
    }

    /// Check if a value reference is valid.
    pub fn value_is_valid(&self, v: Value) -> bool {
        self.values.is_valid(v)
    }

    /// Get the type of a value.
    pub fn value_type(&self, v: Value) -> Type {
        self.values[v].ty()
    }

    /// Get the definition of a value.
    ///
    /// This is either the instruction that defined it or the Block that has the value as an
    /// parameter.
    pub fn value_def(&self, v: Value) -> ValueDef {
        match self.values[v] {
            ValueData::Inst { inst, num, .. } => ValueDef::Result(inst, num as usize),
            ValueData::Param { block, num, .. } => ValueDef::Param(block, num as usize),
            ValueData::Alias { original, .. } => {
                // Make sure we only recurse one level. `resolve_aliases` has safeguards to
                // detect alias loops without overrunning the stack.
                self.value_def(self.resolve_aliases(original))
            }
        }
    }

    /// Determine if `v` is an attached instruction result / block parameter.
    ///
    /// An attached value can't be attached to something else without first being detached.
    ///
    /// Value aliases are not considered to be attached to anything. Use `resolve_aliases()` to
    /// determine if the original aliased value is attached.
    pub fn value_is_attached(&self, v: Value) -> bool {
        use self::ValueData::*;
        match self.values[v] {
            Inst { inst, num, .. } => Some(&v) == self.inst_results(inst).get(num as usize),
            Param { block, num, .. } => Some(&v) == self.block_params(block).get(num as usize),
            Alias { .. } => false,
        }
    }

    /// Resolve value aliases.
    ///
    /// Find the original SSA value that `value` aliases.
    pub fn resolve_aliases(&self, value: Value) -> Value {
        resolve_aliases(&self.values, value)
    }

    /// Resolve all aliases among inst's arguments.
    ///
    /// For each argument of inst which is defined by an alias, replace the
    /// alias with the aliased value.
    pub fn resolve_aliases_in_arguments(&mut self, inst: Inst) {
        for arg in self.insts[inst].arguments_mut(&mut self.value_lists) {
            let resolved = resolve_aliases(&self.values, *arg);
            if resolved != *arg {
                *arg = resolved;
            }
        }
    }

    /// Turn a value into an alias of another.
    ///
    /// Change the `dest` value to behave as an alias of `src`. This means that all uses of `dest`
    /// will behave as if they used that value `src`.
    ///
    /// The `dest` value can't be attached to an instruction or block.
    pub fn change_to_alias(&mut self, dest: Value, src: Value) {
        debug_assert!(!self.value_is_attached(dest));
        // Try to create short alias chains by finding the original source value.
        // This also avoids the creation of loops.
        let original = self.resolve_aliases(src);
        debug_assert_ne!(
            dest, original,
            "Aliasing {} to {} would create a loop",
            dest, src
        );
        let ty = self.value_type(original);
        debug_assert_eq!(
            self.value_type(dest),
            ty,
            "Aliasing {} to {} would change its type {} to {}",
            dest,
            src,
            self.value_type(dest),
            ty
        );
        debug_assert_ne!(ty, types::INVALID);

        self.values[dest] = ValueData::Alias { ty, original };
    }

    /// Replace the results of one instruction with aliases to the results of another.
    ///
    /// Change all the results of `dest_inst` to behave as aliases of
    /// corresponding results of `src_inst`, as if calling change_to_alias for
    /// each.
    ///
    /// After calling this instruction, `dest_inst` will have had its results
    /// cleared, so it likely needs to be removed from the graph.
    ///
    pub fn replace_with_aliases(&mut self, dest_inst: Inst, src_inst: Inst) {
        debug_assert_ne!(
            dest_inst, src_inst,
            "Replacing {} with itself would create a loop",
            dest_inst
        );
        debug_assert_eq!(
            self.results[dest_inst].len(&self.value_lists),
            self.results[src_inst].len(&self.value_lists),
            "Replacing {} with {} would produce a different number of results.",
            dest_inst,
            src_inst
        );

        for (&dest, &src) in self.results[dest_inst]
            .as_slice(&self.value_lists)
            .iter()
            .zip(self.results[src_inst].as_slice(&self.value_lists))
        {
            let original = src;
            let ty = self.value_type(original);
            debug_assert_eq!(
                self.value_type(dest),
                ty,
                "Aliasing {} to {} would change its type {} to {}",
                dest,
                src,
                self.value_type(dest),
                ty
            );
            debug_assert_ne!(ty, types::INVALID);

            self.values[dest] = ValueData::Alias { ty, original };
        }

        self.clear_results(dest_inst);
    }
}

/// Where did a value come from?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueDef {
    /// Value is the n'th result of an instruction.
    Result(Inst, usize),
    /// Value is the n'th parameter to a block.
    Param(Block, usize),
}

impl ValueDef {
    /// Unwrap the instruction where the value was defined, or panic.
    pub fn unwrap_inst(&self) -> Inst {
        self.inst().expect("Value is not an instruction result")
    }

    /// Get the instruction where the value was defined, if any.
    pub fn inst(&self) -> Option<Inst> {
        match *self {
            Self::Result(inst, _) => Some(inst),
            _ => None,
        }
    }

    /// Unwrap the block there the parameter is defined, or panic.
    pub fn unwrap_block(&self) -> Block {
        match *self {
            Self::Param(block, _) => block,
            _ => panic!("Value is not a block parameter"),
        }
    }

    /// Get the program point where the value was defined.
    pub fn pp(self) -> ir::ExpandedProgramPoint {
        self.into()
    }

    /// Get the number component of this definition.
    ///
    /// When multiple values are defined at the same program point, this indicates the index of
    /// this value.
    pub fn num(self) -> usize {
        match self {
            Self::Result(_, n) | Self::Param(_, n) => n,
        }
    }
}

/// Internal table storage for extended values.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
enum ValueData {
    /// Value is defined by an instruction.
    Inst { ty: Type, num: u16, inst: Inst },

    /// Value is a block parameter.
    Param { ty: Type, num: u16, block: Block },

    /// Value is an alias of another value.
    /// An alias value can't be linked as an instruction result or block parameter. It is used as a
    /// placeholder when the original instruction or block has been rewritten or modified.
    Alias { ty: Type, original: Value },
}

impl ValueData {
    fn ty(&self) -> Type {
        match *self {
            ValueData::Inst { ty, .. }
            | ValueData::Param { ty, .. }
            | ValueData::Alias { ty, .. } => ty,
        }
    }
}

/// Instructions.
///
impl DataFlowGraph {
    /// Create a new instruction.
    ///
    /// The type of the first result is indicated by `data.ty`. If the instruction produces
    /// multiple results, also call `make_inst_results` to allocate value table entries.
    pub fn make_inst(&mut self, data: InstructionData) -> Inst {
        let n = self.num_insts() + 1;
        self.results.resize(n);
        self.insts.push(data)
    }

    /// Returns an object that displays `inst`.
    pub fn display_inst<'a>(&'a self, inst: Inst) -> DisplayInst<'a> {
        DisplayInst(self, inst)
    }

    /// Get all value arguments on `inst` as a slice.
    pub fn inst_args(&self, inst: Inst) -> &[Value] {
        self.insts[inst].arguments(&self.value_lists)
    }

    /// Get all value arguments on `inst` as a mutable slice.
    pub fn inst_args_mut(&mut self, inst: Inst) -> &mut [Value] {
        self.insts[inst].arguments_mut(&mut self.value_lists)
    }

    /// Get the fixed value arguments on `inst` as a slice.
    pub fn inst_fixed_args(&self, inst: Inst) -> &[Value] {
        let num_fixed_args = self[inst]
            .opcode()
            .constraints()
            .num_fixed_value_arguments();
        &self.inst_args(inst)[..num_fixed_args]
    }

    /// Get the fixed value arguments on `inst` as a mutable slice.
    pub fn inst_fixed_args_mut(&mut self, inst: Inst) -> &mut [Value] {
        let num_fixed_args = self[inst]
            .opcode()
            .constraints()
            .num_fixed_value_arguments();
        &mut self.inst_args_mut(inst)[..num_fixed_args]
    }

    /// Get the variable value arguments on `inst` as a slice.
    pub fn inst_variable_args(&self, inst: Inst) -> &[Value] {
        let num_fixed_args = self[inst]
            .opcode()
            .constraints()
            .num_fixed_value_arguments();
        &self.inst_args(inst)[num_fixed_args..]
    }

    /// Get the variable value arguments on `inst` as a mutable slice.
    pub fn inst_variable_args_mut(&mut self, inst: Inst) -> &mut [Value] {
        let num_fixed_args = self[inst]
            .opcode()
            .constraints()
            .num_fixed_value_arguments();
        &mut self.inst_args_mut(inst)[num_fixed_args..]
    }

    /// Create result values for an instruction that produces multiple results.
    ///
    /// Instructions that produce no result values only need to be created with `make_inst`,
    /// otherwise call `make_inst_results` to allocate value table entries for the results.
    ///
    /// The result value types are determined from the instruction's value type constraints and the
    /// provided `ctrl_typevar` type for polymorphic instructions. For non-polymorphic
    /// instructions, `ctrl_typevar` is ignored, and `INVALID` can be used.
    ///
    /// The type of the first result value is also set, even if it was already set in the
    /// `InstructionData` passed to `make_inst`. If this function is called with a single-result
    /// instruction, that is the only effect.
    pub fn make_inst_results(&mut self, inst: Inst, ctrl_typevar: Type) -> usize {
        self.make_inst_results_reusing(inst, ctrl_typevar, iter::empty())
    }

    /// Create result values for `inst`, reusing the provided detached values.
    ///
    /// Create a new set of result values for `inst` using `ctrl_typevar` to determine the result
    /// types. Any values provided by `reuse` will be reused. When `reuse` is exhausted or when it
    /// produces `None`, a new value is created.
    pub fn make_inst_results_reusing<I>(
        &mut self,
        inst: Inst,
        ctrl_typevar: Type,
        reuse: I,
    ) -> usize
    where
        I: Iterator<Item = Option<Value>>,
    {
        let mut reuse = reuse.fuse();

        self.results[inst].clear(&mut self.value_lists);

        // Get the call signature if this is a function call.
        if let Some(sig) = self.call_signature(inst) {
            // Create result values corresponding to the call return types.
            debug_assert_eq!(
                self.insts[inst].opcode().constraints().num_fixed_results(),
                0
            );
            let num_results = self.signatures[sig].returns.len();
            for res_idx in 0..num_results {
                let ty = self.signatures[sig].returns[res_idx].value_type;
                if let Some(Some(v)) = reuse.next() {
                    debug_assert_eq!(self.value_type(v), ty, "Reused {} is wrong type", ty);
                    self.attach_result(inst, v);
                } else {
                    self.append_result(inst, ty);
                }
            }
            num_results
        } else {
            // Create result values corresponding to the opcode's constraints.
            let constraints = self.insts[inst].opcode().constraints();
            let num_results = constraints.num_fixed_results();
            for res_idx in 0..num_results {
                let ty = constraints.result_type(res_idx, ctrl_typevar);
                if let Some(Some(v)) = reuse.next() {
                    debug_assert_eq!(self.value_type(v), ty, "Reused {} is wrong type", ty);
                    self.attach_result(inst, v);
                } else {
                    self.append_result(inst, ty);
                }
            }
            num_results
        }
    }

    /// Create a `ReplaceBuilder` that will replace `inst` with a new instruction in place.
    pub fn replace(&mut self, inst: Inst) -> ReplaceBuilder {
        ReplaceBuilder::new(self, inst)
    }

    /// Detach the list of result values from `inst` and return it.
    ///
    /// This leaves `inst` without any result values. New result values can be created by calling
    /// `make_inst_results` or by using a `replace(inst)` builder.
    pub fn detach_results(&mut self, inst: Inst) -> ValueList {
        self.results[inst].take()
    }

    /// Clear the list of result values from `inst`.
    ///
    /// This leaves `inst` without any result values. New result values can be created by calling
    /// `make_inst_results` or by using a `replace(inst)` builder.
    pub fn clear_results(&mut self, inst: Inst) {
        self.results[inst].clear(&mut self.value_lists)
    }

    /// Attach an existing value to the result value list for `inst`.
    ///
    /// The `res` value is appended to the end of the result list.
    ///
    /// This is a very low-level operation. Usually, instruction results with the correct types are
    /// created automatically. The `res` value must not be attached to anything else.
    pub fn attach_result(&mut self, inst: Inst, res: Value) {
        debug_assert!(!self.value_is_attached(res));
        let num = self.results[inst].push(res, &mut self.value_lists);
        debug_assert!(num <= u16::MAX as usize, "Too many result values");
        let ty = self.value_type(res);
        self.values[res] = ValueData::Inst {
            ty,
            num: num as u16,
            inst,
        };
    }

    /// Replace an instruction result with a new value of type `new_type`.
    ///
    /// The `old_value` must be an attached instruction result.
    ///
    /// The old value is left detached, so it should probably be changed into something else.
    ///
    /// Returns the new value.
    pub fn replace_result(&mut self, old_value: Value, new_type: Type) -> Value {
        let (num, inst) = match self.values[old_value] {
            ValueData::Inst { num, inst, .. } => (num, inst),
            _ => panic!("{} is not an instruction result value", old_value),
        };
        let new_value = self.make_value(ValueData::Inst {
            ty: new_type,
            num,
            inst,
        });
        let num = num as usize;
        let attached = mem::replace(
            self.results[inst]
                .get_mut(num, &mut self.value_lists)
                .expect("Replacing detached result"),
            new_value,
        );
        debug_assert_eq!(
            attached,
            old_value,
            "{} wasn't detached from {}",
            old_value,
            self.display_inst(inst)
        );
        new_value
    }

    /// Append a new instruction result value to `inst`.
    pub fn append_result(&mut self, inst: Inst, ty: Type) -> Value {
        let res = self.values.next_key();
        let num = self.results[inst].push(res, &mut self.value_lists);
        debug_assert!(num <= u16::MAX as usize, "Too many result values");
        self.make_value(ValueData::Inst {
            ty,
            inst,
            num: num as u16,
        })
    }

    /// Append a new value argument to an instruction.
    ///
    /// Panics if the instruction doesn't support arguments.
    pub fn append_inst_arg(&mut self, inst: Inst, new_arg: Value) {
        let mut branch_values = self.insts[inst]
            .take_value_list()
            .expect("the instruction doesn't have value arguments");
        branch_values.push(new_arg, &mut self.value_lists);
        self.insts[inst].put_value_list(branch_values)
    }

    /// Get the first result of an instruction.
    ///
    /// This function panics if the instruction doesn't have any result.
    pub fn first_result(&self, inst: Inst) -> Value {
        self.results[inst]
            .first(&self.value_lists)
            .expect("Instruction has no results")
    }

    /// Test if `inst` has any result values currently.
    pub fn has_results(&self, inst: Inst) -> bool {
        !self.results[inst].is_empty()
    }

    /// Return all the results of an instruction.
    pub fn inst_results(&self, inst: Inst) -> &[Value] {
        self.results[inst].as_slice(&self.value_lists)
    }

    /// Return all the results of an instruction as ValueList.
    pub fn inst_results_list(&self, inst: Inst) -> ValueList {
        self.results[inst]
    }

    /// Get the call signature of a direct or indirect call instruction.
    /// Returns `None` if `inst` is not a call instruction.
    pub fn call_signature(&self, inst: Inst) -> Option<SigRef> {
        match self.insts[inst].analyze_call(&self.value_lists) {
            CallInfo::NotACall => None,
            CallInfo::Direct(f, _) => Some(self.ext_funcs[f].signature),
            CallInfo::Indirect(s, _) => Some(s),
        }
    }

    /// Check if `inst` is a branch.
    pub fn analyze_branch(&self, inst: Inst) -> BranchInfo {
        self.insts[inst].analyze_branch(&self.value_lists)
    }

    /// Compute the type of an instruction result from opcode constraints and call signatures.
    ///
    /// This computes the same sequence of result types that `make_inst_results()` above would
    /// assign to the created result values, but it does not depend on `make_inst_results()` being
    /// called first.
    ///
    /// Returns `None` if asked about a result index that is too large.
    pub fn compute_result_type(
        &self,
        inst: Inst,
        result_idx: usize,
        ctrl_typevar: Type,
    ) -> Option<Type> {
        let constraints = self.insts[inst].opcode().constraints();
        let num_fixed_results = constraints.num_fixed_results();

        if result_idx < num_fixed_results {
            return Some(constraints.result_type(result_idx, ctrl_typevar));
        }

        // Not a fixed result, try to extract a return type from the call signature.
        self.call_signature(inst).and_then(|sigref| {
            self.signatures[sigref]
                .returns
                .get(result_idx - num_fixed_results)
                .map(|&arg| arg.value_type)
        })
    }

    /// Get the controlling type variable, or `INVALID` if `inst` isn't polymorphic.
    pub fn ctrl_typevar(&self, inst: Inst) -> Type {
        let constraints = self[inst].opcode().constraints();

        if !constraints.is_polymorphic() {
            types::INVALID
        } else if constraints.requires_typevar_operand() {
            // Not all instruction formats have a designated operand, but in that case
            // `requires_typevar_operand()` should never be true.
            self.value_type(
                self[inst]
                    .typevar_operand(&self.value_lists)
                    .expect("Instruction format doesn't have a designated operand, bad opcode."),
            )
        } else {
            self.value_type(self.first_result(inst))
        }
    }
}

/// Allow immutable access to instructions via indexing.
impl Index<Inst> for DataFlowGraph {
    type Output = InstructionData;

    fn index(&self, inst: Inst) -> &InstructionData {
        &self.insts[inst]
    }
}

/// Allow mutable access to instructions via indexing.
impl IndexMut<Inst> for DataFlowGraph {
    fn index_mut(&mut self, inst: Inst) -> &mut InstructionData {
        &mut self.insts[inst]
    }
}

/// basic blocks.
impl DataFlowGraph {
    /// Create a new basic block.
    pub fn make_block(&mut self) -> Block {
        self.blocks.push(BlockData::new())
    }

    /// Get the number of parameters on `block`.
    pub fn num_block_params(&self, block: Block) -> usize {
        self.blocks[block].params.len(&self.value_lists)
    }

    /// Get the parameters on `block`.
    pub fn block_params(&self, block: Block) -> &[Value] {
        self.blocks[block].params.as_slice(&self.value_lists)
    }

    /// Get the types of the parameters on `block`.
    pub fn block_param_types(&self, block: Block) -> Vec<Type> {
        self.block_params(block)
            .iter()
            .map(|&v| self.value_type(v))
            .collect()
    }

    /// Append a parameter with type `ty` to `block`.
    pub fn append_block_param(&mut self, block: Block, ty: Type) -> Value {
        let param = self.values.next_key();
        let num = self.blocks[block].params.push(param, &mut self.value_lists);
        debug_assert!(num <= u16::MAX as usize, "Too many parameters on block");
        self.make_value(ValueData::Param {
            ty,
            num: num as u16,
            block,
        })
    }

    /// Removes `val` from `block`'s parameters by swapping it with the last parameter on `block`.
    /// Returns the position of `val` before removal.
    ///
    /// *Important*: to ensure O(1) deletion, this method swaps the removed parameter with the
    /// last `block` parameter. This can disrupt all the branch instructions jumping to this
    /// `block` for which you have to change the branch argument order if necessary.
    ///
    /// Panics if `val` is not a block parameter.
    pub fn swap_remove_block_param(&mut self, val: Value) -> usize {
        let (block, num) = if let ValueData::Param { num, block, .. } = self.values[val] {
            (block, num)
        } else {
            panic!("{} must be a block parameter", val);
        };
        self.blocks[block]
            .params
            .swap_remove(num as usize, &mut self.value_lists);
        if let Some(last_arg_val) = self.blocks[block]
            .params
            .get(num as usize, &self.value_lists)
        {
            // We update the position of the old last arg.
            if let ValueData::Param {
                num: ref mut old_num,
                ..
            } = self.values[last_arg_val]
            {
                *old_num = num;
            } else {
                panic!("{} should be a Block parameter", last_arg_val);
            }
        }
        num as usize
    }

    /// Removes `val` from `block`'s parameters by a standard linear time list removal which
    /// preserves ordering. Also updates the values' data.
    pub fn remove_block_param(&mut self, val: Value) {
        let (block, num) = if let ValueData::Param { num, block, .. } = self.values[val] {
            (block, num)
        } else {
            panic!("{} must be a block parameter", val);
        };
        self.blocks[block]
            .params
            .remove(num as usize, &mut self.value_lists);
        for index in num..(self.num_block_params(block) as u16) {
            match self.values[self.blocks[block]
                .params
                .get(index as usize, &self.value_lists)
                .unwrap()]
            {
                ValueData::Param { ref mut num, .. } => {
                    *num -= 1;
                }
                _ => panic!(
                    "{} must be a block parameter",
                    self.blocks[block]
                        .params
                        .get(index as usize, &self.value_lists)
                        .unwrap()
                ),
            }
        }
    }

    /// Append an existing value to `block`'s parameters.
    ///
    /// The appended value can't already be attached to something else.
    ///
    /// In almost all cases, you should be using `append_block_param()` instead of this method.
    pub fn attach_block_param(&mut self, block: Block, param: Value) {
        debug_assert!(!self.value_is_attached(param));
        let num = self.blocks[block].params.push(param, &mut self.value_lists);
        debug_assert!(num <= u16::MAX as usize, "Too many parameters on block");
        let ty = self.value_type(param);
        self.values[param] = ValueData::Param {
            ty,
            num: num as u16,
            block,
        };
    }

    /// Replace a block parameter with a new value of type `ty`.
    ///
    /// The `old_value` must be an attached block parameter. It is removed from its place in the list
    /// of parameters and replaced by a new value of type `new_type`. The new value gets the same
    /// position in the list, and other parameters are not disturbed.
    ///
    /// The old value is left detached, so it should probably be changed into something else.
    ///
    /// Returns the new value.
    pub fn replace_block_param(&mut self, old_value: Value, new_type: Type) -> Value {
        // Create new value identical to the old one except for the type.
        let (block, num) = if let ValueData::Param { num, block, .. } = self.values[old_value] {
            (block, num)
        } else {
            panic!("{} must be a block parameter", old_value);
        };
        let new_arg = self.make_value(ValueData::Param {
            ty: new_type,
            num,
            block,
        });

        self.blocks[block]
            .params
            .as_mut_slice(&mut self.value_lists)[num as usize] = new_arg;
        new_arg
    }

    /// Detach all the parameters from `block` and return them as a `ValueList`.
    ///
    /// This is a quite low-level operation. Sensible things to do with the detached block parameters
    /// is to put them back on the same block with `attach_block_param()` or change them into aliases
    /// with `change_to_alias()`.
    pub fn detach_block_params(&mut self, block: Block) -> ValueList {
        self.blocks[block].params.take()
    }
}

/// Contents of a basic block.
///
/// Parameters on a basic block are values that dominate everything in the block. All
/// branches to this block must provide matching arguments, and the arguments to the entry block must
/// match the function arguments.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
struct BlockData {
    /// List of parameters to this block.
    params: ValueList,
}

impl BlockData {
    fn new() -> Self {
        Self {
            params: ValueList::new(),
        }
    }
}

/// Object that can display an instruction.
pub struct DisplayInst<'a>(&'a DataFlowGraph, Inst);

impl<'a> fmt::Display for DisplayInst<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dfg = self.0;
        let inst = self.1;

        if let Some((first, rest)) = dfg.inst_results(inst).split_first() {
            write!(f, "{}", first)?;
            for v in rest {
                write!(f, ", {}", v)?;
            }
            write!(f, " = ")?;
        }

        let typevar = dfg.ctrl_typevar(inst);
        if typevar.is_invalid() {
            write!(f, "{}", dfg[inst].opcode())?;
        } else {
            write!(f, "{}.{}", dfg[inst].opcode(), typevar)?;
        }
        write_operands(f, dfg, inst)
    }
}

/// Parser routines. These routines should not be used outside the parser.
impl DataFlowGraph {
    /// Set the type of a value. This is only for use in the parser, which needs
    /// to create invalid values for index padding which may be reassigned later.
    #[cold]
    fn set_value_type_for_parser(&mut self, v: Value, t: Type) {
        assert_eq!(
            self.value_type(v),
            types::INVALID,
            "this function is only for assigning types to previously invalid values"
        );
        match self.values[v] {
            ValueData::Inst { ref mut ty, .. }
            | ValueData::Param { ref mut ty, .. }
            | ValueData::Alias { ref mut ty, .. } => *ty = t,
        }
    }

    /// Create result values for `inst`, reusing the provided detached values.
    /// This is similar to `make_inst_results_reusing` except it's only for use
    /// in the parser, which needs to reuse previously invalid values.
    #[cold]
    pub fn make_inst_results_for_parser(
        &mut self,
        inst: Inst,
        ctrl_typevar: Type,
        reuse: &[Value],
    ) -> usize {
        // Get the call signature if this is a function call.
        if let Some(sig) = self.call_signature(inst) {
            assert_eq!(
                self.insts[inst].opcode().constraints().num_fixed_results(),
                0
            );
            for res_idx in 0..self.signatures[sig].returns.len() {
                let ty = self.signatures[sig].returns[res_idx].value_type;
                if let Some(v) = reuse.get(res_idx) {
                    self.set_value_type_for_parser(*v, ty);
                }
            }
        } else {
            let constraints = self.insts[inst].opcode().constraints();
            for res_idx in 0..constraints.num_fixed_results() {
                let ty = constraints.result_type(res_idx, ctrl_typevar);
                if let Some(v) = reuse.get(res_idx) {
                    self.set_value_type_for_parser(*v, ty);
                }
            }
        }

        self.make_inst_results_reusing(inst, ctrl_typevar, reuse.iter().map(|x| Some(*x)))
    }

    /// Similar to `append_block_param`, append a parameter with type `ty` to
    /// `block`, but using value `val`. This is only for use by the parser to
    /// create parameters with specific values.
    #[cold]
    pub fn append_block_param_for_parser(&mut self, block: Block, ty: Type, val: Value) {
        let num = self.blocks[block].params.push(val, &mut self.value_lists);
        assert!(num <= u16::MAX as usize, "Too many parameters on block");
        self.values[val] = ValueData::Param {
            ty,
            num: num as u16,
            block,
        };
    }

    /// Create a new value alias. This is only for use by the parser to create
    /// aliases with specific values, and the printer for testing.
    #[cold]
    pub fn make_value_alias_for_serialization(&mut self, src: Value, dest: Value) {
        assert_ne!(src, Value::reserved_value());
        assert_ne!(dest, Value::reserved_value());

        let ty = if self.values.is_valid(src) {
            self.value_type(src)
        } else {
            // As a special case, if we can't resolve the aliasee yet, use INVALID
            // temporarily. It will be resolved later in parsing.
            types::INVALID
        };
        let data = ValueData::Alias { ty, original: src };
        self.values[dest] = data;
    }

    /// If `v` is already defined as an alias, return its destination value.
    /// Otherwise return None. This allows the parser to coalesce identical
    /// alias definitions, and the printer to identify an alias's immediate target.
    #[cold]
    pub fn value_alias_dest_for_serialization(&self, v: Value) -> Option<Value> {
        if let ValueData::Alias { original, .. } = self.values[v] {
            Some(original)
        } else {
            None
        }
    }

    /// Compute the type of an alias. This is only for use in the parser.
    /// Returns false if an alias cycle was encountered.
    #[cold]
    pub fn set_alias_type_for_parser(&mut self, v: Value) -> bool {
        if let Some(resolved) = maybe_resolve_aliases(&self.values, v) {
            let old_ty = self.value_type(v);
            let new_ty = self.value_type(resolved);
            if old_ty == types::INVALID {
                self.set_value_type_for_parser(v, new_ty);
            } else {
                assert_eq!(old_ty, new_ty);
            }
            true
        } else {
            false
        }
    }

    /// Create an invalid value, to pad the index space. This is only for use by
    /// the parser to pad out the value index space.
    #[cold]
    pub fn make_invalid_value_for_parser(&mut self) {
        let data = ValueData::Alias {
            ty: types::INVALID,
            original: Value::reserved_value(),
        };
        self.make_value(data);
    }

    /// Check if a value reference is valid, while being aware of aliases which
    /// may be unresolved while parsing.
    #[cold]
    pub fn value_is_valid_for_parser(&self, v: Value) -> bool {
        if !self.value_is_valid(v) {
            return false;
        }
        if let ValueData::Alias { ty, .. } = self.values[v] {
            ty != types::INVALID
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::{Cursor, FuncCursor};
    use crate::ir::types;
    use crate::ir::{Function, InstructionData, Opcode, TrapCode};
    use alloc::string::ToString;

    #[test]
    fn make_inst() {
        let mut dfg = DataFlowGraph::new();

        let idata = InstructionData::UnaryImm {
            opcode: Opcode::Iconst,
            imm: 0.into(),
        };
        let inst = dfg.make_inst(idata);

        dfg.make_inst_results(inst, types::I32);
        assert_eq!(inst.to_string(), "inst0");
        assert_eq!(dfg.display_inst(inst).to_string(), "v0 = iconst.i32 0");

        // Immutable reference resolution.
        {
            let immdfg = &dfg;
            let ins = &immdfg[inst];
            assert_eq!(ins.opcode(), Opcode::Iconst);
        }

        // Results.
        let val = dfg.first_result(inst);
        assert_eq!(dfg.inst_results(inst), &[val]);

        assert_eq!(dfg.value_def(val), ValueDef::Result(inst, 0));
        assert_eq!(dfg.value_type(val), types::I32);

        // Replacing results.
        assert!(dfg.value_is_attached(val));
        let v2 = dfg.replace_result(val, types::F64);
        assert!(!dfg.value_is_attached(val));
        assert!(dfg.value_is_attached(v2));
        assert_eq!(dfg.inst_results(inst), &[v2]);
        assert_eq!(dfg.value_def(v2), ValueDef::Result(inst, 0));
        assert_eq!(dfg.value_type(v2), types::F64);
    }

    #[test]
    fn no_results() {
        let mut dfg = DataFlowGraph::new();

        let idata = InstructionData::Trap {
            opcode: Opcode::Trap,
            code: TrapCode::User(0),
        };
        let inst = dfg.make_inst(idata);
        assert_eq!(dfg.display_inst(inst).to_string(), "trap user0");

        // Result slice should be empty.
        assert_eq!(dfg.inst_results(inst), &[]);
    }

    #[test]
    fn block() {
        let mut dfg = DataFlowGraph::new();

        let block = dfg.make_block();
        assert_eq!(block.to_string(), "block0");
        assert_eq!(dfg.num_block_params(block), 0);
        assert_eq!(dfg.block_params(block), &[]);
        assert!(dfg.detach_block_params(block).is_empty());
        assert_eq!(dfg.num_block_params(block), 0);
        assert_eq!(dfg.block_params(block), &[]);

        let arg1 = dfg.append_block_param(block, types::F32);
        assert_eq!(arg1.to_string(), "v0");
        assert_eq!(dfg.num_block_params(block), 1);
        assert_eq!(dfg.block_params(block), &[arg1]);

        let arg2 = dfg.append_block_param(block, types::I16);
        assert_eq!(arg2.to_string(), "v1");
        assert_eq!(dfg.num_block_params(block), 2);
        assert_eq!(dfg.block_params(block), &[arg1, arg2]);

        assert_eq!(dfg.value_def(arg1), ValueDef::Param(block, 0));
        assert_eq!(dfg.value_def(arg2), ValueDef::Param(block, 1));
        assert_eq!(dfg.value_type(arg1), types::F32);
        assert_eq!(dfg.value_type(arg2), types::I16);

        // Swap the two block parameters.
        let vlist = dfg.detach_block_params(block);
        assert_eq!(dfg.num_block_params(block), 0);
        assert_eq!(dfg.block_params(block), &[]);
        assert_eq!(vlist.as_slice(&dfg.value_lists), &[arg1, arg2]);
        dfg.attach_block_param(block, arg2);
        let arg3 = dfg.append_block_param(block, types::I32);
        dfg.attach_block_param(block, arg1);
        assert_eq!(dfg.block_params(block), &[arg2, arg3, arg1]);
    }

    #[test]
    fn replace_block_params() {
        let mut dfg = DataFlowGraph::new();

        let block = dfg.make_block();
        let arg1 = dfg.append_block_param(block, types::F32);

        let new1 = dfg.replace_block_param(arg1, types::I64);
        assert_eq!(dfg.value_type(arg1), types::F32);
        assert_eq!(dfg.value_type(new1), types::I64);
        assert_eq!(dfg.block_params(block), &[new1]);

        dfg.attach_block_param(block, arg1);
        assert_eq!(dfg.block_params(block), &[new1, arg1]);

        let new2 = dfg.replace_block_param(arg1, types::I8);
        assert_eq!(dfg.value_type(arg1), types::F32);
        assert_eq!(dfg.value_type(new2), types::I8);
        assert_eq!(dfg.block_params(block), &[new1, new2]);

        dfg.attach_block_param(block, arg1);
        assert_eq!(dfg.block_params(block), &[new1, new2, arg1]);

        let new3 = dfg.replace_block_param(new2, types::I16);
        assert_eq!(dfg.value_type(new1), types::I64);
        assert_eq!(dfg.value_type(new2), types::I8);
        assert_eq!(dfg.value_type(new3), types::I16);
        assert_eq!(dfg.block_params(block), &[new1, new3, arg1]);
    }

    #[test]
    fn swap_remove_block_params() {
        let mut dfg = DataFlowGraph::new();

        let block = dfg.make_block();
        let arg1 = dfg.append_block_param(block, types::F32);
        let arg2 = dfg.append_block_param(block, types::F32);
        let arg3 = dfg.append_block_param(block, types::F32);
        assert_eq!(dfg.block_params(block), &[arg1, arg2, arg3]);

        dfg.swap_remove_block_param(arg1);
        assert_eq!(dfg.value_is_attached(arg1), false);
        assert_eq!(dfg.value_is_attached(arg2), true);
        assert_eq!(dfg.value_is_attached(arg3), true);
        assert_eq!(dfg.block_params(block), &[arg3, arg2]);
        dfg.swap_remove_block_param(arg2);
        assert_eq!(dfg.value_is_attached(arg2), false);
        assert_eq!(dfg.value_is_attached(arg3), true);
        assert_eq!(dfg.block_params(block), &[arg3]);
        dfg.swap_remove_block_param(arg3);
        assert_eq!(dfg.value_is_attached(arg3), false);
        assert_eq!(dfg.block_params(block), &[]);
    }

    #[test]
    fn aliases() {
        use crate::ir::InstBuilder;

        let mut func = Function::new();
        let block0 = func.dfg.make_block();
        let mut pos = FuncCursor::new(&mut func);
        pos.insert_block(block0);

        // Build a little test program.
        let v1 = pos.ins().iconst(types::I32, 42);

        // Make sure we can resolve value aliases even when values is empty.
        assert_eq!(pos.func.dfg.resolve_aliases(v1), v1);

        let arg0 = pos.func.dfg.append_block_param(block0, types::I32);
        let (s, c) = pos.ins().iadd_ifcout(v1, arg0);
        let iadd = match pos.func.dfg.value_def(s) {
            ValueDef::Result(i, 0) => i,
            _ => panic!(),
        };

        // Remove `c` from the result list.
        pos.func.dfg.clear_results(iadd);
        pos.func.dfg.attach_result(iadd, s);

        // Replace `iadd_ifcout` with a normal `iadd` and an `ifcmp`.
        pos.func.dfg.replace(iadd).iadd(v1, arg0);
        let c2 = pos.ins().ifcmp(s, v1);
        pos.func.dfg.change_to_alias(c, c2);

        assert_eq!(pos.func.dfg.resolve_aliases(c2), c2);
        assert_eq!(pos.func.dfg.resolve_aliases(c), c2);

        // Make a copy of the alias.
        let c3 = pos.ins().copy(c);
        // This does not see through copies.
        assert_eq!(pos.func.dfg.resolve_aliases(c3), c3);
    }
}
