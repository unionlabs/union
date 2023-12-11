//! A Dead-Code Elimination (DCE) pass.
//!
//! Dead code here means instructions that have no side effects and have no
//! result values used by other instructions.

use crate::cursor::{Cursor, FuncCursor};
use crate::dominator_tree::DominatorTree;
use crate::entity::EntityRef;
use crate::inst_predicates::{any_inst_results_used, has_side_effect};
use crate::ir::Function;
use crate::timing;

/// Perform DCE on `func`.
pub fn do_dce(func: &mut Function, domtree: &mut DominatorTree) {
    let _tt = timing::dce();
    debug_assert!(domtree.is_valid());

    let mut live = vec![false; func.dfg.num_values()];
    for &block in domtree.cfg_postorder() {
        let mut pos = FuncCursor::new(func).at_bottom(block);
        while let Some(inst) = pos.prev_inst() {
            {
                if has_side_effect(pos.func, inst)
                    || any_inst_results_used(inst, &live, &pos.func.dfg)
                {
                    for arg in pos.func.dfg.inst_args(inst) {
                        let v = pos.func.dfg.resolve_aliases(*arg);
                        live[v.index()] = true;
                    }
                    continue;
                }
            }
            pos.remove_inst();
        }
    }
}
