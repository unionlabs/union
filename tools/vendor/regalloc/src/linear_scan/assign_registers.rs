use super::{
    analysis::BlockPos, last_use, next_use, IntId, Intervals, Mention, MentionMap,
    OptimalSplitStrategy, RegUses, Statistics, VirtualInterval,
};
use crate::{
    data_structures::{InstPoint, Point, RegVecsAndBounds},
    Function, InstIx, LinearScanOptions, RealReg, RealRegUniverse, Reg, RegAllocError, SpillSlot,
    VirtualReg, NUM_REG_CLASSES,
};

use log::{debug, log_enabled, trace, Level};
use rustc_hash::FxHashMap as HashMap;
use smallvec::SmallVec;
use std::collections::BinaryHeap;
use std::{cmp, cmp::Ordering, fmt};

#[derive(Clone, Copy, PartialEq)]
enum ActiveInt {
    Virtual(IntId),
    Fixed((RealReg, usize)),
}

impl fmt::Debug for ActiveInt {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActiveInt::Virtual(id) => write!(fmt, "virtual({:?})", id),
            ActiveInt::Fixed((rreg, _)) => write!(fmt, "real({:?})", rreg),
        }
    }
}

struct ActivityTracker {
    /// Intervals that are covering the current interval's start position.
    /// TODO Invariant: they always have a register attached to them.
    active: Vec<ActiveInt>,

    /// Intervals that are not covering but end after the current interval's start position.
    /// None means that the interval may have fragments, but they all live after the current
    /// position.
    /// TODO Invariant: they're all fixed registers, so they must have a register attached to them.
    inactive: Vec<(RealReg, usize)>,
}

impl ActivityTracker {
    fn new(intervals: &Intervals, scratches_by_rc: &[Option<RealReg>]) -> Self {
        let mut inactive = Vec::with_capacity(intervals.fixeds.len());
        for fixed in &intervals.fixeds {
            let rreg = fixed.reg;
            if !fixed.frags.is_empty() && scratches_by_rc[rreg.get_class() as usize] != Some(rreg) {
                inactive.push((rreg, 0))
            }
        }

        Self {
            active: Vec::new(),
            inactive,
        }
    }

    fn set_active(&mut self, id: IntId) {
        self.active.push(ActiveInt::Virtual(id));
    }

    fn update(&mut self, start: InstPoint, stats: &mut Option<Statistics>, intervals: &Intervals) {
        // From active, only possible transitions are to active or expired.
        // From inactive, only possible transitions are to inactive, active or expired.
        // => active has an upper bound.
        // => inactive only shrinks.
        let mut to_delete: SmallVec<[usize; 16]> = SmallVec::new();
        let mut new_inactive: SmallVec<[(RealReg, usize); 16]> = SmallVec::new();

        for (i, id) in self.active.iter_mut().enumerate() {
            match id {
                ActiveInt::Virtual(int_id) => {
                    let int = intervals.get(*int_id);

                    if int.location.spill().is_some() {
                        // TODO these shouldn't appear here...
                        to_delete.push(i);
                        continue;
                    }
                    //debug_assert!(int.location.spill().is_none(), "active int must have a reg");

                    if int.end < start {
                        // It's expired, forget about it.
                        to_delete.push(i);
                    } else {
                        // Stays active.
                        debug_assert!(int.covers(start), "no active to inactive transition");
                    }
                }

                ActiveInt::Fixed((rreg, ref mut fix)) => {
                    // Possible transitions: active => { active, inactive, expired }.
                    let frags = &intervals.fixeds[rreg.get_index()].frags;

                    // Fast-forward to the first fragment that contains or is after start.
                    while *fix < frags.len() && start > frags[*fix].last {
                        *fix += 1;
                    }

                    if *fix == frags.len() {
                        // It expired, remove it from the active list.
                        to_delete.push(i);
                    } else if start < frags[*fix].first {
                        // It is now inactive.
                        debug_assert!(!frags[*fix].contains(&start));
                        new_inactive.push((*rreg, *fix));
                        to_delete.push(i);
                    } else {
                        // Otherwise, it's still active.
                        debug_assert!(frags[*fix].contains(&start));
                    }
                }
            }
        }

        for &i in to_delete.iter().rev() {
            self.active.swap_remove(i);
        }
        to_delete.clear();

        for (i, (rreg, fix)) in self.inactive.iter_mut().enumerate() {
            // Possible transitions: inactive => { active, inactive, expired }.
            let frags = &intervals.fixeds[rreg.get_index()].frags;

            // Fast-forward to the first fragment that contains or is after start.
            while *fix < frags.len() && start > frags[*fix].last {
                *fix += 1;
            }

            if *fix == frags.len() {
                // It expired, remove it from the inactive list.
                to_delete.push(i);
            } else if start >= frags[*fix].first {
                // It is now active.
                debug_assert!(frags[*fix].contains(&start));
                self.active.push(ActiveInt::Fixed((*rreg, *fix)));
                to_delete.push(i);
            } else {
                // Otherwise it remains inactive.
                debug_assert!(!frags[*fix].contains(&start));
            }
        }

        for &i in to_delete.iter().rev() {
            self.inactive.swap_remove(i);
        }
        self.inactive.extend(new_inactive.into_vec());

        trace!("active:");
        for aid in &self.active {
            match aid {
                ActiveInt::Virtual(id) => {
                    trace!("  {}", intervals.get(*id));
                }
                ActiveInt::Fixed((real_reg, _frag)) => {
                    trace!("  {}", intervals.fixeds[real_reg.get_index()]);
                }
            }
        }
        trace!("inactive:");
        for &(rreg, fix) in &self.inactive {
            trace!(
                "  {:?} {:?}",
                rreg,
                intervals.fixeds[rreg.get_index()].frags[fix]
            );
        }
        trace!("end update state");

        stats.as_mut().map(|stats| {
            stats.peak_active = usize::max(stats.peak_active, self.active.len());
            stats.peak_inactive = usize::max(stats.peak_inactive, self.inactive.len());
        });
    }
}

pub(crate) fn run<F: Function>(
    opts: &LinearScanOptions,
    func: &F,
    reg_uses: &RegVecsAndBounds,
    reg_universe: &RealRegUniverse,
    scratches_by_rc: &[Option<RealReg>],
    intervals: Intervals,
    stats: Option<Statistics>,
) -> Result<(Intervals, u32), RegAllocError> {
    let mut state = State::new(opts, func, &reg_uses, scratches_by_rc, intervals, stats);
    let mut reusable = ReusableState::new(reg_universe, scratches_by_rc);

    #[cfg(debug_assertions)]
    let mut prev_start = InstPoint::min_value();

    while let Some(id) = state.next_unhandled() {
        debug!("main loop: allocating {}", state.intervals.get(id));

        #[cfg(debug_assertions)]
        {
            let int = state.intervals.get(id);
            debug_assert!(prev_start <= int.start, "main loop must make progress");
            prev_start = int.start;
        }

        if state.intervals.get(id).location.is_none() {
            let int = state.intervals.get(id);

            state
                .activity
                .update(int.start, &mut state.stats, &state.intervals);

            let ok = try_allocate_reg(&mut reusable, id, &mut state);
            if !ok {
                allocate_blocked_reg(&mut reusable, id, &mut state)?;
            }

            if state.intervals.get(id).location.reg().is_some() {
                if maybe_handle_safepoints(&mut state, id) {
                    // Even it if may have been spilled (because of a safepoint), the parent
                    // interval may still be active.
                    state.activity.set_active(id);
                }
            }

            // Reset reusable state.
            reusable.computed_inactive = false;
        }

        debug!("");
    }

    if log_enabled!(Level::Debug) {
        debug!("allocation results (in order):");
        for int in state.intervals.virtuals.iter() {
            debug!("{}", int);
        }
        debug!("");
    }

    Ok((state.intervals, state.next_spill_slot.get()))
}

/// A mapping from real reg to some T.
#[derive(Clone)]
struct RegisterMapping<T> {
    offset: usize,
    regs: Vec<(RealReg, T)>,
    scratch: Option<RealReg>,
    initial_value: T,
    reg_class_index: usize,
}

impl<T: Copy> RegisterMapping<T> {
    fn with_default(
        reg_class_index: usize,
        reg_universe: &RealRegUniverse,
        scratch: Option<RealReg>,
        initial_value: T,
    ) -> Self {
        let mut regs = Vec::new();
        let mut offset = 0;
        // Collect all the registers for the current class.
        if let Some(ref info) = reg_universe.allocable_by_class[reg_class_index] {
            debug_assert!(info.first <= info.last);
            offset = info.first;
            for reg in &reg_universe.regs[info.first..=info.last] {
                debug_assert!(regs.len() == reg.0.get_index() - offset);
                regs.push((reg.0, initial_value));
            }
        };
        Self {
            offset,
            regs,
            scratch,
            initial_value,
            reg_class_index,
        }
    }

    fn clear(&mut self) {
        for reg in self.regs.iter_mut() {
            reg.1 = self.initial_value;
        }
    }

    fn iter<'a>(&'a self) -> RegisterMappingIter<T> {
        RegisterMappingIter {
            iter: self.regs.iter(),
            scratch: self.scratch,
        }
    }
}

struct RegisterMappingIter<'a, T: Copy> {
    iter: std::slice::Iter<'a, (RealReg, T)>,
    scratch: Option<RealReg>,
}

impl<'a, T: Copy> std::iter::Iterator for RegisterMappingIter<'a, T> {
    type Item = &'a (RealReg, T);
    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(pair) => {
                if Some(pair.0) == self.scratch {
                    // Skip to the next one.
                    self.iter.next()
                } else {
                    Some(pair)
                }
            }
            None => None,
        }
    }
}

impl<T> std::ops::Index<RealReg> for RegisterMapping<T> {
    type Output = T;
    fn index(&self, rreg: RealReg) -> &Self::Output {
        debug_assert!(
            rreg.get_class() as usize == self.reg_class_index,
            "trying to index a reg from the wrong class"
        );
        debug_assert!(
            Some(rreg) != self.scratch,
            "trying to const-use the scratch of {:?}",
            rreg.get_class()
        );
        &self.regs[rreg.get_index() - self.offset].1
    }
}

impl<T> std::ops::IndexMut<RealReg> for RegisterMapping<T> {
    fn index_mut(&mut self, rreg: RealReg) -> &mut Self::Output {
        debug_assert!(
            rreg.get_class() as usize == self.reg_class_index,
            "trying to index a reg from the wrong class"
        );
        debug_assert!(
            Some(rreg) != self.scratch,
            "trying to mut-use the scratch of {:?}",
            rreg.get_class()
        );
        &mut self.regs[rreg.get_index() - self.offset].1
    }
}

// State management.

/// Parts of state just reused for recycling memory.
struct ReusableState {
    inactive_intersecting: Vec<(RealReg, InstPoint)>,
    computed_inactive: bool,
    reg_to_instpoint_1: Vec<RegisterMapping<InstPoint>>,
    reg_to_instpoint_2: Vec<RegisterMapping<InstPoint>>,
}

impl ReusableState {
    fn new(reg_universe: &RealRegUniverse, scratches: &[Option<RealReg>]) -> Self {
        let mut reg_to_instpoint_1 = Vec::with_capacity(NUM_REG_CLASSES);

        for i in 0..NUM_REG_CLASSES {
            let scratch = scratches[i];
            reg_to_instpoint_1.push(RegisterMapping::with_default(
                i,
                reg_universe,
                scratch,
                InstPoint::max_value(),
            ));
        }

        let reg_to_instpoint_2 = reg_to_instpoint_1.clone();

        Self {
            inactive_intersecting: Vec::new(),
            computed_inactive: false,
            reg_to_instpoint_1,
            reg_to_instpoint_2,
        }
    }
}

/// A small pair containing the interval id and the instruction point of an interval that is still
/// to be allocated, to be stored in the unhandled list of intervals.
struct IntervalStart(IntId, InstPoint);

impl cmp::PartialEq for IntervalStart {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl cmp::Eq for IntervalStart {}

impl cmp::PartialOrd for IntervalStart {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Note: we want a reverse ordering on start positions, so that we have a MinHeap and not a
        // MaxHeap in UnhandledIntervals.
        other.1.partial_cmp(&self.1)
    }
}

impl cmp::Ord for IntervalStart {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

struct UnhandledIntervals {
    heap: BinaryHeap<IntervalStart>,
}

impl UnhandledIntervals {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(16),
        }
    }

    /// Insert a virtual interval that's unallocated in the list of unhandled intervals.
    ///
    /// This relies on the fact that unhandled intervals's start positions can't change over time.
    fn insert(&mut self, id: IntId, intervals: &Intervals) {
        self.heap.push(IntervalStart(id, intervals.get(id).start))
    }

    /// Get the new unhandled interval, in start order.
    fn next_unhandled(&mut self, _intervals: &Intervals) -> Option<IntId> {
        self.heap.pop().map(|entry| {
            let ret = entry.0;
            debug_assert!(_intervals.get(ret).start == entry.1);
            ret
        })
    }
}

/// State structure, which can be cleared between different calls to register allocation.
/// TODO: split this into clearable fields and non-clearable fields.
struct State<'a, F: Function> {
    func: &'a F,
    reg_uses: &'a RegUses,
    opts: &'a LinearScanOptions,

    intervals: Intervals,

    /// Intervals that are starting after the current interval's start position.
    unhandled: UnhandledIntervals,

    /// Next available spill slot.
    next_spill_slot: SpillSlot,

    /// Maps given virtual registers to the spill slots they should be assigned
    /// to.
    spill_map: HashMap<VirtualReg, SpillSlot>,

    activity: ActivityTracker,
    stats: Option<Statistics>,
}

impl<'a, F: Function> State<'a, F> {
    fn new(
        opts: &'a LinearScanOptions,
        func: &'a F,
        reg_uses: &'a RegUses,
        scratches_by_rc: &[Option<RealReg>],
        intervals: Intervals,
        stats: Option<Statistics>,
    ) -> Self {
        let mut unhandled = UnhandledIntervals::new();
        for int in intervals.virtuals.iter() {
            unhandled.insert(int.id, &intervals);
        }

        let activity = ActivityTracker::new(&intervals, scratches_by_rc);

        Self {
            func,
            reg_uses,
            opts,
            intervals,
            unhandled,
            next_spill_slot: SpillSlot::new(0),
            spill_map: HashMap::default(),
            stats,
            activity,
        }
    }

    fn next_unhandled(&mut self) -> Option<IntId> {
        self.unhandled.next_unhandled(&self.intervals)
    }
    fn insert_unhandled(&mut self, id: IntId) {
        self.unhandled.insert(id, &self.intervals);
    }

    fn spill(&mut self, id: IntId) {
        let int = self.intervals.get(id);
        debug_assert!(int.location.spill().is_none(), "already spilled");
        debug!("spilling {:?}", id);

        let vreg = int.vreg;
        let spill_slot = if let Some(spill_slot) = self.spill_map.get(&vreg) {
            *spill_slot
        } else {
            let size_slot = self.func.get_spillslot_size(vreg.get_class(), vreg);
            let spill_slot = self.next_spill_slot.round_up(size_slot);
            self.next_spill_slot = spill_slot.inc(1);
            self.spill_map.insert(vreg, spill_slot);
            spill_slot
        };

        self.intervals.set_spill(id, spill_slot);
    }
}

#[inline(never)]
fn lazy_compute_inactive(
    intervals: &Intervals,
    activity: &ActivityTracker,
    cur_id: IntId,
    inactive_intersecting: &mut Vec<(RealReg, InstPoint)>,
    computed_inactive: &mut bool,
) {
    if *computed_inactive {
        return;
    }
    inactive_intersecting.clear();

    let int = intervals.get(cur_id);
    let reg_class = int.vreg.get_class();

    for &(rreg, fix) in &activity.inactive {
        if rreg.get_class() != reg_class {
            continue;
        }

        let frags = &intervals.fixeds[rreg.get_index()].frags;
        let mut i = fix;
        while let Some(ref frag) = frags.get(i) {
            if frag.first > int.end {
                break;
            }
            if frag.first >= int.start {
                inactive_intersecting.push((rreg, frag.first));
                break;
            }
            i += 1;
        }
    }

    *computed_inactive = true;
}

/// Transitions intervals from active/inactive into active/inactive/handled.
///
/// An interval tree is stored in the state, containing all the active and
/// inactive intervals. The comparison key is the interval's start point.
///
/// A state update consists in the following. We consider the next interval to
/// allocate, and in particular its start point S.
///
/// 1. remove all the active/inactive intervals that have expired, i.e. their
///    end point is before S.
/// 2. reconsider active/inactive intervals:
///   - if they contain S, they become (or stay) active.
///   - otherwise, they become (or stay) inactive.
///
/// Item 1 is easy to implement, and fast enough.
///
/// Item 2 is a bit trickier. While we could just call `Intervals::covers` for
/// each interval on S, this is quite expensive. In addition to this, it happens
/// that most intervals are inactive. This is explained by the fact that linear
/// scan can create large intervals, if a value is used much later after it's
/// been created, *according to the block ordering*.
///
/// For each interval, we remember the last active fragment, or the first
/// inactive fragment that starts after S. This makes search really fast:
///
/// - if the considered (active or inactive) interval start is before S, then we
/// should look more precisely if it's active or inactive. This might include
/// seeking to the next fragment that contains S.
/// - otherwise, if the considered interval start is *after* S, then it means
/// this interval, as well as all the remaining ones in the interval tree (since
/// they're sorted by starting position) are inactive, and we can escape the
/// loop eagerly.
///
/// The escape for inactive intervals make this function overall cheap.

/// Naive heuristic to select a register when we're not aware of any conflict.
/// Currently, it chooses the register with the furthest next use.
#[inline(never)]
fn select_naive_reg<F: Function>(
    reusable: &mut ReusableState,
    state: &mut State<F>,
    id: IntId,
) -> Option<(RealReg, InstPoint)> {
    let reg_class = state.intervals.get(id).vreg.get_class();
    let free_until_pos = &mut reusable.reg_to_instpoint_1[reg_class as usize];
    free_until_pos.clear();

    let mut num_free = usize::max(1, free_until_pos.regs.len()) - 1;

    // All registers currently in use are blocked.
    for &aid in &state.activity.active {
        let reg = match aid {
            ActiveInt::Virtual(int_id) => {
                if let Some(reg) = state.intervals.get(int_id).location.reg() {
                    reg
                } else {
                    continue;
                }
            }
            ActiveInt::Fixed((real_reg, _)) => real_reg,
        };

        if reg.get_class() == reg_class {
            free_until_pos[reg] = InstPoint::min_value();
            num_free -= 1;
        }
    }

    // Shortcut: if all the registers are taken, don't even bother.
    if num_free == 0 {
        debug_assert!(!free_until_pos
            .iter()
            .any(|pair| pair.1 != InstPoint::min_value()));
        return None;
    }

    // All registers that would be used at the same time as the current interval
    // are partially blocked, up to the point when they start being used.
    lazy_compute_inactive(
        &state.intervals,
        &state.activity,
        id,
        &mut reusable.inactive_intersecting,
        &mut reusable.computed_inactive,
    );

    for &(reg, intersect_at) in reusable.inactive_intersecting.iter() {
        if intersect_at < free_until_pos[reg] {
            free_until_pos[reg] = intersect_at;
        }
    }

    // Find the register with the furthest next use, if there's any.
    let mut best_reg = None;
    let mut best_pos = InstPoint::min_value();
    for &(reg, pos) in free_until_pos.iter() {
        if pos > best_pos {
            best_pos = pos;
            best_reg = Some(reg);
        }
    }

    best_reg.and_then(|reg| Some((reg, best_pos)))
}

#[inline(never)]
fn try_allocate_reg<F: Function>(
    reusable: &mut ReusableState,
    id: IntId,
    state: &mut State<F>,
) -> bool {
    state
        .stats
        .as_mut()
        .map(|stats| stats.num_try_allocate_reg += 1);

    let (best_reg, best_pos) = if let Some(solution) = select_naive_reg(reusable, state, id) {
        solution
    } else {
        debug!("try_allocate_reg: all registers taken, need to spill.");
        return false;
    };
    debug!(
        "try_allocate_reg: best register {:?} has next use at {:?}",
        best_reg, best_pos
    );

    if best_pos <= state.intervals.get(id).end {
        if !state.opts.partial_split || !try_split_regs(state, id, best_pos) {
            return false;
        }
    }

    // At least a partial match: allocate.
    debug!(
        "{:?}: {:?} <- {:?}",
        id,
        state.intervals.get(id).vreg,
        best_reg
    );
    state.intervals.set_reg(id, best_reg);

    state
        .stats
        .as_mut()
        .map(|stats| stats.num_try_allocate_reg_success += 1);

    true
}

#[inline(never)]
fn allocate_blocked_reg<F: Function>(
    reusable: &mut ReusableState,
    cur_id: IntId,
    state: &mut State<F>,
) -> Result<(), RegAllocError> {
    // If the current interval has no uses, spill it directly.
    let first_use = match next_use(
        &state.intervals.get(cur_id),
        InstPoint::min_value(),
        &state.reg_uses,
    ) {
        Some(u) => u,
        None => {
            state.spill(cur_id);
            return Ok(());
        }
    };

    let (start_pos, reg_class) = {
        let int = state.intervals.get(cur_id);
        (int.start, int.vreg.get_class())
    };

    // Note: in this function, "use" isn't just a use as in use-def; it really
    // means a mention, so either a use or a definition.
    //
    // 1. Compute all the positions of next uses for registers of active intervals
    // and inactive intervals that might intersect with the current one.
    // 2. Then use this to select the interval with the further next use.
    // 3. Spill either the current interval or active/inactive intervals with the
    //    selected register.
    // 4. Make sure that the current interval doesn't intersect with the fixed
    //    interval for the selected register.

    // Step 1: compute all the next use positions.
    let next_use_pos = &mut reusable.reg_to_instpoint_1[reg_class as usize];
    next_use_pos.clear();

    let block_pos = &mut reusable.reg_to_instpoint_2[reg_class as usize];
    block_pos.clear();

    trace!(
        "allocate_blocked_reg: searching reg with next use after {:?}",
        start_pos
    );

    for &aid in &state.activity.active {
        match aid {
            ActiveInt::Virtual(int_id) => {
                let int = state.intervals.get(int_id);
                if int.vreg.get_class() != reg_class {
                    continue;
                }
                if let Some(reg) = int.location.reg() {
                    if next_use_pos[reg] != InstPoint::min_value() {
                        if let Some(next_use) =
                            next_use(&state.intervals.get(int_id), start_pos, &state.reg_uses)
                        {
                            next_use_pos[reg] = InstPoint::min(next_use_pos[reg], next_use);
                        }
                    }
                }
            }

            ActiveInt::Fixed((reg, _frag)) => {
                if reg.get_class() == reg_class {
                    block_pos[reg] = InstPoint::min_value();
                    next_use_pos[reg] = InstPoint::min_value();
                }
            }
        }
    }

    lazy_compute_inactive(
        &state.intervals,
        &state.activity,
        cur_id,
        &mut reusable.inactive_intersecting,
        &mut reusable.computed_inactive,
    );

    for &(reg, intersect_pos) in &reusable.inactive_intersecting {
        debug_assert!(reg.get_class() == reg_class);
        if block_pos[reg] == InstPoint::min_value() {
            // This register is already blocked.
            debug_assert!(next_use_pos[reg] == InstPoint::min_value());
            continue;
        }
        block_pos[reg] = InstPoint::min(block_pos[reg], intersect_pos);
        next_use_pos[reg] = InstPoint::min(next_use_pos[reg], intersect_pos);
    }

    // Step 2: find the register with the furthest next use.
    let best_reg = {
        let mut best = None;
        for (reg, pos) in next_use_pos.iter() {
            trace!("allocate_blocked_reg: {:?} has next use at {:?}", reg, pos);
            match best {
                None => best = Some((reg, pos)),
                Some((ref mut best_reg, ref mut best_pos)) => {
                    if *best_pos < pos {
                        *best_pos = pos;
                        *best_reg = reg;
                    }
                }
            }
        }
        match best {
            Some(best) => *best.0,
            None => {
                return Err(RegAllocError::Other(format!(
                    "the {:?} register class has no registers!",
                    reg_class
                )));
            }
        }
    };
    debug!(
        "selecting blocked register {:?} with furthest next use at {:?}",
        best_reg, next_use_pos[best_reg]
    );

    // Step 3: if the next use of the current interval is after the furthest use
    // of the selected register, then we should spill the current interval.
    // Otherwise, spill other intervals.
    debug!(
        "current first used at {:?}, next use of best reg at {:?}",
        first_use, next_use_pos[best_reg]
    );

    if first_use >= next_use_pos[best_reg] {
        if first_use == start_pos {
            return Err(RegAllocError::OutOfRegisters(reg_class));
        }
        debug!("spill current interval");
        let new_int = split(state, cur_id, first_use);
        state.insert_unhandled(new_int);
        state.spill(cur_id);
    } else {
        debug!("taking over register, spilling intersecting intervals");

        // Spill intervals that currently block the selected register.
        state.intervals.set_reg(cur_id, best_reg);

        // If there's an interference with a fixed interval, split at the
        // intersection.
        let int_end = state.intervals.get(cur_id).end;
        if block_pos[best_reg] <= int_end {
            debug!(
                "allocate_blocked_reg: fixed conflict! blocked at {:?}, while ending at {:?}",
                block_pos[best_reg], int_end
            );

            if !state.opts.partial_split || !try_split_regs(state, cur_id, block_pos[best_reg]) {
                split_and_spill(state, cur_id, block_pos[best_reg]);
            }
        }

        for &aid in &state.activity.active {
            match aid {
                ActiveInt::Virtual(int_id) => {
                    let int = state.intervals.get(int_id);
                    if int.vreg.get_class() != reg_class {
                        continue;
                    }
                    if let Some(reg) = int.location.reg() {
                        if reg == best_reg {
                            // spill it!
                            debug!("allocate_blocked_reg: split and spill active stolen reg");
                            split_and_spill(state, int_id, start_pos);
                            break;
                        }
                    }
                }

                ActiveInt::Fixed((_reg, _fix)) => {
                    debug_assert!(
                        _reg != best_reg
                            || state.intervals.get(cur_id).end
                                < state.intervals.fixeds[_reg.get_index()].frags[_fix].first,
                        "can't split fixed active interval"
                    );
                }
            }
        }

        // Inactive virtual intervals would need to be split and spilled here too, but we can't
        // have inactive virtual intervals.
        #[cfg(debug_assertions)]
        for &(reg, intersect_pos) in &reusable.inactive_intersecting {
            debug_assert!(
                reg != best_reg || state.intervals.get(cur_id).end < intersect_pos,
                "can't split fixed inactive interval"
            );
        }
    }

    Ok(())
}

/// Handle safepoints by causing a spill/reload sequence across safepoint instructions, if needed.
///
/// Returns true if the original interval still lives in a register, after this function has been
/// called.
fn maybe_handle_safepoints<F: Function>(state: &mut State<F>, id: IntId) -> bool {
    let int = state.intervals.get(id);
    if !int.ref_typed {
        return true;
    }

    let next_safepoint = int.safepoints().first().clone();
    let sp_iix = match next_safepoint {
        Some(&(sp_iix, _)) => sp_iix,
        None => return true,
    };

    debug_assert!(int.start.iix() <= sp_iix && sp_iix <= int.end.iix());

    let sp_def = InstPoint::new_def(sp_iix);
    if let Some(prev_use) = last_use(&state.intervals.get(id), sp_def, &state.reg_uses) {
        if prev_use == sp_def {
            // The value is being redefined by the instruction; there's no need to keep it alive on the
            // stack, and we can use the same register assignment before and after the safepoint.
            return true;
        }

        // The value is not redefined by the instruction, split and spill before the safepoint.
        split_and_spill(state, id, InstPoint::new_use(sp_iix));
        return true;
    }

    // No use before the safepoint! Split the interval so a new child is put on the allocation
    // queue at its next use, and spill the interval.
    if let Some(nu) = next_use(&state.intervals.get(id), sp_def, &state.reg_uses) {
        let child = split(state, id, nu);
        state.insert_unhandled(child);
    }

    // Not in a register anymore.
    state.spill(id);
    return false;
}

/// Finds an optimal split position, whenever we're given a range of possible
/// positions where to split.
fn find_optimal_split_pos<F: Function>(
    state: &State<F>,
    id: IntId,
    from: InstPoint,
    to: InstPoint,
) -> InstPoint {
    trace!("find_optimal_split_pos between {:?} and {:?}", from, to);

    debug_assert!(from <= to, "split between positions are inconsistent");
    let int = state.intervals.get(id);
    debug_assert!(from >= int.start, "split should happen after the start");
    debug_assert!(to <= int.end, "split should happen before the end");

    if from == to {
        return from;
    }

    let candidate = match state.opts.split_strategy {
        OptimalSplitStrategy::To => Some(to),
        OptimalSplitStrategy::NextFrom => Some(next_pos(from)),
        OptimalSplitStrategy::NextNextFrom => Some(next_pos(next_pos(from))),
        OptimalSplitStrategy::From => {
            // This is the general setting, so win some time and eagerly return here.
            return from;
        }
        OptimalSplitStrategy::PrevTo => Some(prev_pos(to)),
        OptimalSplitStrategy::PrevPrevTo => Some(prev_pos(prev_pos(to))),
        OptimalSplitStrategy::Mid => Some(InstPoint::new_use(InstIx::new(
            (from.iix().get() + to.iix().get()) / 2,
        ))),
    };

    if let Some(pos) = candidate {
        if pos >= from && pos <= to && state.intervals.get(id).covers(pos) {
            return pos;
        }
    }

    from
}

fn prev_pos(mut pos: InstPoint) -> InstPoint {
    match pos.pt() {
        Point::Def => {
            pos.set_pt(Point::Use);
            pos
        }
        Point::Use => {
            pos.set_iix(pos.iix().minus(1));
            pos.set_pt(Point::Def);
            pos
        }
        _ => unreachable!(),
    }
}

fn next_pos(mut pos: InstPoint) -> InstPoint {
    match pos.pt() {
        Point::Use => pos.set_pt(Point::Def),
        Point::Def => {
            pos.set_pt(Point::Use);
            pos.set_iix(pos.iix().plus(1));
        }
        _ => unreachable!(),
    };
    pos
}

/// Splits the given interval between the last use before `split_pos` and
/// `split_pos`.
///
/// In case of two-ways split (i.e. only place to split is precisely split_pos),
/// returns the live interval id for the middle child, to be added back to the
/// list of active/inactive intervals after iterating on these.
fn split_and_spill<F: Function>(state: &mut State<F>, id: IntId, split_pos: InstPoint) {
    let child = match last_use(&state.intervals.get(id), split_pos, &state.reg_uses) {
        Some(last_use) => {
            debug!(
                "split_and_spill {:?}: spill between {:?} and {:?}",
                id, last_use, split_pos
            );

            // Maintain ascending order between the min and max positions.
            let min_pos = InstPoint::min(next_pos(last_use), split_pos);

            // Make sure that if the two positions are the same, we'll be splitting in
            // a position that's in the current interval.
            let optimal_pos = find_optimal_split_pos(state, id, min_pos, split_pos);

            let child = split(state, id, optimal_pos);
            state.spill(child);
            child
        }

        None => {
            // The current interval has no uses before the split position, it can
            // safely be spilled.
            debug!(
                "split_and_spill {:?}: spilling it since no uses before split position",
                id
            );
            state.spill(id);
            id
        }
    };

    // Split until the next register use.
    match next_use(&state.intervals.get(child), split_pos, &state.reg_uses) {
        Some(next_use_pos) => {
            debug!(
                "split spilled interval before next use @ {:?}",
                next_use_pos
            );
            let child = split(state, child, next_use_pos);
            state.insert_unhandled(child);
        }
        None => {
            // Let it be spilled for the rest of its lifetime.
        }
    }

    // In both cases, the spilled child interval can remain on the stack.
    debug!("spilled split child {:?} silently expires", child);
}

/// Try to find a (use) position where to split the interval until the next point at which it
/// becomes unavailable, and put it back into the queue of intervals to allocate later on.  Returns
/// true if it succeeded in finding such a position, false otherwise.
fn try_split_regs<F: Function>(
    state: &mut State<F>,
    id: IntId,
    available_until: InstPoint,
) -> bool {
    state.stats.as_mut().map(|stats| stats.num_reg_splits += 1);

    // Find a position for the split: we'll iterate backwards from the point until the register is
    // available, down to the previous use of the current interval.
    let prev_use = match last_use(&state.intervals.get(id), available_until, &state.reg_uses) {
        Some(prev_use) => prev_use,
        None => state.intervals.get(id).start,
    };

    let split_pos = if state.opts.partial_split_near_end {
        // Split at the position closest to the available_until position.
        let pos = match available_until.pt() {
            Point::Use => prev_pos(prev_pos(available_until)),
            Point::Def => prev_pos(available_until),
            _ => unreachable!(),
        };
        if pos <= prev_use {
            return false;
        }
        pos
    } else {
        // Split at the position closest to the prev_use position. If it was a def, we can split
        // just thereafter, if it was at a use, go to the next use.
        let pos = match prev_use.pt() {
            Point::Use => next_pos(next_pos(prev_use)),
            Point::Def => next_pos(prev_use),
            _ => unreachable!(),
        };
        if pos >= available_until {
            return false;
        }
        pos
    };

    let child = split(state, id, split_pos);
    state.insert_unhandled(child);

    state
        .stats
        .as_mut()
        .map(|stats| stats.num_reg_splits_success += 1);

    true
}

/// Splits the interval at the given position.
///
/// The split position must either be a Def of the current vreg, or it must be
/// at a Use position (otherwise there's no place to put the moves created by
/// the split).
///
/// The id of the new interval is returned, while the parent interval is mutated
/// in place. The child interval starts after (including) at_pos.
#[inline(never)]
fn split<F: Function>(state: &mut State<F>, id: IntId, at_pos: InstPoint) -> IntId {
    debug!("split {:?} at {:?}", id, at_pos);
    trace!("interval: {}", state.intervals.get(id));

    let int = state.intervals.get(id);
    debug_assert!(int.start <= at_pos, "must split after the start");
    debug_assert!(at_pos <= int.end, "must split before the end");

    // We're splitting in the middle of a fragment: [L, R].
    // Split it into two fragments: parent [L, pos[ + child [pos, R].
    debug_assert!(int.start < int.end, "trying to split unit fragment");
    debug_assert!(int.start <= at_pos, "no space to split fragment");

    let parent_start = int.start;
    let parent_end = prev_pos(at_pos);
    let child_start = at_pos;
    let child_end = int.end;

    trace!(
        "split fragment [{:?}; {:?}] into two parts: [{:?}; {:?}] to [{:?}; {:?}]",
        int.start,
        int.end,
        parent_start,
        parent_end,
        child_start,
        child_end
    );

    debug_assert!(parent_start <= parent_end);
    debug_assert!(parent_end <= child_start);
    debug_assert!(child_start <= child_end);

    let vreg = int.vreg;
    let ancestor = int.ancestor;
    let ref_typed = int.ref_typed;

    // Split the register mentions.
    let parent_mentions = state.intervals.get_mut(id).mentions_mut();
    let index = parent_mentions.binary_search_by(|mention| {
        // The comparator function returns the position of the argument compared to the target.

        // Search by index first.
        let iix = mention.0;
        if iix < at_pos.iix() {
            return Ordering::Less;
        }
        if iix > at_pos.iix() {
            return Ordering::Greater;
        }

        // The instruction index is the same. Consider the instruction side now, and compare it
        // with the set. For the purpose of LSRA, mod means use and def.
        let set = mention.1;
        if at_pos.pt() == Point::Use {
            if set.is_use_or_mod() {
                Ordering::Equal
            } else {
                // It has to be Mod or Def. We need to look more to the right of the seeked array.
                // Thus indicate this mention is after the target.
                Ordering::Greater
            }
        } else {
            debug_assert!(at_pos.pt() == Point::Def);
            if set.is_mod_or_def() {
                Ordering::Equal
            } else {
                // Look to the left.
                Ordering::Less
            }
        }
    });

    let index = match index {
        Ok(index) => index,
        Err(index) => index,
    };
    let mut child_mentions = MentionMap::with_capacity(parent_mentions.len() - index);
    for mention in parent_mentions.iter().skip(index) {
        child_mentions.push(mention.clone());
    }
    parent_mentions.truncate(index);

    // Now split block boundaries.
    let parent_boundaries = state.intervals.get(id).block_boundaries();
    let index = parent_boundaries.binary_search_by(|boundary| {
        let inst_range = state.func.block_insns(boundary.bix);
        match boundary.pos {
            BlockPos::Start => {
                let first_inst = InstPoint::new_use(inst_range.first());
                return first_inst.cmp(&at_pos);
            }
            BlockPos::End => {
                let last_inst = InstPoint::new_def(inst_range.last());
                return last_inst.cmp(&at_pos);
            }
        }
    });

    // It's possible that the binary search returns Err, for the edges (if the split position is
    // before the first block boundary, or after the last).
    let index = match index {
        Ok(index) => index,
        Err(index) => index,
    };
    let child_boundaries = state
        .intervals
        .get_mut(id)
        .block_boundaries_mut()
        .split_off(index);

    // Now split the safepoints.
    let child_safepoints = {
        // Remove from the parent all the safepoints that aren't included in the new bounds.
        let mut i = 0;
        let parent_safepoints = state.intervals.get_mut(id).safepoints_mut();
        while let Some(&(sp_iix, _sp_ix)) = parent_safepoints.get(i) {
            if InstPoint::new_use(sp_iix) >= child_start {
                break;
            }
            i += 1;
        }
        let child = parent_safepoints.iter().skip(i).cloned().collect();
        parent_safepoints.truncate(i);
        child
    };

    trace!("child interval has safepoints {:?}", child_safepoints);

    // Phew: eventually create the child interval.
    let child_id = IntId(state.intervals.num_virtual_intervals());
    let mut child_int = VirtualInterval::new(
        child_id,
        vreg,
        child_start,
        child_end,
        child_mentions,
        child_boundaries,
        ref_typed,
        child_safepoints,
    );
    child_int.parent = Some(id);
    child_int.ancestor = ancestor;

    state.intervals.push_interval(child_int);

    state.intervals.get_mut(id).end = parent_end;
    state.intervals.set_child(id, child_id);

    if log_enabled!(Level::Trace) {
        trace!("split results:");
        trace!("- {}", state.intervals.get(id));
        trace!("- {}", state.intervals.get(child_id));
    }

    child_id
}

fn _build_mention_map(reg_uses: &RegUses) -> HashMap<Reg, MentionMap> {
    // Maps reg to its mentions.
    let mut reg_mentions: HashMap<Reg, MentionMap> = HashMap::default();

    // Collect all the mentions.
    for i in 0..reg_uses.num_insns() {
        let iix = InstIx::new(i as u32);
        let regsets = reg_uses.get_reg_sets_for_iix(iix);
        debug_assert!(regsets.is_sanitized());

        for reg in regsets.uses.iter() {
            let mentions = reg_mentions.entry(*reg).or_default();
            if mentions.is_empty() || mentions.last().unwrap().0 != iix {
                mentions.push((iix, Mention::new()));
            }
            mentions.last_mut().unwrap().1.add_use();
        }

        for reg in regsets.mods.iter() {
            let mentions = reg_mentions.entry(*reg).or_default();
            if mentions.is_empty() || mentions.last().unwrap().0 != iix {
                mentions.push((iix, Mention::new()));
            }
            mentions.last_mut().unwrap().1.add_mod();
        }

        for reg in regsets.defs.iter() {
            let mentions = reg_mentions.entry(*reg).or_default();
            if mentions.is_empty() || mentions.last().unwrap().0 != iix {
                mentions.push((iix, Mention::new()));
            }
            mentions.last_mut().unwrap().1.add_def();
        }
    }

    reg_mentions
}
