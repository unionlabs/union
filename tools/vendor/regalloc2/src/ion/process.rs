/*
 * This file was initially derived from the files
 * `js/src/jit/BacktrackingAllocator.h` and
 * `js/src/jit/BacktrackingAllocator.cpp` in Mozilla Firefox, and was
 * originally licensed under the Mozilla Public License 2.0. We
 * subsequently relicensed it to Apache-2.0 WITH LLVM-exception (see
 * https://github.com/bytecodealliance/regalloc2/issues/7).
 *
 * Since the initial port, the design has been substantially evolved
 * and optimized.
 */

//! Main allocation loop that processes bundles.

use super::{
    spill_weight_from_constraint, Env, LiveBundleIndex, LiveBundleVec, LiveRangeFlag,
    LiveRangeIndex, LiveRangeKey, LiveRangeList, LiveRangeListEntry, PRegIndex, RegTraversalIter,
    Requirement, SpillWeight, UseList, VRegIndex,
};
use crate::{
    ion::data_structures::{
        CodeRange, BUNDLE_MAX_NORMAL_SPILL_WEIGHT, MAX_SPLITS_PER_SPILLSET,
        MINIMAL_BUNDLE_SPILL_WEIGHT, MINIMAL_FIXED_BUNDLE_SPILL_WEIGHT,
    },
    Allocation, Function, Inst, InstPosition, OperandConstraint, OperandKind, PReg, ProgPoint,
    RegAllocError,
};
use fxhash::FxHashSet;
use smallvec::{smallvec, SmallVec};
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AllocRegResult {
    Allocated(Allocation),
    Conflict(LiveBundleVec, ProgPoint),
    ConflictWithFixed(u32, ProgPoint),
    ConflictHighCost,
}

impl<'a, F: Function> Env<'a, F> {
    pub fn process_bundles(&mut self) -> Result<(), RegAllocError> {
        while let Some((bundle, reg_hint)) = self.allocation_queue.pop() {
            self.stats.process_bundle_count += 1;
            self.process_bundle(bundle, reg_hint)?;
        }
        self.stats.final_liverange_count = self.ranges.len();
        self.stats.final_bundle_count = self.bundles.len();
        self.stats.spill_bundle_count = self.spilled_bundles.len();

        Ok(())
    }

    pub fn try_to_allocate_bundle_to_reg(
        &mut self,
        bundle: LiveBundleIndex,
        reg: PRegIndex,
        // if the max bundle weight in the conflict set exceeds this
        // cost (if provided), just return
        // `AllocRegResult::ConflictHighCost`.
        max_allowable_cost: Option<u32>,
    ) -> AllocRegResult {
        trace!("try_to_allocate_bundle_to_reg: {:?} -> {:?}", bundle, reg);
        let mut conflicts = smallvec![];
        self.conflict_set.clear();
        let mut max_conflict_weight = 0;
        // Traverse the BTreeMap in order by requesting the whole
        // range spanned by the bundle and iterating over that
        // concurrently with our ranges. Because our ranges are in
        // order, and the BTreeMap is as well, this allows us to have
        // an overall O(n log n) + O(b) complexity, where the PReg has
        // n current ranges and the bundle has b ranges, rather than
        // O(b * n log n) with the simple probe-for-each-bundle-range
        // approach.
        //
        // Note that the comparator function on a CodeRange tests for
        // *overlap*, so we are checking whether the BTree contains
        // any preg range that *overlaps* with range `range`, not
        // literally the range `range`.
        let bundle_ranges = &self.bundles[bundle.index()].ranges;
        let from_key = LiveRangeKey::from_range(&CodeRange {
            from: bundle_ranges.first().unwrap().range.from,
            to: bundle_ranges.first().unwrap().range.from,
        });
        let mut preg_range_iter = self.pregs[reg.index()]
            .allocations
            .btree
            .range(from_key..)
            .peekable();
        trace!(
            "alloc map for {:?} in range {:?}..: {:?}",
            reg,
            from_key,
            self.pregs[reg.index()].allocations.btree
        );
        let mut first_conflict: Option<ProgPoint> = None;

        'ranges: for entry in bundle_ranges {
            trace!(" -> range LR {:?}: {:?}", entry.index, entry.range);
            let key = LiveRangeKey::from_range(&entry.range);

            let mut skips = 0;
            'alloc: loop {
                trace!("  -> PReg range {:?}", preg_range_iter.peek());

                // Advance our BTree traversal until it is >= this bundle
                // range (i.e., skip PReg allocations in the BTree that
                // are completely before this bundle range).

                if preg_range_iter.peek().is_some() && *preg_range_iter.peek().unwrap().0 < key {
                    trace!(
                        "Skipping PReg range {:?}",
                        preg_range_iter.peek().unwrap().0
                    );
                    preg_range_iter.next();
                    skips += 1;
                    if skips >= 16 {
                        let from_pos = entry.range.from;
                        let from_key = LiveRangeKey::from_range(&CodeRange {
                            from: from_pos,
                            to: from_pos,
                        });
                        preg_range_iter = self.pregs[reg.index()]
                            .allocations
                            .btree
                            .range(from_key..)
                            .peekable();
                        skips = 0;
                    }
                    continue 'alloc;
                }
                skips = 0;

                // If there are no more PReg allocations, we're done!
                if preg_range_iter.peek().is_none() {
                    trace!(" -> no more PReg allocations; so no conflict possible!");
                    break 'ranges;
                }

                // If the current PReg range is beyond this range, there is no conflict; continue.
                if *preg_range_iter.peek().unwrap().0 > key {
                    trace!(
                        " -> next PReg allocation is at {:?}; moving to next VReg range",
                        preg_range_iter.peek().unwrap().0
                    );
                    break 'alloc;
                }

                // Otherwise, there is a conflict.
                let preg_key = *preg_range_iter.peek().unwrap().0;
                debug_assert_eq!(preg_key, key); // Assert that this range overlaps.
                let preg_range = preg_range_iter.next().unwrap().1;

                trace!(" -> btree contains range {:?} that overlaps", preg_range);
                if preg_range.is_valid() {
                    trace!("   -> from vreg {:?}", self.ranges[preg_range.index()].vreg);
                    // range from an allocated bundle: find the bundle and add to
                    // conflicts list.
                    let conflict_bundle = self.ranges[preg_range.index()].bundle;
                    trace!("   -> conflict bundle {:?}", conflict_bundle);
                    if self.conflict_set.insert(conflict_bundle) {
                        conflicts.push(conflict_bundle);
                        max_conflict_weight = std::cmp::max(
                            max_conflict_weight,
                            self.bundles[conflict_bundle.index()].cached_spill_weight(),
                        );
                        if max_allowable_cost.is_some()
                            && max_conflict_weight > max_allowable_cost.unwrap()
                        {
                            trace!("   -> reached high cost, retrying early");
                            return AllocRegResult::ConflictHighCost;
                        }
                    }

                    if first_conflict.is_none() {
                        first_conflict = Some(ProgPoint::from_index(std::cmp::max(
                            preg_key.from,
                            key.from,
                        )));
                    }
                } else {
                    trace!("   -> conflict with fixed reservation");
                    // range from a direct use of the PReg (due to clobber).
                    return AllocRegResult::ConflictWithFixed(
                        max_conflict_weight,
                        ProgPoint::from_index(preg_key.from),
                    );
                }
            }
        }

        if conflicts.len() > 0 {
            return AllocRegResult::Conflict(conflicts, first_conflict.unwrap());
        }

        // We can allocate! Add our ranges to the preg's BTree.
        let preg = PReg::from_index(reg.index());
        trace!("  -> bundle {:?} assigned to preg {:?}", bundle, preg);
        self.bundles[bundle.index()].allocation = Allocation::reg(preg);
        for entry in &self.bundles[bundle.index()].ranges {
            self.pregs[reg.index()]
                .allocations
                .btree
                .insert(LiveRangeKey::from_range(&entry.range), entry.index);
        }

        AllocRegResult::Allocated(Allocation::reg(preg))
    }

    pub fn evict_bundle(&mut self, bundle: LiveBundleIndex) {
        trace!(
            "evicting bundle {:?}: alloc {:?}",
            bundle,
            self.bundles[bundle.index()].allocation
        );
        let preg = match self.bundles[bundle.index()].allocation.as_reg() {
            Some(preg) => preg,
            None => {
                trace!(
                    "  -> has no allocation! {:?}",
                    self.bundles[bundle.index()].allocation
                );
                return;
            }
        };
        let preg_idx = PRegIndex::new(preg.index());
        self.bundles[bundle.index()].allocation = Allocation::none();
        for entry in &self.bundles[bundle.index()].ranges {
            trace!(" -> removing LR {:?} from reg {:?}", entry.index, preg_idx);
            self.pregs[preg_idx.index()]
                .allocations
                .btree
                .remove(&LiveRangeKey::from_range(&entry.range));
        }
        let prio = self.bundles[bundle.index()].prio;
        trace!(" -> prio {}; back into queue", prio);
        self.allocation_queue
            .insert(bundle, prio as usize, PReg::invalid());
    }

    pub fn bundle_spill_weight(&self, bundle: LiveBundleIndex) -> u32 {
        self.bundles[bundle.index()].cached_spill_weight()
    }

    pub fn maximum_spill_weight_in_bundle_set(&self, bundles: &LiveBundleVec) -> u32 {
        trace!("maximum_spill_weight_in_bundle_set: {:?}", bundles);
        let m = bundles
            .iter()
            .map(|&b| {
                let w = self.bundles[b.index()].cached_spill_weight();
                trace!("bundle{}: {}", b.index(), w);
                w
            })
            .max()
            .unwrap_or(0);
        trace!(" -> max: {}", m);
        m
    }

    pub fn recompute_bundle_properties(&mut self, bundle: LiveBundleIndex) {
        trace!("recompute bundle properties: bundle {:?}", bundle);

        let minimal;
        let mut fixed = false;
        let mut fixed_def = false;
        let mut stack = false;
        let bundledata = &self.bundles[bundle.index()];
        let first_range = bundledata.ranges[0].index;
        let first_range_data = &self.ranges[first_range.index()];

        self.bundles[bundle.index()].prio = self.compute_bundle_prio(bundle);

        if first_range_data.vreg.is_invalid() {
            trace!("  -> no vreg; minimal and fixed");
            minimal = true;
            fixed = true;
        } else {
            for u in &first_range_data.uses {
                trace!("  -> use: {:?}", u);
                if let OperandConstraint::FixedReg(_) = u.operand.constraint() {
                    trace!("  -> fixed operand at {:?}: {:?}", u.pos, u.operand);
                    fixed = true;
                    if u.operand.kind() == OperandKind::Def {
                        trace!("  -> is fixed def");
                        fixed_def = true;
                    }
                }
                if let OperandConstraint::Stack = u.operand.constraint() {
                    trace!("  -> stack operand at {:?}: {:?}", u.pos, u.operand);
                    stack = true;
                }
                if stack && fixed {
                    break;
                }
            }
            // Minimal if the range covers only one instruction. Note
            // that it could cover just one ProgPoint,
            // i.e. X.Before..X.After, or two ProgPoints,
            // i.e. X.Before..X+1.Before.
            trace!("  -> first range has range {:?}", first_range_data.range);
            let bundle_start = self.bundles[bundle.index()]
                .ranges
                .first()
                .unwrap()
                .range
                .from;
            let bundle_end = self.bundles[bundle.index()].ranges.last().unwrap().range.to;
            minimal = bundle_start.inst() == bundle_end.prev().inst();
            trace!("  -> minimal: {}", minimal);
        }

        let spill_weight = if minimal {
            if fixed {
                trace!("  -> fixed and minimal");
                MINIMAL_FIXED_BUNDLE_SPILL_WEIGHT
            } else {
                trace!("  -> non-fixed and minimal");
                MINIMAL_BUNDLE_SPILL_WEIGHT
            }
        } else {
            let mut total = SpillWeight::zero();
            for entry in &self.bundles[bundle.index()].ranges {
                let range_data = &self.ranges[entry.index.index()];
                trace!(
                    "  -> uses spill weight: +{:?}",
                    range_data.uses_spill_weight()
                );
                total = total + range_data.uses_spill_weight();
            }

            if self.bundles[bundle.index()].prio > 0 {
                let final_weight = (total.to_f32() as u32) / self.bundles[bundle.index()].prio;
                trace!(
                    " -> dividing by prio {}; final weight {}",
                    self.bundles[bundle.index()].prio,
                    final_weight
                );
                std::cmp::min(BUNDLE_MAX_NORMAL_SPILL_WEIGHT, final_weight)
            } else {
                0
            }
        };

        self.bundles[bundle.index()].set_cached_spill_weight_and_props(
            spill_weight,
            minimal,
            fixed,
            fixed_def,
            stack,
        );
    }

    pub fn minimal_bundle(&self, bundle: LiveBundleIndex) -> bool {
        self.bundles[bundle.index()].cached_minimal()
    }

    pub fn recompute_range_properties(&mut self, range: LiveRangeIndex) {
        let rangedata = &mut self.ranges[range.index()];
        let mut w = SpillWeight::zero();
        for u in &rangedata.uses {
            w = w + SpillWeight::from_bits(u.weight);
            trace!("range{}: use {:?}", range.index(), u);
        }
        rangedata.set_uses_spill_weight(w);
        if rangedata.uses.len() > 0 && rangedata.uses[0].operand.kind() == OperandKind::Def {
            // Note that we *set* the flag here, but we never *clear*
            // it: it may be set by a progmove as well (which does not
            // create an explicit use or def), and we want to preserve
            // that. We will never split or trim ranges in a way that
            // removes a def at the front and requires the flag to be
            // cleared.
            rangedata.set_flag(LiveRangeFlag::StartsAtDef);
        }
    }

    pub fn get_or_create_spill_bundle(
        &mut self,
        bundle: LiveBundleIndex,
        create_if_absent: bool,
    ) -> Option<LiveBundleIndex> {
        let ssidx = self.bundles[bundle.index()].spillset;
        let idx = self.spillsets[ssidx.index()].spill_bundle;
        if idx.is_valid() {
            Some(idx)
        } else if create_if_absent {
            let idx = self.create_bundle();
            self.spillsets[ssidx.index()].spill_bundle = idx;
            self.bundles[idx.index()].spillset = ssidx;
            self.spilled_bundles.push(idx);
            Some(idx)
        } else {
            None
        }
    }

    pub fn split_and_requeue_bundle(
        &mut self,
        bundle: LiveBundleIndex,
        mut split_at: ProgPoint,
        reg_hint: PReg,
        // Do we trim the parts around the split and put them in the
        // spill bundle?
        trim_ends_into_spill_bundle: bool,
    ) {
        self.stats.splits += 1;
        trace!(
            "split bundle {:?} at {:?} and requeue with reg hint (for first part) {:?}",
            bundle,
            split_at,
            reg_hint,
        );

        // Split `bundle` at `split_at`, creating new LiveRanges and
        // bundles (and updating vregs' linked lists appropriately),
        // and enqueue the new bundles.

        let spillset = self.bundles[bundle.index()].spillset;

        // Have we reached the maximum split count? If so, fall back
        // to a "minimal bundles and spill bundle" setup for this
        // bundle. See the doc-comment on
        // `split_into_minimal_bundles()` above for more.
        if self.spillsets[spillset.index()].splits >= MAX_SPLITS_PER_SPILLSET {
            self.split_into_minimal_bundles(bundle, reg_hint);
            return;
        }
        self.spillsets[spillset.index()].splits += 1;

        debug_assert!(!self.bundles[bundle.index()].ranges.is_empty());
        // Split point *at* start is OK; this means we peel off
        // exactly one use to create a minimal bundle.
        let bundle_start = self.bundles[bundle.index()]
            .ranges
            .first()
            .unwrap()
            .range
            .from;
        debug_assert!(split_at >= bundle_start);
        let bundle_end = self.bundles[bundle.index()].ranges.last().unwrap().range.to;
        debug_assert!(split_at < bundle_end);

        // Is the split point *at* the start? If so, peel off the
        // first use: set the split point just after it, or just
        // before it if it comes after the start of the bundle.
        if split_at == bundle_start {
            // Find any uses; if none, just chop off one instruction.
            let mut first_use = None;
            'outer: for entry in &self.bundles[bundle.index()].ranges {
                for u in &self.ranges[entry.index.index()].uses {
                    first_use = Some(u.pos);
                    break 'outer;
                }
            }
            trace!(" -> first use loc is {:?}", first_use);
            split_at = match first_use {
                Some(pos) => {
                    if pos.inst() == bundle_start.inst() {
                        ProgPoint::before(pos.inst().next())
                    } else {
                        ProgPoint::before(pos.inst())
                    }
                }
                None => ProgPoint::before(
                    self.bundles[bundle.index()]
                        .ranges
                        .first()
                        .unwrap()
                        .range
                        .from
                        .inst()
                        .next(),
                ),
            };
            trace!(
                "split point is at bundle start; advancing to {:?}",
                split_at
            );
        } else {
            // Don't split in the middle of an instruction -- this could
            // create impossible moves (we cannot insert a move between an
            // instruction's uses and defs).
            if split_at.pos() == InstPosition::After {
                split_at = split_at.next();
            }
            if split_at >= bundle_end {
                split_at = split_at.prev().prev();
            }
        }

        debug_assert!(split_at > bundle_start && split_at < bundle_end);

        // We need to find which LRs fall on each side of the split,
        // which LR we need to split down the middle, then update the
        // current bundle, create a new one, and (re)-queue both.

        trace!(" -> LRs: {:?}", self.bundles[bundle.index()].ranges);

        let mut last_lr_in_old_bundle_idx = 0; // last LR-list index in old bundle
        let mut first_lr_in_new_bundle_idx = 0; // first LR-list index in new bundle
        for (i, entry) in self.bundles[bundle.index()].ranges.iter().enumerate() {
            if split_at > entry.range.from {
                last_lr_in_old_bundle_idx = i;
                first_lr_in_new_bundle_idx = i;
            }
            if split_at < entry.range.to {
                first_lr_in_new_bundle_idx = i;
                break;
            }
        }

        trace!(
            " -> last LR in old bundle: LR {:?}",
            self.bundles[bundle.index()].ranges[last_lr_in_old_bundle_idx]
        );
        trace!(
            " -> first LR in new bundle: LR {:?}",
            self.bundles[bundle.index()].ranges[first_lr_in_new_bundle_idx]
        );

        // Take the sublist of LRs that will go in the new bundle.
        let mut new_lr_list: LiveRangeList = self.bundles[bundle.index()]
            .ranges
            .iter()
            .cloned()
            .skip(first_lr_in_new_bundle_idx)
            .collect();
        self.bundles[bundle.index()]
            .ranges
            .truncate(last_lr_in_old_bundle_idx + 1);
        self.bundles[bundle.index()].ranges.shrink_to_fit();

        // If the first entry in `new_lr_list` is a LR that is split
        // down the middle, replace it with a new LR and chop off the
        // end of the same LR in the original list.
        if split_at > new_lr_list[0].range.from {
            debug_assert_eq!(last_lr_in_old_bundle_idx, first_lr_in_new_bundle_idx);
            let orig_lr = new_lr_list[0].index;
            let new_lr = self.create_liverange(CodeRange {
                from: split_at,
                to: new_lr_list[0].range.to,
            });
            self.ranges[new_lr.index()].vreg = self.ranges[orig_lr.index()].vreg;
            trace!(" -> splitting LR {:?} into {:?}", orig_lr, new_lr);
            let first_use = self.ranges[orig_lr.index()]
                .uses
                .iter()
                .position(|u| u.pos >= split_at)
                .unwrap_or(self.ranges[orig_lr.index()].uses.len());
            let rest_uses: UseList = self.ranges[orig_lr.index()]
                .uses
                .iter()
                .cloned()
                .skip(first_use)
                .collect();
            self.ranges[new_lr.index()].uses = rest_uses;
            self.ranges[orig_lr.index()].uses.truncate(first_use);
            self.ranges[orig_lr.index()].uses.shrink_to_fit();
            self.recompute_range_properties(orig_lr);
            self.recompute_range_properties(new_lr);
            new_lr_list[0].index = new_lr;
            new_lr_list[0].range = self.ranges[new_lr.index()].range;
            self.ranges[orig_lr.index()].range.to = split_at;
            self.bundles[bundle.index()].ranges[last_lr_in_old_bundle_idx].range =
                self.ranges[orig_lr.index()].range;

            // Perform a lazy split in the VReg data. We just
            // append the new LR and its range; we will sort by
            // start of range, and fix up range ends, once when we
            // iterate over the VReg's ranges after allocation
            // completes (this is the only time when order
            // matters).
            self.vregs[self.ranges[new_lr.index()].vreg.index()]
                .ranges
                .push(LiveRangeListEntry {
                    range: self.ranges[new_lr.index()].range,
                    index: new_lr,
                });
        }

        let new_bundle = self.create_bundle();
        trace!(" -> creating new bundle {:?}", new_bundle);
        self.bundles[new_bundle.index()].spillset = spillset;
        for entry in &new_lr_list {
            self.ranges[entry.index.index()].bundle = new_bundle;
        }
        self.bundles[new_bundle.index()].ranges = new_lr_list;

        if trim_ends_into_spill_bundle {
            // Finally, handle moving LRs to the spill bundle when
            // appropriate: If the first range in `new_bundle` or last
            // range in `bundle` has "empty space" beyond the first or
            // last use (respectively), trim it and put an empty LR into
            // the spill bundle.  (We are careful to treat the "starts at
            // def" flag as an implicit first def even if no def-type Use
            // is present.)
            while let Some(entry) = self.bundles[bundle.index()].ranges.last().cloned() {
                let end = entry.range.to;
                let vreg = self.ranges[entry.index.index()].vreg;
                let last_use = self.ranges[entry.index.index()].uses.last().map(|u| u.pos);
                if last_use.is_none() {
                    let spill = self
                        .get_or_create_spill_bundle(bundle, /* create_if_absent = */ true)
                        .unwrap();
                    trace!(
                        " -> bundle {:?} range {:?}: no uses; moving to spill bundle {:?}",
                        bundle,
                        entry.index,
                        spill
                    );
                    self.bundles[spill.index()].ranges.push(entry);
                    self.bundles[bundle.index()].ranges.pop();
                    self.ranges[entry.index.index()].bundle = spill;
                    continue;
                }
                let last_use = last_use.unwrap();
                let split = ProgPoint::before(last_use.inst().next());
                if split < end {
                    let spill = self
                        .get_or_create_spill_bundle(bundle, /* create_if_absent = */ true)
                        .unwrap();
                    self.bundles[bundle.index()]
                        .ranges
                        .last_mut()
                        .unwrap()
                        .range
                        .to = split;
                    self.ranges[self.bundles[bundle.index()]
                        .ranges
                        .last()
                        .unwrap()
                        .index
                        .index()]
                    .range
                    .to = split;
                    let range = CodeRange {
                        from: split,
                        to: end,
                    };
                    let empty_lr = self.create_liverange(range);
                    self.bundles[spill.index()].ranges.push(LiveRangeListEntry {
                        range,
                        index: empty_lr,
                    });
                    self.ranges[empty_lr.index()].bundle = spill;
                    self.vregs[vreg.index()].ranges.push(LiveRangeListEntry {
                        range,
                        index: empty_lr,
                    });
                    trace!(
                        " -> bundle {:?} range {:?}: last use implies split point {:?}",
                        bundle,
                        entry.index,
                        split
                    );
                    trace!(
                    " -> moving trailing empty region to new spill bundle {:?} with new LR {:?}",
                    spill,
                    empty_lr
                );
                }
                break;
            }
            while let Some(entry) = self.bundles[new_bundle.index()].ranges.first().cloned() {
                if self.ranges[entry.index.index()].has_flag(LiveRangeFlag::StartsAtDef) {
                    break;
                }
                let start = entry.range.from;
                let vreg = self.ranges[entry.index.index()].vreg;
                let first_use = self.ranges[entry.index.index()].uses.first().map(|u| u.pos);
                if first_use.is_none() {
                    let spill = self
                        .get_or_create_spill_bundle(new_bundle, /* create_if_absent = */ true)
                        .unwrap();
                    trace!(
                        " -> bundle {:?} range {:?}: no uses; moving to spill bundle {:?}",
                        new_bundle,
                        entry.index,
                        spill
                    );
                    self.bundles[spill.index()].ranges.push(entry);
                    self.bundles[new_bundle.index()].ranges.drain(..1);
                    self.ranges[entry.index.index()].bundle = spill;
                    continue;
                }
                let first_use = first_use.unwrap();
                let split = ProgPoint::before(first_use.inst());
                if split > start {
                    let spill = self
                        .get_or_create_spill_bundle(new_bundle, /* create_if_absent = */ true)
                        .unwrap();
                    self.bundles[new_bundle.index()]
                        .ranges
                        .first_mut()
                        .unwrap()
                        .range
                        .from = split;
                    self.ranges[self.bundles[new_bundle.index()]
                        .ranges
                        .first()
                        .unwrap()
                        .index
                        .index()]
                    .range
                    .from = split;
                    let range = CodeRange {
                        from: start,
                        to: split,
                    };
                    let empty_lr = self.create_liverange(range);
                    self.bundles[spill.index()].ranges.push(LiveRangeListEntry {
                        range,
                        index: empty_lr,
                    });
                    self.ranges[empty_lr.index()].bundle = spill;
                    self.vregs[vreg.index()].ranges.push(LiveRangeListEntry {
                        range,
                        index: empty_lr,
                    });
                    trace!(
                        " -> bundle {:?} range {:?}: first use implies split point {:?}",
                        bundle,
                        entry.index,
                        first_use,
                    );
                    trace!(
                        " -> moving leading empty region to new spill bundle {:?} with new LR {:?}",
                        spill,
                        empty_lr
                    );
                }
                break;
            }
        }

        if self.bundles[bundle.index()].ranges.len() > 0 {
            self.recompute_bundle_properties(bundle);
            let prio = self.bundles[bundle.index()].prio;
            self.allocation_queue
                .insert(bundle, prio as usize, reg_hint);
        }
        if self.bundles[new_bundle.index()].ranges.len() > 0 {
            self.recompute_bundle_properties(new_bundle);
            let prio = self.bundles[new_bundle.index()].prio;
            self.allocation_queue
                .insert(new_bundle, prio as usize, reg_hint);
        }
    }

    /// Splits the given bundle into minimal bundles per Use, falling
    /// back onto the spill bundle. This must work for any bundle no
    /// matter how many conflicts.
    ///
    /// This is meant to solve a quadratic-cost problem that exists
    /// with "normal" splitting as implemented above. With that
    /// procedure, , splitting a bundle produces two
    /// halves. Furthermore, it has cost linear in the length of the
    /// bundle, because the resulting half-bundles have their
    /// requirements recomputed with a new scan, and because we copy
    /// half the use-list over to the tail end sub-bundle.
    ///
    /// This works fine when a bundle has a handful of splits overall,
    /// but not when an input has a systematic pattern of conflicts
    /// that will require O(|bundle|) splits (e.g., every Use is
    /// constrained to a different fixed register than the last
    /// one). In such a case, we get quadratic behavior.
    ///
    /// This method implements a direct split into minimal bundles
    /// along the whole length of the bundle, putting the regions
    /// without uses in the spill bundle. We do this once the number
    /// of splits in an original bundle (tracked by spillset) reaches
    /// a pre-determined limit.
    ///
    /// This basically approximates what a non-splitting allocator
    /// would do: it "spills" the whole bundle to possibly a
    /// stackslot, or a second-chance register allocation at best, via
    /// the spill bundle; and then does minimal reservations of
    /// registers just at uses/defs and moves the "spilled" value
    /// into/out of them immediately.
    pub fn split_into_minimal_bundles(&mut self, bundle: LiveBundleIndex, reg_hint: PReg) {
        let mut removed_lrs: FxHashSet<LiveRangeIndex> = FxHashSet::default();
        let mut removed_lrs_vregs: FxHashSet<VRegIndex> = FxHashSet::default();
        let mut new_lrs: SmallVec<[(VRegIndex, LiveRangeIndex); 16]> = smallvec![];
        let mut new_bundles: SmallVec<[LiveBundleIndex; 16]> = smallvec![];

        let spill = self
            .get_or_create_spill_bundle(bundle, /* create_if_absent = */ true)
            .unwrap();

        trace!(
            "Splitting bundle {:?} into minimal bundles with reg hint {}",
            bundle,
            reg_hint
        );

        let mut last_lr: Option<LiveRangeIndex> = None;
        let mut last_bundle: Option<LiveBundleIndex> = None;
        let mut last_inst: Option<Inst> = None;
        let mut last_vreg: Option<VRegIndex> = None;

        for entry_idx in 0..self.bundles[bundle.index()].ranges.len() {
            // Iterate manually; don't borrow `self`.
            let entry = self.bundles[bundle.index()].ranges[entry_idx];
            let lr_from = entry.range.from;
            let lr_to = entry.range.to;

            removed_lrs.insert(entry.index);
            let vreg = self.ranges[entry.index.index()].vreg;
            removed_lrs_vregs.insert(vreg);
            trace!(" -> removing old LR {:?} for vreg {:?}", entry.index, vreg);

            let mut last_live_pos = entry.range.from;
            for use_idx in 0..self.ranges[entry.index.index()].uses.len() {
                let u = self.ranges[entry.index.index()].uses[use_idx];
                trace!("   -> use {:?} (last_live_pos {:?})", u, last_live_pos);

                // If we just created a LR for this inst at the last
                // pos, add this use to the same LR.
                if Some(u.pos.inst()) == last_inst && Some(vreg) == last_vreg {
                    self.ranges[last_lr.unwrap().index()].uses.push(u);
                    trace!("    -> appended to last LR {:?}", last_lr.unwrap());
                    continue;
                }

                // The minimal bundle runs through the whole inst
                // (up to the Before of the next inst), *unless*
                // the original LR was only over the Before (up to
                // the After) of this inst.
                let to = std::cmp::min(ProgPoint::before(u.pos.inst().next()), lr_to);

                // If the last bundle was at the same inst, add a new
                // LR to the same bundle; otherwise, create a LR and a
                // new bundle.
                if Some(u.pos.inst()) == last_inst {
                    let cr = CodeRange { from: u.pos, to };
                    let lr = self.create_liverange(cr);
                    new_lrs.push((vreg, lr));
                    self.ranges[lr.index()].uses.push(u);
                    self.ranges[lr.index()].vreg = vreg;

                    trace!(
                        "    -> created new LR {:?} but adding to existing bundle {:?}",
                        lr,
                        last_bundle.unwrap()
                    );
                    // Edit the previous LR to end mid-inst.
                    self.bundles[last_bundle.unwrap().index()]
                        .ranges
                        .last_mut()
                        .unwrap()
                        .range
                        .to = u.pos;
                    self.ranges[last_lr.unwrap().index()].range.to = u.pos;
                    // Add this LR to the bundle.
                    self.bundles[last_bundle.unwrap().index()]
                        .ranges
                        .push(LiveRangeListEntry {
                            range: cr,
                            index: lr,
                        });
                    self.ranges[lr.index()].bundle = last_bundle.unwrap();
                    last_live_pos = ProgPoint::before(u.pos.inst().next());
                    continue;
                }

                // Otherwise, create a new LR.
                let pos = ProgPoint::before(u.pos.inst());
                let pos = std::cmp::max(lr_from, pos);
                let cr = CodeRange { from: pos, to };
                let lr = self.create_liverange(cr);
                new_lrs.push((vreg, lr));
                self.ranges[lr.index()].uses.push(u);
                self.ranges[lr.index()].vreg = vreg;

                // Create a new bundle that contains only this LR.
                let new_bundle = self.create_bundle();
                self.ranges[lr.index()].bundle = new_bundle;
                self.bundles[new_bundle.index()].spillset = self.bundles[bundle.index()].spillset;
                self.bundles[new_bundle.index()]
                    .ranges
                    .push(LiveRangeListEntry {
                        range: cr,
                        index: lr,
                    });
                new_bundles.push(new_bundle);

                // If this use was a Def, set the StartsAtDef flag for
                // the new LR. (N.B.: *not* Mod, only Def, because Mod
                // needs an input. This flag specifically indicates
                // that the LR does not require the value to be moved
                // into location at start because it (re)defines the
                // value.)
                if u.operand.kind() == OperandKind::Def {
                    self.ranges[lr.index()].set_flag(LiveRangeFlag::StartsAtDef);
                }

                trace!(
                    "    -> created new LR {:?} range {:?} with new bundle {:?} for this use",
                    lr,
                    cr,
                    new_bundle
                );

                // If there was any intervening range in the LR not
                // covered by the minimal new LR above, add it to the
                // spillset.
                if pos > last_live_pos {
                    let cr = CodeRange {
                        from: last_live_pos,
                        to: pos,
                    };
                    let spill_lr = self.create_liverange(cr);
                    self.ranges[spill_lr.index()].vreg = vreg;
                    self.ranges[spill_lr.index()].bundle = spill;
                    new_lrs.push((vreg, spill_lr));
                    self.bundles[spill.index()].ranges.push(LiveRangeListEntry {
                        range: cr,
                        index: spill_lr,
                    });
                    self.ranges[spill_lr.index()].bundle = spill;
                    trace!(
                        "    -> put intervening range {:?} in new LR {:?} in spill bundle {:?}",
                        cr,
                        spill_lr,
                        spill
                    );
                }
                last_live_pos = ProgPoint::before(u.pos.inst().next());

                last_lr = Some(lr);
                last_bundle = Some(new_bundle);
                last_inst = Some(u.pos.inst());
                last_vreg = Some(vreg);
            }

            // Clear the use-list from the original LR.
            self.ranges[entry.index.index()].uses = Default::default();

            // If there is space from the last use to the end of the
            // LR, put that in the spill bundle too.
            if entry.range.to > last_live_pos {
                let cr = CodeRange {
                    from: last_live_pos,
                    to: entry.range.to,
                };
                let spill_lr = self.create_liverange(cr);
                self.ranges[spill_lr.index()].vreg = vreg;
                self.ranges[spill_lr.index()].bundle = spill;
                new_lrs.push((vreg, spill_lr));
                self.bundles[spill.index()].ranges.push(LiveRangeListEntry {
                    range: cr,
                    index: spill_lr,
                });
                self.ranges[spill_lr.index()].bundle = spill;
                trace!(
                    "    -> put trailing range {:?} in new LR {:?} in spill bundle {:?}",
                    cr,
                    spill_lr,
                    spill
                );
            }
        }

        // Clear the LR list in the original bundle.
        self.bundles[bundle.index()].ranges.clear();
        self.bundles[bundle.index()].ranges.shrink_to_fit();

        // Remove all of the removed LRs from respective vregs' lists.
        for vreg in removed_lrs_vregs {
            self.vregs[vreg.index()]
                .ranges
                .retain(|entry| !removed_lrs.contains(&entry.index));
        }

        // Add the new LRs to their respective vreg lists.
        for (vreg, lr) in new_lrs {
            let range = self.ranges[lr.index()].range;
            let entry = LiveRangeListEntry { range, index: lr };
            self.vregs[vreg.index()].ranges.push(entry);
        }

        // Recompute bundle properties for all new bundles and enqueue
        // them.
        for bundle in new_bundles {
            if self.bundles[bundle.index()].ranges.len() > 0 {
                self.recompute_bundle_properties(bundle);
                let prio = self.bundles[bundle.index()].prio;
                self.allocation_queue
                    .insert(bundle, prio as usize, reg_hint);
            }
        }
    }

    pub fn process_bundle(
        &mut self,
        bundle: LiveBundleIndex,
        reg_hint: PReg,
    ) -> Result<(), RegAllocError> {
        let class = self.spillsets[self.bundles[bundle.index()].spillset.index()].class;
        // Grab a hint from either the queue or our spillset, if any.
        let mut hint_reg = if reg_hint != PReg::invalid() {
            reg_hint
        } else {
            self.spillsets[self.bundles[bundle.index()].spillset.index()].reg_hint
        };
        if self.pregs[hint_reg.index()].is_stack {
            hint_reg = PReg::invalid();
        }
        trace!("process_bundle: bundle {:?} hint {:?}", bundle, hint_reg,);

        let req = match self.compute_requirement(bundle) {
            Ok(req) => req,
            Err(conflict) => {
                // We have to split right away. We'll find a point to
                // split that would allow at least the first half of the
                // split to be conflict-free.
                debug_assert!(
                    !self.minimal_bundle(bundle),
                    "Minimal bundle with conflict!"
                );
                self.split_and_requeue_bundle(
                    bundle,
                    /* split_at_point = */ conflict.suggested_split_point(),
                    reg_hint,
                    /* trim_ends_into_spill_bundle = */
                    conflict.should_trim_edges_around_split(),
                );
                return Ok(());
            }
        };

        // If no requirement at all (because no uses), and *if* a
        // spill bundle is already present, then move the LRs over to
        // the spill bundle right away.
        match req {
            Requirement::Any => {
                if let Some(spill) =
                    self.get_or_create_spill_bundle(bundle, /* create_if_absent = */ false)
                {
                    let mut list =
                        std::mem::replace(&mut self.bundles[bundle.index()].ranges, smallvec![]);
                    for entry in &list {
                        self.ranges[entry.index.index()].bundle = spill;
                    }
                    self.bundles[spill.index()].ranges.extend(list.drain(..));
                    return Ok(());
                }
            }
            _ => {}
        }

        // Try to allocate!
        let mut attempts = 0;
        loop {
            attempts += 1;
            trace!("attempt {}, req {:?}", attempts, req);
            debug_assert!(attempts < 100 * self.func.num_insts());

            let fixed_preg = match req {
                Requirement::FixedReg(preg) | Requirement::FixedStack(preg) => Some(preg),
                Requirement::Register => None,
                Requirement::Stack => {
                    // If we must be on the stack, mark our spillset
                    // as required immediately.
                    self.spillsets[self.bundles[bundle.index()].spillset.index()].required = true;
                    return Ok(());
                }

                Requirement::Any => {
                    self.spilled_bundles.push(bundle);
                    return Ok(());
                }
            };
            // Scan all pregs, or the one fixed preg, and attempt to allocate.

            let mut lowest_cost_evict_conflict_set: Option<LiveBundleVec> = None;
            let mut lowest_cost_evict_conflict_cost: Option<u32> = None;

            let mut lowest_cost_split_conflict_cost: Option<u32> = None;
            let mut lowest_cost_split_conflict_point = ProgPoint::before(Inst::new(0));
            let mut lowest_cost_split_conflict_reg = PReg::invalid();

            // Heuristic: start the scan for an available
            // register at an offset influenced both by our
            // location in the code and by the bundle we're
            // considering. This has the effect of spreading
            // demand more evenly across registers.
            let scan_offset = self.ranges[self.bundles[bundle.index()].ranges[0].index.index()]
                .range
                .from
                .inst()
                .index()
                + bundle.index();

            self.stats.process_bundle_reg_probe_start_any += 1;
            for preg in RegTraversalIter::new(
                self.env,
                class,
                hint_reg,
                PReg::invalid(),
                scan_offset,
                fixed_preg,
            ) {
                self.stats.process_bundle_reg_probes_any += 1;
                let preg_idx = PRegIndex::new(preg.index());
                trace!("trying preg {:?}", preg_idx);

                let scan_limit_cost = match (
                    lowest_cost_evict_conflict_cost,
                    lowest_cost_split_conflict_cost,
                ) {
                    (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
                    _ => None,
                };
                match self.try_to_allocate_bundle_to_reg(bundle, preg_idx, scan_limit_cost) {
                    AllocRegResult::Allocated(alloc) => {
                        self.stats.process_bundle_reg_success_any += 1;
                        trace!(" -> allocated to any {:?}", preg_idx);
                        self.spillsets[self.bundles[bundle.index()].spillset.index()].reg_hint =
                            alloc.as_reg().unwrap();
                        return Ok(());
                    }
                    AllocRegResult::Conflict(bundles, first_conflict_point) => {
                        trace!(
                            " -> conflict with bundles {:?}, first conflict at {:?}",
                            bundles,
                            first_conflict_point
                        );

                        let conflict_cost = self.maximum_spill_weight_in_bundle_set(&bundles);

                        if lowest_cost_evict_conflict_cost.is_none()
                            || conflict_cost < lowest_cost_evict_conflict_cost.unwrap()
                        {
                            lowest_cost_evict_conflict_cost = Some(conflict_cost);
                            lowest_cost_evict_conflict_set = Some(bundles);
                        }

                        let loop_depth = self.cfginfo.approx_loop_depth
                            [self.cfginfo.insn_block[first_conflict_point.inst().index()].index()];
                        let move_cost = spill_weight_from_constraint(
                            OperandConstraint::Reg,
                            loop_depth as usize,
                            /* is_def = */ true,
                        )
                        .to_int();
                        if lowest_cost_split_conflict_cost.is_none()
                            || (conflict_cost + move_cost)
                                < lowest_cost_split_conflict_cost.unwrap()
                        {
                            lowest_cost_split_conflict_cost = Some(conflict_cost + move_cost);
                            lowest_cost_split_conflict_point = first_conflict_point;
                            lowest_cost_split_conflict_reg = preg;
                        }
                    }
                    AllocRegResult::ConflictWithFixed(max_cost, point) => {
                        trace!(" -> conflict with fixed alloc; cost of other bundles up to point is {}, conflict at {:?}", max_cost, point);

                        let loop_depth = self.cfginfo.approx_loop_depth
                            [self.cfginfo.insn_block[point.inst().index()].index()];
                        let move_cost = spill_weight_from_constraint(
                            OperandConstraint::Reg,
                            loop_depth as usize,
                            /* is_def = */ true,
                        )
                        .to_int();

                        if lowest_cost_split_conflict_cost.is_none()
                            || (max_cost + move_cost) < lowest_cost_split_conflict_cost.unwrap()
                        {
                            lowest_cost_split_conflict_cost = Some(max_cost + move_cost);
                            lowest_cost_split_conflict_point = point;
                            lowest_cost_split_conflict_reg = preg;
                        }
                    }
                    AllocRegResult::ConflictHighCost => {
                        // Simply don't consider -- we already have
                        // a lower-cost conflict bundle option
                        // to evict.
                        continue;
                    }
                }
            }

            // Otherwise, we *require* a register, but didn't fit into
            // any with current bundle assignments. Hence, we will need
            // to either split or attempt to evict some bundles.

            trace!(
                " -> lowest cost evict: set {:?}, cost {:?}",
                lowest_cost_evict_conflict_set,
                lowest_cost_evict_conflict_cost,
            );
            trace!(
                " -> lowest cost split: cost {:?}, point {:?}, reg {:?}",
                lowest_cost_split_conflict_cost,
                lowest_cost_split_conflict_point,
                lowest_cost_split_conflict_reg
            );

            // If we reach here, we *must* have an option either to split or evict.
            debug_assert!(
                lowest_cost_split_conflict_cost.is_some()
                    || lowest_cost_evict_conflict_cost.is_some()
            );

            let our_spill_weight = self.bundle_spill_weight(bundle);
            trace!(" -> our spill weight: {}", our_spill_weight);

            // We detect the "too-many-live-registers" case here and
            // return an error cleanly, rather than panicking, because
            // the regalloc.rs fuzzer depends on the register
            // allocator to correctly reject impossible-to-allocate
            // programs in order to discard invalid test cases.
            if self.minimal_bundle(bundle)
                && (attempts >= 2
                    || lowest_cost_evict_conflict_cost.is_none()
                    || lowest_cost_evict_conflict_cost.unwrap() >= our_spill_weight)
            {
                if let Requirement::Register = req {
                    // Check if this is a too-many-live-registers situation.
                    let range = self.bundles[bundle.index()].ranges[0].range;
                    trace!("checking for too many live regs");
                    let mut min_bundles_assigned = 0;
                    let mut fixed_assigned = 0;
                    let mut total_regs = 0;
                    for preg in self.env.preferred_regs_by_class[class as u8 as usize]
                        .iter()
                        .chain(self.env.non_preferred_regs_by_class[class as u8 as usize].iter())
                    {
                        trace!(" -> PR {:?}", preg);
                        let start = LiveRangeKey::from_range(&CodeRange {
                            from: range.from.prev(),
                            to: range.from.prev(),
                        });
                        for (key, lr) in self.pregs[preg.index()].allocations.btree.range(start..) {
                            let preg_range = key.to_range();
                            if preg_range.to <= range.from {
                                continue;
                            }
                            if preg_range.from >= range.to {
                                break;
                            }
                            if lr.is_valid() {
                                if self.minimal_bundle(self.ranges[lr.index()].bundle) {
                                    trace!("  -> min bundle {:?}", lr);
                                    min_bundles_assigned += 1;
                                } else {
                                    trace!("  -> non-min bundle {:?}", lr);
                                }
                            } else {
                                trace!("  -> fixed bundle");
                                fixed_assigned += 1;
                            }
                        }
                        total_regs += 1;
                    }
                    trace!(
                        " -> total {}, fixed {}, min {}",
                        total_regs,
                        fixed_assigned,
                        min_bundles_assigned
                    );
                    if min_bundles_assigned + fixed_assigned >= total_regs {
                        return Err(RegAllocError::TooManyLiveRegs);
                    }
                }

                panic!("Could not allocate minimal bundle, but the allocation problem should be possible to solve");
            }

            // If our bundle's weight is less than or equal to(*) the
            // evict cost, choose to split.  Also pick splitting if
            // we're on our second or more attempt and we didn't
            // allocate.  Also pick splitting if the conflict set is
            // empty, meaning a fixed conflict that can't be evicted.
            //
            // (*) the "equal to" part is very important: it prevents
            // an infinite loop where two bundles with equal spill
            // cost continually evict each other in an infinite
            // allocation loop. In such a case, the first bundle in
            // wins, and the other splits.
            //
            // Note that we don't split if the bundle is minimal.
            if !self.minimal_bundle(bundle)
                && (attempts >= 2
                    || lowest_cost_evict_conflict_cost.is_none()
                    || our_spill_weight <= lowest_cost_evict_conflict_cost.unwrap())
            {
                trace!(
                    " -> deciding to split: our spill weight is {}",
                    self.bundle_spill_weight(bundle)
                );
                let bundle_start = self.bundles[bundle.index()].ranges[0].range.from;
                let mut split_at_point =
                    std::cmp::max(lowest_cost_split_conflict_point, bundle_start);
                let requeue_with_reg = lowest_cost_split_conflict_reg;

                // Adjust `split_at_point` if it is within a deeper loop
                // than the bundle start -- hoist it to just before the
                // first loop header it encounters.
                let bundle_start_depth = self.cfginfo.approx_loop_depth
                    [self.cfginfo.insn_block[bundle_start.inst().index()].index()];
                let split_at_depth = self.cfginfo.approx_loop_depth
                    [self.cfginfo.insn_block[split_at_point.inst().index()].index()];
                if split_at_depth > bundle_start_depth {
                    for block in (self.cfginfo.insn_block[bundle_start.inst().index()].index() + 1)
                        ..=self.cfginfo.insn_block[split_at_point.inst().index()].index()
                    {
                        if self.cfginfo.approx_loop_depth[block] > bundle_start_depth {
                            split_at_point = self.cfginfo.block_entry[block];
                            break;
                        }
                    }
                }

                self.split_and_requeue_bundle(
                    bundle,
                    split_at_point,
                    requeue_with_reg,
                    /* should_trim = */ true,
                );
                return Ok(());
            } else {
                // Evict all bundles in `conflicting bundles` and try again.
                self.stats.evict_bundle_event += 1;
                for &bundle in &lowest_cost_evict_conflict_set.unwrap() {
                    trace!(" -> evicting {:?}", bundle);
                    self.evict_bundle(bundle);
                    self.stats.evict_bundle_count += 1;
                }
            }
        }
    }
}
