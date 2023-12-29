/*
 * Released under the terms of the Apache 2.0 license with LLVM
 * exception. See `LICENSE` for details.
 */

//! Fast postorder computation.

use crate::Block;
use smallvec::{smallvec, SmallVec};

pub fn calculate<'a, SuccFn: Fn(Block) -> &'a [Block]>(
    num_blocks: usize,
    entry: Block,
    succ_blocks: SuccFn,
) -> Vec<Block> {
    let mut ret = vec![];

    // State: visited-block map, and explicit DFS stack.
    let mut visited = vec![];
    visited.resize(num_blocks, false);

    struct State<'a> {
        block: Block,
        succs: &'a [Block],
        next_succ: usize,
    }
    let mut stack: SmallVec<[State; 64]> = smallvec![];

    visited[entry.index()] = true;
    stack.push(State {
        block: entry,
        succs: succ_blocks(entry),
        next_succ: 0,
    });

    while let Some(ref mut state) = stack.last_mut() {
        // Perform one action: push to new succ, skip an already-visited succ, or pop.
        if state.next_succ < state.succs.len() {
            let succ = state.succs[state.next_succ];
            state.next_succ += 1;
            if !visited[succ.index()] {
                visited[succ.index()] = true;
                stack.push(State {
                    block: succ,
                    succs: succ_blocks(succ),
                    next_succ: 0,
                });
            }
        } else {
            ret.push(state.block);
            stack.pop();
        }
    }

    ret
}
