//! Top level module for all analysis activities.

use log::{log_enabled, trace, Level};

use crate::data_structures::*;
use crate::sparse_set::SparseSet;
use crate::AlgorithmWithDefaults;
use crate::{
    analysis_control_flow::{CFGInfo, InstIxToBlockIxMap},
    analysis_reftypes::ReftypeAnalysis,
};
use crate::{
    analysis_data_flow::{
        calc_def_and_use, calc_livein_and_liveout, collect_move_info, compute_reg_to_ranges_maps,
        get_range_frags, get_sanitized_reg_uses_for_func, merge_range_frags,
    },
    analysis_reftypes::core_reftypes_analysis,
};
use crate::{Function, Reg};

//=============================================================================
// Overall analysis return results, for both control- and data-flow analyses.
// All of these failures refer to various problems with the code that the
// client (caller) supplied to us.

#[derive(Clone, Debug)]
pub enum AnalysisError {
    /// A critical edge from "from" to "to" has been found, and should have been
    /// removed by the caller in the first place.
    CriticalEdge { from: BlockIx, to: BlockIx },

    /// Some values in the entry block are live in to the function, but are not
    /// declared as such.
    EntryLiveinValues(Vec<Reg>),

    /// The incoming code has an explicit or implicit mention (use, def or mod)
    /// of a real register, which either (1) isn't listed in the universe at
    /// all, or (2) is one of the `suggested_scratch` registers in the universe.
    /// (1) isn't allowed because the client must mention *all* real registers
    /// in the universe.  (2) isn't allowed because the client promises to us
    /// that the `suggested_scratch` registers really are completely unused in
    /// the incoming code, so that the allocator can use them at literally any
    /// point it wants.
    IllegalRealReg(RealReg),

    /// At least one block is dead.
    UnreachableBlocks,

    /// Implementation limits exceeded.  The incoming function is too big.  It
    /// may contain at most 1 million basic blocks and 16 million instructions.
    ImplementationLimitsExceeded,

    /// Linear scan requires that if a block ends with a control flow
    /// instruction that has at least one register mention (use, mod or def),
    /// then the successor blocks must have a single predecessor.
    ///
    /// In practice, this means that users should consider associated edges to
    /// be "critical" and split them (and maybe remove dead blocks afterwards).
    ///
    /// For details, see the comment in linear_scan::analysis generating this
    /// error.
    LsraCriticalEdge { block: BlockIx, inst: InstIx },
}

impl ToString for AnalysisError {
    fn to_string(&self) -> String {
        match self {
            AnalysisError::CriticalEdge { from, to } => {
                format!("critical edge detected, from {:?} to {:?}", from, to)
            }
            AnalysisError::EntryLiveinValues(regs) => {
                let regs_string = regs
                    .iter()
                    .map(|reg| format!("{:?}", reg))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "entry block has love-in value not present in function liveins: {}",
                    regs_string
                )
            }
            AnalysisError::IllegalRealReg(reg) => format!(
                "instructions mention real register {:?}, which either isn't defined in the
                    register universe, or is a 'suggested_scratch' register",
                reg
            ),
            AnalysisError::UnreachableBlocks => "at least one block is unreachable".to_string(),
            AnalysisError::ImplementationLimitsExceeded => {
                "implementation limits exceeded (more than 1 million blocks or 16 million insns)"
                    .to_string()
            }
            AnalysisError::LsraCriticalEdge { block, inst } => format!(
                "block {:?} ends with control flow instruction {:?} that mentions a register,
                    and at least one of the multiple successors has several predecessors; consider
                    splitting the outgoing edges!",
                block, inst
            ),
        }
    }
}

//=============================================================================
// Top level for all analysis activities.

pub struct AnalysisInfo {
    /// The sanitized per-insn reg-use info
    pub(crate) reg_vecs_and_bounds: RegVecsAndBounds,
    /// The real-reg live ranges
    pub(crate) real_ranges: TypedIxVec<RealRangeIx, RealRange>,
    /// The virtual-reg live ranges
    pub(crate) virtual_ranges: TypedIxVec<VirtualRangeIx, VirtualRange>,
    /// The fragment table
    pub(crate) range_frags: TypedIxVec<RangeFragIx, RangeFrag>,
    /// The fragment metrics table
    pub(crate) range_metrics: TypedIxVec<RangeFragIx, RangeFragMetrics>,
    /// Estimated execution frequency per block
    pub(crate) estimated_frequencies: DepthBasedFrequencies,
    /// Maps InstIxs to BlockIxs
    pub(crate) inst_to_block_map: InstIxToBlockIxMap,
    /// Maps from RealRegs to sets of RealRanges and VirtualRegs to sets of VirtualRanges
    /// (all operating on indices, not the actual objects).  This is only generated in
    /// situations where we need it, hence the `Option`.
    pub(crate) reg_to_ranges_maps: Option<RegToRangesMaps>,
    /// Information about registers connected by moves.  This is only generated in situations
    /// where we need it, hence the `Option`.
    pub(crate) move_info: Option<MoveInfo>,
}

#[inline(never)]
pub fn run_analysis<F: Function>(
    func: &F,
    reg_universe: &RealRegUniverse,
    algorithm: AlgorithmWithDefaults,
    client_wants_stackmaps: bool,
    reftype_class: RegClass,
    reftyped_vregs: &Vec<VirtualReg>, // as supplied by the client
) -> Result<AnalysisInfo, AnalysisError> {
    trace!("run_analysis: begin");
    trace!(
        "  run_analysis: {} blocks, {} insns",
        func.blocks().len(),
        func.insns().len()
    );

    // LSRA uses its own analysis.
    assert!(!client_wants_stackmaps || algorithm != AlgorithmWithDefaults::LinearScan);

    trace!("  run_analysis: begin control flow analysis");

    // First do control flow analysis.  This is (relatively) simple.  Note that
    // this can fail, for various reasons; we propagate the failure if so.
    let cfg_info = CFGInfo::create(func)?;

    // Create the InstIx-to-BlockIx map.  This isn't really control-flow
    // analysis, but needs to be done at some point.
    let inst_to_block_map = InstIxToBlockIxMap::new(func);

    // Annotate each Block with its estimated execution frequency.
    let estimated_frequencies = DepthBasedFrequencies::new(func, &cfg_info);

    trace!("  run_analysis: end control flow analysis");

    // Now perform dataflow analysis.  This is somewhat more complex.
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

    // Verify livein set of entry block against liveins specified by function
    // (e.g., ABI params).
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
    }

    trace!("  run_analysis: end data flow analysis");

    // Dataflow analysis is now complete.  Now compute the virtual and real live
    // ranges, in two steps: (1) compute RangeFrags, and (2) merge them
    // together, guided by flow and liveness info, so as to create the final
    // VirtualRanges and RealRanges.
    trace!("  run_analysis: begin liveness analysis");

    let (frag_ixs_per_reg, frag_env, frag_metrics_env, vreg_classes) = get_range_frags(
        func,
        &reg_vecs_and_bounds,
        &reg_universe,
        &livein_sets_per_block,
        &liveout_sets_per_block,
    );

    // These have to be mut because they may get changed below by the call to
    // `to_reftypes_analysis`.
    let (mut rlr_env, mut vlr_env) = merge_range_frags(
        &frag_ixs_per_reg,
        &frag_env,
        &frag_metrics_env,
        &estimated_frequencies,
        &cfg_info,
        &reg_universe,
        &vreg_classes,
    );

    debug_assert!(liveout_sets_per_block.len() == estimated_frequencies.len());

    if log_enabled!(Level::Trace) {
        trace!("");
        let mut n = 0;
        for rlr in rlr_env.iter() {
            trace!(
                "{:<4?}   {}",
                RealRangeIx::new(n),
                rlr.show_with_rru(&reg_universe)
            );
            n += 1;
        }

        trace!("");
        n = 0;
        for vlr in vlr_env.iter() {
            trace!("{:<4?}   {:?}", VirtualRangeIx::new(n), vlr);
            n += 1;
        }
    }

    // Now a bit of auxiliary info collection, which isn't really either control- or data-flow
    // analysis.

    // For BT and/or reftypes, we'll also need the reg-to-ranges maps and information about moves.
    let (reg_to_ranges_maps, move_info) =
        if client_wants_stackmaps || algorithm == AlgorithmWithDefaults::Backtracking {
            (
                Some(compute_reg_to_ranges_maps(
                    func,
                    &reg_universe,
                    &rlr_env,
                    &vlr_env,
                )),
                Some(collect_move_info(
                    func,
                    &reg_vecs_and_bounds,
                    &estimated_frequencies,
                )),
            )
        } else {
            (None, None)
        };

    trace!("  run_analysis: end liveness analysis");

    if client_wants_stackmaps {
        trace!("  run_analysis: begin reftypes analysis");
        do_reftypes_analysis(
            &mut rlr_env,
            &mut vlr_env,
            &frag_env,
            reg_to_ranges_maps.as_ref().unwrap(), /* safe because of logic just above */
            &move_info.as_ref().unwrap(),         /* ditto */
            reftype_class,
            reftyped_vregs,
        );
        trace!("  run_analysis: end reftypes analysis");
    }

    trace!("run_analysis: end");

    Ok(AnalysisInfo {
        reg_vecs_and_bounds,
        real_ranges: rlr_env,
        virtual_ranges: vlr_env,
        range_frags: frag_env,
        range_metrics: frag_metrics_env,
        estimated_frequencies,
        inst_to_block_map,
        reg_to_ranges_maps,
        move_info,
    })
}

/// A small wrapper for estimated execution frequencies, based on the block's loop depth.
pub(crate) struct DepthBasedFrequencies(TypedIxVec<BlockIx, u32>);

impl DepthBasedFrequencies {
    pub(crate) fn new<F: Function>(func: &F, cfg_info: &CFGInfo) -> Self {
        let mut values = TypedIxVec::new();
        for bix in func.blocks() {
            let mut estimated_frequency = 1;
            let depth = u32::min(cfg_info.depth_map[bix], 3);
            for _ in 0..depth {
                estimated_frequency *= 10;
            }
            assert!(bix == BlockIx::new(values.len()));
            values.push(estimated_frequency);
        }
        Self(values)
    }
    pub(crate) fn len(&self) -> u32 {
        self.0.len()
    }
    pub(crate) fn iter(&self) -> std::slice::Iter<u32> {
        self.0.iter()
    }
    #[inline(always)]
    pub(crate) fn cost(&self, bix: BlockIx) -> u32 {
        self.0[bix]
    }
}

/// Implementation of the reftype analysis for the backtracking algorithm.
struct BacktrackingReftypeAnalysis<'a> {
    rlr_env: &'a mut TypedIxVec<RealRangeIx, RealRange>,
    vlr_env: &'a mut TypedIxVec<VirtualRangeIx, VirtualRange>,
    frag_env: &'a TypedIxVec<RangeFragIx, RangeFrag>,
    reg_to_ranges_maps: &'a RegToRangesMaps,
}

impl<'a> ReftypeAnalysis for BacktrackingReftypeAnalysis<'a> {
    type RangeId = RangeId;

    #[inline(always)]
    fn find_range_id_for_reg(&self, pt: InstPoint, reg: Reg) -> Self::RangeId {
        if reg.is_real() {
            for &rlrix in &self.reg_to_ranges_maps.rreg_to_rlrs_map[reg.get_index() as usize] {
                if self.rlr_env[rlrix]
                    .sorted_frags
                    .contains_pt(self.frag_env, pt)
                {
                    return RangeId::new_real(rlrix);
                }
            }
        } else {
            for &vlrix in &self.reg_to_ranges_maps.vreg_to_vlrs_map[reg.get_index() as usize] {
                if self.vlr_env[vlrix].sorted_frags.contains_pt(pt) {
                    return RangeId::new_virtual(vlrix);
                }
            }
        }
        panic!("do_reftypes_analysis::find_range_for_reg: can't find range");
    }

    #[inline(always)]
    fn mark_reffy(&mut self, range: &Self::RangeId) {
        if range.is_real() {
            let rrange = &mut self.rlr_env[range.to_real()];
            debug_assert!(!rrange.is_ref);
            trace!(" -> rrange {:?} is reffy", range.to_real());
            rrange.is_ref = true;
        } else {
            let vrange = &mut self.vlr_env[range.to_virtual()];
            debug_assert!(!vrange.is_ref);
            trace!(" -> rrange {:?} is reffy", range.to_virtual());
            vrange.is_ref = true;
        }
    }

    #[inline(always)]
    fn insert_reffy_ranges(&self, vreg: VirtualReg, set: &mut SparseSet<Self::RangeId>) {
        for vlr_ix in &self.reg_to_ranges_maps.vreg_to_vlrs_map[vreg.get_index()] {
            trace!("range {:?} is reffy due to reffy vreg {:?}", vlr_ix, vreg);
            set.insert(RangeId::new_virtual(*vlr_ix));
        }
    }
}

fn do_reftypes_analysis(
    // From dataflow/liveness analysis.  Modified by setting their is_ref bit.
    rlr_env: &mut TypedIxVec<RealRangeIx, RealRange>,
    vlr_env: &mut TypedIxVec<VirtualRangeIx, VirtualRange>,
    // From dataflow analysis
    frag_env: &TypedIxVec<RangeFragIx, RangeFrag>,
    reg_to_ranges_maps: &RegToRangesMaps,
    move_info: &MoveInfo,
    // As supplied by the client
    reftype_class: RegClass,
    reftyped_vregs: &Vec<VirtualReg>,
) {
    let mut analysis = BacktrackingReftypeAnalysis {
        rlr_env,
        vlr_env,
        frag_env,
        reg_to_ranges_maps,
    };
    core_reftypes_analysis(&mut analysis, move_info, reftype_class, reftyped_vregs);
}
