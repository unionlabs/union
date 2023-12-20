//! S390x ISA definitions: instruction arguments.

use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::MemFlags;
use crate::isa::s390x::inst::*;
use crate::machinst::MachLabel;
use crate::machinst::{PrettyPrint, Reg};

use std::string::String;

//=============================================================================
// Instruction sub-components (memory addresses): definitions

/// A memory argument to load/store, encapsulating the possible addressing modes.
#[derive(Clone, Debug)]
pub enum MemArg {
    //
    // Real IBM Z addressing modes:
    //
    /// Base register, index register, and 12-bit unsigned displacement.
    BXD12 {
        base: Reg,
        index: Reg,
        disp: UImm12,
        flags: MemFlags,
    },

    /// Base register, index register, and 20-bit signed displacement.
    BXD20 {
        base: Reg,
        index: Reg,
        disp: SImm20,
        flags: MemFlags,
    },

    /// PC-relative Reference to a label.
    Label { target: MachLabel },

    /// PC-relative Reference to a near symbol.
    Symbol {
        name: Box<ExternalName>,
        offset: i32,
        flags: MemFlags,
    },

    //
    // Virtual addressing modes that are lowered at emission time:
    //
    /// Arbitrary offset from a register. Converted to generation of large
    /// offsets with multiple instructions as necessary during code emission.
    RegOffset { reg: Reg, off: i64, flags: MemFlags },

    /// Offset from the stack pointer at function entry.
    InitialSPOffset { off: i64 },

    /// Offset from the "nominal stack pointer", which is where the real SP is
    /// just after stack and spill slots are allocated in the function prologue.
    /// At emission time, this is converted to `SPOffset` with a fixup added to
    /// the offset constant. The fixup is a running value that is tracked as
    /// emission iterates through instructions in linear order, and can be
    /// adjusted up and down with [Inst::VirtualSPOffsetAdj].
    ///
    /// The standard ABI is in charge of handling this (by emitting the
    /// adjustment meta-instructions). It maintains the invariant that "nominal
    /// SP" is where the actual SP is after the function prologue and before
    /// clobber pushes. See the diagram in the documentation for
    /// [crate::isa::s390x::abi](the ABI module) for more details.
    NominalSPOffset { off: i64 },
}

impl MemArg {
    /// Memory reference using an address in a register.
    pub fn reg(reg: Reg, flags: MemFlags) -> MemArg {
        MemArg::BXD12 {
            base: reg,
            index: zero_reg(),
            disp: UImm12::zero(),
            flags,
        }
    }

    /// Memory reference using the sum of two registers as an address.
    pub fn reg_plus_reg(reg1: Reg, reg2: Reg, flags: MemFlags) -> MemArg {
        MemArg::BXD12 {
            base: reg1,
            index: reg2,
            disp: UImm12::zero(),
            flags,
        }
    }

    /// Memory reference using the sum of a register an an offset as address.
    pub fn reg_plus_off(reg: Reg, off: i64, flags: MemFlags) -> MemArg {
        MemArg::RegOffset { reg, off, flags }
    }

    pub(crate) fn get_flags(&self) -> MemFlags {
        match self {
            MemArg::BXD12 { flags, .. } => *flags,
            MemArg::BXD20 { flags, .. } => *flags,
            MemArg::RegOffset { flags, .. } => *flags,
            MemArg::Label { .. } => MemFlags::trusted(),
            MemArg::Symbol { flags, .. } => *flags,
            MemArg::InitialSPOffset { .. } => MemFlags::trusted(),
            MemArg::NominalSPOffset { .. } => MemFlags::trusted(),
        }
    }

    pub(crate) fn can_trap(&self) -> bool {
        !self.get_flags().notrap()
    }

    /// Edit registers with allocations.
    pub fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        match self {
            &MemArg::BXD12 {
                base,
                index,
                disp,
                flags,
            } => MemArg::BXD12 {
                base: allocs.next(base),
                index: allocs.next(index),
                disp,
                flags,
            },
            &MemArg::BXD20 {
                base,
                index,
                disp,
                flags,
            } => MemArg::BXD20 {
                base: allocs.next(base),
                index: allocs.next(index),
                disp,
                flags,
            },
            &MemArg::RegOffset { reg, off, flags } => MemArg::RegOffset {
                reg: allocs.next(reg),
                off,
                flags,
            },
            x => x.clone(),
        }
    }
}

/// A memory argument for an instruction with two memory operands.
/// We cannot use two instances of MemArg, because we do not have
/// two free temp registers that would be needed to reload two
/// addresses in the general case.  Also, two copies of MemArg would
/// increase the size of Inst beyond its current limit.  Use this
/// simplified form instead that never needs any reloads, and suffices
/// for all current users.
#[derive(Clone, Debug)]
pub struct MemArgPair {
    pub base: Reg,
    pub disp: UImm12,
    pub flags: MemFlags,
}

impl MemArgPair {
    /// Convert a MemArg to a MemArgPair if possible.
    pub fn maybe_from_memarg(mem: &MemArg) -> Option<MemArgPair> {
        match mem {
            &MemArg::BXD12 {
                base,
                index,
                disp,
                flags,
            } => {
                if index != zero_reg() {
                    None
                } else {
                    Some(MemArgPair { base, disp, flags })
                }
            }
            &MemArg::RegOffset { reg, off, flags } => {
                if off < 0 {
                    None
                } else {
                    let disp = UImm12::maybe_from_u64(off as u64)?;
                    Some(MemArgPair {
                        base: reg,
                        disp,
                        flags,
                    })
                }
            }
            _ => None,
        }
    }

    pub(crate) fn can_trap(&self) -> bool {
        !self.flags.notrap()
    }

    /// Edit registers with allocations.
    pub fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        MemArgPair {
            base: allocs.next(self.base),
            disp: self.disp,
            flags: self.flags,
        }
    }
}

//=============================================================================
// Instruction sub-components (conditions, branches and branch targets):
// definitions

/// Condition for conditional branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cond {
    mask: u8,
}

impl Cond {
    pub fn from_mask(mask: u8) -> Cond {
        assert!(mask >= 1 && mask <= 14);
        Cond { mask }
    }

    pub fn from_intcc(cc: IntCC) -> Cond {
        let mask = match cc {
            IntCC::Equal => 8,
            IntCC::NotEqual => 4 | 2,
            IntCC::SignedGreaterThanOrEqual => 8 | 2,
            IntCC::SignedGreaterThan => 2,
            IntCC::SignedLessThanOrEqual => 8 | 4,
            IntCC::SignedLessThan => 4,
            IntCC::UnsignedGreaterThanOrEqual => 8 | 2,
            IntCC::UnsignedGreaterThan => 2,
            IntCC::UnsignedLessThanOrEqual => 8 | 4,
            IntCC::UnsignedLessThan => 4,
        };
        Cond { mask }
    }

    pub fn from_floatcc(cc: FloatCC) -> Cond {
        let mask = match cc {
            FloatCC::Ordered => 8 | 4 | 2,
            FloatCC::Unordered => 1,
            FloatCC::Equal => 8,
            FloatCC::NotEqual => 4 | 2 | 1,
            FloatCC::OrderedNotEqual => 4 | 2,
            FloatCC::UnorderedOrEqual => 8 | 1,
            FloatCC::LessThan => 4,
            FloatCC::LessThanOrEqual => 8 | 4,
            FloatCC::GreaterThan => 2,
            FloatCC::GreaterThanOrEqual => 8 | 2,
            FloatCC::UnorderedOrLessThan => 4 | 1,
            FloatCC::UnorderedOrLessThanOrEqual => 8 | 4 | 1,
            FloatCC::UnorderedOrGreaterThan => 2 | 1,
            FloatCC::UnorderedOrGreaterThanOrEqual => 8 | 2 | 1,
        };
        Cond { mask }
    }

    /// Return the inverted condition.
    pub fn invert(self) -> Cond {
        Cond {
            mask: !self.mask & 15,
        }
    }

    /// Return the machine encoding of this condition.
    pub fn bits(self) -> u8 {
        self.mask
    }
}

impl PrettyPrint for MemArg {
    fn pretty_print(&self, _: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        match self {
            &MemArg::BXD12 {
                base, index, disp, ..
            } => {
                let base = allocs.next(base);
                let index = allocs.next(index);
                if base != zero_reg() {
                    if index != zero_reg() {
                        format!(
                            "{}({},{})",
                            disp.pretty_print_default(),
                            show_reg(index),
                            show_reg(base),
                        )
                    } else {
                        format!("{}({})", disp.pretty_print_default(), show_reg(base))
                    }
                } else {
                    if index != zero_reg() {
                        format!("{}({},)", disp.pretty_print_default(), show_reg(index))
                    } else {
                        format!("{}", disp.pretty_print_default())
                    }
                }
            }
            &MemArg::BXD20 {
                base, index, disp, ..
            } => {
                let base = allocs.next(base);
                let index = allocs.next(index);
                if base != zero_reg() {
                    if index != zero_reg() {
                        format!(
                            "{}({},{})",
                            disp.pretty_print_default(),
                            show_reg(index),
                            show_reg(base),
                        )
                    } else {
                        format!("{}({})", disp.pretty_print_default(), show_reg(base))
                    }
                } else {
                    if index != zero_reg() {
                        format!("{}({},)", disp.pretty_print_default(), show_reg(index))
                    } else {
                        format!("{}", disp.pretty_print_default())
                    }
                }
            }
            &MemArg::Label { target } => target.to_string(),
            &MemArg::Symbol {
                ref name, offset, ..
            } => format!("{} + {}", name.display(None), offset),
            // Eliminated by `mem_finalize()`.
            &MemArg::InitialSPOffset { .. }
            | &MemArg::NominalSPOffset { .. }
            | &MemArg::RegOffset { .. } => {
                panic!("Unexpected pseudo mem-arg mode (stack-offset or generic reg-offset)!")
            }
        }
    }
}

impl PrettyPrint for Cond {
    fn pretty_print(&self, _: u8, _: &mut AllocationConsumer<'_>) -> String {
        let s = match self.mask {
            1 => "o",
            2 => "h",
            3 => "nle",
            4 => "l",
            5 => "nhe",
            6 => "lh",
            7 => "ne",
            8 => "e",
            9 => "nlh",
            10 => "he",
            11 => "nl",
            12 => "le",
            13 => "nh",
            14 => "no",
            _ => unreachable!(),
        };
        s.to_string()
    }
}
