//! Pretty-printing for the main data structures.

use crate::data_structures::WritableBase;
use crate::{RealRegUniverse, Reg, Writable};

/// A trait for printing instruction bits and pieces, with the the ability to take a
/// contextualising `RealRegUniverse` that is used to give proper names to registers.
pub trait PrettyPrint {
    /// Return a string that shows the implementing object in context of the given
    /// `RealRegUniverse`, if provided.
    fn show_rru(&self, maybe_reg_universe: Option<&RealRegUniverse>) -> String;
}

/// Same as `PrettyPrint`, but can also take a size hint into account to specialize the displayed
/// string.
pub trait PrettyPrintSized: PrettyPrint {
    /// The same as |show_rru|, but with an optional hint giving a size in bytes. Its
    /// interpretation is object-dependent, and it is intended to pass around enough information to
    /// facilitate printing sub-parts of real registers correctly. Objects may ignore size hints
    /// that are irrelevant to them.
    ///
    /// The default implementation ignores the size hint.
    fn show_rru_sized(&self, maybe_reg_universe: Option<&RealRegUniverse>, _size: u8) -> String {
        self.show_rru(maybe_reg_universe)
    }
}

impl PrettyPrint for Reg {
    fn show_rru(&self, maybe_reg_universe: Option<&RealRegUniverse>) -> String {
        if self.is_real() {
            if let Some(rru) = maybe_reg_universe {
                let reg_ix = self.get_index();
                assert!(
                    reg_ix < rru.regs.len(),
                    "unknown real register with index {:?}",
                    reg_ix
                );
                return rru.regs[reg_ix].1.to_string();
            }
        }
        // The reg is virtual, or we have no universe. Be generic.
        format!("%{:?}", self)
    }
}

impl<R: PrettyPrint + WritableBase> PrettyPrint for Writable<R> {
    fn show_rru(&self, maybe_reg_universe: Option<&RealRegUniverse>) -> String {
        self.to_reg().show_rru(maybe_reg_universe)
    }
}

impl<R: PrettyPrintSized + WritableBase> PrettyPrintSized for Writable<R> {
    fn show_rru_sized(&self, maybe_reg_universe: Option<&RealRegUniverse>, size: u8) -> String {
        self.to_reg().show_rru_sized(maybe_reg_universe, size)
    }
}
