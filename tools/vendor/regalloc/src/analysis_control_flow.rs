//! Performs control flow analysis.

use log::trace;
use std::cmp::Ordering;

use crate::analysis_main::AnalysisError;
use crate::data_structures::{BlockIx, InstIx, Range, Set, TypedIxVec};
use crate::sparse_set::{SparseSetU, SparseSetUIter};
use crate::Function;

use smallvec::SmallVec;

//=============================================================================
// Debugging config.  Set all these to `false` for normal operation.

// DEBUGGING: set to true to cross-check the dominator-tree computation.
const CROSSCHECK_DOMS: bool = false;

//===========================================================================//
//                                                                           //
// CONTROL FLOW ANALYSIS                                                     //
//                                                                           //
//===========================================================================//

//=============================================================================
// Control flow analysis: create the InstIx-to-BlockIx mapping

// This is trivial, but it's sometimes useful to have.
// Note: confusingly, the `Range` here is data_structures::Range, not
// std::ops::Range.
pub struct InstIxToBlockIxMap {
    vek: TypedIxVec<BlockIx, Range<InstIx>>,
}

impl InstIxToBlockIxMap {
    #[inline(never)]
    pub fn new<F: Function>(func: &F) -> Self {
        let mut vek = TypedIxVec::<BlockIx, Range<InstIx>>::new();
        for bix in func.blocks() {
            let r: Range<InstIx> = func.block_insns(bix);
            assert!(r.start() <= r.last_plus1());
            vek.push(r);
        }

        fn cmp_ranges(r1: &Range<InstIx>, r2: &Range<InstIx>) -> Ordering {
            if r1.last_plus1() <= r2.first() {
                return Ordering::Less;
            }
            if r2.last_plus1() <= r1.first() {
                return Ordering::Greater;
            }
            if r1.first() == r2.first() && r1.last_plus1() == r2.last_plus1() {
                return Ordering::Equal;
            }
            // If this happens, F::block_insns is telling us something that isn't right.
            panic!("InstIxToBlockIxMap::cmp_ranges: overlapping InstIx ranges!");
        }

        vek.sort_unstable_by(|r1, r2| cmp_ranges(r1, r2));
        // Sanity check: ascending, non-overlapping, no gaps.  We need this in
        // order to ensure that binary searching in `map` works properly.
        for i in 1..vek.len() {
            let r_m1 = &vek[BlockIx::new(i - 1)];
            let r_m0 = &vek[BlockIx::new(i - 0)];
            assert!(r_m1.last_plus1() == r_m0.first());
        }

        Self { vek }
    }

    #[inline(never)]
    pub fn map(&self, iix: InstIx) -> BlockIx {
        if self.vek.len() > 0 {
            let mut lo = 0isize;
            let mut hi = self.vek.len() as isize - 1;
            loop {
                if lo > hi {
                    break;
                }
                let mid = (lo + hi) / 2;
                let midv = &self.vek[BlockIx::new(mid as u32)];
                if iix < midv.start() {
                    hi = mid - 1;
                    continue;
                }
                if iix >= midv.last_plus1() {
                    lo = mid + 1;
                    continue;
                }
                assert!(midv.start() <= iix && iix < midv.last_plus1());
                return BlockIx::new(mid as u32);
            }
        }
        panic!("InstIxToBlockIxMap::map: can't map {:?}", iix);
    }
}

//=============================================================================
// Control flow analysis: calculation of block successor and predecessor maps

// Returned TypedIxVecs contain one element per block
#[inline(never)]
fn calc_preds_and_succs<F: Function>(
    func: &F,
    num_blocks: u32,
) -> (
    TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
) {
    trace!("      calc_preds_and_succs: begin");

    assert!(func.blocks().len() == num_blocks as usize);

    // First calculate the succ map, since we can do that directly from the
    // Func.
    //
    // Func::finish() ensures that all blocks are non-empty, and that only the
    // last instruction is a control flow transfer.  Hence the following won't
    // miss any edges.
    let mut succ_map = TypedIxVec::<BlockIx, SparseSetU<[BlockIx; 4]>>::new();
    for b in func.blocks() {
        let mut bix_set = SparseSetU::<[BlockIx; 4]>::empty();
        for bix in func.block_succs(b).iter() {
            bix_set.insert(*bix);
        }
        succ_map.push(bix_set);
    }

    // Now invert the mapping
    let mut pred_map = TypedIxVec::<BlockIx, SparseSetU<[BlockIx; 4]>>::new();
    pred_map.resize(num_blocks, SparseSetU::<[BlockIx; 4]>::empty());
    for (src, dst_set) in (0..).zip(succ_map.iter()) {
        for dst in dst_set.iter() {
            pred_map[*dst].insert(BlockIx::new(src));
        }
    }

    // Stay sane ..
    assert!(pred_map.len() == num_blocks);
    assert!(succ_map.len() == num_blocks);

    let mut n = 0;
    trace!("");
    for (preds, succs) in pred_map.iter().zip(succ_map.iter()) {
        trace!(
            "{:<3?}   preds {:<16?}  succs {:?}",
            BlockIx::new(n),
            preds,
            succs
        );
        n += 1;
    }

    trace!("      calc_preds_and_succs: end");
    (pred_map, succ_map)
}

//=============================================================================
// Control flow analysis: calculation of block preorder and postorder sequences

// Returned Vecs contain one element per block.  `None` is returned if the
// sequences do not contain `num_blocks` elements, in which case the input
// contains blocks not reachable from the entry point, and is invalid.
#[inline(never)]
fn calc_preord_and_postord<F: Function>(
    func: &F,
    num_blocks: u32,
    succ_map: &TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
) -> Option<(Vec<BlockIx>, Vec<BlockIx>)> {
    trace!("      calc_preord_and_postord: begin");

    let mut pre_ord = Vec::<BlockIx>::new();
    let mut post_ord = Vec::<BlockIx>::new();

    let mut visited = TypedIxVec::<BlockIx, bool>::new();
    visited.resize(num_blocks, false);

    // Set up initial state: entry block on the stack, marked as visited, and placed at the
    // start of the pre-ord sequence.
    let mut stack = SmallVec::<[(BlockIx, SparseSetUIter<[BlockIx; 4]>); 64]>::new();
    let bix_entry = func.entry_block();
    visited[bix_entry] = true;
    pre_ord.push(bix_entry);
    stack.push((bix_entry, succ_map[bix_entry].iter()));

    'outer: while let Some((bix, bix_succ_iter)) = stack.last_mut() {
        // Consider the block on the top of the stack.  Does it have any successors we
        // haven't yet visited?
        while let Some(bix_next_succ) = bix_succ_iter.next() {
            if !visited[*bix_next_succ] {
                // Yes.  Push just one of them on the stack, along with a newly initialised
                // iterator for it, and continue by considering the new stack top.  Because
                // blocks are only ever pushed onto the stack once, we must also add the
                // block to the pre-ord sequence at this point.
                visited[*bix_next_succ] = true;
                pre_ord.push(*bix_next_succ);
                stack.push((*bix_next_succ, succ_map[*bix_next_succ].iter()));
                continue 'outer;
            }
        }
        // No.  This is the last time we'll ever hear of it.  So add it to the post-ord
        // sequence, remove the now-defunct stack-top item, and move on.
        post_ord.push(*bix);
        stack.pop();
    }

    assert!(pre_ord.len() == post_ord.len());
    assert!(pre_ord.len() <= num_blocks as usize);
    if pre_ord.len() < num_blocks as usize {
        trace!(
            "      calc_preord_and_postord: invalid: {} blocks, {} reachable",
            num_blocks,
            pre_ord.len()
        );
        return None;
    }

    assert!(pre_ord.len() == num_blocks as usize);
    assert!(post_ord.len() == num_blocks as usize);
    #[cfg(debug_assertions)]
    {
        let mut pre_ord_sorted: Vec<BlockIx> = pre_ord.clone();
        let mut post_ord_sorted: Vec<BlockIx> = post_ord.clone();
        pre_ord_sorted.sort_by(|bix1, bix2| bix1.get().partial_cmp(&bix2.get()).unwrap());
        post_ord_sorted.sort_by(|bix1, bix2| bix1.get().partial_cmp(&bix2.get()).unwrap());
        let expected: Vec<BlockIx> = (0..num_blocks).map(|u| BlockIx::new(u)).collect();
        debug_assert!(pre_ord_sorted == expected);
        debug_assert!(post_ord_sorted == expected);
    }

    trace!("      calc_preord_and_postord: end.  {} blocks", num_blocks);
    Some((pre_ord, post_ord))
}

//=============================================================================
// Computation of per-block dominator sets.  Note, this is slow, and will be
// removed at some point.

// Calculate the dominance relationship, given `pred_map` and a start node
// `start`.  The resulting vector maps each block to the set of blocks that
// dominate it. This algorithm is from Fig 7.14 of Muchnick 1997. The
// algorithm is described as simple but not as performant as some others.
#[inline(never)]
fn calc_dom_sets_slow(
    num_blocks: u32,
    pred_map: &TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    post_ord: &Vec<BlockIx>,
    start: BlockIx,
) -> TypedIxVec<BlockIx, Set<BlockIx>> {
    trace!("          calc_dom_sets_slow: begin");

    let mut dom_map = TypedIxVec::<BlockIx, Set<BlockIx>>::new();

    // FIXME find better names for n/d/t sets.
    {
        let root: BlockIx = start;
        let n_set: Set<BlockIx> =
            Set::from_vec((0..num_blocks).map(|bix| BlockIx::new(bix)).collect());
        let mut d_set: Set<BlockIx>;
        let mut t_set: Set<BlockIx>;

        dom_map.resize(num_blocks, Set::<BlockIx>::empty());
        dom_map[root] = Set::unit(root);
        for block_i in 0..num_blocks {
            let block_ix = BlockIx::new(block_i);
            if block_ix != root {
                dom_map[block_ix] = n_set.clone();
            }
        }

        let mut num_iter = 0;
        loop {
            num_iter += 1;
            trace!("          calc_dom_sets_slow:   outer loop {}", num_iter);
            let mut change = false;
            for i in 0..num_blocks {
                // block_ix travels in "reverse preorder"
                let block_ix = post_ord[(num_blocks - 1 - i) as usize];
                if block_ix == root {
                    continue;
                }
                t_set = n_set.clone();
                for pred_ix in pred_map[block_ix].iter() {
                    t_set.intersect(&dom_map[*pred_ix]);
                }
                d_set = t_set.clone();
                d_set.insert(block_ix);
                if !d_set.equals(&dom_map[block_ix]) {
                    change = true;
                    dom_map[block_ix] = d_set;
                }
            }
            if !change {
                break;
            }
        }
    }

    trace!("");
    let mut block_ix = 0;
    for dom_set in dom_map.iter() {
        trace!("{:<3?}   dom_set {:<16?}", BlockIx::new(block_ix), dom_set);
        block_ix += 1;
    }
    trace!("          calc_dom_sets_slow: end");
    dom_map
}

//=============================================================================
// Computation of per-block dominator sets by first computing trees.
//
// This is an implementation of the algorithm described in
//
//   A Simple, Fast Dominance Algorithm
//   Keith D. Cooper, Timothy J. Harvey, and Ken Kennedy
//   Department of Computer Science, Rice University, Houston, Texas, USA
//   TR-06-33870
//   https://www.cs.rice.edu/~keith/EMBED/dom.pdf
//
// which appears to be the de-facto standard scheme for computing dominance
// quickly nowadays.

// Unfortunately it seems like local consts are not allowed in Rust.
const DT_INVALID_POSTORD: u32 = 0xFFFF_FFFF;
const DT_INVALID_BLOCKIX: BlockIx = BlockIx::BlockIx(0xFFFF_FFFF);

// Helper
fn dt_merge_sets(
    idom: &TypedIxVec<BlockIx, BlockIx>,
    bix2rpostord: &TypedIxVec<BlockIx, u32>,
    mut node1: BlockIx,
    mut node2: BlockIx,
) -> BlockIx {
    while node1 != node2 {
        if node1 == DT_INVALID_BLOCKIX || node2 == DT_INVALID_BLOCKIX {
            return DT_INVALID_BLOCKIX;
        }
        let rpo1 = bix2rpostord[node1];
        let rpo2 = bix2rpostord[node2];
        if rpo1 > rpo2 {
            node1 = idom[node1];
        } else if rpo2 > rpo1 {
            node2 = idom[node2];
        }
    }
    assert!(node1 == node2);
    node1
}

/// Get a mapping from a BlockIx to its reverse postorder
/// number and the inverse mapping from a reverse postorder
/// number to its BlockIx.
fn calc_rpostord(
    num_blocks: u32,
    post_ord: &Vec<BlockIx>,
) -> (TypedIxVec<BlockIx, u32>, Vec<BlockIx>) {
    let mut bix2rpostord = TypedIxVec::<BlockIx, u32>::new();
    let mut rpostord2bix = Vec::<BlockIx>::new();
    bix2rpostord.resize(num_blocks, DT_INVALID_POSTORD);
    rpostord2bix.resize(num_blocks as usize, DT_INVALID_BLOCKIX);
    for n in 0..num_blocks {
        // bix visits the blocks in reverse postorder
        let bix = post_ord[(num_blocks - 1 - n) as usize];
        // Hence:
        bix2rpostord[bix] = n;
        // and
        rpostord2bix[n as usize] = bix;
    }
    for n in 0..num_blocks {
        debug_assert!(bix2rpostord[BlockIx::new(n)] < num_blocks);
    }
    (bix2rpostord, rpostord2bix)
}

#[inline(never)]
fn calc_dom_tree(
    num_blocks: u32,
    pred_map: &TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    bix2rpostord: &TypedIxVec<BlockIx, u32>,
    rpostord2bix: &Vec<BlockIx>,
    start: BlockIx,
) -> TypedIxVec<BlockIx, BlockIx> {
    trace!("        calc_dom_tree: begin");

    // We use 2^32-1 as a marker for an invalid BlockIx or postorder number.
    // Hence we need this:
    assert!(num_blocks < DT_INVALID_POSTORD);

    let mut idom = TypedIxVec::<BlockIx, BlockIx>::new();
    idom.resize(num_blocks, DT_INVALID_BLOCKIX);

    // The start node must have itself as a parent.
    idom[start] = start;

    for i in 0..num_blocks {
        let block_ix = BlockIx::new(i);
        let preds_of_i = &pred_map[block_ix];
        // All nodes must be reachable from the root.  That means that all nodes
        // that aren't `start` must have at least one predecessor.  However, we
        // can't assert the inverse case -- that the start node has no
        // predecessors -- because the start node might be a self-loop, in which
        // case it will have itself as a pred.  See tests/domtree_fuzz1.rat.
        if block_ix != start {
            assert!(!preds_of_i.is_empty());
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for n in 0..num_blocks {
            // Consider blocks in reverse postorder.
            let node = rpostord2bix[n as usize];
            assert!(node != DT_INVALID_BLOCKIX);
            let node_preds = &pred_map[node];
            let rponum = bix2rpostord[node];

            let mut parent = DT_INVALID_BLOCKIX;
            if node_preds.is_empty() {
                // No preds, `parent` remains invalid.
            } else {
                for pred in node_preds.iter() {
                    let pred_rpo = bix2rpostord[*pred];
                    if pred_rpo < rponum {
                        parent = *pred;
                        break;
                    }
                }
            }

            if parent != DT_INVALID_BLOCKIX {
                for pred in node_preds.iter() {
                    if *pred == parent {
                        continue;
                    }
                    if idom[*pred] == DT_INVALID_BLOCKIX {
                        continue;
                    }
                    parent = dt_merge_sets(&idom, &bix2rpostord, parent, *pred);
                }
            }

            if parent != DT_INVALID_BLOCKIX && parent != idom[node] {
                idom[node] = parent;
                changed = true;
            }
        }
    }

    // Check what we can.  The start node should be its own parent.  All other
    // nodes should not be their own parent, since we are assured that there are
    // no dead blocks in the graph, and hence that there is only one dominator
    // tree, that covers the whole graph.
    assert!(idom[start] == start);
    for i in 0..num_blocks {
        let block_ix = BlockIx::new(i);
        // All "parent pointers" are valid.
        assert!(idom[block_ix] != DT_INVALID_BLOCKIX);
        // The only node whose parent pointer points to itself is the start node.
        assert!((idom[block_ix] == block_ix) == (block_ix == start));
    }

    if CROSSCHECK_DOMS {
        // Crosscheck the dom tree, by computing dom sets using the simple
        // iterative algorithm.  Then, for each block, construct the dominator set
        // by walking up the tree to the root, and check that it's the same as
        // what the simple algorithm produced.

        trace!("        calc_dom_tree crosscheck: begin");
        let mut post_ord = rpostord2bix.clone();
        post_ord.reverse();
        let slow_sets = calc_dom_sets_slow(num_blocks, pred_map, &post_ord, start);
        assert!(slow_sets.len() == idom.len());

        for i in 0..num_blocks {
            let mut block_ix = BlockIx::new(i);
            let mut set = Set::<BlockIx>::empty();
            loop {
                set.insert(block_ix);
                let other_block_ix = idom[block_ix];
                if other_block_ix == block_ix {
                    break;
                }
                block_ix = other_block_ix;
            }
            assert!(set.to_vec() == slow_sets[BlockIx::new(i)].to_vec());
        }
        trace!("        calc_dom_tree crosscheck: end");
    }

    trace!("        calc_dom_tree: end");
    idom
}

//=============================================================================
// Computation of per-block loop-depths

#[inline(never)]
fn calc_loop_depths(
    num_blocks: u32,
    pred_map: &TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    succ_map: &TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    post_ord: &Vec<BlockIx>,
    start: BlockIx,
) -> TypedIxVec<BlockIx, u32> {
    trace!("      calc_loop_depths: begin");
    let (bix2rpostord, rpostord2bix) = calc_rpostord(num_blocks, post_ord);
    let idom = calc_dom_tree(num_blocks, pred_map, &bix2rpostord, &rpostord2bix, start);

    // Find the loops.  First, find the "loop header nodes", and from those,
    // derive the loops.
    //
    // loop_set headers:
    // A "back edge" m->n is some edge m->n where n dominates m.  'n' is
    // the loop header node.
    //
    // `back_edges` is a set rather than a vector so as to avoid complications
    // that might later arise if the same loop is enumerated more than once.
    //
    // Iterate over all edges (m->n)
    let mut back_edges = Set::<(BlockIx, BlockIx)>::empty();
    for block_m_ix in BlockIx::new(0).dotdot(BlockIx::new(num_blocks)) {
        for block_n_ix in succ_map[block_m_ix].iter() {
            // Figure out if N dominates M.  Do this by walking the dom tree from M
            // back up towards the root, and seeing if we encounter N on the way.
            // This traversal will go in post-order so we can end early if a block
            // with post-order index greater than N is found (which is equivalent to
            // finding a block with reverse post-order index less than N).
            let mut n_dominates_m = false;
            let mut block_ix = block_m_ix;
            loop {
                if block_ix == *block_n_ix {
                    n_dominates_m = true;
                    break;
                }
                let other_block_ix = idom[block_ix];
                if bix2rpostord[other_block_ix] < bix2rpostord[*block_n_ix] {
                    break;
                }
                block_ix = other_block_ix;
            }
            if n_dominates_m {
                //println!("QQQQ back edge {} -> {}",
                //         block_m_ix.show(), block_n_ix.show());
                back_edges.insert((block_m_ix, *block_n_ix));
            }
        }
    }

    // Now collect the sets of Blocks for each loop.  For each back edge,
    // collect up all the blocks in the natural loop defined by the back edge
    // M->N.  This algorithm is from Fig 7.21 of Muchnick 1997 (an excellent
    // book).  Order in `natural_loops` has no particular meaning.
    let mut natural_loops = Vec::<Set<BlockIx>>::new();
    for (block_m_ix, block_n_ix) in back_edges.iter() {
        let mut loop_set: Set<BlockIx>;
        let mut stack: Vec<BlockIx>;
        stack = Vec::<BlockIx>::new();
        loop_set = Set::<BlockIx>::two(*block_m_ix, *block_n_ix);
        if block_m_ix != block_n_ix {
            // The next line is missing in the Muchnick description.  Without it the
            // algorithm doesn't make any sense, though.
            stack.push(*block_m_ix);
            while let Some(block_p_ix) = stack.pop() {
                for block_q_ix in pred_map[block_p_ix].iter() {
                    if !loop_set.contains(*block_q_ix) {
                        loop_set.insert(*block_q_ix);
                        stack.push(*block_q_ix);
                    }
                }
            }
        }
        natural_loops.push(loop_set);
    }

    // Here is a kludgey way to compute the depth of each loop.  First, order
    // `natural_loops` by increasing size, so the largest loops are at the end.
    // Then, repeatedly scan forwards through the vector, in "upper triangular
    // matrix" style.  For each scan, remember the "current loop".  Initially
    // the "current loop is the start point of each scan.  If, during the scan,
    // we encounter a loop which is a superset of the "current loop", change the
    // "current loop" to this new loop, and increment a counter associated with
    // the start point of the scan.  The effect is that the counter records the
    // nesting depth of the loop at the start of the scan.  For this to be
    // completely accurate, I _think_ this requires the property that loops are
    // either disjoint or nested, but are in no case intersecting.

    natural_loops.sort_by(|left_block_set, right_block_set| {
        left_block_set
            .card()
            .partial_cmp(&right_block_set.card())
            .unwrap()
    });

    let num_loops = natural_loops.len();
    let mut loop_depths = Vec::<u32>::new();
    loop_depths.resize(num_loops, 0);

    for i in 0..num_loops {
        let mut curr = i;
        let mut depth = 1;
        for j in i + 1..num_loops {
            debug_assert!(curr < j);
            if natural_loops[curr].is_subset_of(&natural_loops[j]) {
                depth += 1;
                curr = j;
            }
        }
        loop_depths[i] = depth;
    }

    // Now that we have a depth for each loop, we can finally compute the depth
    // for each block.
    let mut depth_map = TypedIxVec::<BlockIx, u32>::new();
    depth_map.resize(num_blocks, 0);
    for (loop_block_indexes, depth) in natural_loops.iter().zip(loop_depths) {
        for loop_block_ix in loop_block_indexes.iter() {
            if depth_map[*loop_block_ix] < depth {
                depth_map[*loop_block_ix] = depth;
            }
        }
    }

    debug_assert!(depth_map.len() == num_blocks);

    let mut n = 0;
    trace!("");
    for (depth, idom_by) in depth_map.iter().zip(idom.iter()) {
        trace!(
            "{:<3?}   depth {}   idom {:?}",
            BlockIx::new(n),
            depth,
            idom_by
        );
        n += 1;
    }

    trace!("      calc_loop_depths: end");
    depth_map
}

//=============================================================================
// Control-flow analysis top level: For a Func: predecessors, successors,
// preord and postord sequences, and loop depths.

// CFGInfo contains CFG-related info computed from a Func.
pub struct CFGInfo {
    // All these TypedIxVecs and plain Vecs contain one element per Block in the
    // Func.

    // Predecessor and successor maps.
    pub pred_map: TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,
    pub succ_map: TypedIxVec<BlockIx, SparseSetU<[BlockIx; 4]>>,

    // Pre- and post-order sequences.  Iterating forwards through these
    // vectors enumerates the blocks in preorder and postorder respectively.
    pub pre_ord: Vec<BlockIx>,
    pub _post_ord: Vec<BlockIx>,

    // This maps from a Block to the loop depth that it is at
    pub depth_map: TypedIxVec<BlockIx, u32>,
}

impl CFGInfo {
    #[inline(never)]
    pub fn create<F: Function>(func: &F) -> Result<Self, AnalysisError> {
        trace!("    CFGInfo::create: begin");

        // Throw out insanely large inputs.  They'll probably cause failure later
        // on.
        let num_blocks_usize = func.blocks().len();
        if num_blocks_usize >= 1 * 1024 * 1024 {
            // 1 million blocks should be enough for anyone.  That will soak up 20
            // index bits, leaving a "safety margin" of 12 bits for indices for
            // induced structures (RangeFragIx, InstIx, VirtualRangeIx, RealRangeIx,
            // etc).
            return Err(AnalysisError::ImplementationLimitsExceeded);
        }

        // Similarly, limit the number of instructions to 16 million.  This allows
        // 16 insns per block with the worst-case number of blocks.  Because each
        // insn typically generates somewhat less than one new value, this check
        // also has the effect of limiting the number of virtual registers to
        // roughly the same amount (16 million).
        if func.insns().len() >= 16 * 1024 * 1024 {
            return Err(AnalysisError::ImplementationLimitsExceeded);
        }

        // Now we know we're safe to narrow it to u32.
        let num_blocks = num_blocks_usize as u32;

        // === BEGIN compute successor and predecessor maps ===
        //
        let (pred_map, succ_map) = calc_preds_and_succs(func, num_blocks);
        assert!(pred_map.len() == num_blocks);
        assert!(succ_map.len() == num_blocks);
        //
        // === END compute successor and predecessor maps ===

        // === BEGIN check that critical edges have been split ===
        //
        for (src, dst_set) in (0..).zip(succ_map.iter()) {
            if dst_set.card() < 2 {
                continue;
            }
            for dst in dst_set.iter() {
                if pred_map[*dst].card() >= 2 {
                    return Err(AnalysisError::CriticalEdge {
                        from: BlockIx::new(src),
                        to: *dst,
                    });
                }
            }
        }
        //
        // === END check that critical edges have been split ===

        // === BEGIN compute preord/postord sequences ===
        //
        let mb_pre_ord_and_post_ord = calc_preord_and_postord(func, num_blocks, &succ_map);
        if mb_pre_ord_and_post_ord.is_none() {
            return Err(AnalysisError::UnreachableBlocks);
        }

        let (pre_ord, post_ord) = mb_pre_ord_and_post_ord.unwrap();
        assert!(pre_ord.len() == num_blocks as usize);
        assert!(post_ord.len() == num_blocks as usize);
        //
        // === END compute preord/postord sequences ===

        // === BEGIN compute loop depth of all Blocks
        //
        let depth_map = calc_loop_depths(
            num_blocks,
            &pred_map,
            &succ_map,
            &post_ord,
            func.entry_block(),
        );
        debug_assert!(depth_map.len() == num_blocks);
        //
        // === END compute loop depth of all Blocks

        trace!("    CFGInfo::create: end");
        Ok(CFGInfo {
            pred_map,
            succ_map,
            pre_ord,
            _post_ord: post_ord,
            depth_map,
        })
    }
}
