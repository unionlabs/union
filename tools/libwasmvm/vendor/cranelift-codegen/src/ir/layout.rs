//! Function layout.
//!
//! The order of basic blocks in a function and the order of instructions in a block is
//! determined by the `Layout` data structure defined in this module.

use crate::entity::SecondaryMap;
use crate::ir::dfg::DataFlowGraph;
use crate::ir::progpoint::{ExpandedProgramPoint, ProgramOrder};
use crate::ir::{Block, Inst};
use crate::packed_option::PackedOption;
use crate::timing;
use core::cmp;
use core::iter::{IntoIterator, Iterator};

/// The `Layout` struct determines the layout of blocks and instructions in a function. It does not
/// contain definitions of instructions or blocks, but depends on `Inst` and `Block` entity references
/// being defined elsewhere.
///
/// This data structure determines:
///
/// - The order of blocks in the function.
/// - Which block contains a given instruction.
/// - The order of instructions with a block.
///
/// While data dependencies are not recorded, instruction ordering does affect control
/// dependencies, so part of the semantics of the program are determined by the layout.
///
#[derive(Clone)]
pub struct Layout {
    /// Linked list nodes for the layout order of blocks Forms a doubly linked list, terminated in
    /// both ends by `None`.
    blocks: SecondaryMap<Block, BlockNode>,

    /// Linked list nodes for the layout order of instructions. Forms a double linked list per block,
    /// terminated in both ends by `None`.
    insts: SecondaryMap<Inst, InstNode>,

    /// First block in the layout order, or `None` when no blocks have been laid out.
    first_block: Option<Block>,

    /// Last block in the layout order, or `None` when no blocks have been laid out.
    last_block: Option<Block>,
}

impl Layout {
    /// Create a new empty `Layout`.
    pub fn new() -> Self {
        Self {
            blocks: SecondaryMap::new(),
            insts: SecondaryMap::new(),
            first_block: None,
            last_block: None,
        }
    }

    /// Clear the layout.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.insts.clear();
        self.first_block = None;
        self.last_block = None;
    }

    /// Returns the capacity of the `BlockData` map.
    pub fn block_capacity(&self) -> usize {
        self.blocks.capacity()
    }
}

/// Sequence numbers.
///
/// All instructions and blocks are given a sequence number that can be used to quickly determine
/// their relative position in the layout. The sequence numbers are not contiguous, but are assigned
/// like line numbers in BASIC: 10, 20, 30, ...
///
/// The block sequence numbers are strictly increasing, and so are the instruction sequence numbers
/// within a block. The instruction sequence numbers are all between the sequence number of their
/// containing block and the following block.
///
/// The result is that sequence numbers work like BASIC line numbers for the textual form of the IR.
type SequenceNumber = u32;

/// Initial stride assigned to new sequence numbers.
const MAJOR_STRIDE: SequenceNumber = 10;

/// Secondary stride used when renumbering locally.
const MINOR_STRIDE: SequenceNumber = 2;

/// Limit on the sequence number range we'll renumber locally. If this limit is exceeded, we'll
/// switch to a full function renumbering.
const LOCAL_LIMIT: SequenceNumber = 100 * MINOR_STRIDE;

/// Compute the midpoint between `a` and `b`.
/// Return `None` if the midpoint would be equal to either.
fn midpoint(a: SequenceNumber, b: SequenceNumber) -> Option<SequenceNumber> {
    debug_assert!(a < b);
    // Avoid integer overflow.
    let m = a + (b - a) / 2;
    if m > a {
        Some(m)
    } else {
        None
    }
}

#[test]
fn test_midpoint() {
    assert_eq!(midpoint(0, 1), None);
    assert_eq!(midpoint(0, 2), Some(1));
    assert_eq!(midpoint(0, 3), Some(1));
    assert_eq!(midpoint(0, 4), Some(2));
    assert_eq!(midpoint(1, 4), Some(2));
    assert_eq!(midpoint(2, 4), Some(3));
    assert_eq!(midpoint(3, 4), None);
    assert_eq!(midpoint(3, 4), None);
}

impl ProgramOrder for Layout {
    fn cmp<A, B>(&self, a: A, b: B) -> cmp::Ordering
    where
        A: Into<ExpandedProgramPoint>,
        B: Into<ExpandedProgramPoint>,
    {
        let a_seq = self.seq(a);
        let b_seq = self.seq(b);
        a_seq.cmp(&b_seq)
    }

    fn is_block_gap(&self, inst: Inst, block: Block) -> bool {
        let i = &self.insts[inst];
        let e = &self.blocks[block];

        i.next.is_none() && i.block == e.prev
    }
}

// Private methods for dealing with sequence numbers.
impl Layout {
    /// Get the sequence number of a program point that must correspond to an entity in the layout.
    fn seq<PP: Into<ExpandedProgramPoint>>(&self, pp: PP) -> SequenceNumber {
        // When `PP = Inst` or `PP = Block`, we expect this dynamic type check to be optimized out.
        match pp.into() {
            ExpandedProgramPoint::Block(block) => self.blocks[block].seq,
            ExpandedProgramPoint::Inst(inst) => self.insts[inst].seq,
        }
    }

    /// Get the last sequence number in `block`.
    fn last_block_seq(&self, block: Block) -> SequenceNumber {
        // Get the seq of the last instruction if it exists, otherwise use the block header seq.
        self.blocks[block]
            .last_inst
            .map(|inst| self.insts[inst].seq)
            .unwrap_or(self.blocks[block].seq)
    }

    /// Assign a valid sequence number to `block` such that the numbers are still monotonic. This may
    /// require renumbering.
    fn assign_block_seq(&mut self, block: Block) {
        debug_assert!(self.is_block_inserted(block));

        // Get the sequence number immediately before `block`, or 0.
        let prev_seq = self.blocks[block]
            .prev
            .map(|prev_block| self.last_block_seq(prev_block))
            .unwrap_or(0);

        // Get the sequence number immediately following `block`.
        let next_seq = if let Some(inst) = self.blocks[block].first_inst.expand() {
            self.insts[inst].seq
        } else if let Some(next_block) = self.blocks[block].next.expand() {
            self.blocks[next_block].seq
        } else {
            // There is nothing after `block`. We can just use a major stride.
            self.blocks[block].seq = prev_seq + MAJOR_STRIDE;
            return;
        };

        // Check if there is room between these sequence numbers.
        if let Some(seq) = midpoint(prev_seq, next_seq) {
            self.blocks[block].seq = seq;
        } else {
            // No available integers between `prev_seq` and `next_seq`. We have to renumber.
            self.renumber_from_block(block, prev_seq + MINOR_STRIDE, prev_seq + LOCAL_LIMIT);
        }
    }

    /// Assign a valid sequence number to `inst` such that the numbers are still monotonic. This may
    /// require renumbering.
    fn assign_inst_seq(&mut self, inst: Inst) {
        let block = self
            .inst_block(inst)
            .expect("inst must be inserted before assigning an seq");

        // Get the sequence number immediately before `inst`.
        let prev_seq = match self.insts[inst].prev.expand() {
            Some(prev_inst) => self.insts[prev_inst].seq,
            None => self.blocks[block].seq,
        };

        // Get the sequence number immediately following `inst`.
        let next_seq = if let Some(next_inst) = self.insts[inst].next.expand() {
            self.insts[next_inst].seq
        } else if let Some(next_block) = self.blocks[block].next.expand() {
            self.blocks[next_block].seq
        } else {
            // There is nothing after `inst`. We can just use a major stride.
            self.insts[inst].seq = prev_seq + MAJOR_STRIDE;
            return;
        };

        // Check if there is room between these sequence numbers.
        if let Some(seq) = midpoint(prev_seq, next_seq) {
            self.insts[inst].seq = seq;
        } else {
            // No available integers between `prev_seq` and `next_seq`. We have to renumber.
            self.renumber_from_inst(inst, prev_seq + MINOR_STRIDE, prev_seq + LOCAL_LIMIT);
        }
    }

    /// Renumber instructions starting from `inst` until the end of the block or until numbers catch
    /// up.
    ///
    /// Return `None` if renumbering has caught up and the sequence is monotonic again. Otherwise
    /// return the last used sequence number.
    ///
    /// If sequence numbers exceed `limit`, switch to a full function renumbering and return `None`.
    fn renumber_insts(
        &mut self,
        inst: Inst,
        seq: SequenceNumber,
        limit: SequenceNumber,
    ) -> Option<SequenceNumber> {
        let mut inst = inst;
        let mut seq = seq;

        loop {
            self.insts[inst].seq = seq;

            // Next instruction.
            inst = match self.insts[inst].next.expand() {
                None => return Some(seq),
                Some(next) => next,
            };

            if seq < self.insts[inst].seq {
                // Sequence caught up.
                return None;
            }

            if seq > limit {
                // We're pushing too many instructions in front of us.
                // Switch to a full function renumbering to make some space.
                self.full_renumber();
                return None;
            }

            seq += MINOR_STRIDE;
        }
    }

    /// Renumber starting from `block` to `seq` and continuing until the sequence numbers are
    /// monotonic again.
    fn renumber_from_block(
        &mut self,
        block: Block,
        first_seq: SequenceNumber,
        limit: SequenceNumber,
    ) {
        let mut block = block;
        let mut seq = first_seq;

        loop {
            self.blocks[block].seq = seq;

            // Renumber instructions in `block`. Stop when the numbers catch up.
            if let Some(inst) = self.blocks[block].first_inst.expand() {
                seq = match self.renumber_insts(inst, seq + MINOR_STRIDE, limit) {
                    Some(s) => s,
                    None => return,
                }
            }

            // Advance to the next block.
            block = match self.blocks[block].next.expand() {
                Some(next) => next,
                None => return,
            };

            // Stop renumbering once the numbers catch up.
            if seq < self.blocks[block].seq {
                return;
            }

            seq += MINOR_STRIDE;
        }
    }

    /// Renumber starting from `inst` to `seq` and continuing until the sequence numbers are
    /// monotonic again.
    fn renumber_from_inst(&mut self, inst: Inst, first_seq: SequenceNumber, limit: SequenceNumber) {
        if let Some(seq) = self.renumber_insts(inst, first_seq, limit) {
            // Renumbering spills over into next block.
            if let Some(next_block) = self.blocks[self.inst_block(inst).unwrap()].next.expand() {
                self.renumber_from_block(next_block, seq + MINOR_STRIDE, limit);
            }
        }
    }

    /// Renumber all blocks and instructions in the layout.
    ///
    /// This doesn't affect the position of anything, but it gives more room in the internal
    /// sequence numbers for inserting instructions later.
    fn full_renumber(&mut self) {
        let _tt = timing::layout_renumber();
        let mut seq = 0;
        let mut next_block = self.first_block;
        while let Some(block) = next_block {
            self.blocks[block].seq = seq;
            seq += MAJOR_STRIDE;
            next_block = self.blocks[block].next.expand();

            let mut next_inst = self.blocks[block].first_inst.expand();
            while let Some(inst) = next_inst {
                self.insts[inst].seq = seq;
                seq += MAJOR_STRIDE;
                next_inst = self.insts[inst].next.expand();
            }
        }
        log::trace!("Renumbered {} program points", seq / MAJOR_STRIDE);
    }
}

/// Methods for laying out blocks.
///
/// An unknown block starts out as *not inserted* in the block layout. The layout is a linear order of
/// inserted blocks. Once a block has been inserted in the layout, instructions can be added. A block
/// can only be removed from the layout when it is empty.
///
/// Since every block must end with a terminator instruction which cannot fall through, the layout of
/// blocks do not affect the semantics of the program.
///
impl Layout {
    /// Is `block` currently part of the layout?
    pub fn is_block_inserted(&self, block: Block) -> bool {
        Some(block) == self.first_block || self.blocks[block].prev.is_some()
    }

    /// Insert `block` as the last block in the layout.
    pub fn append_block(&mut self, block: Block) {
        debug_assert!(
            !self.is_block_inserted(block),
            "Cannot append block that is already in the layout"
        );
        {
            let node = &mut self.blocks[block];
            debug_assert!(node.first_inst.is_none() && node.last_inst.is_none());
            node.prev = self.last_block.into();
            node.next = None.into();
        }
        if let Some(last) = self.last_block {
            self.blocks[last].next = block.into();
        } else {
            self.first_block = Some(block);
        }
        self.last_block = Some(block);
        self.assign_block_seq(block);
    }

    /// Insert `block` in the layout before the existing block `before`.
    pub fn insert_block(&mut self, block: Block, before: Block) {
        debug_assert!(
            !self.is_block_inserted(block),
            "Cannot insert block that is already in the layout"
        );
        debug_assert!(
            self.is_block_inserted(before),
            "block Insertion point not in the layout"
        );
        let after = self.blocks[before].prev;
        {
            let node = &mut self.blocks[block];
            node.next = before.into();
            node.prev = after;
        }
        self.blocks[before].prev = block.into();
        match after.expand() {
            None => self.first_block = Some(block),
            Some(a) => self.blocks[a].next = block.into(),
        }
        self.assign_block_seq(block);
    }

    /// Insert `block` in the layout *after* the existing block `after`.
    pub fn insert_block_after(&mut self, block: Block, after: Block) {
        debug_assert!(
            !self.is_block_inserted(block),
            "Cannot insert block that is already in the layout"
        );
        debug_assert!(
            self.is_block_inserted(after),
            "block Insertion point not in the layout"
        );
        let before = self.blocks[after].next;
        {
            let node = &mut self.blocks[block];
            node.next = before;
            node.prev = after.into();
        }
        self.blocks[after].next = block.into();
        match before.expand() {
            None => self.last_block = Some(block),
            Some(b) => self.blocks[b].prev = block.into(),
        }
        self.assign_block_seq(block);
    }

    /// Remove `block` from the layout.
    pub fn remove_block(&mut self, block: Block) {
        debug_assert!(self.is_block_inserted(block), "block not in the layout");
        debug_assert!(self.first_inst(block).is_none(), "block must be empty.");

        // Clear the `block` node and extract links.
        let prev;
        let next;
        {
            let n = &mut self.blocks[block];
            prev = n.prev;
            next = n.next;
            n.prev = None.into();
            n.next = None.into();
        }
        // Fix up links to `block`.
        match prev.expand() {
            None => self.first_block = next.expand(),
            Some(p) => self.blocks[p].next = next,
        }
        match next.expand() {
            None => self.last_block = prev.expand(),
            Some(n) => self.blocks[n].prev = prev,
        }
    }

    /// Return an iterator over all blocks in layout order.
    pub fn blocks(&self) -> Blocks {
        Blocks {
            layout: self,
            next: self.first_block,
        }
    }

    /// Get the function's entry block.
    /// This is simply the first block in the layout order.
    pub fn entry_block(&self) -> Option<Block> {
        self.first_block
    }

    /// Get the last block in the layout.
    pub fn last_block(&self) -> Option<Block> {
        self.last_block
    }

    /// Get the block preceding `block` in the layout order.
    pub fn prev_block(&self, block: Block) -> Option<Block> {
        self.blocks[block].prev.expand()
    }

    /// Get the block following `block` in the layout order.
    pub fn next_block(&self, block: Block) -> Option<Block> {
        self.blocks[block].next.expand()
    }

    /// Mark a block as "cold".
    ///
    /// This will try to move it out of the ordinary path of execution
    /// when lowered to machine code.
    pub fn set_cold(&mut self, block: Block) {
        self.blocks[block].cold = true;
    }

    /// Is the given block cold?
    pub fn is_cold(&self, block: Block) -> bool {
        self.blocks[block].cold
    }
}

#[derive(Clone, Debug, Default)]
struct BlockNode {
    prev: PackedOption<Block>,
    next: PackedOption<Block>,
    first_inst: PackedOption<Inst>,
    last_inst: PackedOption<Inst>,
    seq: SequenceNumber,
    cold: bool,
}

/// Iterate over blocks in layout order. See [crate::ir::layout::Layout::blocks].
pub struct Blocks<'f> {
    layout: &'f Layout,
    next: Option<Block>,
}

impl<'f> Iterator for Blocks<'f> {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        match self.next {
            Some(block) => {
                self.next = self.layout.next_block(block);
                Some(block)
            }
            None => None,
        }
    }
}

/// Use a layout reference in a for loop.
impl<'f> IntoIterator for &'f Layout {
    type Item = Block;
    type IntoIter = Blocks<'f>;

    fn into_iter(self) -> Blocks<'f> {
        self.blocks()
    }
}

/// Methods for arranging instructions.
///
/// An instruction starts out as *not inserted* in the layout. An instruction can be inserted into
/// a block at a given position.
impl Layout {
    /// Get the block containing `inst`, or `None` if `inst` is not inserted in the layout.
    pub fn inst_block(&self, inst: Inst) -> Option<Block> {
        self.insts[inst].block.into()
    }

    /// Get the block containing the program point `pp`. Panic if `pp` is not in the layout.
    pub fn pp_block<PP>(&self, pp: PP) -> Block
    where
        PP: Into<ExpandedProgramPoint>,
    {
        match pp.into() {
            ExpandedProgramPoint::Block(block) => block,
            ExpandedProgramPoint::Inst(inst) => {
                self.inst_block(inst).expect("Program point not in layout")
            }
        }
    }

    /// Append `inst` to the end of `block`.
    pub fn append_inst(&mut self, inst: Inst, block: Block) {
        debug_assert_eq!(self.inst_block(inst), None);
        debug_assert!(
            self.is_block_inserted(block),
            "Cannot append instructions to block not in layout"
        );
        {
            let block_node = &mut self.blocks[block];
            {
                let inst_node = &mut self.insts[inst];
                inst_node.block = block.into();
                inst_node.prev = block_node.last_inst;
                debug_assert!(inst_node.next.is_none());
            }
            if block_node.first_inst.is_none() {
                block_node.first_inst = inst.into();
            } else {
                self.insts[block_node.last_inst.unwrap()].next = inst.into();
            }
            block_node.last_inst = inst.into();
        }
        self.assign_inst_seq(inst);
    }

    /// Fetch a block's first instruction.
    pub fn first_inst(&self, block: Block) -> Option<Inst> {
        self.blocks[block].first_inst.into()
    }

    /// Fetch a block's last instruction.
    pub fn last_inst(&self, block: Block) -> Option<Inst> {
        self.blocks[block].last_inst.into()
    }

    /// Fetch the instruction following `inst`.
    pub fn next_inst(&self, inst: Inst) -> Option<Inst> {
        self.insts[inst].next.expand()
    }

    /// Fetch the instruction preceding `inst`.
    pub fn prev_inst(&self, inst: Inst) -> Option<Inst> {
        self.insts[inst].prev.expand()
    }

    /// Fetch the first instruction in a block's terminal branch group.
    pub fn canonical_branch_inst(&self, dfg: &DataFlowGraph, block: Block) -> Option<Inst> {
        // Basic blocks permit at most two terminal branch instructions.
        // If two, the former is conditional and the latter is unconditional.
        let last = self.last_inst(block)?;
        if let Some(prev) = self.prev_inst(last) {
            if dfg[prev].opcode().is_branch() {
                return Some(prev);
            }
        }
        Some(last)
    }

    /// Insert `inst` before the instruction `before` in the same block.
    pub fn insert_inst(&mut self, inst: Inst, before: Inst) {
        debug_assert_eq!(self.inst_block(inst), None);
        let block = self
            .inst_block(before)
            .expect("Instruction before insertion point not in the layout");
        let after = self.insts[before].prev;
        {
            let inst_node = &mut self.insts[inst];
            inst_node.block = block.into();
            inst_node.next = before.into();
            inst_node.prev = after;
        }
        self.insts[before].prev = inst.into();
        match after.expand() {
            None => self.blocks[block].first_inst = inst.into(),
            Some(a) => self.insts[a].next = inst.into(),
        }
        self.assign_inst_seq(inst);
    }

    /// Remove `inst` from the layout.
    pub fn remove_inst(&mut self, inst: Inst) {
        let block = self.inst_block(inst).expect("Instruction already removed.");
        // Clear the `inst` node and extract links.
        let prev;
        let next;
        {
            let n = &mut self.insts[inst];
            prev = n.prev;
            next = n.next;
            n.block = None.into();
            n.prev = None.into();
            n.next = None.into();
        }
        // Fix up links to `inst`.
        match prev.expand() {
            None => self.blocks[block].first_inst = next,
            Some(p) => self.insts[p].next = next,
        }
        match next.expand() {
            None => self.blocks[block].last_inst = prev,
            Some(n) => self.insts[n].prev = prev,
        }
    }

    /// Iterate over the instructions in `block` in layout order.
    pub fn block_insts(&self, block: Block) -> Insts {
        Insts {
            layout: self,
            head: self.blocks[block].first_inst.into(),
            tail: self.blocks[block].last_inst.into(),
        }
    }

    /// Iterate over a limited set of instruction which are likely the branches of `block` in layout
    /// order. Any instruction not visited by this iterator is not a branch, but an instruction visited by this may not be a branch.
    pub fn block_likely_branches(&self, block: Block) -> Insts {
        // Note: Checking whether an instruction is a branch or not while walking backward might add
        // extra overhead. However, we know that the number of branches is limited to 2 at the end of
        // each block, and therefore we can just iterate over the last 2 instructions.
        let mut iter = self.block_insts(block);
        let head = iter.head;
        let tail = iter.tail;
        iter.next_back();
        let head = iter.next_back().or(head);
        Insts {
            layout: self,
            head,
            tail,
        }
    }

    /// Split the block containing `before` in two.
    ///
    /// Insert `new_block` after the old block and move `before` and the following instructions to
    /// `new_block`:
    ///
    /// ```text
    /// old_block:
    ///     i1
    ///     i2
    ///     i3 << before
    ///     i4
    /// ```
    /// becomes:
    ///
    /// ```text
    /// old_block:
    ///     i1
    ///     i2
    /// new_block:
    ///     i3 << before
    ///     i4
    /// ```
    pub fn split_block(&mut self, new_block: Block, before: Inst) {
        let old_block = self
            .inst_block(before)
            .expect("The `before` instruction must be in the layout");
        debug_assert!(!self.is_block_inserted(new_block));

        // Insert new_block after old_block.
        let next_block = self.blocks[old_block].next;
        let last_inst = self.blocks[old_block].last_inst;
        {
            let node = &mut self.blocks[new_block];
            node.prev = old_block.into();
            node.next = next_block;
            node.first_inst = before.into();
            node.last_inst = last_inst;
        }
        self.blocks[old_block].next = new_block.into();

        // Fix backwards link.
        if Some(old_block) == self.last_block {
            self.last_block = Some(new_block);
        } else {
            self.blocks[next_block.unwrap()].prev = new_block.into();
        }

        // Disconnect the instruction links.
        let prev_inst = self.insts[before].prev;
        self.insts[before].prev = None.into();
        self.blocks[old_block].last_inst = prev_inst;
        match prev_inst.expand() {
            None => self.blocks[old_block].first_inst = None.into(),
            Some(pi) => self.insts[pi].next = None.into(),
        }

        // Fix the instruction -> block pointers.
        let mut opt_i = Some(before);
        while let Some(i) = opt_i {
            debug_assert_eq!(self.insts[i].block.expand(), Some(old_block));
            self.insts[i].block = new_block.into();
            opt_i = self.insts[i].next.into();
        }

        self.assign_block_seq(new_block);
    }
}

#[derive(Clone, Debug, Default)]
struct InstNode {
    /// The Block containing this instruction, or `None` if the instruction is not yet inserted.
    block: PackedOption<Block>,
    prev: PackedOption<Inst>,
    next: PackedOption<Inst>,
    seq: SequenceNumber,
}

/// Iterate over instructions in a block in layout order. See `Layout::block_insts()`.
pub struct Insts<'f> {
    layout: &'f Layout,
    head: Option<Inst>,
    tail: Option<Inst>,
}

impl<'f> Iterator for Insts<'f> {
    type Item = Inst;

    fn next(&mut self) -> Option<Inst> {
        let rval = self.head;
        if let Some(inst) = rval {
            if self.head == self.tail {
                self.head = None;
                self.tail = None;
            } else {
                self.head = self.layout.insts[inst].next.into();
            }
        }
        rval
    }
}

impl<'f> DoubleEndedIterator for Insts<'f> {
    fn next_back(&mut self) -> Option<Inst> {
        let rval = self.tail;
        if let Some(inst) = rval {
            if self.head == self.tail {
                self.head = None;
                self.tail = None;
            } else {
                self.tail = self.layout.insts[inst].prev.into();
            }
        }
        rval
    }
}

/// A custom serialize and deserialize implementation for [`Layout`].
///
/// This doesn't use a derived implementation as [`Layout`] is a manual implementation of a linked
/// list. Storing it directly as a regular list saves a lot of space.
///
/// The following format is used. (notated in EBNF form)
///
/// ```plain
/// data = block_data * ;
/// block_data = "block_id" , "inst_count" , ( "inst_id" * ) ;
/// ```
#[cfg(feature = "enable-serde")]
mod serde {
    use ::serde::de::{Deserializer, Error, SeqAccess, Visitor};
    use ::serde::ser::{SerializeSeq, Serializer};
    use ::serde::{Deserialize, Serialize};
    use core::convert::TryFrom;
    use core::fmt;
    use core::marker::PhantomData;

    use super::*;

    impl Serialize for Layout {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let size = self.blocks().count() * 2
                + self
                    .blocks()
                    .map(|block| self.block_insts(block).count())
                    .sum::<usize>();
            let mut seq = serializer.serialize_seq(Some(size))?;
            for block in self.blocks() {
                seq.serialize_element(&block)?;
                seq.serialize_element(&u32::try_from(self.block_insts(block).count()).unwrap())?;
                for inst in self.block_insts(block) {
                    seq.serialize_element(&inst)?;
                }
            }
            seq.end()
        }
    }

    impl<'de> Deserialize<'de> for Layout {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(LayoutVisitor {
                marker: PhantomData,
            })
        }
    }

    struct LayoutVisitor {
        marker: PhantomData<fn() -> Layout>,
    }

    impl<'de> Visitor<'de> for LayoutVisitor {
        type Value = Layout;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a `cranelift_codegen::ir::Layout`")
        }

        fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: SeqAccess<'de>,
        {
            let mut layout = Layout::new();

            while let Some(block) = access.next_element::<Block>()? {
                layout.append_block(block);

                let count = access
                    .next_element::<u32>()?
                    .ok_or_else(|| Error::missing_field("count"))?;
                for _ in 0..count {
                    let inst = access
                        .next_element::<Inst>()?
                        .ok_or_else(|| Error::missing_field("inst"))?;
                    layout.append_inst(inst, block);
                }
            }

            Ok(layout)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Layout;
    use crate::cursor::{Cursor, CursorPosition};
    use crate::entity::EntityRef;
    use crate::ir::{Block, Inst, ProgramOrder, SourceLoc};
    use alloc::vec::Vec;
    use core::cmp::Ordering;

    struct LayoutCursor<'f> {
        /// Borrowed function layout. Public so it can be re-borrowed from this cursor.
        pub layout: &'f mut Layout,
        pos: CursorPosition,
    }

    impl<'f> Cursor for LayoutCursor<'f> {
        fn position(&self) -> CursorPosition {
            self.pos
        }

        fn set_position(&mut self, pos: CursorPosition) {
            self.pos = pos;
        }

        fn srcloc(&self) -> SourceLoc {
            unimplemented!()
        }

        fn set_srcloc(&mut self, _srcloc: SourceLoc) {
            unimplemented!()
        }

        fn layout(&self) -> &Layout {
            self.layout
        }

        fn layout_mut(&mut self) -> &mut Layout {
            self.layout
        }
    }

    impl<'f> LayoutCursor<'f> {
        /// Create a new `LayoutCursor` for `layout`.
        /// The cursor holds a mutable reference to `layout` for its entire lifetime.
        pub fn new(layout: &'f mut Layout) -> Self {
            Self {
                layout,
                pos: CursorPosition::Nowhere,
            }
        }
    }

    fn verify(layout: &mut Layout, blocks: &[(Block, &[Inst])]) {
        // Check that blocks are inserted and instructions belong the right places.
        // Check forward linkage with iterators.
        // Check that layout sequence numbers are strictly monotonic.
        {
            let mut seq = 0;
            let mut block_iter = layout.blocks();
            for &(block, insts) in blocks {
                assert!(layout.is_block_inserted(block));
                assert_eq!(block_iter.next(), Some(block));
                assert!(layout.blocks[block].seq > seq);
                seq = layout.blocks[block].seq;

                let mut inst_iter = layout.block_insts(block);
                for &inst in insts {
                    assert_eq!(layout.inst_block(inst), Some(block));
                    assert_eq!(inst_iter.next(), Some(inst));
                    assert!(layout.insts[inst].seq > seq);
                    seq = layout.insts[inst].seq;
                }
                assert_eq!(inst_iter.next(), None);
            }
            assert_eq!(block_iter.next(), None);
        }

        // Check backwards linkage with a cursor.
        let mut cur = LayoutCursor::new(layout);
        for &(block, insts) in blocks.into_iter().rev() {
            assert_eq!(cur.prev_block(), Some(block));
            for &inst in insts.into_iter().rev() {
                assert_eq!(cur.prev_inst(), Some(inst));
            }
            assert_eq!(cur.prev_inst(), None);
        }
        assert_eq!(cur.prev_block(), None);
    }

    #[test]
    fn append_block() {
        let mut layout = Layout::new();
        let e0 = Block::new(0);
        let e1 = Block::new(1);
        let e2 = Block::new(2);

        {
            let imm = &layout;
            assert!(!imm.is_block_inserted(e0));
            assert!(!imm.is_block_inserted(e1));
        }
        verify(&mut layout, &[]);

        layout.append_block(e1);
        assert!(!layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(!layout.is_block_inserted(e2));
        let v: Vec<Block> = layout.blocks().collect();
        assert_eq!(v, [e1]);

        layout.append_block(e2);
        assert!(!layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(layout.is_block_inserted(e2));
        let v: Vec<Block> = layout.blocks().collect();
        assert_eq!(v, [e1, e2]);

        layout.append_block(e0);
        assert!(layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(layout.is_block_inserted(e2));
        let v: Vec<Block> = layout.blocks().collect();
        assert_eq!(v, [e1, e2, e0]);

        {
            let imm = &layout;
            let mut v = Vec::new();
            for e in imm {
                v.push(e);
            }
            assert_eq!(v, [e1, e2, e0]);
        }

        // Test cursor positioning.
        let mut cur = LayoutCursor::new(&mut layout);
        assert_eq!(cur.position(), CursorPosition::Nowhere);
        assert_eq!(cur.next_inst(), None);
        assert_eq!(cur.position(), CursorPosition::Nowhere);
        assert_eq!(cur.prev_inst(), None);
        assert_eq!(cur.position(), CursorPosition::Nowhere);

        assert_eq!(cur.next_block(), Some(e1));
        assert_eq!(cur.position(), CursorPosition::Before(e1));
        assert_eq!(cur.next_inst(), None);
        assert_eq!(cur.position(), CursorPosition::After(e1));
        assert_eq!(cur.next_inst(), None);
        assert_eq!(cur.position(), CursorPosition::After(e1));
        assert_eq!(cur.next_block(), Some(e2));
        assert_eq!(cur.prev_inst(), None);
        assert_eq!(cur.position(), CursorPosition::Before(e2));
        assert_eq!(cur.next_block(), Some(e0));
        assert_eq!(cur.next_block(), None);
        assert_eq!(cur.position(), CursorPosition::Nowhere);

        // Backwards through the blocks.
        assert_eq!(cur.prev_block(), Some(e0));
        assert_eq!(cur.position(), CursorPosition::After(e0));
        assert_eq!(cur.prev_block(), Some(e2));
        assert_eq!(cur.prev_block(), Some(e1));
        assert_eq!(cur.prev_block(), None);
        assert_eq!(cur.position(), CursorPosition::Nowhere);
    }

    #[test]
    fn insert_block() {
        let mut layout = Layout::new();
        let e0 = Block::new(0);
        let e1 = Block::new(1);
        let e2 = Block::new(2);

        {
            let imm = &layout;
            assert!(!imm.is_block_inserted(e0));
            assert!(!imm.is_block_inserted(e1));

            let v: Vec<Block> = layout.blocks().collect();
            assert_eq!(v, []);
        }

        layout.append_block(e1);
        assert!(!layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(!layout.is_block_inserted(e2));
        verify(&mut layout, &[(e1, &[])]);

        layout.insert_block(e2, e1);
        assert!(!layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(layout.is_block_inserted(e2));
        verify(&mut layout, &[(e2, &[]), (e1, &[])]);

        layout.insert_block(e0, e1);
        assert!(layout.is_block_inserted(e0));
        assert!(layout.is_block_inserted(e1));
        assert!(layout.is_block_inserted(e2));
        verify(&mut layout, &[(e2, &[]), (e0, &[]), (e1, &[])]);
    }

    #[test]
    fn insert_block_after() {
        let mut layout = Layout::new();
        let e0 = Block::new(0);
        let e1 = Block::new(1);
        let e2 = Block::new(2);

        layout.append_block(e1);
        layout.insert_block_after(e2, e1);
        verify(&mut layout, &[(e1, &[]), (e2, &[])]);

        layout.insert_block_after(e0, e1);
        verify(&mut layout, &[(e1, &[]), (e0, &[]), (e2, &[])]);
    }

    #[test]
    fn append_inst() {
        let mut layout = Layout::new();
        let e1 = Block::new(1);

        layout.append_block(e1);
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, []);

        let i0 = Inst::new(0);
        let i1 = Inst::new(1);
        let i2 = Inst::new(2);

        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), None);
        assert_eq!(layout.inst_block(i2), None);

        layout.append_inst(i1, e1);
        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), Some(e1));
        assert_eq!(layout.inst_block(i2), None);
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, [i1]);

        layout.append_inst(i2, e1);
        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), Some(e1));
        assert_eq!(layout.inst_block(i2), Some(e1));
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, [i1, i2]);

        // Test double-ended instruction iterator.
        let v: Vec<Inst> = layout.block_insts(e1).rev().collect();
        assert_eq!(v, [i2, i1]);

        layout.append_inst(i0, e1);
        verify(&mut layout, &[(e1, &[i1, i2, i0])]);

        // Test cursor positioning.
        let mut cur = LayoutCursor::new(&mut layout).at_top(e1);
        assert_eq!(cur.position(), CursorPosition::Before(e1));
        assert_eq!(cur.prev_inst(), None);
        assert_eq!(cur.position(), CursorPosition::Before(e1));
        assert_eq!(cur.next_inst(), Some(i1));
        assert_eq!(cur.position(), CursorPosition::At(i1));
        assert_eq!(cur.next_inst(), Some(i2));
        assert_eq!(cur.next_inst(), Some(i0));
        assert_eq!(cur.prev_inst(), Some(i2));
        assert_eq!(cur.position(), CursorPosition::At(i2));
        assert_eq!(cur.next_inst(), Some(i0));
        assert_eq!(cur.position(), CursorPosition::At(i0));
        assert_eq!(cur.next_inst(), None);
        assert_eq!(cur.position(), CursorPosition::After(e1));
        assert_eq!(cur.next_inst(), None);
        assert_eq!(cur.position(), CursorPosition::After(e1));
        assert_eq!(cur.prev_inst(), Some(i0));
        assert_eq!(cur.prev_inst(), Some(i2));
        assert_eq!(cur.prev_inst(), Some(i1));
        assert_eq!(cur.prev_inst(), None);
        assert_eq!(cur.position(), CursorPosition::Before(e1));

        // Test remove_inst.
        cur.goto_inst(i2);
        assert_eq!(cur.remove_inst(), i2);
        verify(cur.layout, &[(e1, &[i1, i0])]);
        assert_eq!(cur.layout.inst_block(i2), None);
        assert_eq!(cur.remove_inst(), i0);
        verify(cur.layout, &[(e1, &[i1])]);
        assert_eq!(cur.layout.inst_block(i0), None);
        assert_eq!(cur.position(), CursorPosition::After(e1));
        cur.layout.remove_inst(i1);
        verify(cur.layout, &[(e1, &[])]);
        assert_eq!(cur.layout.inst_block(i1), None);
    }

    #[test]
    fn insert_inst() {
        let mut layout = Layout::new();
        let e1 = Block::new(1);

        layout.append_block(e1);
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, []);

        let i0 = Inst::new(0);
        let i1 = Inst::new(1);
        let i2 = Inst::new(2);

        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), None);
        assert_eq!(layout.inst_block(i2), None);

        layout.append_inst(i1, e1);
        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), Some(e1));
        assert_eq!(layout.inst_block(i2), None);
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, [i1]);

        layout.insert_inst(i2, i1);
        assert_eq!(layout.inst_block(i0), None);
        assert_eq!(layout.inst_block(i1), Some(e1));
        assert_eq!(layout.inst_block(i2), Some(e1));
        let v: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v, [i2, i1]);

        layout.insert_inst(i0, i1);
        verify(&mut layout, &[(e1, &[i2, i0, i1])]);
    }

    #[test]
    fn multiple_blocks() {
        let mut layout = Layout::new();

        let e0 = Block::new(0);
        let e1 = Block::new(1);

        assert_eq!(layout.entry_block(), None);
        layout.append_block(e0);
        assert_eq!(layout.entry_block(), Some(e0));
        layout.append_block(e1);
        assert_eq!(layout.entry_block(), Some(e0));

        let i0 = Inst::new(0);
        let i1 = Inst::new(1);
        let i2 = Inst::new(2);
        let i3 = Inst::new(3);

        layout.append_inst(i0, e0);
        layout.append_inst(i1, e0);
        layout.append_inst(i2, e1);
        layout.append_inst(i3, e1);

        let v0: Vec<Inst> = layout.block_insts(e0).collect();
        let v1: Vec<Inst> = layout.block_insts(e1).collect();
        assert_eq!(v0, [i0, i1]);
        assert_eq!(v1, [i2, i3]);
    }

    #[test]
    fn split_block() {
        let mut layout = Layout::new();

        let e0 = Block::new(0);
        let e1 = Block::new(1);
        let e2 = Block::new(2);

        let i0 = Inst::new(0);
        let i1 = Inst::new(1);
        let i2 = Inst::new(2);
        let i3 = Inst::new(3);

        layout.append_block(e0);
        layout.append_inst(i0, e0);
        assert_eq!(layout.inst_block(i0), Some(e0));
        layout.split_block(e1, i0);
        assert_eq!(layout.inst_block(i0), Some(e1));

        {
            let mut cur = LayoutCursor::new(&mut layout);
            assert_eq!(cur.next_block(), Some(e0));
            assert_eq!(cur.next_inst(), None);
            assert_eq!(cur.next_block(), Some(e1));
            assert_eq!(cur.next_inst(), Some(i0));
            assert_eq!(cur.next_inst(), None);
            assert_eq!(cur.next_block(), None);

            // Check backwards links.
            assert_eq!(cur.prev_block(), Some(e1));
            assert_eq!(cur.prev_inst(), Some(i0));
            assert_eq!(cur.prev_inst(), None);
            assert_eq!(cur.prev_block(), Some(e0));
            assert_eq!(cur.prev_inst(), None);
            assert_eq!(cur.prev_block(), None);
        }

        layout.append_inst(i1, e0);
        layout.append_inst(i2, e0);
        layout.append_inst(i3, e0);
        layout.split_block(e2, i2);

        assert_eq!(layout.inst_block(i0), Some(e1));
        assert_eq!(layout.inst_block(i1), Some(e0));
        assert_eq!(layout.inst_block(i2), Some(e2));
        assert_eq!(layout.inst_block(i3), Some(e2));

        {
            let mut cur = LayoutCursor::new(&mut layout);
            assert_eq!(cur.next_block(), Some(e0));
            assert_eq!(cur.next_inst(), Some(i1));
            assert_eq!(cur.next_inst(), None);
            assert_eq!(cur.next_block(), Some(e2));
            assert_eq!(cur.next_inst(), Some(i2));
            assert_eq!(cur.next_inst(), Some(i3));
            assert_eq!(cur.next_inst(), None);
            assert_eq!(cur.next_block(), Some(e1));
            assert_eq!(cur.next_inst(), Some(i0));
            assert_eq!(cur.next_inst(), None);
            assert_eq!(cur.next_block(), None);

            assert_eq!(cur.prev_block(), Some(e1));
            assert_eq!(cur.prev_inst(), Some(i0));
            assert_eq!(cur.prev_inst(), None);
            assert_eq!(cur.prev_block(), Some(e2));
            assert_eq!(cur.prev_inst(), Some(i3));
            assert_eq!(cur.prev_inst(), Some(i2));
            assert_eq!(cur.prev_inst(), None);
            assert_eq!(cur.prev_block(), Some(e0));
            assert_eq!(cur.prev_inst(), Some(i1));
            assert_eq!(cur.prev_inst(), None);
            assert_eq!(cur.prev_block(), None);
        }

        // Check `ProgramOrder`.
        assert_eq!(layout.cmp(e2, e2), Ordering::Equal);
        assert_eq!(layout.cmp(e2, i2), Ordering::Less);
        assert_eq!(layout.cmp(i3, i2), Ordering::Greater);

        assert_eq!(layout.is_block_gap(i1, e2), true);
        assert_eq!(layout.is_block_gap(i3, e1), true);
        assert_eq!(layout.is_block_gap(i1, e1), false);
        assert_eq!(layout.is_block_gap(i2, e1), false);
    }
}
