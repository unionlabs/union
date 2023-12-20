//! Computation of basic block order in emitted code.
//!
//! This module handles the translation from CLIF BBs to VCode BBs.
//!
//! The basic idea is that we compute a sequence of "lowered blocks" that
//! correspond to one or more blocks in the graph: (CLIF CFG) `union` (implicit
//! block on *every* edge). Conceptually, the lowering pipeline wants to insert
//! moves for phi-nodes on every block-to-block transfer; these blocks always
//! conceptually exist, but may be merged with an "original" CLIF block (and
//! hence not actually exist; this is equivalent to inserting the blocks only on
//! critical edges).
//!
//! In other words, starting from a CFG like this (where each "CLIF block" and
//! "(edge N->M)" is a separate basic block):
//!
//! ```plain
//!
//!              CLIF block 0
//!               /           \
//!       (edge 0->1)         (edge 0->2)
//!              |                |
//!       CLIF block 1         CLIF block 2
//!              \                /
//!           (edge 1->3)   (edge 2->3)
//!                   \      /
//!                 CLIF block 3
//! ```
//!
//! We can produce a CFG of lowered blocks like so:
//!
//! ```plain
//!            +--------------+
//!            | CLIF block 0 |
//!            +--------------+
//!               /           \
//!     +--------------+     +--------------+
//!     | (edge 0->1)  |     |(edge 0->2)   |
//!     | CLIF block 1 |     | CLIF block 2 |
//!     +--------------+     +--------------+
//!              \                /
//!          +-----------+ +-----------+
//!          |(edge 1->3)| |(edge 2->3)|
//!          +-----------+ +-----------+
//!                   \      /
//!                +------------+
//!                |CLIF block 3|
//!                +------------+
//! ```
//!
//! (note that the edges into CLIF blocks 1 and 2 could be merged with those
//! blocks' original bodies, but the out-edges could not because for simplicity
//! in the successor-function definition, we only ever merge an edge onto one
//! side of an original CLIF block.)
//!
//! Each `LoweredBlock` names just an original CLIF block, an original CLIF
//! block prepended or appended with an edge block (never both, though), or just
//! an edge block.
//!
//! To compute this lowering, we do a DFS over the CLIF-plus-edge-block graph
//! (never actually materialized, just defined by a "successors" function), and
//! compute the reverse postorder.
//!
//! This algorithm isn't perfect w.r.t. generated code quality: we don't, for
//! example, consider any information about whether edge blocks will actually
//! have content, because this computation happens as part of lowering *before*
//! regalloc, and regalloc may or may not insert moves/spills/reloads on any
//! particular edge. But it works relatively well and is conceptually simple.
//! Furthermore, the [MachBuffer] machine-code sink performs final peephole-like
//! branch editing that in practice elides empty blocks and simplifies some of
//! the other redundancies that this scheme produces.

use crate::entity::SecondaryMap;
use crate::fx::{FxHashMap, FxHashSet};
use crate::ir::{Block, Function, Inst, Opcode};
use crate::machinst::lower::visit_block_succs;
use crate::machinst::*;

use smallvec::SmallVec;

/// Mapping from CLIF BBs to VCode BBs.
#[derive(Debug)]
pub struct BlockLoweringOrder {
    /// Lowered blocks, in BlockIndex order. Each block is some combination of
    /// (i) a CLIF block, and (ii) inserted crit-edge blocks before or after;
    /// see [LoweredBlock] for details.
    lowered_order: Vec<LoweredBlock>,
    /// Successors for all lowered blocks, in one serialized vector. Indexed by
    /// the ranges in `lowered_succ_ranges`.
    #[allow(dead_code)]
    lowered_succs: Vec<(Inst, LoweredBlock)>,
    /// BlockIndex values for successors for all lowered blocks, in the same
    /// order as `lowered_succs`.
    lowered_succ_indices: Vec<(Inst, BlockIndex)>,
    /// Ranges in `lowered_succs` giving the successor lists for each lowered
    /// block. Indexed by lowering-order index (`BlockIndex`).
    lowered_succ_ranges: Vec<(usize, usize)>,
    /// Mapping from CLIF BB to BlockIndex (index in lowered order). Note that
    /// some CLIF BBs may not be lowered; in particular, we skip unreachable
    /// blocks.
    #[allow(dead_code)]
    orig_map: SecondaryMap<Block, Option<BlockIndex>>,
    /// Cold blocks. These blocks are not reordered in the
    /// `lowered_order` above; the lowered order must respect RPO
    /// (uses after defs) in order for lowering to be
    /// correct. Instead, this set is used to provide `is_cold()`,
    /// which is used by VCode emission to sink the blocks at the last
    /// moment (when we actually emit bytes into the MachBuffer).
    cold_blocks: FxHashSet<BlockIndex>,
}

/// The origin of a block in the lowered block-order: either an original CLIF
/// block, or an inserted edge-block, or a combination of the two if an edge is
/// non-critical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoweredBlock {
    /// Block in original CLIF, with no merged edge-blocks.
    Orig {
        /// Original CLIF block.
        block: Block,
    },
    /// Block in the original CLIF, plus edge-block to one succ (which is the
    /// one successor of the original block).
    OrigAndEdge {
        /// The original CLIF block contained in this lowered block.
        block: Block,
        /// The edge (jump) instruction transitioning from this block
        /// to the next, i.e., corresponding to the included edge-block. This
        /// will be an instruction in `block`.
        edge_inst: Inst,
        /// The successor CLIF block.
        succ: Block,
    },
    /// Block in the original CLIF, preceded by edge-block from one pred (which
    /// is the one pred of the original block).
    EdgeAndOrig {
        /// The previous CLIF block, i.e., the edge block's predecessor.
        pred: Block,
        /// The edge (jump) instruction corresponding to the included
        /// edge-block. This will be an instruction in `pred`.
        edge_inst: Inst,
        /// The original CLIF block included in this lowered block.
        block: Block,
    },
    /// Split critical edge between two CLIF blocks. This lowered block does not
    /// correspond to any original CLIF blocks; it only serves as an insertion
    /// point for work to happen on the transition from `pred` to `succ`.
    Edge {
        /// The predecessor CLIF block.
        pred: Block,
        /// The edge (jump) instruction corresponding to this edge's transition.
        /// This will be an instruction in `pred`.
        edge_inst: Inst,
        /// The successor CLIF block.
        succ: Block,
    },
}

impl LoweredBlock {
    /// The associated original (CLIF) block included in this lowered block, if
    /// any.
    pub fn orig_block(self) -> Option<Block> {
        match self {
            LoweredBlock::Orig { block, .. }
            | LoweredBlock::OrigAndEdge { block, .. }
            | LoweredBlock::EdgeAndOrig { block, .. } => Some(block),
            LoweredBlock::Edge { .. } => None,
        }
    }

    /// The associated in-edge, if any.
    pub fn in_edge(self) -> Option<(Block, Inst, Block)> {
        match self {
            LoweredBlock::EdgeAndOrig {
                pred,
                edge_inst,
                block,
            } => Some((pred, edge_inst, block)),
            _ => None,
        }
    }

    /// the associated out-edge, if any. Also includes edge-only blocks.
    pub fn out_edge(self) -> Option<(Block, Inst, Block)> {
        match self {
            LoweredBlock::OrigAndEdge {
                block,
                edge_inst,
                succ,
            } => Some((block, edge_inst, succ)),
            LoweredBlock::Edge {
                pred,
                edge_inst,
                succ,
            } => Some((pred, edge_inst, succ)),
            _ => None,
        }
    }
}

impl BlockLoweringOrder {
    /// Compute and return a lowered block order for `f`.
    pub fn new(f: &Function) -> BlockLoweringOrder {
        log::trace!("BlockLoweringOrder: function body {:?}", f);

        // Step 1: compute the in-edge and out-edge count of every block.
        let mut block_in_count = SecondaryMap::with_default(0);
        let mut block_out_count = SecondaryMap::with_default(0);

        // Cache the block successors to avoid re-examining branches below.
        let mut block_succs: SmallVec<[(Inst, Block); 128]> = SmallVec::new();
        let mut block_succ_range = SecondaryMap::with_default((0, 0));
        let mut fallthrough_return_block = None;
        for block in f.layout.blocks() {
            let block_succ_start = block_succs.len();
            visit_block_succs(f, block, |inst, succ| {
                block_out_count[block] += 1;
                block_in_count[succ] += 1;
                block_succs.push((inst, succ));
            });
            let block_succ_end = block_succs.len();
            block_succ_range[block] = (block_succ_start, block_succ_end);

            for inst in f.layout.block_likely_branches(block) {
                if f.dfg[inst].opcode() == Opcode::Return {
                    // Implicit output edge for any return.
                    block_out_count[block] += 1;
                }
                if f.dfg[inst].opcode() == Opcode::FallthroughReturn {
                    // Fallthrough return block must come last.
                    debug_assert!(fallthrough_return_block == None);
                    fallthrough_return_block = Some(block);
                }
            }
        }
        // Implicit input edge for entry block.
        if let Some(entry) = f.layout.entry_block() {
            block_in_count[entry] += 1;
        }

        // Here we define the implicit CLIF-plus-edges graph. There are
        // conceptually two such graphs: the original, with every edge explicit,
        // and the merged one, with blocks (represented by `LoweredBlock`
        // values) that contain original CLIF blocks, edges, or both. This
        // function returns a lowered block's successors as per the latter, with
        // consideration to edge-block merging.
        //
        // Note that there is a property of the block-merging rules below
        // that is very important to ensure we don't miss any lowered blocks:
        // any block in the implicit CLIF-plus-edges graph will *only* be
        // included in one block in the merged graph.
        //
        // This, combined with the property that every edge block is reachable
        // only from one predecessor (and hence cannot be reached by a DFS
        // backedge), means that it is sufficient in our DFS below to track
        // visited-bits per original CLIF block only, not per edge. This greatly
        // simplifies the data structures (no need to keep a sparse hash-set of
        // (block, block) tuples).
        let compute_lowered_succs = |ret: &mut Vec<(Inst, LoweredBlock)>, block: LoweredBlock| {
            let start_idx = ret.len();
            match block {
                LoweredBlock::Orig { block } | LoweredBlock::EdgeAndOrig { block, .. } => {
                    // At an orig block; successors are always edge blocks,
                    // possibly with orig blocks following.
                    let range = block_succ_range[block];
                    for &(edge_inst, succ) in &block_succs[range.0..range.1] {
                        if block_in_count[succ] == 1 {
                            ret.push((
                                edge_inst,
                                LoweredBlock::EdgeAndOrig {
                                    pred: block,
                                    edge_inst,
                                    block: succ,
                                },
                            ));
                        } else {
                            ret.push((
                                edge_inst,
                                LoweredBlock::Edge {
                                    pred: block,
                                    edge_inst,
                                    succ,
                                },
                            ));
                        }
                    }
                }
                LoweredBlock::Edge {
                    succ, edge_inst, ..
                }
                | LoweredBlock::OrigAndEdge {
                    succ, edge_inst, ..
                } => {
                    // At an edge block; successors are always orig blocks,
                    // possibly with edge blocks following.
                    if block_out_count[succ] == 1 {
                        let range = block_succ_range[succ];
                        // check if the one succ is a real CFG edge (vs.
                        // implicit return succ).
                        if range.1 - range.0 > 0 {
                            debug_assert!(range.1 - range.0 == 1);
                            let (succ_edge_inst, succ_succ) = block_succs[range.0];
                            ret.push((
                                edge_inst,
                                LoweredBlock::OrigAndEdge {
                                    block: succ,
                                    edge_inst: succ_edge_inst,
                                    succ: succ_succ,
                                },
                            ));
                        } else {
                            ret.push((edge_inst, LoweredBlock::Orig { block: succ }));
                        }
                    } else {
                        ret.push((edge_inst, LoweredBlock::Orig { block: succ }));
                    }
                }
            }
            let end_idx = ret.len();
            (start_idx, end_idx)
        };

        // Build the explicit LoweredBlock-to-LoweredBlock successors list.
        let mut lowered_succs = vec![];
        let mut lowered_succ_indices = vec![];

        // Step 2: Compute RPO traversal of the implicit CLIF-plus-edge-block graph. Use an
        // explicit stack so we don't overflow the real stack with a deep DFS.
        #[derive(Debug)]
        struct StackEntry {
            this: LoweredBlock,
            succs: (usize, usize), // range in lowered_succs
            cur_succ: usize,       // index in lowered_succs
        }

        let mut stack: SmallVec<[StackEntry; 16]> = SmallVec::new();
        let mut visited = FxHashSet::default();
        let mut postorder = vec![];
        if let Some(entry) = f.layout.entry_block() {
            // FIXME(cfallin): we might be able to use OrigAndEdge. Find a way
            // to not special-case the entry block here.
            let block = LoweredBlock::Orig { block: entry };
            visited.insert(block);
            let range = compute_lowered_succs(&mut lowered_succs, block);
            lowered_succ_indices.resize(lowered_succs.len(), 0);
            stack.push(StackEntry {
                this: block,
                succs: range,
                cur_succ: range.1,
            });
        }

        let mut deferred_last = None;
        while !stack.is_empty() {
            let stack_entry = stack.last_mut().unwrap();
            let range = stack_entry.succs;
            if stack_entry.cur_succ == range.0 {
                let orig_block = stack_entry.this.orig_block();
                if orig_block.is_some() && orig_block == fallthrough_return_block {
                    deferred_last = Some((stack_entry.this, range));
                } else {
                    postorder.push((stack_entry.this, range));
                }
                stack.pop();
            } else {
                // Heuristic: chase the children in reverse. This puts the first
                // successor block first in RPO, all other things being equal,
                // which tends to prioritize loop backedges over out-edges,
                // putting the edge-block closer to the loop body and minimizing
                // live-ranges in linear instruction space.
                let next = lowered_succs[stack_entry.cur_succ - 1].1;
                stack_entry.cur_succ -= 1;
                if visited.contains(&next) {
                    continue;
                }
                visited.insert(next);
                let range = compute_lowered_succs(&mut lowered_succs, next);
                lowered_succ_indices.resize(lowered_succs.len(), 0);
                stack.push(StackEntry {
                    this: next,
                    succs: range,
                    cur_succ: range.1,
                });
            }
        }

        postorder.reverse();
        let mut rpo = postorder;
        if let Some(d) = deferred_last {
            rpo.push(d);
        }

        // Step 3: now that we have RPO, build the BlockIndex/BB fwd/rev maps.
        let mut lowered_order = vec![];
        let mut cold_blocks = FxHashSet::default();
        let mut lowered_succ_ranges = vec![];
        let mut lb_to_bindex = FxHashMap::default();
        for (block, succ_range) in rpo.into_iter() {
            let index = lowered_order.len() as BlockIndex;
            lb_to_bindex.insert(block, index);
            lowered_order.push(block);
            lowered_succ_ranges.push(succ_range);

            if block
                .orig_block()
                .map(|b| f.layout.is_cold(b))
                .unwrap_or(false)
            {
                cold_blocks.insert(index);
            }
        }

        let lowered_succ_indices = lowered_succs
            .iter()
            .map(|&(inst, succ)| (inst, lb_to_bindex.get(&succ).cloned().unwrap()))
            .collect();

        let mut orig_map = SecondaryMap::with_default(None);
        for (i, lb) in lowered_order.iter().enumerate() {
            let i = i as BlockIndex;
            if let Some(b) = lb.orig_block() {
                orig_map[b] = Some(i);
            }
        }

        let result = BlockLoweringOrder {
            lowered_order,
            lowered_succs,
            lowered_succ_indices,
            lowered_succ_ranges,
            orig_map,
            cold_blocks,
        };
        log::trace!("BlockLoweringOrder: {:?}", result);
        result
    }

    /// Get the lowered order of blocks.
    pub fn lowered_order(&self) -> &[LoweredBlock] {
        &self.lowered_order[..]
    }

    /// Get the successor indices for a lowered block.
    pub fn succ_indices(&self, block: BlockIndex) -> &[(Inst, BlockIndex)] {
        let range = self.lowered_succ_ranges[block as usize];
        &self.lowered_succ_indices[range.0..range.1]
    }

    /// Determine whether the given lowered-block index is cold.
    pub fn is_cold(&self, block: BlockIndex) -> bool {
        self.cold_blocks.contains(&block)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cursor::{Cursor, FuncCursor};
    use crate::ir::types::*;
    use crate::ir::{AbiParam, ExternalName, Function, InstBuilder, Signature};
    use crate::isa::CallConv;

    fn build_test_func(n_blocks: usize, edges: &[(usize, usize)]) -> Function {
        assert!(n_blocks > 0);

        let name = ExternalName::testcase("test0");
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(I32));
        let mut func = Function::with_name_signature(name, sig);
        let blocks = (0..n_blocks)
            .map(|i| {
                let bb = func.dfg.make_block();
                assert!(bb.as_u32() == i as u32);
                bb
            })
            .collect::<Vec<_>>();

        let arg0 = func.dfg.append_block_param(blocks[0], I32);

        let mut pos = FuncCursor::new(&mut func);

        let mut edge = 0;
        for i in 0..n_blocks {
            pos.insert_block(blocks[i]);
            let mut succs = vec![];
            while edge < edges.len() && edges[edge].0 == i {
                succs.push(edges[edge].1);
                edge += 1;
            }
            if succs.len() == 0 {
                pos.ins().return_(&[arg0]);
            } else if succs.len() == 1 {
                pos.ins().jump(blocks[succs[0]], &[]);
            } else if succs.len() == 2 {
                pos.ins().brnz(arg0, blocks[succs[0]], &[]);
                pos.ins().jump(blocks[succs[1]], &[]);
            } else {
                panic!("Too many successors");
            }
        }

        func
    }

    #[test]
    fn test_blockorder_diamond() {
        let func = build_test_func(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let order = BlockLoweringOrder::new(&func);

        assert_eq!(order.lowered_order.len(), 6);

        assert!(order.lowered_order[0].orig_block().unwrap().as_u32() == 0);
        assert!(order.lowered_order[0].in_edge().is_none());
        assert!(order.lowered_order[0].out_edge().is_none());

        assert!(order.lowered_order[1].orig_block().unwrap().as_u32() == 1);
        assert!(order.lowered_order[1].in_edge().unwrap().0.as_u32() == 0);
        assert!(order.lowered_order[1].in_edge().unwrap().2.as_u32() == 1);

        assert!(order.lowered_order[2].orig_block().is_none());
        assert!(order.lowered_order[2].in_edge().is_none());
        assert!(order.lowered_order[2].out_edge().unwrap().0.as_u32() == 1);
        assert!(order.lowered_order[2].out_edge().unwrap().2.as_u32() == 3);

        assert!(order.lowered_order[3].orig_block().unwrap().as_u32() == 2);
        assert!(order.lowered_order[3].in_edge().unwrap().0.as_u32() == 0);
        assert!(order.lowered_order[3].in_edge().unwrap().2.as_u32() == 2);
        assert!(order.lowered_order[3].out_edge().is_none());

        assert!(order.lowered_order[4].orig_block().is_none());
        assert!(order.lowered_order[4].in_edge().is_none());
        assert!(order.lowered_order[4].out_edge().unwrap().0.as_u32() == 2);
        assert!(order.lowered_order[4].out_edge().unwrap().2.as_u32() == 3);

        assert!(order.lowered_order[5].orig_block().unwrap().as_u32() == 3);
        assert!(order.lowered_order[5].in_edge().is_none());
        assert!(order.lowered_order[5].out_edge().is_none());
    }

    #[test]
    fn test_blockorder_critedge() {
        //            0
        //          /   \
        //         1     2
        //        /  \     \
        //       3    4    |
        //       |\  _|____|
        //       | \/ |
        //       | /\ |
        //       5    6
        //
        // (3 -> 5, 3 -> 6, 4 -> 6 are critical edges and must be split)
        //
        let func = build_test_func(
            7,
            &[
                (0, 1),
                (0, 2),
                (1, 3),
                (1, 4),
                (2, 5),
                (3, 5),
                (3, 6),
                (4, 6),
            ],
        );
        let order = BlockLoweringOrder::new(&func);

        assert_eq!(order.lowered_order.len(), 11);
        println!("ordered = {:?}", order.lowered_order);

        // block 0
        assert!(order.lowered_order[0].orig_block().unwrap().as_u32() == 0);
        assert!(order.lowered_order[0].in_edge().is_none());
        assert!(order.lowered_order[0].out_edge().is_none());

        // edge 0->1 + block 1
        assert!(order.lowered_order[1].orig_block().unwrap().as_u32() == 1);
        assert!(order.lowered_order[1].in_edge().unwrap().0.as_u32() == 0);
        assert!(order.lowered_order[1].in_edge().unwrap().2.as_u32() == 1);
        assert!(order.lowered_order[1].out_edge().is_none());

        // edge 1->3 + block 3
        assert!(order.lowered_order[2].orig_block().unwrap().as_u32() == 3);
        assert!(order.lowered_order[2].in_edge().unwrap().0.as_u32() == 1);
        assert!(order.lowered_order[2].in_edge().unwrap().2.as_u32() == 3);
        assert!(order.lowered_order[2].out_edge().is_none());

        // edge 3->5
        assert!(order.lowered_order[3].orig_block().is_none());
        assert!(order.lowered_order[3].in_edge().is_none());
        assert!(order.lowered_order[3].out_edge().unwrap().0.as_u32() == 3);
        assert!(order.lowered_order[3].out_edge().unwrap().2.as_u32() == 5);

        // edge 3->6
        assert!(order.lowered_order[4].orig_block().is_none());
        assert!(order.lowered_order[4].in_edge().is_none());
        assert!(order.lowered_order[4].out_edge().unwrap().0.as_u32() == 3);
        assert!(order.lowered_order[4].out_edge().unwrap().2.as_u32() == 6);

        // edge 1->4 + block 4
        assert!(order.lowered_order[5].orig_block().unwrap().as_u32() == 4);
        assert!(order.lowered_order[5].in_edge().unwrap().0.as_u32() == 1);
        assert!(order.lowered_order[5].in_edge().unwrap().2.as_u32() == 4);
        assert!(order.lowered_order[5].out_edge().is_none());

        // edge 4->6
        assert!(order.lowered_order[6].orig_block().is_none());
        assert!(order.lowered_order[6].in_edge().is_none());
        assert!(order.lowered_order[6].out_edge().unwrap().0.as_u32() == 4);
        assert!(order.lowered_order[6].out_edge().unwrap().2.as_u32() == 6);

        // block 6
        assert!(order.lowered_order[7].orig_block().unwrap().as_u32() == 6);
        assert!(order.lowered_order[7].in_edge().is_none());
        assert!(order.lowered_order[7].out_edge().is_none());

        // edge 0->2 + block 2
        assert!(order.lowered_order[8].orig_block().unwrap().as_u32() == 2);
        assert!(order.lowered_order[8].in_edge().unwrap().0.as_u32() == 0);
        assert!(order.lowered_order[8].in_edge().unwrap().2.as_u32() == 2);
        assert!(order.lowered_order[8].out_edge().is_none());

        // edge 2->5
        assert!(order.lowered_order[9].orig_block().is_none());
        assert!(order.lowered_order[9].in_edge().is_none());
        assert!(order.lowered_order[9].out_edge().unwrap().0.as_u32() == 2);
        assert!(order.lowered_order[9].out_edge().unwrap().2.as_u32() == 5);

        // block 5
        assert!(order.lowered_order[10].orig_block().unwrap().as_u32() == 5);
        assert!(order.lowered_order[10].in_edge().is_none());
        assert!(order.lowered_order[10].out_edge().is_none());
    }
}
