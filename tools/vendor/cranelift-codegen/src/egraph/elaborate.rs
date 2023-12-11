//! Elaboration phase: lowers EGraph back to sequences of operations
//! in CFG nodes.

use super::domtree::DomTreeWithChildren;
use super::node::{op_cost, Cost, Node, NodeCtx};
use super::Analysis;
use super::Stats;
use crate::dominator_tree::DominatorTree;
use crate::fx::FxHashSet;
use crate::ir::{Block, Function, Inst, Opcode, RelSourceLoc, Type, Value, ValueList};
use crate::loop_analysis::LoopAnalysis;
use crate::scoped_hash_map::ScopedHashMap;
use crate::trace;
use alloc::vec::Vec;
use cranelift_egraph::{EGraph, Id, Language, NodeKey};
use cranelift_entity::{packed_option::PackedOption, SecondaryMap};
use smallvec::{smallvec, SmallVec};
use std::ops::Add;

type LoopDepth = u32;

pub(crate) struct Elaborator<'a> {
    func: &'a mut Function,
    domtree: &'a DominatorTree,
    loop_analysis: &'a LoopAnalysis,
    node_ctx: &'a NodeCtx,
    egraph: &'a EGraph<NodeCtx, Analysis>,
    id_to_value: ScopedHashMap<Id, IdValue>,
    id_to_best_cost_and_node: SecondaryMap<Id, (Cost, Id)>,
    /// Stack of blocks and loops in current elaboration path.
    loop_stack: SmallVec<[LoopStackEntry; 8]>,
    cur_block: Option<Block>,
    first_branch: SecondaryMap<Block, PackedOption<Inst>>,
    remat_ids: &'a FxHashSet<Id>,
    /// Explicitly-unrolled value elaboration stack.
    elab_stack: Vec<ElabStackEntry>,
    elab_result_stack: Vec<IdValue>,
    /// Explicitly-unrolled block elaboration stack.
    block_stack: Vec<BlockStackEntry>,
    stats: &'a mut Stats,
}

#[derive(Clone, Debug)]
struct LoopStackEntry {
    /// The hoist point: a block that immediately dominates this
    /// loop. May not be an immediate predecessor, but will be a valid
    /// point to place all loop-invariant ops: they must depend only
    /// on inputs that dominate the loop, so are available at (the end
    /// of) this block.
    hoist_block: Block,
    /// The depth in the scope map.
    scope_depth: u32,
}

#[derive(Clone, Debug)]
enum ElabStackEntry {
    /// Next action is to resolve this id into a node and elaborate
    /// args.
    Start { id: Id },
    /// Args have been pushed; waiting for results.
    PendingNode {
        canonical: Id,
        node_key: NodeKey,
        remat: bool,
        num_args: usize,
    },
    /// Waiting for a result to return one projected value of a
    /// multi-value result.
    PendingProjection {
        canonical: Id,
        index: usize,
        ty: Type,
    },
}

#[derive(Clone, Debug)]
enum BlockStackEntry {
    Elaborate { block: Block, idom: Option<Block> },
    Pop,
}

#[derive(Clone, Debug)]
enum IdValue {
    /// A single value.
    Value {
        depth: LoopDepth,
        block: Block,
        value: Value,
    },
    /// Multiple results; indices in `node_args`.
    Values {
        depth: LoopDepth,
        block: Block,
        values: ValueList,
    },
}

impl IdValue {
    fn block(&self) -> Block {
        match self {
            IdValue::Value { block, .. } | IdValue::Values { block, .. } => *block,
        }
    }
}

impl<'a> Elaborator<'a> {
    pub(crate) fn new(
        func: &'a mut Function,
        domtree: &'a DominatorTree,
        loop_analysis: &'a LoopAnalysis,
        egraph: &'a EGraph<NodeCtx, Analysis>,
        node_ctx: &'a NodeCtx,
        remat_ids: &'a FxHashSet<Id>,
        stats: &'a mut Stats,
    ) -> Self {
        let num_blocks = func.dfg.num_blocks();
        let mut id_to_best_cost_and_node =
            SecondaryMap::with_default((Cost::infinity(), Id::invalid()));
        id_to_best_cost_and_node.resize(egraph.classes.len());
        Self {
            func,
            domtree,
            loop_analysis,
            egraph,
            node_ctx,
            id_to_value: ScopedHashMap::with_capacity(egraph.classes.len()),
            id_to_best_cost_and_node,
            loop_stack: smallvec![],
            cur_block: None,
            first_branch: SecondaryMap::with_capacity(num_blocks),
            remat_ids,
            elab_stack: vec![],
            elab_result_stack: vec![],
            block_stack: vec![],
            stats,
        }
    }

    fn cur_loop_depth(&self) -> LoopDepth {
        self.loop_stack.len() as LoopDepth
    }

    fn start_block(&mut self, idom: Option<Block>, block: Block, block_params: &[(Id, Type)]) {
        trace!(
            "start_block: block {:?} with idom {:?} at loop depth {} scope depth {}",
            block,
            idom,
            self.cur_loop_depth(),
            self.id_to_value.depth()
        );

        // Note that if the *entry* block is a loop header, we will
        // not make note of the loop here because it will not have an
        // immediate dominator. We must disallow this case because we
        // will skip adding the `LoopStackEntry` here but our
        // `LoopAnalysis` will otherwise still make note of this loop
        // and loop depths will not match.
        if let Some(idom) = idom {
            if self.loop_analysis.is_loop_header(block).is_some() {
                self.loop_stack.push(LoopStackEntry {
                    // Any code hoisted out of this loop will have code
                    // placed in `idom`, and will have def mappings
                    // inserted in to the scoped hashmap at that block's
                    // level.
                    hoist_block: idom,
                    scope_depth: (self.id_to_value.depth() - 1) as u32,
                });
                trace!(
                    " -> loop header, pushing; depth now {}",
                    self.loop_stack.len()
                );
            }
        } else {
            debug_assert!(
                self.loop_analysis.is_loop_header(block).is_none(),
                "Entry block (domtree root) cannot be a loop header!"
            );
        }

        self.cur_block = Some(block);
        for &(id, ty) in block_params {
            let value = self.func.dfg.append_block_param(block, ty);
            trace!(" -> block param id {:?} value {:?}", id, value);
            self.id_to_value.insert_if_absent(
                id,
                IdValue::Value {
                    depth: self.cur_loop_depth(),
                    block,
                    value,
                },
            );
        }
    }

    fn add_node(&mut self, node: &Node, args: &[Value], to_block: Block) -> ValueList {
        let (instdata, result_ty, arity) = match node {
            Node::Pure { op, ty, arity, .. } | Node::Inst { op, ty, arity, .. } => (
                op.with_args(args, &mut self.func.dfg.value_lists),
                *ty,
                *arity,
            ),
            Node::Load { op, ty, .. } => {
                (op.with_args(args, &mut self.func.dfg.value_lists), *ty, 1)
            }
            _ => panic!("Cannot `add_node()` on block param or projection"),
        };
        let srcloc = match node {
            Node::Inst { srcloc, .. } | Node::Load { srcloc, .. } => *srcloc,
            _ => RelSourceLoc::default(),
        };
        let opcode = instdata.opcode();
        // Is this instruction either an actual terminator (an
        // instruction that must end the block), or at least in the
        // group of branches at the end (including conditional
        // branches that may be followed by an actual terminator)? We
        // call this the "terminator group", and we record the first
        // inst in this group (`first_branch` below) so that we do not
        // insert instructions needed only by args of later
        // instructions in the terminator group in the middle of the
        // terminator group.
        //
        // E.g., for the original sequence
        //   v1 = op ...
        //   brnz vCond, block1
        //   jump block2(v1)
        //
        // elaboration would naively produce
        //
        //   brnz vCond, block1
        //   v1 = op ...
        //   jump block2(v1)
        //
        // but we use the `first_branch` mechanism below to ensure
        // that once we've emitted at least one branch, all other
        // elaborated insts have to go before that. So we emit brnz
        // first, then as we elaborate the jump, we find we need the
        // `op`; we `insert_inst` it *before* the brnz (which is the
        // `first_branch`).
        let is_terminator_group_inst =
            opcode.is_branch() || opcode.is_return() || opcode == Opcode::Trap;
        let inst = self.func.dfg.make_inst(instdata);
        self.func.srclocs[inst] = srcloc;

        if arity == 1 {
            self.func.dfg.append_result(inst, result_ty);
        } else {
            for _ in 0..arity {
                self.func.dfg.append_result(inst, crate::ir::types::INVALID);
            }
        }

        if is_terminator_group_inst {
            self.func.layout.append_inst(inst, to_block);
            if self.first_branch[to_block].is_none() {
                self.first_branch[to_block] = Some(inst).into();
            }
        } else if let Some(branch) = self.first_branch[to_block].into() {
            self.func.layout.insert_inst(inst, branch);
        } else {
            self.func.layout.append_inst(inst, to_block);
        }
        self.func.dfg.inst_results_list(inst)
    }

    fn compute_best_nodes(&mut self) {
        let best = &mut self.id_to_best_cost_and_node;
        for (eclass_id, eclass) in &self.egraph.classes {
            trace!("computing best for eclass {:?}", eclass_id);
            if let Some(child1) = eclass.child1() {
                trace!(" -> child {:?}", child1);
                best[eclass_id] = best[child1];
            }
            if let Some(child2) = eclass.child2() {
                trace!(" -> child {:?}", child2);
                if best[child2].0 < best[eclass_id].0 {
                    best[eclass_id] = best[child2];
                }
            }
            if let Some(node_key) = eclass.get_node() {
                let node = node_key.node(&self.egraph.nodes);
                trace!(" -> eclass {:?}: node {:?}", eclass_id, node);
                let (cost, id) = match node {
                    Node::Param { .. }
                    | Node::Inst { .. }
                    | Node::Load { .. }
                    | Node::Result { .. } => (Cost::zero(), eclass_id),
                    Node::Pure { op, .. } => {
                        let args_cost = self
                            .node_ctx
                            .children(node)
                            .iter()
                            .map(|&arg_id| {
                                trace!("  -> arg {:?}", arg_id);
                                best[arg_id].0
                            })
                            // Can't use `.sum()` for `Cost` types; do
                            // an explicit reduce instead.
                            .fold(Cost::zero(), Cost::add);
                        let level = self.egraph.analysis_value(eclass_id).loop_level;
                        let cost = op_cost(op).at_level(level) + args_cost;
                        (cost, eclass_id)
                    }
                };

                if cost < best[eclass_id].0 {
                    best[eclass_id] = (cost, id);
                }
            }
            debug_assert_ne!(best[eclass_id].0, Cost::infinity());
            debug_assert_ne!(best[eclass_id].1, Id::invalid());
            trace!("best for eclass {:?}: {:?}", eclass_id, best[eclass_id]);
        }
    }

    fn elaborate_eclass_use(&mut self, id: Id) {
        self.elab_stack.push(ElabStackEntry::Start { id });
        self.process_elab_stack();
        debug_assert_eq!(self.elab_result_stack.len(), 1);
        self.elab_result_stack.clear();
    }

    fn process_elab_stack(&mut self) {
        while let Some(entry) = self.elab_stack.last() {
            match entry {
                &ElabStackEntry::Start { id } => {
                    // We always replace the Start entry, so pop it now.
                    self.elab_stack.pop();

                    self.stats.elaborate_visit_node += 1;
                    let canonical = self.egraph.canonical_id(id);
                    trace!("elaborate: id {}", id);

                    let remat = if let Some(val) = self.id_to_value.get(&canonical) {
                        // Look at the defined block, and determine whether this
                        // node kind allows rematerialization if the value comes
                        // from another block. If so, ignore the hit and recompute
                        // below.
                        let remat = val.block() != self.cur_block.unwrap()
                            && self.remat_ids.contains(&canonical);
                        if !remat {
                            trace!("elaborate: id {} -> {:?}", id, val);
                            self.stats.elaborate_memoize_hit += 1;
                            self.elab_result_stack.push(val.clone());
                            continue;
                        }
                        trace!("elaborate: id {} -> remat", id);
                        self.stats.elaborate_memoize_miss_remat += 1;
                        // The op is pure at this point, so it is always valid to
                        // remove from this map.
                        self.id_to_value.remove(&canonical);
                        true
                    } else {
                        self.remat_ids.contains(&canonical)
                    };
                    self.stats.elaborate_memoize_miss += 1;

                    // Get the best option; we use `id` (latest id) here so we
                    // have a full view of the eclass.
                    let (_, best_node_eclass) = self.id_to_best_cost_and_node[id];
                    debug_assert_ne!(best_node_eclass, Id::invalid());

                    trace!(
                        "elaborate: id {} -> best {} -> eclass node {:?}",
                        id,
                        best_node_eclass,
                        self.egraph.classes[best_node_eclass]
                    );
                    let node_key = self.egraph.classes[best_node_eclass].get_node().unwrap();
                    let node = node_key.node(&self.egraph.nodes);
                    trace!(" -> enode {:?}", node);

                    // Is the node a block param? We should never get here if so
                    // (they are inserted when first visiting the block).
                    if matches!(node, Node::Param { .. }) {
                        unreachable!("Param nodes should already be inserted");
                    }

                    // Is the node a result projection? If so, resolve
                    // the value we are projecting a part of, then
                    // eventually return here (saving state with a
                    // PendingProjection).
                    if let Node::Result {
                        value, result, ty, ..
                    } = node
                    {
                        trace!(" -> result; pushing arg value {}", value);
                        self.elab_stack.push(ElabStackEntry::PendingProjection {
                            index: *result,
                            canonical,
                            ty: *ty,
                        });
                        self.elab_stack.push(ElabStackEntry::Start { id: *value });
                        continue;
                    }

                    // We're going to need to emit this
                    // operator. First, enqueue all args to be
                    // elaborated. Push state to receive the results
                    // and later elab this node.
                    let num_args = self.node_ctx.children(&node).len();
                    self.elab_stack.push(ElabStackEntry::PendingNode {
                        canonical,
                        node_key,
                        remat,
                        num_args,
                    });
                    // Push args in reverse order so we process the
                    // first arg first.
                    for &arg_id in self.node_ctx.children(&node).iter().rev() {
                        self.elab_stack.push(ElabStackEntry::Start { id: arg_id });
                    }
                }

                &ElabStackEntry::PendingNode {
                    canonical,
                    node_key,
                    remat,
                    num_args,
                } => {
                    self.elab_stack.pop();

                    let node = node_key.node(&self.egraph.nodes);

                    // We should have all args resolved at this point.
                    let arg_idx = self.elab_result_stack.len() - num_args;
                    let args = &self.elab_result_stack[arg_idx..];

                    // Gather the individual output-CLIF `Value`s.
                    let arg_values: SmallVec<[Value; 8]> = args
                        .iter()
                        .map(|idvalue| match idvalue {
                            IdValue::Value { value, .. } => *value,
                            IdValue::Values { .. } => {
                                panic!("enode depends directly on multi-value result")
                            }
                        })
                        .collect();

                    // Compute max loop depth.
                    let max_loop_depth = args
                        .iter()
                        .map(|idvalue| match idvalue {
                            IdValue::Value { depth, .. } => *depth,
                            IdValue::Values { .. } => unreachable!(),
                        })
                        .max()
                        .unwrap_or(0);

                    // Remove args from result stack.
                    self.elab_result_stack.truncate(arg_idx);

                    // Determine the location at which we emit it. This is the
                    // current block *unless* we hoist above a loop when all args
                    // are loop-invariant (and this op is pure).
                    let (loop_depth, scope_depth, block) = if node.is_non_pure() {
                        // Non-pure op: always at the current location.
                        (
                            self.cur_loop_depth(),
                            self.id_to_value.depth(),
                            self.cur_block.unwrap(),
                        )
                    } else if max_loop_depth == self.cur_loop_depth() || remat {
                        // Pure op, but depends on some value at the current loop
                        // depth, or remat forces it here: as above.
                        (
                            self.cur_loop_depth(),
                            self.id_to_value.depth(),
                            self.cur_block.unwrap(),
                        )
                    } else {
                        // Pure op, and does not depend on any args at current
                        // loop depth: hoist out of loop.
                        self.stats.elaborate_licm_hoist += 1;
                        let data = &self.loop_stack[max_loop_depth as usize];
                        (max_loop_depth, data.scope_depth as usize, data.hoist_block)
                    };
                    // Loop scopes are a subset of all scopes.
                    debug_assert!(scope_depth >= loop_depth as usize);

                    // This is an actual operation; emit the node in sequence now.
                    let results = self.add_node(node, &arg_values[..], block);
                    let results_slice = results.as_slice(&self.func.dfg.value_lists);

                    // Build the result and memoize in the id-to-value map.
                    let result = if results_slice.len() == 1 {
                        IdValue::Value {
                            depth: loop_depth,
                            block,
                            value: results_slice[0],
                        }
                    } else {
                        IdValue::Values {
                            depth: loop_depth,
                            block,
                            values: results,
                        }
                    };

                    self.id_to_value.insert_if_absent_with_depth(
                        canonical,
                        result.clone(),
                        scope_depth,
                    );

                    // Push onto the elab-results stack.
                    self.elab_result_stack.push(result)
                }
                &ElabStackEntry::PendingProjection {
                    ty,
                    index,
                    canonical,
                } => {
                    self.elab_stack.pop();

                    // Grab the input from the elab-result stack.
                    let value = self.elab_result_stack.pop().expect("Should have result");

                    let (depth, block, values) = match value {
                        IdValue::Values {
                            depth,
                            block,
                            values,
                            ..
                        } => (depth, block, values),
                        IdValue::Value { .. } => {
                            unreachable!("Projection nodes should not be used on single results");
                        }
                    };
                    let values = values.as_slice(&self.func.dfg.value_lists);
                    let value = values[index];
                    self.func.dfg.fill_in_value_type(value, ty);
                    let value = IdValue::Value {
                        depth,
                        block,
                        value,
                    };
                    self.id_to_value.insert_if_absent(canonical, value.clone());

                    self.elab_result_stack.push(value);
                }
            }
        }
    }

    fn elaborate_block<'b, PF: Fn(Block) -> &'b [(Id, Type)], SEF: Fn(Block) -> &'b [Id]>(
        &mut self,
        idom: Option<Block>,
        block: Block,
        block_params_fn: &PF,
        block_side_effects_fn: &SEF,
    ) {
        let blockparam_ids_tys = (block_params_fn)(block);
        self.start_block(idom, block, blockparam_ids_tys);
        for &id in (block_side_effects_fn)(block) {
            self.elaborate_eclass_use(id);
        }
    }

    fn elaborate_domtree<'b, PF: Fn(Block) -> &'b [(Id, Type)], SEF: Fn(Block) -> &'b [Id]>(
        &mut self,
        block_params_fn: &PF,
        block_side_effects_fn: &SEF,
        domtree: &DomTreeWithChildren,
    ) {
        let root = domtree.root();
        self.block_stack.push(BlockStackEntry::Elaborate {
            block: root,
            idom: None,
        });
        while let Some(top) = self.block_stack.pop() {
            match top {
                BlockStackEntry::Elaborate { block, idom } => {
                    self.block_stack.push(BlockStackEntry::Pop);
                    self.id_to_value.increment_depth();

                    self.elaborate_block(idom, block, block_params_fn, block_side_effects_fn);

                    // Push children. We are doing a preorder
                    // traversal so we do this after processing this
                    // block above.
                    let block_stack_end = self.block_stack.len();
                    for child in domtree.children(block) {
                        self.block_stack.push(BlockStackEntry::Elaborate {
                            block: child,
                            idom: Some(block),
                        });
                    }
                    // Reverse what we just pushed so we elaborate in
                    // original block order. (The domtree iter is a
                    // single-ended iter over a singly-linked list so
                    // we can't `.rev()` above.)
                    self.block_stack[block_stack_end..].reverse();
                }
                BlockStackEntry::Pop => {
                    self.id_to_value.decrement_depth();
                    if let Some(innermost_loop) = self.loop_stack.last() {
                        if innermost_loop.scope_depth as usize == self.id_to_value.depth() {
                            self.loop_stack.pop();
                        }
                    }
                }
            }
        }
    }

    fn clear_func_body(&mut self) {
        // Clear all instructions and args/results from the DFG. We
        // rebuild them entirely during elaboration. (TODO: reuse the
        // existing inst for the *first* copy of a given node.)
        self.func.dfg.clear_insts();
        // Clear the instructions in every block, but leave the list
        // of blocks and their layout unmodified.
        self.func.layout.clear_insts();
        self.func.srclocs.clear();
    }

    pub(crate) fn elaborate<'b, PF: Fn(Block) -> &'b [(Id, Type)], SEF: Fn(Block) -> &'b [Id]>(
        &mut self,
        block_params_fn: PF,
        block_side_effects_fn: SEF,
    ) {
        let domtree = DomTreeWithChildren::new(self.func, self.domtree);
        self.stats.elaborate_func += 1;
        self.stats.elaborate_func_pre_insts += self.func.dfg.num_insts() as u64;
        self.clear_func_body();
        self.compute_best_nodes();
        self.elaborate_domtree(&block_params_fn, &block_side_effects_fn, &domtree);
        self.stats.elaborate_func_post_insts += self.func.dfg.num_insts() as u64;
    }
}
