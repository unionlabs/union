//! Tables.

use crate::ir::immediates::Uimm64;
use crate::ir::{GlobalValue, Type};
use core::fmt;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Information about a table declaration.
#[derive(Clone)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct TableData {
    /// Global value giving the address of the start of the table.
    pub base_gv: GlobalValue,

    /// Guaranteed minimum table size in elements. Table accesses before `min_size` don't need
    /// bounds checking.
    pub min_size: Uimm64,

    /// Global value giving the current bound of the table, in elements.
    pub bound_gv: GlobalValue,

    /// The size of a table element, in bytes.
    pub element_size: Uimm64,

    /// The index type for the table.
    pub index_type: Type,
}

impl fmt::Display for TableData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("dynamic")?;
        write!(
            f,
            " {}, min {}, bound {}, element_size {}, index_type {}",
            self.base_gv, self.min_size, self.bound_gv, self.element_size, self.index_type
        )
    }
}
