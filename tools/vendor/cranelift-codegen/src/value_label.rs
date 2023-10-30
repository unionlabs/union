use crate::ir::{SourceLoc, ValueLabel};
use crate::HashMap;
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::convert::From;
use core::ops::Deref;
use regalloc::Reg;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Value location range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct ValueLocRange {
    /// The ValueLoc containing a ValueLabel during this range.
    pub loc: LabelValueLoc,
    /// The start of the range. It is an offset in the generated code.
    pub start: u32,
    /// The end of the range. It is an offset in the generated code.
    pub end: u32,
}

/// The particular location for a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum LabelValueLoc {
    /// New-backend Reg.
    Reg(Reg),
    /// New-backend offset from stack pointer.
    SPOffset(i64),
}

/// Resulting map of Value labels and their ranges/locations.
pub type ValueLabelsRanges = HashMap<ValueLabel, Vec<ValueLocRange>>;

#[derive(Eq, Clone, Copy)]
pub struct ComparableSourceLoc(SourceLoc);

impl From<SourceLoc> for ComparableSourceLoc {
    fn from(s: SourceLoc) -> Self {
        Self(s)
    }
}

impl Deref for ComparableSourceLoc {
    type Target = SourceLoc;
    fn deref(&self) -> &SourceLoc {
        &self.0
    }
}

impl PartialOrd for ComparableSourceLoc {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComparableSourceLoc {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.bits().cmp(&other.0.bits())
    }
}

impl PartialEq for ComparableSourceLoc {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
