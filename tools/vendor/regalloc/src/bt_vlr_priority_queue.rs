#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//! Backtracking allocator: the as-yet-unallocated VirtualReg LR prio queue.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::data_structures::{TypedIxVec, VirtualRange, VirtualRangeIx};

//=============================================================================
// The as-yet-unallocated VirtualReg LR prio queue, `VirtualRangePrioQ`.
//
// Relevant methods are parameterised by a VirtualRange env.

// What we seek to do with `VirtualRangePrioQ` is to implement a priority
// queue of as-yet unallocated virtual live ranges.  For each iteration of the
// main allocation loop, we pull out the highest-priority unallocated
// VirtualRange, and either allocate it (somehow), or spill it.
//
// The Rust standard type BinaryHeap gives us an efficient way to implement
// the priority queue.  However, it requires that the queue items supply the
// necessary cost-comparisons by implementing `Ord` on that type.  Hence we
// have to wrap up the items we fundamentally want in the queue, viz,
// `VirtualRangeIx`, into a new type `VirtualRangeIxAndSize` that also carries
// the relevant cost field, `size`.  Then we implement `Ord` for
// `VirtualRangeIxAndSize` so as to only look at the `size` fields.
//
// There is a small twist, however.  Most virtual ranges are small and so will
// have a small `size` field (less than 20, let's say).  For such cases,
// `BinaryHeap` will presumably choose between contenders with the same `size`
// field in some arbitrary way.  This has two disadvantages:
//
// * it makes the exact allocation order, and hence allocation results,
//   dependent on `BinaryHeap`'s arbitrary-choice scheme.  This seems
//   undesirable, and given recent shenanigans resulting from `HashMap` being
//   nondeterministic even in a single-threaded scenario, I don't entirely
//   trust `BinaryHeap` even to be deterministic.
//
// * experimentation with the "qsort" test case shows that breaking ties by
//   selecting the entry that has been in the queue the longest, rather than
//   choosing arbitrarily, gives slightly better allocations (slightly less
//   spilling) in spill-heavy situations (where there are few registers).
//   When there is not much spilling, it makes no difference.
//
// For these reasons, `VirtualRangeIxAndSize` also contains a `tiebreaker`
// field.  The `VirtualRangePrioQ` logic gives a different value of this for
// each `VirtualRangeIxAndSize` it creates.  These numbers start off at 2^32-1
// and decrease towards zero.  They are used as a secondary comparison key in
// the case where the `size` fields are equal.  The effect is that (1)
// tiebreaking is made completely deterministic, and (2) it breaks ties in
// favour of the oldest entry (since that will have the highest `tiebreaker`
// field).
//
// The tiebreaker field will wrap around when it hits zero, but that can only
// happen after processing 2^32-1 virtual live ranges.  In such cases I would
// expect that the allocator would have run out of memory long before, so it's
// academic in practice.  Even if it does wrap around there is no danger to
// the correctness of the allocations.

// Wrap up a VirtualRangeIx and its size, so that we can implement Ord for it
// on the basis of the `size` and `tiebreaker` fields.
//
// NB! Do not derive {,Partial}{Eq,Ord} for this.  It has its own custom
// implementations.
struct VirtualRangeIxAndSize {
    vlrix: VirtualRangeIx,
    size: u16,
    tiebreaker: u32,
}
impl VirtualRangeIxAndSize {
    fn new(vlrix: VirtualRangeIx, size: u16, tiebreaker: u32) -> Self {
        assert!(size > 0);
        Self {
            vlrix,
            size,
            tiebreaker,
        }
    }
}
impl PartialEq for VirtualRangeIxAndSize {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.tiebreaker == other.tiebreaker
    }
}
impl Eq for VirtualRangeIxAndSize {}
impl PartialOrd for VirtualRangeIxAndSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for VirtualRangeIxAndSize {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.size.cmp(&other.size) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => self.tiebreaker.cmp(&other.tiebreaker),
        }
    }
}

//=============================================================================
// VirtualRangePrioQ: public interface

pub struct VirtualRangePrioQ {
    // The set of as-yet unallocated VirtualRangeIxs.  These are indexes into a
    // VirtualRange env that is not stored here.  The VirtualRangeIxs are tagged
    // with the VirtualRange size and a tiebreaker field.
    heap: BinaryHeap<VirtualRangeIxAndSize>,
    tiebreaker_ctr: u32,
}
impl VirtualRangePrioQ {
    pub fn new(vlr_env: &TypedIxVec<VirtualRangeIx, VirtualRange>) -> Self {
        let mut res = Self {
            heap: BinaryHeap::new(),
            tiebreaker_ctr: 0xFFFF_FFFFu32,
        };
        for vlrix in VirtualRangeIx::new(0).dotdot(VirtualRangeIx::new(vlr_env.len())) {
            let to_add = VirtualRangeIxAndSize::new(vlrix, vlr_env[vlrix].size, res.tiebreaker_ctr);
            res.heap.push(to_add);
            res.tiebreaker_ctr -= 1;
        }
        res
    }

    #[inline(never)]
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    #[inline(never)]
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    // Add a VirtualRange index.
    #[inline(never)]
    pub fn add_VirtualRange(
        &mut self,
        vlr_env: &TypedIxVec<VirtualRangeIx, VirtualRange>,
        vlrix: VirtualRangeIx,
    ) {
        let to_add = VirtualRangeIxAndSize::new(vlrix, vlr_env[vlrix].size, self.tiebreaker_ctr);
        self.tiebreaker_ctr -= 1;
        self.heap.push(to_add);
    }

    // Look in `unallocated` to locate the entry referencing the VirtualRange
    // with the largest `size` value.  Remove the ref from `unallocated` and
    // return the VLRIx for said entry.
    #[inline(never)]
    pub fn get_longest_VirtualRange(&mut self) -> Option<VirtualRangeIx> {
        match self.heap.pop() {
            None => None,
            Some(VirtualRangeIxAndSize { vlrix, .. }) => Some(vlrix),
        }
    }

    #[inline(never)]
    pub fn show_with_envs(
        &self,
        vlr_env: &TypedIxVec<VirtualRangeIx, VirtualRange>,
    ) -> Vec<String> {
        let mut resV = vec![];
        for VirtualRangeIxAndSize { vlrix, .. } in self.heap.iter() {
            let mut res = "TODO  ".to_string();
            res += &format!("{:?} = {:?}", vlrix, &vlr_env[*vlrix]);
            resV.push(res);
        }
        resV
    }
}
