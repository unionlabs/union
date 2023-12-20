//! Last-store tracking via alias analysis.
//!
//! We partition memory state into several *disjoint pieces* of
//! "abstract state". There are a finite number of such pieces:
//! currently, we call them "heap", "table", "vmctx", and "other". Any
//! given address in memory belongs to exactly one disjoint piece.
//!
//! One never tracks which piece a concrete address belongs to at
//! runtime; this is a purely static concept. Instead, all
//! memory-accessing instructions (loads and stores) are labeled with
//! one of these four categories in the `MemFlags`. It is forbidden
//! for a load or store to access memory under one category and a
//! later load or store to access the same memory under a different
//! category. This is ensured to be true by construction during
//! frontend translation into CLIF and during legalization.
//!
//! Given that this non-aliasing property is ensured by the producer
//! of CLIF, we can compute a *may-alias* property: one load or store
//! may-alias another load or store if both access the same category
//! of abstract state.
//!
//! The "last store" pass helps to compute this aliasing: we perform a
//! fixpoint analysis to track the last instruction that *might have*
//! written to a given part of abstract state. We also track the block
//! containing this store.
//!
//! We can't say for sure that the "last store" *did* actually write
//! that state, but we know for sure that no instruction *later* than
//! it (up to the current instruction) did. However, we can get a
//! must-alias property from this: if at a given load or store, we
//! look backward to the "last store", *AND* we find that it has
//! exactly the same address expression and value type, then we know
//! that the current instruction's access *must* be to the same memory
//! location.
//!
//! To get this must-alias property, we leverage the node
//! hashconsing. We design the Eq/Hash (node identity relation
//! definition) of the `Node` struct so that all loads with (i) the
//! same "last store", and (ii) the same address expression, and (iii)
//! the same opcode-and-offset, will deduplicate (the first will be
//! computed, and the later ones will use the same value). Furthermore
//! we have an optimization that rewrites a load into the stored value
//! of the last store *if* the last store has the same address
//! expression and constant offset.
//!
//! This gives us two optimizations, "redundant load elimination" and
//! "store-to-load forwarding".
//!
//! In theory we could also do *dead-store elimination*, where if a
//! store overwrites a value earlier written by another store, *and*
//! if no other load/store to the abstract state category occurred,
//! *and* no other trapping instruction occurred (at which point we
//! need an up-to-date memory state because post-trap-termination
//! memory state can be observed), *and* we can prove the original
//! store could not have trapped, then we can eliminate the original
//! store. Because this is so complex, and the conditions for doing it
//! correctly when post-trap state must be correct likely reduce the
//! potential benefit, we don't yet do this.

use crate::flowgraph::ControlFlowGraph;
use crate::fx::{FxHashMap, FxHashSet};
use crate::inst_predicates::has_memory_fence_semantics;
use crate::ir::{Block, Function, Inst, InstructionData, MemFlags, Opcode};
use crate::trace;
use cranelift_entity::{EntityRef, SecondaryMap};
use smallvec::{smallvec, SmallVec};

/// For a given program point, the vector of last-store instruction
/// indices for each disjoint category of abstract state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LastStores {
    heap: MemoryState,
    table: MemoryState,
    vmctx: MemoryState,
    other: MemoryState,
}

/// State of memory seen by a load.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum MemoryState {
    /// State at function entry: nothing is known (but it is one
    /// consistent value, so two loads from "entry" state at the same
    /// address will still provide the same result).
    #[default]
    Entry,
    /// State just after a store by the given instruction. The
    /// instruction is a store from which we can forward.
    Store(Inst),
    /// State just before the given instruction. Used for abstract
    /// value merges at merge-points when we cannot name a single
    /// producing site.
    BeforeInst(Inst),
    /// State just after the given instruction. Used when the
    /// instruction may update the associated state, but is not a
    /// store whose value we can cleanly forward. (E.g., perhaps a
    /// barrier of some sort.)
    AfterInst(Inst),
}

/// Memory state index, packed into a u32.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackedMemoryState(u32);

impl From<MemoryState> for PackedMemoryState {
    fn from(state: MemoryState) -> Self {
        match state {
            MemoryState::Entry => Self(0),
            MemoryState::Store(i) => Self(1 | (i.index() as u32) << 2),
            MemoryState::BeforeInst(i) => Self(2 | (i.index() as u32) << 2),
            MemoryState::AfterInst(i) => Self(3 | (i.index() as u32) << 2),
        }
    }
}

impl PackedMemoryState {
    /// Does this memory state refer to a specific store instruction?
    pub fn as_store(&self) -> Option<Inst> {
        if self.0 & 3 == 1 {
            Some(Inst::from_bits(self.0 >> 2))
        } else {
            None
        }
    }
}

impl LastStores {
    fn update(&mut self, func: &Function, inst: Inst) {
        let opcode = func.dfg[inst].opcode();
        if has_memory_fence_semantics(opcode) {
            self.heap = MemoryState::AfterInst(inst);
            self.table = MemoryState::AfterInst(inst);
            self.vmctx = MemoryState::AfterInst(inst);
            self.other = MemoryState::AfterInst(inst);
        } else if opcode.can_store() {
            if let Some(memflags) = func.dfg[inst].memflags() {
                *self.for_flags(memflags) = MemoryState::Store(inst);
            } else {
                self.heap = MemoryState::AfterInst(inst);
                self.table = MemoryState::AfterInst(inst);
                self.vmctx = MemoryState::AfterInst(inst);
                self.other = MemoryState::AfterInst(inst);
            }
        }
    }

    fn for_flags(&mut self, memflags: MemFlags) -> &mut MemoryState {
        if memflags.heap() {
            &mut self.heap
        } else if memflags.table() {
            &mut self.table
        } else if memflags.vmctx() {
            &mut self.vmctx
        } else {
            &mut self.other
        }
    }

    fn meet_from(&mut self, other: &LastStores, loc: Inst) {
        let meet = |a: MemoryState, b: MemoryState| -> MemoryState {
            match (a, b) {
                (a, b) if a == b => a,
                _ => MemoryState::BeforeInst(loc),
            }
        };

        self.heap = meet(self.heap, other.heap);
        self.table = meet(self.table, other.table);
        self.vmctx = meet(self.vmctx, other.vmctx);
        self.other = meet(self.other, other.other);
    }
}

/// An alias-analysis pass.
pub struct AliasAnalysis {
    /// Last-store instruction (or none) for a given load. Use a hash map
    /// instead of a `SecondaryMap` because this is sparse.
    load_mem_state: FxHashMap<Inst, PackedMemoryState>,
}

impl AliasAnalysis {
    /// Perform an alias analysis pass.
    pub fn new(func: &Function, cfg: &ControlFlowGraph) -> AliasAnalysis {
        log::trace!("alias analysis: input is:\n{:?}", func);
        let block_input = Self::compute_block_input_states(func, cfg);
        let load_mem_state = Self::compute_load_last_stores(func, block_input);
        AliasAnalysis { load_mem_state }
    }

    fn compute_block_input_states(
        func: &Function,
        cfg: &ControlFlowGraph,
    ) -> SecondaryMap<Block, Option<LastStores>> {
        let mut block_input = SecondaryMap::with_capacity(func.dfg.num_blocks());
        let mut worklist: SmallVec<[Block; 16]> = smallvec![];
        let mut worklist_set = FxHashSet::default();
        let entry = func.layout.entry_block().unwrap();
        worklist.push(entry);
        worklist_set.insert(entry);
        block_input[entry] = Some(LastStores::default());

        while let Some(block) = worklist.pop() {
            worklist_set.remove(&block);
            let state = block_input[block].clone().unwrap();

            trace!("alias analysis: input to {} is {:?}", block, state);

            let state = func
                .layout
                .block_insts(block)
                .fold(state, |mut state, inst| {
                    state.update(func, inst);
                    trace!("after {}: state is {:?}", inst, state);
                    state
                });

            for succ in cfg.succ_iter(block) {
                let succ_first_inst = func.layout.first_inst(succ).unwrap();
                let succ_state = &mut block_input[succ];
                let old = succ_state.clone();
                if let Some(succ_state) = succ_state.as_mut() {
                    succ_state.meet_from(&state, succ_first_inst);
                } else {
                    *succ_state = Some(state);
                };
                let updated = *succ_state != old;

                if updated && worklist_set.insert(succ) {
                    worklist.push(succ);
                }
            }
        }

        block_input
    }

    fn compute_load_last_stores(
        func: &Function,
        block_input: SecondaryMap<Block, Option<LastStores>>,
    ) -> FxHashMap<Inst, PackedMemoryState> {
        let mut load_mem_state = FxHashMap::default();
        load_mem_state.reserve(func.dfg.num_insts() / 8);

        for block in func.layout.blocks() {
            let mut state = block_input[block].clone().unwrap();

            for inst in func.layout.block_insts(block) {
                trace!(
                    "alias analysis: scanning at {} with state {:?} ({:?})",
                    inst,
                    state,
                    func.dfg[inst],
                );

                // N.B.: we match `Load` specifically, and not any
                // other kinds of loads (or any opcode such that
                // `opcode.can_load()` returns true), because some
                // "can load" instructions actually have very
                // different semantics (are not just a load of a
                // particularly-typed value). For example, atomic
                // (load/store, RMW, CAS) instructions "can load" but
                // definitely should not participate in store-to-load
                // forwarding or redundant-load elimination. Our goal
                // here is to provide a `MemoryState` just for plain
                // old loads whose semantics we can completely reason
                // about.
                if let InstructionData::Load {
                    opcode: Opcode::Load,
                    flags,
                    ..
                } = func.dfg[inst]
                {
                    let mem_state = *state.for_flags(flags);
                    trace!(
                        "alias analysis: at {}: load with mem_state {:?}",
                        inst,
                        mem_state,
                    );

                    load_mem_state.insert(inst, mem_state.into());
                }

                state.update(func, inst);
            }
        }

        load_mem_state
    }

    /// Get the state seen by a load, if any.
    pub fn get_state_for_load(&self, inst: Inst) -> Option<PackedMemoryState> {
        self.load_mem_state.get(&inst).copied()
    }
}
