use crate::checker::{CheckerContext, CheckerErrors};
use crate::data_structures::{
    BlockIx, InstIx, InstPoint, Point, RangeFrag, RealReg, RealRegUniverse, Reg, SpillSlot,
    TypedIxVec, VirtualReg, Writable,
};
use crate::{
    checker::{CheckerStackmapInfo, Inst as CheckerInst},
    StackmapRequestInfo,
};
use crate::{reg_maps::VrangeRegUsageMapper, Function, RegAllocError};
use log::trace;

use std::result::Result;

//=============================================================================
// InstToInsert and InstToInsertAndPoint

#[derive(Clone, Debug)]
pub(crate) enum InstToInsert {
    Spill {
        to_slot: SpillSlot,
        from_reg: RealReg,
        for_vreg: Option<VirtualReg>,
    },
    Reload {
        to_reg: Writable<RealReg>,
        from_slot: SpillSlot,
        for_vreg: Option<VirtualReg>,
    },
    Move {
        to_reg: Writable<RealReg>,
        from_reg: RealReg,
        for_vreg: VirtualReg,
    },
    /// A spillslot reassignment (to another vreg). In the edited instruction
    /// stream, this is a nop, but this is needed for the checker to properly
    /// track the symbolic values in slots. Always originates from a move
    /// in the original user program whose source and dest vregs are both
    /// spilled.
    ChangeSpillSlotOwnership {
        inst_ix: InstIx,
        slot: SpillSlot,
        from_reg: Reg,
        to_reg: Reg,
    },
}

impl InstToInsert {
    pub(crate) fn construct<F: Function>(&self, f: &F) -> Option<F::Inst> {
        match self {
            &InstToInsert::Spill {
                to_slot,
                from_reg,
                for_vreg,
            } => Some(f.gen_spill(to_slot, from_reg, for_vreg)),
            &InstToInsert::Reload {
                to_reg,
                from_slot,
                for_vreg,
            } => Some(f.gen_reload(to_reg, from_slot, for_vreg)),
            &InstToInsert::Move {
                to_reg,
                from_reg,
                for_vreg,
            } => Some(f.gen_move(to_reg, from_reg, for_vreg)),
            &InstToInsert::ChangeSpillSlotOwnership { .. } => None,
        }
    }

    pub(crate) fn to_checker_inst(&self) -> CheckerInst {
        match self {
            &InstToInsert::Spill {
                to_slot, from_reg, ..
            } => CheckerInst::Spill {
                into: to_slot,
                from: from_reg,
            },
            &InstToInsert::Reload {
                to_reg, from_slot, ..
            } => CheckerInst::Reload {
                into: to_reg,
                from: from_slot,
            },
            &InstToInsert::Move {
                to_reg, from_reg, ..
            } => CheckerInst::Move {
                into: to_reg,
                from: from_reg,
            },
            &InstToInsert::ChangeSpillSlotOwnership {
                inst_ix,
                slot,
                from_reg,
                to_reg,
            } => CheckerInst::ChangeSpillSlotOwnership {
                inst_ix,
                slot,
                from_reg,
                to_reg,
            },
        }
    }
}

// ExtPoint is an extended version of Point.  It plays no role in dataflow analysis or in the
// specification of live ranges.  It exists only to describe where to place the "extra"
// spill/reload instructions required to make stackmap/reftype support work.  If there was no
// need to support stackmaps/reftypes, ExtPoint would not be needed, and Point would be
// adequate.
//
// Recall that Point can denote 4 places within an instruction, with R < U < D < S:
//
// * R(eload): this is where any reload insns for the insn itself are
//   considered to live.
//
// * U(se): this is where the insn is considered to use values from those of
//   its register operands that appear in a Read or Modify role.
//
// * D(ef): this is where the insn is considered to define new values for
//   those of its register operands that appear in a Write or Modify role.
//
// * S(pill): this is where any spill insns for the insn itself are considered
//   to live.
//
// ExtPoint extends that to six places, by adding a new point in between Reload and Use, and one
// between Def and Spill, giving: R < SB < U < D < RA < S:
//
// * (R)eload: unchanged
//
// * SB (Spill before): at this point, reftyped regs will be spilled, if this insn is a safepoint
//
// * (U)se: unchanged
//
// * (D)ef: unchanged
//
// * RA (Reload after): at this point, reftyped regs spilled at SB will be reloaded, if needed,
//   and if this insn is a safepoint
//
// * (S)pill: unchanged
//
// From this it can be seen that the SB and RA points are closest to the instruction "core" --
// the U and D points.  SB and RA describe places where reftyped regs must be spilled/reloaded
// around the core.  Because the SB-RA range falls inside the R-S range, it means the the
// safepoint spill/reload instructions can be added after "normal" spill/reload instructions
// have been created, and it doesn't interact with the logic to create those "normal"
// spill/reload instructions.
//
// In the worst case scenario, a value could be reloaded at R, immediately spilled at SB, then
// possibly modified in memory at the safepoint proper, reloaded at RA, and spilled at S.  That
// is considered to be an unlikely scenario, though.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtPoint {
    Reload = 0,
    SpillBefore = 1,
    Use = 2,
    Def = 3,
    ReloadAfter = 4,
    Spill = 5,
}

impl ExtPoint {
    // Promote a Point to an ExtPoint
    #[inline(always)]
    pub fn from_point(pt: Point) -> Self {
        match pt {
            Point::Reload => ExtPoint::Reload,
            Point::Use => ExtPoint::Use,
            Point::Def => ExtPoint::Def,
            Point::Spill => ExtPoint::Spill,
        }
    }
}

// As the direct analogy to InstPoint, a InstExtPoint pairs an InstIx with an ExtPoint.  In
// contrast to InstPoint, these aren't so performance critical, so there's no fancy bit-packed
// representation as there is for InstPoint.

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstExtPoint {
    pub iix: InstIx,
    pub extpt: ExtPoint,
}

impl InstExtPoint {
    #[inline(always)]
    pub fn new(iix: InstIx, extpt: ExtPoint) -> Self {
        Self { iix, extpt }
    }
    // Promote an InstPoint to an InstExtPoint
    #[inline(always)]
    pub fn from_inst_point(inst_pt: InstPoint) -> Self {
        InstExtPoint {
            iix: inst_pt.iix(),
            extpt: ExtPoint::from_point(inst_pt.pt()),
        }
    }
}

// So, finally, we can specify what we want: an instruction to insert, and a place to insert it.
#[derive(Debug)]
pub(crate) struct InstToInsertAndExtPoint {
    pub(crate) inst: InstToInsert,
    pub(crate) iep: InstExtPoint,
}

impl InstToInsertAndExtPoint {
    #[inline(always)]
    pub(crate) fn new(inst: InstToInsert, iep: InstExtPoint) -> Self {
        Self { inst, iep }
    }
}

//=============================================================================
// Apply all vreg->rreg mappings for the function's instructions, and run
// the checker if required.  This also removes instructions that the core
// algorithm wants removed, by nop-ing them out.

#[inline(never)]
fn map_vregs_to_rregs<F: Function>(
    func: &mut F,
    frag_map: Vec<(RangeFrag, VirtualReg, RealReg)>,
    insts_to_add: &Vec<InstToInsertAndExtPoint>,
    iixs_to_nop_out: &Vec<InstIx>,
    reg_universe: &RealRegUniverse,
    use_checker: bool,
    stackmap_request: Option<&StackmapRequestInfo>,
    stackmaps: &[Vec<SpillSlot>],
) -> Result<(), CheckerErrors> {
    // Set up checker state, if indicated by our configuration.
    let mut checker: Option<CheckerContext> = None;
    let mut insn_blocks: Vec<BlockIx> = vec![];
    if use_checker {
        let stackmap_info =
            stackmap_request.map(|request| CheckerStackmapInfo { request, stackmaps });
        checker = Some(CheckerContext::new(
            func,
            reg_universe,
            insts_to_add,
            stackmap_info,
        ));
        insn_blocks.resize(func.insns().len(), BlockIx::new(0));
        for block_ix in func.blocks() {
            for insn_ix in func.block_insns(block_ix) {
                insn_blocks[insn_ix.get() as usize] = block_ix;
            }
        }
    }

    // Sort the insn nop-out index list, so we can advance through it
    // during the main loop.
    let mut iixs_to_nop_out = iixs_to_nop_out.clone();
    iixs_to_nop_out.sort();

    // Make two copies of the fragment mapping, one sorted by the fragment start
    // points (just the InstIx numbers, ignoring the Point), and one sorted by
    // fragment end points.
    let mut frag_maps_by_start = frag_map.clone();
    let mut frag_maps_by_end = frag_map;

    // -------- Edit the instruction stream --------
    frag_maps_by_start.sort_unstable_by(|(frag, _, _), (other_frag, _, _)| {
        frag.first
            .iix()
            .partial_cmp(&other_frag.first.iix())
            .unwrap()
    });

    frag_maps_by_end.sort_unstable_by(|(frag, _, _), (other_frag, _, _)| {
        frag.last.iix().partial_cmp(&other_frag.last.iix()).unwrap()
    });

    let mut cursor_starts = 0;
    let mut cursor_ends = 0;
    let mut cursor_nop = 0;

    // Allocate the "mapper" data structure that we update incrementally and
    // pass to instruction reg-mapping routines to query.
    let mut mapper = VrangeRegUsageMapper::new(func.get_num_vregs());

    fn is_sane(frag: &RangeFrag) -> bool {
        // "Normal" frag (unrelated to spilling).  No normal frag may start or
        // end at a .s or a .r point.
        if frag.first.pt().is_use_or_def()
            && frag.last.pt().is_use_or_def()
            && frag.first.iix() <= frag.last.iix()
        {
            return true;
        }
        // A spill-related ("bridge") frag.  There are three possibilities,
        // and they correspond exactly to `BridgeKind`.
        if frag.first.pt().is_reload()
            && frag.last.pt().is_use()
            && frag.last.iix() == frag.first.iix()
        {
            // BridgeKind::RtoU
            return true;
        }
        if frag.first.pt().is_reload()
            && frag.last.pt().is_spill()
            && frag.last.iix() == frag.first.iix()
        {
            // BridgeKind::RtoS
            return true;
        }
        if frag.first.pt().is_def()
            && frag.last.pt().is_spill()
            && frag.last.iix() == frag.first.iix()
        {
            // BridgeKind::DtoS
            return true;
        }
        // None of the above apply.  This RangeFrag is insane \o/
        false
    }

    let mut last_insn_ix = -1;
    for insn_ix in func.insn_indices() {
        // Ensure instruction indices are in order. Logic below requires this.
        assert!(insn_ix.get() as i32 > last_insn_ix);
        last_insn_ix = insn_ix.get() as i32;

        // advance [cursorStarts, +num_starts) to the group for insn_ix
        while cursor_starts < frag_maps_by_start.len()
            && frag_maps_by_start[cursor_starts].0.first.iix() < insn_ix
        {
            cursor_starts += 1;
        }
        let mut num_starts = 0;
        while cursor_starts + num_starts < frag_maps_by_start.len()
            && frag_maps_by_start[cursor_starts + num_starts].0.first.iix() == insn_ix
        {
            num_starts += 1;
        }

        // advance [cursorEnds, +num_ends) to the group for insn_ix
        while cursor_ends < frag_maps_by_end.len()
            && frag_maps_by_end[cursor_ends].0.last.iix() < insn_ix
        {
            cursor_ends += 1;
        }
        let mut num_ends = 0;
        while cursor_ends + num_ends < frag_maps_by_end.len()
            && frag_maps_by_end[cursor_ends + num_ends].0.last.iix() == insn_ix
        {
            num_ends += 1;
        }

        // advance cursor_nop in the iixs_to_nop_out list.
        while cursor_nop < iixs_to_nop_out.len() && iixs_to_nop_out[cursor_nop] < insn_ix {
            cursor_nop += 1;
        }

        let nop_this_insn =
            cursor_nop < iixs_to_nop_out.len() && iixs_to_nop_out[cursor_nop] == insn_ix;

        // So now, fragMapsByStart[cursorStarts, +num_starts) are the mappings
        // for fragments that begin at this instruction, in no particular
        // order.  And fragMapsByEnd[cursorEnd, +numEnd) are the RangeFragIxs
        // for fragments that end at this instruction.

        // Sanity check all frags.  In particular, reload and spill frags are
        // heavily constrained.  No functional effect.
        for j in cursor_starts..cursor_starts + num_starts {
            let frag = &frag_maps_by_start[j].0;
            // "It really starts here, as claimed."
            debug_assert!(frag.first.iix() == insn_ix);
            debug_assert!(is_sane(&frag));
        }
        for j in cursor_ends..cursor_ends + num_ends {
            let frag = &frag_maps_by_end[j].0;
            // "It really ends here, as claimed."
            debug_assert!(frag.last.iix() == insn_ix);
            debug_assert!(is_sane(frag));
        }

        // Here's the plan, conceptually (we don't actually clone the map):
        // Update map for I.r:
        //   add frags starting at I.r
        //   no frags should end at I.r (it's a reload insn)
        // Update map for I.u:
        //   add frags starting at I.u
        //   map_uses := map
        //   remove frags ending at I.u
        // Update map for I.d:
        //   add frags starting at I.d
        //   map_defs := map
        //   remove frags ending at I.d
        // Update map for I.s:
        //   no frags should start at I.s (it's a spill insn)
        //   remove frags ending at I.s
        // apply map_uses/map_defs to I

        // To update the running mapper, we:
        // - call `mapper.set_direct(vreg, Some(rreg))` with pre-insn starts.
        // ("use"-map snapshot conceptually happens here)
        // - call `mapper.set_overlay(vreg, None)` with pre-insn, post-reload ends.
        // - call `mapper.set_overlay(vreg, Some(rreg))` with post-insn, pre-spill starts.
        // ("post"-map snapshot conceptually happens here)
        // - call `mapper.finish_overlay()`.
        //
        // - Use the map. `pre` and `post` are correct wrt the instruction.
        //
        // - call `mapper.merge_overlay()` to merge post-updates to main map.
        // - call `mapper.set_direct(vreg, None)` with post-insn, post-spill
        //   ends.

        trace!("current mapper {:?}", mapper);

        // Update map for I.r:
        //   add frags starting at I.r
        //   no frags should end at I.r (it's a reload insn)
        for j in cursor_starts..cursor_starts + num_starts {
            let frag = &frag_maps_by_start[j].0;
            if frag.first.pt().is_reload() {
                //////// STARTS at I.r
                mapper.set_direct(frag_maps_by_start[j].1, Some(frag_maps_by_start[j].2));
            }
        }

        // Update map for I.u:
        //   add frags starting at I.u
        //   map_uses := map
        //   remove frags ending at I.u
        for j in cursor_starts..cursor_starts + num_starts {
            let frag = &frag_maps_by_start[j].0;
            if frag.first.pt().is_use() {
                //////// STARTS at I.u
                mapper.set_direct(frag_maps_by_start[j].1, Some(frag_maps_by_start[j].2));
            }
        }
        for j in cursor_ends..cursor_ends + num_ends {
            let frag = &frag_maps_by_end[j].0;
            if frag.last.pt().is_use() {
                //////// ENDS at I.U
                mapper.set_overlay(frag_maps_by_end[j].1, None);
            }
        }

        trace!("maps after I.u {:?}", mapper);

        // Update map for I.d:
        //   add frags starting at I.d
        //   map_defs := map
        //   remove frags ending at I.d
        for j in cursor_starts..cursor_starts + num_starts {
            let frag = &frag_maps_by_start[j].0;
            if frag.first.pt().is_def() {
                //////// STARTS at I.d
                mapper.set_overlay(frag_maps_by_start[j].1, Some(frag_maps_by_start[j].2));
            }
        }

        mapper.finish_overlay();

        trace!("maps after I.d {:?}", mapper);

        // If we have a checker, update it with spills, reloads, moves, and this
        // instruction, while we have `map_uses` and `map_defs` available.
        if let &mut Some(ref mut checker) = &mut checker {
            let block_ix = insn_blocks[insn_ix.get() as usize];
            checker
                .handle_insn(reg_universe, func, block_ix, insn_ix, &mapper)
                .unwrap();
        }

        // Finally, we have map_uses/map_defs set correctly for this instruction.
        // Apply it.
        if !nop_this_insn {
            trace!("map_regs for {:?}", insn_ix);
            let mut insn = func.get_insn_mut(insn_ix);
            F::map_regs(&mut insn, &mapper);
            trace!("mapped instruction: {:?}", insn);
        } else {
            // N.B. We nop out instructions as requested only *here*, after the
            // checker call, because the checker must observe even elided moves
            // (they may carry useful information about a move between two virtual
            // locations mapped to the same physical location).
            trace!("nop'ing out {:?}", insn_ix);
            let nop = func.gen_zero_len_nop();
            let insn = func.get_insn_mut(insn_ix);
            *insn = nop;
        }

        mapper.merge_overlay();
        for j in cursor_ends..cursor_ends + num_ends {
            let frag = &frag_maps_by_end[j].0;
            if frag.last.pt().is_def() {
                //////// ENDS at I.d
                mapper.set_direct(frag_maps_by_end[j].1, None);
            }
        }

        // Update map for I.s:
        //   no frags should start at I.s (it's a spill insn)
        //   remove frags ending at I.s
        for j in cursor_ends..cursor_ends + num_ends {
            let frag = &frag_maps_by_end[j].0;
            if frag.last.pt().is_spill() {
                //////// ENDS at I.s
                mapper.set_direct(frag_maps_by_end[j].1, None);
            }
        }

        // Update cursorStarts and cursorEnds for the next iteration
        cursor_starts += num_starts;
        cursor_ends += num_ends;
    }

    debug_assert!(mapper.is_empty());

    if use_checker {
        checker.unwrap().run()
    } else {
        Ok(())
    }
}

//=============================================================================
// Take the real-register-only code created by `map_vregs_to_rregs` and
// interleave extra instructions (spills, reloads and moves) that the core
// algorithm has asked us to add.

#[inline(never)]
pub(crate) fn add_spills_reloads_and_moves<F: Function>(
    func: &mut F,
    safepoint_insns: Option<&[InstIx]>,
    mut insts_to_add: Vec<InstToInsertAndExtPoint>,
) -> Result<
    (
        Vec<F::Inst>,
        TypedIxVec<BlockIx, InstIx>,
        TypedIxVec<InstIx, InstIx>,
        Vec<InstIx>,
    ),
    String,
> {
    let empty_safepoint_insts = Vec::new();
    let safepoint_insns = safepoint_insns.unwrap_or(&empty_safepoint_insts);

    // Construct the final code by interleaving the mapped code with the the
    // spills, reloads and moves that we have been requested to insert.  To do
    // that requires having the latter sorted by InstPoint.
    //
    // We also need to examine and update Func::blocks.  This is assumed to
    // be arranged in ascending order of the Block::start fields.
    //
    // Also, if the client requested stackmap creation, then `safepoint_insns` will be
    // non-empty, and we will have to return a vector of the same length, that indicates the
    // location of each safepoint insn in the final code.  `safepoint_insns` is assumed to be
    // sorted in ascending order and duplicate-free.
    //
    // Linear scan relies on the sort being stable here, so make sure to not
    // use an unstable sort. See the comment in `resolve_moves_across blocks`
    // in linear scan's code.

    insts_to_add.sort_by_key(|to_add| to_add.iep.clone());

    let mut cur_inst_to_add = 0;
    let mut cur_block = BlockIx::new(0);

    let mut insns: Vec<F::Inst> = vec![];
    let mut target_map: TypedIxVec<BlockIx, InstIx> = TypedIxVec::new();

    let mut new_to_old_insn_map: TypedIxVec<InstIx, InstIx> = TypedIxVec::new();
    target_map.reserve(func.blocks().len());
    new_to_old_insn_map.reserve(func.insn_indices().len() + insts_to_add.len());

    // Index in `safepoint_insns` of the next safepoint insn we will encounter
    let mut next_safepoint_insn_index = 0;
    let mut new_safepoint_insns = Vec::<InstIx>::new();
    new_safepoint_insns.reserve(safepoint_insns.len());

    for iix in func.insn_indices() {
        // Is `iix` the first instruction in a block?  Meaning, are we
        // starting a new block?
        debug_assert!(cur_block.get() < func.blocks().len() as u32);
        if func.block_insns(cur_block).start() == iix {
            assert!(cur_block.get() == target_map.len());
            target_map.push(InstIx::new(insns.len() as u32));
        }

        // Copy to the output vector, the first the extra insts that are to be placed at the
        // reload point of `iix`, and then the extras for the spill-before point of `iix`.
        while cur_inst_to_add < insts_to_add.len()
            && insts_to_add[cur_inst_to_add].iep <= InstExtPoint::new(iix, ExtPoint::SpillBefore)
        {
            if let Some(inst) = insts_to_add[cur_inst_to_add].inst.construct(func) {
                insns.push(inst);
                new_to_old_insn_map.push(InstIx::invalid_value());
            }
            cur_inst_to_add += 1;
        }

        // Copy the inst at `iix` itself
        if next_safepoint_insn_index < safepoint_insns.len()
            && iix == safepoint_insns[next_safepoint_insn_index]
        {
            new_safepoint_insns.push(InstIx::new(insns.len() as u32));
            next_safepoint_insn_index += 1;
        }
        new_to_old_insn_map.push(iix);
        insns.push(func.get_insn(iix).clone());

        // And copy first, the extra insts that are to be placed at the reload-after point
        // of `iix`, followed by those to be placed at the spill point of `iix`.
        while cur_inst_to_add < insts_to_add.len()
            && insts_to_add[cur_inst_to_add].iep <= InstExtPoint::new(iix, ExtPoint::Spill)
        {
            if let Some(inst) = insts_to_add[cur_inst_to_add].inst.construct(func) {
                insns.push(inst);
                new_to_old_insn_map.push(InstIx::invalid_value());
            }
            cur_inst_to_add += 1;
        }

        // Is `iix` the last instruction in a block?
        if iix == func.block_insns(cur_block).last() {
            debug_assert!(cur_block.get() < func.blocks().len() as u32);
            cur_block = cur_block.plus(1);
        }
    }

    debug_assert!(cur_inst_to_add == insts_to_add.len());
    debug_assert!(cur_block.get() == func.blocks().len() as u32);
    debug_assert!(next_safepoint_insn_index == safepoint_insns.len());
    debug_assert!(new_safepoint_insns.len() == safepoint_insns.len());

    Ok((insns, target_map, new_to_old_insn_map, new_safepoint_insns))
}

//=============================================================================
// Main function

#[inline(never)]
pub(crate) fn edit_inst_stream<F: Function>(
    func: &mut F,
    insts_to_add: Vec<InstToInsertAndExtPoint>,
    iixs_to_nop_out: &Vec<InstIx>,
    frag_map: Vec<(RangeFrag, VirtualReg, RealReg)>,
    reg_universe: &RealRegUniverse,
    use_checker: bool,
    stackmap_request: Option<&StackmapRequestInfo>,
    stackmaps: &[Vec<SpillSlot>],
) -> Result<
    (
        Vec<F::Inst>,
        TypedIxVec<BlockIx, InstIx>,
        TypedIxVec<InstIx, InstIx>,
        Vec<InstIx>,
    ),
    RegAllocError,
> {
    map_vregs_to_rregs(
        func,
        frag_map,
        &insts_to_add,
        iixs_to_nop_out,
        reg_universe,
        use_checker,
        stackmap_request,
        stackmaps,
    )
    .map_err(|e| RegAllocError::RegChecker(e))?;

    add_spills_reloads_and_moves(
        func,
        stackmap_request.map(|request| request.safepoint_insns.as_slice()),
        insts_to_add,
    )
    .map_err(|e| RegAllocError::Other(e))
}
