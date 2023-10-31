//! A Constant-Phi-Node removal pass.

use crate::dominator_tree::DominatorTree;
use crate::entity::EntityList;
use crate::fx::FxHashMap;
use crate::fx::FxHashSet;
use crate::ir::instructions::BranchInfo;
use crate::ir::Function;
use crate::ir::{Block, Inst, Value};
use crate::timing;

use smallvec::{smallvec, SmallVec};
use std::vec::Vec;

// A note on notation.  For the sake of clarity, this file uses the phrase
// "formal parameters" to mean the `Value`s listed in the block head, and
// "actual parameters" to mean the `Value`s passed in a branch or a jump:
//
// block4(v16: i32, v18: i32):    <-- formal parameters
//   ...
//   brnz v27, block7(v22, v24)   <-- actual parameters
//   jump block6

// This transformation pass (conceptually) partitions all values in the
// function into two groups:
//
// * Group A: values defined by block formal parameters, except for the entry block.
//
// * Group B: All other values: that is, values defined by instructions,
//   and the formals of the entry block.
//
// For each value in Group A, it attempts to establish whether it will have
// the value of exactly one member of Group B.  If so, the formal parameter is
// deleted, all corresponding actual parameters (in jumps/branches to the
// defining block) are deleted, and a rename is inserted.
//
// The entry block is special-cased because (1) we don't know what values flow
// to its formals and (2) in any case we can't change its formals.
//
// Work proceeds in three phases.
//
// * Phase 1: examine all instructions.  For each block, make up a useful
//   grab-bag of information, `BlockSummary`, that summarises the block's
//   formals and jump/branch instruction.  This is used by Phases 2 and 3.
//
// * Phase 2: for each value in Group A, try to find a single Group B value
//   that flows to it.  This is done using a classical iterative forward
//   dataflow analysis over a simple constant-propagation style lattice.  It
//   converges quickly in practice -- I have seen at most 4 iterations.  This
//   is relatively cheap because the iteration is done over the
//   `BlockSummary`s, and does not visit each instruction.  The resulting
//   fixed point is stored in a `SolverState`.
//
// * Phase 3: using the `SolverState` and `BlockSummary`, edit the function to
//   remove redundant formals and actuals, and to insert suitable renames.
//
// Note that the effectiveness of the analysis depends on on the fact that
// there are no copy instructions in Cranelift's IR.  If there were, the
// computation of `actual_absval` in Phase 2 would have to be extended to
// chase through such copies.
//
// For large functions, the analysis cost using the new AArch64 backend is about
// 0.6% of the non-optimising compile time, as measured by instruction counts.
// This transformation usually pays for itself several times over, though, by
// reducing the isel/regalloc cost downstream.  Gains of up to 7% have been
// seen for large functions.

// The `Value`s (Group B) that can flow to a formal parameter (Group A).
#[derive(Clone, Copy, Debug, PartialEq)]
enum AbstractValue {
    // Two or more values flow to this formal.
    Many,
    // Exactly one value, as stated, flows to this formal.  The `Value`s that
    // can appear here are exactly: `Value`s defined by `Inst`s, plus the
    // `Value`s defined by the formals of the entry block.  Note that this is
    // exactly the set of `Value`s that are *not* tracked in the solver below
    // (see `SolverState`).
    One(Value /*Group B*/),
    // No value flows to this formal.
    None,
}

impl AbstractValue {
    fn join(self, other: AbstractValue) -> AbstractValue {
        match (self, other) {
            // Joining with `None` has no effect
            (AbstractValue::None, p2) => p2,
            (p1, AbstractValue::None) => p1,
            // Joining with `Many` produces `Many`
            (AbstractValue::Many, _p2) => AbstractValue::Many,
            (_p1, AbstractValue::Many) => AbstractValue::Many,
            // The only interesting case
            (AbstractValue::One(v1), AbstractValue::One(v2)) => {
                if v1 == v2 {
                    AbstractValue::One(v1)
                } else {
                    AbstractValue::Many
                }
            }
        }
    }
    fn is_one(self) -> bool {
        if let AbstractValue::One(_) = self {
            true
        } else {
            false
        }
    }
}

// For some block, a useful bundle of info.  The `Block` itself is not stored
// here since it will be the key in the associated `FxHashMap` -- see
// `summaries` below.  For the `SmallVec` tuning params: most blocks have
// few parameters, hence `4`.  And almost all blocks have either one or two
// successors, hence `2`.
#[derive(Debug)]
struct BlockSummary {
    // Formal parameters for this `Block`
    formals: SmallVec<[Value; 4] /*Group A*/>,
    // For each `Inst` in this block that transfers to another block: the
    // `Inst` itself, the destination `Block`, and the actual parameters
    // passed.  We don't bother to include transfers that pass zero parameters
    // since that makes more work for the solver for no purpose.
    dests: SmallVec<[(Inst, Block, SmallVec<[Value; 4] /*both Groups A and B*/>); 2]>,
}
impl BlockSummary {
    fn new(formals: SmallVec<[Value; 4]>) -> Self {
        Self {
            formals,
            dests: smallvec![],
        }
    }
}

// Solver state.  This holds a AbstractValue for each formal parameter, except
// for those from the entry block.
struct SolverState {
    absvals: FxHashMap<Value /*Group A*/, AbstractValue>,
}
impl SolverState {
    fn new() -> Self {
        Self {
            absvals: FxHashMap::default(),
        }
    }
    fn get(&self, actual: Value) -> AbstractValue {
        match self.absvals.get(&actual) {
            Some(lp) => *lp,
            None => panic!("SolverState::get: formal param {:?} is untracked?!", actual),
        }
    }
    fn maybe_get(&self, actual: Value) -> Option<&AbstractValue> {
        self.absvals.get(&actual)
    }
    fn set(&mut self, actual: Value, lp: AbstractValue) {
        match self.absvals.insert(actual, lp) {
            Some(_old_lp) => {}
            None => panic!("SolverState::set: formal param {:?} is untracked?!", actual),
        }
    }
}

/// Detect phis in `func` that will only ever produce one value, using a
/// classic forward dataflow analysis.  Then remove them.
#[inline(never)]
pub fn do_remove_constant_phis(func: &mut Function, domtree: &mut DominatorTree) {
    let _tt = timing::remove_constant_phis();
    debug_assert!(domtree.is_valid());

    // Get the blocks, in reverse postorder
    let mut blocks_reverse_postorder = Vec::<Block>::new();
    for block in domtree.cfg_postorder() {
        blocks_reverse_postorder.push(*block);
    }
    blocks_reverse_postorder.reverse();

    // Phase 1 of 3: for each block, make a summary containing all relevant
    // info.  The solver will iterate over the summaries, rather than having
    // to inspect each instruction in each block.
    let mut summaries = FxHashMap::<Block, BlockSummary>::default();

    for b in &blocks_reverse_postorder {
        let formals = func.dfg.block_params(*b);
        let mut summary = BlockSummary::new(SmallVec::from(formals));

        for inst in func.layout.block_insts(*b) {
            let idetails = &func.dfg[inst];
            // Note that multi-dest transfers (i.e., branch tables) don't
            // carry parameters in our IR, so we only have to care about
            // `SingleDest` here.
            if let BranchInfo::SingleDest(dest, _) = idetails.analyze_branch(&func.dfg.value_lists)
            {
                let inst_var_args = func.dfg.inst_variable_args(inst);
                // Skip branches/jumps that carry no params.
                if inst_var_args.len() > 0 {
                    let mut actuals = SmallVec::<[Value; 4]>::new();
                    for arg in inst_var_args {
                        let arg = func.dfg.resolve_aliases(*arg);
                        actuals.push(arg);
                    }
                    summary.dests.push((inst, dest, actuals));
                }
            }
        }

        // Ensure the invariant that all blocks (except for the entry) appear
        // in the summary, *unless* they have neither formals nor any
        // param-carrying branches/jumps.
        if formals.len() > 0 || summary.dests.len() > 0 {
            summaries.insert(*b, summary);
        }
    }

    // Phase 2 of 3: iterate over the summaries in reverse postorder,
    // computing new `AbstractValue`s for each tracked `Value`.  The set of
    // tracked `Value`s is exactly Group A as described above.

    let entry_block = func
        .layout
        .entry_block()
        .expect("remove_constant_phis: entry block unknown");

    // Set up initial solver state
    let mut state = SolverState::new();

    for b in &blocks_reverse_postorder {
        // For each block, get the formals
        if *b == entry_block {
            continue;
        }
        let formals: &[Value] = func.dfg.block_params(*b);
        for formal in formals {
            let mb_old_absval = state.absvals.insert(*formal, AbstractValue::None);
            assert!(mb_old_absval.is_none());
        }
    }

    // Solve: repeatedly traverse the blocks in reverse postorder, until there
    // are no changes.
    let mut iter_no = 0;
    loop {
        iter_no += 1;
        let mut changed = false;

        for src in &blocks_reverse_postorder {
            let mb_src_summary = summaries.get(src);
            // The src block might have no summary.  This means it has no
            // branches/jumps that carry parameters *and* it doesn't take any
            // parameters itself.  Phase 1 ensures this.  So we can ignore it.
            if mb_src_summary.is_none() {
                continue;
            }
            let src_summary = mb_src_summary.unwrap();
            for (_inst, dst, src_actuals) in &src_summary.dests {
                assert!(*dst != entry_block);
                // By contrast, the dst block must have a summary.  Phase 1
                // will have only included an entry in `src_summary.dests` if
                // that branch/jump carried at least one parameter.  So the
                // dst block does take parameters, so it must have a summary.
                let dst_summary = summaries
                    .get(dst)
                    .expect("remove_constant_phis: dst block has no summary");
                let dst_formals = &dst_summary.formals;
                assert!(src_actuals.len() == dst_formals.len());
                for (formal, actual) in dst_formals.iter().zip(src_actuals.iter()) {
                    // Find the abstract value for `actual`.  If it is a block
                    // formal parameter then the most recent abstract value is
                    // to be found in the solver state.  If not, then it's a
                    // real value defining point (not a phi), in which case
                    // return it itself.
                    let actual_absval = match state.maybe_get(*actual) {
                        Some(pt) => *pt,
                        None => AbstractValue::One(*actual),
                    };

                    // And `join` the new value with the old.
                    let formal_absval_old = state.get(*formal);
                    let formal_absval_new = formal_absval_old.join(actual_absval);
                    if formal_absval_new != formal_absval_old {
                        changed = true;
                        state.set(*formal, formal_absval_new);
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }
    let mut n_consts = 0;
    for absval in state.absvals.values() {
        if absval.is_one() {
            n_consts += 1;
        }
    }

    // Phase 3 of 3: edit the function to remove constant formals, using the
    // summaries and the final solver state as a guide.

    // Make up a set of blocks that need editing.
    let mut need_editing = FxHashSet::<Block>::default();
    for (block, summary) in &summaries {
        if *block == entry_block {
            continue;
        }
        for formal in &summary.formals {
            let formal_absval = state.get(*formal);
            if formal_absval.is_one() {
                need_editing.insert(*block);
                break;
            }
        }
    }

    // Firstly, deal with the formals.  For each formal which is redundant,
    // remove it, and also add a reroute from it to the constant value which
    // it we know it to be.
    for b in &need_editing {
        let mut del_these = SmallVec::<[(Value, Value); 32]>::new();
        let formals: &[Value] = func.dfg.block_params(*b);
        for formal in formals {
            // The state must give an absval for `formal`.
            if let AbstractValue::One(replacement_val) = state.get(*formal) {
                del_these.push((*formal, replacement_val));
            }
        }
        // We can delete the formals in any order.  However,
        // `remove_block_param` works by sliding backwards all arguments to
        // the right of the it is asked to delete.  Hence when removing more
        // than one formal, it is significantly more efficient to ask it to
        // remove the rightmost formal first, and hence this `reverse`.
        del_these.reverse();
        for (redundant_formal, replacement_val) in del_these {
            func.dfg.remove_block_param(redundant_formal);
            func.dfg.change_to_alias(redundant_formal, replacement_val);
        }
    }

    // Secondly, visit all branch insns.  If the destination has had its
    // formals changed, change the actuals accordingly.  Don't scan all insns,
    // rather just visit those as listed in the summaries we prepared earlier.
    for (_src_block, summary) in &summaries {
        for (inst, dst_block, _src_actuals) in &summary.dests {
            if !need_editing.contains(dst_block) {
                continue;
            }

            let old_actuals = func.dfg[*inst].take_value_list().unwrap();
            let num_old_actuals = old_actuals.len(&func.dfg.value_lists);
            let num_fixed_actuals = func.dfg[*inst]
                .opcode()
                .constraints()
                .num_fixed_value_arguments();
            let dst_summary = summaries.get(&dst_block).unwrap();

            // Check that the numbers of arguments make sense.
            assert!(num_fixed_actuals <= num_old_actuals);
            assert!(num_fixed_actuals + dst_summary.formals.len() == num_old_actuals);

            // Create a new value list.
            let mut new_actuals = EntityList::<Value>::new();
            // Copy the fixed args to the new list
            for i in 0..num_fixed_actuals {
                let val = old_actuals.get(i, &func.dfg.value_lists).unwrap();
                new_actuals.push(val, &mut func.dfg.value_lists);
            }

            // Copy the variable args (the actual block params) to the new
            // list, filtering out redundant ones.
            for i in 0..dst_summary.formals.len() {
                let actual_i = old_actuals
                    .get(num_fixed_actuals + i, &func.dfg.value_lists)
                    .unwrap();
                let formal_i = dst_summary.formals[i];
                let is_redundant = state.get(formal_i).is_one();
                if !is_redundant {
                    new_actuals.push(actual_i, &mut func.dfg.value_lists);
                }
            }
            func.dfg[*inst].put_value_list(new_actuals);
        }
    }

    log::debug!(
        "do_remove_constant_phis: done, {} iters.   {} formals, of which {} const.",
        iter_no,
        state.absvals.len(),
        n_consts
    );
}
