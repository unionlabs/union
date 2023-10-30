//! A verifier for ensuring that functions are well formed.
//! It verifies:
//!
//! block integrity
//!
//! - All instructions reached from the `block_insts` iterator must belong to
//!   the block as reported by `inst_block()`.
//! - Every block must end in a terminator instruction, and no other instruction
//!   can be a terminator.
//! - Every value in the `block_params` iterator belongs to the block as reported by `value_block`.
//!
//! Instruction integrity
//!
//! - The instruction format must match the opcode.
//! - All result values must be created for multi-valued instructions.
//! - All referenced entities must exist. (Values, blocks, stack slots, ...)
//! - Instructions must not reference (eg. branch to) the entry block.
//!
//! SSA form
//!
//! - Values must be defined by an instruction that exists and that is inserted in
//!   a block, or be an argument of an existing block.
//! - Values used by an instruction must dominate the instruction.
//!
//! Control flow graph and dominator tree integrity:
//!
//! - All predecessors in the CFG must be branches to the block.
//! - All branches to a block must be present in the CFG.
//! - A recomputed dominator tree is identical to the existing one.
//!
//! Type checking
//!
//! - Compare input and output values against the opcode's type constraints.
//!   For polymorphic opcodes, determine the controlling type variable first.
//! - Branches and jumps must pass arguments to destination blocks that match the
//!   expected types exactly. The number of arguments must match.
//! - All blocks in a jump table must take no arguments.
//! - Function calls are type checked against their signature.
//! - The entry block must take arguments that match the signature of the current
//!   function.
//! - All return instructions must have return value operands matching the current
//!   function signature.
//!
//! Global values
//!
//! - Detect cycles in global values.
//! - Detect use of 'vmctx' global value when no corresponding parameter is defined.
//!
//! TODO:
//! Ad hoc checking
//!
//! - Stack slot loads and stores must be in-bounds.
//! - Immediate constraints for certain opcodes, like `udiv_imm v3, 0`.
//! - `Insertlane` and `extractlane` instructions have immediate lane numbers that must be in
//!   range for their polymorphic type.
//! - Swizzle and shuffle instructions take a variable number of lane arguments. The number
//!   of arguments must match the destination type, and the lane indexes must be in range.

use self::flags::verify_flags;
use crate::dbg::DisplayList;
use crate::dominator_tree::DominatorTree;
use crate::entity::SparseSet;
use crate::flowgraph::{BlockPredecessor, ControlFlowGraph};
use crate::ir;
use crate::ir::entities::AnyEntity;
use crate::ir::instructions::{BranchInfo, CallInfo, InstructionFormat, ResolvedConstraint};
use crate::ir::{
    types, ArgumentPurpose, Block, Constant, FuncRef, Function, GlobalValue, Inst, JumpTable,
    Opcode, SigRef, StackSlot, Type, Value, ValueDef, ValueList,
};
use crate::isa::TargetIsa;
use crate::iterators::IteratorExtras;
use crate::print_errors::pretty_verifier_error;
use crate::settings::FlagsOrIsa;
use crate::timing;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::fmt::{self, Display, Formatter};

mod flags;

/// A verifier error.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VerifierError {
    /// The entity causing the verifier error.
    pub location: AnyEntity,
    /// Optionally provide some context for the given location; e.g., for `inst42` provide
    /// `Some("v3 = iconst.i32 0")` for more comprehensible errors.
    pub context: Option<String>,
    /// The error message.
    pub message: String,
}

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
impl std::error::Error for VerifierError {}

impl Display for VerifierError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self.context {
            None => write!(f, "{}: {}", self.location, self.message),
            Some(context) => write!(f, "{} ({}): {}", self.location, context, self.message),
        }
    }
}

/// Convenience converter for making error-reporting less verbose.
///
/// Converts a tuple of `(location, context, message)` to a `VerifierError`.
/// ```
/// use cranelift_codegen::verifier::VerifierErrors;
/// use cranelift_codegen::ir::Inst;
/// let mut errors = VerifierErrors::new();
/// errors.report((Inst::from_u32(42), "v3 = iadd v1, v2", "iadd cannot be used with values of this type"));
/// // note the double parenthenses to use this syntax
/// ```
impl<L, C, M> From<(L, C, M)> for VerifierError
where
    L: Into<AnyEntity>,
    C: Into<String>,
    M: Into<String>,
{
    fn from(items: (L, C, M)) -> Self {
        let (location, context, message) = items;
        Self {
            location: location.into(),
            context: Some(context.into()),
            message: message.into(),
        }
    }
}

/// Convenience converter for making error-reporting less verbose.
///
/// Same as above but without `context`.
impl<L, M> From<(L, M)> for VerifierError
where
    L: Into<AnyEntity>,
    M: Into<String>,
{
    fn from(items: (L, M)) -> Self {
        let (location, message) = items;
        Self {
            location: location.into(),
            context: None,
            message: message.into(),
        }
    }
}

/// Result of a step in the verification process.
///
/// Functions that return `VerifierStepResult<()>` should also take a
/// mutable reference to `VerifierErrors` as argument in order to report
/// errors.
///
/// Here, `Ok` represents a step that **did not lead to a fatal error**,
/// meaning that the verification process may continue. However, other (non-fatal)
/// errors might have been reported through the previously mentioned `VerifierErrors`
/// argument.
pub type VerifierStepResult<T> = Result<T, ()>;

/// Result of a verification operation.
///
/// Unlike `VerifierStepResult<()>` which may be `Ok` while still having reported
/// errors, this type always returns `Err` if an error (fatal or not) was reported.
pub type VerifierResult<T> = Result<T, VerifierErrors>;

/// List of verifier errors.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct VerifierErrors(pub Vec<VerifierError>);

// This is manually implementing Error and Display instead of using thiserror to reduce the amount
// of dependencies used by Cranelift.
impl std::error::Error for VerifierErrors {}

impl VerifierErrors {
    /// Return a new `VerifierErrors` struct.
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Return whether no errors were reported.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return whether one or more errors were reported.
    #[inline]
    pub fn has_error(&self) -> bool {
        !self.0.is_empty()
    }

    /// Return a `VerifierStepResult` that is fatal if at least one error was reported,
    /// and non-fatal otherwise.
    #[inline]
    pub fn as_result(&self) -> VerifierStepResult<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Report an error, adding it to the list of errors.
    pub fn report(&mut self, error: impl Into<VerifierError>) {
        self.0.push(error.into());
    }

    /// Report a fatal error and return `Err`.
    pub fn fatal(&mut self, error: impl Into<VerifierError>) -> VerifierStepResult<()> {
        self.report(error);
        Err(())
    }

    /// Report a non-fatal error and return `Ok`.
    pub fn nonfatal(&mut self, error: impl Into<VerifierError>) -> VerifierStepResult<()> {
        self.report(error);
        Ok(())
    }
}

impl From<Vec<VerifierError>> for VerifierErrors {
    fn from(v: Vec<VerifierError>) -> Self {
        Self(v)
    }
}

impl Into<Vec<VerifierError>> for VerifierErrors {
    fn into(self) -> Vec<VerifierError> {
        self.0
    }
}

impl Into<VerifierResult<()>> for VerifierErrors {
    fn into(self) -> VerifierResult<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl Display for VerifierErrors {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for err in &self.0 {
            writeln!(f, "- {}", err)?;
        }
        Ok(())
    }
}

/// Verify `func`.
pub fn verify_function<'a, FOI: Into<FlagsOrIsa<'a>>>(
    func: &Function,
    fisa: FOI,
) -> VerifierResult<()> {
    let _tt = timing::verifier();
    let mut errors = VerifierErrors::default();
    let verifier = Verifier::new(func, fisa.into());
    let result = verifier.run(&mut errors);
    if errors.is_empty() {
        result.unwrap();
        Ok(())
    } else {
        Err(errors)
    }
}

/// Verify `func` after checking the integrity of associated context data structures `cfg` and
/// `domtree`.
pub fn verify_context<'a, FOI: Into<FlagsOrIsa<'a>>>(
    func: &Function,
    cfg: &ControlFlowGraph,
    domtree: &DominatorTree,
    fisa: FOI,
    errors: &mut VerifierErrors,
) -> VerifierStepResult<()> {
    let _tt = timing::verifier();
    let verifier = Verifier::new(func, fisa.into());
    if cfg.is_valid() {
        verifier.cfg_integrity(cfg, errors)?;
    }
    if domtree.is_valid() {
        verifier.domtree_integrity(domtree, errors)?;
    }
    verifier.run(errors)
}

struct Verifier<'a> {
    func: &'a Function,
    expected_cfg: ControlFlowGraph,
    expected_domtree: DominatorTree,
    isa: Option<&'a dyn TargetIsa>,
}

impl<'a> Verifier<'a> {
    pub fn new(func: &'a Function, fisa: FlagsOrIsa<'a>) -> Self {
        let expected_cfg = ControlFlowGraph::with_function(func);
        let expected_domtree = DominatorTree::with_function(func, &expected_cfg);
        Self {
            func,
            expected_cfg,
            expected_domtree,
            isa: fisa.isa,
        }
    }

    /// Determine a contextual error string for an instruction.
    #[inline]
    fn context(&self, inst: Inst) -> String {
        self.func.dfg.display_inst(inst).to_string()
    }

    // Check for:
    //  - cycles in the global value declarations.
    //  - use of 'vmctx' when no special parameter declares it.
    fn verify_global_values(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        let mut cycle_seen = false;
        let mut seen = SparseSet::new();

        'gvs: for gv in self.func.global_values.keys() {
            seen.clear();
            seen.insert(gv);

            let mut cur = gv;
            loop {
                match self.func.global_values[cur] {
                    ir::GlobalValueData::Load { base, .. }
                    | ir::GlobalValueData::IAddImm { base, .. } => {
                        if seen.insert(base).is_some() {
                            if !cycle_seen {
                                errors.report((
                                    gv,
                                    format!("global value cycle: {}", DisplayList(seen.as_slice())),
                                ));
                                // ensures we don't report the cycle multiple times
                                cycle_seen = true;
                            }
                            continue 'gvs;
                        }

                        cur = base;
                    }
                    _ => break,
                }
            }

            match self.func.global_values[gv] {
                ir::GlobalValueData::VMContext { .. } => {
                    if self
                        .func
                        .special_param(ir::ArgumentPurpose::VMContext)
                        .is_none()
                    {
                        errors.report((gv, format!("undeclared vmctx reference {}", gv)));
                    }
                }
                ir::GlobalValueData::IAddImm {
                    base, global_type, ..
                } => {
                    if !global_type.is_int() {
                        errors.report((
                            gv,
                            format!("iadd_imm global value with non-int type {}", global_type),
                        ));
                    } else if let Some(isa) = self.isa {
                        let base_type = self.func.global_values[base].global_type(isa);
                        if global_type != base_type {
                            errors.report((
                                gv,
                                format!(
                                    "iadd_imm type {} differs from operand type {}",
                                    global_type, base_type
                                ),
                            ));
                        }
                    }
                }
                ir::GlobalValueData::Load { base, .. } => {
                    if let Some(isa) = self.isa {
                        let base_type = self.func.global_values[base].global_type(isa);
                        let pointer_type = isa.pointer_type();
                        if base_type != pointer_type {
                            errors.report((
                                gv,
                                format!(
                                    "base {} has type {}, which is not the pointer type {}",
                                    base, base_type, pointer_type
                                ),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        // Invalid global values shouldn't stop us from verifying the rest of the function
        Ok(())
    }

    fn verify_heaps(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        if let Some(isa) = self.isa {
            for (heap, heap_data) in &self.func.heaps {
                let base = heap_data.base;
                if !self.func.global_values.is_valid(base) {
                    return errors.nonfatal((heap, format!("invalid base global value {}", base)));
                }

                let pointer_type = isa.pointer_type();
                let base_type = self.func.global_values[base].global_type(isa);
                if base_type != pointer_type {
                    errors.report((
                        heap,
                        format!(
                            "heap base has type {}, which is not the pointer type {}",
                            base_type, pointer_type
                        ),
                    ));
                }

                if let ir::HeapStyle::Dynamic { bound_gv, .. } = heap_data.style {
                    if !self.func.global_values.is_valid(bound_gv) {
                        return errors
                            .nonfatal((heap, format!("invalid bound global value {}", bound_gv)));
                    }

                    let bound_type = self.func.global_values[bound_gv].global_type(isa);
                    if pointer_type != bound_type {
                        errors.report((
                            heap,
                            format!(
                                "heap pointer type {} differs from the type of its bound, {}",
                                pointer_type, bound_type
                            ),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn verify_tables(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        if let Some(isa) = self.isa {
            for (table, table_data) in &self.func.tables {
                let base = table_data.base_gv;
                if !self.func.global_values.is_valid(base) {
                    return errors.nonfatal((table, format!("invalid base global value {}", base)));
                }

                let pointer_type = isa.pointer_type();
                let base_type = self.func.global_values[base].global_type(isa);
                if base_type != pointer_type {
                    errors.report((
                        table,
                        format!(
                            "table base has type {}, which is not the pointer type {}",
                            base_type, pointer_type
                        ),
                    ));
                }

                let bound_gv = table_data.bound_gv;
                if !self.func.global_values.is_valid(bound_gv) {
                    return errors
                        .nonfatal((table, format!("invalid bound global value {}", bound_gv)));
                }

                let index_type = table_data.index_type;
                let bound_type = self.func.global_values[bound_gv].global_type(isa);
                if index_type != bound_type {
                    errors.report((
                        table,
                        format!(
                            "table index type {} differs from the type of its bound, {}",
                            index_type, bound_type
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    fn verify_jump_tables(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        for (jt, jt_data) in &self.func.jump_tables {
            for &block in jt_data.iter() {
                self.verify_block(jt, block, errors)?;
            }
        }
        Ok(())
    }

    /// Check that the given block can be encoded as a BB, by checking that only
    /// branching instructions are ending the block.
    fn encodable_as_bb(&self, block: Block, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        match self.func.is_block_basic(block) {
            Ok(()) => Ok(()),
            Err((inst, message)) => errors.fatal((inst, self.context(inst), message)),
        }
    }

    fn block_integrity(
        &self,
        block: Block,
        inst: Inst,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let is_terminator = self.func.dfg[inst].opcode().is_terminator();
        let is_last_inst = self.func.layout.last_inst(block) == Some(inst);

        if is_terminator && !is_last_inst {
            // Terminating instructions only occur at the end of blocks.
            return errors.fatal((
                inst,
                self.context(inst),
                format!(
                    "a terminator instruction was encountered before the end of {}",
                    block
                ),
            ));
        }
        if is_last_inst && !is_terminator {
            return errors.fatal((block, "block does not end in a terminator instruction"));
        }

        // Instructions belong to the correct block.
        let inst_block = self.func.layout.inst_block(inst);
        if inst_block != Some(block) {
            return errors.fatal((
                inst,
                self.context(inst),
                format!("should belong to {} not {:?}", block, inst_block),
            ));
        }

        // Parameters belong to the correct block.
        for &arg in self.func.dfg.block_params(block) {
            match self.func.dfg.value_def(arg) {
                ValueDef::Param(arg_block, _) => {
                    if block != arg_block {
                        return errors.fatal((arg, format!("does not belong to {}", block)));
                    }
                }
                _ => {
                    return errors.fatal((arg, "expected an argument, found a result"));
                }
            }
        }

        Ok(())
    }

    fn instruction_integrity(
        &self,
        inst: Inst,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let inst_data = &self.func.dfg[inst];
        let dfg = &self.func.dfg;

        // The instruction format matches the opcode
        if inst_data.opcode().format() != InstructionFormat::from(inst_data) {
            return errors.fatal((
                inst,
                self.context(inst),
                "instruction opcode doesn't match instruction format",
            ));
        }

        let num_fixed_results = inst_data.opcode().constraints().num_fixed_results();
        // var_results is 0 if we aren't a call instruction
        let var_results = dfg
            .call_signature(inst)
            .map_or(0, |sig| dfg.signatures[sig].returns.len());
        let total_results = num_fixed_results + var_results;

        // All result values for multi-valued instructions are created
        let got_results = dfg.inst_results(inst).len();
        if got_results != total_results {
            return errors.fatal((
                inst,
                self.context(inst),
                format!(
                    "expected {} result values, found {}",
                    total_results, got_results,
                ),
            ));
        }

        self.verify_entity_references(inst, errors)
    }

    fn verify_entity_references(
        &self,
        inst: Inst,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        use crate::ir::instructions::InstructionData::*;

        for &arg in self.func.dfg.inst_args(inst) {
            self.verify_inst_arg(inst, arg, errors)?;

            // All used values must be attached to something.
            let original = self.func.dfg.resolve_aliases(arg);
            if !self.func.dfg.value_is_attached(original) {
                errors.report((
                    inst,
                    self.context(inst),
                    format!("argument {} -> {} is not attached", arg, original),
                ));
            }
        }

        for &res in self.func.dfg.inst_results(inst) {
            self.verify_inst_result(inst, res, errors)?;
        }

        match self.func.dfg[inst] {
            MultiAry { ref args, .. } => {
                self.verify_value_list(inst, args, errors)?;
            }
            Jump {
                destination,
                ref args,
                ..
            }
            | Branch {
                destination,
                ref args,
                ..
            }
            | BranchInt {
                destination,
                ref args,
                ..
            }
            | BranchFloat {
                destination,
                ref args,
                ..
            }
            | BranchIcmp {
                destination,
                ref args,
                ..
            } => {
                self.verify_block(inst, destination, errors)?;
                self.verify_value_list(inst, args, errors)?;
            }
            BranchTable {
                table, destination, ..
            } => {
                self.verify_block(inst, destination, errors)?;
                self.verify_jump_table(inst, table, errors)?;
            }
            Call {
                func_ref, ref args, ..
            } => {
                self.verify_func_ref(inst, func_ref, errors)?;
                self.verify_value_list(inst, args, errors)?;
            }
            CallIndirect {
                sig_ref, ref args, ..
            } => {
                self.verify_sig_ref(inst, sig_ref, errors)?;
                self.verify_value_list(inst, args, errors)?;
            }
            FuncAddr { func_ref, .. } => {
                self.verify_func_ref(inst, func_ref, errors)?;
            }
            StackLoad { stack_slot, .. } | StackStore { stack_slot, .. } => {
                self.verify_stack_slot(inst, stack_slot, errors)?;
            }
            UnaryGlobalValue { global_value, .. } => {
                self.verify_global_value(inst, global_value, errors)?;
            }
            HeapAddr { heap, .. } => {
                self.verify_heap(inst, heap, errors)?;
            }
            TableAddr { table, .. } => {
                self.verify_table(inst, table, errors)?;
            }
            LoadComplex { ref args, .. } => {
                self.verify_value_list(inst, args, errors)?;
            }
            StoreComplex { ref args, .. } => {
                self.verify_value_list(inst, args, errors)?;
            }

            NullAry {
                opcode: Opcode::GetPinnedReg,
            }
            | Unary {
                opcode: Opcode::SetPinnedReg,
                ..
            } => {
                if let Some(isa) = &self.isa {
                    if !isa.flags().enable_pinned_reg() {
                        return errors.fatal((
                            inst,
                            self.context(inst),
                            "GetPinnedReg/SetPinnedReg cannot be used without enable_pinned_reg",
                        ));
                    }
                } else {
                    return errors.fatal((
                        inst,
                        self.context(inst),
                        "GetPinnedReg/SetPinnedReg need an ISA!",
                    ));
                }
            }
            Unary {
                opcode: Opcode::Bitcast,
                arg,
            } => {
                self.verify_bitcast(inst, arg, errors)?;
            }
            UnaryConst {
                opcode: Opcode::Vconst,
                constant_handle,
                ..
            } => {
                self.verify_constant_size(inst, constant_handle, errors)?;
            }

            // Exhaustive list so we can't forget to add new formats
            AtomicCas { .. }
            | AtomicRmw { .. }
            | LoadNoOffset { .. }
            | StoreNoOffset { .. }
            | Unary { .. }
            | UnaryConst { .. }
            | UnaryImm { .. }
            | UnaryIeee32 { .. }
            | UnaryIeee64 { .. }
            | UnaryBool { .. }
            | Binary { .. }
            | BinaryImm8 { .. }
            | BinaryImm64 { .. }
            | Ternary { .. }
            | TernaryImm8 { .. }
            | Shuffle { .. }
            | IntCompare { .. }
            | IntCompareImm { .. }
            | IntCond { .. }
            | FloatCompare { .. }
            | FloatCond { .. }
            | IntSelect { .. }
            | Load { .. }
            | Store { .. }
            | Trap { .. }
            | CondTrap { .. }
            | IntCondTrap { .. }
            | FloatCondTrap { .. }
            | NullAry { .. } => {}
        }

        Ok(())
    }

    fn verify_block(
        &self,
        loc: impl Into<AnyEntity>,
        e: Block,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.dfg.block_is_valid(e) || !self.func.layout.is_block_inserted(e) {
            return errors.fatal((loc, format!("invalid block reference {}", e)));
        }
        if let Some(entry_block) = self.func.layout.entry_block() {
            if e == entry_block {
                return errors.fatal((loc, format!("invalid reference to entry block {}", e)));
            }
        }
        Ok(())
    }

    fn verify_sig_ref(
        &self,
        inst: Inst,
        s: SigRef,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.dfg.signatures.is_valid(s) {
            errors.fatal((
                inst,
                self.context(inst),
                format!("invalid signature reference {}", s),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_func_ref(
        &self,
        inst: Inst,
        f: FuncRef,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.dfg.ext_funcs.is_valid(f) {
            errors.nonfatal((
                inst,
                self.context(inst),
                format!("invalid function reference {}", f),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_stack_slot(
        &self,
        inst: Inst,
        ss: StackSlot,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.stack_slots.is_valid(ss) {
            errors.nonfatal((
                inst,
                self.context(inst),
                format!("invalid stack slot {}", ss),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_global_value(
        &self,
        inst: Inst,
        gv: GlobalValue,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.global_values.is_valid(gv) {
            errors.nonfatal((
                inst,
                self.context(inst),
                format!("invalid global value {}", gv),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_heap(
        &self,
        inst: Inst,
        heap: ir::Heap,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.heaps.is_valid(heap) {
            errors.nonfatal((inst, self.context(inst), format!("invalid heap {}", heap)))
        } else {
            Ok(())
        }
    }

    fn verify_table(
        &self,
        inst: Inst,
        table: ir::Table,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.tables.is_valid(table) {
            errors.nonfatal((inst, self.context(inst), format!("invalid table {}", table)))
        } else {
            Ok(())
        }
    }

    fn verify_value_list(
        &self,
        inst: Inst,
        l: &ValueList,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !l.is_valid(&self.func.dfg.value_lists) {
            errors.nonfatal((
                inst,
                self.context(inst),
                format!("invalid value list reference {:?}", l),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_jump_table(
        &self,
        inst: Inst,
        j: JumpTable,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        if !self.func.jump_tables.is_valid(j) {
            errors.nonfatal((
                inst,
                self.context(inst),
                format!("invalid jump table reference {}", j),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_value(
        &self,
        loc_inst: Inst,
        v: Value,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let dfg = &self.func.dfg;
        if !dfg.value_is_valid(v) {
            errors.nonfatal((
                loc_inst,
                self.context(loc_inst),
                format!("invalid value reference {}", v),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_inst_arg(
        &self,
        loc_inst: Inst,
        v: Value,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        self.verify_value(loc_inst, v, errors)?;

        let dfg = &self.func.dfg;
        let loc_block = self.func.layout.pp_block(loc_inst);
        let is_reachable = self.expected_domtree.is_reachable(loc_block);

        // SSA form
        match dfg.value_def(v) {
            ValueDef::Result(def_inst, _) => {
                // Value is defined by an instruction that exists.
                if !dfg.inst_is_valid(def_inst) {
                    return errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("{} is defined by invalid instruction {}", v, def_inst),
                    ));
                }
                // Defining instruction is inserted in a block.
                if self.func.layout.inst_block(def_inst) == None {
                    return errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("{} is defined by {} which has no block", v, def_inst),
                    ));
                }
                // Defining instruction dominates the instruction that uses the value.
                if is_reachable {
                    if !self
                        .expected_domtree
                        .dominates(def_inst, loc_inst, &self.func.layout)
                    {
                        return errors.fatal((
                            loc_inst,
                            self.context(loc_inst),
                            format!("uses value {} from non-dominating {}", v, def_inst),
                        ));
                    }
                    if def_inst == loc_inst {
                        return errors.fatal((
                            loc_inst,
                            self.context(loc_inst),
                            format!("uses value {} from itself", v),
                        ));
                    }
                }
            }
            ValueDef::Param(block, _) => {
                // Value is defined by an existing block.
                if !dfg.block_is_valid(block) {
                    return errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("{} is defined by invalid block {}", v, block),
                    ));
                }
                // Defining block is inserted in the layout
                if !self.func.layout.is_block_inserted(block) {
                    return errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("{} is defined by {} which is not in the layout", v, block),
                    ));
                }
                // The defining block dominates the instruction using this value.
                if is_reachable
                    && !self
                        .expected_domtree
                        .dominates(block, loc_inst, &self.func.layout)
                {
                    return errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("uses value arg from non-dominating {}", block),
                    ));
                }
            }
        }
        Ok(())
    }

    fn verify_inst_result(
        &self,
        loc_inst: Inst,
        v: Value,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        self.verify_value(loc_inst, v, errors)?;

        match self.func.dfg.value_def(v) {
            ValueDef::Result(def_inst, _) => {
                if def_inst != loc_inst {
                    errors.fatal((
                        loc_inst,
                        self.context(loc_inst),
                        format!("instruction result {} is not defined by the instruction", v),
                    ))
                } else {
                    Ok(())
                }
            }
            ValueDef::Param(_, _) => errors.fatal((
                loc_inst,
                self.context(loc_inst),
                format!("instruction result {} is not defined by the instruction", v),
            )),
        }
    }

    fn verify_bitcast(
        &self,
        inst: Inst,
        arg: Value,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let typ = self.func.dfg.ctrl_typevar(inst);
        let value_type = self.func.dfg.value_type(arg);

        if typ.lane_bits() < value_type.lane_bits() {
            errors.fatal((
                inst,
                format!(
                    "The bitcast argument {} doesn't fit in a type of {} bits",
                    arg,
                    typ.lane_bits()
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn verify_constant_size(
        &self,
        inst: Inst,
        constant: Constant,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let type_size = self.func.dfg.ctrl_typevar(inst).bytes() as usize;
        let constant_size = self.func.dfg.constants.get(constant).len();
        if type_size != constant_size {
            errors.fatal((
                inst,
                format!(
                    "The instruction expects {} to have a size of {} bytes but it has {}",
                    constant, type_size, constant_size
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn domtree_integrity(
        &self,
        domtree: &DominatorTree,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        // We consider two `DominatorTree`s to be equal if they return the same immediate
        // dominator for each block. Therefore the current domtree is valid if it matches the freshly
        // computed one.
        for block in self.func.layout.blocks() {
            let expected = self.expected_domtree.idom(block);
            let got = domtree.idom(block);
            if got != expected {
                return errors.fatal((
                    block,
                    format!(
                        "invalid domtree, expected idom({}) = {:?}, got {:?}",
                        block, expected, got
                    ),
                ));
            }
        }
        // We also verify if the postorder defined by `DominatorTree` is sane
        if domtree.cfg_postorder().len() != self.expected_domtree.cfg_postorder().len() {
            return errors.fatal((
                AnyEntity::Function,
                "incorrect number of Blocks in postorder traversal",
            ));
        }
        for (index, (&test_block, &true_block)) in domtree
            .cfg_postorder()
            .iter()
            .zip(self.expected_domtree.cfg_postorder().iter())
            .enumerate()
        {
            if test_block != true_block {
                return errors.fatal((
                    test_block,
                    format!(
                        "invalid domtree, postorder block number {} should be {}, got {}",
                        index, true_block, test_block
                    ),
                ));
            }
        }
        // We verify rpo_cmp on pairs of adjacent blocks in the postorder
        for (&prev_block, &next_block) in domtree.cfg_postorder().iter().adjacent_pairs() {
            if self
                .expected_domtree
                .rpo_cmp(prev_block, next_block, &self.func.layout)
                != Ordering::Greater
            {
                return errors.fatal((
                    next_block,
                    format!(
                        "invalid domtree, rpo_cmp does not says {} is greater than {}",
                        prev_block, next_block
                    ),
                ));
            }
        }
        Ok(())
    }

    fn typecheck_entry_block_params(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        if let Some(block) = self.func.layout.entry_block() {
            let expected_types = &self.func.signature.params;
            let block_param_count = self.func.dfg.num_block_params(block);

            if block_param_count != expected_types.len() {
                return errors.fatal((
                    block,
                    format!(
                        "entry block parameters ({}) must match function signature ({})",
                        block_param_count,
                        expected_types.len()
                    ),
                ));
            }

            for (i, &arg) in self.func.dfg.block_params(block).iter().enumerate() {
                let arg_type = self.func.dfg.value_type(arg);
                if arg_type != expected_types[i].value_type {
                    errors.report((
                        block,
                        format!(
                            "entry block parameter {} expected to have type {}, got {}",
                            i, expected_types[i], arg_type
                        ),
                    ));
                }
            }
        }

        errors.as_result()
    }

    fn typecheck(&self, inst: Inst, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        let inst_data = &self.func.dfg[inst];
        let constraints = inst_data.opcode().constraints();

        let ctrl_type = if let Some(value_typeset) = constraints.ctrl_typeset() {
            // For polymorphic opcodes, determine the controlling type variable first.
            let ctrl_type = self.func.dfg.ctrl_typevar(inst);

            if !value_typeset.contains(ctrl_type) {
                errors.report((
                    inst,
                    self.context(inst),
                    format!("has an invalid controlling type {}", ctrl_type),
                ));
            }

            ctrl_type
        } else {
            // Non-polymorphic instructions don't check the controlling type variable, so `Option`
            // is unnecessary and we can just make it `INVALID`.
            types::INVALID
        };

        // Typechecking instructions is never fatal
        let _ = self.typecheck_results(inst, ctrl_type, errors);
        let _ = self.typecheck_fixed_args(inst, ctrl_type, errors);
        let _ = self.typecheck_variable_args(inst, errors);
        let _ = self.typecheck_return(inst, errors);
        let _ = self.typecheck_special(inst, ctrl_type, errors);

        Ok(())
    }

    fn typecheck_results(
        &self,
        inst: Inst,
        ctrl_type: Type,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let mut i = 0;
        for &result in self.func.dfg.inst_results(inst) {
            let result_type = self.func.dfg.value_type(result);
            let expected_type = self.func.dfg.compute_result_type(inst, i, ctrl_type);
            if let Some(expected_type) = expected_type {
                if result_type != expected_type {
                    errors.report((
                        inst,
                        self.context(inst),
                        format!(
                            "expected result {} ({}) to have type {}, found {}",
                            i, result, expected_type, result_type
                        ),
                    ));
                }
            } else {
                return errors.nonfatal((
                    inst,
                    self.context(inst),
                    "has more result values than expected",
                ));
            }
            i += 1;
        }

        // There aren't any more result types left.
        if self.func.dfg.compute_result_type(inst, i, ctrl_type) != None {
            return errors.nonfatal((
                inst,
                self.context(inst),
                "has fewer result values than expected",
            ));
        }
        Ok(())
    }

    fn typecheck_fixed_args(
        &self,
        inst: Inst,
        ctrl_type: Type,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let constraints = self.func.dfg[inst].opcode().constraints();

        for (i, &arg) in self.func.dfg.inst_fixed_args(inst).iter().enumerate() {
            let arg_type = self.func.dfg.value_type(arg);
            match constraints.value_argument_constraint(i, ctrl_type) {
                ResolvedConstraint::Bound(expected_type) => {
                    if arg_type != expected_type {
                        errors.report((
                            inst,
                            self.context(inst),
                            format!(
                                "arg {} ({}) has type {}, expected {}",
                                i, arg, arg_type, expected_type
                            ),
                        ));
                    }
                }
                ResolvedConstraint::Free(type_set) => {
                    if !type_set.contains(arg_type) {
                        errors.report((
                            inst,
                            self.context(inst),
                            format!(
                                "arg {} ({}) with type {} failed to satisfy type set {:?}",
                                i, arg, arg_type, type_set
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn typecheck_variable_args(
        &self,
        inst: Inst,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        match self.func.dfg.analyze_branch(inst) {
            BranchInfo::SingleDest(block, _) => {
                let iter = self
                    .func
                    .dfg
                    .block_params(block)
                    .iter()
                    .map(|&v| self.func.dfg.value_type(v));
                self.typecheck_variable_args_iterator(inst, iter, errors)?;
            }
            BranchInfo::Table(table, block) => {
                if let Some(block) = block {
                    let arg_count = self.func.dfg.num_block_params(block);
                    if arg_count != 0 {
                        return errors.nonfatal((
                            inst,
                            self.context(inst),
                            format!(
                                "takes no arguments, but had target {} with {} arguments",
                                block, arg_count,
                            ),
                        ));
                    }
                }
                for block in self.func.jump_tables[table].iter() {
                    let arg_count = self.func.dfg.num_block_params(*block);
                    if arg_count != 0 {
                        return errors.nonfatal((
                            inst,
                            self.context(inst),
                            format!(
                                "takes no arguments, but had target {} with {} arguments",
                                block, arg_count,
                            ),
                        ));
                    }
                }
            }
            BranchInfo::NotABranch => {}
        }

        match self.func.dfg[inst].analyze_call(&self.func.dfg.value_lists) {
            CallInfo::Direct(func_ref, _) => {
                let sig_ref = self.func.dfg.ext_funcs[func_ref].signature;
                let arg_types = self.func.dfg.signatures[sig_ref]
                    .params
                    .iter()
                    .map(|a| a.value_type);
                self.typecheck_variable_args_iterator(inst, arg_types, errors)?;
            }
            CallInfo::Indirect(sig_ref, _) => {
                let arg_types = self.func.dfg.signatures[sig_ref]
                    .params
                    .iter()
                    .map(|a| a.value_type);
                self.typecheck_variable_args_iterator(inst, arg_types, errors)?;
            }
            CallInfo::NotACall => {}
        }
        Ok(())
    }

    fn typecheck_variable_args_iterator<I: Iterator<Item = Type>>(
        &self,
        inst: Inst,
        iter: I,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let variable_args = self.func.dfg.inst_variable_args(inst);
        let mut i = 0;

        for expected_type in iter {
            if i >= variable_args.len() {
                // Result count mismatch handled below, we want the full argument count first though
                i += 1;
                continue;
            }
            let arg = variable_args[i];
            let arg_type = self.func.dfg.value_type(arg);
            if expected_type != arg_type {
                errors.report((
                    inst,
                    self.context(inst),
                    format!(
                        "arg {} ({}) has type {}, expected {}",
                        i, variable_args[i], arg_type, expected_type
                    ),
                ));
            }
            i += 1;
        }
        if i != variable_args.len() {
            return errors.nonfatal((
                inst,
                self.context(inst),
                format!(
                    "mismatched argument count for `{}`: got {}, expected {}",
                    self.func.dfg.display_inst(inst),
                    variable_args.len(),
                    i,
                ),
            ));
        }
        Ok(())
    }

    fn typecheck_return(&self, inst: Inst, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        if self.func.dfg[inst].opcode().is_return() {
            let args = self.func.dfg.inst_variable_args(inst);
            let expected_types = &self.func.signature.returns;
            if args.len() != expected_types.len() {
                return errors.nonfatal((
                    inst,
                    self.context(inst),
                    "arguments of return must match function signature",
                ));
            }
            for (i, (&arg, &expected_type)) in args.iter().zip(expected_types).enumerate() {
                let arg_type = self.func.dfg.value_type(arg);
                if arg_type != expected_type.value_type {
                    errors.report((
                        inst,
                        self.context(inst),
                        format!(
                            "arg {} ({}) has type {}, must match function signature of {}",
                            i, arg, arg_type, expected_type
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    // Check special-purpose type constraints that can't be expressed in the normal opcode
    // constraints.
    fn typecheck_special(
        &self,
        inst: Inst,
        ctrl_type: Type,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        match self.func.dfg[inst] {
            ir::InstructionData::Unary { opcode, arg } => {
                let arg_type = self.func.dfg.value_type(arg);
                match opcode {
                    Opcode::Bextend | Opcode::Uextend | Opcode::Sextend | Opcode::Fpromote => {
                        if arg_type.lane_count() != ctrl_type.lane_count() {
                            return errors.nonfatal((
                                inst,
                                self.context(inst),
                                format!(
                                    "input {} and output {} must have same number of lanes",
                                    arg_type, ctrl_type,
                                ),
                            ));
                        }
                        if arg_type.lane_bits() >= ctrl_type.lane_bits() {
                            return errors.nonfatal((
                                inst,
                                self.context(inst),
                                format!(
                                    "input {} must be smaller than output {}",
                                    arg_type, ctrl_type,
                                ),
                            ));
                        }
                    }
                    Opcode::Breduce | Opcode::Ireduce | Opcode::Fdemote => {
                        if arg_type.lane_count() != ctrl_type.lane_count() {
                            return errors.nonfatal((
                                inst,
                                self.context(inst),
                                format!(
                                    "input {} and output {} must have same number of lanes",
                                    arg_type, ctrl_type,
                                ),
                            ));
                        }
                        if arg_type.lane_bits() <= ctrl_type.lane_bits() {
                            return errors.nonfatal((
                                inst,
                                self.context(inst),
                                format!(
                                    "input {} must be larger than output {}",
                                    arg_type, ctrl_type,
                                ),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            ir::InstructionData::HeapAddr { heap, arg, .. } => {
                let index_type = self.func.dfg.value_type(arg);
                let heap_index_type = self.func.heaps[heap].index_type;
                if index_type != heap_index_type {
                    return errors.nonfatal((
                        inst,
                        self.context(inst),
                        format!(
                            "index type {} differs from heap index type {}",
                            index_type, heap_index_type,
                        ),
                    ));
                }
            }
            ir::InstructionData::TableAddr { table, arg, .. } => {
                let index_type = self.func.dfg.value_type(arg);
                let table_index_type = self.func.tables[table].index_type;
                if index_type != table_index_type {
                    return errors.nonfatal((
                        inst,
                        self.context(inst),
                        format!(
                            "index type {} differs from table index type {}",
                            index_type, table_index_type,
                        ),
                    ));
                }
            }
            ir::InstructionData::UnaryGlobalValue { global_value, .. } => {
                if let Some(isa) = self.isa {
                    let inst_type = self.func.dfg.value_type(self.func.dfg.first_result(inst));
                    let global_type = self.func.global_values[global_value].global_type(isa);
                    if inst_type != global_type {
                        return errors.nonfatal((
                            inst, self.context(inst),
                            format!(
                                "global_value instruction with type {} references global value with type {}",
                                inst_type, global_type
                            )),
                        );
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn cfg_integrity(
        &self,
        cfg: &ControlFlowGraph,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let mut expected_succs = BTreeSet::<Block>::new();
        let mut got_succs = BTreeSet::<Block>::new();
        let mut expected_preds = BTreeSet::<Inst>::new();
        let mut got_preds = BTreeSet::<Inst>::new();

        for block in self.func.layout.blocks() {
            expected_succs.extend(self.expected_cfg.succ_iter(block));
            got_succs.extend(cfg.succ_iter(block));

            let missing_succs: Vec<Block> =
                expected_succs.difference(&got_succs).cloned().collect();
            if !missing_succs.is_empty() {
                errors.report((
                    block,
                    format!("cfg lacked the following successor(s) {:?}", missing_succs),
                ));
                continue;
            }

            let excess_succs: Vec<Block> = got_succs.difference(&expected_succs).cloned().collect();
            if !excess_succs.is_empty() {
                errors.report((
                    block,
                    format!("cfg had unexpected successor(s) {:?}", excess_succs),
                ));
                continue;
            }

            expected_preds.extend(
                self.expected_cfg
                    .pred_iter(block)
                    .map(|BlockPredecessor { inst, .. }| inst),
            );
            got_preds.extend(
                cfg.pred_iter(block)
                    .map(|BlockPredecessor { inst, .. }| inst),
            );

            let missing_preds: Vec<Inst> = expected_preds.difference(&got_preds).cloned().collect();
            if !missing_preds.is_empty() {
                errors.report((
                    block,
                    format!(
                        "cfg lacked the following predecessor(s) {:?}",
                        missing_preds
                    ),
                ));
                continue;
            }

            let excess_preds: Vec<Inst> = got_preds.difference(&expected_preds).cloned().collect();
            if !excess_preds.is_empty() {
                errors.report((
                    block,
                    format!("cfg had unexpected predecessor(s) {:?}", excess_preds),
                ));
                continue;
            }

            expected_succs.clear();
            got_succs.clear();
            expected_preds.clear();
            got_preds.clear();
        }
        errors.as_result()
    }

    fn immediate_constraints(
        &self,
        inst: Inst,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<()> {
        let inst_data = &self.func.dfg[inst];

        match *inst_data {
            ir::InstructionData::Store { flags, .. }
            | ir::InstructionData::StoreComplex { flags, .. } => {
                if flags.readonly() {
                    errors.fatal((
                        inst,
                        self.context(inst),
                        "A store instruction cannot have the `readonly` MemFlag",
                    ))
                } else {
                    Ok(())
                }
            }
            ir::InstructionData::BinaryImm8 {
                opcode: ir::instructions::Opcode::Extractlane,
                imm: lane,
                arg,
                ..
            }
            | ir::InstructionData::TernaryImm8 {
                opcode: ir::instructions::Opcode::Insertlane,
                imm: lane,
                args: [arg, _],
                ..
            } => {
                // We must be specific about the opcodes above because other instructions are using
                // the same formats.
                let ty = self.func.dfg.value_type(arg);
                if u16::from(lane) >= ty.lane_count() {
                    errors.fatal((
                        inst,
                        self.context(inst),
                        format!("The lane {} does not index into the type {}", lane, ty,),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn typecheck_function_signature(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        self.func
            .signature
            .params
            .iter()
            .enumerate()
            .filter(|(_, &param)| param.value_type == types::INVALID)
            .for_each(|(i, _)| {
                errors.report((
                    AnyEntity::Function,
                    format!("Parameter at position {} has an invalid type", i),
                ));
            });

        self.func
            .signature
            .returns
            .iter()
            .enumerate()
            .filter(|(_, &ret)| ret.value_type == types::INVALID)
            .for_each(|(i, _)| {
                errors.report((
                    AnyEntity::Function,
                    format!("Return value at position {} has an invalid type", i),
                ))
            });

        self.func
            .signature
            .returns
            .iter()
            .enumerate()
            .for_each(|(i, ret)| {
                if let ArgumentPurpose::StructArgument(_) = ret.purpose {
                    errors.report((
                        AnyEntity::Function,
                        format!("Return value at position {} can't be an struct argument", i),
                    ))
                }
            });

        if errors.has_error() {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn run(&self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        self.verify_global_values(errors)?;
        self.verify_heaps(errors)?;
        self.verify_tables(errors)?;
        self.verify_jump_tables(errors)?;
        self.typecheck_entry_block_params(errors)?;
        self.typecheck_function_signature(errors)?;

        for block in self.func.layout.blocks() {
            if self.func.layout.first_inst(block).is_none() {
                return errors.fatal((block, format!("{} cannot be empty", block)));
            }
            for inst in self.func.layout.block_insts(block) {
                self.block_integrity(block, inst, errors)?;
                self.instruction_integrity(inst, errors)?;
                self.typecheck(inst, errors)?;
                self.immediate_constraints(inst, errors)?;
            }

            self.encodable_as_bb(block, errors)?;
        }

        verify_flags(self.func, &self.expected_cfg, errors)?;

        if !errors.is_empty() {
            log::warn!(
                "Found verifier errors in function:\n{}",
                pretty_verifier_error(self.func, None, errors.clone())
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Verifier, VerifierError, VerifierErrors};
    use crate::entity::EntityList;
    use crate::ir::instructions::{InstructionData, Opcode};
    use crate::ir::{types, AbiParam, Function};
    use crate::settings;

    macro_rules! assert_err_with_msg {
        ($e:expr, $msg:expr) => {
            match $e.0.get(0) {
                None => panic!("Expected an error"),
                Some(&VerifierError { ref message, .. }) => {
                    if !message.contains($msg) {
                        #[cfg(feature = "std")]
                        panic!("'{}' did not contain the substring '{}'", message, $msg);
                        #[cfg(not(feature = "std"))]
                        panic!("error message did not contain the expected substring");
                    }
                }
            }
        };
    }

    #[test]
    fn empty() {
        let func = Function::new();
        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());
        let mut errors = VerifierErrors::default();

        assert_eq!(verifier.run(&mut errors), Ok(()));
        assert!(errors.0.is_empty());
    }

    #[test]
    fn bad_instruction_format() {
        let mut func = Function::new();
        let block0 = func.dfg.make_block();
        func.layout.append_block(block0);
        let nullary_with_bad_opcode = func.dfg.make_inst(InstructionData::UnaryImm {
            opcode: Opcode::F32const,
            imm: 0.into(),
        });
        func.layout.append_inst(nullary_with_bad_opcode, block0);
        func.layout.append_inst(
            func.dfg.make_inst(InstructionData::Jump {
                opcode: Opcode::Jump,
                destination: block0,
                args: EntityList::default(),
            }),
            block0,
        );
        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());
        let mut errors = VerifierErrors::default();

        let _ = verifier.run(&mut errors);

        assert_err_with_msg!(errors, "instruction format");
    }

    #[test]
    fn test_function_invalid_param() {
        let mut func = Function::new();
        func.signature.params.push(AbiParam::new(types::INVALID));

        let mut errors = VerifierErrors::default();
        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());

        let _ = verifier.typecheck_function_signature(&mut errors);
        assert_err_with_msg!(errors, "Parameter at position 0 has an invalid type");
    }

    #[test]
    fn test_function_invalid_return_value() {
        let mut func = Function::new();
        func.signature.returns.push(AbiParam::new(types::INVALID));

        let mut errors = VerifierErrors::default();
        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());

        let _ = verifier.typecheck_function_signature(&mut errors);
        assert_err_with_msg!(errors, "Return value at position 0 has an invalid type");
    }

    #[test]
    fn test_printing_contextual_errors() {
        // Build function.
        let mut func = Function::new();
        let block0 = func.dfg.make_block();
        func.layout.append_block(block0);

        // Build instruction: v0, v1 = iconst 42
        let inst = func.dfg.make_inst(InstructionData::UnaryImm {
            opcode: Opcode::Iconst,
            imm: 42.into(),
        });
        func.dfg.append_result(inst, types::I32);
        func.dfg.append_result(inst, types::I32);
        func.layout.append_inst(inst, block0);

        // Setup verifier.
        let mut errors = VerifierErrors::default();
        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());

        // Now the error message, when printed, should contain the instruction sequence causing the
        // error (i.e. v0, v1 = iconst.i32 42) and not only its entity value (i.e. inst0)
        let _ = verifier.typecheck_results(inst, types::I32, &mut errors);
        assert_eq!(
            format!("{}", errors.0[0]),
            "inst0 (v0, v1 = iconst.i32 42): has more result values than expected"
        )
    }

    #[test]
    fn test_empty_block() {
        let mut func = Function::new();
        let block0 = func.dfg.make_block();
        func.layout.append_block(block0);

        let flags = &settings::Flags::new(settings::builder());
        let verifier = Verifier::new(&func, flags.into());
        let mut errors = VerifierErrors::default();
        let _ = verifier.run(&mut errors);

        assert_err_with_msg!(errors, "block0 cannot be empty");
    }
}
