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

//! Bundle merging.

use super::{
    Env, LiveBundleIndex, LiveRangeIndex, LiveRangeKey, SpillSet, SpillSetIndex, SpillSlotIndex,
    VRegIndex,
};
use crate::{
    ion::data_structures::BlockparamOut, Function, Inst, OperandConstraint, OperandKind, PReg,
};
use smallvec::smallvec;

impl<'a, F: Function> Env<'a, F> {
    pub fn merge_bundles(&mut self, from: LiveBundleIndex, to: LiveBundleIndex) -> bool {
        if from == to {
            // Merge bundle into self -- trivial merge.
            return true;
        }
        trace!(
            "merging from bundle{} to bundle{}",
            from.index(),
            to.index()
        );

        // Both bundles must deal with the same RegClass.
        let from_rc = self.spillsets[self.bundles[from.index()].spillset.index()].class;
        let to_rc = self.spillsets[self.bundles[to.index()].spillset.index()].class;
        if from_rc != to_rc {
            trace!(" -> mismatching reg classes");
            return false;
        }

        // If either bundle is already assigned (due to a pinned vreg), don't merge.
        if self.bundles[from.index()].allocation.is_some()
            || self.bundles[to.index()].allocation.is_some()
        {
            trace!("one of the bundles is already assigned (pinned)");
            return false;
        }

        #[cfg(debug_assertions)]
        {
            // Sanity check: both bundles should contain only ranges with appropriate VReg classes.
            for entry in &self.bundles[from.index()].ranges {
                let vreg = self.ranges[entry.index.index()].vreg;
                debug_assert_eq!(from_rc, self.vreg(vreg).class());
            }
            for entry in &self.bundles[to.index()].ranges {
                let vreg = self.ranges[entry.index.index()].vreg;
                debug_assert_eq!(to_rc, self.vreg(vreg).class());
            }
        }

        // Check for overlap in LiveRanges and for conflicting
        // requirements.
        let ranges_from = &self.bundles[from.index()].ranges[..];
        let ranges_to = &self.bundles[to.index()].ranges[..];
        let mut idx_from = 0;
        let mut idx_to = 0;
        let mut range_count = 0;
        while idx_from < ranges_from.len() && idx_to < ranges_to.len() {
            range_count += 1;
            if range_count > 200 {
                trace!(
                    "reached merge complexity (range_count = {}); exiting",
                    range_count
                );
                // Limit merge complexity.
                return false;
            }

            if ranges_from[idx_from].range.from >= ranges_to[idx_to].range.to {
                idx_to += 1;
            } else if ranges_to[idx_to].range.from >= ranges_from[idx_from].range.to {
                idx_from += 1;
            } else {
                // Overlap -- cannot merge.
                trace!(
                    " -> overlap between {:?} and {:?}, exiting",
                    ranges_from[idx_from].index,
                    ranges_to[idx_to].index
                );
                return false;
            }
        }

        // Avoid merging if either side has a fixed-reg def: this can
        // result in an impossible-to-solve allocation problem if
        // there is a fixed-reg use in the same reg on the same
        // instruction.
        if self.bundles[from.index()].cached_fixed_def()
            || self.bundles[to.index()].cached_fixed_def()
        {
            trace!(" -> one bundle has a fixed def; aborting merge");
            return false;
        }

        // Check for a requirements conflict.
        if self.bundles[from.index()].cached_stack()
            || self.bundles[from.index()].cached_fixed()
            || self.bundles[to.index()].cached_stack()
            || self.bundles[to.index()].cached_fixed()
        {
            if self.merge_bundle_requirements(from, to).is_err() {
                trace!(" -> conflicting requirements; aborting merge");
                return false;
            }
        }

        trace!(" -> committing to merge");

        // If we reach here, then the bundles do not overlap -- merge
        // them!  We do this with a merge-sort-like scan over both
        // lists, building a new range list and replacing the list on
        // `to` when we're done.
        if ranges_from.is_empty() {
            // `from` bundle is empty -- trivial merge.
            trace!(" -> from bundle{} is empty; trivial merge", from.index());
            return true;
        }
        if ranges_to.is_empty() {
            // `to` bundle is empty -- just move the list over from
            // `from` and set `bundle` up-link on all ranges.
            trace!(" -> to bundle{} is empty; trivial merge", to.index());
            let list = std::mem::replace(&mut self.bundles[from.index()].ranges, smallvec![]);
            for entry in &list {
                self.ranges[entry.index.index()].bundle = to;

                if self.annotations_enabled {
                    self.annotate(
                        entry.range.from,
                        format!(
                            " MERGE range{} v{} from bundle{} to bundle{}",
                            entry.index.index(),
                            self.ranges[entry.index.index()].vreg.index(),
                            from.index(),
                            to.index(),
                        ),
                    );
                }
            }
            self.bundles[to.index()].ranges = list;

            if self.bundles[from.index()].cached_stack() {
                self.bundles[to.index()].set_cached_stack();
            }
            if self.bundles[from.index()].cached_fixed() {
                self.bundles[to.index()].set_cached_fixed();
            }

            return true;
        }

        trace!(
            "merging: ranges_from = {:?} ranges_to = {:?}",
            ranges_from,
            ranges_to
        );

        // Two non-empty lists of LiveRanges: concatenate and
        // sort. This is faster than a mergesort-like merge into a new
        // list, empirically.
        let from_list = std::mem::replace(&mut self.bundles[from.index()].ranges, smallvec![]);
        for entry in &from_list {
            self.ranges[entry.index.index()].bundle = to;
        }
        self.bundles[to.index()]
            .ranges
            .extend_from_slice(&from_list[..]);
        self.bundles[to.index()]
            .ranges
            .sort_unstable_by_key(|entry| entry.range.from);

        if self.annotations_enabled {
            trace!("merging: merged = {:?}", self.bundles[to.index()].ranges);
            let mut last_range = None;
            for i in 0..self.bundles[to.index()].ranges.len() {
                let entry = self.bundles[to.index()].ranges[i];
                if last_range.is_some() {
                    debug_assert!(last_range.unwrap() < entry.range);
                }
                last_range = Some(entry.range);

                if self.ranges[entry.index.index()].bundle == from {
                    self.annotate(
                        entry.range.from,
                        format!(
                            " MERGE range{} v{} from bundle{} to bundle{}",
                            entry.index.index(),
                            self.ranges[entry.index.index()].vreg.index(),
                            from.index(),
                            to.index(),
                        ),
                    );
                }

                trace!(
                    " -> merged result for bundle{}: range{}",
                    to.index(),
                    entry.index.index(),
                );
            }
        }

        if self.bundles[from.index()].spillset != self.bundles[to.index()].spillset {
            let from_vregs = std::mem::replace(
                &mut self.spillsets[self.bundles[from.index()].spillset.index()].vregs,
                smallvec![],
            );
            let to_vregs = &mut self.spillsets[self.bundles[to.index()].spillset.index()].vregs;
            for vreg in from_vregs {
                if !to_vregs.contains(&vreg) {
                    to_vregs.push(vreg);
                }
            }
        }

        if self.bundles[from.index()].cached_stack() {
            self.bundles[to.index()].set_cached_stack();
        }
        if self.bundles[from.index()].cached_fixed() {
            self.bundles[to.index()].set_cached_fixed();
        }

        true
    }

    pub fn merge_vreg_bundles(&mut self) {
        // Create a bundle for every vreg, initially.
        trace!("merge_vreg_bundles: creating vreg bundles");
        for vreg in 0..self.vregs.len() {
            let vreg = VRegIndex::new(vreg);
            if self.vregs[vreg.index()].ranges.is_empty() {
                continue;
            }

            // If this is a pinned vreg, go ahead and add it to the
            // commitment map, and avoid creating a bundle entirely.
            if let Some(preg) = self.func.is_pinned_vreg(self.vreg(vreg)) {
                for entry in &self.vregs[vreg.index()].ranges {
                    let key = LiveRangeKey::from_range(&entry.range);
                    self.pregs[preg.index()]
                        .allocations
                        .btree
                        .insert(key, LiveRangeIndex::invalid());
                }
                continue;
            }

            let bundle = self.create_bundle();
            self.bundles[bundle.index()].ranges = self.vregs[vreg.index()].ranges.clone();
            trace!("vreg v{} gets bundle{}", vreg.index(), bundle.index());
            for entry in &self.bundles[bundle.index()].ranges {
                trace!(
                    " -> with LR range{}: {:?}",
                    entry.index.index(),
                    entry.range
                );
                self.ranges[entry.index.index()].bundle = bundle;
            }

            let mut fixed = false;
            let mut fixed_def = false;
            let mut stack = false;
            for entry in &self.bundles[bundle.index()].ranges {
                for u in &self.ranges[entry.index.index()].uses {
                    if let OperandConstraint::FixedReg(_) = u.operand.constraint() {
                        fixed = true;
                        if u.operand.kind() == OperandKind::Def {
                            fixed_def = true;
                        }
                    }
                    if let OperandConstraint::Stack = u.operand.constraint() {
                        stack = true;
                    }
                    if fixed && stack && fixed_def {
                        break;
                    }
                }
            }
            if fixed {
                self.bundles[bundle.index()].set_cached_fixed();
            }
            if fixed_def {
                self.bundles[bundle.index()].set_cached_fixed_def();
            }
            if stack {
                self.bundles[bundle.index()].set_cached_stack();
            }

            // Create a spillslot for this bundle.
            let ssidx = SpillSetIndex::new(self.spillsets.len());
            let reg = self.vreg(vreg);
            let size = self.func.spillslot_size(reg.class()) as u8;
            self.spillsets.push(SpillSet {
                vregs: smallvec![vreg],
                slot: SpillSlotIndex::invalid(),
                size,
                required: false,
                class: reg.class(),
                reg_hint: PReg::invalid(),
                spill_bundle: LiveBundleIndex::invalid(),
                splits: 0,
            });
            self.bundles[bundle.index()].spillset = ssidx;
        }

        for inst in 0..self.func.num_insts() {
            let inst = Inst::new(inst);

            // Attempt to merge Reuse-constraint operand outputs with the
            // corresponding inputs.
            for op in self.func.inst_operands(inst) {
                if let OperandConstraint::Reuse(reuse_idx) = op.constraint() {
                    let src_vreg = op.vreg();
                    let dst_vreg = self.func.inst_operands(inst)[reuse_idx].vreg();
                    if self.func.is_pinned_vreg(src_vreg).is_some()
                        || self.func.is_pinned_vreg(dst_vreg).is_some()
                    {
                        continue;
                    }

                    trace!(
                        "trying to merge reused-input def: src {} to dst {}",
                        src_vreg,
                        dst_vreg
                    );
                    let src_bundle =
                        self.ranges[self.vregs[src_vreg.vreg()].ranges[0].index.index()].bundle;
                    debug_assert!(src_bundle.is_valid());
                    let dest_bundle =
                        self.ranges[self.vregs[dst_vreg.vreg()].ranges[0].index.index()].bundle;
                    debug_assert!(dest_bundle.is_valid());
                    self.merge_bundles(/* from */ dest_bundle, /* to */ src_bundle);
                }
            }
        }

        // Attempt to merge blockparams with their inputs.
        for i in 0..self.blockparam_outs.len() {
            let BlockparamOut {
                from_vreg, to_vreg, ..
            } = self.blockparam_outs[i];
            trace!(
                "trying to merge blockparam v{} with input v{}",
                to_vreg.index(),
                from_vreg.index()
            );
            let to_bundle = self.ranges[self.vregs[to_vreg.index()].ranges[0].index.index()].bundle;
            debug_assert!(to_bundle.is_valid());
            let from_bundle =
                self.ranges[self.vregs[from_vreg.index()].ranges[0].index.index()].bundle;
            debug_assert!(from_bundle.is_valid());
            trace!(
                " -> from bundle{} to bundle{}",
                from_bundle.index(),
                to_bundle.index()
            );
            self.merge_bundles(from_bundle, to_bundle);
        }

        // Attempt to merge move srcs/dsts.
        for i in 0..self.prog_move_merges.len() {
            let (src, dst) = self.prog_move_merges[i];
            trace!("trying to merge move src LR {:?} to dst LR {:?}", src, dst);
            let src = self.resolve_merged_lr(src);
            let dst = self.resolve_merged_lr(dst);
            trace!(
                "resolved LR-construction merging chains: move-merge is now src LR {:?} to dst LR {:?}",
                src,
                dst
            );

            let dst_vreg = self.vreg(self.ranges[dst.index()].vreg);
            let src_vreg = self.vreg(self.ranges[src.index()].vreg);
            if self.func.is_pinned_vreg(src_vreg).is_some()
                && self.func.is_pinned_vreg(dst_vreg).is_some()
            {
                continue;
            }
            if let Some(preg) = self.func.is_pinned_vreg(src_vreg) {
                let dest_bundle = self.ranges[dst.index()].bundle;
                let spillset = self.bundles[dest_bundle.index()].spillset;
                self.spillsets[spillset.index()].reg_hint = preg;
                continue;
            }
            if let Some(preg) = self.func.is_pinned_vreg(dst_vreg) {
                let src_bundle = self.ranges[src.index()].bundle;
                let spillset = self.bundles[src_bundle.index()].spillset;
                self.spillsets[spillset.index()].reg_hint = preg;
                continue;
            }

            let src_bundle = self.ranges[src.index()].bundle;
            debug_assert!(src_bundle.is_valid());
            let dest_bundle = self.ranges[dst.index()].bundle;
            debug_assert!(dest_bundle.is_valid());
            self.stats.prog_move_merge_attempt += 1;
            if self.merge_bundles(/* from */ dest_bundle, /* to */ src_bundle) {
                self.stats.prog_move_merge_success += 1;
            }
        }

        trace!("done merging bundles");
    }

    pub fn resolve_merged_lr(&self, mut lr: LiveRangeIndex) -> LiveRangeIndex {
        let mut iter = 0;
        while iter < 100 && self.ranges[lr.index()].merged_into.is_valid() {
            lr = self.ranges[lr.index()].merged_into;
            iter += 1;
        }
        lr
    }

    pub fn compute_bundle_prio(&self, bundle: LiveBundleIndex) -> u32 {
        // The priority is simply the total "length" -- the number of
        // instructions covered by all LiveRanges.
        let mut total = 0;
        for entry in &self.bundles[bundle.index()].ranges {
            total += entry.range.len() as u32;
        }
        total
    }

    pub fn queue_bundles(&mut self) {
        for bundle in 0..self.bundles.len() {
            trace!("enqueueing bundle{}", bundle);
            if self.bundles[bundle].ranges.is_empty() {
                trace!(" -> no ranges; skipping");
                continue;
            }
            let bundle = LiveBundleIndex::new(bundle);
            let prio = self.compute_bundle_prio(bundle);
            trace!(" -> prio {}", prio);
            self.bundles[bundle.index()].prio = prio;
            self.recompute_bundle_properties(bundle);
            self.allocation_queue
                .insert(bundle, prio as usize, PReg::invalid());
        }
        self.stats.merged_bundle_count = self.allocation_queue.heap.len();
    }
}
