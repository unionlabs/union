//! Egraph-based mid-end optimization framework.

use crate::dominator_tree::DominatorTree;
use crate::egraph::stores::PackedMemoryState;
use crate::flowgraph::ControlFlowGraph;
use crate::loop_analysis::{LoopAnalysis, LoopLevel};
use crate::trace;
use crate::{
    fx::{FxHashMap, FxHashSet},
    inst_predicates::has_side_effect,
    ir::{Block, Function, Inst, InstructionData, InstructionImms, Opcode, Type},
};
use alloc::vec::Vec;
use core::ops::Range;
use cranelift_egraph::{EGraph, Id, Language, NewOrExisting};
use cranelift_entity::EntityList;
use cranelift_entity::SecondaryMap;

mod domtree;
mod elaborate;
mod node;
mod stores;

use elaborate::Elaborator;
pub use node::{Node, NodeCtx};
pub use stores::{AliasAnalysis, MemoryState};

pub struct FuncEGraph<'a> {
    /// Dominator tree, used for elaboration pass.
    domtree: &'a DominatorTree,
    /// Loop analysis results, used for built-in LICM during elaboration.
    loop_analysis: &'a LoopAnalysis,
    /// Last-store tracker for integrated alias analysis during egraph build.
    alias_analysis: AliasAnalysis,
    /// The egraph itself.
    pub(crate) egraph: EGraph<NodeCtx, Analysis>,
    /// "node context", containing arenas for node data.
    pub(crate) node_ctx: NodeCtx,
    /// Ranges in `side_effect_ids` for sequences of side-effecting
    /// eclasses per block.
    side_effects: SecondaryMap<Block, Range<u32>>,
    side_effect_ids: Vec<Id>,
    /// Map from store instructions to their nodes; used for store-to-load forwarding.
    pub(crate) store_nodes: FxHashMap<Inst, (Type, Id)>,
    /// Ranges in `blockparam_ids_tys` for sequences of blockparam
    /// eclass IDs and types per block.
    blockparams: SecondaryMap<Block, Range<u32>>,
    blockparam_ids_tys: Vec<(Id, Type)>,
    /// Which canonical node IDs do we want to rematerialize in each
    /// block where they're used?
    pub(crate) remat_ids: FxHashSet<Id>,
    /// Which canonical node IDs have an enode whose value subsumes
    /// all others it's unioned with?
    pub(crate) subsume_ids: FxHashSet<Id>,
    /// Statistics recorded during the process of building,
    /// optimizing, and lowering out of this egraph.
    pub(crate) stats: Stats,
    /// Current rewrite-recursion depth. Used to enforce a finite
    /// limit on rewrite rule application so that we don't get stuck
    /// in an infinite chain.
    pub(crate) rewrite_depth: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Stats {
    pub(crate) node_created: u64,
    pub(crate) node_param: u64,
    pub(crate) node_result: u64,
    pub(crate) node_pure: u64,
    pub(crate) node_inst: u64,
    pub(crate) node_load: u64,
    pub(crate) node_dedup_query: u64,
    pub(crate) node_dedup_hit: u64,
    pub(crate) node_dedup_miss: u64,
    pub(crate) node_ctor_created: u64,
    pub(crate) node_ctor_deduped: u64,
    pub(crate) node_union: u64,
    pub(crate) node_subsume: u64,
    pub(crate) store_map_insert: u64,
    pub(crate) side_effect_nodes: u64,
    pub(crate) rewrite_rule_invoked: u64,
    pub(crate) rewrite_depth_limit: u64,
    pub(crate) store_to_load_forward: u64,
    pub(crate) elaborate_visit_node: u64,
    pub(crate) elaborate_memoize_hit: u64,
    pub(crate) elaborate_memoize_miss: u64,
    pub(crate) elaborate_memoize_miss_remat: u64,
    pub(crate) elaborate_licm_hoist: u64,
    pub(crate) elaborate_func: u64,
    pub(crate) elaborate_func_pre_insts: u64,
    pub(crate) elaborate_func_post_insts: u64,
}

impl<'a> FuncEGraph<'a> {
    /// Create a new EGraph for the given function. Requires the
    /// domtree to be precomputed as well; the domtree is used for
    /// scheduling when lowering out of the egraph.
    pub fn new(
        func: &Function,
        domtree: &'a DominatorTree,
        loop_analysis: &'a LoopAnalysis,
        cfg: &ControlFlowGraph,
    ) -> FuncEGraph<'a> {
        let num_values = func.dfg.num_values();
        let num_blocks = func.dfg.num_blocks();
        let node_count_estimate = num_values * 2;
        let alias_analysis = AliasAnalysis::new(func, cfg);
        let mut this = Self {
            domtree,
            loop_analysis,
            alias_analysis,
            egraph: EGraph::with_capacity(node_count_estimate, Some(Analysis)),
            node_ctx: NodeCtx::with_capacity_for_dfg(&func.dfg),
            side_effects: SecondaryMap::with_capacity(num_blocks),
            side_effect_ids: Vec::with_capacity(node_count_estimate),
            store_nodes: FxHashMap::default(),
            blockparams: SecondaryMap::with_capacity(num_blocks),
            blockparam_ids_tys: Vec::with_capacity(num_blocks * 10),
            remat_ids: FxHashSet::default(),
            subsume_ids: FxHashSet::default(),
            stats: Default::default(),
            rewrite_depth: 0,
        };
        this.store_nodes.reserve(func.dfg.num_values() / 8);
        this.remat_ids.reserve(func.dfg.num_values() / 4);
        this.subsume_ids.reserve(func.dfg.num_values() / 4);
        this.build(func);
        this
    }

    fn build(&mut self, func: &Function) {
        // Mapping of SSA `Value` to eclass ID.
        let mut value_to_id = FxHashMap::default();

        // For each block in RPO, create an enode for block entry, for
        // each block param, and for each instruction.
        for &block in self.domtree.cfg_postorder().iter().rev() {
            let loop_level = self.loop_analysis.loop_level(block);
            let blockparam_start =
                u32::try_from(self.blockparam_ids_tys.len()).expect("Overflow in blockparam count");
            for (i, &value) in func.dfg.block_params(block).iter().enumerate() {
                let ty = func.dfg.value_type(value);
                let param = self
                    .egraph
                    .add(
                        Node::Param {
                            block,
                            index: i
                                .try_into()
                                .expect("blockparam index should fit in Node::Param"),
                            ty,
                            loop_level,
                        },
                        &mut self.node_ctx,
                    )
                    .get();
                value_to_id.insert(value, param);
                self.blockparam_ids_tys.push((param, ty));
                self.stats.node_created += 1;
                self.stats.node_param += 1;
            }
            let blockparam_end =
                u32::try_from(self.blockparam_ids_tys.len()).expect("Overflow in blockparam count");
            self.blockparams[block] = blockparam_start..blockparam_end;

            let side_effect_start =
                u32::try_from(self.side_effect_ids.len()).expect("Overflow in side-effect count");
            for inst in func.layout.block_insts(block) {
                // Build args from SSA values.
                let args = EntityList::from_iter(
                    func.dfg.inst_args(inst).iter().map(|&arg| {
                        let arg = func.dfg.resolve_aliases(arg);
                        *value_to_id
                            .get(&arg)
                            .expect("Must have seen def before this use")
                    }),
                    &mut self.node_ctx.args,
                );

                let results = func.dfg.inst_results(inst);
                let ty = if results.len() == 1 {
                    func.dfg.value_type(results[0])
                } else {
                    crate::ir::types::INVALID
                };

                let load_mem_state = self.alias_analysis.get_state_for_load(inst);
                let is_readonly_load = match func.dfg[inst] {
                    InstructionData::Load {
                        opcode: Opcode::Load,
                        flags,
                        ..
                    } => flags.readonly() && flags.notrap(),
                    _ => false,
                };

                // Create the egraph node.
                let op = InstructionImms::from(&func.dfg[inst]);
                let opcode = op.opcode();
                let srcloc = func.srclocs[inst];
                let arity = u16::try_from(results.len())
                    .expect("More than 2^16 results from an instruction");

                let node = if is_readonly_load {
                    self.stats.node_created += 1;
                    self.stats.node_pure += 1;
                    Node::Pure {
                        op,
                        args,
                        ty,
                        arity,
                    }
                } else if let Some(load_mem_state) = load_mem_state {
                    let addr = args.as_slice(&self.node_ctx.args)[0];
                    trace!("load at inst {} has mem state {:?}", inst, load_mem_state);
                    self.stats.node_created += 1;
                    self.stats.node_load += 1;
                    Node::Load {
                        op,
                        ty,
                        addr,
                        mem_state: load_mem_state,
                        srcloc,
                    }
                } else if has_side_effect(func, inst) || opcode.can_load() {
                    self.stats.node_created += 1;
                    self.stats.node_inst += 1;
                    Node::Inst {
                        op,
                        args,
                        ty,
                        arity,
                        srcloc,
                        loop_level,
                    }
                } else {
                    self.stats.node_created += 1;
                    self.stats.node_pure += 1;
                    Node::Pure {
                        op,
                        args,
                        ty,
                        arity,
                    }
                };
                let dedup_needed = self.node_ctx.needs_dedup(&node);
                let is_pure = matches!(node, Node::Pure { .. });

                let mut id = self.egraph.add(node, &mut self.node_ctx);

                if dedup_needed {
                    self.stats.node_dedup_query += 1;
                    match id {
                        NewOrExisting::New(_) => {
                            self.stats.node_dedup_miss += 1;
                        }
                        NewOrExisting::Existing(_) => {
                            self.stats.node_dedup_hit += 1;
                        }
                    }
                }

                if opcode == Opcode::Store {
                    let store_data_ty = func.dfg.value_type(func.dfg.inst_args(inst)[0]);
                    self.store_nodes.insert(inst, (store_data_ty, id.get()));
                    self.stats.store_map_insert += 1;
                }

                // Loads that did not already merge into an existing
                // load: try to forward from a store (store-to-load
                // forwarding).
                if let NewOrExisting::New(new_id) = id {
                    if load_mem_state.is_some() {
                        let opt_id = crate::opts::store_to_load(new_id, self);
                        trace!("store_to_load: {} -> {}", new_id, opt_id);
                        if opt_id != new_id {
                            id = NewOrExisting::Existing(opt_id);
                        }
                    }
                }

                // Now either optimize (for new pure nodes), or add to
                // the side-effecting list (for all other new nodes).
                let id = match id {
                    NewOrExisting::Existing(id) => id,
                    NewOrExisting::New(id) if is_pure => {
                        // Apply all optimization rules immediately; the
                        // aegraph (acyclic egraph) works best when we do
                        // this so all uses pick up the eclass with all
                        // possible enodes.
                        crate::opts::optimize_eclass(id, self)
                    }
                    NewOrExisting::New(id) => {
                        self.side_effect_ids.push(id);
                        self.stats.side_effect_nodes += 1;
                        id
                    }
                };

                // Create results and save in Value->Id map.
                match results {
                    &[] => {}
                    &[one_result] => {
                        trace!("build: value {} -> id {}", one_result, id);
                        value_to_id.insert(one_result, id);
                    }
                    many_results => {
                        debug_assert!(many_results.len() > 1);
                        for (i, &result) in many_results.iter().enumerate() {
                            let ty = func.dfg.value_type(result);
                            let projection = self
                                .egraph
                                .add(
                                    Node::Result {
                                        value: id,
                                        result: i,
                                        ty,
                                    },
                                    &mut self.node_ctx,
                                )
                                .get();
                            self.stats.node_created += 1;
                            self.stats.node_result += 1;
                            trace!("build: value {} -> id {}", result, projection);
                            value_to_id.insert(result, projection);
                        }
                    }
                }
            }

            let side_effect_end =
                u32::try_from(self.side_effect_ids.len()).expect("Overflow in side-effect count");
            let side_effect_range = side_effect_start..side_effect_end;
            self.side_effects[block] = side_effect_range;
        }
    }

    /// Scoped elaboration: compute a final ordering of op computation
    /// for each block and replace the given Func body.
    ///
    /// This works in concert with the domtree. We do a preorder
    /// traversal of the domtree, tracking a scoped map from Id to
    /// (new) Value. The map's scopes correspond to levels in the
    /// domtree.
    ///
    /// At each block, we iterate forward over the side-effecting
    /// eclasses, and recursively generate their arg eclasses, then
    /// emit the ops themselves.
    ///
    /// To use an eclass in a given block, we first look it up in the
    /// scoped map, and get the Value if already present. If not, we
    /// need to generate it. We emit the extracted enode for this
    /// eclass after recursively generating its args. Eclasses are
    /// thus computed "as late as possible", but then memoized into
    /// the Id-to-Value map and available to all dominated blocks and
    /// for the rest of this block. (This subsumes GVN.)
    pub fn elaborate(&mut self, func: &mut Function) {
        let mut elab = Elaborator::new(
            func,
            self.domtree,
            self.loop_analysis,
            &self.egraph,
            &self.node_ctx,
            &self.remat_ids,
            &mut self.stats,
        );
        elab.elaborate(
            |block| {
                let blockparam_range = self.blockparams[block].clone();
                &self.blockparam_ids_tys
                    [blockparam_range.start as usize..blockparam_range.end as usize]
            },
            |block| {
                let side_effect_range = self.side_effects[block].clone();
                &self.side_effect_ids
                    [side_effect_range.start as usize..side_effect_range.end as usize]
            },
        );
    }
}

/// State for egraph analysis that computes all needed properties.
pub(crate) struct Analysis;

/// Analysis results for each eclass id.
#[derive(Clone, Debug)]
pub(crate) struct AnalysisValue {
    pub(crate) loop_level: LoopLevel,
}

impl Default for AnalysisValue {
    fn default() -> Self {
        Self {
            loop_level: LoopLevel::root(),
        }
    }
}

impl cranelift_egraph::Analysis for Analysis {
    type L = NodeCtx;
    type Value = AnalysisValue;

    fn for_node(
        &self,
        ctx: &NodeCtx,
        n: &Node,
        values: &SecondaryMap<Id, AnalysisValue>,
    ) -> AnalysisValue {
        let loop_level = match n {
            &Node::Pure { ref args, .. } => args
                .as_slice(&ctx.args)
                .iter()
                .map(|&arg| values[arg].loop_level)
                .max()
                .unwrap_or(LoopLevel::root()),
            &Node::Load { addr, .. } => values[addr].loop_level,
            &Node::Result { value, .. } => values[value].loop_level,
            &Node::Inst { loop_level, .. } | &Node::Param { loop_level, .. } => loop_level,
        };

        AnalysisValue { loop_level }
    }

    fn meet(&self, _ctx: &NodeCtx, v1: &AnalysisValue, v2: &AnalysisValue) -> AnalysisValue {
        AnalysisValue {
            loop_level: std::cmp::max(v1.loop_level, v2.loop_level),
        }
    }
}
