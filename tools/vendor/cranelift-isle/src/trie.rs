//! Trie construction.

use crate::ir::{lower_rule, ExprSequence, PatternInst};
use crate::log;
use crate::sema::{TermEnv, TermId};
use std::collections::BTreeMap;

/// Construct the tries for each term.
pub fn build_tries(termenv: &TermEnv) -> BTreeMap<TermId, TrieNode> {
    let mut builder = TermFunctionsBuilder::default();
    builder.build(termenv);
    log!("builder: {:?}", builder);
    builder.finalize()
}

/// One "input symbol" for the decision tree that handles matching on
/// a term. Each symbol represents one step: we either run a match op,
/// or we finish the match.
///
/// Note that in the original Peepmatic scheme, the input-symbol to
/// the FSM was specified slightly differently. The automaton
/// responded to alphabet symbols that corresponded only to match
/// results, and the "extra state" was used at each automaton node to
/// represent the op to run next. This extra state differentiated
/// nodes that would otherwise be merged together by
/// deduplication. That scheme works well enough, but the "extra
/// state" is slightly confusing and diverges slightly from a pure
/// automaton.
///
/// Instead, here, we imagine that the user of the automaton/trie can
/// query the possible transition edges out of the current state. Each
/// of these edges corresponds to one possible match op to run. After
/// running a match op, we reach a new state corresponding to
/// successful matches up to that point.
///
/// However, it's a bit more subtle than this. Consider the
/// prioritization problem. We want to give the DSL user the ability
/// to change the order in which rules apply, for example to have a
/// tier of "fallback rules" that apply only if more custom rules do
/// not match.
///
/// A somewhat simplistic answer to this problem is "more specific
/// rule wins". However, this implies the existence of a total
/// ordering of linearized match sequences that may not fully capture
/// the intuitive meaning of "more specific". Consider three left-hand
/// sides:
///
/// - (A _ _)
/// - (A (B _) _)
/// - (A _ (B _))
///
/// Intuitively, the first is the least specific. Given the input `(A
/// (B 1) (B 2))`, we can say for sure that the first should not be
/// chosen, because either the second or third would match "more" of
/// the input tree. But which of the second and third should be
/// chosen? A "lexicographic ordering" rule would say that we sort
/// left-hand sides such that the `(B _)` sub-pattern comes before the
/// wildcard `_`, so the second rule wins. But that is arbitrarily
/// privileging one over the other based on the order of the
/// arguments.
///
/// Instead, we can accept explicit priorities from the user to allow
/// either choice. So we need a data structure that can associate
/// matching inputs *with priorities* to outputs.
///
/// Next, we build a decision tree rather than an FSM. Why? Because
/// we're compiling to a structured language, Rust, and states become
/// *program points* rather than *data*, we cannot easily support a
/// DAG structure. In other words, we are not producing a FSM that we
/// can interpret at runtime; rather we are compiling code in which
/// each state corresponds to a sequence of statements and
/// control-flow that branches to a next state, we naturally need
/// nesting; we cannot codegen arbitrary state transitions in an
/// efficient manner. We could support a limited form of DAG that
/// reifies "diamonds" (two alternate paths that reconverge), but
/// supporting this in a way that lets the output refer to values from
/// either side is very complex (we need to invent phi-nodes), and the
/// cases where we want to do this rather than invoke a sub-term (that
/// is compiled to a separate function) are rare. Finally, note that
/// one reason to deduplicate nodes and turn a tree back into a DAG --
/// "output-suffix sharing" as some other instruction-rewriter
/// engines, such as Peepmatic, do -- is not done, because all
/// "output" occurs at leaf nodes; this is necessary because we do not
/// want to start invoking external constructors until we are sure of
/// the match. Some of the code-sharing advantages of the "suffix
/// sharing" scheme can be obtained in a more flexible and
/// user-controllable way (with less understanding of internal
/// compiler logic needed) by factoring logic into different internal
/// terms, which become different compiled functions. This is likely
/// to happen anyway as part of good software engineering practice.
///
/// We prepare for codegen by building a "prioritized trie", where the
/// trie associates input strings with priorities to output values.
/// Each input string is a sequence of match operators followed by an
/// "end of match" token, and each output is a sequence of ops that
/// build the output expression. Each input-output mapping is
/// associated with a priority. The goal of the trie is to generate a
/// decision-tree procedure that lets us execute match ops in a
/// deterministic way, eventually landing at a state that corresponds
/// to the highest-priority matching rule and can produce the output.
///
/// To build this trie, we construct nodes with edges to child nodes;
/// each edge consists of (i) one input token (a `PatternInst` or
/// EOM), and (ii) the priority of rules along this edge. We do not
/// merge rules of different priorities, because the logic to do so is
/// complex and error-prone, necessitating "splits" when we merge
/// together a set of rules over a priority range but later introduce
/// a new possible match op in the "middle" of the range. (E.g., match
/// op A at prio 10, B at prio 5, A at prio 0.) In fact, a previous
/// version of the ISLE compiler worked this way, but in practice the
/// complexity was unneeded.
///
/// To add a rule to this trie, we perform the usual trie-insertion
/// logic, creating edges and subnodes where necessary. A new edge is
/// necessary whenever an edge does not exist for the (priority,
/// symbol) tuple.
///
/// Note that this means that multiple edges with a single match-op
/// may exist, with different priorities.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrieSymbol {
    /// Run a match operation to continue matching a LHS.
    Match {
        /// The match operation to run.
        op: PatternInst,
    },
    /// We successfully matched a LHS.
    EndOfMatch,
}

impl TrieSymbol {
    fn is_eom(&self) -> bool {
        match self {
            TrieSymbol::EndOfMatch => true,
            _ => false,
        }
    }
}

/// A priority.
pub type Prio = i64;

/// An edge in our term trie.
#[derive(Clone, Debug)]
pub struct TrieEdge {
    /// The priority for this edge's sub-trie.
    pub prio: Prio,
    /// The match operation to perform for this edge.
    pub symbol: TrieSymbol,
    /// This edge's sub-trie.
    pub node: TrieNode,
}

/// A node in the term trie.
#[derive(Clone, Debug)]
pub enum TrieNode {
    /// One or more patterns could match.
    ///
    /// Maybe one pattern already has matched, but there are more (higher
    /// priority and/or same priority but more specific) patterns that could
    /// still match.
    Decision {
        /// The child sub-tries that we can match from this point on.
        edges: Vec<TrieEdge>,
    },

    /// The successful match of an LHS pattern, and here is its RHS expression.
    Leaf {
        /// The priority of this rule.
        prio: Prio,
        /// The RHS expression to evaluate upon a successful LHS pattern match.
        output: ExprSequence,
    },

    /// No LHS pattern matches.
    Empty,
}

impl TrieNode {
    fn is_empty(&self) -> bool {
        matches!(self, &TrieNode::Empty)
    }

    fn insert(
        &mut self,
        prio: Prio,
        mut input: impl Iterator<Item = TrieSymbol>,
        output: ExprSequence,
    ) -> bool {
        // Take one input symbol. There must be *at least* one, EOM if
        // nothing else.
        let op = input
            .next()
            .expect("Cannot insert into trie with empty input sequence");
        let is_last = op.is_eom();

        // If we are empty, turn into a decision node.
        if self.is_empty() {
            *self = TrieNode::Decision { edges: vec![] };
        }

        // We must be a decision node.
        let edges = match self {
            &mut TrieNode::Decision { ref mut edges } => edges,
            _ => panic!("insert on leaf node!"),
        };

        // Now find or insert the appropriate edge.
        let edge = edges
            .iter()
            .position(|edge| edge.symbol == op && edge.prio == prio)
            .unwrap_or_else(|| {
                edges.push(TrieEdge {
                    prio,
                    symbol: op,
                    node: TrieNode::Empty,
                });
                edges.len() - 1
            });

        let edge = &mut edges[edge];

        if is_last {
            if !edge.node.is_empty() {
                // If a leaf node already exists at an overlapping
                // prio for this op, there are two competing rules, so
                // we can't insert this one.
                return false;
            }
            edge.node = TrieNode::Leaf { prio, output };
            true
        } else {
            edge.node.insert(prio, input, output)
        }
    }

    /// Sort edges by priority.
    pub fn sort(&mut self) {
        match self {
            TrieNode::Decision { edges } => {
                // Sort by priority, highest integer value first; then
                // by trie symbol.
                edges.sort_by_cached_key(|edge| (-edge.prio, edge.symbol.clone()));
                for child in edges {
                    child.node.sort();
                }
            }
            _ => {}
        }
    }

    /// Get a pretty-printed version of this trie, for debugging.
    pub fn pretty(&self) -> String {
        let mut s = String::new();
        pretty_rec(&mut s, self, "");
        return s;

        fn pretty_rec(s: &mut String, node: &TrieNode, indent: &str) {
            match node {
                TrieNode::Decision { edges } => {
                    s.push_str(indent);
                    s.push_str("TrieNode::Decision:\n");

                    let new_indent = indent.to_owned() + "    ";
                    for edge in edges {
                        s.push_str(indent);
                        s.push_str(&format!(
                            "  edge: prio = {:?}, symbol: {:?}\n",
                            edge.prio, edge.symbol
                        ));
                        pretty_rec(s, &edge.node, &new_indent);
                    }
                }
                TrieNode::Empty | TrieNode::Leaf { .. } => {
                    s.push_str(indent);
                    s.push_str(&format!("{:?}\n", node));
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct TermFunctionsBuilder {
    builders_by_term: BTreeMap<TermId, TrieNode>,
}

impl TermFunctionsBuilder {
    fn build(&mut self, termenv: &TermEnv) {
        log!("termenv: {:?}", termenv);
        for rule in termenv.rules.iter() {
            let (pattern, expr) = lower_rule(termenv, rule.id);

            log!(
                "build:\n- rule {:?}\n- pattern {:?}\n- expr {:?}",
                rule,
                pattern,
                expr
            );

            let symbols = pattern
                .insts
                .into_iter()
                .map(|op| TrieSymbol::Match { op })
                .chain(std::iter::once(TrieSymbol::EndOfMatch));

            self.builders_by_term
                .entry(rule.root_term)
                .or_insert(TrieNode::Empty)
                .insert(rule.prio, symbols, expr);
        }

        for builder in self.builders_by_term.values_mut() {
            builder.sort();
        }
    }

    fn finalize(self) -> BTreeMap<TermId, TrieNode> {
        self.builders_by_term
    }
}
