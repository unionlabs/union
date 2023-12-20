//! This module implements lowering (instruction selection) from Cranelift IR
//! to machine instructions with virtual registers. This is *almost* the final
//! machine code, except for register allocation.

// TODO: separate the IR-query core of `Lower` from the lowering logic built on
// top of it, e.g. the side-effect/coloring analysis and the scan support.

use crate::entity::SecondaryMap;
use crate::fx::{FxHashMap, FxHashSet};
use crate::inst_predicates::{has_lowering_side_effect, is_constant_64bit};
use crate::ir::{
    types::{FFLAGS, IFLAGS},
    ArgumentPurpose, Block, Constant, ConstantData, DataFlowGraph, ExternalName, Function,
    GlobalValue, GlobalValueData, Immediate, Inst, InstructionData, MemFlags, Opcode, RelSourceLoc,
    Type, Value, ValueDef, ValueLabelAssignments, ValueLabelStart,
};
use crate::machinst::{
    non_writable_value_regs, writable_value_regs, BlockIndex, BlockLoweringOrder, Callee,
    LoweredBlock, MachLabel, Reg, SigSet, VCode, VCodeBuilder, VCodeConstant, VCodeConstantData,
    VCodeConstants, VCodeInst, ValueRegs, Writable,
};
use crate::{trace, CodegenResult};
use alloc::vec::Vec;
use regalloc2::{MachineEnv, PRegSet};
use smallvec::{smallvec, SmallVec};
use std::fmt::Debug;

use super::{VCodeBuildDirection, VRegAllocator};

/// An "instruction color" partitions CLIF instructions by side-effecting ops.
/// All instructions with the same "color" are guaranteed not to be separated by
/// any side-effecting op (for this purpose, loads are also considered
/// side-effecting, to avoid subtle questions w.r.t. the memory model), and
/// furthermore, it is guaranteed that for any two instructions A and B such
/// that color(A) == color(B), either A dominates B and B postdominates A, or
/// vice-versa. (For now, in practice, only ops in the same basic block can ever
/// have the same color, trivially providing the second condition.) Intuitively,
/// this means that the ops of the same color must always execute "together", as
/// part of one atomic contiguous section of the dynamic execution trace, and
/// they can be freely permuted (modulo true dataflow dependencies) without
/// affecting program behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct InstColor(u32);
impl InstColor {
    fn new(n: u32) -> InstColor {
        InstColor(n)
    }

    /// Get an arbitrary index representing this color. The index is unique
    /// *within a single function compilation*, but indices may be reused across
    /// functions.
    pub fn get(self) -> u32 {
        self.0
    }
}

/// A representation of all of the ways in which a value is available, aside
/// from as a direct register.
///
/// - An instruction, if it would be allowed to occur at the current location
///   instead (see [Lower::get_input_as_source_or_const()] for more details).
///
/// - A constant, if the value is known to be a constant.
#[derive(Clone, Copy, Debug)]
pub struct NonRegInput {
    /// An instruction produces this value (as the given output), and its
    /// computation (and side-effect if applicable) could occur at the
    /// current instruction's location instead.
    ///
    /// If this instruction's operation is merged into the current instruction,
    /// the backend must call [Lower::sink_inst()].
    ///
    /// This enum indicates whether this use of the source instruction
    /// is unique or not.
    pub inst: InputSourceInst,
    /// The value is a known constant.
    pub constant: Option<u64>,
}

/// When examining an input to an instruction, this enum provides one
/// of several options: there is or isn't a single instruction (that
/// we can see and merge with) that produces that input's value, and
/// we are or aren't the single user of that instruction.
#[derive(Clone, Copy, Debug)]
pub enum InputSourceInst {
    /// The input in question is the single, unique use of the given
    /// instruction and output index, and it can be sunk to the
    /// location of this input.
    UniqueUse(Inst, usize),
    /// The input in question is one of multiple uses of the given
    /// instruction. It can still be sunk to the location of this
    /// input.
    Use(Inst, usize),
    /// We cannot determine which instruction produced the input, or
    /// it is one of several instructions (e.g., due to a control-flow
    /// merge and blockparam), or the source instruction cannot be
    /// allowed to sink to the current location due to side-effects.
    None,
}

impl InputSourceInst {
    /// Get the instruction and output index for this source, whether
    /// we are its single or one of many users.
    pub fn as_inst(&self) -> Option<(Inst, usize)> {
        match self {
            &InputSourceInst::UniqueUse(inst, output_idx)
            | &InputSourceInst::Use(inst, output_idx) => Some((inst, output_idx)),
            &InputSourceInst::None => None,
        }
    }
}

/// A machine backend.
pub trait LowerBackend {
    /// The machine instruction type.
    type MInst: VCodeInst;

    /// Lower a single instruction.
    ///
    /// For a branch, this function should not generate the actual branch
    /// instruction. However, it must force any values it needs for the branch
    /// edge (block-param actuals) into registers, because the actual branch
    /// generation (`lower_branch_group()`) happens *after* any possible merged
    /// out-edge.
    fn lower(&self, ctx: &mut Lower<Self::MInst>, inst: Inst) -> CodegenResult<()>;

    /// Lower a block-terminating group of branches (which together can be seen
    /// as one N-way branch), given a vcode MachLabel for each target.
    fn lower_branch_group(
        &self,
        ctx: &mut Lower<Self::MInst>,
        insts: &[Inst],
        targets: &[MachLabel],
    ) -> CodegenResult<()>;

    /// A bit of a hack: give a fixed register that always holds the result of a
    /// `get_pinned_reg` instruction, if known.  This allows elision of moves
    /// into the associated vreg, instead using the real reg directly.
    fn maybe_pinned_reg(&self) -> Option<Reg> {
        None
    }
}

/// Machine-independent lowering driver / machine-instruction container. Maintains a correspondence
/// from original Inst to MachInsts.
pub struct Lower<'func, I: VCodeInst> {
    /// The function to lower.
    f: &'func Function,

    /// Machine-independent flags.
    flags: crate::settings::Flags,

    /// The set of allocatable registers.
    allocatable: PRegSet,

    /// Lowered machine instructions.
    vcode: VCodeBuilder<I>,

    /// VReg allocation context, given to the vcode field at build time to finalize the vcode.
    vregs: VRegAllocator<I>,

    /// Mapping from `Value` (SSA value in IR) to virtual register.
    value_regs: SecondaryMap<Value, ValueRegs<Reg>>,

    /// sret registers, if needed.
    sret_reg: Option<ValueRegs<Reg>>,

    /// Instruction colors at block exits. From this map, we can recover all
    /// instruction colors by scanning backward from the block end and
    /// decrementing on any color-changing (side-effecting) instruction.
    block_end_colors: SecondaryMap<Block, InstColor>,

    /// Instruction colors at side-effecting ops. This is the *entry* color,
    /// i.e., the version of global state that exists before an instruction
    /// executes.  For each side-effecting instruction, the *exit* color is its
    /// entry color plus one.
    side_effect_inst_entry_colors: FxHashMap<Inst, InstColor>,

    /// Current color as we scan during lowering. While we are lowering an
    /// instruction, this is equal to the color *at entry to* the instruction.
    cur_scan_entry_color: Option<InstColor>,

    /// Current instruction as we scan during lowering.
    cur_inst: Option<Inst>,

    /// Instruction constant values, if known.
    inst_constants: FxHashMap<Inst, u64>,

    /// Use-counts per SSA value, as counted in the input IR. These
    /// are "coarsened", in the abstract-interpretation sense: we only
    /// care about "0, 1, many" states, as this is all we need and
    /// this lets us do an efficient fixpoint analysis.
    ///
    /// See doc comment on `ValueUseState` for more details.
    value_ir_uses: SecondaryMap<Value, ValueUseState>,

    /// Actual uses of each SSA value so far, incremented while lowering.
    value_lowered_uses: SecondaryMap<Value, u32>,

    /// Effectful instructions that have been sunk; they are not codegen'd at
    /// their original locations.
    inst_sunk: FxHashSet<Inst>,

    /// Instructions collected for the CLIF inst in progress, in forward order.
    ir_insts: Vec<I>,

    /// The register to use for GetPinnedReg, if any, on this architecture.
    pinned_reg: Option<Reg>,
}

/// How is a value used in the IR?
///
/// This can be seen as a coarsening of an integer count. We only need
/// distinct states for zero, one, or many.
///
/// This analysis deserves further explanation. The basic idea is that
/// we want to allow instruction lowering to know whether a value that
/// an instruction references is *only* referenced by that one use, or
/// by others as well. This is necessary to know when we might want to
/// move a side-effect: we cannot, for example, duplicate a load, so
/// we cannot let instruction lowering match a load as part of a
/// subpattern and potentially incorporate it.
///
/// Note that a lot of subtlety comes into play once we have
/// *indirect* uses. The classical example of this in our development
/// history was the x86 compare instruction, which is incorporated
/// into flags users (e.g. `selectif`, `trueif`, branches) and can
/// subsequently incorporate loads, or at least we would like it
/// to. However, danger awaits: the compare might be the only user of
/// a load, so we might think we can just move the load (and nothing
/// is duplicated -- success!), except that the compare itself is
/// codegen'd in multiple places, where it is incorporated as a
/// subpattern itself.
///
/// So we really want a notion of "unique all the way along the
/// matching path". Rust's `&T` and `&mut T` offer a partial analogy
/// to the semantics that we want here: we want to know when we've
/// matched a unique use of an instruction, and that instruction's
/// unique use of another instruction, etc, just as `&mut T` can only
/// be obtained by going through a chain of `&mut T`. If one has a
/// `&T` to a struct containing `&mut T` (one of several uses of an
/// instruction that itself has a unique use of an instruction), one
/// can only get a `&T` (one can only get a "I am one of several users
/// of this instruction" result).
///
/// We could track these paths, either dynamically as one "looks up the operand
/// tree" or precomputed. But the former requires state and means that the
/// `Lower` API carries that state implicitly, which we'd like to avoid if we
/// can. And the latter implies O(n^2) storage: it is an all-pairs property (is
/// inst `i` unique from the point of view of `j`).
///
/// To make matters even a little more complex still, a value that is
/// not uniquely used when initially viewing the IR can *become*
/// uniquely used, at least as a root allowing further unique uses of
/// e.g. loads to merge, if no other instruction actually merges
/// it. To be more concrete, if we have `v1 := load; v2 := op v1; v3
/// := op v2; v4 := op v2` then `v2` is non-uniquely used, so from the
/// point of view of lowering `v4` or `v3`, we cannot merge the load
/// at `v1`. But if we decide just to use the assigned register for
/// `v2` at both `v3` and `v4`, then we only actually codegen `v2`
/// once, so it *is* a unique root at that point and we *can* merge
/// the load.
///
/// Note also that the color scheme is not sufficient to give us this
/// information, for various reasons: reasoning about side-effects
/// does not tell us about potential duplication of uses through pure
/// ops.
///
/// To keep things simple and avoid error-prone lowering APIs that
/// would extract more information about whether instruction merging
/// happens or not (we don't have that info now, and it would be
/// difficult to refactor to get it and make that refactor 100%
/// correct), we give up on the above "can become unique if not
/// actually merged" point. Instead, we compute a
/// transitive-uniqueness. That is what this enum represents.
///
/// To define it plainly: a value is `Unused` if no references exist
/// to it; `Once` if only one other op refers to it, *and* that other
/// op is `Unused` or `Once`; and `Multiple` otherwise. In other
/// words, `Multiple` is contagious: even if an op's result value is
/// directly used only once in the CLIF, that value is `Multiple` if
/// the op that uses it is itself used multiple times (hence could be
/// codegen'd multiple times). In brief, this analysis tells us
/// whether, if every op merged all of its operand tree, a given op
/// could be codegen'd in more than one place.
///
/// To compute this, we first consider direct uses. At this point
/// `Unused` answers are correct, `Multiple` answers are correct, but
/// some `Once`s may change to `Multiple`s. Then we propagate
/// `Multiple` transitively using a workqueue/fixpoint algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ValueUseState {
    /// Not used at all.
    Unused,
    /// Used exactly once.
    Once,
    /// Used multiple times.
    Multiple,
}

impl ValueUseState {
    /// Add one use.
    fn inc(&mut self) {
        let new = match self {
            Self::Unused => Self::Once,
            Self::Once | Self::Multiple => Self::Multiple,
        };
        *self = new;
    }
}

/// Notion of "relocation distance". This gives an estimate of how far away a symbol will be from a
/// reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocDistance {
    /// Target of relocation is "nearby". The threshold for this is fuzzy but should be interpreted
    /// as approximately "within the compiled output of one module"; e.g., within AArch64's +/-
    /// 128MB offset. If unsure, use `Far` instead.
    Near,
    /// Target of relocation could be anywhere in the address space.
    Far,
}

impl<'func, I: VCodeInst> Lower<'func, I> {
    /// Prepare a new lowering context for the given IR function.
    pub fn new(
        f: &'func Function,
        flags: crate::settings::Flags,
        machine_env: &MachineEnv,
        abi: Callee<I::ABIMachineSpec>,
        emit_info: I::Info,
        block_order: BlockLoweringOrder,
        sigs: SigSet,
    ) -> CodegenResult<Self> {
        let constants = VCodeConstants::with_capacity(f.dfg.constants.len());
        let vcode = VCodeBuilder::new(
            sigs,
            abi,
            emit_info,
            block_order,
            constants,
            VCodeBuildDirection::Backward,
        );

        let mut vregs = VRegAllocator::new();

        let mut value_regs = SecondaryMap::with_default(ValueRegs::invalid());

        // Assign a vreg to each block param and each inst result.
        for bb in f.layout.blocks() {
            for &param in f.dfg.block_params(bb) {
                let ty = f.dfg.value_type(param);
                if value_regs[param].is_invalid() {
                    let regs = vregs.alloc(ty)?;
                    value_regs[param] = regs;
                    trace!("bb {} param {}: regs {:?}", bb, param, regs);
                }
            }
            for inst in f.layout.block_insts(bb) {
                for &result in f.dfg.inst_results(inst) {
                    let ty = f.dfg.value_type(result);
                    if value_regs[result].is_invalid() && !ty.is_invalid() {
                        let regs = vregs.alloc(ty)?;
                        value_regs[result] = regs;
                        trace!(
                            "bb {} inst {} ({:?}): result {} regs {:?}",
                            bb,
                            inst,
                            f.dfg[inst],
                            result,
                            regs,
                        );
                    }
                }
            }
        }

        // Make a sret register, if one is needed.
        let mut sret_reg = None;
        for ret in &vcode.abi().signature().returns.clone() {
            if ret.purpose == ArgumentPurpose::StructReturn {
                assert!(sret_reg.is_none());
                sret_reg = Some(vregs.alloc(ret.value_type)?);
            }
        }

        // Compute instruction colors, find constant instructions, and find instructions with
        // side-effects, in one combined pass.
        let mut cur_color = 0;
        let mut block_end_colors = SecondaryMap::with_default(InstColor::new(0));
        let mut side_effect_inst_entry_colors = FxHashMap::default();
        let mut inst_constants = FxHashMap::default();
        for bb in f.layout.blocks() {
            cur_color += 1;
            for inst in f.layout.block_insts(bb) {
                let side_effect = has_lowering_side_effect(f, inst);

                trace!("bb {} inst {} has color {}", bb, inst, cur_color);
                if side_effect {
                    side_effect_inst_entry_colors.insert(inst, InstColor::new(cur_color));
                    trace!(" -> side-effecting; incrementing color for next inst");
                    cur_color += 1;
                }

                // Determine if this is a constant; if so, add to the table.
                if let Some(c) = is_constant_64bit(f, inst) {
                    trace!(" -> constant: {}", c);
                    inst_constants.insert(inst, c);
                }
            }

            block_end_colors[bb] = InstColor::new(cur_color);
        }

        let value_ir_uses = Self::compute_use_states(f);

        Ok(Lower {
            f,
            flags,
            allocatable: PRegSet::from(machine_env),
            vcode,
            vregs,
            value_regs,
            sret_reg,
            block_end_colors,
            side_effect_inst_entry_colors,
            inst_constants,
            value_ir_uses,
            value_lowered_uses: SecondaryMap::default(),
            inst_sunk: FxHashSet::default(),
            cur_scan_entry_color: None,
            cur_inst: None,
            ir_insts: vec![],
            pinned_reg: None,
        })
    }

    pub fn sigs(&self) -> &SigSet {
        self.vcode.sigs()
    }

    pub fn sigs_mut(&mut self) -> &mut SigSet {
        self.vcode.sigs_mut()
    }

    /// Pre-analysis: compute `value_ir_uses`. See comment on
    /// `ValueUseState` for a description of what this analysis
    /// computes.
    fn compute_use_states<'a>(f: &'a Function) -> SecondaryMap<Value, ValueUseState> {
        // We perform the analysis without recursion, so we don't
        // overflow the stack on long chains of ops in the input.
        //
        // This is sort of a hybrid of a "shallow use-count" pass and
        // a DFS. We iterate over all instructions and mark their args
        // as used. However when we increment a use-count to
        // "Multiple" we push its args onto the stack and do a DFS,
        // immediately marking the whole dependency tree as
        // Multiple. Doing both (shallow use-counting over all insts,
        // and deep Multiple propagation) lets us trim both
        // traversals, stopping recursion when a node is already at
        // the appropriate state.
        //
        // In particular, note that the *coarsening* into {Unused,
        // Once, Multiple} is part of what makes this pass more
        // efficient than a full indirect-use-counting pass.

        let mut value_ir_uses: SecondaryMap<Value, ValueUseState> =
            SecondaryMap::with_default(ValueUseState::Unused);

        // Stack of iterators over Values as we do DFS to mark
        // Multiple-state subtrees.
        type StackVec<'a> = SmallVec<[std::slice::Iter<'a, Value>; 16]>;
        let mut stack: StackVec = smallvec![];

        // Push args for a given inst onto the DFS stack.
        let push_args_on_stack = |stack: &mut StackVec<'a>, value| {
            trace!(" -> pushing args for {} onto stack", value);
            if let ValueDef::Result(src_inst, _) = f.dfg.value_def(value) {
                stack.push(f.dfg.inst_args(src_inst).iter());
            }
        };

        // Do a DFS through `value_ir_uses` to mark a subtree as
        // Multiple.
        let mark_all_uses_as_multiple =
            |value_ir_uses: &mut SecondaryMap<Value, ValueUseState>, stack: &mut StackVec<'a>| {
                while let Some(iter) = stack.last_mut() {
                    if let Some(&value) = iter.next() {
                        let value = f.dfg.resolve_aliases(value);
                        trace!(" -> DFS reaches {}", value);
                        if value_ir_uses[value] == ValueUseState::Multiple {
                            // Truncate DFS here: no need to go further,
                            // as whole subtree must already be Multiple.
                            #[cfg(debug_assertions)]
                            {
                                // With debug asserts, check one level
                                // of that invariant at least.
                                if let ValueDef::Result(src_inst, _) = f.dfg.value_def(value) {
                                    debug_assert!(f.dfg.inst_args(src_inst).iter().all(|&arg| {
                                        let arg = f.dfg.resolve_aliases(arg);
                                        value_ir_uses[arg] == ValueUseState::Multiple
                                    }));
                                }
                            }
                            continue;
                        }
                        value_ir_uses[value] = ValueUseState::Multiple;
                        trace!(" -> became Multiple");
                        push_args_on_stack(stack, value);
                    } else {
                        // Empty iterator, discard.
                        stack.pop();
                    }
                }
            };

        for inst in f
            .layout
            .blocks()
            .flat_map(|block| f.layout.block_insts(block))
        {
            // If this inst produces multiple values, we must mark all
            // of its args as Multiple, because otherwise two uses
            // could come in as Once on our two different results.
            let force_multiple = f.dfg.inst_results(inst).len() > 1;

            // Iterate over all args of all instructions, noting an
            // additional use on each operand. If an operand becomes Multiple,
            for &arg in f.dfg.inst_args(inst) {
                let arg = f.dfg.resolve_aliases(arg);
                let old = value_ir_uses[arg];
                if force_multiple {
                    trace!(
                        "forcing arg {} to Multiple because of multiple results of user inst",
                        arg
                    );
                    value_ir_uses[arg] = ValueUseState::Multiple;
                } else {
                    value_ir_uses[arg].inc();
                }
                let new = value_ir_uses[arg];
                trace!("arg {} used, old state {:?}, new {:?}", arg, old, new,);
                // On transition to Multiple, do DFS.
                if old != ValueUseState::Multiple && new == ValueUseState::Multiple {
                    push_args_on_stack(&mut stack, arg);
                    mark_all_uses_as_multiple(&mut value_ir_uses, &mut stack);
                }
            }
        }

        value_ir_uses
    }

    fn gen_arg_setup(&mut self) {
        if let Some(entry_bb) = self.f.layout.entry_block() {
            trace!(
                "gen_arg_setup: entry BB {} args are:\n{:?}",
                entry_bb,
                self.f.dfg.block_params(entry_bb)
            );

            // Make the vmctx available in debuginfo.
            if let Some(vmctx_val) = self.f.special_param(ArgumentPurpose::VMContext) {
                self.emit_value_label_marks_for_value(vmctx_val);
            }

            for (i, param) in self.f.dfg.block_params(entry_bb).iter().enumerate() {
                if !self.vcode.abi().arg_is_needed_in_body(i) {
                    continue;
                }
                let regs = writable_value_regs(self.value_regs[*param]);
                for insn in self
                    .vcode
                    .vcode
                    .abi
                    .gen_copy_arg_to_regs(&self.vcode.vcode.sigs, i, regs, &mut self.vregs)
                    .into_iter()
                {
                    self.emit(insn);
                }
                if self.abi().signature().params[i].purpose == ArgumentPurpose::StructReturn {
                    assert!(regs.len() == 1);
                    let ty = self.abi().signature().params[i].value_type;
                    // The ABI implementation must have ensured that a StructReturn
                    // arg is present in the return values.
                    assert!(self
                        .abi()
                        .signature()
                        .returns
                        .iter()
                        .position(|ret| ret.purpose == ArgumentPurpose::StructReturn)
                        .is_some());
                    self.emit(I::gen_move(
                        Writable::from_reg(self.sret_reg.unwrap().regs()[0]),
                        regs.regs()[0].to_reg(),
                        ty,
                    ));
                }
            }
            if let Some(insn) = self
                .vcode
                .vcode
                .abi
                .gen_retval_area_setup(&self.vcode.vcode.sigs, &mut self.vregs)
            {
                self.emit(insn);
            }

            // The `args` instruction below must come first. Finish
            // the current "IR inst" (with a default source location,
            // as for other special instructions inserted during
            // lowering) and continue the scan backward.
            self.finish_ir_inst(Default::default());

            if let Some(insn) = self.vcode.vcode.abi.take_args() {
                self.emit(insn);
            }
        }
    }

    /// Generate the return instruction.
    pub fn gen_return(&mut self, rets: Vec<ValueRegs<Reg>>) {
        let mut out_rets = vec![];

        let mut rets = rets.into_iter();
        for (i, ret) in self
            .abi()
            .signature()
            .returns
            .clone()
            .into_iter()
            .enumerate()
        {
            let regs = if ret.purpose == ArgumentPurpose::StructReturn {
                self.sret_reg.unwrap().clone()
            } else {
                rets.next().unwrap()
            };

            let (regs, insns) = self.vcode.abi().gen_copy_regs_to_retval(
                self.vcode.sigs(),
                i,
                regs,
                &mut self.vregs,
            );
            out_rets.extend(regs);
            for insn in insns {
                self.emit(insn);
            }
        }

        // Hack: generate a virtual instruction that uses vmctx in
        // order to keep it alive for the duration of the function,
        // for the benefit of debuginfo.
        if self.f.dfg.values_labels.is_some() {
            if let Some(vmctx_val) = self.f.special_param(ArgumentPurpose::VMContext) {
                let vmctx_reg = self.value_regs[vmctx_val].only_reg().unwrap();
                self.emit(I::gen_dummy_use(vmctx_reg));
            }
        }

        let inst = self.abi().gen_ret(out_rets);
        self.emit(inst);
    }

    /// Has this instruction been sunk to a use-site (i.e., away from its
    /// original location)?
    fn is_inst_sunk(&self, inst: Inst) -> bool {
        self.inst_sunk.contains(&inst)
    }

    // Is any result of this instruction needed?
    fn is_any_inst_result_needed(&self, inst: Inst) -> bool {
        self.f
            .dfg
            .inst_results(inst)
            .iter()
            .any(|&result| self.value_lowered_uses[result] > 0)
    }

    fn lower_clif_block<B: LowerBackend<MInst = I>>(
        &mut self,
        backend: &B,
        block: Block,
    ) -> CodegenResult<()> {
        self.cur_scan_entry_color = Some(self.block_end_colors[block]);
        // Lowering loop:
        // - For each non-branch instruction, in reverse order:
        //   - If side-effecting (load, store, branch/call/return,
        //     possible trap), or if used outside of this block, or if
        //     demanded by another inst, then lower.
        //
        // That's it! Lowering of side-effecting ops will force all *needed*
        // (live) non-side-effecting ops to be lowered at the right places, via
        // the `use_input_reg()` callback on the `Lower` (that's us). That's
        // because `use_input_reg()` sets the eager/demand bit for any insts
        // whose result registers are used.
        //
        // We set the VCodeBuilder to "backward" mode, so we emit
        // blocks in reverse order wrt the BlockIndex sequence, and
        // emit instructions in reverse order within blocks.  Because
        // the machine backend calls `ctx.emit()` in forward order, we
        // collect per-IR-inst lowered instructions in `ir_insts`,
        // then reverse these and append to the VCode at the end of
        // each IR instruction.
        for inst in self.f.layout.block_insts(block).rev() {
            let data = &self.f.dfg[inst];
            let has_side_effect = has_lowering_side_effect(self.f, inst);
            // If  inst has been sunk to another location, skip it.
            if self.is_inst_sunk(inst) {
                continue;
            }
            // Are any outputs used at least once?
            let value_needed = self.is_any_inst_result_needed(inst);
            trace!(
                "lower_clif_block: block {} inst {} ({:?}) is_branch {} side_effect {} value_needed {}",
                block,
                inst,
                data,
                data.opcode().is_branch(),
                has_side_effect,
                value_needed,
            );

            // Update scan state to color prior to this inst (as we are scanning
            // backward).
            self.cur_inst = Some(inst);
            if has_side_effect {
                let entry_color = *self
                    .side_effect_inst_entry_colors
                    .get(&inst)
                    .expect("every side-effecting inst should have a color-map entry");
                self.cur_scan_entry_color = Some(entry_color);
            }

            // Skip lowering branches; these are handled separately
            // (see `lower_clif_branches()` below).
            if self.f.dfg[inst].opcode().is_branch() {
                continue;
            }

            // Normal instruction: codegen if the instruction is side-effecting
            // or any of its outputs its used.
            if has_side_effect || value_needed {
                trace!("lowering: inst {}: {:?}", inst, self.f.dfg[inst]);
                backend.lower(self, inst)?;
            }

            let loc = self.srcloc(inst);
            self.finish_ir_inst(loc);

            // Emit value-label markers if needed, to later recover
            // debug mappings. This must happen before the instruction
            // (so after we emit, in bottom-to-top pass).
            self.emit_value_label_markers_for_inst(inst);
        }

        // Add the block params to this block.
        self.add_block_params(block)?;

        self.cur_scan_entry_color = None;
        Ok(())
    }

    fn add_block_params(&mut self, block: Block) -> CodegenResult<()> {
        for &param in self.f.dfg.block_params(block) {
            let ty = self.f.dfg.value_type(param);
            let (_reg_rcs, reg_tys) = I::rc_for_type(ty)?;
            debug_assert_eq!(reg_tys.len(), self.value_regs[param].len());
            for (&reg, &rty) in self.value_regs[param].regs().iter().zip(reg_tys.iter()) {
                let vreg = reg.to_virtual_reg().unwrap();
                self.vregs.set_vreg_type(vreg, rty);
                self.vcode.add_block_param(vreg);
            }
        }
        Ok(())
    }

    fn get_value_labels<'a>(&'a self, val: Value, depth: usize) -> Option<&'a [ValueLabelStart]> {
        if let Some(ref values_labels) = self.f.dfg.values_labels {
            trace!(
                "get_value_labels: val {} -> {} -> {:?}",
                val,
                self.f.dfg.resolve_aliases(val),
                values_labels.get(&self.f.dfg.resolve_aliases(val))
            );
            let val = self.f.dfg.resolve_aliases(val);
            match values_labels.get(&val) {
                Some(&ValueLabelAssignments::Starts(ref list)) => Some(&list[..]),
                Some(&ValueLabelAssignments::Alias { value, .. }) if depth < 10 => {
                    self.get_value_labels(value, depth + 1)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn emit_value_label_marks_for_value(&mut self, val: Value) {
        let regs = self.value_regs[val];
        if regs.len() > 1 {
            return;
        }
        let reg = regs.only_reg().unwrap();

        if let Some(label_starts) = self.get_value_labels(val, 0) {
            let labels = label_starts
                .iter()
                .map(|&ValueLabelStart { label, .. }| label)
                .collect::<FxHashSet<_>>();
            for label in labels {
                trace!(
                    "value labeling: defines val {:?} -> reg {:?} -> label {:?}",
                    val,
                    reg,
                    label,
                );
                self.vcode.add_value_label(reg, label);
            }
        }
    }

    fn emit_value_label_markers_for_inst(&mut self, inst: Inst) {
        if self.f.dfg.values_labels.is_none() {
            return;
        }

        trace!(
            "value labeling: srcloc {}: inst {}",
            self.srcloc(inst),
            inst
        );
        for &val in self.f.dfg.inst_results(inst) {
            self.emit_value_label_marks_for_value(val);
        }
    }

    fn emit_value_label_markers_for_block_args(&mut self, block: Block) {
        if self.f.dfg.values_labels.is_none() {
            return;
        }

        trace!("value labeling: block {}", block);
        for &arg in self.f.dfg.block_params(block) {
            self.emit_value_label_marks_for_value(arg);
        }
        self.finish_ir_inst(Default::default());
    }

    fn finish_ir_inst(&mut self, loc: RelSourceLoc) {
        self.vcode.set_srcloc(loc);
        // The VCodeBuilder builds in reverse order (and reverses at
        // the end), but `ir_insts` is in forward order, so reverse
        // it.
        for inst in self.ir_insts.drain(..).rev() {
            self.vcode.push(inst);
        }
    }

    fn finish_bb(&mut self) {
        self.vcode.end_bb();
    }

    fn lower_clif_branches<B: LowerBackend<MInst = I>>(
        &mut self,
        backend: &B,
        // Lowered block index:
        bindex: BlockIndex,
        // Original CLIF block:
        block: Block,
        branches: &SmallVec<[Inst; 2]>,
        targets: &SmallVec<[MachLabel; 2]>,
    ) -> CodegenResult<()> {
        trace!(
            "lower_clif_branches: block {} branches {:?} targets {:?}",
            block,
            branches,
            targets,
        );
        // When considering code-motion opportunities, consider the current
        // program point to be the first branch.
        self.cur_inst = Some(branches[0]);
        backend.lower_branch_group(self, branches, targets)?;
        let loc = self.srcloc(branches[0]);
        self.finish_ir_inst(loc);
        // Add block param outputs for current block.
        self.lower_branch_blockparam_args(bindex);
        Ok(())
    }

    fn lower_branch_blockparam_args(&mut self, block: BlockIndex) {
        for succ_idx in 0..self.vcode.block_order().succ_indices(block).len() {
            // Avoid immutable borrow by explicitly indexing.
            let (inst, succ) = self.vcode.block_order().succ_indices(block)[succ_idx];
            // Get branch args and convert to Regs.
            let branch_args = self.f.dfg.inst_variable_args(inst);
            let mut branch_arg_vregs: SmallVec<[Reg; 16]> = smallvec![];
            for &arg in branch_args {
                let arg = self.f.dfg.resolve_aliases(arg);
                let regs = self.put_value_in_regs(arg);
                for &vreg in regs.regs() {
                    let vreg = self.vcode.resolve_vreg_alias(vreg.into());
                    branch_arg_vregs.push(vreg.into());
                }
            }
            self.vcode.add_succ(succ, &branch_arg_vregs[..]);
        }
        self.finish_ir_inst(Default::default());
    }

    fn collect_branches_and_targets(
        &self,
        bindex: BlockIndex,
        _bb: Block,
        branches: &mut SmallVec<[Inst; 2]>,
        targets: &mut SmallVec<[MachLabel; 2]>,
    ) {
        branches.clear();
        targets.clear();
        let mut last_inst = None;
        for &(inst, succ) in self.vcode.block_order().succ_indices(bindex) {
            // Avoid duplicates: this ensures a br_table is only inserted once.
            if last_inst != Some(inst) {
                branches.push(inst);
            } else {
                debug_assert!(self.f.dfg[inst].opcode() == Opcode::BrTable);
                debug_assert!(branches.len() == 1);
            }
            last_inst = Some(inst);
            targets.push(MachLabel::from_block(succ));
        }
    }

    /// Lower the function.
    pub fn lower<B: LowerBackend<MInst = I>>(mut self, backend: &B) -> CodegenResult<VCode<I>> {
        trace!("about to lower function: {:?}", self.f);

        // Initialize the ABI object, giving it temps if requested.
        let temps = self
            .vcode
            .abi()
            .temps_needed(self.sigs())
            .into_iter()
            .map(|temp_ty| self.alloc_tmp(temp_ty).only_reg().unwrap())
            .collect::<Vec<_>>();
        self.vcode.init_abi(temps);

        // Get the pinned reg here (we only parameterize this function on `B`,
        // not the whole `Lower` impl).
        self.pinned_reg = backend.maybe_pinned_reg();

        self.vcode.set_entry(BlockIndex::new(0));

        // Reused vectors for branch lowering.
        let mut branches: SmallVec<[Inst; 2]> = SmallVec::new();
        let mut targets: SmallVec<[MachLabel; 2]> = SmallVec::new();

        // get a copy of the lowered order; we hold this separately because we
        // need a mut ref to the vcode to mutate it below.
        let lowered_order: SmallVec<[LoweredBlock; 64]> = self
            .vcode
            .block_order()
            .lowered_order()
            .iter()
            .cloned()
            .collect();

        // Main lowering loop over lowered blocks.
        for (bindex, lb) in lowered_order.iter().enumerate().rev() {
            let bindex = BlockIndex::new(bindex);

            // Lower the block body in reverse order (see comment in
            // `lower_clif_block()` for rationale).

            // End branches.
            if let Some(bb) = lb.orig_block() {
                self.collect_branches_and_targets(bindex, bb, &mut branches, &mut targets);
                if branches.len() > 0 {
                    self.lower_clif_branches(backend, bindex, bb, &branches, &targets)?;
                    self.finish_ir_inst(self.srcloc(branches[0]));
                }
            } else {
                // If no orig block, this must be a pure edge block;
                // get the successor and emit a jump. Add block params
                // according to the one successor, and pass them
                // through; note that the successor must have an
                // original block.
                let (_, succ) = self.vcode.block_order().succ_indices(bindex)[0];

                let orig_succ = lowered_order[succ.index()];
                let orig_succ = orig_succ
                    .orig_block()
                    .expect("Edge block succ must be body block");

                let mut branch_arg_vregs: SmallVec<[Reg; 16]> = smallvec![];
                for ty in self.f.dfg.block_param_types(orig_succ) {
                    let regs = self.vregs.alloc(ty)?;
                    for &reg in regs.regs() {
                        branch_arg_vregs.push(reg);
                        let vreg = reg.to_virtual_reg().unwrap();
                        self.vcode.add_block_param(vreg);
                    }
                }
                self.vcode.add_succ(succ, &branch_arg_vregs[..]);

                self.emit(I::gen_jump(MachLabel::from_block(succ)));
                self.finish_ir_inst(Default::default());
            }

            // Original block body.
            if let Some(bb) = lb.orig_block() {
                self.lower_clif_block(backend, bb)?;
                self.emit_value_label_markers_for_block_args(bb);
            }

            if bindex.index() == 0 {
                // Set up the function with arg vreg inits.
                self.gen_arg_setup();
                self.finish_ir_inst(Default::default());
            }

            self.finish_bb();
        }

        // Now that we've emitted all instructions into the
        // VCodeBuilder, let's build the VCode.
        let vcode = self.vcode.build(self.allocatable, self.vregs);
        trace!("built vcode: {:?}", vcode);

        Ok(vcode)
    }
}

/// Function-level queries.
impl<'func, I: VCodeInst> Lower<'func, I> {
    pub fn dfg(&self) -> &DataFlowGraph {
        &self.f.dfg
    }

    /// Get the `Callee`.
    pub fn abi(&self) -> &Callee<I::ABIMachineSpec> {
        self.vcode.abi()
    }

    /// Get the `Callee`.
    pub fn abi_mut(&mut self) -> &mut Callee<I::ABIMachineSpec> {
        self.vcode.abi_mut()
    }
}

/// Instruction input/output queries.
impl<'func, I: VCodeInst> Lower<'func, I> {
    /// Get the instdata for a given IR instruction.
    pub fn data(&self, ir_inst: Inst) -> &InstructionData {
        &self.f.dfg[ir_inst]
    }

    /// Likewise, but starting with a GlobalValue identifier.
    pub fn symbol_value_data<'b>(
        &'b self,
        global_value: GlobalValue,
    ) -> Option<(&'b ExternalName, RelocDistance, i64)> {
        let gvdata = &self.f.global_values[global_value];
        match gvdata {
            &GlobalValueData::Symbol {
                ref name,
                ref offset,
                ..
            } => {
                let offset = offset.bits();
                let dist = gvdata.maybe_reloc_distance().unwrap();
                Some((name, dist, offset))
            }
            _ => None,
        }
    }

    /// Returns the memory flags of a given memory access.
    pub fn memflags(&self, ir_inst: Inst) -> Option<MemFlags> {
        match &self.f.dfg[ir_inst] {
            &InstructionData::AtomicCas { flags, .. } => Some(flags),
            &InstructionData::AtomicRmw { flags, .. } => Some(flags),
            &InstructionData::Load { flags, .. }
            | &InstructionData::LoadNoOffset { flags, .. }
            | &InstructionData::Store { flags, .. } => Some(flags),
            &InstructionData::StoreNoOffset { flags, .. } => Some(flags),
            _ => None,
        }
    }

    /// Get the source location for a given instruction.
    pub fn srcloc(&self, ir_inst: Inst) -> RelSourceLoc {
        self.f.rel_srclocs()[ir_inst]
    }

    /// Get the number of inputs to the given IR instruction.
    pub fn num_inputs(&self, ir_inst: Inst) -> usize {
        self.f.dfg.inst_args(ir_inst).len()
    }

    /// Get the number of outputs to the given IR instruction.
    pub fn num_outputs(&self, ir_inst: Inst) -> usize {
        self.f.dfg.inst_results(ir_inst).len()
    }

    /// Get the type for an instruction's input.
    pub fn input_ty(&self, ir_inst: Inst, idx: usize) -> Type {
        self.value_ty(self.input_as_value(ir_inst, idx))
    }

    /// Get the type for a value.
    pub fn value_ty(&self, val: Value) -> Type {
        self.f.dfg.value_type(val)
    }

    /// Get the type for an instruction's output.
    pub fn output_ty(&self, ir_inst: Inst, idx: usize) -> Type {
        self.f.dfg.value_type(self.f.dfg.inst_results(ir_inst)[idx])
    }

    /// Get the value of a constant instruction (`iconst`, etc.) as a 64-bit
    /// value, if possible.
    pub fn get_constant(&self, ir_inst: Inst) -> Option<u64> {
        self.inst_constants.get(&ir_inst).cloned()
    }

    /// Get the input as one of two options other than a direct register:
    ///
    /// - An instruction, given that it is effect-free or able to sink its
    ///   effect to the current instruction being lowered, and given it has only
    ///   one output, and if effect-ful, given that this is the only use;
    /// - A constant, if the value is a constant.
    ///
    /// The instruction input may be available in either of these forms.  It may
    /// be available in neither form, if the conditions are not met; if so, use
    /// `put_input_in_regs()` instead to get it in a register.
    ///
    /// If the backend merges the effect of a side-effecting instruction, it
    /// must call `sink_inst()`. When this is called, it indicates that the
    /// effect has been sunk to the current scan location. The sunk
    /// instruction's result(s) must have *no* uses remaining, because it will
    /// not be codegen'd (it has been integrated into the current instruction).
    pub fn input_as_value(&self, ir_inst: Inst, idx: usize) -> Value {
        let val = self.f.dfg.inst_args(ir_inst)[idx];
        self.f.dfg.resolve_aliases(val)
    }

    /// Like `get_input_as_source_or_const` but with a `Value`.
    pub fn get_input_as_source_or_const(&self, ir_inst: Inst, idx: usize) -> NonRegInput {
        let val = self.input_as_value(ir_inst, idx);
        self.get_value_as_source_or_const(val)
    }

    /// Resolves a particular input of an instruction to the `Value` that it is
    /// represented with.
    pub fn get_value_as_source_or_const(&self, val: Value) -> NonRegInput {
        trace!(
            "get_input_for_val: val {} at cur_inst {:?} cur_scan_entry_color {:?}",
            val,
            self.cur_inst,
            self.cur_scan_entry_color,
        );
        let inst = match self.f.dfg.value_def(val) {
            // OK to merge source instruction if (i) we have a source
            // instruction, and:
            // - It has no side-effects, OR
            // - It has a side-effect, has one output value, that one
            //   output has only one use, directly or indirectly (so
            //   cannot be duplicated -- see comment on
            //   `ValueUseState`), and the instruction's color is *one
            //   less than* the current scan color.
            //
            //   This latter set of conditions is testing whether a
            //   side-effecting instruction can sink to the current scan
            //   location; this is possible if the in-color of this inst is
            //   equal to the out-color of the producing inst, so no other
            //   side-effecting ops occur between them (which will only be true
            //   if they are in the same BB, because color increments at each BB
            //   start).
            //
            //   If it is actually sunk, then in `merge_inst()`, we update the
            //   scan color so that as we scan over the range past which the
            //   instruction was sunk, we allow other instructions (that came
            //   prior to the sunk instruction) to sink.
            ValueDef::Result(src_inst, result_idx) => {
                let src_side_effect = has_lowering_side_effect(self.f, src_inst);
                trace!(" -> src inst {}", src_inst);
                trace!(" -> has lowering side effect: {}", src_side_effect);
                if !src_side_effect {
                    // Pure instruction: always possible to
                    // sink. Let's determine whether we are the only
                    // user or not.
                    if self.value_ir_uses[val] == ValueUseState::Once {
                        InputSourceInst::UniqueUse(src_inst, result_idx)
                    } else {
                        InputSourceInst::Use(src_inst, result_idx)
                    }
                } else {
                    // Side-effect: test whether this is the only use of the
                    // only result of the instruction, and whether colors allow
                    // the code-motion.
                    trace!(
                        " -> side-effecting op {} for val {}: use state {:?}",
                        src_inst,
                        val,
                        self.value_ir_uses[val]
                    );
                    if self.cur_scan_entry_color.is_some()
                        && self.value_ir_uses[val] == ValueUseState::Once
                        && self.num_outputs(src_inst) == 1
                        && self
                            .side_effect_inst_entry_colors
                            .get(&src_inst)
                            .unwrap()
                            .get()
                            + 1
                            == self.cur_scan_entry_color.unwrap().get()
                    {
                        InputSourceInst::UniqueUse(src_inst, 0)
                    } else {
                        InputSourceInst::None
                    }
                }
            }
            _ => InputSourceInst::None,
        };
        let constant = inst.as_inst().and_then(|(inst, _)| self.get_constant(inst));

        NonRegInput { inst, constant }
    }

    /// Increment the reference count for the Value, ensuring that it gets lowered.
    pub fn increment_lowered_uses(&mut self, val: Value) {
        self.value_lowered_uses[val] += 1
    }

    /// Put the `idx`th input into register(s) and return the assigned register.
    pub fn put_input_in_regs(&mut self, ir_inst: Inst, idx: usize) -> ValueRegs<Reg> {
        let val = self.f.dfg.inst_args(ir_inst)[idx];
        self.put_value_in_regs(val)
    }

    /// Put the given value into register(s) and return the assigned register.
    pub fn put_value_in_regs(&mut self, val: Value) -> ValueRegs<Reg> {
        let val = self.f.dfg.resolve_aliases(val);
        trace!("put_value_in_regs: val {}", val);

        // Assert that the value is not `iflags`/`fflags`-typed; these
        // cannot be reified into normal registers. TODO(#3249)
        // eventually remove the `iflags` type altogether!
        let ty = self.f.dfg.value_type(val);
        assert!(ty != IFLAGS && ty != FFLAGS);

        if let Some(inst) = self.f.dfg.value_def(val).inst() {
            assert!(!self.inst_sunk.contains(&inst));
        }

        // If the value is a constant, then (re)materialize it at each
        // use. This lowers register pressure. (Only do this if we are
        // not using egraph-based compilation; the egraph framework
        // more efficiently rematerializes constants where needed.)
        if !self.flags.use_egraphs() {
            if let Some(c) = self
                .f
                .dfg
                .value_def(val)
                .inst()
                .and_then(|inst| self.get_constant(inst))
            {
                let regs = self.alloc_tmp(ty);
                trace!(" -> regs {:?}", regs);
                assert!(regs.is_valid());

                let insts = I::gen_constant(regs, c.into(), ty, |ty| {
                    self.alloc_tmp(ty).only_reg().unwrap()
                });
                for inst in insts {
                    self.emit(inst);
                }
                return non_writable_value_regs(regs);
            }
        }

        let regs = self.value_regs[val];
        trace!(" -> regs {:?}", regs);
        assert!(regs.is_valid());

        self.value_lowered_uses[val] += 1;

        regs
    }

    /// Get the `idx`th output register(s) of the given IR instruction.
    ///
    /// When `backend.lower_inst_to_regs(ctx, inst)` is called, it is expected
    /// that the backend will write results to these output register(s).  This
    /// register will always be "fresh"; it is guaranteed not to overlap with
    /// any of the inputs, and can be freely used as a scratch register within
    /// the lowered instruction sequence, as long as its final value is the
    /// result of the computation.
    pub fn get_output(&self, ir_inst: Inst, idx: usize) -> ValueRegs<Writable<Reg>> {
        let val = self.f.dfg.inst_results(ir_inst)[idx];
        writable_value_regs(self.value_regs[val])
    }
}

/// Codegen primitives: allocate temps, emit instructions, set result registers,
/// ask for an input to be gen'd into a register.
impl<'func, I: VCodeInst> Lower<'func, I> {
    /// Get a new temp.
    pub fn alloc_tmp(&mut self, ty: Type) -> ValueRegs<Writable<Reg>> {
        writable_value_regs(self.vregs.alloc(ty).unwrap())
    }

    /// Emit a machine instruction.
    pub fn emit(&mut self, mach_inst: I) {
        trace!("emit: {:?}", mach_inst);
        self.ir_insts.push(mach_inst);
    }

    /// Indicate that the side-effect of an instruction has been sunk to the
    /// current scan location. This should only be done with the instruction's
    /// original results are not used (i.e., `put_input_in_regs` is not invoked
    /// for the input produced by the sunk instruction), otherwise the
    /// side-effect will occur twice.
    pub fn sink_inst(&mut self, ir_inst: Inst) {
        assert!(has_lowering_side_effect(self.f, ir_inst));
        assert!(self.cur_scan_entry_color.is_some());

        for result in self.dfg().inst_results(ir_inst) {
            assert!(self.value_lowered_uses[*result] == 0);
        }

        let sunk_inst_entry_color = self
            .side_effect_inst_entry_colors
            .get(&ir_inst)
            .cloned()
            .unwrap();
        let sunk_inst_exit_color = InstColor::new(sunk_inst_entry_color.get() + 1);
        assert!(sunk_inst_exit_color == self.cur_scan_entry_color.unwrap());
        self.cur_scan_entry_color = Some(sunk_inst_entry_color);
        self.inst_sunk.insert(ir_inst);
    }

    /// Retrieve immediate data given a handle.
    pub fn get_immediate_data(&self, imm: Immediate) -> &ConstantData {
        self.f.dfg.immediates.get(imm).unwrap()
    }

    /// Retrieve constant data given a handle.
    pub fn get_constant_data(&self, constant_handle: Constant) -> &ConstantData {
        self.f.dfg.constants.get(constant_handle)
    }

    /// Indicate that a constant should be emitted.
    pub fn use_constant(&mut self, constant: VCodeConstantData) -> VCodeConstant {
        self.vcode.constants().insert(constant)
    }

    /// Cause the value in `reg` to be in a virtual reg, by copying it into a new virtual reg
    /// if `reg` is a real reg.  `ty` describes the type of the value in `reg`.
    pub fn ensure_in_vreg(&mut self, reg: Reg, ty: Type) -> Reg {
        if reg.to_virtual_reg().is_some() {
            reg
        } else {
            let new_reg = self.alloc_tmp(ty).only_reg().unwrap();
            self.emit(I::gen_move(new_reg, reg, ty));
            new_reg.to_reg()
        }
    }

    /// Note that one vreg is to be treated as an alias of another.
    pub fn set_vreg_alias(&mut self, from: Reg, to: Reg) {
        trace!("set vreg alias: from {:?} to {:?}", from, to);
        self.vcode.set_vreg_alias(from, to);
    }
}
