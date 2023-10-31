use super::{FixedInterval, IntId, Intervals, Mention, MentionMap, Safepoints, VirtualInterval};
use crate::{
    analysis_control_flow::{CFGInfo, InstIxToBlockIxMap},
    analysis_data_flow::collect_move_info,
    analysis_data_flow::{
        calc_def_and_use, calc_livein_and_liveout, get_sanitized_reg_uses_for_func, reg_ix_to_reg,
        reg_to_reg_ix,
    },
    analysis_main::DepthBasedFrequencies,
    analysis_reftypes::{core_reftypes_analysis, ReftypeAnalysis},
    data_structures::*,
    sparse_set::SparseSet,
    union_find::UnionFind,
    AnalysisError, Function, RealRegUniverse, RegClass, StackmapRequestInfo, TypedIxVec,
};
use log::{log_enabled, trace, Level};
use smallvec::{smallvec, SmallVec};
use std::{cmp::Ordering, fmt, mem};

#[derive(Clone, Copy, PartialEq, Eq, Ord)]
pub(crate) enum BlockPos {
    Start,
    End,
}

// Start < End.
impl PartialOrd for BlockPos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if *self == BlockPos::Start && *other == BlockPos::End {
            Ordering::Less
        } else if *self == BlockPos::End && *other == BlockPos::Start {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }
}

#[derive(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub(crate) struct BlockBoundary {
    pub(crate) bix: BlockIx,
    pub(crate) pos: BlockPos,
}

#[derive(Clone)]
pub(crate) struct RangeFrag {
    pub(crate) first: InstPoint,
    pub(crate) last: InstPoint,
    pub(crate) mentions: MentionMap,
    pub(crate) safepoints: Safepoints,
    pub(crate) ref_typed: bool,
}

impl fmt::Debug for RangeFrag {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let safepoint_str = if !self.safepoints.is_empty() {
            format!(
                "; safepoints: {}",
                self.safepoints
                    .iter()
                    .map(|pt| format!("{:?}", pt))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            "".into()
        };
        write!(fmt, "[{:?}; {:?}{}]", self.first, self.last, safepoint_str)
    }
}

impl RangeFrag {
    fn new<F: Function>(
        func: &F,
        bix: BlockIx,
        first: InstPoint,
        last: InstPoint,
        mentions: MentionMap,
        ref_typed: bool,
        safepoints: Safepoints,
    ) -> (Self, RangeFragMetrics) {
        debug_assert!(func.block_insns(bix).len() >= 1);
        debug_assert!(func.block_insns(bix).contains(first.iix()));
        debug_assert!(func.block_insns(bix).contains(last.iix()));
        debug_assert!(first <= last);

        let first_in_block = InstPoint::new_use(func.block_insns(bix).first());
        let last_in_block = InstPoint::new_def(func.block_insns(bix).last());
        let kind = match (first == first_in_block, last == last_in_block) {
            (false, false) => RangeFragKind::Local,
            (false, true) => RangeFragKind::LiveOut,
            (true, false) => RangeFragKind::LiveIn,
            (true, true) => RangeFragKind::Thru,
        };

        (
            RangeFrag {
                first,
                last,
                mentions,
                safepoints,
                ref_typed,
            },
            RangeFragMetrics { bix, kind },
        )
    }

    #[inline(always)]
    pub(crate) fn contains(&self, inst: &InstPoint) -> bool {
        self.first <= *inst && *inst <= self.last
    }
}

struct RangeFragMetrics {
    bix: BlockIx,
    kind: RangeFragKind,
}

pub(crate) struct AnalysisInfo {
    /// Control-flow graph information.
    pub(crate) cfg: CFGInfo,
    /// The sanitized per-insn reg-use info.
    pub(crate) reg_vecs_and_bounds: RegVecsAndBounds,
    /// All the intervals, fixed or virtual.
    pub(crate) intervals: Intervals,
    /// Liveins per block.
    pub(crate) liveins: TypedIxVec<BlockIx, SparseSet<Reg>>,
    /// Liveouts per block.
    pub(crate) liveouts: TypedIxVec<BlockIx, SparseSet<Reg>>,
    /// Maps InstIxs to BlockIxs.
    pub(crate) _inst_to_block_map: InstIxToBlockIxMap,
}

#[inline(never)]
pub(crate) fn run<F: Function>(
    func: &F,
    reg_universe: &RealRegUniverse,
    stackmap_request: Option<&StackmapRequestInfo>,
) -> Result<AnalysisInfo, AnalysisError> {
    trace!(
        "run_analysis: begin: {} blocks, {} insns",
        func.blocks().len(),
        func.insns().len()
    );

    // First do control flow analysis.  This is (relatively) simple.  Note that this can fail, for
    // various reasons; we propagate the failure if so.  Also create the InstIx-to-BlockIx map;
    // this isn't really control-flow analysis, but needs to be done at some point.

    trace!("  run_analysis: begin control flow analysis");
    let cfg_info = CFGInfo::create(func)?;
    let inst_to_block_map = InstIxToBlockIxMap::new(func);
    trace!("  run_analysis: end control flow analysis");

    trace!("  run_analysis: begin data flow analysis");

    // See `get_sanitized_reg_uses_for_func` for the meaning of "sanitized".
    let reg_vecs_and_bounds = get_sanitized_reg_uses_for_func(func, reg_universe)
        .map_err(|reg| AnalysisError::IllegalRealReg(reg))?;
    assert!(reg_vecs_and_bounds.is_sanitized());

    // Calculate block-local def/use sets.
    let (def_sets_per_block, use_sets_per_block) =
        calc_def_and_use(func, &reg_vecs_and_bounds, &reg_universe);
    debug_assert!(def_sets_per_block.len() == func.blocks().len() as u32);
    debug_assert!(use_sets_per_block.len() == func.blocks().len() as u32);

    // Calculate live-in and live-out sets per block, using the traditional
    // iterate-to-a-fixed-point scheme.
    // `liveout_sets_per_block` is amended below for return blocks, hence `mut`.

    let (livein_sets_per_block, mut liveout_sets_per_block) = calc_livein_and_liveout(
        func,
        &def_sets_per_block,
        &use_sets_per_block,
        &cfg_info,
        &reg_universe,
    );
    debug_assert!(livein_sets_per_block.len() == func.blocks().len() as u32);
    debug_assert!(liveout_sets_per_block.len() == func.blocks().len() as u32);

    // Verify livein set of entry block against liveins specified by function (e.g., ABI params).
    let func_liveins = SparseSet::from_vec(
        func.func_liveins()
            .to_vec()
            .into_iter()
            .map(|rreg| rreg.to_reg())
            .collect(),
    );
    if !livein_sets_per_block[func.entry_block()].is_subset_of(&func_liveins) {
        let mut regs = livein_sets_per_block[func.entry_block()].clone();
        regs.remove(&func_liveins);
        return Err(AnalysisError::EntryLiveinValues(regs.to_vec()));
    }

    // Add function liveouts to every block ending in a return.
    let func_liveouts = SparseSet::from_vec(
        func.func_liveouts()
            .to_vec()
            .into_iter()
            .map(|rreg| rreg.to_reg())
            .collect(),
    );

    for block in func.blocks() {
        let last_iix = func.block_insns(block).last();
        if func.is_ret(last_iix) {
            liveout_sets_per_block[block].union(&func_liveouts);
        }

        // While we're here: consider if the (ending) control flow instruction has register mentions (any
        // use/def/mod).
        //
        // If that's the case, then the successor blocks must have at most one predecessor,
        // otherwise inter-blocks fix-up moves may interfere with the control flow instruction
        // register mentions, resulting in allocations impossible to solve.
        let bounds = &reg_vecs_and_bounds.bounds[last_iix];
        if bounds.uses_len + bounds.defs_len + bounds.mods_len > 0 {
            for &succ_ix in cfg_info.succ_map[block].iter() {
                if cfg_info.pred_map[succ_ix].card() > 1 {
                    return Err(AnalysisError::LsraCriticalEdge {
                        block,
                        inst: last_iix,
                    });
                }
            }
        }
    }

    trace!("  run_analysis: end data flow analysis");

    trace!("  run_analysis: begin liveness analysis");
    let (frag_ixs_per_reg, mut frag_env, frag_metrics_env, vreg_classes) = get_range_frags(
        func,
        &reg_universe,
        stackmap_request.map(|req| req.safepoint_insns.as_slice()),
        &reg_vecs_and_bounds,
        &livein_sets_per_block,
        &liveout_sets_per_block,
    );

    let (mut fixed_intervals, mut virtual_intervals, vreg_to_vranges) = merge_range_frags(
        func,
        &reg_universe,
        &frag_ixs_per_reg,
        &mut frag_env,
        &frag_metrics_env,
        &cfg_info,
        &vreg_classes,
        stackmap_request.is_some(),
    )?;
    trace!("  run_analysis: end liveness analysis");

    // Make sure the fixed interval's fragment are sorted, to allow for binary search in misc
    // contexts.
    for fixed in fixed_intervals.iter_mut() {
        fixed.frags.sort_unstable_by_key(|frag| frag.first);
    }

    if let Some(stackmap_request) = stackmap_request {
        // TODO depth-based is fine, using the plain depth would be sufficient too.
        let estimator = DepthBasedFrequencies::new(func, &cfg_info);
        let move_info = collect_move_info(func, &reg_vecs_and_bounds, &estimator);

        do_reftype_analysis(
            &move_info,
            &mut fixed_intervals,
            &mut virtual_intervals,
            &vreg_to_vranges,
            &frag_env,
            stackmap_request,
        );
    }

    let intervals = Intervals {
        virtuals: virtual_intervals,
        fixeds: fixed_intervals,
    };

    trace!("run_analysis: end");

    Ok(AnalysisInfo {
        cfg: cfg_info,
        reg_vecs_and_bounds,
        intervals,
        liveins: livein_sets_per_block,
        liveouts: liveout_sets_per_block,
        _inst_to_block_map: inst_to_block_map,
    })
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum RangeId {
    Fixed(RealReg, usize),
    Virtual(usize),
}

struct LsraReftypeAnalysis<'a> {
    fixed_intervals: &'a mut [FixedInterval],
    virtual_intervals: &'a mut [VirtualInterval],
    vreg_to_vranges: &'a VirtualRegToRanges,
    frag_env: &'a [RangeFrag],
}

impl<'a> ReftypeAnalysis for LsraReftypeAnalysis<'a> {
    type RangeId = RangeId;

    fn find_range_id_for_reg(&self, pt: InstPoint, reg: Reg) -> Self::RangeId {
        if reg.is_real() {
            let frag_index = self.fixed_intervals[reg.get_index()].find_frag(pt);
            return RangeId::Fixed(reg.to_real_reg(), frag_index);
        }

        let vreg = reg.to_virtual_reg();
        let vranges = &self.vreg_to_vranges[vreg.get_index()];
        for vrange in vranges {
            // If any fragment in the range contains the point, then the whole range must be reffy.
            let found = vrange
                .frag_ixs
                .binary_search_by(|&ix| {
                    let frag = &self.frag_env[ix.get() as usize];
                    if pt < frag.first {
                        Ordering::Greater
                    } else if pt >= frag.first && pt <= frag.last {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                })
                .is_ok();
            if found {
                return RangeId::Virtual(vrange.int.0);
            }
        }

        panic!("should have found a (vreg; vrange) containing the move!");
    }

    fn insert_reffy_ranges(&self, vreg: VirtualReg, set: &mut SparseSet<Self::RangeId>) {
        for vrange in &self.vreg_to_vranges[vreg.get_index()] {
            trace!(
                "range {:?} is reffy due to reffy vreg {:?}",
                vrange.int,
                vreg
            );
            set.insert(RangeId::Virtual(vrange.int.0));
        }
    }

    fn mark_reffy(&mut self, range: &Self::RangeId) {
        match range {
            RangeId::Fixed(rreg, frag_ix) => {
                let frag = &mut self.fixed_intervals[rreg.get_index() as usize].frags[*frag_ix];
                trace!(
                    "Fragment {} of interval for {:?} is reftyped.",
                    *frag_ix,
                    rreg
                );
                frag.ref_typed = true;
            }
            RangeId::Virtual(int_id) => {
                let int = &mut self.virtual_intervals[*int_id];
                int.ref_typed = true;
                trace!("Virtual interval {:?} is reftyped", int.id);
            }
        }
    }
}

/// Given intervals initially marked as reffy or not according to their appearance in the
/// reftyped_vregs array, computes the transitive closure of reftypeness over the set of moves
/// present in the program.
fn do_reftype_analysis(
    move_info: &MoveInfo,
    fixed_intervals: &mut [FixedInterval],
    virtual_intervals: &mut [VirtualInterval],
    vreg_to_vranges: &VirtualRegToRanges,
    frag_env: &[RangeFrag],
    stackmap_request: &StackmapRequestInfo,
) {
    let reftyped_vregs = &stackmap_request.reftyped_vregs;
    let reftype_class = stackmap_request.reftype_class;
    let mut analysis = LsraReftypeAnalysis {
        fixed_intervals,
        virtual_intervals,
        vreg_to_vranges,
        frag_env,
    };
    core_reftypes_analysis(&mut analysis, move_info, reftype_class, reftyped_vregs);
}

/// Calculate all the RangeFrags for `bix`.  Add them to `out_frags` and
/// corresponding metrics data to `out_frag_metrics`.  Add to `out_map`, the
/// associated RangeFragIxs, segregated by Reg.  `bix`, `livein`, `liveout` and
/// `rvb` are expected to be valid in the context of the Func `f` (duh!).
#[inline(never)]
fn get_range_frags_for_block<F: Function>(
    func: &F,
    rvb: &RegVecsAndBounds,
    reg_universe: &RealRegUniverse,
    vreg_classes: &Vec<RegClass>,
    safepoints: Option<&[InstIx]>,
    bix: BlockIx,
    livein: &SparseSet<Reg>,
    liveout: &SparseSet<Reg>,
    // Temporary state reusable across function calls.
    visited: &mut Vec<u32>,
    state: &mut Vec</*rreg index, then vreg index, */ Option<RangeFrag>>,
    // Effectively results.
    out_map: &mut Vec<SmallVec<[RangeFragIx; 8]>>,
    out_frags: &mut Vec<RangeFrag>,
    out_frag_metrics: &mut Vec<RangeFragMetrics>,
) {
    // Iterate to the next safepoint contained in this block.
    let first_sp_ix = {
        let mut first = None;
        if let Some(safepoints) = safepoints {
            let first_block_iix = func.block_insns(bix).first();
            for (i, sp_iix) in safepoints.iter().enumerate() {
                if *sp_iix >= first_block_iix {
                    first = Some(i);
                    break;
                }
            }
        }
        first
    };

    let mut emit_range_frag =
        |r: Reg, mut frag: RangeFrag, frag_metrics: RangeFragMetrics, num_real_regs: u32| {
            // Make a list of all the safepoints present in this range.
            if let (Some(ref safepoints), Some(first_sp_ix)) = (safepoints, first_sp_ix) {
                let mut sp_ix = first_sp_ix;
                while let Some(sp_iix) = safepoints.get(sp_ix) {
                    if InstPoint::new_use(*sp_iix) >= frag.first {
                        break;
                    }
                    sp_ix += 1;
                }
                while let Some(sp_iix) = safepoints.get(sp_ix) {
                    if InstPoint::new_use(*sp_iix) > frag.last {
                        break;
                    }
                    frag.safepoints.push((safepoints[sp_ix], sp_ix));
                    sp_ix += 1;
                }
            }

            let fix = RangeFragIx::new(out_frags.len() as u32);
            out_frags.push(frag);
            out_frag_metrics.push(frag_metrics);

            let out_map_index = reg_to_reg_ix(num_real_regs, r) as usize;
            out_map[out_map_index].push(fix);
        };

    // Some handy constants.
    debug_assert!(func.block_insns(bix).len() >= 1);
    let first_pt_in_block = InstPoint::new_use(func.block_insns(bix).first());
    let last_pt_in_block = InstPoint::new_def(func.block_insns(bix).last());

    // Clear the running state.
    visited.clear();

    let num_real_regs = reg_universe.regs.len() as u32;

    // First, set up `state` as if all of `livein` had been written just prior to the block.
    for reg in livein.iter() {
        let reg_state_ix = reg_to_reg_ix(num_real_regs, *reg) as usize;
        debug_assert!(state[reg_state_ix].is_none());
        state[reg_state_ix] = Some(RangeFrag {
            mentions: MentionMap::new(),
            first: first_pt_in_block,
            last: first_pt_in_block,
            safepoints: Default::default(),
            ref_typed: false,
        });
        visited.push(reg_state_ix as u32);
    }

    // Now visit each instruction in turn, examining first the registers it reads, then those it
    // modifies, and finally those it writes.
    for inst_ix in func.block_insns(bix) {
        let bounds_for_inst = &rvb.bounds[inst_ix];

        // Examine reads: they extend an existing RangeFrag to the U point of the reading
        // insn.
        for i in bounds_for_inst.uses_start as usize
            ..bounds_for_inst.uses_start as usize + bounds_for_inst.uses_len as usize
        {
            let reg = &rvb.vecs.uses[i];
            let reg_state_ix = reg_to_reg_ix(num_real_regs, *reg) as usize;

            let prev_frag = state[reg_state_ix]
                .as_mut()
                .expect("trying to use a register not defined or listed in liveins");

            // This the first or subsequent read after a write.  Note that the "write" can be
            // either a real write, or due to the fact that `r` is listed in `livein`.  We don't
            // care here.
            let new_last = InstPoint::new_use(inst_ix);
            debug_assert!(prev_frag.last <= new_last);
            prev_frag.last = new_last;

            // This first loop iterates over all the uses for the first time, so there shouldn't be
            // any duplicates.
            debug_assert!(!prev_frag.mentions.iter().any(|tuple| tuple.0 == inst_ix));
            let mut mention_set = Mention::new();
            mention_set.add_use();
            prev_frag.mentions.push((inst_ix, mention_set));
        }

        // Examine modifies.  These are handled almost identically to reads, except that they
        // extend an existing RangeFrag down to the D point of the modifying insn.
        for i in bounds_for_inst.mods_start as usize
            ..bounds_for_inst.mods_start as usize + bounds_for_inst.mods_len as usize
        {
            let reg = &rvb.vecs.mods[i];
            let reg_state_ix = reg_to_reg_ix(num_real_regs, *reg) as usize;

            let prev_frag = state[reg_state_ix]
                .as_mut()
                .expect("trying to mod a register not defined or listed in liveins");

            // This the first or subsequent modify after a write.
            let new_last = InstPoint::new_def(inst_ix);
            debug_assert!(prev_frag.last <= new_last);
            prev_frag.last = new_last;

            prev_frag.mentions.push((inst_ix, {
                let mut mention_set = Mention::new();
                mention_set.add_mod();
                mention_set
            }));
        }

        // Examine writes (but not writes implied by modifies).  The general idea is that a write
        // causes us to terminate the existing RangeFrag, if any, add it to the results,
        // and start a new frag.
        for i in bounds_for_inst.defs_start as usize
            ..bounds_for_inst.defs_start as usize + bounds_for_inst.defs_len as usize
        {
            let reg = &rvb.vecs.defs[i];
            let reg_state_ix = reg_to_reg_ix(num_real_regs, *reg) as usize;

            match &mut state[reg_state_ix] {
                // First mention of a Reg we've never heard of before.
                // Start a new RangeFrag for it and keep going.
                None => {
                    let new_pt = InstPoint::new_def(inst_ix);
                    let mut mention_set = Mention::new();
                    mention_set.add_def();
                    state[reg_state_ix] = Some(RangeFrag {
                        first: new_pt,
                        last: new_pt,
                        mentions: smallvec![(inst_ix, mention_set)],
                        ref_typed: false,
                        safepoints: smallvec![],
                    })
                }

                // There's already a RangeFrag for `r`.  This write will start a new one, so
                // flush the existing one and note this write.
                Some(RangeFrag {
                    ref mut first,
                    ref mut last,
                    ref mut mentions,
                    ref mut ref_typed,
                    ref mut safepoints,
                }) => {
                    // Steal the mentions and replace the mutable ref by an empty vector for reuse.
                    let stolen_mentions = mem::replace(mentions, MentionMap::new());
                    let stolen_safepoints = mem::replace(safepoints, Default::default());

                    let (frag, frag_metrics) = RangeFrag::new(
                        func,
                        bix,
                        *first,
                        *last,
                        stolen_mentions,
                        *ref_typed,
                        stolen_safepoints,
                    );
                    emit_range_frag(*reg, frag, frag_metrics, num_real_regs);

                    let mut mention_set = Mention::new();
                    mention_set.add_def();
                    mentions.push((inst_ix, mention_set));

                    // Reuse the previous entry for this new definition of the same vreg.
                    let new_pt = InstPoint::new_def(inst_ix);
                    *first = new_pt;
                    *last = new_pt;
                }
            }

            visited.push(reg_state_ix as u32);
        }
    }

    // We are at the end of the block.  We still have to deal with live-out Regs.  We must also
    // deal with RangeFrag in `state` that are for registers not listed as live-out.

    // Deal with live-out Regs.  Treat each one as if it is read just after the block.
    for reg in liveout.iter() {
        // Remove the entry from `state` so that the following loop doesn't process it again.
        let reg_state_ix = reg_to_reg_ix(num_real_regs, *reg) as usize;
        let prev_frag = mem::replace(&mut state[reg_state_ix], None)
            .expect("a liveout register must have been defined before");
        let (frag, frag_metrics) = RangeFrag::new(
            func,
            bix,
            prev_frag.first,
            last_pt_in_block,
            prev_frag.mentions,
            prev_frag.ref_typed,
            prev_frag.safepoints,
        );
        emit_range_frag(*reg, frag, frag_metrics, num_real_regs);
    }

    // Finally, round up any remaining RangeFrag left in `state`.
    for r_state_ix in visited {
        if let Some(prev_frag) = &mut state[*r_state_ix as usize] {
            let r = reg_ix_to_reg(reg_universe, vreg_classes, *r_state_ix);
            let (frag, frag_metrics) = RangeFrag::new(
                func,
                bix,
                prev_frag.first,
                prev_frag.last,
                mem::replace(&mut prev_frag.mentions, MentionMap::new()),
                prev_frag.ref_typed,
                mem::replace(&mut prev_frag.safepoints, Default::default()),
            );
            emit_range_frag(r, frag, frag_metrics, num_real_regs);
            state[*r_state_ix as usize] = None;
        }
    }
}

#[inline(never)]
fn get_range_frags<F: Function>(
    func: &F,
    reg_universe: &RealRegUniverse,
    safepoints: Option<&[InstIx]>,
    rvb: &RegVecsAndBounds,
    liveins: &TypedIxVec<BlockIx, SparseSet<Reg>>,
    liveouts: &TypedIxVec<BlockIx, SparseSet<Reg>>,
) -> (
    Vec</*rreg index, then vreg index, */ SmallVec<[RangeFragIx; 8]>>,
    Vec<RangeFrag>,
    Vec<RangeFragMetrics>,
    Vec</*vreg index,*/ RegClass>,
) {
    trace!("    get_range_frags: begin");
    debug_assert!(liveins.len() == func.blocks().len() as u32);
    debug_assert!(liveouts.len() == func.blocks().len() as u32);
    debug_assert!(rvb.is_sanitized());

    let mut vreg_classes = vec![RegClass::INVALID; func.get_num_vregs()];
    for r in rvb
        .vecs
        .uses
        .iter()
        .chain(rvb.vecs.defs.iter())
        .chain(rvb.vecs.mods.iter())
    {
        if r.is_real() {
            continue;
        }
        let r_ix = r.get_index();
        let vreg_classes_ptr = &mut vreg_classes[r_ix];
        if *vreg_classes_ptr == RegClass::INVALID {
            *vreg_classes_ptr = r.get_class();
        } else {
            debug_assert_eq!(*vreg_classes_ptr, r.get_class());
        }
    }

    let num_real_regs = reg_universe.regs.len();
    let num_virtual_regs = vreg_classes.len();
    let num_regs = num_real_regs + num_virtual_regs;

    // Reused by the function below.
    let mut tmp_state = vec![None; num_regs];
    let mut tmp_visited = Vec::with_capacity(32);

    let mut result_map = vec![SmallVec::new(); num_regs];
    let mut result_frags = Vec::new();
    let mut result_frag_metrics = Vec::new();
    for bix in func.blocks() {
        get_range_frags_for_block(
            func,
            &rvb,
            reg_universe,
            &vreg_classes,
            safepoints,
            bix,
            &liveins[bix],
            &liveouts[bix],
            &mut tmp_visited,
            &mut tmp_state,
            &mut result_map,
            &mut result_frags,
            &mut result_frag_metrics,
        );
    }

    assert!(tmp_state.len() == num_regs);
    assert!(result_map.len() == num_regs);
    assert!(vreg_classes.len() == num_virtual_regs);
    // This is pretty cheap (once per fn) and any failure will be catastrophic since it means we
    // may have forgotten some live range fragments.  Hence `assert!` and not `debug_assert!`.
    for state_elem in &tmp_state {
        assert!(state_elem.is_none());
    }

    if log_enabled!(Level::Trace) {
        trace!("");
        let mut n = 0;
        for frag in result_frags.iter() {
            trace!("{:<3?}   {:?}", RangeFragIx::new(n), frag);
            n += 1;
        }

        trace!("");
        for (reg_ix, frag_ixs) in result_map.iter().enumerate() {
            if frag_ixs.len() == 0 {
                continue;
            }
            let reg = reg_ix_to_reg(reg_universe, &vreg_classes, reg_ix as u32);
            trace!(
                "frags for {}   {:?}",
                reg.show_with_rru(reg_universe),
                frag_ixs
            );
        }
    }

    trace!("    get_range_frags: end");
    assert!(result_frags.len() == result_frag_metrics.len());

    (result_map, result_frags, result_frag_metrics, vreg_classes)
}

#[derive(Clone)]
struct SimplifiedVirtualRange {
    int: IntId,
    frag_ixs: SmallVec<[RangeFragIx; 4]>,
}

type VirtualRegToRanges = Vec<SmallVec<[SimplifiedVirtualRange; 4]>>;

#[inline(never)]
fn merge_range_frags<F: Function>(
    func: &F,
    reg_universe: &RealRegUniverse,
    frag_ix_vec_per_reg: &[SmallVec<[RangeFragIx; 8]>],
    frag_env: &mut Vec<RangeFrag>,
    frag_metrics_env: &Vec<RangeFragMetrics>,
    cfg_info: &CFGInfo,
    vreg_classes: &Vec</*vreg index,*/ RegClass>,
    wants_stackmaps: bool,
) -> Result<(Vec<FixedInterval>, Vec<VirtualInterval>, VirtualRegToRanges), AnalysisError> {
    trace!("    merge_range_frags: begin");
    if log_enabled!(Level::Trace) {
        let mut stats_num_total_incoming_frags = 0;
        for all_frag_ixs_for_reg in frag_ix_vec_per_reg.iter() {
            stats_num_total_incoming_frags += all_frag_ixs_for_reg.len();
        }
        trace!("      in: {} in frag_env", frag_env.len());
        trace!(
            "      in: {} regs containing in total {} frags",
            frag_ix_vec_per_reg.len(),
            stats_num_total_incoming_frags
        );
    }

    debug_assert!(frag_env.len() == frag_metrics_env.len());

    let mut vreg_to_vranges: VirtualRegToRanges = vec![
        SmallVec::<[SimplifiedVirtualRange; 4]>::new(
        );
        if wants_stackmaps {
            func.get_num_vregs()
        } else {
            0
        }
    ];

    // Prefill fixed intervals, one per real register.
    let mut result_fixed = Vec::with_capacity(reg_universe.regs.len() as usize);
    for rreg in reg_universe.regs.iter() {
        result_fixed.push(FixedInterval {
            reg: rreg.0,
            frags: Vec::new(),
        });
    }

    let mut result_virtual = Vec::new();

    let mut triples = Vec::<(RangeFragIx, RangeFragKind, BlockIx)>::new();

    // BEGIN per_reg_loop
    for (reg_ix, all_frag_ixs_for_reg) in frag_ix_vec_per_reg.iter().enumerate() {
        let reg = reg_ix_to_reg(reg_universe, vreg_classes, reg_ix as u32);

        let num_reg_frags = all_frag_ixs_for_reg.len();

        // The reg might never have been mentioned at all, especially if it's a real reg.
        if num_reg_frags == 0 {
            continue;
        }

        // Do some shortcutting.  First off, if there's only one frag for this reg, we can directly
        // give it its own live range, and have done.
        if num_reg_frags == 1 {
            flush_interval(
                &mut result_fixed,
                &mut result_virtual,
                &mut vreg_to_vranges,
                wants_stackmaps,
                reg,
                all_frag_ixs_for_reg,
                &frag_metrics_env,
                frag_env,
            )?;
            continue;
        }

        // BEGIN merge `all_frag_ixs_for_reg` entries as much as possible.
        // but .. if we come across independents (RangeKind::Local), pull them out
        // immediately.
        triples.clear();

        // Create `triples`.  We will use it to guide the merging phase, but it is immutable there.
        for fix in all_frag_ixs_for_reg {
            let frag_metrics = &frag_metrics_env[fix.get() as usize];

            if frag_metrics.kind == RangeFragKind::Local {
                // This frag is Local (standalone).  Give it its own Range and move on.  This is an
                // optimisation, but it's also necessary: the main fragment-merging logic below
                // relies on the fact that the fragments it is presented with are all either
                // LiveIn, LiveOut or Thru.
                flush_interval(
                    &mut result_fixed,
                    &mut result_virtual,
                    &mut vreg_to_vranges,
                    wants_stackmaps,
                    reg,
                    &[*fix],
                    &frag_metrics_env,
                    frag_env,
                )?;
                continue;
            }

            // This frag isn't Local (standalone) so we have to process it the slow way.
            triples.push((*fix, frag_metrics.kind, frag_metrics.bix));
        }

        let triples_len = triples.len();

        // This is the core of the merging algorithm.
        //
        // For each ix@(fix, kind, bix) in `triples` (order unimportant):
        //
        // (1) "Merge with blocks that are live 'downstream' from here":
        //     if fix is live-out or live-through:
        //        for b in succs[bix]
        //           for each ix2@(fix2, kind2, bix2) in `triples`
        //              if bix2 == b && kind2 is live-in or live-through:
        //                  merge(ix, ix2)
        //
        // (2) "Merge with blocks that are live 'upstream' from here":
        //     if fix is live-in or live-through:
        //        for b in preds[bix]
        //           for each ix2@(fix2, kind2, bix2) in `triples`
        //              if bix2 == b && kind2 is live-out or live-through:
        //                  merge(ix, ix2)
        //
        // `triples` remains unchanged.  The equivalence class info is accumulated
        // in `eclasses_uf` instead.  `eclasses_uf` entries are indices into
        // `triples`.
        //
        // Now, you might think it necessary to do both (1) and (2).  But no, they
        // are mutually redundant, since if two blocks are connected by a live
        // flow from one to the other, then they are also connected in the other
        // direction.  Hence checking one of the directions is enough.
        let mut eclasses_uf = UnionFind::<usize>::new(triples_len);

        // We have two schemes for group merging, one of which is N^2 in the
        // length of triples, the other is N-log-N, but with higher constant
        // factors.  Some experimentation with the bz2 test on a Cortex A57 puts
        // the optimal crossover point between 200 and 300; it's not critical.
        // Having this protects us against bad behaviour for huge inputs whilst
        // still being fast for small inputs.
        if triples_len <= 250 {
            // The simple way, which is N^2 in the length of `triples`.
            for (ix, (_fix, kind, bix)) in triples.iter().enumerate() {
                // Deal with liveness flows outbound from `fix`. Meaning, (1) above.
                if *kind == RangeFragKind::LiveOut || *kind == RangeFragKind::Thru {
                    for b in cfg_info.succ_map[*bix].iter() {
                        // Visit all entries in `triples` that are for `b`.
                        for (ix2, (_fix2, kind2, bix2)) in triples.iter().enumerate() {
                            if *bix2 != *b || *kind2 == RangeFragKind::LiveOut {
                                continue;
                            }
                            debug_assert!(
                                *kind2 == RangeFragKind::LiveIn || *kind2 == RangeFragKind::Thru
                            );
                            // Now we know that liveness for this reg "flows" from `triples[ix]` to
                            // `triples[ix2]`.  So those two frags must be part of the same live
                            // range.  Note this.
                            if ix != ix2 {
                                eclasses_uf.union(ix, ix2); // Order of args irrelevant
                            }
                        }
                    }
                }
            } // outermost iteration over `triples`
        } else {
            // The more complex way, which is N-log-N in the length of `triples`.  This is the same
            // as the simple way, except that the innermost loop, which is a linear search in
            // `triples` to find entries for some block `b`, is replaced by a binary search.  This
            // means that `triples` first needs to be sorted by block index.
            triples.sort_unstable_by_key(|(_, _, bix)| *bix);

            for (ix, (_fix, kind, bix)) in triples.iter().enumerate() {
                // Deal with liveness flows outbound from `fix`.  Meaning, (1) above.
                if *kind == RangeFragKind::LiveOut || *kind == RangeFragKind::Thru {
                    for b in cfg_info.succ_map[*bix].iter() {
                        // Visit all entries in `triples` that are for `b`.  Binary search
                        // `triples` to find the lowest-indexed entry for `b`.
                        let mut ix_left = 0;
                        let mut ix_right = triples_len;
                        while ix_left < ix_right {
                            let m = (ix_left + ix_right) >> 1;
                            if triples[m].2 < *b {
                                ix_left = m + 1;
                            } else {
                                ix_right = m;
                            }
                        }

                        // It might be that there is no block for `b` in the sequence.  That's
                        // legit; it just means that block `bix` jumps to a successor where the
                        // associated register isn't live-in/thru.  A failure to find `b` can be
                        // indicated one of two ways:
                        //
                        // * ix_left == triples_len
                        // * ix_left < triples_len and b < triples[ix_left].b
                        //
                        // In both cases I *think* the 'loop_over_entries_for_b below will not do
                        // anything.  But this is all a bit hairy, so let's convert the second
                        // variant into the first, so as to make it obvious that the loop won't do
                        // anything.

                        // ix_left now holds the lowest index of any `triples` entry for block `b`.
                        // Assert this.
                        if ix_left < triples_len && *b < triples[ix_left].2 {
                            ix_left = triples_len;
                        }
                        if ix_left < triples_len {
                            assert!(ix_left == 0 || triples[ix_left - 1].2 < *b);
                        }

                        // ix2 plays the same role as in the quadratic version.  ix_left and
                        // ix_right are not used after this point.
                        let mut ix2 = ix_left;
                        loop {
                            let (_fix2, kind2, bix2) = match triples.get(ix2) {
                                None => break,
                                Some(triple) => *triple,
                            };
                            if *b < bix2 {
                                // We've come to the end of the sequence of `b`-blocks.
                                break;
                            }
                            debug_assert!(*b == bix2);
                            if kind2 == RangeFragKind::LiveOut {
                                ix2 += 1;
                                continue;
                            }
                            // Now we know that liveness for this reg "flows" from `triples[ix]` to
                            // `triples[ix2]`.  So those two frags must be part of the same live
                            // range.  Note this.
                            eclasses_uf.union(ix, ix2);
                            ix2 += 1;
                        }

                        if ix2 + 1 < triples_len {
                            debug_assert!(*b < triples[ix2 + 1].2);
                        }
                    }
                }
            }
        }

        // Now `eclasses_uf` contains the results of the merging-search.  Visit each of its
        // equivalence classes in turn, and convert each into a virtual or real live range as
        // appropriate.
        let eclasses = eclasses_uf.get_equiv_classes();
        for leader_triple_ix in eclasses.equiv_class_leaders_iter() {
            // `leader_triple_ix` is an eclass leader.  Enumerate the whole eclass.
            let mut frag_ixs = SmallVec::<[RangeFragIx; 4]>::new();
            for triple_ix in eclasses.equiv_class_elems_iter(leader_triple_ix) {
                frag_ixs.push(triples[triple_ix].0 /*first field is frag ix*/);
            }
            flush_interval(
                &mut result_fixed,
                &mut result_virtual,
                &mut vreg_to_vranges,
                wants_stackmaps,
                reg,
                &frag_ixs,
                &frag_metrics_env,
                frag_env,
            )?;
        }
        // END merge `all_frag_ixs_for_reg` entries as much as possible
    } // END per reg loop

    // Sort each vrange's fragments by start point, so as to be able to perform binary search later
    // on when doing the reftype analysis.
    for entry in vreg_to_vranges.iter_mut() {
        for vrange in entry {
            vrange
                .frag_ixs
                .sort_unstable_by_key(|&fix| frag_env[fix.get() as usize].first);
        }
    }

    trace!("    merge_range_frags: end");

    Ok((result_fixed, result_virtual, vreg_to_vranges))
}

#[inline(never)]
fn flush_interval(
    result_real: &mut Vec<FixedInterval>,
    result_virtual: &mut Vec<VirtualInterval>,
    vreg_to_vranges: &mut VirtualRegToRanges,
    wants_stackmaps: bool,
    reg: Reg,
    frag_ixs: &[RangeFragIx],
    metrics: &[RangeFragMetrics],
    frags: &mut Vec<RangeFrag>,
) -> Result<(), AnalysisError> {
    if reg.is_real() {
        // Append all the RangeFrags to this fixed interval. They'll get sorted later.
        let fixed_int = &mut result_real[reg.to_real_reg().get_index()];
        fixed_int.frags.reserve(frag_ixs.len());
        for &frag_ix in frag_ixs {
            let frag = &mut frags[frag_ix.get() as usize];
            fixed_int.frags.push(RangeFrag {
                first: frag.first,
                last: frag.last,
                mentions: mem::replace(&mut frag.mentions, MentionMap::new()),
                ref_typed: false,
                safepoints: Default::default(),
            })
        }
        return Ok(());
    }

    debug_assert!(reg.is_virtual());

    let (start, end, mentions, block_boundaries, safepoints) = {
        // Merge all the mentions together.
        let capacity = frag_ixs
            .iter()
            .map(|fix| frags[fix.get() as usize].mentions.len())
            .sum();

        let mut start = InstPoint::max_value();
        let mut end = InstPoint::min_value();

        // Merge all the register mentions and safepoints together.

        // TODO rework this!
        let mut mentions = MentionMap::with_capacity(capacity);
        let mut safepoints: Safepoints = Default::default();
        for frag in frag_ixs.iter().map(|fix| &frags[fix.get() as usize]) {
            mentions.extend(frag.mentions.iter().cloned());
            safepoints.extend(frag.safepoints.iter().cloned());
            start = InstPoint::min(start, frag.first);
            end = InstPoint::max(end, frag.last);
        }
        safepoints.sort_unstable_by_key(|tuple| tuple.0);
        mentions.sort_unstable_by_key(|tuple| tuple.0);

        // Merge mention set that are at the same instruction.
        let mut s = 0;
        let mut e;
        let mut to_remove = Vec::new();
        while s < mentions.len() {
            e = s;
            while e + 1 < mentions.len() && mentions[s].0 == mentions[e + 1].0 {
                e += 1;
            }
            if s != e {
                let mut i = s + 1;
                while i <= e {
                    if mentions[i].1.is_use() {
                        mentions[s].1.add_use();
                    }
                    if mentions[i].1.is_mod() {
                        mentions[s].1.add_mod();
                    }
                    if mentions[i].1.is_def() {
                        mentions[s].1.add_def();
                    }
                    i += 1;
                }
                for i in s + 1..=e {
                    to_remove.push(i);
                }
            }
            s = e + 1;
        }

        for &i in to_remove.iter().rev() {
            // TODO not efficient.
            mentions.remove(i);
        }

        // Retrieve all the block boundary information from the range metrics.

        let mut block_boundaries = Vec::new();
        for fix in frag_ixs.iter() {
            let metric = &metrics[fix.get() as usize];
            let bix = metric.bix;
            // Unfortunately, the RangeFragKind are imprecise: e.g. LiveIn means that the
            // fragment's first instruction coincides with the block's first instruction, not that
            // it really is live in. Moreover, we could use the livein/liveout sets to figure this
            // out more precisely, but they could refer to another Interval for the same vreg. So
            // we end up storing slightly more boundaries that we ought to: it's not a correctness
            // issue: the resolve_moves pass may skip such block boundaries when it requires those.
            match metric.kind {
                RangeFragKind::Local => {}
                RangeFragKind::LiveIn => {
                    block_boundaries.push(BlockBoundary {
                        bix,
                        pos: BlockPos::Start,
                    });
                }
                RangeFragKind::LiveOut => {
                    block_boundaries.push(BlockBoundary {
                        bix,
                        pos: BlockPos::End,
                    });
                }
                RangeFragKind::Thru => {
                    block_boundaries.push(BlockBoundary {
                        bix,
                        pos: BlockPos::Start,
                    });
                    block_boundaries.push(BlockBoundary {
                        bix,
                        pos: BlockPos::End,
                    });
                }
            }
        }

        // Lexicographic sort: first by block index, then by position.
        block_boundaries.sort_unstable();

        (start, end, mentions, block_boundaries, safepoints)
    };

    // If any frag associated to this interval has been marked as reftyped, this is reftyped.
    let ref_typed = frag_ixs
        .iter()
        .any(|fix| frags[fix.get() as usize].ref_typed);

    let id = IntId(result_virtual.len());
    let mut int = VirtualInterval::new(
        id,
        reg.to_virtual_reg(),
        start,
        end,
        mentions,
        block_boundaries,
        ref_typed,
        safepoints,
    );
    int.ancestor = Some(id);

    result_virtual.push(int);

    if wants_stackmaps {
        vreg_to_vranges[reg.get_index()].push(SimplifiedVirtualRange {
            int: id,
            frag_ixs: frag_ixs.into_iter().cloned().collect(),
        })
    }

    Ok(())
}
