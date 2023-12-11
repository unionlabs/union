//! Optimization driver using ISLE rewrite rules on an egraph.

use crate::egraph::Analysis;
use crate::egraph::FuncEGraph;
pub use crate::egraph::{Node, NodeCtx};
use crate::ir::condcodes;
pub use crate::ir::condcodes::{FloatCC, IntCC};
pub use crate::ir::immediates::{Ieee32, Ieee64, Imm64, Offset32, Uimm32, Uimm64, Uimm8};
pub use crate::ir::types::*;
pub use crate::ir::{
    dynamic_to_fixed, AtomicRmwOp, Block, Constant, DynamicStackSlot, FuncRef, GlobalValue, Heap,
    HeapImm, Immediate, InstructionImms, JumpTable, MemFlags, Opcode, StackSlot, Table, TrapCode,
    Type, Value,
};
use crate::isle_common_prelude_methods;
use crate::machinst::isle::*;
use crate::trace;
pub use cranelift_egraph::{Id, NewOrExisting, NodeIter};
use cranelift_entity::{EntityList, EntityRef};
use smallvec::SmallVec;
use std::marker::PhantomData;

pub type IdArray = EntityList<Id>;
#[allow(dead_code)]
pub type Unit = ();
pub type Range = (usize, usize);

pub type ConstructorVec<T> = SmallVec<[T; 8]>;

mod generated_code;
use generated_code::ContextIter;

struct IsleContext<'a, 'b> {
    egraph: &'a mut FuncEGraph<'b>,
}

const REWRITE_LIMIT: usize = 5;

pub fn optimize_eclass<'a>(id: Id, egraph: &mut FuncEGraph<'a>) -> Id {
    trace!("running rules on eclass {}", id.index());
    egraph.stats.rewrite_rule_invoked += 1;

    if egraph.rewrite_depth > REWRITE_LIMIT {
        egraph.stats.rewrite_depth_limit += 1;
        return id;
    }
    egraph.rewrite_depth += 1;

    // Find all possible rewrites and union them in, returning the
    // union.
    let mut ctx = IsleContext { egraph };
    let optimized_ids = generated_code::constructor_simplify(&mut ctx, id);
    let mut union_id = id;
    if let Some(mut ids) = optimized_ids {
        while let Some(new_id) = ids.next(&mut ctx) {
            if ctx.egraph.subsume_ids.contains(&new_id) {
                trace!(" -> eclass {} subsumes {}", new_id, id);
                ctx.egraph.stats.node_subsume += 1;
                // Merge in the unionfind so canonicalization still
                // works, but take *only* the subsuming ID, and break
                // now.
                ctx.egraph.egraph.unionfind.union(union_id, new_id);
                union_id = new_id;
                break;
            }
            ctx.egraph.stats.node_union += 1;
            let old_union_id = union_id;
            union_id = ctx
                .egraph
                .egraph
                .union(&ctx.egraph.node_ctx, union_id, new_id);
            trace!(
                " -> union eclass {} with {} to get {}",
                new_id,
                old_union_id,
                union_id
            );
        }
    }
    trace!(" -> optimize {} got {}", id, union_id);
    ctx.egraph.rewrite_depth -= 1;
    union_id
}

pub(crate) fn store_to_load<'a>(id: Id, egraph: &mut FuncEGraph<'a>) -> Id {
    // Note that we only examine the latest enode in the eclass: opts
    // are invoked for every new enode added to an eclass, so
    // traversing the whole eclass would be redundant.
    let load_key = egraph.egraph.classes[id].get_node().unwrap();
    if let Node::Load {
        op:
            InstructionImms::Load {
                opcode: Opcode::Load,
                offset: load_offset,
                ..
            },
        ty: load_ty,
        addr: load_addr,
        mem_state,
        ..
    } = load_key.node(&egraph.egraph.nodes)
    {
        if let Some(store_inst) = mem_state.as_store() {
            trace!(" -> got load op for id {}", id);
            if let Some((store_ty, store_id)) = egraph.store_nodes.get(&store_inst) {
                trace!(" -> got store id: {} ty: {}", store_id, store_ty);
                let store_key = egraph.egraph.classes[*store_id].get_node().unwrap();
                if let Node::Inst {
                    op:
                        InstructionImms::Store {
                            opcode: Opcode::Store,
                            offset: store_offset,
                            ..
                        },
                    args: store_args,
                    ..
                } = store_key.node(&egraph.egraph.nodes)
                {
                    let store_args = store_args.as_slice(&egraph.node_ctx.args);
                    let store_data = store_args[0];
                    let store_addr = store_args[1];
                    if *load_offset == *store_offset
                        && *load_ty == *store_ty
                        && egraph.egraph.unionfind.equiv_id_mut(*load_addr, store_addr)
                    {
                        trace!(" -> same offset, type, address; forwarding");
                        egraph.stats.store_to_load_forward += 1;
                        return store_data;
                    }
                }
            }
        }
    }

    id
}

struct NodesEtorIter<'a, 'b>
where
    'b: 'a,
{
    root: Id,
    iter: NodeIter<NodeCtx, Analysis>,
    _phantom1: PhantomData<&'a ()>,
    _phantom2: PhantomData<&'b ()>,
}

impl<'a, 'b> generated_code::ContextIter for NodesEtorIter<'a, 'b>
where
    'b: 'a,
{
    type Context = IsleContext<'a, 'b>;
    type Output = (Type, InstructionImms, IdArray);

    fn next(&mut self, ctx: &mut IsleContext<'a, 'b>) -> Option<Self::Output> {
        while let Some(node) = self.iter.next(&ctx.egraph.egraph) {
            trace!("iter from root {}: node {:?}", self.root, node);
            match node {
                Node::Pure {
                    op,
                    args,
                    ty,
                    arity,
                }
                | Node::Inst {
                    op,
                    args,
                    ty,
                    arity,
                    ..
                } if *arity == 1 => {
                    return Some((*ty, op.clone(), args.clone()));
                }
                _ => {}
            }
        }
        None
    }
}

impl<'a, 'b> generated_code::Context for IsleContext<'a, 'b> {
    isle_common_prelude_methods!();

    fn eclass_type(&mut self, eclass: Id) -> Option<Type> {
        let mut iter = self.egraph.egraph.enodes(eclass);
        while let Some(node) = iter.next(&self.egraph.egraph) {
            match node {
                &Node::Pure { ty, arity, .. } | &Node::Inst { ty, arity, .. } if arity == 1 => {
                    return Some(ty);
                }
                &Node::Load { ty, .. } => return Some(ty),
                &Node::Result { ty, .. } => return Some(ty),
                &Node::Param { ty, .. } => return Some(ty),
                _ => {}
            }
        }
        None
    }

    fn at_loop_level(&mut self, eclass: Id) -> (u8, Id) {
        (
            self.egraph.egraph.analysis_value(eclass).loop_level.level() as u8,
            eclass,
        )
    }

    type enodes_etor_iter = NodesEtorIter<'a, 'b>;

    fn enodes_etor(&mut self, eclass: Id) -> Option<NodesEtorIter<'a, 'b>> {
        Some(NodesEtorIter {
            root: eclass,
            iter: self.egraph.egraph.enodes(eclass),
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        })
    }

    fn pure_enode_ctor(&mut self, ty: Type, op: &InstructionImms, args: IdArray) -> Id {
        let op = op.clone();
        match self.egraph.egraph.add(
            Node::Pure {
                op,
                args,
                ty,
                arity: 1,
            },
            &mut self.egraph.node_ctx,
        ) {
            NewOrExisting::New(id) => {
                self.egraph.stats.node_created += 1;
                self.egraph.stats.node_pure += 1;
                self.egraph.stats.node_ctor_created += 1;
                optimize_eclass(id, self.egraph)
            }
            NewOrExisting::Existing(id) => {
                self.egraph.stats.node_ctor_deduped += 1;
                id
            }
        }
    }

    fn id_array_0_etor(&mut self, arg0: IdArray) -> Option<()> {
        let values = arg0.as_slice(&self.egraph.node_ctx.args);
        if values.len() == 0 {
            Some(())
        } else {
            None
        }
    }

    fn id_array_0_ctor(&mut self) -> IdArray {
        EntityList::default()
    }

    fn id_array_1_etor(&mut self, arg0: IdArray) -> Option<Id> {
        let values = arg0.as_slice(&self.egraph.node_ctx.args);
        if values.len() == 1 {
            Some(values[0])
        } else {
            None
        }
    }

    fn id_array_1_ctor(&mut self, arg0: Id) -> IdArray {
        EntityList::from_iter([arg0].into_iter(), &mut self.egraph.node_ctx.args)
    }

    fn id_array_2_etor(&mut self, arg0: IdArray) -> Option<(Id, Id)> {
        let values = arg0.as_slice(&self.egraph.node_ctx.args);
        if values.len() == 2 {
            Some((values[0], values[1]))
        } else {
            None
        }
    }

    fn id_array_2_ctor(&mut self, arg0: Id, arg1: Id) -> IdArray {
        EntityList::from_iter([arg0, arg1].into_iter(), &mut self.egraph.node_ctx.args)
    }

    fn id_array_3_etor(&mut self, arg0: IdArray) -> Option<(Id, Id, Id)> {
        let values = arg0.as_slice(&self.egraph.node_ctx.args);
        if values.len() == 3 {
            Some((values[0], values[1], values[2]))
        } else {
            None
        }
    }

    fn id_array_3_ctor(&mut self, arg0: Id, arg1: Id, arg2: Id) -> IdArray {
        EntityList::from_iter(
            [arg0, arg1, arg2].into_iter(),
            &mut self.egraph.node_ctx.args,
        )
    }

    fn remat(&mut self, id: Id) -> Id {
        trace!("remat: {}", id);
        self.egraph.remat_ids.insert(id);
        id
    }

    fn subsume(&mut self, id: Id) -> Id {
        trace!("subsume: {}", id);
        self.egraph.subsume_ids.insert(id);
        id
    }
}
