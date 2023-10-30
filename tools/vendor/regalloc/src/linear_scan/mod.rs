//! pub(crate) Implementation of the linear scan allocator algorithm.
//!
//! This tries to follow the implementation as suggested by:
//!   Optimized Interval Splitting in a Linear Scan Register Allocator,
//!     by Wimmer et al., 2005

use log::{log_enabled, trace, Level};

use std::env;
use std::fmt;
use std::{cmp::Ordering, default};

use crate::{
    checker::CheckerContext, reg_maps::MentionRegUsageMapper, Function, RealRegUniverse,
    RegAllocError, RegAllocResult, RegClass, Set, SpillSlot, VirtualReg, NUM_REG_CLASSES,
};
use crate::{
    checker::CheckerStackmapInfo,
    inst_stream::{add_spills_reloads_and_moves, InstToInsertAndExtPoint},
};
use crate::{
    data_structures::{BlockIx, InstIx, InstPoint, Point, RealReg, RegVecsAndBounds},
    CheckerErrors, StackmapRequestInfo,
};

use analysis::{AnalysisInfo, RangeFrag};
use smallvec::SmallVec;

use self::analysis::{BlockBoundary, BlockPos};

mod analysis;
mod assign_registers;
mod resolve_moves;

#[derive(Default)]
pub(crate) struct Statistics {
    only_large: bool,

    num_fixed: usize,
    num_vregs: usize,
    num_virtual_ranges: usize,

    peak_active: usize,
    peak_inactive: usize,

    num_try_allocate_reg: usize,
    num_try_allocate_reg_success: usize,

    num_reg_splits: usize,
    num_reg_splits_success: usize,
}

impl Drop for Statistics {
    fn drop(&mut self) {
        if self.only_large && self.num_vregs < 1000 {
            return;
        }
        println!(
            "stats: {} fixed; {} vreg; {} vranges; {} peak-active; {} peak-inactive, {} direct-alloc; {} total-alloc; {} partial-splits; {} partial-splits-attempts",
            self.num_fixed,
            self.num_vregs,
            self.num_virtual_ranges,
            self.peak_active,
            self.peak_inactive,
            self.num_try_allocate_reg_success,
            self.num_try_allocate_reg,
            self.num_reg_splits_success,
            self.num_reg_splits,
        );
    }
}

/// Which strategy should we use when trying to find the best split position?
/// TODO Consider loop depth to avoid splitting in the middle of a loop
/// whenever possible.
#[derive(Copy, Clone, Debug)]
enum OptimalSplitStrategy {
    From,
    To,
    NextFrom,
    NextNextFrom,
    PrevTo,
    PrevPrevTo,
    Mid,
}

#[derive(Clone)]
pub struct LinearScanOptions {
    split_strategy: OptimalSplitStrategy,
    partial_split: bool,
    partial_split_near_end: bool,
    stats: bool,
    large_stats: bool,
}

impl default::Default for LinearScanOptions {
    fn default() -> Self {
        // Useful for debugging.
        let optimal_split_strategy = match env::var("LSRA_SPLIT") {
            Ok(s) => match s.as_str() {
                "t" | "to" => OptimalSplitStrategy::To,
                "n" => OptimalSplitStrategy::NextFrom,
                "nn" => OptimalSplitStrategy::NextNextFrom,
                "p" => OptimalSplitStrategy::PrevTo,
                "pp" => OptimalSplitStrategy::PrevPrevTo,
                "m" | "mid" => OptimalSplitStrategy::Mid,
                _ => OptimalSplitStrategy::From,
            },
            Err(_) => OptimalSplitStrategy::From,
        };

        let large_stats = env::var("LSRA_LARGE_STATS").is_ok();
        let stats = env::var("LSRA_STATS").is_ok() || large_stats;

        let partial_split = env::var("LSRA_PARTIAL").is_ok();
        let partial_split_near_end = env::var("LSRA_PARTIAL_END").is_ok();

        Self {
            split_strategy: optimal_split_strategy,
            partial_split,
            partial_split_near_end,
            stats,
            large_stats,
        }
    }
}

impl fmt::Debug for LinearScanOptions {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "linear scan")?;
        write!(fmt, "  split: {:?}", self.split_strategy)
    }
}

// Local shorthands.
type RegUses = RegVecsAndBounds;

/// A unique identifier for an interval.
#[derive(Clone, Copy, PartialEq, Eq)]
struct IntId(pub(crate) usize);

impl fmt::Debug for IntId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "int{}", self.0)
    }
}

#[derive(Clone)]
struct FixedInterval {
    reg: RealReg,
    frags: Vec<RangeFrag>,
}

impl fmt::Display for FixedInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fixed {:?} [", self.reg)?;
        for (i, frag) in self.frags.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if frag.ref_typed {
                write!(f, "ref ")?;
            }
            write!(f, "({:?}, {:?})", frag.first, frag.last)?;
        }
        write!(f, "]")
    }
}

impl FixedInterval {
    /// Find the fragment that contains the given instruction point.
    /// May crash if the point doesn't belong to any fragment.
    pub(crate) fn find_frag(&self, pt: InstPoint) -> usize {
        self.frags
            .binary_search_by(|frag| {
                if pt < frag.first {
                    Ordering::Greater
                } else if pt >= frag.first && pt <= frag.last {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            })
            .unwrap()
    }
}

type Safepoints = SmallVec<[(InstIx, usize); 8]>;

#[derive(Clone)]
pub(crate) struct VirtualInterval {
    id: IntId,
    vreg: VirtualReg,

    /// Is this interval used for a reference type?
    ref_typed: bool,

    /// Parent interval in the split tree.
    parent: Option<IntId>,
    ancestor: Option<IntId>,
    /// Child interval, if it has one, in the split tree.
    child: Option<IntId>,

    /// Location assigned to this live interval.
    location: Location,

    mentions: MentionMap,
    block_boundaries: Vec<BlockBoundary>,
    safepoints: Safepoints,
    start: InstPoint,
    end: InstPoint,
}

impl fmt::Display for VirtualInterval {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "virtual {:?}", self.id)?;
        if self.ref_typed {
            write!(fmt, " ref")?;
        }
        if let Some(ref p) = self.parent {
            write!(fmt, " (parent={:?})", p)?;
        }
        write!(
            fmt,
            ": {:?} {} [{:?}; {:?}]",
            self.vreg, self.location, self.start, self.end
        )?;
        write!(
            fmt,
            " boundaries=[{}]",
            self.block_boundaries
                .iter()
                .map(|boundary| format!(
                    "{:?}{}",
                    boundary.bix,
                    if boundary.pos == BlockPos::Start {
                        "s"
                    } else {
                        "e"
                    }
                ))
                .collect::<Vec<_>>()
                .join(", ")
        )?;
        if !self.safepoints.is_empty() {
            write!(fmt, " safepoints=[")?;
            for (i, sp) in self.safepoints.iter().enumerate() {
                if i > 0 {
                    write!(fmt, ", {:?}", sp.0)?;
                } else {
                    write!(fmt, "{:?}", sp.0)?;
                }
            }
            write!(fmt, "]")?;
        }
        Ok(())
    }
}

impl VirtualInterval {
    fn new(
        id: IntId,
        vreg: VirtualReg,
        start: InstPoint,
        end: InstPoint,
        mentions: MentionMap,
        block_boundaries: Vec<BlockBoundary>,
        ref_typed: bool,
        safepoints: Safepoints,
    ) -> Self {
        Self {
            id,
            vreg,
            parent: None,
            ancestor: None,
            child: None,
            location: Location::None,
            mentions,
            block_boundaries,
            safepoints,
            start,
            end,
            ref_typed,
        }
    }
    fn safepoints(&self) -> &Safepoints {
        &self.safepoints
    }
    fn safepoints_mut(&mut self) -> &mut Safepoints {
        &mut self.safepoints
    }
    fn mentions(&self) -> &MentionMap {
        &self.mentions
    }
    fn mentions_mut(&mut self) -> &mut MentionMap {
        &mut self.mentions
    }
    fn block_boundaries(&self) -> &[BlockBoundary] {
        &self.block_boundaries
    }
    fn block_boundaries_mut(&mut self) -> &mut Vec<BlockBoundary> {
        &mut self.block_boundaries
    }
    fn covers(&self, pos: InstPoint) -> bool {
        self.start <= pos && pos <= self.end
    }
}

/// This data structure tracks the mentions of a register (virtual or real) at a precise
/// instruction point. It's a set encoded as three flags, one for each of use/mod/def.
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Mention(u8);

impl fmt::Debug for Mention {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut comma = false;
        if self.0 & 1 == 1 {
            write!(fmt, "use")?;
            comma = true;
        }
        if (self.0 >> 1) & 1 == 1 {
            if comma {
                write!(fmt, ",")?;
            }
            write!(fmt, "mod")?;
            comma = true;
        }
        if (self.0 >> 2) & 1 == 1 {
            if comma {
                write!(fmt, ",")?;
            }
            write!(fmt, "def")?;
        }
        Ok(())
    }
}

impl Mention {
    fn new() -> Self {
        Self(0)
    }

    // Setters.
    fn add_use(&mut self) {
        self.0 |= 1 << 0;
    }
    fn add_mod(&mut self) {
        self.0 |= 1 << 1;
    }
    fn add_def(&mut self) {
        self.0 |= 1 << 2;
    }

    // Getters.
    fn is_use(&self) -> bool {
        (self.0 & 0b001) != 0
    }
    fn is_mod(&self) -> bool {
        (self.0 & 0b010) != 0
    }
    fn is_def(&self) -> bool {
        (self.0 & 0b100) != 0
    }
    fn is_use_or_mod(&self) -> bool {
        (self.0 & 0b011) != 0
    }
    fn is_mod_or_def(&self) -> bool {
        (self.0 & 0b110) != 0
    }
}

pub type MentionMap = SmallVec<[(InstIx, Mention); 2]>;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Location {
    None,
    Reg(RealReg),
    Stack(SpillSlot),
}

impl Location {
    pub(crate) fn reg(&self) -> Option<RealReg> {
        match self {
            Location::Reg(reg) => Some(*reg),
            _ => None,
        }
    }
    pub(crate) fn spill(&self) -> Option<SpillSlot> {
        match self {
            Location::Stack(slot) => Some(*slot),
            _ => None,
        }
    }
    pub(crate) fn is_none(&self) -> bool {
        match self {
            Location::None => true,
            _ => false,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::None => write!(fmt, "none"),
            Location::Reg(reg) => write!(fmt, "{:?}", reg),
            Location::Stack(slot) => write!(fmt, "{:?}", slot),
        }
    }
}

/// A group of live intervals.
pub struct Intervals {
    virtuals: Vec<VirtualInterval>,
    fixeds: Vec<FixedInterval>,
}

impl Intervals {
    fn get(&self, int_id: IntId) -> &VirtualInterval {
        &self.virtuals[int_id.0]
    }
    fn get_mut(&mut self, int_id: IntId) -> &mut VirtualInterval {
        &mut self.virtuals[int_id.0]
    }
    fn num_virtual_intervals(&self) -> usize {
        self.virtuals.len()
    }

    // Mutators.
    fn set_reg(&mut self, int_id: IntId, reg: RealReg) {
        let int = self.get_mut(int_id);
        debug_assert!(int.location.is_none());
        int.location = Location::Reg(reg);
    }
    fn set_spill(&mut self, int_id: IntId, slot: SpillSlot) {
        let int = self.get_mut(int_id);
        debug_assert!(int.location.spill().is_none());
        int.location = Location::Stack(slot);
    }
    fn push_interval(&mut self, int: VirtualInterval) {
        debug_assert!(int.id.0 == self.virtuals.len());
        self.virtuals.push(int);
    }
    fn set_child(&mut self, int_id: IntId, child_id: IntId) {
        if let Some(prev_child) = self.virtuals[int_id.0].child.clone() {
            self.virtuals[child_id.0].child = Some(prev_child);
            self.virtuals[prev_child.0].parent = Some(child_id);
        }
        self.virtuals[int_id.0].child = Some(child_id);
    }
}

/// Finds the first use for the current interval that's located after the given
/// `pos` (included), in a broad sense of use (any of use, def or mod).
///
/// Extends to the left, that is, "modified" means "used".
#[inline(never)]
fn next_use(interval: &VirtualInterval, pos: InstPoint, _reg_uses: &RegUses) -> Option<InstPoint> {
    if log_enabled!(Level::Trace) {
        trace!("find next use of {} after {:?}", interval, pos);
    }

    let mentions = interval.mentions();
    let target = InstPoint::max(pos, interval.start);

    let ret = match mentions.binary_search_by_key(&target.iix(), |mention| mention.0) {
        Ok(index) => {
            // Either the selected index is a perfect match, or the next mention is
            // the correct answer.
            let mention = &mentions[index];
            if target.pt() == Point::Use {
                if mention.1.is_use_or_mod() {
                    Some(InstPoint::new_use(mention.0))
                } else {
                    Some(InstPoint::new_def(mention.0))
                }
            } else if target.pt() == Point::Def && mention.1.is_mod_or_def() {
                Some(target)
            } else if index == mentions.len() - 1 {
                None
            } else {
                let mention = &mentions[index + 1];
                if mention.1.is_use_or_mod() {
                    Some(InstPoint::new_use(mention.0))
                } else {
                    Some(InstPoint::new_def(mention.0))
                }
            }
        }

        Err(index) => {
            if index == mentions.len() {
                None
            } else {
                let mention = &mentions[index];
                if mention.1.is_use_or_mod() {
                    Some(InstPoint::new_use(mention.0))
                } else {
                    Some(InstPoint::new_def(mention.0))
                }
            }
        }
    };

    // TODO once the mentions are properly split, this could be removed, in
    // theory.
    let ret = match ret {
        Some(pos) => {
            if pos <= interval.end {
                Some(pos)
            } else {
                None
            }
        }
        None => None,
    };

    ret
}

/// Finds the last use of a vreg before a given target, including it in possible
/// return values.
/// Extends to the right, that is, modified means "def".
#[inline(never)]
fn last_use(interval: &VirtualInterval, pos: InstPoint, _reg_uses: &RegUses) -> Option<InstPoint> {
    if log_enabled!(Level::Trace) {
        trace!("searching last use of {} before {:?}", interval, pos,);
    }

    let mentions = interval.mentions();

    let target = InstPoint::min(pos, interval.end);

    let ret = match mentions.binary_search_by_key(&target.iix(), |mention| mention.0) {
        Ok(index) => {
            // Either the selected index is a perfect match, or the previous mention
            // is the correct answer.
            let mention = &mentions[index];
            if target.pt() == Point::Def {
                if mention.1.is_mod_or_def() {
                    Some(InstPoint::new_def(mention.0))
                } else {
                    Some(InstPoint::new_use(mention.0))
                }
            } else if target.pt() == Point::Use && mention.1.is_use() {
                Some(target)
            } else if index == 0 {
                None
            } else {
                let mention = &mentions[index - 1];
                if mention.1.is_mod_or_def() {
                    Some(InstPoint::new_def(mention.0))
                } else {
                    Some(InstPoint::new_use(mention.0))
                }
            }
        }

        Err(index) => {
            if index == 0 {
                None
            } else {
                let mention = &mentions[index - 1];
                if mention.1.is_mod_or_def() {
                    Some(InstPoint::new_def(mention.0))
                } else {
                    Some(InstPoint::new_use(mention.0))
                }
            }
        }
    };

    // TODO once the mentions are properly split, this could be removed, in
    // theory.
    let ret = match ret {
        Some(pos) => {
            if pos >= interval.start {
                Some(pos)
            } else {
                None
            }
        }
        None => None,
    };

    trace!("mentions: {:?}", mentions);
    trace!("found: {:?}", ret);

    ret
}

/// Checks that each register class has its own scratch register in addition to one available
/// register, and creates a mapping of register class -> scratch register.
fn compute_scratches(
    reg_universe: &RealRegUniverse,
) -> Result<Vec<Option<RealReg>>, RegAllocError> {
    let mut scratches_by_rc = vec![None; NUM_REG_CLASSES];
    for i in 0..NUM_REG_CLASSES {
        if let Some(info) = &reg_universe.allocable_by_class[i] {
            if info.first == info.last {
                return Err(RegAllocError::Other(
                    "at least 2 registers required for linear scan".into(),
                ));
            }
            let scratch = if let Some(suggested_reg) = info.suggested_scratch {
                reg_universe.regs[suggested_reg].0
            } else {
                return Err(RegAllocError::MissingSuggestedScratchReg(
                    RegClass::rc_from_u32(i as u32),
                ));
            };
            scratches_by_rc[i] = Some(scratch);
        }
    }
    Ok(scratches_by_rc)
}

/// Allocator top level.
///
/// `func` is modified so that, when this function returns, it will contain no VirtualReg uses.
///
/// Allocation can fail if there are insufficient registers to even generate spill/reload code, or
/// if the function appears to have any undefined VirtualReg/RealReg uses.
#[inline(never)]
pub(crate) fn run<F: Function>(
    func: &mut F,
    reg_universe: &RealRegUniverse,
    stackmap_request: Option<&StackmapRequestInfo>,
    use_checker: bool,
    opts: &LinearScanOptions,
) -> Result<RegAllocResult<F>, RegAllocError> {
    let AnalysisInfo {
        reg_vecs_and_bounds: reg_uses,
        intervals,
        liveins,
        liveouts,
        cfg,
        ..
    } = analysis::run(func, reg_universe, stackmap_request)
        .map_err(|err| RegAllocError::Analysis(err))?;

    let scratches_by_rc = compute_scratches(reg_universe)?;

    let stats = if opts.stats {
        let mut stats = Statistics::default();
        stats.num_fixed = intervals.fixeds.len();
        stats.num_virtual_ranges = intervals.virtuals.len();
        stats.num_vregs = intervals
            .virtuals
            .iter()
            .map(|virt| virt.vreg.get_index())
            .fold(0, |a, b| usize::max(a, b));
        stats.only_large = opts.large_stats;
        Some(stats)
    } else {
        None
    };

    if log_enabled!(Level::Trace) {
        trace!("fixed intervals:");
        for int in &intervals.fixeds {
            trace!("{}", int);
        }
        trace!("");
        trace!("unassigned intervals:");
        for int in &intervals.virtuals {
            trace!("{}", int);
            for mention in &int.mentions {
                trace!("  mention @ {:?}: {:?}", mention.0, mention.1);
            }
        }
        trace!("");
    }

    let (intervals, mut num_spill_slots) = assign_registers::run(
        opts,
        func,
        &reg_uses,
        reg_universe,
        &scratches_by_rc,
        intervals,
        stats,
    )?;

    let virtuals = &intervals.virtuals;

    let memory_moves = resolve_moves::run(
        func,
        &cfg,
        &reg_uses,
        virtuals,
        &liveins,
        &liveouts,
        &mut num_spill_slots,
        &scratches_by_rc,
    );

    apply_registers(
        func,
        virtuals,
        memory_moves,
        reg_universe,
        num_spill_slots,
        use_checker,
        stackmap_request,
    )
}

#[inline(never)]
fn set_registers<F: Function>(
    func: &mut F,
    virtual_intervals: &Vec<VirtualInterval>,
    reg_universe: &RealRegUniverse,
    use_checker: bool,
    memory_moves: &Vec<InstToInsertAndExtPoint>,
    stackmap_request: Option<&StackmapRequestInfo>,
    stackmaps: &[Vec<SpillSlot>],
) -> Result<Set<RealReg>, CheckerErrors> {
    trace!("set_registers");

    let mut clobbered_registers = Set::empty();

    // Collect all the regs per instruction and mention set.
    let capacity = virtual_intervals
        .iter()
        .map(|int| int.mentions.len())
        .fold(0, |a, b| a + b);

    if capacity == 0 {
        // No virtual registers have been allocated, exit early.
        return Ok(clobbered_registers);
    }

    let mut mention_map = Vec::with_capacity(capacity);

    for int in virtual_intervals {
        let rreg = match int.location.reg() {
            Some(rreg) => rreg,
            _ => continue,
        };
        trace!("int: {}", int);
        trace!("  {:?}", int.mentions);
        for &mention in &int.mentions {
            mention_map.push((mention.0, mention.1, int.vreg, rreg));
        }
    }

    // Sort by instruction index.
    mention_map.sort_unstable_by_key(|quad| quad.0);

    // Iterate over all the mentions.
    let mut mapper = MentionRegUsageMapper::new();

    // Set up checker state, if indicated by our configuration.
    let mut checker: Option<CheckerContext> = None;
    let mut insn_blocks: Vec<BlockIx> = vec![];
    if use_checker {
        let stackmap_info =
            stackmap_request.map(|request| CheckerStackmapInfo { request, stackmaps });
        checker = Some(CheckerContext::new(
            func,
            reg_universe,
            memory_moves,
            stackmap_info,
        ));
        insn_blocks.resize(func.insns().len(), BlockIx::new(0));
        for block_ix in func.blocks() {
            for insn_ix in func.block_insns(block_ix) {
                insn_blocks[insn_ix.get() as usize] = block_ix;
            }
        }
    }

    let mut cur_quad_ix = 0;
    for func_inst_ix in func.insn_indices() {
        // Several items in the mention_map array may refer to the same instruction index, so
        // iterate over all of them that are related to the current instruction index.
        while let Some((iix, mention_set, vreg, rreg)) = mention_map.get(cur_quad_ix) {
            if func_inst_ix != *iix {
                break;
            }

            trace!(
                "{:?}: {:?} is in {:?} at {:?}",
                iix,
                vreg,
                rreg,
                mention_set
            );

            // Fill in new information at the given index.
            if mention_set.is_use() {
                if let Some(prev_rreg) = mapper.lookup_use(*vreg) {
                    debug_assert_eq!(prev_rreg, *rreg, "different use allocs for {:?}", vreg);
                }
                mapper.set_use(*vreg, *rreg);
            }

            let included_in_clobbers = func.is_included_in_clobbers(func.get_insn(*iix));
            if mention_set.is_mod() {
                if let Some(prev_rreg) = mapper.lookup_use(*vreg) {
                    debug_assert_eq!(prev_rreg, *rreg, "different use allocs for {:?}", vreg);
                }
                if let Some(prev_rreg) = mapper.lookup_def(*vreg) {
                    debug_assert_eq!(prev_rreg, *rreg, "different def allocs for {:?}", vreg);
                }

                mapper.set_use(*vreg, *rreg);
                mapper.set_def(*vreg, *rreg);
                if included_in_clobbers {
                    clobbered_registers.insert(*rreg);
                }
            }

            if mention_set.is_def() {
                if let Some(prev_rreg) = mapper.lookup_def(*vreg) {
                    debug_assert_eq!(prev_rreg, *rreg, "different def allocs for {:?}", *vreg);
                }

                mapper.set_def(*vreg, *rreg);
                if included_in_clobbers {
                    clobbered_registers.insert(*rreg);
                }
            }

            cur_quad_ix += 1;
        }

        // At this point we've correctly filled the mapper; actually map the virtual registers to
        // the real ones in the Function.
        trace!("map_regs for {:?}", func_inst_ix);

        // If available, make sure to update the checker's state *before* actually mapping the
        // register; the checker must see the function with virtual registers, not real ones.
        if let Some(ref mut checker) = checker {
            let block_ix = insn_blocks[func_inst_ix.get() as usize];
            checker
                .handle_insn(reg_universe, func, block_ix, func_inst_ix, &mapper)
                .unwrap();
        }

        let mut inst = func.get_insn_mut(func_inst_ix);
        F::map_regs(&mut inst, &mapper);

        mapper.clear();
    }

    if let Some(checker) = checker {
        checker.run()?;
    }

    Ok(clobbered_registers)
}

fn compute_stackmaps(
    intervals: &[VirtualInterval],
    stackmap_request: Option<&StackmapRequestInfo>,
) -> Vec<Vec<SpillSlot>> {
    if let Some(request) = stackmap_request {
        let mut stackmaps = vec![Vec::new(); request.safepoint_insns.len()];
        for int in intervals {
            if !int.ref_typed {
                continue;
            }
            if let Some(slot) = int.location.spill() {
                for &(_sp_iix, sp_ix) in &int.safepoints {
                    stackmaps[sp_ix].push(slot);
                }
            }
        }
        stackmaps
    } else {
        vec![]
    }
}

/// Fills in the register assignments into instructions.
#[inline(never)]
fn apply_registers<F: Function>(
    func: &mut F,
    virtual_intervals: &Vec<VirtualInterval>,
    memory_moves: Vec<InstToInsertAndExtPoint>,
    reg_universe: &RealRegUniverse,
    num_spill_slots: u32,
    use_checker: bool,
    stackmap_request: Option<&StackmapRequestInfo>,
) -> Result<RegAllocResult<F>, RegAllocError> {
    trace!("apply_registers");

    let stackmaps = compute_stackmaps(virtual_intervals, stackmap_request.clone());

    let clobbered_registers = set_registers(
        func,
        virtual_intervals,
        reg_universe,
        use_checker,
        &memory_moves,
        stackmap_request,
        &stackmaps,
    )
    .map_err(|err| RegAllocError::RegChecker(err))?;

    let (final_insns, target_map, new_to_old_insn_map, new_safepoint_insns) =
        add_spills_reloads_and_moves(
            func,
            stackmap_request.map(|request| request.safepoint_insns.as_slice()),
            memory_moves,
        )
        .map_err(|e| RegAllocError::Other(e))?;

    // And now remove from the clobbered registers set, all those not available to the allocator.
    // But not removing the reserved regs, since we might have modified those.
    clobbered_registers.filter_map(|&reg| {
        if reg.get_index() >= reg_universe.allocable {
            None
        } else {
            Some(reg)
        }
    });

    Ok(RegAllocResult {
        insns: final_insns,
        target_map,
        orig_insn_map: new_to_old_insn_map,
        clobbered_registers,
        num_spill_slots,
        block_annotations: None,
        stackmaps,
        new_safepoint_insns,
    })
}
