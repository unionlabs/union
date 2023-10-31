//! Verify CPU flags values.

use crate::entity::{EntitySet, SecondaryMap};
use crate::flowgraph::{BlockPredecessor, ControlFlowGraph};
use crate::ir;
use crate::ir::instructions::BranchInfo;
use crate::packed_option::PackedOption;
use crate::timing;
use crate::verifier::{VerifierErrors, VerifierStepResult};

/// Verify that CPU flags are used correctly.
///
/// The value types `iflags` and `fflags` represent CPU flags which usually live in a
/// special-purpose register, so they can't be used as freely as other value types that can live in
/// any register.
///
/// We verify the following conditions:
///
/// - At most one flags value can be live at a time.
/// - A flags value can not be live across an instruction that clobbers the flags.
///
///
pub fn verify_flags(
    func: &ir::Function,
    cfg: &ControlFlowGraph,
    errors: &mut VerifierErrors,
) -> VerifierStepResult<()> {
    let _tt = timing::verify_flags();
    let mut verifier = FlagsVerifier {
        func,
        cfg,
        livein: SecondaryMap::new(),
    };
    verifier.check(errors)
}

struct FlagsVerifier<'a> {
    func: &'a ir::Function,
    cfg: &'a ControlFlowGraph,

    /// The single live-in flags value (if any) for each block.
    livein: SecondaryMap<ir::Block, PackedOption<ir::Value>>,
}

impl<'a> FlagsVerifier<'a> {
    fn check(&mut self, errors: &mut VerifierErrors) -> VerifierStepResult<()> {
        // List of blocks that need to be processed. blocks may be re-added to this list when we detect
        // that one of their successor blocks needs a live-in flags value.
        let mut worklist = EntitySet::with_capacity(self.func.layout.block_capacity());
        for block in self.func.layout.blocks() {
            worklist.insert(block);
        }

        while let Some(block) = worklist.pop() {
            if let Some(value) = self.visit_block(block, errors)? {
                // The block has live-in flags. Check if the value changed.
                match self.livein[block].expand() {
                    // Revisit any predecessor blocks the first time we see a live-in for `block`.
                    None => {
                        self.livein[block] = value.into();
                        for BlockPredecessor { block: pred, .. } in self.cfg.pred_iter(block) {
                            worklist.insert(pred);
                        }
                    }
                    Some(old) if old != value => {
                        return errors.fatal((
                            block,
                            format!("conflicting live-in CPU flags: {} and {}", old, value),
                        ));
                    }
                    x => assert_eq!(x, Some(value)),
                }
            } else {
                // Existing live-in flags should never be able to disappear.
                assert_eq!(self.livein[block].expand(), None);
            }
        }

        Ok(())
    }

    /// Check flags usage in `block` and return the live-in flags value, if any.
    fn visit_block(
        &self,
        block: ir::Block,
        errors: &mut VerifierErrors,
    ) -> VerifierStepResult<Option<ir::Value>> {
        // The single currently live flags value.
        let mut live_val = None;

        // Visit instructions backwards so we can track liveness accurately.
        for inst in self.func.layout.block_insts(block).rev() {
            // Check if `inst` interferes with existing live flags.
            if let Some(live) = live_val {
                for &res in self.func.dfg.inst_results(inst) {
                    if res == live {
                        // We've reached the def of `live_flags`, so it is no longer live above.
                        live_val = None;
                    } else if self.func.dfg.value_type(res).is_flags() {
                        errors
                            .report((inst, format!("{} clobbers live CPU flags in {}", res, live)));
                        return Err(());
                    }
                }
            }

            // Now look for live ranges of CPU flags that end here.
            for &arg in self.func.dfg.inst_args(inst) {
                if self.func.dfg.value_type(arg).is_flags() {
                    merge(&mut live_val, arg, inst, errors)?;
                }
            }

            // Include live-in flags to successor blocks.
            match self.func.dfg.analyze_branch(inst) {
                BranchInfo::NotABranch => {}
                BranchInfo::SingleDest(dest, _) => {
                    if let Some(val) = self.livein[dest].expand() {
                        merge(&mut live_val, val, inst, errors)?;
                    }
                }
                BranchInfo::Table(jt, dest) => {
                    if let Some(dest) = dest {
                        if let Some(val) = self.livein[dest].expand() {
                            merge(&mut live_val, val, inst, errors)?;
                        }
                    }
                    for dest in self.func.jump_tables[jt].iter() {
                        if let Some(val) = self.livein[*dest].expand() {
                            merge(&mut live_val, val, inst, errors)?;
                        }
                    }
                }
            }
        }

        // Return the required live-in flags value.
        Ok(live_val)
    }
}

// Merge live flags values, or return an error on conflicting values.
fn merge(
    a: &mut Option<ir::Value>,
    b: ir::Value,
    inst: ir::Inst,
    errors: &mut VerifierErrors,
) -> VerifierStepResult<()> {
    if let Some(va) = *a {
        if b != va {
            return errors.fatal((
                inst,
                format!("conflicting live CPU flags: {} and {}", va, b),
            ));
        }
    } else {
        *a = Some(b);
    }

    Ok(())
}
