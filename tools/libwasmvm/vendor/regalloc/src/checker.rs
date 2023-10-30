//! Checker: verifies that spills/reloads/moves retain equivalent dataflow to original, vreg-based
//! code.
//!
//! The basic idea is that we track symbolic values as they flow through spills and reloads.
//! The symbolic values represent particular virtual or real registers in the original
//! function body presented to the register allocator. Any instruction in the original
//! function body (i.e., not added by the allocator) conceptually generates a symbolic
//! value "Rn" or "Vn" when storing to (or modifying) a real or virtual register. This
//! includes moves (from e.g. phi-node lowering): they also generate a new value.
//!
//! In other words, the dataflow analysis state at each program point is:
//!
//!   - map `R` of: real reg -> lattice value  (top > Rn/Vn symbols (unordered) > bottom)
//!   - map `S` of: spill slot -> lattice value (same)
//!
//! And the transfer functions for each statement type are:
//!
//!   - spill (inserted by RA):    [ store spill_i, R_j ]
//!
//!       S[spill_i] := R[R_j]
//!
//!   - reload (inserted by RA):   [ load R_i, spill_j ]
//!
//!       R[R_i] := S[spill_j]
//!
//!   - move (inserted by RA):     [ R_i := R_j ]
//!
//!       R[R_i] := R[R_j]
//!
//!   - statement in pre-regalloc function [ V_i := op V_j, V_k, ... ]
//!     with allocated form                [ R_i := op R_j, R_k, ... ]
//!
//!       R[R_i] := `V_i`
//!
//!     In other words, a statement, even after allocation, generates a symbol
//!     that corresponds to its original virtual-register def.
//!
//!     (N.B.: moves in pre-regalloc function fall into this last case -- they
//!      are "just another operation" and generate a new symbol)
//!
//!     (Slight extension for multi-def ops, and ops with "modify" args: the op
//!      generates symbol `V_i` into reg `R_i` allocated for that particular def/mod).
//!
//! The initial state is: for each real reg R_livein where R_livein is in the livein set, we set
//! R[R_livein] to `R_livein`.
//!
//! At control-flow join points, the symbols meet using a very simple lattice meet-function:
//! two different symbols in the same real-reg or spill-slot meet to "conflicted"; otherwise,
//! the symbol meets with itself to produce itself (reflexivity).
//!
//! To check correctness, we first find the dataflow fixpoint with the above lattice and
//! transfer/meet functions. Then, at each op, we examine the dataflow solution at the preceding
//! program point, and check that the real reg for each op arg (input/use) contains the symbol
//! corresponding to the original (usually virtual) register specified for this arg.

#![allow(dead_code)]

use crate::data_structures::{
    BlockIx, InstIx, Map, RealReg, RealRegUniverse, Reg, RegSets, SpillSlot, VirtualReg, Writable,
};
use crate::inst_stream::{ExtPoint, InstExtPoint, InstToInsertAndExtPoint};
use crate::{analysis_data_flow::get_san_reg_sets_for_insn, StackmapRequestInfo};
use crate::{Function, RegUsageMapper};

use rustc_hash::FxHashSet;
use std::collections::VecDeque;
use std::default::Default;
use std::hash::Hash;
use std::result::Result;

use log::debug;

/// A set of errors detected by the regalloc checker.
#[derive(Clone, Debug)]
pub struct CheckerErrors {
    errors: Vec<CheckerError>,
}

/// A single error detected by the regalloc checker.
#[derive(Clone, Debug)]
pub enum CheckerError {
    MissingAllocationForReg {
        reg: VirtualReg,
        inst: InstIx,
    },
    UnknownValueInReg {
        real_reg: RealReg,
        inst: InstIx,
    },
    IncorrectValueInReg {
        actual: Reg,
        expected: Reg,
        real_reg: RealReg,
        inst: InstIx,
    },
    UnknownValueInSlot {
        slot: SpillSlot,
        expected: Reg,
        inst: InstIx,
    },
    IncorrectValueInSlot {
        slot: SpillSlot,
        expected: Reg,
        actual: Reg,
        inst: InstIx,
    },
    StackMapSpecifiesNonRefSlot {
        inst: InstIx,
        slot: SpillSlot,
    },
    StackMapSpecifiesUndefinedSlot {
        inst: InstIx,
        slot: SpillSlot,
    },
}

/// Abstract state for a storage slot (real register or spill slot).
///
/// Forms a lattice with \top (`Unknown`), \bot (`Conflicted`), and a number of mutually unordered
/// value-points in between, one per real or virtual register. Any two different registers
/// meet to \bot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckerValue {
    /// "top" value: this storage slot has no known value.
    Unknown,
    /// "bottom" value: this storage slot has a conflicted value.
    Conflicted,
    /// Reg: this storage slot has a value that originated as a def into
    /// the given register, either implicitly (RealRegs at beginning of
    /// function) or explicitly (as an instruction's def).
    ///
    /// The boolean flag indicates whether the value is reference-typed.
    Reg(Reg, bool),
}

impl Default for CheckerValue {
    fn default() -> CheckerValue {
        CheckerValue::Unknown
    }
}

impl CheckerValue {
    /// Meet function of the abstract-interpretation value lattice.
    fn meet(&self, other: &CheckerValue) -> CheckerValue {
        match (self, other) {
            (&CheckerValue::Unknown, _) => *other,
            (_, &CheckerValue::Unknown) => *self,
            (&CheckerValue::Conflicted, _) => *self,
            (_, &CheckerValue::Conflicted) => *other,
            (&CheckerValue::Reg(r1, ref1), &CheckerValue::Reg(r2, ref2)) if r1 == r2 => {
                CheckerValue::Reg(r1, ref1 || ref2)
            }
            _ => CheckerValue::Conflicted,
        }
    }
}

/// State that steps through program points as we scan over the instruction stream.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckerState {
    /// For each RealReg, abstract state.
    reg_values: Map<RealReg, CheckerValue>,
    /// For each spill slot, abstract state.
    spill_slots: Map<SpillSlot, CheckerValue>,
}

impl Default for CheckerState {
    fn default() -> CheckerState {
        CheckerState {
            reg_values: Map::default(),
            spill_slots: Map::default(),
        }
    }
}

fn merge_map<K: Copy + Clone + PartialEq + Eq + Hash>(
    into: &mut Map<K, CheckerValue>,
    from: &Map<K, CheckerValue>,
) {
    for (k, v) in from {
        let into_v = into.entry(*k).or_insert(Default::default());
        let merged = into_v.meet(v);
        *into_v = merged;
    }
}

impl CheckerState {
    /// Create a new checker state.
    fn new() -> CheckerState {
        Default::default()
    }

    /// Produce an entry checker state with all real regs holding themselves, symbolically.
    fn entry_state(ru: &RealRegUniverse) -> CheckerState {
        let mut state = CheckerState::new();
        for &(rreg, _) in &ru.regs {
            state
                .reg_values
                .insert(rreg, CheckerValue::Reg(rreg.to_reg(), false));
        }
        state
    }

    /// Merge this checker state with another at a CFG join-point.
    fn meet_with(&mut self, other: &CheckerState) {
        merge_map(&mut self.reg_values, &other.reg_values);
        merge_map(&mut self.spill_slots, &other.spill_slots);
    }

    /// Check an instruction against this state.
    fn check(&self, inst: &Inst) -> Result<(), CheckerError> {
        match inst {
            &Inst::Op {
                inst_ix,
                ref uses_orig,
                ref uses,
                ..
            } => {
                // For each use, check the mapped RealReg's symbolic value; it must
                // be the original reg.
                assert!(uses_orig.len() == uses.len());
                for (orig, mapped) in uses_orig.iter().cloned().zip(uses.iter().cloned()) {
                    let val = self
                        .reg_values
                        .get(&mapped)
                        .cloned()
                        .unwrap_or(Default::default());
                    debug!(
                        "checker: inst {:?}: orig {:?}, mapped {:?}, checker state {:?}",
                        inst, orig, mapped, val
                    );
                    match val {
                        CheckerValue::Unknown | CheckerValue::Conflicted => {
                            return Err(CheckerError::UnknownValueInReg {
                                real_reg: mapped,
                                inst: inst_ix,
                            });
                        }
                        CheckerValue::Reg(r, _) if r != orig => {
                            return Err(CheckerError::IncorrectValueInReg {
                                actual: r,
                                expected: orig,
                                real_reg: mapped,
                                inst: inst_ix,
                            });
                        }
                        _ => {}
                    }
                }
            }
            &Inst::ChangeSpillSlotOwnership {
                inst_ix,
                slot,
                from_reg,
                ..
            } => {
                let val = self
                    .spill_slots
                    .get(&slot)
                    .cloned()
                    .unwrap_or(Default::default());
                debug!("checker: inst {:?}: slot value {:?}", inst, val);
                match val {
                    CheckerValue::Unknown | CheckerValue::Conflicted => {
                        return Err(CheckerError::UnknownValueInSlot {
                            slot,
                            expected: from_reg,
                            inst: inst_ix,
                        });
                    }
                    CheckerValue::Reg(r, _) if r != from_reg => {
                        return Err(CheckerError::IncorrectValueInSlot {
                            slot,
                            expected: from_reg,
                            actual: r,
                            inst: inst_ix,
                        });
                    }
                    _ => {}
                }
            }
            &Inst::Safepoint { inst_ix, ref slots } => {
                self.check_stackmap(inst_ix, slots)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn check_stackmap(&self, inst: InstIx, slots: &Vec<SpillSlot>) -> Result<(), CheckerError> {
        // N.B.: it's OK for the stackmap to omit a slot that has a ref value in
        // it; it might be dead. We simply update such a slot's value to
        // 'undefined' in the transfer function.
        for &slot in slots {
            match self.spill_slots.get(&slot) {
                Some(CheckerValue::Reg(_, false)) => {
                    return Err(CheckerError::StackMapSpecifiesNonRefSlot { inst, slot });
                }
                Some(CheckerValue::Reg(_, true)) => {
                    // OK.
                }
                _ => {
                    return Err(CheckerError::StackMapSpecifiesUndefinedSlot { inst, slot });
                }
            }
        }
        Ok(())
    }

    fn update_stackmap(&mut self, slots: &Vec<SpillSlot>) {
        for (&slot, val) in &mut self.spill_slots {
            if let &mut CheckerValue::Reg(_, true) = val {
                let in_stackmap = slots.binary_search(&slot).is_ok();
                if !in_stackmap {
                    *val = CheckerValue::Unknown;
                }
            }
        }
        // Mark registers holding reference values as unknown, forcing the need for a reload around
        // the safepoint.
        for (_, val) in &mut self.reg_values {
            if let &mut CheckerValue::Reg(_, true) = val {
                *val = CheckerValue::Unknown;
            }
        }
    }

    /// Update according to instruction.
    fn update(&mut self, inst: &Inst) {
        match inst {
            &Inst::Op {
                ref defs_orig,
                ref defs,
                ref defs_reftyped,
                ..
            } => {
                // For each def, set the symbolic value of the mapped RealReg to a
                // symbol corresponding to the original def.
                assert!(defs_orig.len() == defs.len());
                for i in 0..defs.len() {
                    let orig = defs_orig[i];
                    let mapped = defs[i];
                    let reftyped = defs_reftyped[i];
                    self.reg_values
                        .insert(mapped, CheckerValue::Reg(orig, reftyped));
                }
            }
            &Inst::Move { into, from } => {
                let val = self
                    .reg_values
                    .get(&from)
                    .cloned()
                    .unwrap_or(Default::default());
                self.reg_values.insert(into.to_reg(), val);
            }
            &Inst::ChangeSpillSlotOwnership { slot, to_reg, .. } => {
                let reftyped = if let Some(val) = self.spill_slots.get(&slot) {
                    match val {
                        &CheckerValue::Reg(_, reftyped) => reftyped,
                        _ => false,
                    }
                } else {
                    false
                };
                self.spill_slots
                    .insert(slot, CheckerValue::Reg(to_reg, reftyped));
            }
            &Inst::Spill { into, from } => {
                let val = self
                    .reg_values
                    .get(&from)
                    .cloned()
                    .unwrap_or(Default::default());
                self.spill_slots.insert(into, val);
            }
            &Inst::Reload { into, from } => {
                let val = self
                    .spill_slots
                    .get(&from)
                    .cloned()
                    .unwrap_or(Default::default());
                self.reg_values.insert(into.to_reg(), val);
            }
            &Inst::Safepoint { ref slots, .. } => {
                self.update_stackmap(slots);
            }
        }
    }
}

/// An instruction representation in the checker's BB summary.
#[derive(Clone, Debug)]
pub(crate) enum Inst {
    /// A register spill into memory.
    Spill { into: SpillSlot, from: RealReg },
    /// A register reload from memory.
    Reload {
        into: Writable<RealReg>,
        from: SpillSlot,
    },
    /// A regalloc-inserted move (not a move in the original program!)
    Move {
        into: Writable<RealReg>,
        from: RealReg,
    },
    /// A spillslot ghost move (between vregs) resulting from an user-program
    /// move whose source and destination regs are both vregs that are currently
    /// spilled.
    ChangeSpillSlotOwnership {
        inst_ix: InstIx,
        slot: SpillSlot,
        from_reg: Reg,
        to_reg: Reg,
    },
    /// A regular instruction with fixed use and def slots. Contains both
    /// the original registers (as given to the regalloc) and the allocated ones.
    Op {
        inst_ix: InstIx,
        defs_orig: Vec<Reg>,
        uses_orig: Vec<Reg>,
        defs: Vec<RealReg>,
        uses: Vec<RealReg>,
        defs_reftyped: Vec<bool>,
    },
    /// A safepoint, with a list of expected slots.
    Safepoint {
        inst_ix: InstIx,
        slots: Vec<SpillSlot>,
    },
}

#[derive(Debug)]
pub(crate) struct Checker {
    bb_entry: BlockIx,
    bb_in: Map<BlockIx, CheckerState>,
    bb_succs: Map<BlockIx, Vec<BlockIx>>,
    bb_insts: Map<BlockIx, Vec<Inst>>,
    reftyped_vregs: FxHashSet<VirtualReg>,
    has_run: bool,
}

fn map_regs<F: Fn(VirtualReg) -> Option<RealReg>>(
    inst: InstIx,
    regs: &[Reg],
    f: &F,
) -> Result<Vec<RealReg>, CheckerErrors> {
    let mut errors = Vec::new();
    let real_regs = regs
        .iter()
        .map(|r| {
            if r.is_virtual() {
                f(r.to_virtual_reg()).unwrap_or_else(|| {
                    errors.push(CheckerError::MissingAllocationForReg {
                        reg: r.to_virtual_reg(),
                        inst,
                    });
                    // Provide a dummy value for the register, it'll never be read.
                    Reg::new_real(r.get_class(), 0x0, 0).to_real_reg()
                })
            } else {
                r.to_real_reg()
            }
        })
        .collect();
    if errors.is_empty() {
        Ok(real_regs)
    } else {
        Err(CheckerErrors { errors })
    }
}

impl Checker {
    /// Create a new checker for the given function, initializing CFG info immediately.
    /// The client should call the `add_*()` methods to add abstract instructions to each
    /// BB before invoking `run()` to check for errors.
    pub(crate) fn new<F: Function>(
        f: &F,
        ru: &RealRegUniverse,
        reftyped_vregs: &[VirtualReg],
    ) -> Checker {
        let mut bb_in = Map::default();
        let mut bb_succs = Map::default();
        let mut bb_insts = Map::default();

        for block in f.blocks() {
            bb_in.insert(block, Default::default());
            bb_succs.insert(block, f.block_succs(block).to_vec());
            bb_insts.insert(block, vec![]);
        }

        bb_in.insert(f.entry_block(), CheckerState::entry_state(ru));

        let reftyped_vregs = reftyped_vregs.iter().cloned().collect::<FxHashSet<_>>();
        Checker {
            bb_entry: f.entry_block(),
            bb_in,
            bb_succs,
            bb_insts,
            reftyped_vregs,
            has_run: false,
        }
    }

    /// Add an abstract instruction (spill, reload, or move) to a BB.
    ///
    /// Can also accept an `Inst::Op`, but `add_op()` is better-suited
    /// for this.
    pub(crate) fn add_inst(&mut self, block: BlockIx, inst: Inst) {
        let insts = self.bb_insts.get_mut(&block).unwrap();
        insts.push(inst);
    }

    /// Add a "normal" instruction that uses, modifies, and/or defines certain
    /// registers. The `SanitizedInstRegUses` must be the pre-allocation state;
    /// the `mapper` must be provided to give the virtual -> real mappings at
    /// the program points immediately before and after this instruction.
    pub(crate) fn add_op<RUM: RegUsageMapper>(
        &mut self,
        block: BlockIx,
        inst_ix: InstIx,
        regsets: &RegSets,
        mapper: &RUM,
    ) -> Result<(), CheckerErrors> {
        debug!(
            "add_op: block {} inst {} regsets {:?}",
            block.get(),
            inst_ix.get(),
            regsets
        );
        assert!(regsets.is_sanitized());
        let mut uses_set = regsets.uses.clone();
        let mut defs_set = regsets.defs.clone();
        uses_set.union(&regsets.mods);
        defs_set.union(&regsets.mods);
        if uses_set.is_empty() && defs_set.is_empty() {
            return Ok(());
        }

        let uses_orig = uses_set.to_vec();
        let defs_orig = defs_set.to_vec();
        let uses = map_regs(inst_ix, &uses_orig[..], &|vreg| mapper.get_use(vreg))?;
        let defs = map_regs(inst_ix, &defs_orig[..], &|vreg| mapper.get_def(vreg))?;
        let defs_reftyped = defs_orig
            .iter()
            .map(|reg| reg.is_virtual() && self.reftyped_vregs.contains(&reg.to_virtual_reg()))
            .collect();
        let insts = self.bb_insts.get_mut(&block).unwrap();
        let op = Inst::Op {
            inst_ix,
            uses_orig,
            defs_orig,
            uses,
            defs,
            defs_reftyped,
        };
        debug!("add_op: adding {:?}", op);
        insts.push(op);
        Ok(())
    }

    /// Perform the dataflow analysis to compute checker state at each BB entry.
    fn analyze(&mut self) {
        let mut queue = VecDeque::new();
        queue.push_back(self.bb_entry);

        while !queue.is_empty() {
            let block = queue.pop_front().unwrap();
            let mut state = self.bb_in.get(&block).cloned().unwrap();
            debug!("analyze: block {} has state {:?}", block.get(), state);
            for inst in self.bb_insts.get(&block).unwrap() {
                state.update(inst);
                debug!("analyze: inst {:?} -> state {:?}", inst, state);
            }

            for succ in self.bb_succs.get(&block).unwrap() {
                let cur_succ_in = self.bb_in.get(succ).unwrap();
                let mut new_state = state.clone();
                new_state.meet_with(cur_succ_in);
                let changed = &new_state != cur_succ_in;
                if changed {
                    debug!(
                        "analyze: block {} state changed from {:?} to {:?}; pushing onto queue",
                        succ.get(),
                        cur_succ_in,
                        new_state
                    );
                    self.bb_in.insert(*succ, new_state);
                    queue.push_back(*succ);
                }
            }
        }
    }

    /// Using BB-start state computed by `analyze()`, step the checker state
    /// through each BB and check each instruction's register allocations
    /// for errors.
    fn find_errors(&self) -> Result<(), CheckerErrors> {
        let mut errors = vec![];
        for (block, input) in &self.bb_in {
            let mut state = input.clone();
            for inst in self.bb_insts.get(block).unwrap() {
                if let Err(e) = state.check(inst) {
                    debug!("Checker error: {:?}", e);
                    errors.push(e);
                }
                state.update(inst);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(CheckerErrors { errors })
        }
    }

    /// Find any errors, returning `Err(CheckerErrors)` with all errors found
    /// or `Ok(())` otherwise.
    pub(crate) fn run(mut self) -> Result<(), CheckerErrors> {
        debug!("Checker: full body is:\n{:?}", self.bb_insts);
        self.has_run = true;
        self.analyze();
        self.find_errors()
    }
}

/// A wrapper around `Checker` that assists its use with `InstToInsertAndExtPoint`s and
/// `Function` together.
pub(crate) struct CheckerContext {
    checker: Checker,
    checker_inst_map: Map<InstExtPoint, Vec<Inst>>,
}

/// Full infromation needed by the checker for checking reference types.
pub(crate) struct CheckerStackmapInfo<'a> {
    pub(crate) request: &'a StackmapRequestInfo,
    pub(crate) stackmaps: &'a [Vec<SpillSlot>],
}

impl CheckerContext {
    /// Create a new checker context for the given function, which is about to be edited with the
    /// given instruction insertions.
    pub(crate) fn new<F: Function>(
        f: &F,
        ru: &RealRegUniverse,
        insts_to_add: &Vec<InstToInsertAndExtPoint>,
        stackmap_info: Option<CheckerStackmapInfo>,
    ) -> CheckerContext {
        let mut checker_inst_map: Map<InstExtPoint, Vec<Inst>> = Map::default();
        for &InstToInsertAndExtPoint { ref inst, ref iep } in insts_to_add {
            let checker_insts = checker_inst_map
                .entry(iep.clone())
                .or_insert_with(|| vec![]);
            checker_insts.push(inst.to_checker_inst());
        }

        let reftyped_vregs = if let Some(info) = stackmap_info {
            assert!(info.request.safepoint_insns.len() == info.stackmaps.len());
            for (iix, slots) in info
                .request
                .safepoint_insns
                .iter()
                .zip(info.stackmaps.iter())
            {
                let iep = InstExtPoint::new(*iix, ExtPoint::Use);
                let mut slots = slots.clone();
                slots.sort();
                checker_inst_map
                    .entry(iep)
                    .or_insert_with(|| vec![])
                    .push(Inst::Safepoint {
                        inst_ix: *iix,
                        slots,
                    });
            }
            info.request.reftyped_vregs.as_slice()
        } else {
            &[]
        };

        let checker = Checker::new(f, ru, reftyped_vregs);
        CheckerContext {
            checker,
            checker_inst_map,
        }
    }

    /// Update the checker with the given instruction and the given pre- and post-maps. Instructions
    /// within a block must be visited in program order.
    pub(crate) fn handle_insn<F: Function, RUM: RegUsageMapper>(
        &mut self,
        ru: &RealRegUniverse,
        func: &F,
        bix: BlockIx,
        iix: InstIx,
        mapper: &RUM,
    ) -> Result<(), CheckerErrors> {
        let empty = vec![];
        let mut skip_inst = false;

        debug!("CheckerContext::handle_insn: inst {:?}", iix,);

        for &pre_point in &[ExtPoint::Reload, ExtPoint::SpillBefore, ExtPoint::Use] {
            let pre_point = InstExtPoint::new(iix, pre_point);
            for checker_inst in self.checker_inst_map.get(&pre_point).unwrap_or(&empty) {
                debug!("at inst {:?}: pre checker_inst: {:?}", iix, checker_inst);
                self.checker.add_inst(bix, checker_inst.clone());
                if let Inst::ChangeSpillSlotOwnership { .. } = checker_inst {
                    // Unlike spills/reloads/moves inserted by the regalloc, ChangeSpillSlotOwnership
                    // pseudo-insts replace the instruction itself.
                    skip_inst = true;
                }
            }
        }

        if !skip_inst {
            let regsets = get_san_reg_sets_for_insn::<F>(func.get_insn(iix), ru)
                .expect("only existing real registers at this point");
            assert!(regsets.is_sanitized());

            debug!(
                "at inst {:?}: regsets {:?} mapper {:?}",
                iix, regsets, mapper
            );
            self.checker.add_op(bix, iix, &regsets, mapper)?;
        }

        for &post_point in &[ExtPoint::ReloadAfter, ExtPoint::Spill] {
            let post_point = InstExtPoint::new(iix, post_point);
            for checker_inst in self.checker_inst_map.get(&post_point).unwrap_or(&empty) {
                debug!("at inst {:?}: post checker_inst: {:?}", iix, checker_inst);
                self.checker.add_inst(bix, checker_inst.clone());
            }
        }

        Ok(())
    }

    /// Run the underlying checker, once all instructions have been added.
    pub(crate) fn run(self) -> Result<(), CheckerErrors> {
        self.checker.run()
    }
}

impl Drop for Checker {
    fn drop(&mut self) {
        if !self.has_run {
            panic!("Programmer error: the CheckerContext run() function hasn't been called");
        }
    }
}
