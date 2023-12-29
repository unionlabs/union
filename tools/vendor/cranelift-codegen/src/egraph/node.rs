//! Node definition for EGraph representation.

use super::PackedMemoryState;
use crate::ir::{Block, DataFlowGraph, InstructionImms, Opcode, RelSourceLoc, Type};
use crate::loop_analysis::LoopLevel;
use cranelift_egraph::{CtxEq, CtxHash, Id, Language, UnionFind};
use cranelift_entity::{EntityList, ListPool};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum Node {
    /// A blockparam. Effectively an input/root; does not refer to
    /// predecessors' branch arguments, because this would create
    /// cycles.
    Param {
        /// CLIF block this param comes from.
        block: Block,
        /// Index of blockparam within block.
        index: u32,
        /// Type of the value.
        ty: Type,
        /// The loop level of this Param.
        loop_level: LoopLevel,
    },
    /// A CLIF instruction that is pure (has no side-effects). Not
    /// tied to any location; we will compute a set of locations at
    /// which to compute this node during lowering back out of the
    /// egraph.
    Pure {
        /// The instruction data, without SSA values.
        op: InstructionImms,
        /// eclass arguments to the operator.
        args: EntityList<Id>,
        /// Type of result, if one.
        ty: Type,
        /// Number of results.
        arity: u16,
    },
    /// A CLIF instruction that has side-effects or is otherwise not
    /// representable by `Pure`.
    Inst {
        /// The instruction data, without SSA values.
        op: InstructionImms,
        /// eclass arguments to the operator.
        args: EntityList<Id>,
        /// Type of result, if one.
        ty: Type,
        /// Number of results.
        arity: u16,
        /// The source location to preserve.
        srcloc: RelSourceLoc,
        /// The loop level of this Inst.
        loop_level: LoopLevel,
    },
    /// A projection of one result of an `Inst` or `Pure`.
    Result {
        /// `Inst` or `Pure` node.
        value: Id,
        /// Index of the result we want.
        result: usize,
        /// Type of the value.
        ty: Type,
    },

    /// A load instruction. Nominally a side-effecting `Inst` (and
    /// included in the list of side-effecting roots so it will always
    /// be elaborated), but represented as a distinct kind of node so
    /// that we can leverage deduplication to do
    /// redundant-load-elimination for free (and make store-to-load
    /// forwarding much easier).
    Load {
        // -- identity depends on:
        /// The original load operation. Must have one argument, the
        /// address.
        op: InstructionImms,
        /// The type of the load result.
        ty: Type,
        /// Address argument. Actual address has an offset, which is
        /// included in `op` (and thus already considered as part of
        /// the key).
        addr: Id,
        /// The abstract memory state that this load accesses.
        mem_state: PackedMemoryState,

        // -- not included in dedup key:
        /// Source location, for traps. Not included in Eq/Hash.
        srcloc: RelSourceLoc,
    },
}

impl Node {
    pub(crate) fn is_non_pure(&self) -> bool {
        match self {
            Node::Inst { .. } | Node::Load { .. } => true,
            _ => false,
        }
    }
}

/// Shared pools for type and id lists in nodes.
pub struct NodeCtx {
    /// Arena for arg eclass-ID lists.
    pub args: ListPool<Id>,
}

impl NodeCtx {
    pub(crate) fn with_capacity_for_dfg(dfg: &DataFlowGraph) -> Self {
        let n_args = dfg.value_lists.capacity();
        Self {
            args: ListPool::with_capacity(n_args),
        }
    }
}

impl NodeCtx {
    fn ids_eq(&self, a: &EntityList<Id>, b: &EntityList<Id>, uf: &mut UnionFind) -> bool {
        let a = a.as_slice(&self.args);
        let b = b.as_slice(&self.args);
        a.len() == b.len() && a.iter().zip(b.iter()).all(|(&a, &b)| uf.equiv_id_mut(a, b))
    }

    fn hash_ids<H: Hasher>(&self, a: &EntityList<Id>, hash: &mut H, uf: &mut UnionFind) {
        let a = a.as_slice(&self.args);
        for &id in a {
            uf.hash_id_mut(hash, id);
        }
    }
}

impl CtxEq<Node, Node> for NodeCtx {
    fn ctx_eq(&self, a: &Node, b: &Node, uf: &mut UnionFind) -> bool {
        match (a, b) {
            (
                &Node::Param {
                    block,
                    index,
                    ty,
                    loop_level: _,
                },
                &Node::Param {
                    block: other_block,
                    index: other_index,
                    ty: other_ty,
                    loop_level: _,
                },
            ) => block == other_block && index == other_index && ty == other_ty,
            (
                &Node::Result { value, result, ty },
                &Node::Result {
                    value: other_value,
                    result: other_result,
                    ty: other_ty,
                },
            ) => uf.equiv_id_mut(value, other_value) && result == other_result && ty == other_ty,
            (
                &Node::Pure {
                    ref op,
                    ref args,
                    ty,
                    arity: _,
                },
                &Node::Pure {
                    op: ref other_op,
                    args: ref other_args,
                    ty: other_ty,
                    arity: _,
                },
            ) => *op == *other_op && self.ids_eq(args, other_args, uf) && ty == other_ty,
            (
                &Node::Inst { ref args, .. },
                &Node::Inst {
                    args: ref other_args,
                    ..
                },
            ) => self.ids_eq(args, other_args, uf),
            (
                &Node::Load {
                    ref op,
                    ty,
                    addr,
                    mem_state,
                    ..
                },
                &Node::Load {
                    op: ref other_op,
                    ty: other_ty,
                    addr: other_addr,
                    mem_state: other_mem_state,
                    // Explicitly exclude: `inst` and `srcloc`. We
                    // want loads to merge if identical in
                    // opcode/offset, address expression, and last
                    // store (this does implicit
                    // redundant-load-elimination.)
                    //
                    // Note however that we *do* include `ty` (the
                    // type) and match on that: we otherwise would
                    // have no way of disambiguating loads of
                    // different widths to the same address.
                    ..
                },
            ) => {
                op == other_op
                    && ty == other_ty
                    && uf.equiv_id_mut(addr, other_addr)
                    && mem_state == other_mem_state
            }
            _ => false,
        }
    }
}

impl CtxHash<Node> for NodeCtx {
    fn ctx_hash(&self, value: &Node, uf: &mut UnionFind) -> u64 {
        let mut state = crate::fx::FxHasher::default();
        std::mem::discriminant(value).hash(&mut state);
        match value {
            &Node::Param {
                block,
                index,
                ty: _,
                loop_level: _,
            } => {
                block.hash(&mut state);
                index.hash(&mut state);
            }
            &Node::Result {
                value,
                result,
                ty: _,
            } => {
                uf.hash_id_mut(&mut state, value);
                result.hash(&mut state);
            }
            &Node::Pure {
                ref op,
                ref args,
                ty,
                arity: _,
            } => {
                op.hash(&mut state);
                self.hash_ids(args, &mut state, uf);
                ty.hash(&mut state);
            }
            &Node::Inst { ref args, .. } => {
                self.hash_ids(args, &mut state, uf);
            }
            &Node::Load {
                ref op,
                ty,
                addr,
                mem_state,
                ..
            } => {
                op.hash(&mut state);
                ty.hash(&mut state);
                uf.hash_id_mut(&mut state, addr);
                mem_state.hash(&mut state);
            }
        }

        state.finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Cost(u32);
impl Cost {
    pub(crate) fn at_level(&self, loop_level: LoopLevel) -> Cost {
        let loop_level = std::cmp::min(2, loop_level.level());
        let multiplier = 1u32 << ((10 * loop_level) as u32);
        Cost(self.0.saturating_mul(multiplier)).finite()
    }

    pub(crate) fn infinity() -> Cost {
        // 2^32 - 1 is, uh, pretty close to infinite... (we use `Cost`
        // only for heuristics and always saturate so this suffices!)
        Cost(u32::MAX)
    }

    pub(crate) fn zero() -> Cost {
        Cost(0)
    }

    /// Clamp this cost at a "finite" value. Can be used in
    /// conjunction with saturating ops to avoid saturating into
    /// `infinity()`.
    fn finite(self) -> Cost {
        Cost(std::cmp::min(u32::MAX - 1, self.0))
    }
}

impl std::default::Default for Cost {
    fn default() -> Cost {
        Cost::zero()
    }
}

impl std::ops::Add<Cost> for Cost {
    type Output = Cost;
    fn add(self, other: Cost) -> Cost {
        Cost(self.0.saturating_add(other.0)).finite()
    }
}

pub(crate) fn op_cost(op: &InstructionImms) -> Cost {
    match op.opcode() {
        // Constants.
        Opcode::Iconst | Opcode::F32const | Opcode::F64const => Cost(0),
        // Extends/reduces.
        Opcode::Uextend | Opcode::Sextend | Opcode::Ireduce | Opcode::Iconcat | Opcode::Isplit => {
            Cost(1)
        }
        // "Simple" arithmetic.
        Opcode::Iadd
        | Opcode::Isub
        | Opcode::Band
        | Opcode::BandNot
        | Opcode::Bor
        | Opcode::BorNot
        | Opcode::Bxor
        | Opcode::BxorNot
        | Opcode::Bnot => Cost(2),
        // Everything else.
        _ => Cost(3),
    }
}

impl Language for NodeCtx {
    type Node = Node;

    fn children<'a>(&'a self, node: &'a Node) -> &'a [Id] {
        match node {
            Node::Param { .. } => &[],
            Node::Pure { args, .. } | Node::Inst { args, .. } => args.as_slice(&self.args),
            Node::Load { addr, .. } => std::slice::from_ref(addr),
            Node::Result { value, .. } => std::slice::from_ref(value),
        }
    }

    fn children_mut<'a>(&'a mut self, node: &'a mut Node) -> &'a mut [Id] {
        match node {
            Node::Param { .. } => &mut [],
            Node::Pure { args, .. } | Node::Inst { args, .. } => args.as_mut_slice(&mut self.args),
            Node::Load { addr, .. } => std::slice::from_mut(addr),
            Node::Result { value, .. } => std::slice::from_mut(value),
        }
    }

    fn needs_dedup(&self, node: &Node) -> bool {
        match node {
            Node::Pure { .. } | Node::Load { .. } => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn node_size() {
        use super::*;
        assert_eq!(std::mem::size_of::<InstructionImms>(), 16);
        assert_eq!(std::mem::size_of::<Node>(), 32);
    }
}
