//! This implements the VCode container: a CFG of Insts that have been lowered.
//!
//! VCode is virtual-register code. An instruction in VCode is almost a machine
//! instruction; however, its register slots can refer to virtual registers in
//! addition to real machine registers.
//!
//! VCode is structured with traditional basic blocks, and
//! each block must be terminated by an unconditional branch (one target), a
//! conditional branch (two targets), or a return (no targets). Note that this
//! slightly differs from the machine code of most ISAs: in most ISAs, a
//! conditional branch has one target (and the not-taken case falls through).
//! However, we expect that machine backends will elide branches to the following
//! block (i.e., zero-offset jumps), and will be able to codegen a branch-cond /
//! branch-uncond pair if *both* targets are not fallthrough. This allows us to
//! play with layout prior to final binary emission, as well, if we want.
//!
//! See the main module comment in `mod.rs` for more details on the VCode-based
//! backend pipeline.

use crate::fx::FxHashMap;
use crate::fx::FxHashSet;
use crate::ir::RelSourceLoc;
use crate::ir::{self, types, Constant, ConstantData, DynamicStackSlot, LabelValueLoc, ValueLabel};
use crate::machinst::*;
use crate::timing;
use crate::trace;
use crate::CodegenError;
use crate::ValueLocRange;
use regalloc2::{
    Edit, Function as RegallocFunction, InstOrEdit, InstRange, Operand, OperandKind, PReg, PRegSet,
    RegClass, VReg,
};

use alloc::vec::Vec;
use cranelift_entity::{entity_impl, Keys, PrimaryMap};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

/// Index referring to an instruction in VCode.
pub type InsnIndex = regalloc2::Inst;

/// Index referring to a basic block in VCode.
pub type BlockIndex = regalloc2::Block;

/// VCodeInst wraps all requirements for a MachInst to be in VCode: it must be
/// a `MachInst` and it must be able to emit itself at least to a `SizeCodeSink`.
pub trait VCodeInst: MachInst + MachInstEmit {}
impl<I: MachInst + MachInstEmit> VCodeInst for I {}

/// A function in "VCode" (virtualized-register code) form, after
/// lowering.  This is essentially a standard CFG of basic blocks,
/// where each basic block consists of lowered instructions produced
/// by the machine-specific backend.
///
/// Note that the VCode is immutable once produced, and is not
/// modified by register allocation in particular. Rather, register
/// allocation on the `VCode` produces a separate `regalloc2::Output`
/// struct, and this can be passed to `emit`. `emit` in turn does not
/// modify the vcode, but produces an `EmitResult`, which contains the
/// machine code itself, and the associated disassembly and/or
/// metadata as requested.
pub struct VCode<I: VCodeInst> {
    /// VReg IR-level types.
    vreg_types: Vec<Type>,

    /// Lowered machine instructions in order corresponding to the original IR.
    insts: Vec<I>,

    /// Operands: pre-regalloc references to virtual registers with
    /// constraints, in one flattened array. This allows the regalloc
    /// to efficiently access all operands without requiring expensive
    /// matches or method invocations on insts.
    operands: Vec<Operand>,

    /// Operand index ranges: for each instruction in `insts`, there
    /// is a tuple here providing the range in `operands` for that
    /// instruction's operands.
    operand_ranges: Vec<(u32, u32)>,

    /// Clobbers: a sparse map from instruction indices to clobber masks.
    clobbers: FxHashMap<InsnIndex, PRegSet>,

    /// Move information: for a given InsnIndex, (src, dst) operand pair.
    is_move: FxHashMap<InsnIndex, (Operand, Operand)>,

    /// Source locations for each instruction. (`SourceLoc` is a `u32`, so it is
    /// reasonable to keep one of these per instruction.)
    srclocs: Vec<RelSourceLoc>,

    /// Entry block.
    entry: BlockIndex,

    /// Block instruction indices.
    block_ranges: Vec<(InsnIndex, InsnIndex)>,

    /// Block successors: index range in the `block_succs_preds` list.
    block_succ_range: Vec<(u32, u32)>,

    /// Block predecessors: index range in the `block_succs_preds` list.
    block_pred_range: Vec<(u32, u32)>,

    /// Block successor and predecessor lists, concatenated into one
    /// Vec. The `block_succ_range` and `block_pred_range` lists of
    /// tuples above give (start, end) ranges within this list that
    /// correspond to each basic block's successors or predecessors,
    /// respectively.
    block_succs_preds: Vec<regalloc2::Block>,

    /// Block parameters: index range in `block_params` below.
    block_params_range: Vec<(u32, u32)>,

    /// Block parameter lists, concatenated into one vec. The
    /// `block_params_range` list of tuples above gives (start, end)
    /// ranges within this list that correspond to each basic block's
    /// blockparam vregs.
    block_params: Vec<regalloc2::VReg>,

    /// Outgoing block arguments on branch instructions, concatenated
    /// into one list.
    ///
    /// Note that this is conceptually a 3D array: we have a VReg list
    /// per block, per successor. We flatten those three dimensions
    /// into this 1D vec, then store index ranges in two levels of
    /// indirection.
    ///
    /// Indexed by the indices in `branch_block_arg_succ_range`.
    branch_block_args: Vec<regalloc2::VReg>,

    /// Array of sequences of (start, end) tuples in
    /// `branch_block_args`, one for each successor; these sequences
    /// for each block are concatenated.
    ///
    /// Indexed by the indices in `branch_block_arg_succ_range`.
    branch_block_arg_range: Vec<(u32, u32)>,

    /// For a given block, indices in `branch_block_arg_range`
    /// corresponding to all of its successors.
    branch_block_arg_succ_range: Vec<(u32, u32)>,

    /// VReg aliases. Each key in this table is translated to its
    /// value when gathering Operands from instructions. Aliases are
    /// not chased transitively (we do not further look up the
    /// translated reg to see if it is another alias).
    ///
    /// We use these aliases to rename an instruction's expected
    /// result vregs to the returned vregs from lowering, which are
    /// usually freshly-allocated temps.
    ///
    /// Operands and branch arguments will already have been
    /// translated through this alias table; but it helps to make
    /// sense of instructions when pretty-printed, for example.
    vreg_aliases: FxHashMap<regalloc2::VReg, regalloc2::VReg>,

    /// Block-order information.
    block_order: BlockLoweringOrder,

    /// ABI object.
    pub(crate) abi: Callee<I::ABIMachineSpec>,

    /// Constant information used during code emission. This should be
    /// immutable across function compilations within the same module.
    emit_info: I::Info,

    /// Reference-typed `regalloc2::VReg`s. The regalloc requires
    /// these in a dense slice (as opposed to querying the
    /// reftype-status of each vreg) for efficient iteration.
    reftyped_vregs: Vec<VReg>,

    /// Constants.
    constants: VCodeConstants,

    /// Value labels for debuginfo attached to vregs.
    debug_value_labels: Vec<(VReg, InsnIndex, InsnIndex, u32)>,

    pub(crate) sigs: SigSet,
}

/// The result of `VCode::emit`. Contains all information computed
/// during emission: actual machine code, optionally a disassembly,
/// and optionally metadata about the code layout.
pub struct EmitResult<I: VCodeInst> {
    /// The MachBuffer containing the machine code.
    pub buffer: MachBuffer<I>,

    /// Offset of each basic block, recorded during emission. Computed
    /// only if `debug_value_labels` is non-empty.
    pub bb_offsets: Vec<CodeOffset>,

    /// Final basic-block edges, in terms of code offsets of
    /// bb-starts. Computed only if `debug_value_labels` is non-empty.
    pub bb_edges: Vec<(CodeOffset, CodeOffset)>,

    /// Final instruction offsets, recorded during emission. Computed
    /// only if `debug_value_labels` is non-empty.
    pub inst_offsets: Vec<CodeOffset>,

    /// Final length of function body.
    pub func_body_len: CodeOffset,

    /// The pretty-printed disassembly, if any. This uses the same
    /// pretty-printing for MachInsts as the pre-regalloc VCode Debug
    /// implementation, but additionally includes the prologue and
    /// epilogue(s), and makes use of the regalloc results.
    pub disasm: Option<String>,

    /// Offsets of sized stackslots.
    pub sized_stackslot_offsets: PrimaryMap<StackSlot, u32>,

    /// Offsets of dynamic stackslots.
    pub dynamic_stackslot_offsets: PrimaryMap<DynamicStackSlot, u32>,

    /// Value-labels information (debug metadata).
    pub value_labels_ranges: ValueLabelsRanges,

    /// Stack frame size.
    pub frame_size: u32,

    /// The alignment requirement for pc-relative loads.
    pub alignment: u32,
}

/// A builder for a VCode function body.
///
/// This builder has the ability to accept instructions in either
/// forward or reverse order, depending on the pass direction that
/// produces the VCode. The lowering from CLIF to VCode<MachInst>
/// ordinarily occurs in reverse order (in order to allow instructions
/// to be lowered only if used, and not merged) so a reversal will
/// occur at the end of lowering to ensure the VCode is in machine
/// order.
///
/// If built in reverse, block and instruction indices used once the
/// VCode is built are relative to the final (reversed) order, not the
/// order of construction. Note that this means we do not know the
/// final block or instruction indices when building, so we do not
/// hand them out. (The user is assumed to know them when appending
/// terminator instructions with successor blocks.)
pub struct VCodeBuilder<I: VCodeInst> {
    /// In-progress VCode.
    pub(crate) vcode: VCode<I>,

    /// In what direction is the build occuring?
    direction: VCodeBuildDirection,

    /// Index of the last block-start in the vcode.
    block_start: usize,

    /// Start of succs for the current block in the concatenated succs list.
    succ_start: usize,

    /// Start of blockparams for the current block in the concatenated
    /// blockparams list.
    block_params_start: usize,

    /// Start of successor blockparam arg list entries in
    /// the concatenated branch_block_arg_range list.
    branch_block_arg_succ_start: usize,

    /// Current source location.
    cur_srcloc: RelSourceLoc,

    /// Debug-value label in-progress map, keyed by label. For each
    /// label, we keep disjoint ranges mapping to vregs. We'll flatten
    /// this into (vreg, range, label) tuples when done.
    debug_info: FxHashMap<ValueLabel, Vec<(InsnIndex, InsnIndex, VReg)>>,
}

/// Direction in which a VCodeBuilder builds VCode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VCodeBuildDirection {
    // TODO: add `Forward` once we need it and can test it adequately.
    /// Backward-build pass: we expect the producer to call `emit()`
    /// with instructions in reverse program order within each block.
    Backward,
}

impl<I: VCodeInst> VCodeBuilder<I> {
    /// Create a new VCodeBuilder.
    pub fn new(
        sigs: SigSet,
        abi: Callee<I::ABIMachineSpec>,
        emit_info: I::Info,
        block_order: BlockLoweringOrder,
        constants: VCodeConstants,
        direction: VCodeBuildDirection,
    ) -> VCodeBuilder<I> {
        let vcode = VCode::new(sigs, abi, emit_info, block_order, constants);

        VCodeBuilder {
            vcode,
            direction,
            block_start: 0,
            succ_start: 0,
            block_params_start: 0,
            branch_block_arg_succ_start: 0,
            cur_srcloc: Default::default(),
            debug_info: FxHashMap::default(),
        }
    }

    pub fn init_abi(&mut self, temps: Vec<Writable<Reg>>) {
        self.vcode.abi.init(&self.vcode.sigs, temps);
    }

    /// Access the ABI object.
    pub fn abi(&self) -> &Callee<I::ABIMachineSpec> {
        &self.vcode.abi
    }

    /// Access the ABI object.
    pub fn abi_mut(&mut self) -> &mut Callee<I::ABIMachineSpec> {
        &mut self.vcode.abi
    }

    pub fn sigs(&self) -> &SigSet {
        &self.vcode.sigs
    }

    pub fn sigs_mut(&mut self) -> &mut SigSet {
        &mut self.vcode.sigs
    }

    /// Access to the BlockLoweringOrder object.
    pub fn block_order(&self) -> &BlockLoweringOrder {
        &self.vcode.block_order
    }

    /// Set the current block as the entry block.
    pub fn set_entry(&mut self, block: BlockIndex) {
        self.vcode.entry = block;
    }

    /// End the current basic block. Must be called after emitting vcode insts
    /// for IR insts and prior to ending the function (building the VCode).
    pub fn end_bb(&mut self) {
        let start_idx = self.block_start;
        let end_idx = self.vcode.insts.len();
        self.block_start = end_idx;
        // Add the instruction index range to the list of blocks.
        self.vcode
            .block_ranges
            .push((InsnIndex::new(start_idx), InsnIndex::new(end_idx)));
        // End the successors list.
        let succ_end = self.vcode.block_succs_preds.len();
        self.vcode
            .block_succ_range
            .push((self.succ_start as u32, succ_end as u32));
        self.succ_start = succ_end;
        // End the blockparams list.
        let block_params_end = self.vcode.block_params.len();
        self.vcode
            .block_params_range
            .push((self.block_params_start as u32, block_params_end as u32));
        self.block_params_start = block_params_end;
        // End the branch blockparam args list.
        let branch_block_arg_succ_end = self.vcode.branch_block_arg_range.len();
        self.vcode.branch_block_arg_succ_range.push((
            self.branch_block_arg_succ_start as u32,
            branch_block_arg_succ_end as u32,
        ));
        self.branch_block_arg_succ_start = branch_block_arg_succ_end;
    }

    pub fn add_block_param(&mut self, param: VirtualReg) {
        self.vcode.block_params.push(param.into());
    }

    fn add_branch_args_for_succ(&mut self, args: &[Reg]) {
        let start = self.vcode.branch_block_args.len();
        self.vcode
            .branch_block_args
            .extend(args.iter().map(|&arg| VReg::from(arg)));
        let end = self.vcode.branch_block_args.len();
        self.vcode
            .branch_block_arg_range
            .push((start as u32, end as u32));
    }

    /// Push an instruction for the current BB and current IR inst
    /// within the BB.
    pub fn push(&mut self, insn: I) {
        self.vcode.insts.push(insn);
        self.vcode.srclocs.push(self.cur_srcloc);
    }

    /// Add a successor block with branch args.
    pub fn add_succ(&mut self, block: BlockIndex, args: &[Reg]) {
        self.vcode.block_succs_preds.push(block);
        self.add_branch_args_for_succ(args);
    }

    /// Set the current source location.
    pub fn set_srcloc(&mut self, srcloc: RelSourceLoc) {
        self.cur_srcloc = srcloc;
    }

    /// Add a debug value label to a register.
    pub fn add_value_label(&mut self, reg: Reg, label: ValueLabel) {
        // We'll fix up labels in reverse(). Because we're generating
        // code bottom-to-top, the liverange of the label goes *from*
        // the last index at which was defined (or 0, which is the end
        // of the eventual function) *to* just this instruction, and
        // no further.
        let inst = InsnIndex::new(self.vcode.insts.len());
        let labels = self.debug_info.entry(label).or_insert_with(|| vec![]);
        let last = labels
            .last()
            .map(|(_start, end, _vreg)| *end)
            .unwrap_or(InsnIndex::new(0));
        labels.push((last, inst, reg.into()));
    }

    pub fn set_vreg_alias(&mut self, from: Reg, to: Reg) {
        let from = from.into();
        let resolved_to = self.resolve_vreg_alias(to.into());
        // Disallow cycles (see below).
        assert_ne!(resolved_to, from);
        self.vcode.vreg_aliases.insert(from, resolved_to);
    }

    pub fn resolve_vreg_alias(&self, from: regalloc2::VReg) -> regalloc2::VReg {
        Self::resolve_vreg_alias_impl(&self.vcode.vreg_aliases, from)
    }

    fn resolve_vreg_alias_impl(
        aliases: &FxHashMap<regalloc2::VReg, regalloc2::VReg>,
        from: regalloc2::VReg,
    ) -> regalloc2::VReg {
        // We prevent cycles from existing by resolving targets of
        // aliases eagerly before setting them. If the target resolves
        // to the origin of the alias, then a cycle would be created
        // and the alias is disallowed. Because of the structure of
        // SSA code (one instruction can refer to another's defs but
        // not vice-versa, except indirectly through
        // phis/blockparams), cycles should not occur as we use
        // aliases to redirect vregs to the temps that actually define
        // them.

        let mut vreg = from;
        while let Some(to) = aliases.get(&vreg) {
            vreg = *to;
        }
        vreg
    }

    /// Access the constants.
    pub fn constants(&mut self) -> &mut VCodeConstants {
        &mut self.vcode.constants
    }

    fn compute_preds_from_succs(&mut self) {
        // Compute predecessors from successors. In order to gather
        // all preds for a block into a contiguous sequence, we build
        // a list of (succ, pred) tuples and then sort.
        let mut succ_pred_edges: Vec<(BlockIndex, BlockIndex)> =
            Vec::with_capacity(self.vcode.block_succs_preds.len());
        for (pred, &(start, end)) in self.vcode.block_succ_range.iter().enumerate() {
            let pred = BlockIndex::new(pred);
            for i in start..end {
                let succ = BlockIndex::new(self.vcode.block_succs_preds[i as usize].index());
                succ_pred_edges.push((succ, pred));
            }
        }
        succ_pred_edges.sort_unstable();

        let mut i = 0;
        for succ in 0..self.vcode.num_blocks() {
            let succ = BlockIndex::new(succ);
            let start = self.vcode.block_succs_preds.len();
            while i < succ_pred_edges.len() && succ_pred_edges[i].0 == succ {
                let pred = succ_pred_edges[i].1;
                self.vcode.block_succs_preds.push(pred);
                i += 1;
            }
            let end = self.vcode.block_succs_preds.len();
            self.vcode.block_pred_range.push((start as u32, end as u32));
        }
    }

    /// Called once, when a build in Backward order is complete, to
    /// perform the overall reversal (into final forward order) and
    /// finalize metadata accordingly.
    fn reverse_and_finalize(&mut self) {
        let n_insts = self.vcode.insts.len();
        if n_insts == 0 {
            return;
        }

        // Reverse the per-block and per-inst sequences.
        self.vcode.block_ranges.reverse();
        // block_params_range is indexed by block (and blocks were
        // traversed in reverse) so we reverse it; but block-param
        // sequences in the concatenated vec can remain in reverse
        // order (it is effectively an arena of arbitrarily-placed
        // referenced sequences).
        self.vcode.block_params_range.reverse();
        // Likewise, we reverse block_succ_range, but the block_succ
        // concatenated array can remain as-is.
        self.vcode.block_succ_range.reverse();
        self.vcode.insts.reverse();
        self.vcode.srclocs.reverse();
        // Likewise, branch_block_arg_succ_range is indexed by block
        // so must be reversed.
        self.vcode.branch_block_arg_succ_range.reverse();

        // To translate an instruction index *endpoint* in reversed
        // order to forward order, compute `n_insts - i`.
        //
        // Why not `n_insts - 1 - i`? That would be correct to
        // translate an individual instruction index (for ten insts 0
        // to 9 inclusive, inst 0 becomes 9, and inst 9 becomes
        // 0). But for the usual inclusive-start, exclusive-end range
        // idiom, inclusive starts become exclusive ends and
        // vice-versa, so e.g. an (inclusive) start of 0 becomes an
        // (exclusive) end of 10.
        let translate = |inst: InsnIndex| InsnIndex::new(n_insts - inst.index());

        // Edit the block-range instruction indices.
        for tuple in &mut self.vcode.block_ranges {
            let (start, end) = *tuple;
            *tuple = (translate(end), translate(start)); // Note reversed order.
        }

        // Generate debug-value labels based on per-label maps.
        for (label, tuples) in &self.debug_info {
            for &(start, end, vreg) in tuples {
                let vreg = self.resolve_vreg_alias(vreg);
                let fwd_start = translate(end);
                let fwd_end = translate(start);
                self.vcode
                    .debug_value_labels
                    .push((vreg, fwd_start, fwd_end, label.as_u32()));
            }
        }

        // Now sort debug value labels by VReg, as required
        // by regalloc2.
        self.vcode
            .debug_value_labels
            .sort_unstable_by_key(|(vreg, _, _, _)| *vreg);
    }

    fn collect_operands(&mut self, allocatable: PRegSet) {
        for (i, insn) in self.vcode.insts.iter().enumerate() {
            // Push operands from the instruction onto the operand list.
            //
            // We rename through the vreg alias table as we collect
            // the operands. This is better than a separate post-pass
            // over operands, because it has more cache locality:
            // operands only need to pass through L1 once. This is
            // also better than renaming instructions'
            // operands/registers while lowering, because here we only
            // need to do the `match` over the instruction to visit
            // its register fields (which is slow, branchy code) once.

            let vreg_aliases = &self.vcode.vreg_aliases;
            let mut op_collector =
                OperandCollector::new(&mut self.vcode.operands, allocatable, |vreg| {
                    Self::resolve_vreg_alias_impl(vreg_aliases, vreg)
                });
            insn.get_operands(&mut op_collector);
            let (ops, clobbers) = op_collector.finish();
            self.vcode.operand_ranges.push(ops);

            if clobbers != PRegSet::default() {
                self.vcode.clobbers.insert(InsnIndex::new(i), clobbers);
            }

            if let Some((dst, src)) = insn.is_move() {
                let src = Operand::reg_use(Self::resolve_vreg_alias_impl(vreg_aliases, src.into()));
                let dst = Operand::reg_def(Self::resolve_vreg_alias_impl(
                    vreg_aliases,
                    dst.to_reg().into(),
                ));
                // Note that regalloc2 requires these in (src, dst) order.
                self.vcode.is_move.insert(InsnIndex::new(i), (src, dst));
            }
        }

        // Translate blockparam args via the vreg aliases table as well.
        for arg in &mut self.vcode.branch_block_args {
            let new_arg = Self::resolve_vreg_alias_impl(&self.vcode.vreg_aliases, *arg);
            trace!("operandcollector: block arg {:?} -> {:?}", arg, new_arg);
            *arg = new_arg;
        }
    }

    /// Build the final VCode.
    pub fn build(mut self, allocatable: PRegSet, vregs: VRegAllocator<I>) -> VCode<I> {
        self.vcode.vreg_types = vregs.vreg_types;
        self.vcode.reftyped_vregs = vregs.reftyped_vregs;

        if self.direction == VCodeBuildDirection::Backward {
            self.reverse_and_finalize();
        }
        self.collect_operands(allocatable);

        // Apply register aliases to the `reftyped_vregs` list since this list
        // will be returned directly to `regalloc2` eventually and all
        // operands/results of instructions will use the alias-resolved vregs
        // from `regalloc2`'s perspective.
        //
        // Also note that `reftyped_vregs` can't have duplicates, so after the
        // aliases are applied duplicates are removed.
        for reg in self.vcode.reftyped_vregs.iter_mut() {
            *reg = Self::resolve_vreg_alias_impl(&self.vcode.vreg_aliases, *reg);
        }
        self.vcode.reftyped_vregs.sort();
        self.vcode.reftyped_vregs.dedup();

        self.compute_preds_from_succs();
        self.vcode.debug_value_labels.sort_unstable();
        self.vcode
    }
}

/// Is this type a reference type?
fn is_reftype(ty: Type) -> bool {
    ty == types::R64 || ty == types::R32
}

impl<I: VCodeInst> VCode<I> {
    /// New empty VCode.
    fn new(
        sigs: SigSet,
        abi: Callee<I::ABIMachineSpec>,
        emit_info: I::Info,
        block_order: BlockLoweringOrder,
        constants: VCodeConstants,
    ) -> VCode<I> {
        let n_blocks = block_order.lowered_order().len();
        VCode {
            sigs,
            vreg_types: vec![],
            insts: Vec::with_capacity(10 * n_blocks),
            operands: Vec::with_capacity(30 * n_blocks),
            operand_ranges: Vec::with_capacity(10 * n_blocks),
            clobbers: FxHashMap::default(),
            is_move: FxHashMap::default(),
            srclocs: Vec::with_capacity(10 * n_blocks),
            entry: BlockIndex::new(0),
            block_ranges: Vec::with_capacity(n_blocks),
            block_succ_range: Vec::with_capacity(n_blocks),
            block_succs_preds: Vec::with_capacity(2 * n_blocks),
            block_pred_range: Vec::with_capacity(n_blocks),
            block_params_range: Vec::with_capacity(n_blocks),
            block_params: Vec::with_capacity(5 * n_blocks),
            branch_block_args: Vec::with_capacity(10 * n_blocks),
            branch_block_arg_range: Vec::with_capacity(2 * n_blocks),
            branch_block_arg_succ_range: Vec::with_capacity(n_blocks),
            block_order,
            abi,
            emit_info,
            reftyped_vregs: vec![],
            constants,
            debug_value_labels: vec![],
            vreg_aliases: FxHashMap::with_capacity_and_hasher(10 * n_blocks, Default::default()),
        }
    }

    /// Get the number of blocks. Block indices will be in the range `0 ..
    /// (self.num_blocks() - 1)`.
    pub fn num_blocks(&self) -> usize {
        self.block_ranges.len()
    }

    /// The number of lowered instructions.
    pub fn num_insts(&self) -> usize {
        self.insts.len()
    }

    /// Get the successors for a block.
    pub fn succs(&self, block: BlockIndex) -> &[BlockIndex] {
        let (start, end) = self.block_succ_range[block.index()];
        &self.block_succs_preds[start as usize..end as usize]
    }

    fn compute_clobbers(&self, regalloc: &regalloc2::Output) -> Vec<Writable<RealReg>> {
        // Compute clobbered registers.
        let mut clobbered = vec![];
        let mut clobbered_set = FxHashSet::default();

        // All moves are included in clobbers.
        for edit in &regalloc.edits {
            let Edit::Move { to, .. } = edit.1;
            if let Some(preg) = to.as_reg() {
                let reg = RealReg::from(preg);
                if clobbered_set.insert(reg) {
                    clobbered.push(Writable::from_reg(reg));
                }
            }
        }

        for (i, (start, end)) in self.operand_ranges.iter().enumerate() {
            // Skip this instruction if not "included in clobbers" as
            // per the MachInst. (Some backends use this to implement
            // ABI specifics; e.g., excluding calls of the same ABI as
            // the current function from clobbers, because by
            // definition everything clobbered by the call can be
            // clobbered by this function without saving as well.)
            if !self.insts[i].is_included_in_clobbers() {
                continue;
            }

            let start = *start as usize;
            let end = *end as usize;
            let operands = &self.operands[start..end];
            let allocs = &regalloc.allocs[start..end];
            for (operand, alloc) in operands.iter().zip(allocs.iter()) {
                // We're interested only in writes (Mods or Defs).
                if operand.kind() == OperandKind::Use {
                    continue;
                }
                if let Some(preg) = alloc.as_reg() {
                    let reg = RealReg::from(preg);
                    if clobbered_set.insert(reg) {
                        clobbered.push(Writable::from_reg(reg));
                    }
                }
            }

            // Also add explicitly-clobbered registers.
            for preg in self
                .clobbers
                .get(&InsnIndex::new(i))
                .cloned()
                .unwrap_or_default()
            {
                let reg = RealReg::from(preg);
                if clobbered_set.insert(reg) {
                    clobbered.push(Writable::from_reg(reg));
                }
            }
        }

        clobbered
    }

    /// Emit the instructions to a `MachBuffer`, containing fixed-up
    /// code and external reloc/trap/etc. records ready for use. Takes
    /// the regalloc results as well.
    ///
    /// Returns the machine code itself, and optionally metadata
    /// and/or a disassembly, as an `EmitResult`. The `VCode` itself
    /// is consumed by the emission process.
    pub fn emit(
        mut self,
        regalloc: &regalloc2::Output,
        want_disasm: bool,
        want_metadata: bool,
    ) -> EmitResult<I>
    where
        I: VCodeInst,
    {
        // To write into disasm string.
        use core::fmt::Write;

        let _tt = timing::vcode_emit();
        let mut buffer = MachBuffer::new();
        let mut bb_starts: Vec<Option<CodeOffset>> = vec![];

        // The first M MachLabels are reserved for block indices, the next N MachLabels for
        // constants.
        buffer.reserve_labels_for_blocks(self.num_blocks());
        buffer.reserve_labels_for_constants(&self.constants);

        // Construct the final order we emit code in: cold blocks at the end.
        let mut final_order: SmallVec<[BlockIndex; 16]> = smallvec![];
        let mut cold_blocks: SmallVec<[BlockIndex; 16]> = smallvec![];
        for block in 0..self.num_blocks() {
            let block = BlockIndex::new(block);
            if self.block_order.is_cold(block) {
                cold_blocks.push(block);
            } else {
                final_order.push(block);
            }
        }
        final_order.extend(cold_blocks.clone());

        // Compute/save info we need for the prologue: clobbers and
        // number of spillslots.
        //
        // We clone `abi` here because we will mutate it as we
        // generate the prologue and set other info, but we can't
        // mutate `VCode`. The info it usually carries prior to
        // setting clobbers is fairly minimal so this should be
        // relatively cheap.
        let clobbers = self.compute_clobbers(regalloc);
        self.abi.set_num_spillslots(regalloc.num_spillslots);
        self.abi.set_clobbered(clobbers);

        // We need to generate the prologue in order to get the ABI
        // object into the right state first. We'll emit it when we
        // hit the right block below.
        let prologue_insts = self.abi.gen_prologue(&self.sigs);

        // Emit blocks.
        let mut cur_srcloc = None;
        let mut last_offset = None;
        let mut inst_offsets = vec![];
        let mut state = I::State::new(&self.abi);

        let mut disasm = String::new();

        if !self.debug_value_labels.is_empty() {
            inst_offsets.resize(self.insts.len(), 0);
        }

        // Count edits per block ahead of time; this is needed for
        // lookahead island emission. (We could derive it per-block
        // with binary search in the edit list, but it's more
        // efficient to do it in one pass here.)
        let mut ra_edits_per_block: SmallVec<[u32; 64]> = smallvec![];
        let mut edit_idx = 0;
        for block in 0..self.num_blocks() {
            let end_inst = self.block_ranges[block].1;
            let start_edit_idx = edit_idx;
            while edit_idx < regalloc.edits.len() && regalloc.edits[edit_idx].0.inst() < end_inst {
                edit_idx += 1;
            }
            let end_edit_idx = edit_idx;
            ra_edits_per_block.push((end_edit_idx - start_edit_idx) as u32);
        }

        let is_forward_edge_cfi_enabled = self.abi.is_forward_edge_cfi_enabled();

        for (block_order_idx, &block) in final_order.iter().enumerate() {
            trace!("emitting block {:?}", block);
            let new_offset = I::align_basic_block(buffer.cur_offset());
            while new_offset > buffer.cur_offset() {
                // Pad with NOPs up to the aligned block offset.
                let nop = I::gen_nop((new_offset - buffer.cur_offset()) as usize);
                nop.emit(&[], &mut buffer, &self.emit_info, &mut Default::default());
            }
            assert_eq!(buffer.cur_offset(), new_offset);

            let do_emit = |inst: &I,
                           allocs: &[Allocation],
                           disasm: &mut String,
                           buffer: &mut MachBuffer<I>,
                           state: &mut I::State| {
                if want_disasm && !inst.is_args() {
                    let mut s = state.clone();
                    writeln!(disasm, "  {}", inst.pretty_print_inst(allocs, &mut s)).unwrap();
                }
                inst.emit(allocs, buffer, &self.emit_info, state);
            };

            // Is this the first block? Emit the prologue directly if so.
            if block == self.entry {
                trace!(" -> entry block");
                buffer.start_srcloc(Default::default());
                state.pre_sourceloc(Default::default());
                for inst in &prologue_insts {
                    do_emit(&inst, &[], &mut disasm, &mut buffer, &mut state);
                }
                buffer.end_srcloc();
            }

            // Now emit the regular block body.

            buffer.bind_label(MachLabel::from_block(block));

            if want_disasm {
                writeln!(&mut disasm, "block{}:", block.index()).unwrap();
            }

            if want_metadata {
                // Track BB starts. If we have backed up due to MachBuffer
                // branch opts, note that the removed blocks were removed.
                let cur_offset = buffer.cur_offset();
                if last_offset.is_some() && cur_offset <= last_offset.unwrap() {
                    for i in (0..bb_starts.len()).rev() {
                        if bb_starts[i].is_some() && cur_offset > bb_starts[i].unwrap() {
                            break;
                        }
                        bb_starts[i] = None;
                    }
                }
                bb_starts.push(Some(cur_offset));
                last_offset = Some(cur_offset);
            }

            if let Some(block_start) = I::gen_block_start(
                self.block_order.is_indirect_branch_target(block),
                is_forward_edge_cfi_enabled,
            ) {
                do_emit(&block_start, &[], &mut disasm, &mut buffer, &mut state);
            }

            for inst_or_edit in regalloc.block_insts_and_edits(&self, block) {
                match inst_or_edit {
                    InstOrEdit::Inst(iix) => {
                        if !self.debug_value_labels.is_empty() {
                            // If we need to produce debug info,
                            // record the offset of each instruction
                            // so that we can translate value-label
                            // ranges to machine-code offsets.

                            // Cold blocks violate monotonicity
                            // assumptions elsewhere (that
                            // instructions in inst-index order are in
                            // order in machine code), so we omit
                            // their offsets here. Value-label range
                            // generation below will skip empty ranges
                            // and ranges with to-offsets of zero.
                            if !self.block_order.is_cold(block) {
                                inst_offsets[iix.index()] = buffer.cur_offset();
                            }
                        }

                        if self.insts[iix.index()].is_move().is_some() {
                            // Skip moves in the pre-regalloc program;
                            // all of these are incorporated by the
                            // regalloc into its unified move handling
                            // and they come out the other end, if
                            // still needed (not elided), as
                            // regalloc-inserted moves.
                            continue;
                        }

                        // Update the srcloc at this point in the buffer.
                        let srcloc = self.srclocs[iix.index()];
                        if cur_srcloc != Some(srcloc) {
                            if cur_srcloc.is_some() {
                                buffer.end_srcloc();
                            }
                            buffer.start_srcloc(srcloc);
                            cur_srcloc = Some(srcloc);
                        }
                        state.pre_sourceloc(cur_srcloc.unwrap_or_default());

                        // If this is a safepoint, compute a stack map
                        // and pass it to the emit state.
                        if self.insts[iix.index()].is_safepoint() {
                            let mut safepoint_slots: SmallVec<[SpillSlot; 8]> = smallvec![];
                            // Find the contiguous range of
                            // (progpoint, allocation) safepoint slot
                            // records in `regalloc.safepoint_slots`
                            // for this instruction index.
                            let safepoint_slots_start = regalloc
                                .safepoint_slots
                                .binary_search_by(|(progpoint, _alloc)| {
                                    if progpoint.inst() >= iix {
                                        std::cmp::Ordering::Greater
                                    } else {
                                        std::cmp::Ordering::Less
                                    }
                                })
                                .unwrap_err();

                            for (_, alloc) in regalloc.safepoint_slots[safepoint_slots_start..]
                                .iter()
                                .take_while(|(progpoint, _)| progpoint.inst() == iix)
                            {
                                let slot = alloc.as_stack().unwrap();
                                safepoint_slots.push(slot);
                            }
                            if !safepoint_slots.is_empty() {
                                let stack_map = self
                                    .abi
                                    .spillslots_to_stack_map(&safepoint_slots[..], &state);
                                state.pre_safepoint(stack_map);
                            }
                        }

                        // Get the allocations for this inst from the regalloc result.
                        let allocs = regalloc.inst_allocs(iix);

                        // If the instruction we are about to emit is
                        // a return, place an epilogue at this point
                        // (and don't emit the return; the actual
                        // epilogue will contain it).
                        if self.insts[iix.index()].is_term() == MachTerminator::Ret {
                            for inst in self.abi.gen_epilogue() {
                                do_emit(&inst, &[], &mut disasm, &mut buffer, &mut state);
                            }
                        } else {
                            // Emit the instruction!
                            do_emit(
                                &self.insts[iix.index()],
                                allocs,
                                &mut disasm,
                                &mut buffer,
                                &mut state,
                            );
                        }
                    }

                    InstOrEdit::Edit(Edit::Move { from, to }) => {
                        // Create a move/spill/reload instruction and
                        // immediately emit it.
                        match (from.as_reg(), to.as_reg()) {
                            (Some(from), Some(to)) => {
                                // Reg-to-reg move.
                                let from_rreg = Reg::from(from);
                                let to_rreg = Writable::from_reg(Reg::from(to));
                                debug_assert_eq!(from.class(), to.class());
                                let ty = I::canonical_type_for_rc(from.class());
                                let mv = I::gen_move(to_rreg, from_rreg, ty);
                                do_emit(&mv, &[], &mut disasm, &mut buffer, &mut state);
                            }
                            (Some(from), None) => {
                                // Spill from register to spillslot.
                                let to = to.as_stack().unwrap();
                                let from_rreg = RealReg::from(from);
                                let spill = self.abi.gen_spill(to, from_rreg);
                                do_emit(&spill, &[], &mut disasm, &mut buffer, &mut state);
                            }
                            (None, Some(to)) => {
                                // Load from spillslot to register.
                                let from = from.as_stack().unwrap();
                                let to_rreg = Writable::from_reg(RealReg::from(to));
                                let reload = self.abi.gen_reload(to_rreg, from);
                                do_emit(&reload, &[], &mut disasm, &mut buffer, &mut state);
                            }
                            (None, None) => {
                                panic!("regalloc2 should have eliminated stack-to-stack moves!");
                            }
                        }
                    }
                }
            }

            if cur_srcloc.is_some() {
                buffer.end_srcloc();
                cur_srcloc = None;
            }

            // Do we need an island? Get the worst-case size of the
            // next BB and see if, having emitted that many bytes, we
            // will be beyond the deadline.
            if block_order_idx < final_order.len() - 1 {
                let next_block = final_order[block_order_idx + 1];
                let next_block_range = self.block_ranges[next_block.index()];
                let next_block_size =
                    (next_block_range.1.index() - next_block_range.0.index()) as u32;
                let next_block_ra_insertions = ra_edits_per_block[next_block.index()];
                let worst_case_next_bb =
                    I::worst_case_size() * (next_block_size + next_block_ra_insertions);
                if buffer.island_needed(worst_case_next_bb) {
                    buffer.emit_island(worst_case_next_bb);
                }
            }
        }

        // Emit the constants used by the function.
        let mut alignment = 1;
        for (constant, data) in self.constants.iter() {
            alignment = data.alignment().max(alignment);

            let label = buffer.get_label_for_constant(constant);
            buffer.defer_constant(label, data.alignment(), data.as_slice(), u32::max_value());
        }

        let func_body_len = buffer.cur_offset();

        // Create `bb_edges` and final (filtered) `bb_starts`.
        let mut bb_edges = vec![];
        let mut bb_offsets = vec![];
        if want_metadata {
            for block in 0..self.num_blocks() {
                if bb_starts[block].is_none() {
                    // Block was deleted by MachBuffer; skip.
                    continue;
                }
                let from = bb_starts[block].unwrap();

                bb_offsets.push(from);
                // Resolve each `succ` label and add edges.
                let succs = self.block_succs(BlockIndex::new(block));
                for &succ in succs.iter() {
                    let to = buffer.resolve_label_offset(MachLabel::from_block(succ));
                    bb_edges.push((from, to));
                }
            }
        }

        let value_labels_ranges =
            self.compute_value_labels_ranges(regalloc, &inst_offsets[..], func_body_len);
        let frame_size = self.abi.frame_size();

        EmitResult {
            buffer,
            bb_offsets,
            bb_edges,
            inst_offsets,
            func_body_len,
            disasm: if want_disasm { Some(disasm) } else { None },
            sized_stackslot_offsets: self.abi.sized_stackslot_offsets().clone(),
            dynamic_stackslot_offsets: self.abi.dynamic_stackslot_offsets().clone(),
            value_labels_ranges,
            frame_size,
            alignment,
        }
    }

    fn compute_value_labels_ranges(
        &self,
        regalloc: &regalloc2::Output,
        inst_offsets: &[CodeOffset],
        func_body_len: u32,
    ) -> ValueLabelsRanges {
        if self.debug_value_labels.is_empty() {
            return ValueLabelsRanges::default();
        }

        let mut value_labels_ranges: ValueLabelsRanges = HashMap::new();
        for &(label, from, to, alloc) in &regalloc.debug_locations {
            let ranges = value_labels_ranges
                .entry(ValueLabel::from_u32(label))
                .or_insert_with(|| vec![]);
            let from_offset = inst_offsets[from.inst().index()];
            let to_offset = if to.inst().index() == inst_offsets.len() {
                func_body_len
            } else {
                inst_offsets[to.inst().index()]
            };

            // Empty range or to-offset of zero can happen because of
            // cold blocks (see above).
            if to_offset == 0 || from_offset == to_offset {
                continue;
            }

            let loc = if let Some(preg) = alloc.as_reg() {
                LabelValueLoc::Reg(Reg::from(preg))
            } else {
                // We can't translate spillslot locations at the
                // moment because ValueLabelLoc requires an
                // instantaneous SP offset, and this can *change*
                // within the range we have here because of callsites
                // adjusting SP temporarily. To avoid the complexity
                // of accurately plumbing through nominal-SP
                // adjustment sites, we just omit debug info for
                // values that are spilled. Not ideal, but debug info
                // is best-effort.
                continue;
            };

            ranges.push(ValueLocRange {
                loc,
                // ValueLocRanges are recorded by *instruction-end
                // offset*. `from_offset` is the *start* of the
                // instruction; that is the same as the end of another
                // instruction, so we only want to begin coverage once
                // we are past the previous instruction's end.
                start: from_offset + 1,
                // Likewise, `end` is exclusive, but we want to
                // *include* the end of the last
                // instruction. `to_offset` is the start of the
                // `to`-instruction, which is the exclusive end, i.e.,
                // the first instruction not covered. That
                // instruction's start is the same as the end of the
                // last instruction that is included, so we go one
                // byte further to be sure to include it.
                end: to_offset + 1,
            });
        }

        value_labels_ranges
    }

    /// Get the IR block for a BlockIndex, if one exists.
    pub fn bindex_to_bb(&self, block: BlockIndex) -> Option<ir::Block> {
        self.block_order.lowered_order()[block.index()].orig_block()
    }

    #[inline]
    fn assert_no_vreg_aliases<'a>(&self, list: &'a [VReg]) -> &'a [VReg] {
        for vreg in list {
            self.assert_not_vreg_alias(*vreg);
        }
        list
    }

    #[inline]
    fn assert_not_vreg_alias(&self, vreg: VReg) -> VReg {
        debug_assert!(VCodeBuilder::<I>::resolve_vreg_alias_impl(&self.vreg_aliases, vreg) == vreg);
        vreg
    }

    #[inline]
    fn assert_operand_not_vreg_alias(&self, op: Operand) -> Operand {
        // It should be true by construction that `Operand`s do not contain any
        // aliased vregs since they're all collected and mapped when the VCode
        // is itself constructed.
        self.assert_not_vreg_alias(op.vreg());
        op
    }
}

impl<I: VCodeInst> RegallocFunction for VCode<I> {
    fn num_insts(&self) -> usize {
        self.insts.len()
    }

    fn num_blocks(&self) -> usize {
        self.block_ranges.len()
    }

    fn entry_block(&self) -> BlockIndex {
        self.entry
    }

    fn block_insns(&self, block: BlockIndex) -> InstRange {
        let (start, end) = self.block_ranges[block.index()];
        InstRange::forward(start, end)
    }

    fn block_succs(&self, block: BlockIndex) -> &[BlockIndex] {
        let (start, end) = self.block_succ_range[block.index()];
        &self.block_succs_preds[start as usize..end as usize]
    }

    fn block_preds(&self, block: BlockIndex) -> &[BlockIndex] {
        let (start, end) = self.block_pred_range[block.index()];
        &self.block_succs_preds[start as usize..end as usize]
    }

    fn block_params(&self, block: BlockIndex) -> &[VReg] {
        let (start, end) = self.block_params_range[block.index()];
        let ret = &self.block_params[start as usize..end as usize];
        // Currently block params are never aliased to another vreg, but
        // double-check just to be sure.
        self.assert_no_vreg_aliases(ret)
    }

    fn branch_blockparams(&self, block: BlockIndex, _insn: InsnIndex, succ_idx: usize) -> &[VReg] {
        let (succ_range_start, succ_range_end) = self.branch_block_arg_succ_range[block.index()];
        let succ_ranges =
            &self.branch_block_arg_range[succ_range_start as usize..succ_range_end as usize];
        let (branch_block_args_start, branch_block_args_end) = succ_ranges[succ_idx];
        let ret = &self.branch_block_args
            [branch_block_args_start as usize..branch_block_args_end as usize];
        self.assert_no_vreg_aliases(ret)
    }

    fn is_ret(&self, insn: InsnIndex) -> bool {
        match self.insts[insn.index()].is_term() {
            MachTerminator::Ret => true,
            _ => false,
        }
    }

    fn is_branch(&self, insn: InsnIndex) -> bool {
        match self.insts[insn.index()].is_term() {
            MachTerminator::Cond | MachTerminator::Uncond | MachTerminator::Indirect => true,
            _ => false,
        }
    }

    fn requires_refs_on_stack(&self, insn: InsnIndex) -> bool {
        self.insts[insn.index()].is_safepoint()
    }

    fn is_move(&self, insn: InsnIndex) -> Option<(Operand, Operand)> {
        let (a, b) = self.is_move.get(&insn)?;
        Some((
            self.assert_operand_not_vreg_alias(*a),
            self.assert_operand_not_vreg_alias(*b),
        ))
    }

    fn inst_operands(&self, insn: InsnIndex) -> &[Operand] {
        let (start, end) = self.operand_ranges[insn.index()];
        let ret = &self.operands[start as usize..end as usize];
        for op in ret {
            self.assert_operand_not_vreg_alias(*op);
        }
        ret
    }

    fn inst_clobbers(&self, insn: InsnIndex) -> PRegSet {
        self.clobbers.get(&insn).cloned().unwrap_or_default()
    }

    fn num_vregs(&self) -> usize {
        std::cmp::max(self.vreg_types.len(), first_user_vreg_index())
    }

    fn reftype_vregs(&self) -> &[VReg] {
        self.assert_no_vreg_aliases(&self.reftyped_vregs[..])
    }

    fn debug_value_labels(&self) -> &[(VReg, InsnIndex, InsnIndex, u32)] {
        // VRegs here are inserted into `debug_value_labels` after code is
        // generated and aliases are fully defined, so no double-check that
        // aliases are not lingering.
        for (vreg, ..) in self.debug_value_labels.iter() {
            self.assert_not_vreg_alias(*vreg);
        }
        &self.debug_value_labels[..]
    }

    fn is_pinned_vreg(&self, vreg: VReg) -> Option<PReg> {
        pinned_vreg_to_preg(vreg)
    }

    fn spillslot_size(&self, regclass: RegClass) -> usize {
        self.abi.get_spillslot_size(regclass) as usize
    }

    fn allow_multiple_vreg_defs(&self) -> bool {
        // At least the s390x backend requires this, because the
        // `Loop` pseudo-instruction aggregates all Operands so pinned
        // vregs (RealRegs) may occur more than once.
        true
    }
}

impl<I: VCodeInst> fmt::Debug for VCode<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "VCode {{")?;
        writeln!(f, "  Entry block: {}", self.entry.index())?;

        let mut state = Default::default();

        let mut alias_keys = self.vreg_aliases.keys().cloned().collect::<Vec<_>>();
        alias_keys.sort_unstable();
        for key in alias_keys {
            let dest = self.vreg_aliases.get(&key).unwrap();
            writeln!(f, "  {:?} := {:?}", Reg::from(key), Reg::from(*dest))?;
        }

        for block in 0..self.num_blocks() {
            let block = BlockIndex::new(block);
            writeln!(f, "Block {}:", block.index())?;
            if let Some(bb) = self.bindex_to_bb(block) {
                writeln!(f, "    (original IR block: {})", bb)?;
            }
            for succ in self.succs(block) {
                writeln!(f, "    (successor: Block {})", succ.index())?;
            }
            let (start, end) = self.block_ranges[block.index()];
            writeln!(
                f,
                "    (instruction range: {} .. {})",
                start.index(),
                end.index()
            )?;
            for inst in start.index()..end.index() {
                writeln!(
                    f,
                    "  Inst {}: {}",
                    inst,
                    self.insts[inst].pretty_print_inst(&[], &mut state)
                )?;
            }
        }

        writeln!(f, "}}")?;
        Ok(())
    }
}

/// This structure manages VReg allocation during the lifetime of the VCodeBuilder.
pub struct VRegAllocator<I> {
    /// Next virtual register number to allocate.
    next_vreg: usize,

    /// VReg IR-level types.
    vreg_types: Vec<Type>,

    /// A set with the same contents as `reftyped_vregs`, in order to
    /// avoid inserting more than once.
    reftyped_vregs_set: FxHashSet<VReg>,

    /// Reference-typed `regalloc2::VReg`s. The regalloc requires
    /// these in a dense slice (as opposed to querying the
    /// reftype-status of each vreg) for efficient iteration.
    reftyped_vregs: Vec<VReg>,

    /// The type of instruction that this allocator makes registers for.
    _inst: core::marker::PhantomData<I>,
}

impl<I: VCodeInst> VRegAllocator<I> {
    /// Make a new VRegAllocator.
    pub fn new() -> Self {
        Self {
            next_vreg: first_user_vreg_index(),
            vreg_types: vec![],
            reftyped_vregs_set: FxHashSet::default(),
            reftyped_vregs: vec![],
            _inst: core::marker::PhantomData::default(),
        }
    }

    /// Allocate a fresh ValueRegs.
    pub fn alloc(&mut self, ty: Type) -> CodegenResult<ValueRegs<Reg>> {
        let v = self.next_vreg;
        let (regclasses, tys) = I::rc_for_type(ty)?;
        self.next_vreg += regclasses.len();
        if self.next_vreg >= VReg::MAX {
            return Err(CodegenError::CodeTooLarge);
        }

        let regs: ValueRegs<Reg> = match regclasses {
            &[rc0] => ValueRegs::one(VReg::new(v, rc0).into()),
            &[rc0, rc1] => ValueRegs::two(VReg::new(v, rc0).into(), VReg::new(v + 1, rc1).into()),
            // We can extend this if/when we support 32-bit targets; e.g.,
            // an i128 on a 32-bit machine will need up to four machine regs
            // for a `Value`.
            _ => panic!("Value must reside in 1 or 2 registers"),
        };
        for (&reg_ty, &reg) in tys.iter().zip(regs.regs().iter()) {
            self.set_vreg_type(reg.to_virtual_reg().unwrap(), reg_ty);
        }
        Ok(regs)
    }

    /// Set the type of this virtual register.
    pub fn set_vreg_type(&mut self, vreg: VirtualReg, ty: Type) {
        if self.vreg_types.len() <= vreg.index() {
            self.vreg_types.resize(vreg.index() + 1, ir::types::INVALID);
        }
        self.vreg_types[vreg.index()] = ty;
        if is_reftype(ty) {
            let vreg: VReg = vreg.into();
            if self.reftyped_vregs_set.insert(vreg) {
                self.reftyped_vregs.push(vreg);
            }
        }
    }
}

/// This structure tracks the large constants used in VCode that will be emitted separately by the
/// [MachBuffer].
///
/// First, during the lowering phase, constants are inserted using
/// [VCodeConstants.insert]; an intermediate handle, [VCodeConstant], tracks what constants are
/// used in this phase. Some deduplication is performed, when possible, as constant
/// values are inserted.
///
/// Secondly, during the emission phase, the [MachBuffer] assigns [MachLabel]s for each of the
/// constants so that instructions can refer to the value's memory location. The [MachBuffer]
/// then writes the constant values to the buffer.
#[derive(Default)]
pub struct VCodeConstants {
    constants: PrimaryMap<VCodeConstant, VCodeConstantData>,
    pool_uses: HashMap<Constant, VCodeConstant>,
    well_known_uses: HashMap<*const [u8], VCodeConstant>,
    u64s: HashMap<[u8; 8], VCodeConstant>,
}
impl VCodeConstants {
    /// Initialize the structure with the expected number of constants.
    pub fn with_capacity(expected_num_constants: usize) -> Self {
        Self {
            constants: PrimaryMap::with_capacity(expected_num_constants),
            pool_uses: HashMap::with_capacity(expected_num_constants),
            well_known_uses: HashMap::new(),
            u64s: HashMap::new(),
        }
    }

    /// Insert a constant; using this method indicates that a constant value will be used and thus
    /// will be emitted to the `MachBuffer`. The current implementation can deduplicate constants
    /// that are [VCodeConstantData::Pool] or [VCodeConstantData::WellKnown] but not
    /// [VCodeConstantData::Generated].
    pub fn insert(&mut self, data: VCodeConstantData) -> VCodeConstant {
        match data {
            VCodeConstantData::Generated(_) => self.constants.push(data),
            VCodeConstantData::Pool(constant, _) => match self.pool_uses.get(&constant) {
                None => {
                    let vcode_constant = self.constants.push(data);
                    self.pool_uses.insert(constant, vcode_constant);
                    vcode_constant
                }
                Some(&vcode_constant) => vcode_constant,
            },
            VCodeConstantData::WellKnown(data_ref) => {
                match self.well_known_uses.entry(data_ref as *const [u8]) {
                    Entry::Vacant(v) => {
                        let vcode_constant = self.constants.push(data);
                        v.insert(vcode_constant);
                        vcode_constant
                    }
                    Entry::Occupied(o) => *o.get(),
                }
            }
            VCodeConstantData::U64(value) => match self.u64s.entry(value) {
                Entry::Vacant(v) => {
                    let vcode_constant = self.constants.push(data);
                    v.insert(vcode_constant);
                    vcode_constant
                }
                Entry::Occupied(o) => *o.get(),
            },
        }
    }

    /// Return the number of constants inserted.
    pub fn len(&self) -> usize {
        self.constants.len()
    }

    /// Iterate over the [VCodeConstant] keys inserted in this structure.
    pub fn keys(&self) -> Keys<VCodeConstant> {
        self.constants.keys()
    }

    /// Iterate over the [VCodeConstant] keys and the data (as a byte slice) inserted in this
    /// structure.
    pub fn iter(&self) -> impl Iterator<Item = (VCodeConstant, &VCodeConstantData)> {
        self.constants.iter()
    }
}

/// A use of a constant by one or more VCode instructions; see [VCodeConstants].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VCodeConstant(u32);
entity_impl!(VCodeConstant);

/// Identify the different types of constant that can be inserted into [VCodeConstants]. Tracking
/// these separately instead of as raw byte buffers allows us to avoid some duplication.
pub enum VCodeConstantData {
    /// A constant already present in the Cranelift IR
    /// [ConstantPool](crate::ir::constant::ConstantPool).
    Pool(Constant, ConstantData),
    /// A reference to a well-known constant value that is statically encoded within the compiler.
    WellKnown(&'static [u8]),
    /// A constant value generated during lowering; the value may depend on the instruction context
    /// which makes it difficult to de-duplicate--if possible, use other variants.
    Generated(ConstantData),
    /// A constant of at most 64 bits. These are deduplicated as
    /// well. Stored as a fixed-size array of `u8` so that we do not
    /// encounter endianness problems when cross-compiling.
    U64([u8; 8]),
}
impl VCodeConstantData {
    /// Retrieve the constant data as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            VCodeConstantData::Pool(_, d) | VCodeConstantData::Generated(d) => d.as_slice(),
            VCodeConstantData::WellKnown(d) => d,
            VCodeConstantData::U64(value) => &value[..],
        }
    }

    /// Calculate the alignment of the constant data.
    pub fn alignment(&self) -> u32 {
        if self.as_slice().len() <= 8 {
            8
        } else {
            16
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn size_of_constant_structs() {
        assert_eq!(size_of::<Constant>(), 4);
        assert_eq!(size_of::<VCodeConstant>(), 4);
        assert_eq!(size_of::<ConstantData>(), 24);
        assert_eq!(size_of::<VCodeConstantData>(), 32);
        assert_eq!(
            size_of::<PrimaryMap<VCodeConstant, VCodeConstantData>>(),
            24
        );
        // TODO The VCodeConstants structure's memory size could be further optimized.
        // With certain versions of Rust, each `HashMap` in `VCodeConstants` occupied at
        // least 48 bytes, making an empty `VCodeConstants` cost 120 bytes.
    }
}
