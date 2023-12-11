//! # ægraph (aegraph, or acyclic e-graph) implementation.
//!
//! An aegraph is a form of e-graph. We will first describe the
//! e-graph, then the aegraph as a slightly less powerful but highly
//! optimized variant of it.
//!
//! The main goal of this library is to be explicitly memory-efficient
//! and light on allocations. We need to be as fast and as small as
//! possible in order to minimize impact on compile time in a
//! production compiler.
//!
//! ## The e-graph
//!
//! An e-graph, or equivalence graph, is a kind of node-based
//! intermediate representation (IR) data structure that consists of
//! *eclasses* and *enodes*. An eclass contains one or more enodes;
//! semantically an eclass is like a value, and an enode is one way to
//! compute that value. If several enodes are in one eclass, the data
//! structure is asserting that any of these enodes, if evaluated,
//! would produce the value.
//!
//! An e-graph also contains a deduplicating hash-map of nodes, so if
//! the user creates the same e-node more than once, they get the same
//! e-class ID.
//!
//! In the usual use-case, an e-graph is used to build a sea-of-nodes
//! IR for a function body or other expression-based code, and then
//! *rewrite rules* are applied to the e-graph. Each rewrite
//! potentially introduces a new e-node that is equivalent to an
//! existing e-node, and then unions the two e-nodes' classes
//! together.
//!
//! In the trivial case this results in an e-class containing a series
//! of e-nodes that are newly added -- all known forms of an
//! expression -- but Note how if a rewrite rule rewrites into an
//! existing e-node (discovered via deduplication), rewriting can
//! result in unioning of two e-classes that have existed for some
//! time.
//!
//! An e-graph's enodes refer to *classes* for their arguments, rather
//! than other nodes directly. This is key to the ability of an
//! e-graph to canonicalize: when two e-classes that are already used
//! as arguments by other e-nodes are unioned, all e-nodes that refer
//! to those e-classes are themselves re-canonicalized. This can
//! result in "cascading" unioning of eclasses, in a process that
//! discovers the transitive implications of all individual
//! equalities. This process is known as "equality saturation".
//!
//! ## The acyclic e-graph (aegraph)
//!
//! An e-graph is powerful, but it can also be expensive to build and
//! saturate: there are often many different forms an expression can
//! take (because many different rewrites are possible), and cascading
//! canonicalization requires heavyweight data structure bookkeeping
//! that is expensive to maintain.
//!
//! This crate introduces the aegraph: an acyclic e-graph. This data
//! structure stores an e-class as an *immutable persistent data
//! structure*. An id can refer to some *level* of an eclass: a
//! snapshot of the nodes in the eclass at one point in time. The
//! nodes referred to by this id never change, though the eclass may
//! grow later.
//!
//! A *union* is also an operation that creates a new eclass id: the
//! original eclass IDs refer to the original eclass contents, while
//! the id resulting from the `union()` operation refers to an eclass
//! that has all nodes.
//!
//! In order to allow for adequate canonicalization, an enode normally
//! stores the *latest* eclass id for each argument, but computes
//! hashes and equality using a *canonical* eclass id. We define such
//! a canonical id with a union-find data structure, just as for a
//! traditional e-graph. It is normally the lowest id referring to
//! part of the eclass.
//!
//! The persistent/immutable nature of this data structure yields one
//! extremely important property: it is acyclic! This simplifies
//! operation greatly:
//!
//! - When "elaborating" out of the e-graph back to linearized code,
//!   so that we can generate machine code, we do not need to break
//!   cycles. A given enode cannot indirectly refer back to itself.
//!
//! - When applying rewrite rules, the nodes visible from a given id
//!   for an eclass never change. This means that we only need to
//!   apply rewrite rules at that node id *once*.
//!
//! ## Data Structure and Example
//!
//! Each eclass id refers to a table entry ("eclass node", which is
//! different than an "enode") that can be one of:
//!
//! - A single enode;
//! - An enode and an earlier eclass id it is appended to (a "child"
//!   eclass node);
//! - A "union node" with two earlier eclass ids.
//!
//! Building the aegraph consists solely of adding new entries to the
//! end of this table of eclass nodes. An enode referenced from any
//! given eclass node can only refer to earlier eclass ids.
//!
//! For example, consider the following eclass table:
//!
//! ```plain
//!
//!    eclass/enode table
//!
//!     eclass1    iconst(1)
//!     eclass2    blockparam(block0, 0)
//!     eclass3    iadd(eclass1, eclass2)
//! ```
//!
//! This represents the expression `iadd(blockparam(block0, 0),
//! iconst(1))` (as the sole enode for eclass3).
//!
//! Now, say that as we further build the function body, we add
//! another enode `iadd(eclass3, iconst(1))`. The `iconst(1)` will be
//! deduplicated to `eclass1`, and the toplevel `iadd` will become its
//! own new eclass (`eclass4`).
//!
//! ```plain
//!     eclass4    iadd(eclass3, eclass1)
//! ```
//!
//! Now we apply our body of rewrite rules, and these results can
//! combine `x + 1 + 1` into `x + 2`; so we get:
//!
//! ```plain
//!     eclass5    iconst(2)
//!     eclass6    union(iadd(eclass2, eclass5), eclass4)
//! ```
//!
//! Note that we added the nodes for the new expression, and then we
//! union'd it with the earlier `eclass4`. Logically this represents a
//! single eclass that contains two nodes -- the `x + 1 + 1` and `x +
//! 2` representations -- and the *latest* id for the eclass,
//! `eclass6`, can reach all nodes in the eclass (here the node stored
//! in `eclass6` and the earlier one in `elcass4`).
//!
//! ## aegraph vs. egraph
//!
//! Where does an aegraph fall short of an e-graph -- or in other
//! words, why maintain the data structures to allow for full
//! (re)canonicalization at all, with e.g. parent pointers to
//! recursively update parents?
//!
//! This question deserves further study, but right now, it appears
//! that the difference is limited to a case like the following:
//!
//! - expression E1 is interned into the aegraph.
//! - expression E2 is interned into the aegraph. It uses E1 as an
//!   argument to one or more operators, and so refers to the
//!   (currently) latest id for E1.
//! - expression E3 is interned into the aegraph. A rewrite rule fires
//!   that unions E3 with E1.
//!
//! In an e-graph, the last action would trigger a re-canonicalization
//! of all "parents" (users) of E1; so E2 would be re-canonicalized
//! using an id that represents the union of E1 and E3. At
//! code-generation time, E2 could choose to use a value computed by
//! either E1's or E3's operator. In an aegraph, this is not the case:
//! E2's e-class and e-nodes are immutable once created, so E2 refers
//! only to E1's representation of the value (a "slice" of the whole
//! e-class).
//!
//! While at first this sounds quite limiting, there actually appears
//! to be a nice mutually-beneficial interaction with the immediate
//! application of rewrite rules: by applying all rewrites we know
//! about right when E1 is interned, E2 can refer to the best version
//! when it is created. The above scenario only leads to a missed
//! optimization if:
//!
//! - a rewrite rule exists from E3 to E1, but not E1 to E3; and
//! - E3 is *cheaper* than E1.
//!
//! Or in other words, this only matters if there is a rewrite rule
//! that rewrites into a more expensive direction. This is unlikely
//! for the sorts of rewrite rules we plan to write; it may matter
//! more if many possible equalities are expressed, such as
//! associativity, commutativity, etc.
//!
//! Note that the above represents the best of our understanding, but
//! there may be cases we have missed; a more complete examination of
//! this question would involve building a full equality saturation
//! loop on top of the (a)egraph in this crate, and testing with many
//! benchmarks to see if it makes any difference.
//!
//! ## Rewrite Rules (FLAX: Fast Localized Aegraph eXpansion)
//!
//! The most common use of an e-graph or aegraph is to serve as the IR
//! for a compiler. In this use-case, we usually wish to transform the
//! program using a body of rewrite rules that represent valid
//! transformations (equivalent and hopefully simpler ways of
//! computing results). An aegraph supports applying rules in a fairly
//! straightforward way: whenever a new eclass entry is added to the
//! table, we invoke a toplevel "apply all rewrite rules" entry
//! point. This entry point creates new nodes as needed, and when
//! done, unions the rewritten nodes with the original. We thus
//! *immediately* expand a new value into all of its representations.
//!
//! This immediate expansion stands in contrast to a traditional
//! "equality saturation" e-egraph system, in which it is usually best
//! to apply rules in batches and then fix up the
//! canonicalization. This approach was introduced in the `egg`
//! e-graph engine [^1]. We call our system FLAX (because flax is an
//! alternative to egg): Fast Localized Aegraph eXpansion.
//!
//! The reason that this is possible in an aegraph but not
//! (efficiently, at least) in a traditional e-graph is that the data
//! structure nodes are immutable once created: an eclass id will
//! always refer to a fixed set of enodes. There is no
//! recanonicalizing of eclass arguments as they union; but also this
//! is not usually necessary, because args will have already been
//! processed and eagerly rewritten as well. In other words, eager
//! rewriting and the immutable data structure mutually allow each
//! other to be practical; both work together.
//!
//! [^1]: M Willsey, C Nandi, Y R Wang, O Flatt, Z Tatlock, P
//!       Panchekha. "egg: Fast and Flexible Equality Saturation." In
//!       POPL 2021. <https://dl.acm.org/doi/10.1145/3434304>

use cranelift_entity::PrimaryMap;
use cranelift_entity::{entity_impl, packed_option::ReservedValue, SecondaryMap};
use smallvec::{smallvec, SmallVec};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

mod bumpvec;
mod ctxhash;
mod unionfind;

pub use bumpvec::{BumpArena, BumpSlice, BumpVec};
pub use ctxhash::{CtxEq, CtxHash, CtxHashMap, Entry};
pub use unionfind::UnionFind;

/// An eclass ID.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(u32);
entity_impl!(Id, "eclass");

impl Id {
    pub fn invalid() -> Id {
        Self::reserved_value()
    }
}
impl std::default::Default for Id {
    fn default() -> Self {
        Self::invalid()
    }
}

/// A trait implemented by all "languages" (types that can be enodes).
pub trait Language: CtxEq<Self::Node, Self::Node> + CtxHash<Self::Node> {
    type Node: Debug;
    fn children<'a>(&'a self, node: &'a Self::Node) -> &'a [Id];
    fn children_mut<'a>(&'a mut self, ctx: &'a mut Self::Node) -> &'a mut [Id];
    fn needs_dedup(&self, node: &Self::Node) -> bool;
}

/// A trait that allows the aegraph to compute a property of each
/// node as it is created.
pub trait Analysis {
    type L: Language;
    type Value: Clone + Default;
    fn for_node(
        &self,
        ctx: &Self::L,
        n: &<Self::L as Language>::Node,
        values: &SecondaryMap<Id, Self::Value>,
    ) -> Self::Value;
    fn meet(&self, ctx: &Self::L, v1: &Self::Value, v2: &Self::Value) -> Self::Value;
}

/// Conditionally-compiled trace-log macro. (Borrowed from
/// `cranelift-codegen`; it's not worth factoring out a common
/// subcrate for this.)
#[macro_export]
macro_rules! trace {
    ($($tt:tt)*) => {
        if cfg!(feature = "trace-log") {
            ::log::trace!($($tt)*);
        }
    };
}

/// An egraph.
pub struct EGraph<L: Language, A: Analysis<L = L>> {
    /// Node-allocation arena.
    pub nodes: Vec<L::Node>,
    /// Hash-consing map from Nodes to eclass IDs.
    node_map: CtxHashMap<NodeKey, Id>,
    /// Eclass definitions. Each eclass consists of an enode, and
    /// child pointer to the rest of the eclass.
    pub classes: PrimaryMap<Id, EClass>,
    /// Union-find for canonical ID generation. This lets us name an
    /// eclass with a canonical ID that is the same for all
    /// generations of the class.
    pub unionfind: UnionFind,
    /// Analysis and per-node state.
    pub analysis: Option<(A, SecondaryMap<Id, A::Value>)>,
}

/// A reference to a node.
#[derive(Clone, Copy, Debug)]
pub struct NodeKey {
    index: u32,
}

impl NodeKey {
    fn from_node_idx(node_idx: usize) -> NodeKey {
        NodeKey {
            index: u32::try_from(node_idx).unwrap(),
        }
    }

    /// Get the node for this NodeKey, given the `nodes` from the
    /// appropriate `EGraph`.
    pub fn node<'a, N>(&self, nodes: &'a [N]) -> &'a N {
        &nodes[self.index as usize]
    }

    fn bits(self) -> u32 {
        self.index
    }

    fn from_bits(bits: u32) -> Self {
        NodeKey { index: bits }
    }
}

struct NodeKeyCtx<'a, 'b, L: Language> {
    nodes: &'a [L::Node],
    node_ctx: &'b L,
}

impl<'a, 'b, L: Language> CtxEq<NodeKey, NodeKey> for NodeKeyCtx<'a, 'b, L> {
    fn ctx_eq(&self, a: &NodeKey, b: &NodeKey, uf: &mut UnionFind) -> bool {
        let a = a.node(self.nodes);
        let b = b.node(self.nodes);
        self.node_ctx.ctx_eq(a, b, uf)
    }
}

impl<'a, 'b, L: Language> CtxHash<NodeKey> for NodeKeyCtx<'a, 'b, L> {
    fn ctx_hash(&self, value: &NodeKey, uf: &mut UnionFind) -> u64 {
        self.node_ctx.ctx_hash(value.node(self.nodes), uf)
    }
}

/// An EClass entry. Contains either a single new enode and a child
/// eclass (i.e., adds one new enode), or unions two child eclasses
/// together.
#[derive(Debug, Clone, Copy)]
pub struct EClass {
    // formats:
    //
    // 00 | unused  (31 bits)        | NodeKey (31 bits)
    // 01 | eclass_child   (31 bits) | NodeKey (31 bits)
    // 10 | eclass_child_1 (31 bits) | eclass_child_id_2 (31 bits)
    bits: u64,
}

impl EClass {
    fn node(node: NodeKey) -> EClass {
        let node_idx = node.bits() as u64;
        debug_assert!(node_idx < (1 << 31));
        EClass {
            bits: (0b00 << 62) | node_idx,
        }
    }

    fn node_and_child(node: NodeKey, eclass_child: Id) -> EClass {
        let node_idx = node.bits() as u64;
        debug_assert!(node_idx < (1 << 31));
        debug_assert!(eclass_child != Id::invalid());
        let child = eclass_child.0 as u64;
        debug_assert!(child < (1 << 31));
        EClass {
            bits: (0b01 << 62) | (child << 31) | node_idx,
        }
    }

    fn union(child1: Id, child2: Id) -> EClass {
        debug_assert!(child1 != Id::invalid());
        let child1 = child1.0 as u64;
        debug_assert!(child1 < (1 << 31));

        debug_assert!(child2 != Id::invalid());
        let child2 = child2.0 as u64;
        debug_assert!(child2 < (1 << 31));

        EClass {
            bits: (0b10 << 62) | (child1 << 31) | child2,
        }
    }

    /// Get the node, if any, from a node-only or node-and-child
    /// eclass.
    pub fn get_node(&self) -> Option<NodeKey> {
        self.as_node()
            .or_else(|| self.as_node_and_child().map(|(node, _)| node))
    }

    /// Get the first child, if any.
    pub fn child1(&self) -> Option<Id> {
        self.as_node_and_child()
            .map(|(_, p1)| p1)
            .or(self.as_union().map(|(p1, _)| p1))
    }

    /// Get the second child, if any.
    pub fn child2(&self) -> Option<Id> {
        self.as_union().map(|(_, p2)| p2)
    }

    /// If this EClass is just a lone enode, return it.
    pub fn as_node(&self) -> Option<NodeKey> {
        if (self.bits >> 62) == 0b00 {
            let node_idx = (self.bits & ((1 << 31) - 1)) as u32;
            Some(NodeKey::from_bits(node_idx))
        } else {
            None
        }
    }

    /// If this EClass is one new enode and a child, return the node
    /// and child ID.
    pub fn as_node_and_child(&self) -> Option<(NodeKey, Id)> {
        if (self.bits >> 62) == 0b01 {
            let node_idx = (self.bits & ((1 << 31) - 1)) as u32;
            let child = ((self.bits >> 31) & ((1 << 31) - 1)) as u32;
            Some((NodeKey::from_bits(node_idx), Id::from_bits(child)))
        } else {
            None
        }
    }

    /// If this EClass is the union variety, return the two child
    /// EClasses. Both are guaranteed not to be `Id::invalid()`.
    pub fn as_union(&self) -> Option<(Id, Id)> {
        if (self.bits >> 62) == 0b10 {
            let child1 = ((self.bits >> 31) & ((1 << 31) - 1)) as u32;
            let child2 = (self.bits & ((1 << 31) - 1)) as u32;
            Some((Id::from_bits(child1), Id::from_bits(child2)))
        } else {
            None
        }
    }
}

/// A new or existing `T` when adding to a deduplicated set or data
/// structure, like an egraph.
#[derive(Clone, Copy, Debug)]
pub enum NewOrExisting<T> {
    New(T),
    Existing(T),
}

impl<T> NewOrExisting<T> {
    /// Get the underlying value.
    pub fn get(self) -> T {
        match self {
            NewOrExisting::New(t) => t,
            NewOrExisting::Existing(t) => t,
        }
    }
}

impl<L: Language, A: Analysis<L = L>> EGraph<L, A>
where
    L::Node: 'static,
{
    /// Create a new aegraph.
    pub fn new(analysis: Option<A>) -> Self {
        let analysis = analysis.map(|a| (a, SecondaryMap::new()));
        Self {
            nodes: vec![],
            node_map: CtxHashMap::new(),
            classes: PrimaryMap::new(),
            unionfind: UnionFind::new(),
            analysis,
        }
    }

    /// Create a new aegraph with the given capacity.
    pub fn with_capacity(nodes: usize, analysis: Option<A>) -> Self {
        let analysis = analysis.map(|a| (a, SecondaryMap::with_capacity(nodes)));
        Self {
            nodes: Vec::with_capacity(nodes),
            node_map: CtxHashMap::with_capacity(nodes),
            classes: PrimaryMap::with_capacity(nodes),
            unionfind: UnionFind::with_capacity(nodes),
            analysis,
        }
    }

    /// Add a new node.
    pub fn add(&mut self, node: L::Node, node_ctx: &L) -> NewOrExisting<Id> {
        // Push the node. We can then build a NodeKey that refers to
        // it and look for an existing interned copy. If one exists,
        // we can pop the pushed node and return the existing Id.
        let node_idx = self.nodes.len();
        trace!("adding node: {:?}", node);
        let needs_dedup = node_ctx.needs_dedup(&node);
        self.nodes.push(node);

        let key = NodeKey::from_node_idx(node_idx);
        if needs_dedup {
            let ctx = NodeKeyCtx {
                nodes: &self.nodes[..],
                node_ctx,
            };

            match self.node_map.entry(key, &ctx, &mut self.unionfind) {
                Entry::Occupied(o) => {
                    let eclass_id = *o.get();
                    self.nodes.pop();
                    trace!(" -> existing id {}", eclass_id);
                    NewOrExisting::Existing(eclass_id)
                }
                Entry::Vacant(v) => {
                    // We're creating a new eclass now.
                    let eclass_id = self.classes.push(EClass::node(key));
                    trace!(" -> new node and eclass: {}", eclass_id);
                    self.unionfind.add(eclass_id);

                    // Add to interning map with a NodeKey referring to the eclass.
                    v.insert(eclass_id);

                    // Update analysis.
                    let node_ctx = ctx.node_ctx;
                    self.update_analysis_new(node_ctx, eclass_id, key);

                    NewOrExisting::New(eclass_id)
                }
            }
        } else {
            let eclass_id = self.classes.push(EClass::node(key));
            self.unionfind.add(eclass_id);
            NewOrExisting::New(eclass_id)
        }
    }

    /// Merge one eclass into another, maintaining the acyclic
    /// property (args must have lower eclass Ids than the eclass
    /// containing the node with those args). Returns the Id of the
    /// merged eclass.
    pub fn union(&mut self, ctx: &L, a: Id, b: Id) -> Id {
        assert_ne!(a, Id::invalid());
        assert_ne!(b, Id::invalid());
        let (a, b) = (std::cmp::max(a, b), std::cmp::min(a, b));
        trace!("union: id {} and id {}", a, b);
        if a == b {
            trace!(" -> no-op");
            return a;
        }

        self.unionfind.union(a, b);

        // If the younger eclass has no child, we can link it
        // directly and return that eclass. Otherwise, we create a new
        // union eclass.
        if let Some(node) = self.classes[a].as_node() {
            trace!(
                " -> id {} is one-node eclass; making into node-and-child with id {}",
                a,
                b
            );
            self.classes[a] = EClass::node_and_child(node, b);
            self.update_analysis_union(ctx, a, a, b);
            return a;
        }

        let u = self.classes.push(EClass::union(a, b));
        self.unionfind.add(u);
        self.unionfind.union(u, b);
        trace!(" -> union id {} and id {} into id {}", a, b, u);
        self.update_analysis_union(ctx, u, a, b);
        u
    }

    /// Get the canonical ID for an eclass. This may be an older
    /// generation, so will not be able to see all enodes in the
    /// eclass; but it will allow us to unambiguously refer to an
    /// eclass, even across merging.
    pub fn canonical_id_mut(&mut self, eclass: Id) -> Id {
        self.unionfind.find_and_update(eclass)
    }

    /// Get the canonical ID for an eclass. This may be an older
    /// generation, so will not be able to see all enodes in the
    /// eclass; but it will allow us to unambiguously refer to an
    /// eclass, even across merging.
    pub fn canonical_id(&self, eclass: Id) -> Id {
        self.unionfind.find(eclass)
    }

    /// Get the enodes for a given eclass.
    pub fn enodes(&self, eclass: Id) -> NodeIter<L, A> {
        NodeIter {
            stack: smallvec![eclass],
            _phantom1: PhantomData,
            _phantom2: PhantomData,
        }
    }

    /// Update analysis for a given eclass node (new-enode case).
    fn update_analysis_new(&mut self, ctx: &L, eclass: Id, node: NodeKey) {
        if let Some((analysis, state)) = self.analysis.as_mut() {
            let node = node.node(&self.nodes);
            state[eclass] = analysis.for_node(ctx, node, state);
        }
    }

    /// Update analysis for a given eclass node (union case).
    fn update_analysis_union(&mut self, ctx: &L, eclass: Id, a: Id, b: Id) {
        if let Some((analysis, state)) = self.analysis.as_mut() {
            let a = &state[a];
            let b = &state[b];
            state[eclass] = analysis.meet(ctx, a, b);
        }
    }

    /// Get the analysis value for a given eclass. Panics if no analysis is present.
    pub fn analysis_value(&self, eclass: Id) -> &A::Value {
        &self.analysis.as_ref().unwrap().1[eclass]
    }
}

/// An iterator over all nodes in an eclass.
///
/// Because eclasses are immutable once created, this does *not* need
/// to hold an open borrow on the egraph; it is free to add new nodes,
/// while our existing Ids will remain valid.
pub struct NodeIter<L: Language, A: Analysis<L = L>> {
    stack: SmallVec<[Id; 8]>,
    _phantom1: PhantomData<L>,
    _phantom2: PhantomData<A>,
}

impl<L: Language, A: Analysis<L = L>> NodeIter<L, A> {
    #[inline(always)]
    pub fn next<'a>(&mut self, egraph: &'a EGraph<L, A>) -> Option<&'a L::Node> {
        while let Some(next) = self.stack.pop() {
            let eclass = egraph.classes[next];
            if let Some(node) = eclass.as_node() {
                return Some(&egraph.nodes[node.index as usize]);
            } else if let Some((node, child)) = eclass.as_node_and_child() {
                if child != Id::invalid() {
                    self.stack.push(child);
                }
                return Some(&egraph.nodes[node.index as usize]);
            } else if let Some((child1, child2)) = eclass.as_union() {
                debug_assert!(child1 != Id::invalid());
                debug_assert!(child2 != Id::invalid());
                self.stack.push(child2);
                self.stack.push(child1);
                continue;
            } else {
                unreachable!("Invalid eclass format");
            }
        }
        None
    }
}
