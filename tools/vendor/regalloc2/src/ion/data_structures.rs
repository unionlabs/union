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

//! Data structures for backtracking allocator.

use super::liveranges::SpillWeight;
use crate::cfg::CFGInfo;
use crate::index::ContainerComparator;
use crate::indexset::IndexSet;
use crate::{
    define_index, Allocation, Block, Edit, Function, Inst, MachineEnv, Operand, PReg, ProgPoint,
    RegClass, VReg,
};
use fxhash::FxHashSet;
use smallvec::SmallVec;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;

/// A range from `from` (inclusive) to `to` (exclusive).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeRange {
    pub from: ProgPoint,
    pub to: ProgPoint,
}

impl CodeRange {
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.from == self.to
    }
    #[inline(always)]
    pub fn contains(&self, other: &Self) -> bool {
        other.from >= self.from && other.to <= self.to
    }
    #[inline(always)]
    pub fn contains_point(&self, other: ProgPoint) -> bool {
        other >= self.from && other < self.to
    }
    #[inline(always)]
    pub fn overlaps(&self, other: &Self) -> bool {
        other.to > self.from && other.from < self.to
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.to.inst().index() - self.from.inst().index()
    }
    /// Returns the range covering just one program point.
    #[inline(always)]
    pub fn singleton(pos: ProgPoint) -> CodeRange {
        CodeRange {
            from: pos,
            to: pos.next(),
        }
    }
}

impl std::cmp::PartialOrd for CodeRange {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for CodeRange {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.to <= other.from {
            Ordering::Less
        } else if self.from >= other.to {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

define_index!(LiveBundleIndex);
define_index!(LiveRangeIndex);
define_index!(SpillSetIndex);
define_index!(UseIndex);
define_index!(VRegIndex);
define_index!(PRegIndex);
define_index!(SpillSlotIndex);

/// Used to carry small sets of bundles, e.g. for conflict sets.
pub type LiveBundleVec = SmallVec<[LiveBundleIndex; 4]>;

#[derive(Clone, Copy, Debug)]
pub struct LiveRangeListEntry {
    pub range: CodeRange,
    pub index: LiveRangeIndex,
}

pub type LiveRangeList = SmallVec<[LiveRangeListEntry; 4]>;
pub type UseList = SmallVec<[Use; 4]>;

#[derive(Clone, Debug)]
pub struct LiveRange {
    pub range: CodeRange,

    pub vreg: VRegIndex,
    pub bundle: LiveBundleIndex,
    pub uses_spill_weight_and_flags: u32,

    pub uses: UseList,

    pub merged_into: LiveRangeIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum LiveRangeFlag {
    StartsAtDef = 1,
}

impl LiveRange {
    #[inline(always)]
    pub fn set_flag(&mut self, flag: LiveRangeFlag) {
        self.uses_spill_weight_and_flags |= (flag as u32) << 29;
    }
    #[inline(always)]
    pub fn clear_flag(&mut self, flag: LiveRangeFlag) {
        self.uses_spill_weight_and_flags &= !((flag as u32) << 29);
    }
    #[inline(always)]
    pub fn assign_flag(&mut self, flag: LiveRangeFlag, val: bool) {
        let bit = if val { (flag as u32) << 29 } else { 0 };
        self.uses_spill_weight_and_flags &= 0xe000_0000;
        self.uses_spill_weight_and_flags |= bit;
    }
    #[inline(always)]
    pub fn has_flag(&self, flag: LiveRangeFlag) -> bool {
        self.uses_spill_weight_and_flags & ((flag as u32) << 29) != 0
    }
    #[inline(always)]
    pub fn flag_word(&self) -> u32 {
        self.uses_spill_weight_and_flags & 0xe000_0000
    }
    #[inline(always)]
    pub fn merge_flags(&mut self, flag_word: u32) {
        self.uses_spill_weight_and_flags |= flag_word;
    }
    #[inline(always)]
    pub fn uses_spill_weight(&self) -> SpillWeight {
        let bits = (self.uses_spill_weight_and_flags & 0x1fff_ffff) << 2;
        SpillWeight::from_f32(f32::from_bits(bits))
    }
    #[inline(always)]
    pub fn set_uses_spill_weight(&mut self, weight: SpillWeight) {
        let weight_bits = (weight.to_f32().to_bits() >> 2) & 0x1fff_ffff;
        self.uses_spill_weight_and_flags =
            (self.uses_spill_weight_and_flags & 0xe000_0000) | weight_bits;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Use {
    pub operand: Operand,
    pub pos: ProgPoint,
    pub slot: u8,
    pub weight: u16,
}

impl Use {
    #[inline(always)]
    pub fn new(operand: Operand, pos: ProgPoint, slot: u8) -> Self {
        Self {
            operand,
            pos,
            slot,
            // Weight is updated on insertion into LR.
            weight: 0,
        }
    }
}

pub const SLOT_NONE: u8 = u8::MAX;

#[derive(Clone, Debug)]
pub struct LiveBundle {
    pub ranges: LiveRangeList,
    pub spillset: SpillSetIndex,
    pub allocation: Allocation,
    pub prio: u32, // recomputed after every bulk update
    pub spill_weight_and_props: u32,
}

pub const BUNDLE_MAX_SPILL_WEIGHT: u32 = (1 << 28) - 1;
pub const MINIMAL_FIXED_BUNDLE_SPILL_WEIGHT: u32 = BUNDLE_MAX_SPILL_WEIGHT;
pub const MINIMAL_BUNDLE_SPILL_WEIGHT: u32 = BUNDLE_MAX_SPILL_WEIGHT - 1;
pub const BUNDLE_MAX_NORMAL_SPILL_WEIGHT: u32 = BUNDLE_MAX_SPILL_WEIGHT - 2;

impl LiveBundle {
    #[inline(always)]
    pub fn set_cached_spill_weight_and_props(
        &mut self,
        spill_weight: u32,
        minimal: bool,
        fixed: bool,
        fixed_def: bool,
        stack: bool,
    ) {
        debug_assert!(spill_weight <= BUNDLE_MAX_SPILL_WEIGHT);
        self.spill_weight_and_props = spill_weight
            | (if minimal { 1 << 31 } else { 0 })
            | (if fixed { 1 << 30 } else { 0 })
            | (if fixed_def { 1 << 29 } else { 0 })
            | (if stack { 1 << 28 } else { 0 });
    }

    #[inline(always)]
    pub fn cached_minimal(&self) -> bool {
        self.spill_weight_and_props & (1 << 31) != 0
    }

    #[inline(always)]
    pub fn cached_fixed(&self) -> bool {
        self.spill_weight_and_props & (1 << 30) != 0
    }

    #[inline(always)]
    pub fn cached_fixed_def(&self) -> bool {
        self.spill_weight_and_props & (1 << 29) != 0
    }

    #[inline(always)]
    pub fn cached_stack(&self) -> bool {
        self.spill_weight_and_props & (1 << 28) != 0
    }

    #[inline(always)]
    pub fn set_cached_fixed(&mut self) {
        self.spill_weight_and_props |= 1 << 30;
    }

    #[inline(always)]
    pub fn set_cached_fixed_def(&mut self) {
        self.spill_weight_and_props |= 1 << 29;
    }

    #[inline(always)]
    pub fn set_cached_stack(&mut self) {
        self.spill_weight_and_props |= 1 << 28;
    }

    #[inline(always)]
    pub fn cached_spill_weight(&self) -> u32 {
        self.spill_weight_and_props & ((1 << 28) - 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BundleProperties {
    pub minimal: bool,
    pub fixed: bool,
}

/// Calculate the maximum `N` inline capacity for a `SmallVec<[T; N]>` we can
/// have without bloating its size to be larger than a `Vec<T>`.
const fn no_bloat_capacity<T>() -> usize {
    // `Vec<T>` is three words: `(pointer, capacity, length)`.
    //
    // A `SmallVec<[T; N]>` replaces the first two members with the following:
    //
    //     union {
    //         Inline([T; N]),
    //         Heap(pointer, capacity),
    //     }
    //
    // So if `size_of([T; N]) == size_of(pointer) + size_of(capacity)` then we
    // get the maximum inline capacity without bloat.
    std::mem::size_of::<usize>() * 2 / std::mem::size_of::<T>()
}

#[derive(Clone, Debug)]
pub struct SpillSet {
    pub vregs: SmallVec<[VRegIndex; no_bloat_capacity::<VRegIndex>()]>,
    pub slot: SpillSlotIndex,
    pub reg_hint: PReg,
    pub class: RegClass,
    pub spill_bundle: LiveBundleIndex,
    pub required: bool,
    pub size: u8,
    pub splits: u8,
}

pub(crate) const MAX_SPLITS_PER_SPILLSET: u8 = 10;

#[derive(Clone, Debug)]
pub struct VRegData {
    pub ranges: LiveRangeList,
    pub blockparam: Block,
    pub is_ref: bool,
    // We don't initially know the RegClass until we observe a use of the VReg.
    pub class: Option<RegClass>,
}

#[derive(Clone, Debug)]
pub struct PRegData {
    pub allocations: LiveRangeSet,
    pub is_stack: bool,
}

#[derive(Clone, Debug)]
pub struct MultiFixedRegFixup {
    pub pos: ProgPoint,
    pub from_slot: u8,
    pub to_slot: u8,
    pub level: FixedRegFixupLevel,
    pub to_preg: PRegIndex,
    pub vreg: VRegIndex,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FixedRegFixupLevel {
    /// A fixup copy for the initial fixed reg; must come first.
    Initial,
    /// A fixup copy from the first fixed reg to other fixed regs for
    /// the same vreg; must come second.
    Secondary,
}

/// The field order is significant: these are sorted so that a
/// scan over vregs, then blocks in each range, can scan in
/// order through this (sorted) list and add allocs to the
/// half-move list.
///
/// The fields in this struct are reversed in sort order so that the entire
/// struct can be treated as a u128 for sorting purposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct BlockparamOut {
    pub to_vreg: VRegIndex,
    pub to_block: Block,
    pub from_block: Block,
    pub from_vreg: VRegIndex,
}
impl BlockparamOut {
    #[inline(always)]
    pub fn key(&self) -> u128 {
        u128_key(
            self.from_vreg.raw_u32(),
            self.from_block.raw_u32(),
            self.to_block.raw_u32(),
            self.to_vreg.raw_u32(),
        )
    }
}

/// As above for `BlockparamIn`, field order is significant.
///
/// The fields in this struct are reversed in sort order so that the entire
/// struct can be treated as a u128 for sorting purposes.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct BlockparamIn {
    pub from_block: Block,
    pub to_block: Block,
    pub to_vreg: VRegIndex,
}
impl BlockparamIn {
    #[inline(always)]
    pub fn key(&self) -> u128 {
        u128_key(
            self.to_vreg.raw_u32(),
            self.to_block.raw_u32(),
            self.from_block.raw_u32(),
            0,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Env<'a, F: Function> {
    pub func: &'a F,
    pub env: &'a MachineEnv,
    pub cfginfo: CFGInfo,
    pub liveins: Vec<IndexSet>,
    pub liveouts: Vec<IndexSet>,
    pub blockparam_outs: Vec<BlockparamOut>,
    pub blockparam_ins: Vec<BlockparamIn>,

    pub ranges: Vec<LiveRange>,
    pub bundles: Vec<LiveBundle>,
    pub spillsets: Vec<SpillSet>,
    pub vregs: Vec<VRegData>,
    pub pregs: Vec<PRegData>,
    pub allocation_queue: PrioQueue,
    pub safepoints: Vec<Inst>, // Sorted list of safepoint insts.
    pub safepoints_per_vreg: HashMap<usize, HashSet<Inst>>,

    pub spilled_bundles: Vec<LiveBundleIndex>,
    pub spillslots: Vec<SpillSlotData>,
    pub slots_by_size: Vec<SpillSlotList>,

    pub extra_spillslots_by_class: [SmallVec<[Allocation; 2]>; 2],
    pub preferred_victim_by_class: [PReg; 2],

    // Program moves: these are moves in the provided program that we
    // handle with our internal machinery, in order to avoid the
    // overhead of ordinary operand processing. We expect the client
    // to not generate any code for instructions that return
    // `Some(..)` for `.is_move()`, and instead use the edits that we
    // provide to implement those moves (or some simplified version of
    // them) post-regalloc.
    //
    // (from-vreg, inst, from-alloc), sorted by (from-vreg, inst)
    pub prog_move_srcs: Vec<((VRegIndex, Inst), Allocation)>,
    // (to-vreg, inst, to-alloc), sorted by (to-vreg, inst)
    pub prog_move_dsts: Vec<((VRegIndex, Inst), Allocation)>,
    // (from-vreg, to-vreg) for bundle-merging.
    pub prog_move_merges: Vec<(LiveRangeIndex, LiveRangeIndex)>,

    // When multiple fixed-register constraints are present on a
    // single VReg at a single program point (this can happen for,
    // e.g., call args that use the same value multiple times), we
    // remove all but one of the fixed-register constraints, make a
    // note here, and add a clobber with that PReg instread to keep
    // the register available. When we produce the final edit-list, we
    // will insert a copy from wherever the VReg's primary allocation
    // was to the approprate PReg.
    pub multi_fixed_reg_fixups: Vec<MultiFixedRegFixup>,

    pub inserted_moves: Vec<InsertedMove>,

    // Output:
    pub edits: Vec<(PosWithPrio, Edit)>,
    pub allocs: Vec<Allocation>,
    pub inst_alloc_offsets: Vec<u32>,
    pub num_spillslots: u32,
    pub safepoint_slots: Vec<(ProgPoint, Allocation)>,
    pub debug_locations: Vec<(u32, ProgPoint, ProgPoint, Allocation)>,

    pub allocated_bundle_count: usize,

    pub stats: Stats,

    // For debug output only: a list of textual annotations at every
    // ProgPoint to insert into the final allocated program listing.
    pub debug_annotations: std::collections::HashMap<ProgPoint, Vec<String>>,
    pub annotations_enabled: bool,

    // Cached allocation for `try_to_allocate_bundle_to_reg` to avoid allocating
    // a new HashSet on every call.
    pub conflict_set: FxHashSet<LiveBundleIndex>,
}

impl<'a, F: Function> Env<'a, F> {
    /// Get the VReg (with bundled RegClass) from a vreg index.
    #[inline]
    pub fn vreg(&self, index: VRegIndex) -> VReg {
        let class = self.vregs[index.index()]
            .class
            .expect("trying to get a VReg before observing its class");
        VReg::new(index.index(), class)
    }

    /// Record the class of a VReg. We learn this only when we observe
    /// the VRegs in use.
    pub fn observe_vreg_class(&mut self, vreg: VReg) {
        let old_class = self.vregs[vreg.vreg()].class.replace(vreg.class());
        // We should never observe two different classes for two
        // mentions of a VReg in the source program.
        debug_assert!(old_class == None || old_class == Some(vreg.class()));
    }

    /// Is this vreg actually used in the source program?
    pub fn is_vreg_used(&self, index: VRegIndex) -> bool {
        self.vregs[index.index()].class.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct SpillSlotData {
    pub ranges: LiveRangeSet,
    pub slots: u32,
    pub alloc: Allocation,
}

#[derive(Clone, Debug)]
pub struct SpillSlotList {
    pub slots: SmallVec<[SpillSlotIndex; 32]>,
    pub probe_start: usize,
}

impl SpillSlotList {
    /// Get the next spillslot index in probing order, wrapping around
    /// at the end of the slots list.
    pub(crate) fn next_index(&self, index: usize) -> usize {
        debug_assert!(index < self.slots.len());
        if index == self.slots.len() - 1 {
            0
        } else {
            index + 1
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrioQueue {
    pub heap: std::collections::BinaryHeap<PrioQueueEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrioQueueEntry {
    pub prio: u32,
    pub bundle: LiveBundleIndex,
    pub reg_hint: PReg,
}

#[derive(Clone, Debug)]
pub struct LiveRangeSet {
    pub btree: BTreeMap<LiveRangeKey, LiveRangeIndex>,
}

#[derive(Clone, Copy, Debug)]
pub struct LiveRangeKey {
    pub from: u32,
    pub to: u32,
}

impl LiveRangeKey {
    #[inline(always)]
    pub fn from_range(range: &CodeRange) -> Self {
        Self {
            from: range.from.to_index(),
            to: range.to.to_index(),
        }
    }

    #[inline(always)]
    pub fn to_range(&self) -> CodeRange {
        CodeRange {
            from: ProgPoint::from_index(self.from),
            to: ProgPoint::from_index(self.to),
        }
    }
}

impl std::cmp::PartialEq for LiveRangeKey {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.to > other.from && self.from < other.to
    }
}
impl std::cmp::Eq for LiveRangeKey {}
impl std::cmp::PartialOrd for LiveRangeKey {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for LiveRangeKey {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.to <= other.from {
            std::cmp::Ordering::Less
        } else if self.from >= other.to {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

pub struct PrioQueueComparator<'a> {
    pub prios: &'a [usize],
}
impl<'a> ContainerComparator for PrioQueueComparator<'a> {
    type Ix = LiveBundleIndex;
    fn compare(&self, a: Self::Ix, b: Self::Ix) -> std::cmp::Ordering {
        self.prios[a.index()].cmp(&self.prios[b.index()])
    }
}

impl PrioQueue {
    pub fn new() -> Self {
        PrioQueue {
            heap: std::collections::BinaryHeap::new(),
        }
    }

    #[inline(always)]
    pub fn insert(&mut self, bundle: LiveBundleIndex, prio: usize, reg_hint: PReg) {
        self.heap.push(PrioQueueEntry {
            prio: prio as u32,
            bundle,
            reg_hint,
        });
    }

    #[inline(always)]
    pub fn is_empty(self) -> bool {
        self.heap.is_empty()
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<(LiveBundleIndex, PReg)> {
        self.heap.pop().map(|entry| (entry.bundle, entry.reg_hint))
    }
}

impl LiveRangeSet {
    pub(crate) fn new() -> Self {
        Self {
            btree: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InsertedMove {
    pub pos_prio: PosWithPrio,
    pub from_alloc: Allocation,
    pub to_alloc: Allocation,
    pub to_vreg: VReg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InsertMovePrio {
    InEdgeMoves,
    BlockParam,
    Regular,
    PostRegular,
    MultiFixedRegInitial,
    MultiFixedRegSecondary,
    ReusedInput,
    OutEdgeMoves,
}

/// The fields in this struct are reversed in sort order so that the entire
/// struct can be treated as a u64 for sorting purposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct PosWithPrio {
    pub prio: u32,
    pub pos: ProgPoint,
}

impl PosWithPrio {
    #[inline]
    pub fn key(self) -> u64 {
        u64_key(self.pos.to_index(), self.prio)
    }
}

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "enable-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Stats {
    pub livein_blocks: usize,
    pub livein_iterations: usize,
    pub initial_liverange_count: usize,
    pub merged_bundle_count: usize,
    pub prog_moves: usize,
    pub prog_moves_dead_src: usize,
    pub prog_move_merge_attempt: usize,
    pub prog_move_merge_success: usize,
    pub process_bundle_count: usize,
    pub process_bundle_reg_probes_fixed: usize,
    pub process_bundle_reg_success_fixed: usize,
    pub process_bundle_bounding_range_probe_start_any: usize,
    pub process_bundle_bounding_range_probes_any: usize,
    pub process_bundle_bounding_range_success_any: usize,
    pub process_bundle_reg_probe_start_any: usize,
    pub process_bundle_reg_probes_any: usize,
    pub process_bundle_reg_success_any: usize,
    pub evict_bundle_event: usize,
    pub evict_bundle_count: usize,
    pub splits: usize,
    pub splits_clobbers: usize,
    pub splits_hot: usize,
    pub splits_conflicts: usize,
    pub splits_defs: usize,
    pub splits_all: usize,
    pub final_liverange_count: usize,
    pub final_bundle_count: usize,
    pub spill_bundle_count: usize,
    pub spill_bundle_reg_probes: usize,
    pub spill_bundle_reg_success: usize,
    pub blockparam_ins_count: usize,
    pub blockparam_outs_count: usize,
    pub halfmoves_count: usize,
    pub edits_count: usize,
}

// Helper function for generating sorting keys. The order of arguments is from
// the most significant field to the least significant one.
//
// These work best when the fields are stored in reverse order in memory so that
// they can be loaded with a single u64 load on little-endian machines.
#[inline(always)]
pub fn u64_key(b: u32, a: u32) -> u64 {
    a as u64 | (b as u64) << 32
}
#[inline(always)]
pub fn u128_key(d: u32, c: u32, b: u32, a: u32) -> u128 {
    a as u128 | (b as u128) << 32 | (c as u128) << 64 | (d as u128) << 96
}
