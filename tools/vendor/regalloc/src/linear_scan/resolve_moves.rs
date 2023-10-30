use super::{analysis::BlockPos, next_use, IntId, Location, RegUses, VirtualInterval};
use crate::{
    analysis_control_flow::CFGInfo,
    data_structures::{BlockIx, InstPoint, Point},
    inst_stream::{InstExtPoint, InstToInsert, InstToInsertAndExtPoint},
    sparse_set::SparseSet,
    Function, RealReg, Reg, SpillSlot, TypedIxVec, VirtualReg, Writable,
};

use log::{debug, trace};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use smallvec::SmallVec;
use std::fmt;

fn resolve_moves_in_block<F: Function>(
    func: &F,
    intervals: &Vec<VirtualInterval>,
    reg_uses: &RegUses,
    scratches_by_rc: &[Option<RealReg>],
    spill_slot: &mut u32,
    moves_in_blocks: &mut Vec<InstToInsertAndExtPoint>,
    tmp_ordered_moves: &mut Vec<MoveOp>,
    tmp_stack: &mut Vec<MoveOp>,
) {
    let mut block_ends = HashSet::default();
    let mut block_starts = HashSet::default();
    for bix in func.blocks() {
        let insts = func.block_insns(bix);
        block_ends.insert(insts.last());
        block_starts.insert(insts.first());
    }

    let mut reloads_at_inst = HashMap::default();
    let mut spills_at_inst = Vec::new();

    for interval in intervals {
        let parent_id = match interval.parent {
            Some(pid) => pid,
            None => {
                // In unreachable code, it's possible that a given interval has no
                // parents and is assigned to a stack location for its whole lifetime.
                //
                // In reachable code, the analysis only create intervals for virtual
                // registers with at least one register use, so a parentless interval (=
                // hasn't ever been split) can't live in a stack slot.
                #[cfg(debug_assertions)]
                debug_assert!(
                    interval.location.spill().is_none()
                        || (next_use(interval, InstPoint::min_value(), reg_uses,).is_none())
                );
                continue;
            }
        };

        let parent = &intervals[parent_id.0];

        // If this is a move between blocks, handle it as such.
        if parent.end.pt() == Point::Def
            && interval.start.pt() == Point::Use
            && block_ends.contains(&parent.end.iix())
            && block_starts.contains(&interval.start.iix())
        {
            continue;
        }

        let child_start = interval.start;
        let vreg = interval.vreg;

        match interval.location {
            Location::None => panic!("interval has no location after regalloc!"),

            Location::Reg(rreg) => {
                // Reconnect with the parent location, by adding a move if needed.
                if let Some(next_use) = next_use(interval, child_start, reg_uses) {
                    // No need to reload before a new definition.
                    if next_use.pt() == Point::Def {
                        continue;
                    }
                };

                let mut at_inst = child_start;
                match at_inst.pt() {
                    Point::Use => {
                        at_inst.set_pt(Point::Reload);
                    }
                    Point::Def => {
                        at_inst.set_pt(Point::Spill);
                    }
                    _ => unreachable!(),
                }

                let entry = reloads_at_inst.entry(at_inst).or_insert_with(|| Vec::new());

                match parent.location {
                    Location::None => unreachable!(),

                    Location::Reg(from_rreg) => {
                        if from_rreg != rreg {
                            debug!(
                                "inblock fixup: {:?} move {:?} -> {:?} at {:?}",
                                interval.id, from_rreg, rreg, at_inst
                            );
                            entry.push(MoveOp::new_move(from_rreg, rreg, vreg));
                        }
                    }

                    Location::Stack(spill) => {
                        debug!(
                            "inblock fixup: {:?} reload {:?} -> {:?} at {:?}",
                            interval.id, spill, rreg, at_inst
                        );
                        entry.push(MoveOp::new_reload(spill, rreg, vreg));
                    }
                }
            }

            Location::Stack(spill) => {
                // This interval has been spilled (i.e. split). Spill after the last def or before
                // the last use.
                let mut at_inst = parent.end;
                at_inst.set_pt(if at_inst.pt() == Point::Use {
                    Point::Reload
                } else {
                    debug_assert!(at_inst.pt() == Point::Def);
                    Point::Spill
                });

                match parent.location {
                    Location::None => unreachable!(),

                    Location::Reg(rreg) => {
                        debug!(
                            "inblock fixup: {:?} spill {:?} -> {:?} at {:?}",
                            interval.id, rreg, spill, at_inst
                        );
                        spills_at_inst.push(InstToInsertAndExtPoint::new(
                            InstToInsert::Spill {
                                to_slot: spill,
                                from_reg: rreg,
                                for_vreg: Some(vreg),
                            },
                            InstExtPoint::from_inst_point(at_inst),
                        ));
                    }

                    Location::Stack(parent_spill) => {
                        debug_assert_eq!(parent_spill, spill);
                    }
                }
            }
        }
    }

    // Flush the memory moves caused by in-block fixups. Conceptually, the spills
    // must happen after the right locations have been set, that is, after the
    // reloads. Reloads may include several moves that must happen in parallel
    // (e.g. if two real regs must be swapped), so process them first. Once all
    // the parallel assignments have been done, push forward all the spills.
    for (at_inst, mut pending_moves) in reloads_at_inst {
        schedule_moves(&mut pending_moves, tmp_ordered_moves, tmp_stack);
        emit_moves(
            at_inst,
            &tmp_ordered_moves,
            spill_slot,
            scratches_by_rc,
            moves_in_blocks,
        );
    }

    moves_in_blocks.append(&mut spills_at_inst);
}

#[derive(Default, Clone)]
struct BlockInfo {
    start: SmallVec<[(VirtualReg, IntId); 4]>,
    end: SmallVec<[(VirtualReg, IntId); 4]>,
}

static UNSORTED_THRESHOLD: usize = 8;

impl BlockInfo {
    #[inline(never)]
    fn insert(&mut self, pos: BlockPos, vreg: VirtualReg, id: IntId) {
        match pos {
            BlockPos::Start => {
                #[cfg(debug_assertions)]
                debug_assert!(self.start.iter().find(|prev| prev.0 == vreg).is_none());
                self.start.push((vreg, id));
            }
            BlockPos::End => {
                #[cfg(debug_assertions)]
                debug_assert!(self.end.iter().find(|prev| prev.0 == vreg).is_none());
                self.end.push((vreg, id));
            }
        }
    }

    #[inline(never)]
    fn finish(&mut self) {
        if self.start.len() >= UNSORTED_THRESHOLD {
            self.start.sort_unstable_by_key(|pair| pair.0);
        }
        if self.end.len() >= UNSORTED_THRESHOLD {
            self.end.sort_unstable_by_key(|pair| pair.0);
        }
    }

    #[inline(never)]
    fn lookup(&self, pos: BlockPos, vreg: &VirtualReg) -> IntId {
        let array = match pos {
            BlockPos::Start => &self.start,
            BlockPos::End => &self.end,
        };
        if array.len() >= UNSORTED_THRESHOLD {
            array[array.binary_search_by_key(vreg, |pair| pair.0).unwrap()].1
        } else {
            array
                .iter()
                .find(|el| el.0 == *vreg)
                .expect("should have found target reg")
                .1
        }
    }
}

/// For each block, collect a mapping of block_{start, end} -> actual location, to make the
/// across-blocks fixup phase fast.
#[inline(never)]
fn collect_block_infos<F: Function>(
    func: &F,
    intervals: &Vec<VirtualInterval>,
    liveins: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    liveouts: &TypedIxVec<BlockIx, SparseSet<Reg>>,
) -> Vec<BlockInfo> {
    // Preallocate the block information, with the final size of each vector.
    let mut infos = Vec::with_capacity(func.blocks().len());
    for bix in func.blocks() {
        infos.push(BlockInfo {
            start: SmallVec::with_capacity(liveins[bix].card()),
            end: SmallVec::with_capacity(liveouts[bix].card()),
        });
    }

    // For each interval, add the boundary information to the block info data structure.
    for int in intervals {
        let vreg = int.vreg;
        let id = int.id;
        for boundary in &int.block_boundaries {
            let bix = boundary.bix;
            let pos = boundary.pos;
            match pos {
                BlockPos::Start => {
                    // In theory, this could be an assertion, if analysis was precise and meaning
                    // that RangeFragKind::Thru/LiveIn really meant that (it actually means that
                    // the first fragment inst coincided with the block's first inst).
                    if !liveins[bix].contains(vreg.to_reg()) {
                        continue;
                    }
                }
                BlockPos::End => {
                    // See above comment.
                    if !liveouts[bix].contains(vreg.to_reg()) {
                        continue;
                    }
                }
            }
            infos[bix.get() as usize].insert(pos, vreg, id);
        }
    }

    for info in infos.iter_mut() {
        info.finish();
    }

    infos
}

/// Figure the sequence of parallel moves to insert at block boundaries:
/// - for each block
///  - for each liveout vreg in this block
///    - for each successor of this block
///      - if the locations allocated in the block and its successor don't
///      match, insert a pending move from one location to the other.
///
/// Once that's done:
/// - resolve cycles in the pending moves
/// - generate real moves from the pending moves.
#[inline(never)]
fn resolve_moves_across_blocks<F: Function>(
    func: &F,
    cfg: &CFGInfo,
    liveins: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    liveouts: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    intervals: &Vec<VirtualInterval>,
    scratches_by_rc: &[Option<RealReg>],
    spill_slot: &mut u32,
    moves_at_block_starts: &mut Vec<InstToInsertAndExtPoint>,
    moves_at_block_ends: &mut Vec<InstToInsertAndExtPoint>,
    tmp_ordered_moves: &mut Vec<MoveOp>,
    tmp_stack: &mut Vec<MoveOp>,
) {
    let mut parallel_move_map = HashMap::default();

    let block_info = collect_block_infos(func, intervals, liveins, liveouts);

    let mut seen_successors = HashSet::default();
    for block in func.blocks() {
        let successors = &cfg.succ_map[block];

        // Where to insert the fixup move, if needed? Per API contract, there are no more critical
        // edges: so this block is the only successor to its predecessors, or the only predecessor
        // to its successors in the control flow graph.
        // If there's more than one successor to the current block, then inserting the fix-up moves
        // in the current block may impact all the other successors: in this case, we should insert
        // the instruction at the end of the current block. If the successors all have only one
        // predecessor (the current block), then we can insert the fixup move at the beginning of
        // the successors safely.
        let all_succ_have_one_pred = successors
            .iter()
            .all(|succ| cfg.pred_map[*succ].card() == 1);
        assert!(successors.card() == 1 || all_succ_have_one_pred);

        for &reg in liveouts[block].iter() {
            let vreg = if let Some(vreg) = reg.as_virtual_reg() {
                vreg
            } else {
                continue;
            };

            seen_successors.clear();

            let cur_id = block_info[block.get() as usize].lookup(BlockPos::End, &vreg);
            let cur_int = &intervals[cur_id.0];
            let loc_at_cur_end = cur_int.location;

            for &succ in successors.iter() {
                if !liveins[succ].contains(reg) {
                    // This variable isn't live in this block.
                    continue;
                }
                if !seen_successors.insert(succ) {
                    continue;
                }

                let succ_id = block_info[succ.get() as usize].lookup(BlockPos::Start, &vreg);
                let succ_int = &intervals[succ_id.0];

                // If the two intervals aren't related to the same virtual range, then the move is
                // not required.
                if cur_int.ancestor != succ_int.ancestor {
                    continue;
                }

                let loc_at_succ_start = succ_int.location;

                let (at_inst, block_pos) = if all_succ_have_one_pred {
                    // At the beginning of the successors (each only has a single predecessor).
                    let pos = InstPoint::new_reload(func.block_insns(succ).first());
                    (pos, BlockPos::Start)
                } else {
                    // Before the control flow instruction.
                    let pos = InstPoint::new_reload(func.block_insns(block).last());
                    (pos, BlockPos::End)
                };

                let pending_moves = parallel_move_map
                    .entry(at_inst)
                    .or_insert_with(|| (Vec::new(), block_pos));

                match (loc_at_cur_end, loc_at_succ_start) {
                    (Location::Reg(cur_rreg), Location::Reg(succ_rreg)) => {
                        if cur_rreg == succ_rreg {
                            continue;
                        }
                        debug!(
                          "boundary fixup: move {:?} -> {:?} at {:?} for {:?} between {:?} and {:?}",
                          cur_rreg,
                          succ_rreg,
                          at_inst,
                          vreg,
                          block,
                          succ
                        );
                        pending_moves
                            .0
                            .push(MoveOp::new_move(cur_rreg, succ_rreg, vreg));
                    }

                    (Location::Reg(cur_rreg), Location::Stack(spillslot)) => {
                        debug!(
                          "boundary fixup: spill {:?} -> {:?} at {:?} for {:?} between {:?} and {:?}",
                          cur_rreg,
                          spillslot,
                          at_inst,
                          vreg,
                          block,
                          succ
                        );
                        pending_moves
                            .0
                            .push(MoveOp::new_spill(cur_rreg, spillslot, vreg));
                    }

                    (Location::Stack(spillslot), Location::Reg(rreg)) => {
                        debug!(
                          "boundary fixup: reload {:?} -> {:?} at {:?} for {:?} between {:?} and {:?}",
                          spillslot,
                          rreg,
                          at_inst,
                          vreg,
                          block,
                          succ
                        );
                        pending_moves
                            .0
                            .push(MoveOp::new_reload(spillslot, rreg, vreg));
                    }

                    (Location::Stack(left_spill_slot), Location::Stack(right_spill_slot)) => {
                        // Stack to stack should not happen here, since two ranges for the
                        // same vreg can't be intersecting, so the same stack slot ought to
                        // be reused in this case.
                        debug_assert_eq!(
                          left_spill_slot, right_spill_slot,
                          "Moves from stack to stack only happen on the same vreg, thus the same stack slot"
                        );
                        continue;
                    }

                    (_, _) => {
                        panic!("register or stack slots must have been allocated.");
                    }
                };
            }
        }

        // Flush the memory moves caused by block fixups for this block.
        for (at_inst, (move_insts, block_pos)) in parallel_move_map.iter_mut() {
            schedule_moves(move_insts, tmp_ordered_moves, tmp_stack);

            match block_pos {
                BlockPos::Start => {
                    emit_moves(
                        *at_inst,
                        &tmp_ordered_moves,
                        spill_slot,
                        scratches_by_rc,
                        moves_at_block_starts,
                    );
                }
                BlockPos::End => {
                    emit_moves(
                        *at_inst,
                        &tmp_ordered_moves,
                        spill_slot,
                        scratches_by_rc,
                        moves_at_block_ends,
                    );
                }
            };
        }

        parallel_move_map.clear();
    }

    debug!("");
}

#[inline(never)]
pub(crate) fn run<F: Function>(
    func: &F,
    cfg: &CFGInfo,
    reg_uses: &RegUses,
    intervals: &Vec<VirtualInterval>,
    liveins: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    liveouts: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    spill_slot: &mut u32,
    scratches_by_rc: &[Option<RealReg>],
) -> Vec<InstToInsertAndExtPoint> {
    debug!("resolve_moves");

    // Keep three lists of moves to insert:
    // - moves across blocks, that must happen at the start of blocks,
    // - moves within a given block,
    // - moves across blocks, that must happen at the end of blocks.
    //
    // To maintain the property that these moves are eventually sorted at the end, we'll compute
    // the final array of moves by concatenating these three arrays. `inst_stream` uses a stable
    // sort, making sure the at-block-start/within-block/at-block-end will be respected.
    let mut moves_at_block_starts = Vec::new();
    let mut moves_at_block_ends = Vec::new();
    let mut moves_in_blocks = Vec::new();

    let mut tmp_stack = Vec::new();
    let mut tmp_ordered_moves = Vec::new();
    resolve_moves_in_block(
        func,
        intervals,
        reg_uses,
        scratches_by_rc,
        spill_slot,
        &mut moves_in_blocks,
        &mut tmp_ordered_moves,
        &mut tmp_stack,
    );

    resolve_moves_across_blocks(
        func,
        cfg,
        liveins,
        liveouts,
        intervals,
        scratches_by_rc,
        spill_slot,
        &mut moves_at_block_starts,
        &mut moves_at_block_ends,
        &mut tmp_ordered_moves,
        &mut tmp_stack,
    );

    let mut insts_and_points = moves_at_block_starts;
    insts_and_points.reserve(moves_in_blocks.len() + moves_at_block_ends.len());
    insts_and_points.append(&mut moves_in_blocks);
    insts_and_points.append(&mut moves_at_block_ends);

    insts_and_points
}

#[derive(PartialEq, Debug)]
enum MoveOperand {
    Reg(RealReg),
    Stack(SpillSlot),
}

impl MoveOperand {
    fn aliases(&self, other: &Self) -> bool {
        self == other
    }
}

struct MoveOp {
    from: MoveOperand,
    to: MoveOperand,
    vreg: VirtualReg,
    cycle_begin: Option<usize>,
    cycle_end: Option<usize>,
}

impl fmt::Debug for MoveOp {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}: {:?} -> {:?}", self.vreg, self.from, self.to)?;
        if let Some(ref begin) = self.cycle_begin {
            write!(fmt, ", start of cycle #{}", begin)?;
        }
        if let Some(ref end) = self.cycle_end {
            write!(fmt, ", end of cycle #{}", end)?;
        }
        Ok(())
    }
}

impl MoveOp {
    fn new_move(from: RealReg, to: RealReg, vreg: VirtualReg) -> Self {
        Self {
            from: MoveOperand::Reg(from),
            to: MoveOperand::Reg(to),
            vreg,
            cycle_begin: None,
            cycle_end: None,
        }
    }

    fn new_spill(from: RealReg, to: SpillSlot, vreg: VirtualReg) -> Self {
        Self {
            from: MoveOperand::Reg(from),
            to: MoveOperand::Stack(to),
            vreg,
            cycle_begin: None,
            cycle_end: None,
        }
    }

    fn new_reload(from: SpillSlot, to: RealReg, vreg: VirtualReg) -> Self {
        Self {
            from: MoveOperand::Stack(from),
            to: MoveOperand::Reg(to),
            vreg,
            cycle_begin: None,
            cycle_end: None,
        }
    }

    fn gen_inst(&self) -> InstToInsert {
        match self.from {
            MoveOperand::Reg(from) => match self.to {
                MoveOperand::Reg(to) => InstToInsert::Move {
                    to_reg: Writable::from_reg(to),
                    from_reg: from,
                    for_vreg: self.vreg,
                },
                MoveOperand::Stack(to) => InstToInsert::Spill {
                    to_slot: to,
                    from_reg: from,
                    for_vreg: Some(self.vreg),
                },
            },
            MoveOperand::Stack(from) => match self.to {
                MoveOperand::Reg(to) => InstToInsert::Reload {
                    to_reg: Writable::from_reg(to),
                    from_slot: from,
                    for_vreg: Some(self.vreg),
                },
                MoveOperand::Stack(_to) => unreachable!("stack to stack move"),
            },
        }
    }
}

fn find_blocking_move<'a>(
    pending: &'a mut Vec<MoveOp>,
    last: &MoveOp,
) -> Option<(usize, &'a mut MoveOp)> {
    for (i, other) in pending.iter_mut().enumerate() {
        if other.from.aliases(&last.to) {
            return Some((i, other));
        }
    }
    None
}

fn find_cycled_move<'a>(
    stack: &'a mut Vec<MoveOp>,
    from: &mut usize,
    last: &MoveOp,
) -> Option<&'a mut MoveOp> {
    for i in *from..stack.len() {
        *from += 1;
        let other = &stack[i];
        if other.from.aliases(&last.to) {
            return Some(&mut stack[i]);
        }
    }
    None
}

/// Given a pending list of moves, returns a list of moves ordered in a correct
/// way, i.e., no move clobbers another one.
#[inline(never)]
fn schedule_moves(
    pending: &mut Vec<MoveOp>,
    ordered_moves: &mut Vec<MoveOp>,
    stack: &mut Vec<MoveOp>,
) {
    ordered_moves.clear();

    let mut num_cycles = 0;
    let mut cur_cycles = 0;

    trace!("pending moves: {:#?}", pending);

    while let Some(pm) = pending.pop() {
        trace!("handling pending move {:?}", pm);
        debug_assert!(
            pm.from != pm.to,
            "spurious moves should not have been inserted"
        );

        stack.clear();
        stack.push(pm);

        while !stack.is_empty() {
            let blocking_pair = find_blocking_move(pending, stack.last().unwrap());

            if let Some((blocking_idx, blocking)) = blocking_pair {
                trace!("found blocker: {:?}", blocking);
                let mut stack_cur = 0;

                let has_cycles =
                    if let Some(mut cycled) = find_cycled_move(stack, &mut stack_cur, blocking) {
                        trace!("found cycle: {:?}", cycled);
                        debug_assert!(cycled.cycle_end.is_none());
                        cycled.cycle_end = Some(cur_cycles);
                        true
                    } else {
                        false
                    };

                if has_cycles {
                    loop {
                        match find_cycled_move(stack, &mut stack_cur, blocking) {
                            Some(ref mut cycled) => {
                                trace!("found more cycles ending on blocker: {:?}", cycled);
                                debug_assert!(cycled.cycle_end.is_none());
                                cycled.cycle_end = Some(cur_cycles);
                            }
                            None => break,
                        }
                    }

                    debug_assert!(blocking.cycle_begin.is_none());
                    blocking.cycle_begin = Some(cur_cycles);
                    cur_cycles += 1;
                }

                let blocking = pending.remove(blocking_idx);
                stack.push(blocking);
            } else {
                // There's no blocking move! We can push this in the ordered list of
                // moves.
                // TODO IonMonkey has more optimizations for this case.
                let last = stack.pop().unwrap();
                ordered_moves.push(last);
            }
        }

        if num_cycles < cur_cycles {
            num_cycles = cur_cycles;
        }
        cur_cycles = 0;
    }
}

#[inline(never)]
fn emit_moves(
    at_inst: InstPoint,
    ordered_moves: &Vec<MoveOp>,
    num_spill_slots: &mut u32,
    scratches_by_rc: &[Option<RealReg>],
    moves_in_blocks: &mut Vec<InstToInsertAndExtPoint>,
) {
    let mut spill_slot = None;
    let mut in_cycle = false;

    trace!("emit_moves");

    for mov in ordered_moves {
        if let Some(_) = &mov.cycle_end {
            debug_assert!(in_cycle);

            // There is some pattern:
            //   (A -> B)
            //   (B -> A)
            // This case handles (B -> A), which we reach last. We emit a move from
            // the saved value of B, to A.
            match mov.to {
                MoveOperand::Reg(dst_reg) => {
                    let inst = InstToInsert::Reload {
                        to_reg: Writable::from_reg(dst_reg),
                        from_slot: spill_slot.expect("should have a cycle spill slot"),
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    trace!(
                        "finishing cycle: {:?} -> {:?}",
                        spill_slot.unwrap(),
                        dst_reg
                    );
                }
                MoveOperand::Stack(dst_spill) => {
                    let scratch = scratches_by_rc[mov.vreg.get_class() as usize]
                        .expect("missing scratch reg");
                    let inst = InstToInsert::Reload {
                        to_reg: Writable::from_reg(scratch),
                        from_slot: spill_slot.expect("should have a cycle spill slot"),
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    let inst = InstToInsert::Spill {
                        to_slot: dst_spill,
                        from_reg: scratch,
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    trace!(
                        "finishing cycle: {:?} -> {:?} -> {:?}",
                        spill_slot.unwrap(),
                        scratch,
                        dst_spill
                    );
                }
            };

            in_cycle = false;
            continue;
        }

        if let Some(_) = &mov.cycle_begin {
            debug_assert!(!in_cycle);

            // There is some pattern:
            //   (A -> B)
            //   (B -> A)
            // This case handles (A -> B), which we reach first. We save B, then allow
            // the original move to continue.
            match spill_slot {
                Some(_) => {}
                None => {
                    spill_slot = Some(SpillSlot::new(*num_spill_slots));
                    *num_spill_slots += 1;
                }
            }

            match mov.to {
                MoveOperand::Reg(src_reg) => {
                    let inst = InstToInsert::Spill {
                        to_slot: spill_slot.unwrap(),
                        from_reg: src_reg,
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    trace!("starting cycle: {:?} -> {:?}", src_reg, spill_slot.unwrap());
                }
                MoveOperand::Stack(src_spill) => {
                    let scratch = scratches_by_rc[mov.vreg.get_class() as usize]
                        .expect("missing scratch reg");
                    let inst = InstToInsert::Reload {
                        to_reg: Writable::from_reg(scratch),
                        from_slot: src_spill,
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    let inst = InstToInsert::Spill {
                        to_slot: spill_slot.expect("should have a cycle spill slot"),
                        from_reg: scratch,
                        for_vreg: Some(mov.vreg),
                    };
                    moves_in_blocks.push(InstToInsertAndExtPoint::new(
                        inst,
                        InstExtPoint::from_inst_point(at_inst),
                    ));
                    trace!(
                        "starting cycle: {:?} -> {:?} -> {:?}",
                        src_spill,
                        scratch,
                        spill_slot.unwrap()
                    );
                }
            };

            in_cycle = true;
        }

        // A normal move which is not part of a cycle.
        let inst = mov.gen_inst();
        moves_in_blocks.push(InstToInsertAndExtPoint::new(
            inst,
            InstExtPoint::from_inst_point(at_inst),
        ));
        trace!("moving {:?} -> {:?}", mov.from, mov.to);
    }
}
