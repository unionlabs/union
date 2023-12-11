//! Memory operation flags.

use core::fmt;

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

enum FlagBit {
    Notrap,
    Aligned,
    Readonly,
    LittleEndian,
    BigEndian,
    /// Accesses only the "heap" part of abstract state. Used for
    /// alias analysis. Mutually exclusive with "table" and "vmctx".
    Heap,
    /// Accesses only the "table" part of abstract state. Used for
    /// alias analysis. Mutually exclusive with "heap" and "vmctx".
    Table,
    /// Accesses only the "vmctx" part of abstract state. Used for
    /// alias analysis. Mutually exclusive with "heap" and "table".
    Vmctx,
}

const NAMES: [&str; 8] = [
    "notrap", "aligned", "readonly", "little", "big", "heap", "table", "vmctx",
];

/// Endianness of a memory access.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Endianness {
    /// Little-endian
    Little,
    /// Big-endian
    Big,
}

/// Flags for memory operations like load/store.
///
/// Each of these flags introduce a limited form of undefined behavior. The flags each enable
/// certain optimizations that need to make additional assumptions. Generally, the semantics of a
/// program does not change when a flag is removed, but adding a flag will.
///
/// In addition, the flags determine the endianness of the memory access.  By default,
/// any memory access uses the native endianness determined by the target ISA.  This can
/// be overridden for individual accesses by explicitly specifying little- or big-endian
/// semantics via the flags.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct MemFlags {
    bits: u8,
}

impl MemFlags {
    /// Create a new empty set of flags.
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    /// Create a set of flags representing an access from a "trusted" address, meaning it's
    /// known to be aligned and non-trapping.
    pub fn trusted() -> Self {
        let mut result = Self::new();
        result.set_notrap();
        result.set_aligned();
        result
    }

    /// Read a flag bit.
    fn read(self, bit: FlagBit) -> bool {
        self.bits & (1 << bit as usize) != 0
    }

    /// Set a flag bit.
    fn set(&mut self, bit: FlagBit) {
        self.bits |= 1 << bit as usize
    }

    /// Set a flag bit by name.
    ///
    /// Returns true if the flag was found and set, false for an unknown flag name.
    /// Will also return false when trying to set inconsistent endianness flags.
    pub fn set_by_name(&mut self, name: &str) -> bool {
        match NAMES.iter().position(|&s| s == name) {
            Some(bit) => {
                let bits = self.bits | 1 << bit;
                if (bits & (1 << FlagBit::LittleEndian as usize)) != 0
                    && (bits & (1 << FlagBit::BigEndian as usize)) != 0
                {
                    false
                } else {
                    self.bits = bits;
                    true
                }
            }
            None => false,
        }
    }

    /// Return endianness of the memory access.  This will return the endianness
    /// explicitly specified by the flags if any, and will default to the native
    /// endianness otherwise.  The native endianness has to be provided by the
    /// caller since it is not explicitly encoded in CLIF IR -- this allows a
    /// front end to create IR without having to know the target endianness.
    pub fn endianness(self, native_endianness: Endianness) -> Endianness {
        if self.read(FlagBit::LittleEndian) {
            Endianness::Little
        } else if self.read(FlagBit::BigEndian) {
            Endianness::Big
        } else {
            native_endianness
        }
    }

    /// Set endianness of the memory access.
    pub fn set_endianness(&mut self, endianness: Endianness) {
        match endianness {
            Endianness::Little => self.set(FlagBit::LittleEndian),
            Endianness::Big => self.set(FlagBit::BigEndian),
        };
        assert!(!(self.read(FlagBit::LittleEndian) && self.read(FlagBit::BigEndian)));
    }

    /// Set endianness of the memory access, returning new flags.
    pub fn with_endianness(mut self, endianness: Endianness) -> Self {
        self.set_endianness(endianness);
        self
    }

    /// Test if the `notrap` flag is set.
    ///
    /// Normally, trapping is part of the semantics of a load/store operation. If the platform
    /// would cause a trap when accessing the effective address, the Cranelift memory operation is
    /// also required to trap.
    ///
    /// The `notrap` flag tells Cranelift that the memory is *accessible*, which means that
    /// accesses will not trap. This makes it possible to delete an unused load or a dead store
    /// instruction.
    pub fn notrap(self) -> bool {
        self.read(FlagBit::Notrap)
    }

    /// Set the `notrap` flag.
    pub fn set_notrap(&mut self) {
        self.set(FlagBit::Notrap)
    }

    /// Set the `notrap` flag, returning new flags.
    pub fn with_notrap(mut self) -> Self {
        self.set_notrap();
        self
    }

    /// Test if the `aligned` flag is set.
    ///
    /// By default, Cranelift memory instructions work with any unaligned effective address. If the
    /// `aligned` flag is set, the instruction is permitted to trap or return a wrong result if the
    /// effective address is misaligned.
    pub fn aligned(self) -> bool {
        self.read(FlagBit::Aligned)
    }

    /// Set the `aligned` flag.
    pub fn set_aligned(&mut self) {
        self.set(FlagBit::Aligned)
    }

    /// Set the `aligned` flag, returning new flags.
    pub fn with_aligned(mut self) -> Self {
        self.set_aligned();
        self
    }

    /// Test if the `readonly` flag is set.
    ///
    /// Loads with this flag have no memory dependencies.
    /// This results in undefined behavior if the dereferenced memory is mutated at any time
    /// between when the function is called and when it is exited.
    pub fn readonly(self) -> bool {
        self.read(FlagBit::Readonly)
    }

    /// Set the `readonly` flag.
    pub fn set_readonly(&mut self) {
        self.set(FlagBit::Readonly)
    }

    /// Set the `readonly` flag, returning new flags.
    pub fn with_readonly(mut self) -> Self {
        self.set_readonly();
        self
    }

    /// Test if the `heap` bit is set.
    ///
    /// Loads and stores with this flag accesses the "heap" part of
    /// abstract state. This is disjoint from the "table", "vmctx",
    /// and "other" parts of abstract state. In concrete terms, this
    /// means that behavior is undefined if the same memory is also
    /// accessed by another load/store with one of the other
    /// alias-analysis bits (`table`, `vmctx`) set, or `heap` not set.
    pub fn heap(self) -> bool {
        self.read(FlagBit::Heap)
    }

    /// Set the `heap` bit. See the notes about mutual exclusion with
    /// other bits in `heap()`.
    pub fn set_heap(&mut self) {
        assert!(!self.table() && !self.vmctx());
        self.set(FlagBit::Heap);
    }

    /// Set the `heap` bit, returning new flags.
    pub fn with_heap(mut self) -> Self {
        self.set_heap();
        self
    }

    /// Test if the `table` bit is set.
    ///
    /// Loads and stores with this flag accesses the "table" part of
    /// abstract state. This is disjoint from the "heap", "vmctx",
    /// and "other" parts of abstract state. In concrete terms, this
    /// means that behavior is undefined if the same memory is also
    /// accessed by another load/store with one of the other
    /// alias-analysis bits (`heap`, `vmctx`) set, or `table` not set.
    pub fn table(self) -> bool {
        self.read(FlagBit::Table)
    }

    /// Set the `table` bit. See the notes about mutual exclusion with
    /// other bits in `table()`.
    pub fn set_table(&mut self) {
        assert!(!self.heap() && !self.vmctx());
        self.set(FlagBit::Table);
    }

    /// Set the `table` bit, returning new flags.
    pub fn with_table(mut self) -> Self {
        self.set_table();
        self
    }

    /// Test if the `vmctx` bit is set.
    ///
    /// Loads and stores with this flag accesses the "vmctx" part of
    /// abstract state. This is disjoint from the "heap", "table",
    /// and "other" parts of abstract state. In concrete terms, this
    /// means that behavior is undefined if the same memory is also
    /// accessed by another load/store with one of the other
    /// alias-analysis bits (`heap`, `table`) set, or `vmctx` not set.
    pub fn vmctx(self) -> bool {
        self.read(FlagBit::Vmctx)
    }

    /// Set the `vmctx` bit. See the notes about mutual exclusion with
    /// other bits in `vmctx()`.
    pub fn set_vmctx(&mut self) {
        assert!(!self.heap() && !self.table());
        self.set(FlagBit::Vmctx);
    }

    /// Set the `vmctx` bit, returning new flags.
    pub fn with_vmctx(mut self) -> Self {
        self.set_vmctx();
        self
    }
}

impl fmt::Display for MemFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, n) in NAMES.iter().enumerate() {
            if self.bits & (1 << i) != 0 {
                write!(f, " {}", n)?;
            }
        }
        Ok(())
    }
}
