//! A Loop Invariant Code Motion optimization pass

use crate::cursor::{Cursor, FuncCursor};
use crate::dominator_tree::DominatorTree;
use crate::entity::{EntityList, ListPool};
use crate::flowgraph::{BlockPredecessor, ControlFlowGraph};
use crate::fx::FxHashSet;
use crate::ir::{
    Block, DataFlowGraph, Function, Inst, InstBuilder, InstructionData, Layout, Opcode, Type, Value,
};
use crate::loop_analysis::{Loop, LoopAnalysis};
use crate::timing;
use alloc::vec::Vec;

/// Performs the LICM pass by detecting loops within the CFG and moving
/// loop-invariant instructions out of them.
/// Changes the CFG and domtree in-place during the operation.
pub fn do_licm(
    func: &mut Function,
    cfg: &mut ControlFlowGraph,
    domtree: &mut DominatorTree,
    loop_analysis: &mut LoopAnalysis,
) {
    let _tt = timing::licm();
    debug_assert!(cfg.is_valid());
    debug_assert!(domtree.is_valid());
    debug_assert!(loop_analysis.is_valid());

    for lp in loop_analysis.loops() {
        // For each loop that we want to optimize we determine the set of loop-invariant
        // instructions
        let invariant_insts = remove_loop_invariant_instructions(lp, func, cfg, loop_analysis);
        // Then we create the loop's pre-header and fill it with the invariant instructions
        // Then we remove the invariant instructions from the loop body
        if !invariant_insts.is_empty() {
            // If the loop has a natural pre-header we use it, otherwise we create it.
            let mut pos;
            match has_pre_header(&func.layout, cfg, domtree, loop_analysis.loop_header(lp)) {
                None => {
                    let pre_header =
                        create_pre_header(loop_analysis.loop_header(lp), func, cfg, domtree);
                    pos = FuncCursor::new(func).at_last_inst(pre_header);
                }
                // If there is a natural pre-header we insert new instructions just before the
                // related jumping instruction (which is not necessarily at the end).
                Some((_, last_inst)) => {
                    pos = FuncCursor::new(func).at_inst(last_inst);
                }
            };
            // The last instruction of the pre-header is the termination instruction (usually
            // a jump) so we need to insert just before this.
            for inst in invariant_insts {
                pos.insert_inst(inst);
            }
        }
    }
    // We have to recompute the domtree to account for the changes
    cfg.compute(func);
    domtree.compute(func, cfg);
}

/// Insert a pre-header before the header, modifying the function layout and CFG to reflect it.
/// A jump instruction to the header is placed at the end of the pre-header.
fn create_pre_header(
    header: Block,
    func: &mut Function,
    cfg: &mut ControlFlowGraph,
    domtree: &DominatorTree,
) -> Block {
    let pool = &mut ListPool::<Value>::new();
    let header_args_values = func.dfg.block_params(header).to_vec();
    let header_args_types: Vec<Type> = header_args_values
        .into_iter()
        .map(|val| func.dfg.value_type(val))
        .collect();
    let pre_header = func.dfg.make_block();
    let mut pre_header_args_value: EntityList<Value> = EntityList::new();
    for typ in header_args_types {
        pre_header_args_value.push(func.dfg.append_block_param(pre_header, typ), pool);
    }

    for BlockPredecessor {
        inst: last_inst, ..
    } in cfg.pred_iter(header)
    {
        // We only follow normal edges (not the back edges)
        if !domtree.dominates(header, last_inst, &func.layout) {
            func.rewrite_branch_destination(last_inst, header, pre_header);
        }
    }

    // Inserts the pre-header at the right place in the layout.
    let mut pos = FuncCursor::new(func).at_top(header);
    pos.insert_block(pre_header);
    pos.next_inst();
    pos.ins().jump(header, pre_header_args_value.as_slice(pool));

    pre_header
}

/// Detects if a loop header has a natural pre-header.
///
/// A loop header has a pre-header if there is only one predecessor that the header doesn't
/// dominate.
/// Returns the pre-header Block and the instruction jumping to the header.
fn has_pre_header(
    layout: &Layout,
    cfg: &ControlFlowGraph,
    domtree: &DominatorTree,
    header: Block,
) -> Option<(Block, Inst)> {
    let mut result = None;
    for BlockPredecessor {
        block: pred_block,
        inst: branch_inst,
    } in cfg.pred_iter(header)
    {
        // We only count normal edges (not the back edges)
        if !domtree.dominates(header, branch_inst, layout) {
            if result.is_some() {
                // We have already found one, there are more than one
                return None;
            }
            if branch_inst != layout.last_inst(pred_block).unwrap()
                || cfg.succ_iter(pred_block).nth(1).is_some()
            {
                // It's along a critical edge, so don't use it.
                return None;
            }
            result = Some((pred_block, branch_inst));
        }
    }
    result
}

/// Test whether the given opcode is unsafe to even consider for LICM.
fn trivially_unsafe_for_licm(opcode: Opcode) -> bool {
    opcode.can_store()
        || opcode.is_call()
        || opcode.is_branch()
        || opcode.is_terminator()
        || opcode.is_return()
        || opcode.can_trap()
        || opcode.other_side_effects()
        || opcode.writes_cpu_flags()
}

fn is_unsafe_load(inst_data: &InstructionData) -> bool {
    match *inst_data {
        InstructionData::Load { flags, .. } | InstructionData::LoadComplex { flags, .. } => {
            !flags.readonly() || !flags.notrap()
        }
        _ => inst_data.opcode().can_load(),
    }
}

/// Test whether the given instruction is loop-invariant.
fn is_loop_invariant(inst: Inst, dfg: &DataFlowGraph, loop_values: &FxHashSet<Value>) -> bool {
    if trivially_unsafe_for_licm(dfg[inst].opcode()) {
        return false;
    }

    if is_unsafe_load(&dfg[inst]) {
        return false;
    }

    let inst_args = dfg.inst_args(inst);
    for arg in inst_args {
        let arg = dfg.resolve_aliases(*arg);
        if loop_values.contains(&arg) {
            return false;
        }
    }
    true
}

/// Traverses a loop in reverse post-order from a header block and identify loop-invariant
/// instructions. These loop-invariant instructions are then removed from the code and returned
/// (in reverse post-order) for later use.
fn remove_loop_invariant_instructions(
    lp: Loop,
    func: &mut Function,
    cfg: &ControlFlowGraph,
    loop_analysis: &LoopAnalysis,
) -> Vec<Inst> {
    let mut loop_values: FxHashSet<Value> = FxHashSet();
    let mut invariant_insts: Vec<Inst> = Vec::new();
    let mut pos = FuncCursor::new(func);
    // We traverse the loop block in reverse post-order.
    for block in postorder_blocks_loop(loop_analysis, cfg, lp).iter().rev() {
        // Arguments of the block are loop values
        for val in pos.func.dfg.block_params(*block) {
            loop_values.insert(*val);
        }
        pos.goto_top(*block);
        #[cfg_attr(feature = "cargo-clippy", allow(clippy::block_in_if_condition_stmt))]
        while let Some(inst) = pos.next_inst() {
            if is_loop_invariant(inst, &pos.func.dfg, &loop_values) {
                // If all the instruction's argument are defined outside the loop
                // then this instruction is loop-invariant
                invariant_insts.push(inst);
                // We remove it from the loop
                pos.remove_inst_and_step_back();
            } else {
                // If the instruction is not loop-invariant we push its results in the set of
                // loop values
                for out in pos.func.dfg.inst_results(inst) {
                    loop_values.insert(*out);
                }
            }
        }
    }
    invariant_insts
}

/// Return blocks from a loop in post-order, starting from an entry point in the block.
fn postorder_blocks_loop(
    loop_analysis: &LoopAnalysis,
    cfg: &ControlFlowGraph,
    lp: Loop,
) -> Vec<Block> {
    let mut grey = FxHashSet();
    let mut black = FxHashSet();
    let mut stack = vec![loop_analysis.loop_header(lp)];
    let mut postorder = Vec::new();

    while !stack.is_empty() {
        let node = stack.pop().unwrap();
        if !grey.contains(&node) {
            // This is a white node. Mark it as gray.
            grey.insert(node);
            stack.push(node);
            // Get any children we've never seen before.
            for child in cfg.succ_iter(node) {
                if loop_analysis.is_in_loop(child, lp) && !grey.contains(&child) {
                    stack.push(child);
                }
            }
        } else if !black.contains(&node) {
            postorder.push(node);
            black.insert(node);
        }
    }
    postorder
}
