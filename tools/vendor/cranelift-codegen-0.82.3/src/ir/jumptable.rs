//! Jump table representation.
//!
//! Jump tables are declared in the preamble and assigned an `ir::entities::JumpTable` reference.
//! The actual table of destinations is stored in a `JumpTableData` struct defined in this module.

use crate::ir::entities::Block;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};
use core::slice::{Iter, IterMut};

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Contents of a jump table.
///
/// All jump tables use 0-based indexing and are densely populated.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct JumpTableData {
    // Table entries.
    table: Vec<Block>,
}

impl JumpTableData {
    /// Create a new empty jump table.
    pub fn new() -> Self {
        Self { table: Vec::new() }
    }

    /// Create a new empty jump table with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            table: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of table entries.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Append a table entry.
    pub fn push_entry(&mut self, dest: Block) {
        self.table.push(dest)
    }

    /// Checks if any of the entries branch to `block`.
    pub fn branches_to(&self, block: Block) -> bool {
        self.table.iter().any(|target_block| *target_block == block)
    }

    /// Access the whole table as a slice.
    pub fn as_slice(&self) -> &[Block] {
        self.table.as_slice()
    }

    /// Access the whole table as a mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [Block] {
        self.table.as_mut_slice()
    }

    /// Returns an iterator over the table.
    pub fn iter(&self) -> Iter<Block> {
        self.table.iter()
    }

    /// Returns an iterator that allows modifying each value.
    pub fn iter_mut(&mut self) -> IterMut<Block> {
        self.table.iter_mut()
    }

    /// Clears all entries in this jump table.
    pub fn clear(&mut self) {
        self.table.clear();
    }
}

impl Display for JumpTableData {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "jump_table [")?;
        match self.table.first() {
            None => (),
            Some(first) => write!(fmt, "{}", first)?,
        }
        for block in self.table.iter().skip(1) {
            write!(fmt, ", {}", block)?;
        }
        write!(fmt, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::JumpTableData;
    use crate::entity::EntityRef;
    use crate::ir::Block;
    use alloc::string::ToString;

    #[test]
    fn empty() {
        let jt = JumpTableData::new();

        assert_eq!(jt.as_slice().get(0), None);
        assert_eq!(jt.as_slice().get(10), None);

        assert_eq!(jt.to_string(), "jump_table []");

        let v = jt.as_slice();
        assert_eq!(v, []);
    }

    #[test]
    fn insert() {
        let e1 = Block::new(1);
        let e2 = Block::new(2);

        let mut jt = JumpTableData::new();

        jt.push_entry(e1);
        jt.push_entry(e2);
        jt.push_entry(e1);

        assert_eq!(jt.to_string(), "jump_table [block1, block2, block1]");

        let v = jt.as_slice();
        assert_eq!(v, [e1, e2, e1]);
    }
}
