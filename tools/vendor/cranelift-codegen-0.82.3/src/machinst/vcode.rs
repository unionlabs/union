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
use crate::ir::{self, types, Constant, ConstantData, SourceLoc};
use crate::machinst::*;
use crate::settings;
use crate::timing;
use regalloc::Function as RegallocFunction;
use regalloc::Set as RegallocSet;
use regalloc::{
    BlockIx, InstIx, PrettyPrint, Range, RegAllocResult, RegClass, RegUsageCollector,
    RegUsageMapper, SpillSlot, StackmapRequestInfo,
};

use alloc::boxed::Box;
use alloc::{borrow::Cow, vec::Vec};
use cranelift_entity::{entity_impl, Keys, PrimaryMap};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::iter;
use std::string::String;

/// Index referring to an instruction in VCode.
pub type InsnIndex = u32;
/// Index referring to a basic block in VCode.
pub type BlockIndex = u32;

/// VCodeInst wraps all requirements for a MachInst to be in VCode: it must be
/// a `MachInst` and it must be able to emit itself at least to a `SizeCodeSink`.
pub trait VCodeInst: MachInst + MachInstEmit {}
impl<I: MachInst + MachInstEmit> VCodeInst for I {}

/// A function in "VCode" (virtualized-register code) form, after lowering.
/// This is essentially a standard CFG of basic blocks, where each basic block
/// consists of lowered instructions produced by the machine-specific backend.
pub struct VCode<I: VCodeInst> {
    /// Function liveins.
    liveins: RegallocSet<RealReg>,

    /// Function liveouts.
    liveouts: RegallocSet<RealReg>,

    /// VReg IR-level types.
    vreg_types: Vec<Type>,

    /// Do we have any ref values among our vregs?
    have_ref_values: bool,

    /// Lowered machine instructions in order corresponding to the original IR.
    insts: Vec<I>,

    /// Source locations for each instruction. (`SourceLoc` is a `u32`, so it is
    /// reasonable to keep one of these per instruction.)
    srclocs: Vec<SourceLoc>,

    /// Entry block.
    entry: BlockIndex,

    /// Block instruction indices.
    block_ranges: Vec<(InsnIndex, InsnIndex)>,

    /// Block successors: index range in the successor-list below.
    block_succ_range: Vec<(usize, usize)>,

    /// Block successor lists, concatenated into one Vec. The `block_succ_range`
    /// list of tuples above gives (start, end) ranges within this list that
    /// correspond to each basic block's successors.
    block_succs: Vec<BlockIx>,

    /// Block-order information.
    block_order: BlockLoweringOrder,

    /// ABI object.
    abi: Box<dyn ABICallee<I = I>>,

    /// Constant information used during code emission. This should be
    /// immutable across function compilations within the same module.
    emit_info: I::Info,

    /// Safepoint instruction indices. Filled in post-regalloc. (Prior to
    /// regalloc, the safepoint instructions are listed in the separate
    /// `StackmapRequestInfo` held separate from the `VCode`.)
    safepoint_insns: Vec<InsnIndex>,

    /// For each safepoint entry in `safepoint_insns`, a list of `SpillSlot`s.
    /// These are used to generate actual stack maps at emission. Filled in
    /// post-regalloc.
    safepoint_slots: Vec<Vec<SpillSlot>>,

    /// Do we generate debug info?
    generate_debug_info: bool,

    /// Instruction end offsets, instruction indices at each label,
    /// total buffer size, and start of cold code.  Only present if
    /// `generate_debug_info` is set.
    insts_layout: RefCell<InstsLayoutInfo>,

    /// Constants.
    constants: VCodeConstants,

    /// Are any debug value-labels present? If not, we can skip the
    /// post-emission analysis.
    has_value_labels: bool,
}

#[derive(Debug, Default)]
pub(crate) struct InstsLayoutInfo {
    pub(crate) inst_end_offsets: Vec<CodeOffset>,
    pub(crate) label_inst_indices: Vec<CodeOffset>,
    pub(crate) start_of_cold_code: Option<CodeOffset>,
}

/// A builder for a VCode function body. This builder is designed for the
/// lowering approach that we take: we traverse basic blocks in forward
/// (original IR) order, but within each basic block, we generate code from
/// bottom to top; and within each IR instruction that we visit in this reverse
/// order, we emit machine instructions in *forward* order again.
///
/// Hence, to produce the final instructions in proper order, we perform two
/// swaps.  First, the machine instructions (`I` instances) are produced in
/// forward order for an individual IR instruction. Then these are *reversed*
/// and concatenated to `bb_insns` at the end of the IR instruction lowering.
/// The `bb_insns` vec will thus contain all machine instructions for a basic
/// block, in reverse order. Finally, when we're done with a basic block, we
/// reverse the whole block's vec of instructions again, and concatenate onto
/// the VCode's insts.
pub struct VCodeBuilder<I: VCodeInst> {
    /// In-progress VCode.
    vcode: VCode<I>,

    /// In-progress stack map-request info.
    stack_map_info: StackmapRequestInfo,

    /// Index of the last block-start in the vcode.
    block_start: InsnIndex,

    /// Start of succs for the current block in the concatenated succs list.
    succ_start: usize,

    /// Current source location.
    cur_srcloc: SourceLoc,
}

impl<I: VCodeInst> VCodeBuilder<I> {
    /// Create a new VCodeBuilder.
    pub fn new(
        abi: Box<dyn ABICallee<I = I>>,
        emit_info: I::Info,
        block_order: BlockLoweringOrder,
        constants: VCodeConstants,
    ) -> VCodeBuilder<I> {
        let reftype_class = I::ref_type_regclass(abi.flags());
        let vcode = VCode::new(
            abi,
            emit_info,
            block_order,
            constants,
            /* generate_debug_info = */ true,
        );
        let stack_map_info = StackmapRequestInfo {
            reftype_class,
            reftyped_vregs: vec![],
            safepoint_insns: vec![],
        };

        VCodeBuilder {
            vcode,
            stack_map_info,
            block_start: 0,
            succ_start: 0,
            cur_srcloc: SourceLoc::default(),
        }
    }

    /// Access the ABI object.
    pub fn abi(&mut self) -> &mut dyn ABICallee<I = I> {
        &mut *self.vcode.abi
    }

    /// Access to the BlockLoweringOrder object.
    pub fn block_order(&self) -> &BlockLoweringOrder {
        &self.vcode.block_order
    }

    /// Set the type of a VReg.
    pub fn set_vreg_type(&mut self, vreg: VirtualReg, ty: Type) {
        if self.vcode.vreg_types.len() <= vreg.get_index() {
            self.vcode
                .vreg_types
                .resize(vreg.get_index() + 1, ir::types::I8);
        }
        self.vcode.vreg_types[vreg.get_index()] = ty;
        if is_reftype(ty) {
            self.stack_map_info.reftyped_vregs.push(vreg);
            self.vcode.have_ref_values = true;
        }
    }

    /// Set the current block as the entry block.
    pub fn set_entry(&mut self, block: BlockIndex) {
        self.vcode.entry = block;
    }

    /// End the current basic block. Must be called after emitting vcode insts
    /// for IR insts and prior to ending the function (building the VCode).
    pub fn end_bb(&mut self) {
        let start_idx = self.block_start;
        let end_idx = self.vcode.insts.len() as InsnIndex;
        self.block_start = end_idx;
        // Add the instruction index range to the list of blocks.
        self.vcode.block_ranges.push((start_idx, end_idx));
        // End the successors list.
        let succ_end = self.vcode.block_succs.len();
        self.vcode
            .block_succ_range
            .push((self.succ_start, succ_end));
        self.succ_start = succ_end;
    }

    /// Push an instruction for the current BB and current IR inst within the BB.
    pub fn push(&mut self, insn: I, is_safepoint: bool) {
        match insn.is_term() {
            MachTerminator::None | MachTerminator::Ret => {}
            MachTerminator::Uncond(target) => {
                self.vcode.block_succs.push(BlockIx::new(target.get()));
            }
            MachTerminator::Cond(true_branch, false_branch) => {
                self.vcode.block_succs.push(BlockIx::new(true_branch.get()));
                self.vcode
                    .block_succs
                    .push(BlockIx::new(false_branch.get()));
            }
            MachTerminator::Indirect(targets) => {
                for target in targets {
                    self.vcode.block_succs.push(BlockIx::new(target.get()));
                }
            }
        }
        if insn.defines_value_label().is_some() {
            self.vcode.has_value_labels = true;
        }
        self.vcode.insts.push(insn);
        self.vcode.srclocs.push(self.cur_srcloc);
        if is_safepoint {
            self.stack_map_info
                .safepoint_insns
                .push(InstIx::new((self.vcode.insts.len() - 1) as u32));
        }
    }

    /// Set the current source location.
    pub fn set_srcloc(&mut self, srcloc: SourceLoc) {
        self.cur_srcloc = srcloc;
    }

    /// Access the constants.
    pub fn constants(&mut self) -> &mut VCodeConstants {
        &mut self.vcode.constants
    }

    /// Build the final VCode, returning the vcode itself as well as auxiliary
    /// information, such as the stack map request information.
    pub fn build(self) -> (VCode<I>, StackmapRequestInfo) {
        // TODO: come up with an abstraction for "vcode and auxiliary data". The
        // auxiliary data needs to be separate from the vcode so that it can be
        // referenced as the vcode is mutated (e.g. by the register allocator).
        (self.vcode, self.stack_map_info)
    }
}

fn is_redundant_move<I: VCodeInst>(insn: &I) -> bool {
    if let Some((to, from)) = insn.is_move() {
        to.to_reg() == from
    } else {
        false
    }
}

/// Is this type a reference type?
fn is_reftype(ty: Type) -> bool {
    ty == types::R64 || ty == types::R32
}

impl<I: VCodeInst> VCode<I> {
    /// New empty VCode.
    fn new(
        abi: Box<dyn ABICallee<I = I>>,
        emit_info: I::Info,
        block_order: BlockLoweringOrder,
        constants: VCodeConstants,
        generate_debug_info: bool,
    ) -> VCode<I> {
        VCode {
            liveins: abi.liveins(),
            liveouts: abi.liveouts(),
            vreg_types: vec![],
            have_ref_values: false,
            insts: vec![],
            srclocs: vec![],
            entry: 0,
            block_ranges: vec![],
            block_succ_range: vec![],
            block_succs: vec![],
            block_order,
            abi,
            emit_info,
            safepoint_insns: vec![],
            safepoint_slots: vec![],
            generate_debug_info,
            insts_layout: RefCell::new(Default::default()),
            constants,
            has_value_labels: false,
        }
    }

    /// Returns the flags controlling this function's compilation.
    pub fn flags(&self) -> &settings::Flags {
        self.abi.flags()
    }

    /// Get the IR-level type of a VReg.
    pub fn vreg_type(&self, vreg: VirtualReg) -> Type {
        self.vreg_types[vreg.get_index()]
    }

    /// Get the number of blocks. Block indices will be in the range `0 ..
    /// (self.num_blocks() - 1)`.
    pub fn num_blocks(&self) -> usize {
        self.block_ranges.len()
    }

    /// Stack frame size for the full function's body.
    pub fn frame_size(&self) -> u32 {
        self.abi.frame_size()
    }

    /// Get the successors for a block.
    pub fn succs(&self, block: BlockIndex) -> &[BlockIx] {
        let (start, end) = self.block_succ_range[block as usize];
        &self.block_succs[start..end]
    }

    /// Take the results of register allocation, with a sequence of
    /// instructions including spliced fill/reload/move instructions, and replace
    /// the VCode with them.
    pub fn replace_insns_from_regalloc(&mut self, result: RegAllocResult<Self>) {
        // Record the spillslot count and clobbered registers for the ABI/stack
        // setup code.
        self.abi.set_num_spillslots(result.num_spill_slots as usize);
        self.abi
            .set_clobbered(result.clobbered_registers.map(|r| Writable::from_reg(*r)));

        let mut final_insns = vec![];
        let mut final_block_ranges = vec![(0, 0); self.num_blocks()];
        let mut final_srclocs = vec![];
        let mut final_safepoint_insns = vec![];
        let mut safept_idx = 0;

        assert!(result.target_map.elems().len() == self.num_blocks());
        for block in 0..self.num_blocks() {
            let start = result.target_map.elems()[block].get() as usize;
            let end = if block == self.num_blocks() - 1 {
                result.insns.len()
            } else {
                result.target_map.elems()[block + 1].get() as usize
            };
            let block = block as BlockIndex;
            let final_start = final_insns.len() as InsnIndex;

            if block == self.entry {
                // Start with the prologue.
                let prologue = self.abi.gen_prologue();
                let len = prologue.len();
                final_insns.extend(prologue.into_iter());
                final_srclocs.extend(iter::repeat(SourceLoc::default()).take(len));
            }

            for i in start..end {
                let insn = &result.insns[i];

                // Elide redundant moves at this point (we only know what is
                // redundant once registers are allocated).
                if is_redundant_move(insn) {
                    continue;
                }

                // Is there a srcloc associated with this insn? Look it up based on original
                // instruction index (if new insn corresponds to some original insn, i.e., is not
                // an inserted load/spill/move).
                let orig_iix = result.orig_insn_map[InstIx::new(i as u32)];
                let srcloc = if orig_iix.is_invalid() {
                    SourceLoc::default()
                } else {
                    self.srclocs[orig_iix.get() as usize]
                };

                // Whenever encountering a return instruction, replace it
                // with the epilogue.
                let is_ret = insn.is_term() == MachTerminator::Ret;
                if is_ret {
                    let epilogue = self.abi.gen_epilogue();
                    let len = epilogue.len();
                    final_insns.extend(epilogue.into_iter());
                    final_srclocs.extend(iter::repeat(srcloc).take(len));
                } else {
                    final_insns.push(insn.clone());
                    final_srclocs.push(srcloc);
                }

                // Was this instruction a safepoint instruction? Add its final
                // index to the safepoint insn-index list if so.
                if safept_idx < result.new_safepoint_insns.len()
                    && (result.new_safepoint_insns[safept_idx].get() as usize) == i
                {
                    let idx = final_insns.len() - 1;
                    final_safepoint_insns.push(idx as InsnIndex);
                    safept_idx += 1;
                }
            }

            let final_end = final_insns.len() as InsnIndex;
            final_block_ranges[block as usize] = (final_start, final_end);
        }

        debug_assert!(final_insns.len() == final_srclocs.len());

        self.insts = final_insns;
        self.srclocs = final_srclocs;
        self.block_ranges = final_block_ranges;
        self.safepoint_insns = final_safepoint_insns;

        // Save safepoint slot-lists. These will be passed to the `EmitState`
        // for the machine backend during emission so that it can do
        // target-specific translations of slot numbers to stack offsets.
        self.safepoint_slots = result.stackmaps;
    }

    /// Emit the instructions to a `MachBuffer`, containing fixed-up code and external
    /// reloc/trap/etc. records ready for use.
    pub fn emit(
        &self,
    ) -> (
        MachBuffer<I>,
        Vec<CodeOffset>,
        Vec<(CodeOffset, CodeOffset)>,
    )
    where
        I: MachInstEmit,
    {
        let _tt = timing::vcode_emit();
        let mut buffer = MachBuffer::new();
        let mut state = I::State::new(&*self.abi);
        let cfg_metadata = self.flags().machine_code_cfg_info();
        let mut bb_starts: Vec<Option<CodeOffset>> = vec![];

        // The first M MachLabels are reserved for block indices, the next N MachLabels for
        // constants.
        buffer.reserve_labels_for_blocks(self.num_blocks() as BlockIndex);
        buffer.reserve_labels_for_constants(&self.constants);

        let mut inst_end_offsets = vec![0; self.insts.len()];
        let mut label_inst_indices = vec![0; self.num_blocks()];

        // Map from instruction index to index in
        // `safepoint_slots`. We need this because we emit
        // instructions out-of-order, while the safepoint_insns /
        // safepoint_slots data structures are sorted in instruction
        // order.
        let mut safepoint_indices: FxHashMap<u32, usize> = FxHashMap::default();
        for (safepoint_idx, iix) in self.safepoint_insns.iter().enumerate() {
            // Disregard safepoints that ended up having no live refs.
            if self.safepoint_slots[safepoint_idx].len() > 0 {
                safepoint_indices.insert(*iix, safepoint_idx);
            }
        }

        // Construct the final order we emit code in: cold blocks at the end.
        let mut final_order: SmallVec<[BlockIndex; 16]> = smallvec![];
        let mut cold_blocks: SmallVec<[BlockIndex; 16]> = smallvec![];
        for block in 0..self.num_blocks() {
            let block = block as BlockIndex;
            if self.block_order.is_cold(block) {
                cold_blocks.push(block);
            } else {
                final_order.push(block);
            }
        }
        let first_cold_block = cold_blocks.first().cloned();
        final_order.extend(cold_blocks.clone());

        // Emit blocks.
        let mut cur_srcloc = None;
        let mut last_offset = None;
        let mut start_of_cold_code = None;
        for block in final_order {
            let new_offset = I::align_basic_block(buffer.cur_offset());
            while new_offset > buffer.cur_offset() {
                // Pad with NOPs up to the aligned block offset.
                let nop = I::gen_nop((new_offset - buffer.cur_offset()) as usize);
                nop.emit(&mut buffer, &self.emit_info, &mut Default::default());
            }
            assert_eq!(buffer.cur_offset(), new_offset);

            if Some(block) == first_cold_block {
                start_of_cold_code = Some(buffer.cur_offset());
            }

            let (start, end) = self.block_ranges[block as usize];
            buffer.bind_label(MachLabel::from_block(block));
            label_inst_indices[block as usize] = start;

            if cfg_metadata {
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

            for iix in start..end {
                let srcloc = self.srclocs[iix as usize];
                if cur_srcloc != Some(srcloc) {
                    if cur_srcloc.is_some() {
                        buffer.end_srcloc();
                    }
                    buffer.start_srcloc(srcloc);
                    cur_srcloc = Some(srcloc);
                }
                state.pre_sourceloc(cur_srcloc.unwrap_or(SourceLoc::default()));

                if let Some(safepoint_idx) = safepoint_indices.get(&iix) {
                    let stack_map = self
                        .abi
                        .spillslots_to_stack_map(&self.safepoint_slots[*safepoint_idx][..], &state);
                    state.pre_safepoint(stack_map);
                }

                self.insts[iix as usize].emit(&mut buffer, &self.emit_info, &mut state);

                if self.generate_debug_info {
                    // Buffer truncation may have happened since last inst append; trim inst-end
                    // layout info as appropriate.
                    let l = &mut inst_end_offsets[0..iix as usize];
                    for end in l.iter_mut().rev() {
                        if *end > buffer.cur_offset() {
                            *end = buffer.cur_offset();
                        } else {
                            break;
                        }
                    }
                    inst_end_offsets[iix as usize] = buffer.cur_offset();
                }
            }

            if cur_srcloc.is_some() {
                buffer.end_srcloc();
                cur_srcloc = None;
            }

            // Do we need an island? Get the worst-case size of the next BB and see if, having
            // emitted that many bytes, we will be beyond the deadline.
            if block < (self.num_blocks() - 1) as BlockIndex {
                let next_block = block + 1;
                let next_block_range = self.block_ranges[next_block as usize];
                let next_block_size = next_block_range.1 - next_block_range.0;
                let worst_case_next_bb = I::worst_case_size() * next_block_size;
                if buffer.island_needed(worst_case_next_bb) {
                    buffer.emit_island(worst_case_next_bb);
                }
            }
        }

        // Emit the constants used by the function.
        for (constant, data) in self.constants.iter() {
            let label = buffer.get_label_for_constant(constant);
            buffer.defer_constant(label, data.alignment(), data.as_slice(), u32::max_value());
        }

        if self.generate_debug_info {
            for end in inst_end_offsets.iter_mut().rev() {
                if *end > buffer.cur_offset() {
                    *end = buffer.cur_offset();
                } else {
                    break;
                }
            }
            *self.insts_layout.borrow_mut() = InstsLayoutInfo {
                inst_end_offsets,
                label_inst_indices,
                start_of_cold_code,
            };
        }

        // Create `bb_edges` and final (filtered) `bb_starts`.
        let mut final_bb_starts = vec![];
        let mut bb_edges = vec![];
        if cfg_metadata {
            for block in 0..self.num_blocks() {
                if bb_starts[block].is_none() {
                    // Block was deleted by MachBuffer; skip.
                    continue;
                }
                let from = bb_starts[block].unwrap();

                final_bb_starts.push(from);
                // Resolve each `succ` label and add edges.
                let succs = self.block_succs(BlockIx::new(block as u32));
                for succ in succs.iter() {
                    let to = buffer.resolve_label_offset(MachLabel::from_block(succ.get()));
                    bb_edges.push((from, to));
                }
            }
        }

        (buffer, final_bb_starts, bb_edges)
    }

    /// Generates value-label ranges.
    pub fn value_labels_ranges(&self) -> ValueLabelsRanges {
        if !self.has_value_labels {
            return ValueLabelsRanges::default();
        }

        let layout_info = &self.insts_layout.borrow();
        debug::compute(&self.insts, &*layout_info)
    }

    /// Get the offsets of stackslots.
    pub fn stackslot_offsets(&self) -> &PrimaryMap<StackSlot, u32> {
        self.abi.stackslot_offsets()
    }

    /// Get the IR block for a BlockIndex, if one exists.
    pub fn bindex_to_bb(&self, block: BlockIndex) -> Option<ir::Block> {
        self.block_order.lowered_order()[block as usize].orig_block()
    }
}

impl<I: VCodeInst> RegallocFunction for VCode<I> {
    type Inst = I;

    fn insns(&self) -> &[I] {
        &self.insts[..]
    }

    fn insns_mut(&mut self) -> &mut [I] {
        &mut self.insts[..]
    }

    fn get_insn(&self, insn: InstIx) -> &I {
        &self.insts[insn.get() as usize]
    }

    fn get_insn_mut(&mut self, insn: InstIx) -> &mut I {
        &mut self.insts[insn.get() as usize]
    }

    fn blocks(&self) -> Range<BlockIx> {
        Range::new(BlockIx::new(0), self.block_ranges.len())
    }

    fn entry_block(&self) -> BlockIx {
        BlockIx::new(self.entry)
    }

    fn block_insns(&self, block: BlockIx) -> Range<InstIx> {
        let (start, end) = self.block_ranges[block.get() as usize];
        Range::new(InstIx::new(start), (end - start) as usize)
    }

    fn block_succs(&self, block: BlockIx) -> Cow<[BlockIx]> {
        let (start, end) = self.block_succ_range[block.get() as usize];
        Cow::Borrowed(&self.block_succs[start..end])
    }

    fn is_ret(&self, insn: InstIx) -> bool {
        match self.insts[insn.get() as usize].is_term() {
            MachTerminator::Ret => true,
            _ => false,
        }
    }

    fn is_included_in_clobbers(&self, insn: &I) -> bool {
        insn.is_included_in_clobbers()
    }

    fn get_regs(insn: &I, collector: &mut RegUsageCollector) {
        insn.get_regs(collector)
    }

    fn map_regs<RUM: RegUsageMapper>(insn: &mut I, mapper: &RUM) {
        insn.map_regs(mapper);
    }

    fn is_move(&self, insn: &I) -> Option<(Writable<Reg>, Reg)> {
        insn.is_move()
    }

    fn get_num_vregs(&self) -> usize {
        self.vreg_types.len()
    }

    fn get_spillslot_size(&self, regclass: RegClass, _: VirtualReg) -> u32 {
        self.abi.get_spillslot_size(regclass)
    }

    fn gen_spill(&self, to_slot: SpillSlot, from_reg: RealReg, _: Option<VirtualReg>) -> I {
        self.abi.gen_spill(to_slot, from_reg)
    }

    fn gen_reload(
        &self,
        to_reg: Writable<RealReg>,
        from_slot: SpillSlot,
        _: Option<VirtualReg>,
    ) -> I {
        self.abi.gen_reload(to_reg, from_slot)
    }

    fn gen_move(&self, to_reg: Writable<RealReg>, from_reg: RealReg, vreg: VirtualReg) -> I {
        let ty = self.vreg_type(vreg);
        I::gen_move(to_reg.map(|r| r.to_reg()), from_reg.to_reg(), ty)
    }

    fn gen_zero_len_nop(&self) -> I {
        I::gen_nop(0)
    }

    fn maybe_direct_reload(&self, insn: &I, reg: VirtualReg, slot: SpillSlot) -> Option<I> {
        insn.maybe_direct_reload(reg, slot)
    }

    fn func_liveins(&self) -> RegallocSet<RealReg> {
        self.liveins.clone()
    }

    fn func_liveouts(&self) -> RegallocSet<RealReg> {
        self.liveouts.clone()
    }
}

impl<I: VCodeInst> fmt::Debug for VCode<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "VCode_Debug {{")?;
        writeln!(f, "  Entry block: {}", self.entry)?;

        for block in 0..self.num_blocks() {
            writeln!(f, "Block {}:", block,)?;
            for succ in self.succs(block as BlockIndex) {
                writeln!(f, "  (successor: Block {})", succ.get())?;
            }
            let (start, end) = self.block_ranges[block];
            writeln!(f, "  (instruction range: {} .. {})", start, end)?;
            for inst in start..end {
                writeln!(f, "  Inst {}: {:?}", inst, self.insts[inst as usize])?;
            }
        }

        writeln!(f, "}}")?;
        Ok(())
    }
}

/// Pretty-printing with `RealRegUniverse` context.
impl<I: VCodeInst> PrettyPrint for VCode<I> {
    fn show_rru(&self, mb_rru: Option<&RealRegUniverse>) -> String {
        use std::fmt::Write;

        let mut s = String::new();
        write!(&mut s, "VCode_ShowWithRRU {{{{\n").unwrap();
        write!(&mut s, "  Entry block: {}\n", self.entry).unwrap();

        let mut state = Default::default();
        let mut safepoint_idx = 0;
        for i in 0..self.num_blocks() {
            let block = i as BlockIndex;

            write!(&mut s, "Block {}:\n", block).unwrap();
            if let Some(bb) = self.bindex_to_bb(block) {
                write!(&mut s, "  (original IR block: {})\n", bb).unwrap();
            }
            for succ in self.succs(block) {
                write!(&mut s, "  (successor: Block {})\n", succ.get()).unwrap();
            }
            let (start, end) = self.block_ranges[block as usize];
            write!(&mut s, "  (instruction range: {} .. {})\n", start, end).unwrap();
            for inst in start..end {
                if safepoint_idx < self.safepoint_insns.len()
                    && self.safepoint_insns[safepoint_idx] == inst
                {
                    write!(
                        &mut s,
                        "      (safepoint: slots {:?} with EmitState {:?})\n",
                        self.safepoint_slots[safepoint_idx], state,
                    )
                    .unwrap();
                    safepoint_idx += 1;
                }
                write!(
                    &mut s,
                    "  Inst {}:   {}\n",
                    inst,
                    self.insts[inst as usize].pretty_print(mb_rru, &mut state)
                )
                .unwrap();
            }
        }

        write!(&mut s, "}}}}\n").unwrap();

        s
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
}
impl VCodeConstants {
    /// Initialize the structure with the expected number of constants.
    pub fn with_capacity(expected_num_constants: usize) -> Self {
        Self {
            constants: PrimaryMap::with_capacity(expected_num_constants),
            pool_uses: HashMap::with_capacity(expected_num_constants),
            well_known_uses: HashMap::new(),
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
                match self.well_known_uses.get(&(data_ref as *const [u8])) {
                    None => {
                        let vcode_constant = self.constants.push(data);
                        self.well_known_uses
                            .insert(data_ref as *const [u8], vcode_constant);
                        vcode_constant
                    }
                    Some(&vcode_constant) => vcode_constant,
                }
            }
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
}
impl VCodeConstantData {
    /// Retrieve the constant data as a byte slice.
    pub fn as_slice(&self) -> &[u8] {
        match self {
            VCodeConstantData::Pool(_, d) | VCodeConstantData::Generated(d) => d.as_slice(),
            VCodeConstantData::WellKnown(d) => d,
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
