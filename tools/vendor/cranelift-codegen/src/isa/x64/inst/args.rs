//! Instruction operand sub-components (aka "parts"): definitions and printing.

use super::regs::{self};
use super::EmitState;
use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::{MemFlags, Type};
use crate::isa::x64::inst::regs::pretty_print_reg;
use crate::isa::x64::inst::Inst;
use crate::machinst::*;
use regalloc2::VReg;
use smallvec::{smallvec, SmallVec};
use std::fmt;
use std::string::String;

/// An extenstion trait for converting `Writable{Xmm,Gpr}` to `Writable<Reg>`.
pub trait ToWritableReg {
    fn to_writable_reg(&self) -> Writable<Reg>;
}

/// An extension trait for converting `Writable<Reg>` to `Writable{Xmm,Gpr}`.
pub trait FromWritableReg: Sized {
    fn from_writable_reg(w: Writable<Reg>) -> Option<Self>;
}

/// A macro for defining a newtype of `Reg` that enforces some invariant about
/// the wrapped `Reg` (such as that it is of a particular register class).
macro_rules! newtype_of_reg {
    (
        $newtype_reg:ident,
        $newtype_writable_reg:ident,
        $newtype_option_writable_reg:ident,
        $newtype_reg_mem:ident,
        $newtype_reg_mem_imm:ident,
        $newtype_imm8_reg:ident,
        |$check_reg:ident| $check:expr
    ) => {
        /// A newtype wrapper around `Reg`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $newtype_reg(Reg);

        impl PartialEq<Reg> for $newtype_reg {
            fn eq(&self, other: &Reg) -> bool {
                self.0 == *other
            }
        }

        impl From<$newtype_reg> for Reg {
            fn from(r: $newtype_reg) -> Self {
                r.0
            }
        }

        impl $newtype_reg {
            /// Create this newtype from the given register, or return `None` if the register
            /// is not a valid instance of this newtype.
            pub fn new($check_reg: Reg) -> Option<Self> {
                if $check {
                    Some(Self($check_reg))
                } else {
                    None
                }
            }

            /// Get this newtype's underlying `Reg`.
            pub fn to_reg(self) -> Reg {
                self.0
            }
        }

        // Convenience impl so that people working with this newtype can use it
        // "just like" a plain `Reg`.
        //
        // NB: We cannot implement `DerefMut` because that would let people do
        // nasty stuff like `*my_gpr.deref_mut() = some_xmm_reg`, breaking the
        // invariants that `Gpr` provides.
        impl std::ops::Deref for $newtype_reg {
            type Target = Reg;

            fn deref(&self) -> &Reg {
                &self.0
            }
        }

        pub type $newtype_writable_reg = Writable<$newtype_reg>;

        #[allow(dead_code)] // Used by some newtypes and not others.
        pub type $newtype_option_writable_reg = Option<Writable<$newtype_reg>>;

        impl ToWritableReg for $newtype_writable_reg {
            fn to_writable_reg(&self) -> Writable<Reg> {
                Writable::from_reg(self.to_reg().to_reg())
            }
        }

        impl FromWritableReg for $newtype_writable_reg {
            fn from_writable_reg(w: Writable<Reg>) -> Option<Self> {
                Some(Writable::from_reg($newtype_reg::new(w.to_reg())?))
            }
        }

        /// A newtype wrapper around `RegMem` for general-purpose registers.
        #[derive(Clone, Debug)]
        pub struct $newtype_reg_mem(RegMem);

        impl From<$newtype_reg_mem> for RegMem {
            fn from(rm: $newtype_reg_mem) -> Self {
                rm.0
            }
        }

        impl From<$newtype_reg> for $newtype_reg_mem {
            fn from(r: $newtype_reg) -> Self {
                $newtype_reg_mem(RegMem::reg(r.into()))
            }
        }

        impl $newtype_reg_mem {
            /// Construct a `RegMem` newtype from the given `RegMem`, or return
            /// `None` if the `RegMem` is not a valid instance of this `RegMem`
            /// newtype.
            pub fn new(rm: RegMem) -> Option<Self> {
                match rm {
                    RegMem::Mem { addr: _ } => Some(Self(rm)),
                    RegMem::Reg { reg: $check_reg } if $check => Some(Self(rm)),
                    RegMem::Reg { reg: _ } => None,
                }
            }

            /// Convert this newtype into its underlying `RegMem`.
            pub fn to_reg_mem(self) -> RegMem {
                self.0
            }

            #[allow(dead_code)] // Used by some newtypes and not others.
            pub fn get_operands<F: Fn(VReg) -> VReg>(
                &self,
                collector: &mut OperandCollector<'_, F>,
            ) {
                self.0.get_operands(collector);
            }
        }
        impl PrettyPrint for $newtype_reg_mem {
            fn pretty_print(&self, size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
                self.0.pretty_print(size, allocs)
            }
        }

        /// A newtype wrapper around `RegMemImm`.
        #[derive(Clone, Debug)]
        pub struct $newtype_reg_mem_imm(RegMemImm);

        impl From<$newtype_reg_mem_imm> for RegMemImm {
            fn from(rmi: $newtype_reg_mem_imm) -> RegMemImm {
                rmi.0
            }
        }

        impl From<$newtype_reg> for $newtype_reg_mem_imm {
            fn from(r: $newtype_reg) -> Self {
                $newtype_reg_mem_imm(RegMemImm::reg(r.into()))
            }
        }

        impl $newtype_reg_mem_imm {
            /// Construct this newtype from the given `RegMemImm`, or return
            /// `None` if the `RegMemImm` is not a valid instance of this
            /// newtype.
            pub fn new(rmi: RegMemImm) -> Option<Self> {
                match rmi {
                    RegMemImm::Imm { .. } => Some(Self(rmi)),
                    RegMemImm::Mem { addr: _ } => Some(Self(rmi)),
                    RegMemImm::Reg { reg: $check_reg } if $check => Some(Self(rmi)),
                    RegMemImm::Reg { reg: _ } => None,
                }
            }

            /// Convert this newtype into its underlying `RegMemImm`.
            #[allow(dead_code)] // Used by some newtypes and not others.
            pub fn to_reg_mem_imm(self) -> RegMemImm {
                self.0
            }

            #[allow(dead_code)] // Used by some newtypes and not others.
            pub fn get_operands<F: Fn(VReg) -> VReg>(
                &self,
                collector: &mut OperandCollector<'_, F>,
            ) {
                self.0.get_operands(collector);
            }
        }

        impl PrettyPrint for $newtype_reg_mem_imm {
            fn pretty_print(&self, size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
                self.0.pretty_print(size, allocs)
            }
        }

        /// A newtype wrapper around `Imm8Reg`.
        #[derive(Clone, Debug)]
        #[allow(dead_code)] // Used by some newtypes and not others.
        pub struct $newtype_imm8_reg(Imm8Reg);

        impl From<$newtype_reg> for $newtype_imm8_reg {
            fn from(r: $newtype_reg) -> Self {
                Self(Imm8Reg::Reg { reg: r.to_reg() })
            }
        }

        impl $newtype_imm8_reg {
            /// Construct this newtype from the given `Imm8Reg`, or return
            /// `None` if the `Imm8Reg` is not a valid instance of this newtype.
            #[allow(dead_code)] // Used by some newtypes and not others.
            pub fn new(imm8_reg: Imm8Reg) -> Option<Self> {
                match imm8_reg {
                    Imm8Reg::Imm8 { .. } => Some(Self(imm8_reg)),
                    Imm8Reg::Reg { reg: $check_reg } if $check => Some(Self(imm8_reg)),
                    Imm8Reg::Reg { reg: _ } => None,
                }
            }

            /// Convert this newtype into its underlying `Imm8Reg`.
            #[allow(dead_code)] // Used by some newtypes and not others.
            pub fn to_imm8_reg(self) -> Imm8Reg {
                self.0
            }
        }
    };
}

// Define a newtype of `Reg` for general-purpose registers.
newtype_of_reg!(
    Gpr,
    WritableGpr,
    OptionWritableGpr,
    GprMem,
    GprMemImm,
    Imm8Gpr,
    |reg| reg.class() == RegClass::Int
);

// Define a newtype of `Reg` for XMM registers.
newtype_of_reg!(
    Xmm,
    WritableXmm,
    OptionWritableXmm,
    XmmMem,
    XmmMemImm,
    Imm8Xmm,
    |reg| reg.class() == RegClass::Float
);

// N.B.: `Amode` is defined in `inst.isle`. We add some convenience
// constructors here.

// Re-export the type from the ISLE generated code.
pub use crate::isa::x64::lower::isle::generated_code::Amode;

impl Amode {
    pub(crate) fn imm_reg(simm32: u32, base: Reg) -> Self {
        debug_assert!(base.class() == RegClass::Int);
        Self::ImmReg {
            simm32,
            base,
            flags: MemFlags::trusted(),
        }
    }

    pub(crate) fn imm_reg_reg_shift(simm32: u32, base: Gpr, index: Gpr, shift: u8) -> Self {
        debug_assert!(base.class() == RegClass::Int);
        debug_assert!(index.class() == RegClass::Int);
        debug_assert!(shift <= 3);
        Self::ImmRegRegShift {
            simm32,
            base,
            index,
            shift,
            flags: MemFlags::trusted(),
        }
    }

    pub(crate) fn rip_relative(target: MachLabel) -> Self {
        Self::RipRelative { target }
    }

    pub(crate) fn with_flags(&self, flags: MemFlags) -> Self {
        match self {
            &Self::ImmReg { simm32, base, .. } => Self::ImmReg {
                simm32,
                base,
                flags,
            },
            &Self::ImmRegRegShift {
                simm32,
                base,
                index,
                shift,
                ..
            } => Self::ImmRegRegShift {
                simm32,
                base,
                index,
                shift,
                flags,
            },
            _ => panic!("Amode {:?} cannot take memflags", self),
        }
    }

    /// Add the registers mentioned by `self` to `collector`.
    pub(crate) fn get_operands<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            Amode::ImmReg { base, .. } => {
                if *base != regs::rbp() && *base != regs::rsp() {
                    collector.reg_use(*base);
                }
            }
            Amode::ImmRegRegShift { base, index, .. } => {
                debug_assert_ne!(base.to_reg(), regs::rbp());
                debug_assert_ne!(base.to_reg(), regs::rsp());
                collector.reg_use(base.to_reg());
                debug_assert_ne!(index.to_reg(), regs::rbp());
                debug_assert_ne!(index.to_reg(), regs::rsp());
                collector.reg_use(index.to_reg());
            }
            Amode::RipRelative { .. } => {
                // RIP isn't involved in regalloc.
            }
        }
    }

    /// Same as `get_operands`, but add the registers in the "late" phase.
    pub(crate) fn get_operands_late<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            Amode::ImmReg { base, .. } => {
                collector.reg_late_use(*base);
            }
            Amode::ImmRegRegShift { base, index, .. } => {
                collector.reg_late_use(base.to_reg());
                collector.reg_late_use(index.to_reg());
            }
            Amode::RipRelative { .. } => {
                // RIP isn't involved in regalloc.
            }
        }
    }

    pub(crate) fn get_flags(&self) -> MemFlags {
        match self {
            Amode::ImmReg { flags, .. } | Amode::ImmRegRegShift { flags, .. } => *flags,
            Amode::RipRelative { .. } => MemFlags::trusted(),
        }
    }

    pub(crate) fn can_trap(&self) -> bool {
        !self.get_flags().notrap()
    }

    pub(crate) fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        // The order in which we consume allocs here must match the
        // order in which we produce operands in get_operands() above.
        match self {
            &Amode::ImmReg {
                simm32,
                base,
                flags,
            } => {
                let base = if base == regs::rsp() || base == regs::rbp() {
                    base
                } else {
                    allocs.next(base)
                };
                Amode::ImmReg {
                    simm32,
                    flags,
                    base,
                }
            }
            &Amode::ImmRegRegShift {
                simm32,
                base,
                index,
                shift,
                flags,
            } => Amode::ImmRegRegShift {
                simm32,
                shift,
                flags,
                base: Gpr::new(allocs.next(*base)).unwrap(),
                index: Gpr::new(allocs.next(*index)).unwrap(),
            },
            &Amode::RipRelative { target } => Amode::RipRelative { target },
        }
    }

    /// Offset the amode by a fixed offset.
    pub(crate) fn offset(&self, offset: u32) -> Self {
        let mut ret = self.clone();
        match &mut ret {
            &mut Amode::ImmReg { ref mut simm32, .. } => *simm32 += offset,
            &mut Amode::ImmRegRegShift { ref mut simm32, .. } => *simm32 += offset,
            _ => panic!("Cannot offset amode: {:?}", self),
        }
        ret
    }
}

impl PrettyPrint for Amode {
    fn pretty_print(&self, _size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        match self {
            Amode::ImmReg { simm32, base, .. } => {
                // Note: size is always 8; the address is 64 bits,
                // even if the addressed operand is smaller.
                format!("{}({})", *simm32 as i32, pretty_print_reg(*base, 8, allocs))
            }
            Amode::ImmRegRegShift {
                simm32,
                base,
                index,
                shift,
                ..
            } => format!(
                "{}({},{},{})",
                *simm32 as i32,
                pretty_print_reg(base.to_reg(), 8, allocs),
                pretty_print_reg(index.to_reg(), 8, allocs),
                1 << shift
            ),
            Amode::RipRelative { ref target } => format!("label{}(%rip)", target.get()),
        }
    }
}

/// A Memory Address. These denote a 64-bit value only.
/// Used for usual addressing modes as well as addressing modes used during compilation, when the
/// moving SP offset is not known.
#[derive(Clone, Debug)]
pub enum SyntheticAmode {
    /// A real amode.
    Real(Amode),

    /// A (virtual) offset to the "nominal SP" value, which will be recomputed as we push and pop
    /// within the function.
    NominalSPOffset { simm32: u32 },

    /// A virtual offset to a constant that will be emitted in the constant section of the buffer.
    ConstantOffset(VCodeConstant),
}

impl SyntheticAmode {
    pub(crate) fn nominal_sp_offset(simm32: u32) -> Self {
        SyntheticAmode::NominalSPOffset { simm32 }
    }

    /// Add the registers mentioned by `self` to `collector`.
    pub(crate) fn get_operands<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            SyntheticAmode::Real(addr) => addr.get_operands(collector),
            SyntheticAmode::NominalSPOffset { .. } => {
                // Nothing to do; the base is SP and isn't involved in regalloc.
            }
            SyntheticAmode::ConstantOffset(_) => {}
        }
    }

    /// Same as `get_operands`, but add the register in the "late" phase.
    pub(crate) fn get_operands_late<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            SyntheticAmode::Real(addr) => addr.get_operands_late(collector),
            SyntheticAmode::NominalSPOffset { .. } => {
                // Nothing to do; the base is SP and isn't involved in regalloc.
            }
            SyntheticAmode::ConstantOffset(_) => {}
        }
    }

    pub(crate) fn finalize(&self, state: &mut EmitState, buffer: &MachBuffer<Inst>) -> Amode {
        match self {
            SyntheticAmode::Real(addr) => addr.clone(),
            SyntheticAmode::NominalSPOffset { simm32 } => {
                let off = *simm32 as i64 + state.virtual_sp_offset;
                // TODO will require a sequence of add etc.
                assert!(
                    off <= u32::max_value() as i64,
                    "amode finalize: add sequence NYI"
                );
                Amode::imm_reg(off as u32, regs::rsp())
            }
            SyntheticAmode::ConstantOffset(c) => {
                Amode::rip_relative(buffer.get_label_for_constant(*c))
            }
        }
    }

    pub(crate) fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        match self {
            SyntheticAmode::Real(addr) => SyntheticAmode::Real(addr.with_allocs(allocs)),
            &SyntheticAmode::NominalSPOffset { .. } | &SyntheticAmode::ConstantOffset { .. } => {
                self.clone()
            }
        }
    }
}

impl Into<SyntheticAmode> for Amode {
    fn into(self) -> SyntheticAmode {
        SyntheticAmode::Real(self)
    }
}

impl Into<SyntheticAmode> for VCodeConstant {
    fn into(self) -> SyntheticAmode {
        SyntheticAmode::ConstantOffset(self)
    }
}

impl PrettyPrint for SyntheticAmode {
    fn pretty_print(&self, _size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        match self {
            // See note in `Amode` regarding constant size of `8`.
            SyntheticAmode::Real(addr) => addr.pretty_print(8, allocs),
            SyntheticAmode::NominalSPOffset { simm32 } => {
                format!("rsp({} + virtual offset)", *simm32 as i32)
            }
            SyntheticAmode::ConstantOffset(c) => format!("const({})", c.as_u32()),
        }
    }
}

/// An operand which is either an integer Register, a value in Memory or an Immediate.  This can
/// denote an 8, 16, 32 or 64 bit value.  For the Immediate form, in the 8- and 16-bit case, only
/// the lower 8 or 16 bits of `simm32` is relevant.  In the 64-bit case, the value denoted by
/// `simm32` is its sign-extension out to 64 bits.
#[derive(Clone, Debug)]
pub enum RegMemImm {
    Reg { reg: Reg },
    Mem { addr: SyntheticAmode },
    Imm { simm32: u32 },
}

impl RegMemImm {
    pub(crate) fn reg(reg: Reg) -> Self {
        debug_assert!(reg.class() == RegClass::Int || reg.class() == RegClass::Float);
        Self::Reg { reg }
    }
    pub(crate) fn mem(addr: impl Into<SyntheticAmode>) -> Self {
        Self::Mem { addr: addr.into() }
    }
    pub(crate) fn imm(simm32: u32) -> Self {
        Self::Imm { simm32 }
    }

    /// Asserts that in register mode, the reg class is the one that's expected.
    pub(crate) fn assert_regclass_is(&self, expected_reg_class: RegClass) {
        if let Self::Reg { reg } = self {
            debug_assert_eq!(reg.class(), expected_reg_class);
        }
    }

    /// Add the regs mentioned by `self` to `collector`.
    pub(crate) fn get_operands<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            Self::Reg { reg } => collector.reg_use(*reg),
            Self::Mem { addr } => addr.get_operands(collector),
            Self::Imm { .. } => {}
        }
    }

    pub(crate) fn to_reg(&self) -> Option<Reg> {
        match self {
            Self::Reg { reg } => Some(*reg),
            _ => None,
        }
    }

    pub(crate) fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        match self {
            Self::Reg { reg } => Self::Reg {
                reg: allocs.next(*reg),
            },
            Self::Mem { addr } => Self::Mem {
                addr: addr.with_allocs(allocs),
            },
            Self::Imm { .. } => self.clone(),
        }
    }
}

impl PrettyPrint for RegMemImm {
    fn pretty_print(&self, size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        match self {
            Self::Reg { reg } => pretty_print_reg(*reg, size, allocs),
            Self::Mem { addr } => addr.pretty_print(size, allocs),
            Self::Imm { simm32 } => format!("${}", *simm32 as i32),
        }
    }
}

/// An operand which is either an 8-bit integer immediate or a register.
#[derive(Clone, Debug)]
pub enum Imm8Reg {
    Imm8 { imm: u8 },
    Reg { reg: Reg },
}

impl From<u8> for Imm8Reg {
    fn from(imm: u8) -> Self {
        Self::Imm8 { imm }
    }
}

impl From<Reg> for Imm8Reg {
    fn from(reg: Reg) -> Self {
        Self::Reg { reg }
    }
}

/// An operand which is either an integer Register or a value in Memory.  This can denote an 8, 16,
/// 32, 64, or 128 bit value.
#[derive(Clone, Debug)]
pub enum RegMem {
    Reg { reg: Reg },
    Mem { addr: SyntheticAmode },
}

impl RegMem {
    pub(crate) fn reg(reg: Reg) -> Self {
        debug_assert!(reg.class() == RegClass::Int || reg.class() == RegClass::Float);
        Self::Reg { reg }
    }
    pub(crate) fn mem(addr: impl Into<SyntheticAmode>) -> Self {
        Self::Mem { addr: addr.into() }
    }
    /// Asserts that in register mode, the reg class is the one that's expected.
    pub(crate) fn assert_regclass_is(&self, expected_reg_class: RegClass) {
        if let Self::Reg { reg } = self {
            debug_assert_eq!(reg.class(), expected_reg_class);
        }
    }
    /// Add the regs mentioned by `self` to `collector`.
    pub(crate) fn get_operands<F: Fn(VReg) -> VReg>(
        &self,
        collector: &mut OperandCollector<'_, F>,
    ) {
        match self {
            RegMem::Reg { reg } => collector.reg_use(*reg),
            RegMem::Mem { addr, .. } => addr.get_operands(collector),
        }
    }
    pub(crate) fn to_reg(&self) -> Option<Reg> {
        match self {
            RegMem::Reg { reg } => Some(*reg),
            _ => None,
        }
    }

    pub(crate) fn with_allocs(&self, allocs: &mut AllocationConsumer<'_>) -> Self {
        match self {
            RegMem::Reg { reg } => RegMem::Reg {
                reg: allocs.next(*reg),
            },
            RegMem::Mem { addr } => RegMem::Mem {
                addr: addr.with_allocs(allocs),
            },
        }
    }
}

impl From<Writable<Reg>> for RegMem {
    fn from(r: Writable<Reg>) -> Self {
        RegMem::reg(r.to_reg())
    }
}

impl PrettyPrint for RegMem {
    fn pretty_print(&self, size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        match self {
            RegMem::Reg { reg } => pretty_print_reg(*reg, size, allocs),
            RegMem::Mem { addr, .. } => addr.pretty_print(size, allocs),
        }
    }
}

/// Some basic ALU operations.  TODO: maybe add Adc, Sbb.
#[derive(Copy, Clone, PartialEq)]
pub enum AluRmiROpcode {
    Add,
    Adc,
    Sub,
    Sbb,
    And,
    Or,
    Xor,
    /// The signless, non-extending (N x N -> N, for N in {32,64}) variant.
    Mul,
}

impl fmt::Debug for AluRmiROpcode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            AluRmiROpcode::Add => "add",
            AluRmiROpcode::Adc => "adc",
            AluRmiROpcode::Sub => "sub",
            AluRmiROpcode::Sbb => "sbb",
            AluRmiROpcode::And => "and",
            AluRmiROpcode::Or => "or",
            AluRmiROpcode::Xor => "xor",
            AluRmiROpcode::Mul => "imul",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for AluRmiROpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, PartialEq)]
pub enum UnaryRmROpcode {
    /// Bit-scan reverse.
    Bsr,
    /// Bit-scan forward.
    Bsf,
    /// Counts leading zeroes (Leading Zero CouNT).
    Lzcnt,
    /// Counts trailing zeroes (Trailing Zero CouNT).
    Tzcnt,
    /// Counts the number of ones (POPulation CouNT).
    Popcnt,
}

impl UnaryRmROpcode {
    pub(crate) fn available_from(&self) -> SmallVec<[InstructionSet; 2]> {
        match self {
            UnaryRmROpcode::Bsr | UnaryRmROpcode::Bsf => smallvec![],
            UnaryRmROpcode::Lzcnt => smallvec![InstructionSet::Lzcnt],
            UnaryRmROpcode::Tzcnt => smallvec![InstructionSet::BMI1],
            UnaryRmROpcode::Popcnt => smallvec![InstructionSet::Popcnt],
        }
    }
}

impl fmt::Debug for UnaryRmROpcode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnaryRmROpcode::Bsr => write!(fmt, "bsr"),
            UnaryRmROpcode::Bsf => write!(fmt, "bsf"),
            UnaryRmROpcode::Lzcnt => write!(fmt, "lzcnt"),
            UnaryRmROpcode::Tzcnt => write!(fmt, "tzcnt"),
            UnaryRmROpcode::Popcnt => write!(fmt, "popcnt"),
        }
    }
}

impl fmt::Display for UnaryRmROpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CmpOpcode {
    /// CMP instruction: compute `a - b` and set flags from result.
    Cmp,
    /// TEST instruction: compute `a & b` and set flags from result.
    Test,
}

#[derive(Debug)]
pub(crate) enum InstructionSet {
    SSE,
    SSE2,
    SSSE3,
    SSE41,
    SSE42,
    Popcnt,
    Lzcnt,
    BMI1,
    #[allow(dead_code)] // never constructed (yet).
    BMI2,
    FMA,
    AVX512BITALG,
    AVX512DQ,
    AVX512F,
    AVX512VBMI,
    AVX512VL,
}

/// Some SSE operations requiring 2 operands r/m and r.
#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)] // some variants here aren't used just yet
pub enum SseOpcode {
    Addps,
    Addpd,
    Addss,
    Addsd,
    Andps,
    Andpd,
    Andnps,
    Andnpd,
    Blendvpd,
    Blendvps,
    Comiss,
    Comisd,
    Cmpps,
    Cmppd,
    Cmpss,
    Cmpsd,
    Cvtdq2ps,
    Cvtdq2pd,
    Cvtpd2ps,
    Cvtps2pd,
    Cvtsd2ss,
    Cvtsd2si,
    Cvtsi2ss,
    Cvtsi2sd,
    Cvtss2si,
    Cvtss2sd,
    Cvttpd2dq,
    Cvttps2dq,
    Cvttss2si,
    Cvttsd2si,
    Divps,
    Divpd,
    Divss,
    Divsd,
    Insertps,
    Maxps,
    Maxpd,
    Maxss,
    Maxsd,
    Minps,
    Minpd,
    Minss,
    Minsd,
    Movaps,
    Movapd,
    Movd,
    Movdqa,
    Movdqu,
    Movlhps,
    Movmskps,
    Movmskpd,
    Movq,
    Movss,
    Movsd,
    Movups,
    Movupd,
    Mulps,
    Mulpd,
    Mulss,
    Mulsd,
    Orps,
    Orpd,
    Pabsb,
    Pabsw,
    Pabsd,
    Packssdw,
    Packsswb,
    Packusdw,
    Packuswb,
    Paddb,
    Paddd,
    Paddq,
    Paddw,
    Paddsb,
    Paddsw,
    Paddusb,
    Paddusw,
    Palignr,
    Pand,
    Pandn,
    Pavgb,
    Pavgw,
    Pblendvb,
    Pcmpeqb,
    Pcmpeqw,
    Pcmpeqd,
    Pcmpeqq,
    Pcmpgtb,
    Pcmpgtw,
    Pcmpgtd,
    Pcmpgtq,
    Pextrb,
    Pextrw,
    Pextrd,
    Pinsrb,
    Pinsrw,
    Pinsrd,
    Pmaddubsw,
    Pmaddwd,
    Pmaxsb,
    Pmaxsw,
    Pmaxsd,
    Pmaxub,
    Pmaxuw,
    Pmaxud,
    Pminsb,
    Pminsw,
    Pminsd,
    Pminub,
    Pminuw,
    Pminud,
    Pmovmskb,
    Pmovsxbd,
    Pmovsxbw,
    Pmovsxbq,
    Pmovsxwd,
    Pmovsxwq,
    Pmovsxdq,
    Pmovzxbd,
    Pmovzxbw,
    Pmovzxbq,
    Pmovzxwd,
    Pmovzxwq,
    Pmovzxdq,
    Pmuldq,
    Pmulhw,
    Pmulhuw,
    Pmulhrsw,
    Pmulld,
    Pmullw,
    Pmuludq,
    Por,
    Pshufb,
    Pshufd,
    Psllw,
    Pslld,
    Psllq,
    Psraw,
    Psrad,
    Psrlw,
    Psrld,
    Psrlq,
    Psubb,
    Psubd,
    Psubq,
    Psubw,
    Psubsb,
    Psubsw,
    Psubusb,
    Psubusw,
    Ptest,
    Punpckhbw,
    Punpckhwd,
    Punpcklbw,
    Punpcklwd,
    Pxor,
    Rcpss,
    Roundps,
    Roundpd,
    Roundss,
    Roundsd,
    Rsqrtss,
    Shufps,
    Sqrtps,
    Sqrtpd,
    Sqrtss,
    Sqrtsd,
    Subps,
    Subpd,
    Subss,
    Subsd,
    Ucomiss,
    Ucomisd,
    Unpcklps,
    Xorps,
    Xorpd,
}

impl SseOpcode {
    /// Which `InstructionSet` is the first supporting this opcode?
    pub(crate) fn available_from(&self) -> InstructionSet {
        use InstructionSet::*;
        match self {
            SseOpcode::Addps
            | SseOpcode::Addss
            | SseOpcode::Andps
            | SseOpcode::Andnps
            | SseOpcode::Comiss
            | SseOpcode::Cmpps
            | SseOpcode::Cmpss
            | SseOpcode::Cvtsi2ss
            | SseOpcode::Cvtss2si
            | SseOpcode::Cvttss2si
            | SseOpcode::Divps
            | SseOpcode::Divss
            | SseOpcode::Maxps
            | SseOpcode::Maxss
            | SseOpcode::Minps
            | SseOpcode::Minss
            | SseOpcode::Movaps
            | SseOpcode::Movlhps
            | SseOpcode::Movmskps
            | SseOpcode::Movss
            | SseOpcode::Movups
            | SseOpcode::Mulps
            | SseOpcode::Mulss
            | SseOpcode::Orps
            | SseOpcode::Rcpss
            | SseOpcode::Rsqrtss
            | SseOpcode::Shufps
            | SseOpcode::Sqrtps
            | SseOpcode::Sqrtss
            | SseOpcode::Subps
            | SseOpcode::Subss
            | SseOpcode::Ucomiss
            | SseOpcode::Unpcklps
            | SseOpcode::Xorps => SSE,

            SseOpcode::Addpd
            | SseOpcode::Addsd
            | SseOpcode::Andpd
            | SseOpcode::Andnpd
            | SseOpcode::Cmppd
            | SseOpcode::Cmpsd
            | SseOpcode::Comisd
            | SseOpcode::Cvtdq2ps
            | SseOpcode::Cvtdq2pd
            | SseOpcode::Cvtpd2ps
            | SseOpcode::Cvtps2pd
            | SseOpcode::Cvtsd2ss
            | SseOpcode::Cvtsd2si
            | SseOpcode::Cvtsi2sd
            | SseOpcode::Cvtss2sd
            | SseOpcode::Cvttpd2dq
            | SseOpcode::Cvttps2dq
            | SseOpcode::Cvttsd2si
            | SseOpcode::Divpd
            | SseOpcode::Divsd
            | SseOpcode::Maxpd
            | SseOpcode::Maxsd
            | SseOpcode::Minpd
            | SseOpcode::Minsd
            | SseOpcode::Movapd
            | SseOpcode::Movd
            | SseOpcode::Movmskpd
            | SseOpcode::Movq
            | SseOpcode::Movsd
            | SseOpcode::Movupd
            | SseOpcode::Movdqa
            | SseOpcode::Movdqu
            | SseOpcode::Mulpd
            | SseOpcode::Mulsd
            | SseOpcode::Orpd
            | SseOpcode::Packssdw
            | SseOpcode::Packsswb
            | SseOpcode::Packuswb
            | SseOpcode::Paddb
            | SseOpcode::Paddd
            | SseOpcode::Paddq
            | SseOpcode::Paddw
            | SseOpcode::Paddsb
            | SseOpcode::Paddsw
            | SseOpcode::Paddusb
            | SseOpcode::Paddusw
            | SseOpcode::Pand
            | SseOpcode::Pandn
            | SseOpcode::Pavgb
            | SseOpcode::Pavgw
            | SseOpcode::Pcmpeqb
            | SseOpcode::Pcmpeqw
            | SseOpcode::Pcmpeqd
            | SseOpcode::Pcmpgtb
            | SseOpcode::Pcmpgtw
            | SseOpcode::Pcmpgtd
            | SseOpcode::Pextrw
            | SseOpcode::Pinsrw
            | SseOpcode::Pmaddubsw
            | SseOpcode::Pmaddwd
            | SseOpcode::Pmaxsw
            | SseOpcode::Pmaxub
            | SseOpcode::Pminsw
            | SseOpcode::Pminub
            | SseOpcode::Pmovmskb
            | SseOpcode::Pmulhw
            | SseOpcode::Pmulhuw
            | SseOpcode::Pmullw
            | SseOpcode::Pmuludq
            | SseOpcode::Por
            | SseOpcode::Pshufd
            | SseOpcode::Psllw
            | SseOpcode::Pslld
            | SseOpcode::Psllq
            | SseOpcode::Psraw
            | SseOpcode::Psrad
            | SseOpcode::Psrlw
            | SseOpcode::Psrld
            | SseOpcode::Psrlq
            | SseOpcode::Psubb
            | SseOpcode::Psubd
            | SseOpcode::Psubq
            | SseOpcode::Psubw
            | SseOpcode::Psubsb
            | SseOpcode::Psubsw
            | SseOpcode::Psubusb
            | SseOpcode::Psubusw
            | SseOpcode::Punpckhbw
            | SseOpcode::Punpckhwd
            | SseOpcode::Punpcklbw
            | SseOpcode::Punpcklwd
            | SseOpcode::Pxor
            | SseOpcode::Sqrtpd
            | SseOpcode::Sqrtsd
            | SseOpcode::Subpd
            | SseOpcode::Subsd
            | SseOpcode::Ucomisd
            | SseOpcode::Xorpd => SSE2,

            SseOpcode::Pabsb
            | SseOpcode::Pabsw
            | SseOpcode::Pabsd
            | SseOpcode::Palignr
            | SseOpcode::Pmulhrsw
            | SseOpcode::Pshufb => SSSE3,

            SseOpcode::Blendvpd
            | SseOpcode::Blendvps
            | SseOpcode::Insertps
            | SseOpcode::Packusdw
            | SseOpcode::Pblendvb
            | SseOpcode::Pcmpeqq
            | SseOpcode::Pextrb
            | SseOpcode::Pextrd
            | SseOpcode::Pinsrb
            | SseOpcode::Pinsrd
            | SseOpcode::Pmaxsb
            | SseOpcode::Pmaxsd
            | SseOpcode::Pmaxuw
            | SseOpcode::Pmaxud
            | SseOpcode::Pminsb
            | SseOpcode::Pminsd
            | SseOpcode::Pminuw
            | SseOpcode::Pminud
            | SseOpcode::Pmovsxbd
            | SseOpcode::Pmovsxbw
            | SseOpcode::Pmovsxbq
            | SseOpcode::Pmovsxwd
            | SseOpcode::Pmovsxwq
            | SseOpcode::Pmovsxdq
            | SseOpcode::Pmovzxbd
            | SseOpcode::Pmovzxbw
            | SseOpcode::Pmovzxbq
            | SseOpcode::Pmovzxwd
            | SseOpcode::Pmovzxwq
            | SseOpcode::Pmovzxdq
            | SseOpcode::Pmuldq
            | SseOpcode::Pmulld
            | SseOpcode::Ptest
            | SseOpcode::Roundps
            | SseOpcode::Roundpd
            | SseOpcode::Roundss
            | SseOpcode::Roundsd => SSE41,

            SseOpcode::Pcmpgtq => SSE42,
        }
    }

    /// Returns the src operand size for an instruction.
    pub(crate) fn src_size(&self) -> u8 {
        match self {
            SseOpcode::Movd => 4,
            _ => 8,
        }
    }

    /// Does an XmmRmmRImm with this opcode use src1? FIXME: split
    /// into separate instructions.
    pub(crate) fn uses_src1(&self) -> bool {
        match self {
            SseOpcode::Pextrb => false,
            SseOpcode::Pextrw => false,
            SseOpcode::Pextrd => false,
            SseOpcode::Pshufd => false,
            SseOpcode::Roundss => false,
            SseOpcode::Roundsd => false,
            SseOpcode::Roundps => false,
            SseOpcode::Roundpd => false,
            _ => true,
        }
    }
}

impl fmt::Debug for SseOpcode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            SseOpcode::Addps => "addps",
            SseOpcode::Addpd => "addpd",
            SseOpcode::Addss => "addss",
            SseOpcode::Addsd => "addsd",
            SseOpcode::Andpd => "andpd",
            SseOpcode::Andps => "andps",
            SseOpcode::Andnps => "andnps",
            SseOpcode::Andnpd => "andnpd",
            SseOpcode::Blendvpd => "blendvpd",
            SseOpcode::Blendvps => "blendvps",
            SseOpcode::Cmpps => "cmpps",
            SseOpcode::Cmppd => "cmppd",
            SseOpcode::Cmpss => "cmpss",
            SseOpcode::Cmpsd => "cmpsd",
            SseOpcode::Comiss => "comiss",
            SseOpcode::Comisd => "comisd",
            SseOpcode::Cvtdq2ps => "cvtdq2ps",
            SseOpcode::Cvtdq2pd => "cvtdq2pd",
            SseOpcode::Cvtpd2ps => "cvtpd2ps",
            SseOpcode::Cvtps2pd => "cvtps2pd",
            SseOpcode::Cvtsd2ss => "cvtsd2ss",
            SseOpcode::Cvtsd2si => "cvtsd2si",
            SseOpcode::Cvtsi2ss => "cvtsi2ss",
            SseOpcode::Cvtsi2sd => "cvtsi2sd",
            SseOpcode::Cvtss2si => "cvtss2si",
            SseOpcode::Cvtss2sd => "cvtss2sd",
            SseOpcode::Cvttpd2dq => "cvttpd2dq",
            SseOpcode::Cvttps2dq => "cvttps2dq",
            SseOpcode::Cvttss2si => "cvttss2si",
            SseOpcode::Cvttsd2si => "cvttsd2si",
            SseOpcode::Divps => "divps",
            SseOpcode::Divpd => "divpd",
            SseOpcode::Divss => "divss",
            SseOpcode::Divsd => "divsd",
            SseOpcode::Insertps => "insertps",
            SseOpcode::Maxps => "maxps",
            SseOpcode::Maxpd => "maxpd",
            SseOpcode::Maxss => "maxss",
            SseOpcode::Maxsd => "maxsd",
            SseOpcode::Minps => "minps",
            SseOpcode::Minpd => "minpd",
            SseOpcode::Minss => "minss",
            SseOpcode::Minsd => "minsd",
            SseOpcode::Movaps => "movaps",
            SseOpcode::Movapd => "movapd",
            SseOpcode::Movd => "movd",
            SseOpcode::Movdqa => "movdqa",
            SseOpcode::Movdqu => "movdqu",
            SseOpcode::Movlhps => "movlhps",
            SseOpcode::Movmskps => "movmskps",
            SseOpcode::Movmskpd => "movmskpd",
            SseOpcode::Movq => "movq",
            SseOpcode::Movss => "movss",
            SseOpcode::Movsd => "movsd",
            SseOpcode::Movups => "movups",
            SseOpcode::Movupd => "movupd",
            SseOpcode::Mulps => "mulps",
            SseOpcode::Mulpd => "mulpd",
            SseOpcode::Mulss => "mulss",
            SseOpcode::Mulsd => "mulsd",
            SseOpcode::Orpd => "orpd",
            SseOpcode::Orps => "orps",
            SseOpcode::Pabsb => "pabsb",
            SseOpcode::Pabsw => "pabsw",
            SseOpcode::Pabsd => "pabsd",
            SseOpcode::Packssdw => "packssdw",
            SseOpcode::Packsswb => "packsswb",
            SseOpcode::Packusdw => "packusdw",
            SseOpcode::Packuswb => "packuswb",
            SseOpcode::Paddb => "paddb",
            SseOpcode::Paddd => "paddd",
            SseOpcode::Paddq => "paddq",
            SseOpcode::Paddw => "paddw",
            SseOpcode::Paddsb => "paddsb",
            SseOpcode::Paddsw => "paddsw",
            SseOpcode::Paddusb => "paddusb",
            SseOpcode::Paddusw => "paddusw",
            SseOpcode::Palignr => "palignr",
            SseOpcode::Pand => "pand",
            SseOpcode::Pandn => "pandn",
            SseOpcode::Pavgb => "pavgb",
            SseOpcode::Pavgw => "pavgw",
            SseOpcode::Pblendvb => "pblendvb",
            SseOpcode::Pcmpeqb => "pcmpeqb",
            SseOpcode::Pcmpeqw => "pcmpeqw",
            SseOpcode::Pcmpeqd => "pcmpeqd",
            SseOpcode::Pcmpeqq => "pcmpeqq",
            SseOpcode::Pcmpgtb => "pcmpgtb",
            SseOpcode::Pcmpgtw => "pcmpgtw",
            SseOpcode::Pcmpgtd => "pcmpgtd",
            SseOpcode::Pcmpgtq => "pcmpgtq",
            SseOpcode::Pextrb => "pextrb",
            SseOpcode::Pextrw => "pextrw",
            SseOpcode::Pextrd => "pextrd",
            SseOpcode::Pinsrb => "pinsrb",
            SseOpcode::Pinsrw => "pinsrw",
            SseOpcode::Pinsrd => "pinsrd",
            SseOpcode::Pmaddubsw => "pmaddubsw",
            SseOpcode::Pmaddwd => "pmaddwd",
            SseOpcode::Pmaxsb => "pmaxsb",
            SseOpcode::Pmaxsw => "pmaxsw",
            SseOpcode::Pmaxsd => "pmaxsd",
            SseOpcode::Pmaxub => "pmaxub",
            SseOpcode::Pmaxuw => "pmaxuw",
            SseOpcode::Pmaxud => "pmaxud",
            SseOpcode::Pminsb => "pminsb",
            SseOpcode::Pminsw => "pminsw",
            SseOpcode::Pminsd => "pminsd",
            SseOpcode::Pminub => "pminub",
            SseOpcode::Pminuw => "pminuw",
            SseOpcode::Pminud => "pminud",
            SseOpcode::Pmovmskb => "pmovmskb",
            SseOpcode::Pmovsxbd => "pmovsxbd",
            SseOpcode::Pmovsxbw => "pmovsxbw",
            SseOpcode::Pmovsxbq => "pmovsxbq",
            SseOpcode::Pmovsxwd => "pmovsxwd",
            SseOpcode::Pmovsxwq => "pmovsxwq",
            SseOpcode::Pmovsxdq => "pmovsxdq",
            SseOpcode::Pmovzxbd => "pmovzxbd",
            SseOpcode::Pmovzxbw => "pmovzxbw",
            SseOpcode::Pmovzxbq => "pmovzxbq",
            SseOpcode::Pmovzxwd => "pmovzxwd",
            SseOpcode::Pmovzxwq => "pmovzxwq",
            SseOpcode::Pmovzxdq => "pmovzxdq",
            SseOpcode::Pmuldq => "pmuldq",
            SseOpcode::Pmulhw => "pmulhw",
            SseOpcode::Pmulhuw => "pmulhuw",
            SseOpcode::Pmulhrsw => "pmulhrsw",
            SseOpcode::Pmulld => "pmulld",
            SseOpcode::Pmullw => "pmullw",
            SseOpcode::Pmuludq => "pmuludq",
            SseOpcode::Por => "por",
            SseOpcode::Pshufb => "pshufb",
            SseOpcode::Pshufd => "pshufd",
            SseOpcode::Psllw => "psllw",
            SseOpcode::Pslld => "pslld",
            SseOpcode::Psllq => "psllq",
            SseOpcode::Psraw => "psraw",
            SseOpcode::Psrad => "psrad",
            SseOpcode::Psrlw => "psrlw",
            SseOpcode::Psrld => "psrld",
            SseOpcode::Psrlq => "psrlq",
            SseOpcode::Psubb => "psubb",
            SseOpcode::Psubd => "psubd",
            SseOpcode::Psubq => "psubq",
            SseOpcode::Psubw => "psubw",
            SseOpcode::Psubsb => "psubsb",
            SseOpcode::Psubsw => "psubsw",
            SseOpcode::Psubusb => "psubusb",
            SseOpcode::Psubusw => "psubusw",
            SseOpcode::Ptest => "ptest",
            SseOpcode::Punpckhbw => "punpckhbw",
            SseOpcode::Punpckhwd => "punpckhwd",
            SseOpcode::Punpcklbw => "punpcklbw",
            SseOpcode::Punpcklwd => "punpcklwd",
            SseOpcode::Pxor => "pxor",
            SseOpcode::Rcpss => "rcpss",
            SseOpcode::Roundps => "roundps",
            SseOpcode::Roundpd => "roundpd",
            SseOpcode::Roundss => "roundss",
            SseOpcode::Roundsd => "roundsd",
            SseOpcode::Rsqrtss => "rsqrtss",
            SseOpcode::Shufps => "shufps",
            SseOpcode::Sqrtps => "sqrtps",
            SseOpcode::Sqrtpd => "sqrtpd",
            SseOpcode::Sqrtss => "sqrtss",
            SseOpcode::Sqrtsd => "sqrtsd",
            SseOpcode::Subps => "subps",
            SseOpcode::Subpd => "subpd",
            SseOpcode::Subss => "subss",
            SseOpcode::Subsd => "subsd",
            SseOpcode::Ucomiss => "ucomiss",
            SseOpcode::Ucomisd => "ucomisd",
            SseOpcode::Unpcklps => "unpcklps",
            SseOpcode::Xorps => "xorps",
            SseOpcode::Xorpd => "xorpd",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for SseOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, PartialEq)]
pub enum AvxOpcode {
    Vfmadd213ss,
    Vfmadd213sd,
    Vfmadd213ps,
    Vfmadd213pd,
}

impl AvxOpcode {
    /// Which `InstructionSet`s support the opcode?
    pub(crate) fn available_from(&self) -> SmallVec<[InstructionSet; 2]> {
        match self {
            AvxOpcode::Vfmadd213ss
            | AvxOpcode::Vfmadd213sd
            | AvxOpcode::Vfmadd213ps
            | AvxOpcode::Vfmadd213pd => smallvec![InstructionSet::FMA],
        }
    }
}

impl fmt::Debug for AvxOpcode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            AvxOpcode::Vfmadd213ss => "vfmadd213ss",
            AvxOpcode::Vfmadd213sd => "vfmadd213sd",
            AvxOpcode::Vfmadd213ps => "vfmadd213ps",
            AvxOpcode::Vfmadd213pd => "vfmadd213pd",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for AvxOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Clone, PartialEq)]
pub enum Avx512Opcode {
    Vcvtudq2ps,
    Vpabsq,
    Vpermi2b,
    Vpmullq,
    Vpopcntb,
}

impl Avx512Opcode {
    /// Which `InstructionSet`s support the opcode?
    pub(crate) fn available_from(&self) -> SmallVec<[InstructionSet; 2]> {
        match self {
            Avx512Opcode::Vcvtudq2ps => {
                smallvec![InstructionSet::AVX512F, InstructionSet::AVX512VL]
            }
            Avx512Opcode::Vpabsq => smallvec![InstructionSet::AVX512F, InstructionSet::AVX512VL],
            Avx512Opcode::Vpermi2b => {
                smallvec![InstructionSet::AVX512VL, InstructionSet::AVX512VBMI]
            }
            Avx512Opcode::Vpmullq => smallvec![InstructionSet::AVX512VL, InstructionSet::AVX512DQ],
            Avx512Opcode::Vpopcntb => {
                smallvec![InstructionSet::AVX512VL, InstructionSet::AVX512BITALG]
            }
        }
    }
}

impl fmt::Debug for Avx512Opcode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Avx512Opcode::Vcvtudq2ps => "vcvtudq2ps",
            Avx512Opcode::Vpabsq => "vpabsq",
            Avx512Opcode::Vpermi2b => "vpermi2b",
            Avx512Opcode::Vpmullq => "vpmullq",
            Avx512Opcode::Vpopcntb => "vpopcntb",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for Avx512Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// This defines the ways a value can be extended: either signed- or zero-extension, or none for
/// types that are not extended. Contrast with [ExtMode], which defines the widths from and to which
/// values can be extended.
#[allow(dead_code)]
#[derive(Clone, PartialEq)]
pub enum ExtKind {
    None,
    SignExtend,
    ZeroExtend,
}

/// These indicate ways of extending (widening) a value, using the Intel
/// naming: B(yte) = u8, W(ord) = u16, L(ong)word = u32, Q(uad)word = u64
#[derive(Clone, PartialEq)]
pub enum ExtMode {
    /// Byte -> Longword.
    BL,
    /// Byte -> Quadword.
    BQ,
    /// Word -> Longword.
    WL,
    /// Word -> Quadword.
    WQ,
    /// Longword -> Quadword.
    LQ,
}

impl ExtMode {
    /// Calculate the `ExtMode` from passed bit lengths of the from/to types.
    pub(crate) fn new(from_bits: u16, to_bits: u16) -> Option<ExtMode> {
        match (from_bits, to_bits) {
            (1, 8) | (1, 16) | (1, 32) | (8, 16) | (8, 32) => Some(ExtMode::BL),
            (1, 64) | (8, 64) => Some(ExtMode::BQ),
            (16, 32) => Some(ExtMode::WL),
            (16, 64) => Some(ExtMode::WQ),
            (32, 64) => Some(ExtMode::LQ),
            _ => None,
        }
    }

    /// Return the source register size in bytes.
    pub(crate) fn src_size(&self) -> u8 {
        match self {
            ExtMode::BL | ExtMode::BQ => 1,
            ExtMode::WL | ExtMode::WQ => 2,
            ExtMode::LQ => 4,
        }
    }

    /// Return the destination register size in bytes.
    pub(crate) fn dst_size(&self) -> u8 {
        match self {
            ExtMode::BL | ExtMode::WL => 4,
            ExtMode::BQ | ExtMode::WQ | ExtMode::LQ => 8,
        }
    }
}

impl fmt::Debug for ExtMode {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            ExtMode::BL => "bl",
            ExtMode::BQ => "bq",
            ExtMode::WL => "wl",
            ExtMode::WQ => "wq",
            ExtMode::LQ => "lq",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for ExtMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// These indicate the form of a scalar shift/rotate: left, signed right, unsigned right.
#[derive(Clone, Copy)]
pub enum ShiftKind {
    ShiftLeft,
    /// Inserts zeros in the most significant bits.
    ShiftRightLogical,
    /// Replicates the sign bit in the most significant bits.
    ShiftRightArithmetic,
    RotateLeft,
    RotateRight,
}

impl fmt::Debug for ShiftKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            ShiftKind::ShiftLeft => "shl",
            ShiftKind::ShiftRightLogical => "shr",
            ShiftKind::ShiftRightArithmetic => "sar",
            ShiftKind::RotateLeft => "rol",
            ShiftKind::RotateRight => "ror",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for ShiftKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// What kind of division or remainer instruction this is?
#[derive(Clone, Eq, PartialEq)]
pub enum DivOrRemKind {
    SignedDiv,
    UnsignedDiv,
    SignedRem,
    UnsignedRem,
}

impl DivOrRemKind {
    pub(crate) fn is_signed(&self) -> bool {
        match self {
            DivOrRemKind::SignedDiv | DivOrRemKind::SignedRem => true,
            _ => false,
        }
    }

    pub(crate) fn is_div(&self) -> bool {
        match self {
            DivOrRemKind::SignedDiv | DivOrRemKind::UnsignedDiv => true,
            _ => false,
        }
    }
}

/// These indicate condition code tests.  Not all are represented since not all are useful in
/// compiler-generated code.
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum CC {
    ///  overflow
    O = 0,
    /// no overflow
    NO = 1,

    /// < unsigned
    B = 2,
    /// >= unsigned
    NB = 3,

    /// zero
    Z = 4,
    /// not-zero
    NZ = 5,

    /// <= unsigned
    BE = 6,
    /// > unsigned
    NBE = 7,

    /// negative
    S = 8,
    /// not-negative
    NS = 9,

    /// < signed
    L = 12,
    /// >= signed
    NL = 13,

    /// <= signed
    LE = 14,
    /// > signed
    NLE = 15,

    /// parity
    P = 10,

    /// not parity
    NP = 11,
}

impl CC {
    pub(crate) fn from_intcc(intcc: IntCC) -> Self {
        match intcc {
            IntCC::Equal => CC::Z,
            IntCC::NotEqual => CC::NZ,
            IntCC::SignedGreaterThanOrEqual => CC::NL,
            IntCC::SignedGreaterThan => CC::NLE,
            IntCC::SignedLessThanOrEqual => CC::LE,
            IntCC::SignedLessThan => CC::L,
            IntCC::UnsignedGreaterThanOrEqual => CC::NB,
            IntCC::UnsignedGreaterThan => CC::NBE,
            IntCC::UnsignedLessThanOrEqual => CC::BE,
            IntCC::UnsignedLessThan => CC::B,
        }
    }

    pub(crate) fn invert(&self) -> Self {
        match self {
            CC::O => CC::NO,
            CC::NO => CC::O,

            CC::B => CC::NB,
            CC::NB => CC::B,

            CC::Z => CC::NZ,
            CC::NZ => CC::Z,

            CC::BE => CC::NBE,
            CC::NBE => CC::BE,

            CC::S => CC::NS,
            CC::NS => CC::S,

            CC::L => CC::NL,
            CC::NL => CC::L,

            CC::LE => CC::NLE,
            CC::NLE => CC::LE,

            CC::P => CC::NP,
            CC::NP => CC::P,
        }
    }

    pub(crate) fn get_enc(self) -> u8 {
        self as u8
    }
}

impl fmt::Debug for CC {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            CC::O => "o",
            CC::NO => "no",
            CC::B => "b",
            CC::NB => "nb",
            CC::Z => "z",
            CC::NZ => "nz",
            CC::BE => "be",
            CC::NBE => "nbe",
            CC::S => "s",
            CC::NS => "ns",
            CC::L => "l",
            CC::NL => "nl",
            CC::LE => "le",
            CC::NLE => "nle",
            CC::P => "p",
            CC::NP => "np",
        };
        write!(fmt, "{}", name)
    }
}

impl fmt::Display for CC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// Encode the ways that floats can be compared. This is used in float comparisons such as `cmpps`,
/// e.g.; it is distinguished from other float comparisons (e.g. `ucomiss`) in that those use EFLAGS
/// whereas [FcmpImm] is used as an immediate.
#[derive(Clone, Copy)]
pub enum FcmpImm {
    Equal = 0x00,
    LessThan = 0x01,
    LessThanOrEqual = 0x02,
    Unordered = 0x03,
    NotEqual = 0x04,
    UnorderedOrGreaterThanOrEqual = 0x05,
    UnorderedOrGreaterThan = 0x06,
    Ordered = 0x07,
}

impl FcmpImm {
    pub(crate) fn encode(self) -> u8 {
        self as u8
    }
}

impl From<FloatCC> for FcmpImm {
    fn from(cond: FloatCC) -> Self {
        match cond {
            FloatCC::Equal => FcmpImm::Equal,
            FloatCC::LessThan => FcmpImm::LessThan,
            FloatCC::LessThanOrEqual => FcmpImm::LessThanOrEqual,
            FloatCC::Unordered => FcmpImm::Unordered,
            FloatCC::NotEqual => FcmpImm::NotEqual,
            FloatCC::UnorderedOrGreaterThanOrEqual => FcmpImm::UnorderedOrGreaterThanOrEqual,
            FloatCC::UnorderedOrGreaterThan => FcmpImm::UnorderedOrGreaterThan,
            FloatCC::Ordered => FcmpImm::Ordered,
            _ => panic!("unable to create comparison predicate for {}", cond),
        }
    }
}

/// Encode the rounding modes used as part of the Rounding Control field.
/// Note, these rounding immediates only consider the rounding control field
/// (i.e. the rounding mode) which only take up the first two bits when encoded.
/// However the rounding immediate which this field helps make up, also includes
/// bits 3 and 4 which define the rounding select and precision mask respectively.
/// These two bits are not defined here and are implictly set to zero when encoded.
#[derive(Clone, Copy)]
pub enum RoundImm {
    RoundNearest = 0x00,
    RoundDown = 0x01,
    RoundUp = 0x02,
    RoundZero = 0x03,
}

impl RoundImm {
    pub(crate) fn encode(self) -> u8 {
        self as u8
    }
}

/// An operand's size in bits.
#[derive(Clone, Copy, PartialEq)]
pub enum OperandSize {
    Size8,
    Size16,
    Size32,
    Size64,
}

impl OperandSize {
    pub(crate) fn from_bytes(num_bytes: u32) -> Self {
        match num_bytes {
            1 => OperandSize::Size8,
            2 => OperandSize::Size16,
            4 => OperandSize::Size32,
            8 => OperandSize::Size64,
            _ => unreachable!("Invalid OperandSize: {}", num_bytes),
        }
    }

    // Computes the OperandSize for a given type.
    // For vectors, the OperandSize of the lanes is returned.
    pub(crate) fn from_ty(ty: Type) -> Self {
        Self::from_bytes(ty.lane_type().bytes())
    }

    // Check that the value of self is one of the allowed sizes.
    pub(crate) fn is_one_of(&self, sizes: &[Self]) -> bool {
        sizes.iter().any(|val| *self == *val)
    }

    pub(crate) fn to_bytes(&self) -> u8 {
        match self {
            Self::Size8 => 1,
            Self::Size16 => 2,
            Self::Size32 => 4,
            Self::Size64 => 8,
        }
    }

    pub(crate) fn to_bits(&self) -> u8 {
        self.to_bytes() * 8
    }
}

/// An x64 memory fence kind.
#[derive(Clone)]
#[allow(dead_code)]
pub enum FenceKind {
    /// `mfence` instruction ("Memory Fence")
    MFence,
    /// `lfence` instruction ("Load Fence")
    LFence,
    /// `sfence` instruction ("Store Fence")
    SFence,
}
