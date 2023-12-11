//! Debug info analysis: computes value-label ranges from value-label markers in
//! generated VCode.
//!
//! We "reverse-engineer" debug info like this because it is far more reliable
//! than generating it while emitting code and keeping it in sync.
//!
//! This works by (i) observing "value-label marker" instructions, which are
//! semantically just an assignment from a register to a "value label" (which
//! one can think of as another register; they represent, e.g., Wasm locals) at
//! a certain point in the code, and (ii) observing loads and stores to the
//! stack and register moves.
//!
//! We track, at every program point, the correspondence between each value
//! label and *all* locations in which it resides. E.g., if it is stored to the
//! stack, we remember that it is in both a register and the stack slot; but if
//! the register is later overwritten, then we have it just in the stack slot.
//! This allows us to avoid false-positives observing loads/stores that we think
//! are spillslots but really aren't.
//!
//! We do a standard forward dataflow analysis to compute this info.

use crate::ir::ValueLabel;
use crate::machinst::*;
use crate::value_label::{LabelValueLoc, ValueLabelsRanges, ValueLocRange};
use log::trace;
use regalloc::{Reg, RegUsageCollector};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Location of a labeled value: in a register or in a stack slot. Note that a
/// value may live in more than one location; `AnalysisInfo` maps each
/// value-label to multiple `ValueLoc`s.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ValueLoc {
    Reg(Reg),
    /// Nominal-SP offset.
    Stack(i64),
}

impl From<ValueLoc> for LabelValueLoc {
    fn from(v: ValueLoc) -> Self {
        match v {
            ValueLoc::Reg(r) => LabelValueLoc::Reg(r),
            ValueLoc::Stack(off) => LabelValueLoc::SPOffset(off),
        }
    }
}

impl ValueLoc {
    fn is_reg(self) -> bool {
        match self {
            ValueLoc::Reg(_) => true,
            _ => false,
        }
    }
    fn is_stack(self) -> bool {
        match self {
            ValueLoc::Stack(_) => true,
            _ => false,
        }
    }
}

/// Mappings at one program point.
#[derive(Clone, Debug)]
struct AnalysisInfo {
    /// Nominal SP relative to real SP. If `None`, then the offset is
    /// indeterminate (i.e., we merged to the lattice 'bottom' element). This
    /// should not happen in well-formed code.
    nominal_sp_offset: Option<i64>,
    /// Forward map from labeled values to sets of locations.
    label_to_locs: HashMap<ValueLabel, HashSet<ValueLoc>>,
    /// Reverse map for each register indicating the value it holds, if any.
    reg_to_label: HashMap<Reg, ValueLabel>,
    /// Reverse map for each stack offset indicating the value it holds, if any.
    stack_to_label: HashMap<i64, ValueLabel>,
}

/// Get the registers written (mod'd or def'd) by a machine instruction.
fn get_inst_writes<M: MachInst>(m: &M) -> Vec<Reg> {
    // TODO: expose this part of regalloc.rs's interface publicly.
    let mut vecs = RegUsageCollector::get_empty_reg_vecs_test_framework_only(false);
    let mut coll = RegUsageCollector::new(&mut vecs);
    m.get_regs(&mut coll);
    vecs.defs.extend(vecs.mods.into_iter());
    vecs.defs
}

impl AnalysisInfo {
    /// Create a new analysis state. This is the "top" lattice element at which
    /// the fixpoint dataflow analysis starts.
    fn new() -> Self {
        AnalysisInfo {
            nominal_sp_offset: Some(0),
            label_to_locs: HashMap::new(),
            reg_to_label: HashMap::new(),
            stack_to_label: HashMap::new(),
        }
    }

    /// Remove all locations for a given labeled value. Used when the labeled
    /// value is redefined (so old values become stale).
    fn clear_label(&mut self, label: ValueLabel) {
        if let Some(locs) = self.label_to_locs.remove(&label) {
            for loc in locs {
                match loc {
                    ValueLoc::Reg(r) => {
                        self.reg_to_label.remove(&r);
                    }
                    ValueLoc::Stack(off) => {
                        self.stack_to_label.remove(&off);
                    }
                }
            }
        }
    }

    /// Remove a label from a register, if any. Used, e.g., if the register is
    /// overwritten.
    fn clear_reg(&mut self, reg: Reg) {
        if let Some(label) = self.reg_to_label.remove(&reg) {
            if let Some(locs) = self.label_to_locs.get_mut(&label) {
                locs.remove(&ValueLoc::Reg(reg));
            }
        }
    }

    /// Remove a label from a stack offset, if any. Used, e.g., when the stack
    /// slot is overwritten.
    fn clear_stack_off(&mut self, off: i64) {
        if let Some(label) = self.stack_to_label.remove(&off) {
            if let Some(locs) = self.label_to_locs.get_mut(&label) {
                locs.remove(&ValueLoc::Stack(off));
            }
        }
    }

    /// Indicate that a labeled value is newly defined and its new value is in
    /// `reg`.
    fn def_label_at_reg(&mut self, label: ValueLabel, reg: Reg) {
        self.clear_label(label);
        self.label_to_locs
            .entry(label)
            .or_insert_with(|| HashSet::new())
            .insert(ValueLoc::Reg(reg));
        self.reg_to_label.insert(reg, label);
    }

    /// Process a store from a register to a stack slot (offset).
    fn store_reg(&mut self, reg: Reg, off: i64) {
        self.clear_stack_off(off);
        if let Some(label) = self.reg_to_label.get(&reg) {
            if let Some(locs) = self.label_to_locs.get_mut(label) {
                locs.insert(ValueLoc::Stack(off));
            }
            self.stack_to_label.insert(off, *label);
        }
    }

    /// Process a load from a stack slot (offset) to a register.
    fn load_reg(&mut self, reg: Reg, off: i64) {
        self.clear_reg(reg);
        if let Some(&label) = self.stack_to_label.get(&off) {
            if let Some(locs) = self.label_to_locs.get_mut(&label) {
                locs.insert(ValueLoc::Reg(reg));
            }
            self.reg_to_label.insert(reg, label);
        }
    }

    /// Process a move from one register to another.
    fn move_reg(&mut self, to: Reg, from: Reg) {
        self.clear_reg(to);
        if let Some(&label) = self.reg_to_label.get(&from) {
            if let Some(locs) = self.label_to_locs.get_mut(&label) {
                locs.insert(ValueLoc::Reg(to));
            }
            self.reg_to_label.insert(to, label);
        }
    }

    /// Update the analysis state w.r.t. an instruction's effects. Given the
    /// state just before `inst`, this method updates `self` to be the state
    /// just after `inst`.
    fn step<M: MachInst>(&mut self, inst: &M) {
        for write in get_inst_writes(inst) {
            self.clear_reg(write);
        }
        if let Some((label, reg)) = inst.defines_value_label() {
            self.def_label_at_reg(label, reg);
        }
        match inst.stack_op_info() {
            Some(MachInstStackOpInfo::LoadNomSPOff(reg, offset)) => {
                self.load_reg(reg, offset + self.nominal_sp_offset.unwrap());
            }
            Some(MachInstStackOpInfo::StoreNomSPOff(reg, offset)) => {
                self.store_reg(reg, offset + self.nominal_sp_offset.unwrap());
            }
            Some(MachInstStackOpInfo::NomSPAdj(offset)) => {
                if self.nominal_sp_offset.is_some() {
                    self.nominal_sp_offset = Some(self.nominal_sp_offset.unwrap() + offset);
                }
            }
            _ => {}
        }
        if let Some((to, from)) = inst.is_move() {
            let to = to.to_reg();
            self.move_reg(to, from);
        }
    }
}

/// Trait used to implement the dataflow analysis' meet (intersect) function
/// onthe `AnalysisInfo` components. For efficiency, this is implemented as a
/// mutation on the LHS, rather than a pure functional operation.
trait IntersectFrom {
    fn intersect_from(&mut self, other: &Self) -> IntersectResult;
}

/// Result of an intersection operation. Indicates whether the mutated LHS
/// (which becomes the intersection result) differs from the original LHS. Also
/// indicates if the value has become "empty" and should be removed from a
/// parent container, if any.
struct IntersectResult {
    /// Did the intersection change the LHS input (the one that was mutated into
    /// the result)? This is needed to drive the fixpoint loop; when no more
    /// changes occur, then we have converted.
    changed: bool,
    /// Is the resulting value "empty"? This can be used when a container, such
    /// as a map, holds values of this (intersection result) type; when
    /// `is_empty` is true for the merge of the values at a particular key, we
    /// can remove that key from the merged (intersected) result. This is not
    /// necessary for analysis correctness but reduces the memory and runtime
    /// cost of the fixpoint loop.
    is_empty: bool,
}

impl IntersectFrom for AnalysisInfo {
    fn intersect_from(&mut self, other: &Self) -> IntersectResult {
        let mut changed = false;
        changed |= self
            .nominal_sp_offset
            .intersect_from(&other.nominal_sp_offset)
            .changed;
        changed |= self
            .label_to_locs
            .intersect_from(&other.label_to_locs)
            .changed;
        changed |= self
            .reg_to_label
            .intersect_from(&other.reg_to_label)
            .changed;
        changed |= self
            .stack_to_label
            .intersect_from(&other.stack_to_label)
            .changed;
        IntersectResult {
            changed,
            is_empty: false,
        }
    }
}

impl<K, V> IntersectFrom for HashMap<K, V>
where
    K: Copy + Eq + Hash,
    V: IntersectFrom,
{
    /// Intersection for hashmap: remove keys that are not in both inputs;
    /// recursively intersect values for keys in common.
    fn intersect_from(&mut self, other: &Self) -> IntersectResult {
        let mut changed = false;
        let mut remove_keys = vec![];
        for k in self.keys() {
            if !other.contains_key(k) {
                remove_keys.push(*k);
            }
        }
        for k in &remove_keys {
            changed = true;
            self.remove(k);
        }

        remove_keys.clear();
        for k in other.keys() {
            if let Some(v) = self.get_mut(k) {
                let result = v.intersect_from(other.get(k).unwrap());
                changed |= result.changed;
                if result.is_empty {
                    remove_keys.push(*k);
                }
            }
        }
        for k in &remove_keys {
            changed = true;
            self.remove(k);
        }

        IntersectResult {
            changed,
            is_empty: self.len() == 0,
        }
    }
}
impl<T> IntersectFrom for HashSet<T>
where
    T: Copy + Eq + Hash,
{
    /// Intersection for hashset: just take the set intersection.
    fn intersect_from(&mut self, other: &Self) -> IntersectResult {
        let mut changed = false;
        let mut remove = vec![];
        for val in self.iter() {
            if !other.contains(val) {
                remove.push(*val);
            }
        }
        for val in remove {
            changed = true;
            self.remove(&val);
        }

        IntersectResult {
            changed,
            is_empty: self.len() == 0,
        }
    }
}
impl IntersectFrom for ValueLabel {
    // Intersection for labeled value: remove if not equal. This is equivalent
    // to a three-level lattice with top, bottom, and unordered set of
    // individual labels in between.
    fn intersect_from(&mut self, other: &Self) -> IntersectResult {
        IntersectResult {
            changed: false,
            is_empty: *self != *other,
        }
    }
}
impl<T> IntersectFrom for Option<T>
where
    T: Copy + Eq,
{
    /// Intersectino for Option<T>: recursively intersect if both `Some`, else
    /// `None`.
    fn intersect_from(&mut self, other: &Self) -> IntersectResult {
        let mut changed = false;
        if !(self.is_some() && other.is_some() && self == other) {
            changed = true;
            *self = None;
        }
        IntersectResult {
            changed,
            is_empty: self.is_none(),
        }
    }
}

/// Compute the value-label ranges (locations for program-point ranges for
/// labeled values) from a given `VCode` compilation result.
///
/// In order to compute this information, we perform a dataflow analysis on the
/// machine code. To do so, and translate the results into a form usable by the
/// debug-info consumers, we need to know two additional things:
///
/// - The machine-code layout (code offsets) of the instructions. DWARF is
///   encoded in terms of instruction *ends* (and we reason about value
///   locations at program points *after* instructions, to match this), so we
///   take an array `inst_ends`, giving us code offsets for each instruction's
///   end-point. (Note that this is one *past* the last byte; so a 4-byte
///   instruction at offset 0 has an end offset of 4.)
///
/// - The locations of the labels to which branches will jump. Branches can tell
///   us about their targets in terms of `MachLabel`s, but we don't know where
///   those `MachLabel`s will be placed in the linear array of instructions.  We
///   take the array `label_insn_index` to provide this info: for a label with
///   index `l`, `label_insn_index[l]` is the index of the instruction before
///   which that label is bound.
pub(crate) fn compute<I: VCodeInst>(
    insts: &[I],
    layout_info: &InstsLayoutInfo,
) -> ValueLabelsRanges {
    let inst_start = |idx: usize| {
        if idx == 0 {
            0
        } else {
            layout_info.inst_end_offsets[idx - 1]
        }
    };

    trace!("compute: insts =");
    for i in 0..insts.len() {
        trace!(
            " #{} end: {} -> {:?}",
            i,
            layout_info.inst_end_offsets[i],
            insts[i]
        );
    }
    trace!("label_insn_index: {:?}", layout_info.label_inst_indices);

    // Info at each block head, indexed by label.
    let mut block_starts: HashMap<u32, AnalysisInfo> = HashMap::new();

    // Initialize state at entry.
    block_starts.insert(0, AnalysisInfo::new());

    // Worklist: label indices for basic blocks.
    let mut worklist = Vec::new();
    let mut worklist_set = HashSet::new();
    worklist.push(0);
    worklist_set.insert(0);

    while !worklist.is_empty() {
        let block = worklist.pop().unwrap();
        worklist_set.remove(&block);

        let mut state = block_starts.get(&block).unwrap().clone();
        trace!("at block {} -> state: {:?}", block, state);
        // Iterate for each instruction in the block (we break at the first
        // terminator we see).
        let mut index = layout_info.label_inst_indices[block as usize];
        while index < insts.len() as u32 {
            state.step(&insts[index as usize]);
            trace!(" -> inst #{}: {:?}", index, insts[index as usize]);
            trace!("    --> state: {:?}", state);

            let term = insts[index as usize].is_term();
            if term.is_term() {
                for succ in term.get_succs() {
                    trace!("    SUCCESSOR block {}", succ.get());
                    if let Some(succ_state) = block_starts.get_mut(&succ.get()) {
                        trace!("       orig state: {:?}", succ_state);
                        if succ_state.intersect_from(&state).changed {
                            if worklist_set.insert(succ.get()) {
                                worklist.push(succ.get());
                            }
                            trace!("        (changed)");
                        }
                        trace!("       new state: {:?}", succ_state);
                    } else {
                        // First time seeing this block
                        block_starts.insert(succ.get(), state.clone());
                        worklist.push(succ.get());
                        worklist_set.insert(succ.get());
                    }
                }
                break;
            }

            index += 1;
        }
    }

    // Now iterate over blocks one last time, collecting
    // value-label locations.

    let mut value_labels_ranges: ValueLabelsRanges = HashMap::new();
    for block in 0..layout_info.label_inst_indices.len() {
        let start_index = layout_info.label_inst_indices[block];
        let end_index = if block == layout_info.label_inst_indices.len() - 1 {
            insts.len() as u32
        } else {
            layout_info.label_inst_indices[block + 1]
        };
        let block = block as u32;
        let mut state = block_starts.get(&block).unwrap().clone();
        for index in start_index..end_index {
            let offset = inst_start(index as usize);
            let end = layout_info.inst_end_offsets[index as usize];

            // Cold blocks cause instructions to occur out-of-order wrt
            // others. We rely on the monotonic mapping from instruction
            // index to offset in machine code for this analysis to work,
            // so we just skip debuginfo for cold blocks. This should be
            // generally fine, as cold blocks generally constitute
            // slowpaths for expansions of particular ops, rather than
            // user-written code.
            if layout_info.start_of_cold_code.is_some()
                && offset >= layout_info.start_of_cold_code.unwrap()
            {
                continue;
            }

            assert!(offset <= end);
            state.step(&insts[index as usize]);

            for (label, locs) in &state.label_to_locs {
                trace!("   inst {} has label {:?} -> locs {:?}", index, label, locs);
                // Find an appropriate loc: a register if possible, otherwise pick the first stack
                // loc.
                let reg = locs.iter().cloned().find(|l| l.is_reg());
                let loc = reg.or_else(|| locs.iter().cloned().find(|l| l.is_stack()));
                if let Some(loc) = loc {
                    let loc = LabelValueLoc::from(loc);
                    let list = value_labels_ranges.entry(*label).or_insert_with(|| vec![]);
                    // If the existing location list for this value-label is
                    // either empty, or has an end location that does not extend
                    // to the current offset, then we have to append a new
                    // entry. Otherwise, we can extend the current entry.
                    //
                    // Note that `end` is one past the end of the instruction;
                    // it appears that `end` is exclusive, so a mapping valid at
                    // offset 5 will have start = 5, end = 6.
                    if list
                        .last()
                        .map(|last| last.end <= offset || last.loc != loc)
                        .unwrap_or(true)
                    {
                        list.push(ValueLocRange {
                            loc,
                            start: end,
                            end: end + 1,
                        });
                    } else {
                        list.last_mut().unwrap().end = end + 1;
                    }
                }
            }
        }
    }

    trace!("ret: {:?}", value_labels_ranges);
    value_labels_ranges
}
