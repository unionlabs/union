//! Definitions for registers, operands, etc. Provides a thin
//! interface over the register allocator so that we can more easily
//! swap it out or shim it when necessary.

use alloc::{string::String, vec::Vec};
use core::{fmt::Debug, hash::Hash};
use regalloc2::{Allocation, Operand, OperandConstraint, PReg, PRegSet, VReg};

#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// The first 128 vregs (64 int, 64 float/vec) are "pinned" to
/// physical registers: this means that they are always constrained to
/// the corresponding register at all use/mod/def sites.
///
/// Arbitrary vregs can also be constrained to physical registers at
/// particular use/def/mod sites, and this is preferable; but pinned
/// vregs allow us to migrate code that has been written using
/// RealRegs directly.
const PINNED_VREGS: usize = 128;

/// Convert a `VReg` to its pinned `PReg`, if any.
pub fn pinned_vreg_to_preg(vreg: VReg) -> Option<PReg> {
    if vreg.vreg() < PINNED_VREGS {
        Some(PReg::from_index(vreg.vreg()))
    } else {
        None
    }
}

/// Give the first available vreg for generated code (i.e., after all
/// pinned vregs).
pub fn first_user_vreg_index() -> usize {
    // This is just the constant defined above, but we keep the
    // constant private and expose only this helper function with the
    // specific name in order to ensure other parts of the code don't
    // open-code and depend on the index-space scheme.
    PINNED_VREGS
}

/// A register named in an instruction. This register can be either a
/// virtual register or a fixed physical register. It does not have
/// any constraints applied to it: those can be added later in
/// `MachInst::get_operands()` when the `Reg`s are converted to
/// `Operand`s.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Reg(VReg);

impl Reg {
    /// Get the physical register (`RealReg`), if this register is
    /// one.
    pub fn to_real_reg(self) -> Option<RealReg> {
        if pinned_vreg_to_preg(self.0).is_some() {
            Some(RealReg(self.0))
        } else {
            None
        }
    }

    /// Get the virtual (non-physical) register, if this register is
    /// one.
    pub fn to_virtual_reg(self) -> Option<VirtualReg> {
        if pinned_vreg_to_preg(self.0).is_none() {
            Some(VirtualReg(self.0))
        } else {
            None
        }
    }

    /// Get the class of this register.
    pub fn class(self) -> RegClass {
        self.0.class()
    }

    /// Is this a real (physical) reg?
    pub fn is_real(self) -> bool {
        self.to_real_reg().is_some()
    }

    /// Is this a virtual reg?
    pub fn is_virtual(self) -> bool {
        self.to_virtual_reg().is_some()
    }
}

impl std::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(rreg) = self.to_real_reg() {
            let preg: PReg = rreg.into();
            write!(f, "{}", preg)
        } else if let Some(vreg) = self.to_virtual_reg() {
            let vreg: VReg = vreg.into();
            write!(f, "{}", vreg)
        } else {
            unreachable!()
        }
    }
}

/// A real (physical) register. This corresponds to one of the target
/// ISA's named registers and can be used as an instruction operand.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct RealReg(VReg);

impl RealReg {
    /// Get the class of this register.
    pub fn class(self) -> RegClass {
        self.0.class()
    }

    pub fn hw_enc(self) -> u8 {
        PReg::from(self).hw_enc() as u8
    }
}

impl std::fmt::Debug for RealReg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Reg::from(*self).fmt(f)
    }
}

/// A virtual register. This can be allocated into a real (physical)
/// register of the appropriate register class, but which one is not
/// specified. Virtual registers are used when generating `MachInst`s,
/// before register allocation occurs, in order to allow us to name as
/// many register-carried values as necessary.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct VirtualReg(VReg);

impl VirtualReg {
    /// Get the class of this register.
    pub fn class(self) -> RegClass {
        self.0.class()
    }

    pub fn index(self) -> usize {
        self.0.vreg()
    }
}

impl std::fmt::Debug for VirtualReg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Reg::from(*self).fmt(f)
    }
}

/// A type wrapper that indicates a register type is writable. The
/// underlying register can be extracted, and the type wrapper can be
/// built using an arbitrary register. Hence, this type-level wrapper
/// is not strictly a guarantee. However, "casting" to a writable
/// register is an explicit operation for which we can
/// audit. Ordinarily, internal APIs in the compiler backend should
/// take a `Writable<Reg>` whenever the register is written, and the
/// usual, frictionless way to get one of these is to allocate a new
/// temporary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Writable<T: Clone + Copy + Debug + PartialEq + Eq + PartialOrd + Ord + Hash> {
    reg: T,
}

impl<T: Clone + Copy + Debug + PartialEq + Eq + PartialOrd + Ord + Hash> Writable<T> {
    /// Explicitly construct a `Writable<T>` from a `T`. As noted in
    /// the documentation for `Writable`, this is not hidden or
    /// disallowed from the outside; anyone can perform the "cast";
    /// but it is explicit so that we can audit the use sites.
    pub fn from_reg(reg: T) -> Writable<T> {
        Writable { reg }
    }

    /// Get the underlying register, which can be read.
    pub fn to_reg(self) -> T {
        self.reg
    }

    /// Map the underlying register to another value or type.
    pub fn map<U, F>(self, f: F) -> Writable<U>
    where
        U: Clone + Copy + Debug + PartialEq + Eq + PartialOrd + Ord + Hash,
        F: Fn(T) -> U,
    {
        Writable { reg: f(self.reg) }
    }
}

// Conversions between regalloc2 types (VReg) and our types
// (VirtualReg, RealReg, Reg).

impl std::convert::From<regalloc2::VReg> for Reg {
    fn from(vreg: regalloc2::VReg) -> Reg {
        Reg(vreg)
    }
}

impl std::convert::From<regalloc2::VReg> for VirtualReg {
    fn from(vreg: regalloc2::VReg) -> VirtualReg {
        debug_assert!(pinned_vreg_to_preg(vreg).is_none());
        VirtualReg(vreg)
    }
}

impl std::convert::From<regalloc2::VReg> for RealReg {
    fn from(vreg: regalloc2::VReg) -> RealReg {
        debug_assert!(pinned_vreg_to_preg(vreg).is_some());
        RealReg(vreg)
    }
}

impl std::convert::From<Reg> for regalloc2::VReg {
    /// Extract the underlying `regalloc2::VReg`. Note that physical
    /// registers also map to particular (special) VRegs, so this
    /// method can be used either on virtual or physical `Reg`s.
    fn from(reg: Reg) -> regalloc2::VReg {
        reg.0
    }
}

impl std::convert::From<VirtualReg> for regalloc2::VReg {
    fn from(reg: VirtualReg) -> regalloc2::VReg {
        reg.0
    }
}

impl std::convert::From<RealReg> for regalloc2::VReg {
    fn from(reg: RealReg) -> regalloc2::VReg {
        reg.0
    }
}

impl std::convert::From<RealReg> for regalloc2::PReg {
    fn from(reg: RealReg) -> regalloc2::PReg {
        PReg::from_index(reg.0.vreg())
    }
}

impl std::convert::From<regalloc2::PReg> for RealReg {
    fn from(preg: regalloc2::PReg) -> RealReg {
        RealReg(VReg::new(preg.index(), preg.class()))
    }
}

impl std::convert::From<regalloc2::PReg> for Reg {
    fn from(preg: regalloc2::PReg) -> Reg {
        Reg(VReg::new(preg.index(), preg.class()))
    }
}

impl std::convert::From<RealReg> for Reg {
    fn from(reg: RealReg) -> Reg {
        Reg(reg.0)
    }
}

impl std::convert::From<VirtualReg> for Reg {
    fn from(reg: VirtualReg) -> Reg {
        Reg(reg.0)
    }
}

/// A spill slot.
pub type SpillSlot = regalloc2::SpillSlot;

/// A register class. Each register in the ISA has one class, and the
/// classes are disjoint. Most modern ISAs will have just two classes:
/// the integer/general-purpose registers (GPRs), and the float/vector
/// registers (typically used for both).
///
/// Note that unlike some other compiler backend/register allocator
/// designs, we do not allow for overlapping classes, i.e. registers
/// that belong to more than one class, because doing so makes the
/// allocation problem significantly more complex. Instead, when a
/// register can be addressed under different names for different
/// sizes (for example), the backend author should pick classes that
/// denote some fundamental allocation unit that encompasses the whole
/// register. For example, always allocate 128-bit vector registers
/// `v0`..`vN`, even though `f32` and `f64` values may use only the
/// low 32/64 bits of those registers and name them differently.
pub type RegClass = regalloc2::RegClass;

/// An OperandCollector is a wrapper around a Vec of Operands
/// (flattened array for a whole sequence of instructions) that
/// gathers operands from a single instruction and provides the range
/// in the flattened array.
#[derive(Debug)]
pub struct OperandCollector<'a, F: Fn(VReg) -> VReg> {
    operands: &'a mut Vec<Operand>,
    operands_start: usize,
    clobbers: PRegSet,

    /// The subset of physical registers that are allocatable.
    allocatable: PRegSet,

    renamer: F,
}

impl<'a, F: Fn(VReg) -> VReg> OperandCollector<'a, F> {
    /// Start gathering operands into one flattened operand array.
    pub fn new(operands: &'a mut Vec<Operand>, allocatable: PRegSet, renamer: F) -> Self {
        let operands_start = operands.len();
        Self {
            operands,
            operands_start,
            clobbers: PRegSet::default(),
            allocatable,
            renamer,
        }
    }

    /// Returns true if no reuse_def constraints have been added.
    pub fn no_reuse_def(&self) -> bool {
        !self.operands[self.operands_start..]
            .iter()
            .any(|operand| match operand.constraint() {
                OperandConstraint::Reuse(_) => true,
                _ => false,
            })
    }

    fn is_allocatable_preg(&self, reg: PReg) -> bool {
        self.allocatable.contains(reg)
    }

    /// Add an operand.
    fn add_operand(&mut self, operand: Operand) {
        let vreg = (self.renamer)(operand.vreg());
        let operand = Operand::new(vreg, operand.constraint(), operand.kind(), operand.pos());
        self.operands.push(operand);
    }

    /// Finish the operand collection and return the tuple giving the
    /// range of indices in the flattened operand array, and the
    /// clobber set.
    pub fn finish(self) -> ((u32, u32), PRegSet) {
        let start = self.operands_start as u32;
        let end = self.operands.len() as u32;
        ((start, end), self.clobbers)
    }

    /// Add a use of a fixed, nonallocatable physical register.
    pub fn reg_fixed_nonallocatable(&mut self, preg: PReg) {
        debug_assert!(!self.is_allocatable_preg(preg));
        self.add_operand(Operand::fixed_nonallocatable(preg))
    }

    /// Add a register use, at the start of the instruction (`Before`
    /// position).
    pub fn reg_use(&mut self, reg: Reg) {
        if let Some(rreg) = reg.to_real_reg() {
            self.reg_fixed_nonallocatable(rreg.into());
        } else {
            debug_assert!(reg.is_virtual());
            self.add_operand(Operand::reg_use(reg.into()));
        }
    }

    /// Add a register use, at the end of the instruction (`After` position).
    pub fn reg_late_use(&mut self, reg: Reg) {
        if let Some(rreg) = reg.to_real_reg() {
            self.reg_fixed_nonallocatable(rreg.into());
        } else {
            debug_assert!(reg.is_virtual());
            self.add_operand(Operand::reg_use_at_end(reg.into()));
        }
    }

    /// Add multiple register uses.
    pub fn reg_uses(&mut self, regs: &[Reg]) {
        for &reg in regs {
            self.reg_use(reg);
        }
    }

    /// Add a register def, at the end of the instruction (`After`
    /// position). Use only when this def will be written after all
    /// uses are read.
    pub fn reg_def(&mut self, reg: Writable<Reg>) {
        if let Some(rreg) = reg.to_reg().to_real_reg() {
            self.reg_fixed_nonallocatable(rreg.into());
        } else {
            debug_assert!(reg.to_reg().is_virtual());
            self.add_operand(Operand::reg_def(reg.to_reg().into()));
        }
    }

    /// Add multiple register defs.
    pub fn reg_defs(&mut self, regs: &[Writable<Reg>]) {
        for &reg in regs {
            self.reg_def(reg);
        }
    }

    /// Add a register "early def", which logically occurs at the
    /// beginning of the instruction, alongside all uses. Use this
    /// when the def may be written before all uses are read; the
    /// regalloc will ensure that it does not overwrite any uses.
    pub fn reg_early_def(&mut self, reg: Writable<Reg>) {
        if let Some(rreg) = reg.to_reg().to_real_reg() {
            self.reg_fixed_nonallocatable(rreg.into());
        } else {
            debug_assert!(reg.to_reg().is_virtual());
            self.add_operand(Operand::reg_def_at_start(reg.to_reg().into()));
        }
    }

    /// Add a register "fixed use", which ties a vreg to a particular
    /// RealReg at this point.
    pub fn reg_fixed_use(&mut self, reg: Reg, rreg: Reg) {
        debug_assert!(reg.is_virtual());
        let rreg = rreg.to_real_reg().expect("fixed reg is not a RealReg");
        debug_assert!(self.is_allocatable_preg(rreg.into()));
        self.add_operand(Operand::reg_fixed_use(reg.into(), rreg.into()));
    }

    /// Add a register "fixed def", which ties a vreg to a particular
    /// RealReg at this point.
    pub fn reg_fixed_def(&mut self, reg: Writable<Reg>, rreg: Reg) {
        debug_assert!(reg.to_reg().is_virtual());
        let rreg = rreg.to_real_reg().expect("fixed reg is not a RealReg");
        debug_assert!(self.is_allocatable_preg(rreg.into()));
        self.add_operand(Operand::reg_fixed_def(reg.to_reg().into(), rreg.into()));
    }

    /// Add a register def that reuses an earlier use-operand's
    /// allocation. The index of that earlier operand (relative to the
    /// current instruction's start of operands) must be known.
    pub fn reg_reuse_def(&mut self, reg: Writable<Reg>, idx: usize) {
        if let Some(rreg) = reg.to_reg().to_real_reg() {
            // In some cases we see real register arguments to a reg_reuse_def
            // constraint. We assume the creator knows what they're doing
            // here, though we do also require that the real register be a
            // fixed-nonallocatable register.
            self.reg_fixed_nonallocatable(rreg.into());
        } else {
            // The operand we're reusing must not be fixed-nonallocatable, as
            // that would imply that the register has been allocated to a
            // virtual register.
            self.add_operand(Operand::reg_reuse_def(reg.to_reg().into(), idx));
        }
    }

    /// Add a register clobber set. This is a set of registers that
    /// are written by the instruction, so must be reserved (not used)
    /// for the whole instruction, but are not used afterward.
    pub fn reg_clobbers(&mut self, regs: PRegSet) {
        self.clobbers.union_from(regs);
    }
}

/// Pretty-print part of a disassembly, with knowledge of
/// operand/instruction size, and optionally with regalloc
/// results. This can be used, for example, to print either `rax` or
/// `eax` for the register by those names on x86-64, depending on a
/// 64- or 32-bit context.
pub trait PrettyPrint {
    fn pretty_print(&self, size_bytes: u8, allocs: &mut AllocationConsumer<'_>) -> String;

    fn pretty_print_default(&self) -> String {
        self.pretty_print(0, &mut AllocationConsumer::new(&[]))
    }
}

/// A consumer of an (optional) list of Allocations along with Regs
/// that provides RealRegs where available.
///
/// This is meant to be used during code emission or
/// pretty-printing. In at least the latter case, regalloc results may
/// or may not be available, so we may end up printing either vregs or
/// rregs. Even pre-regalloc, though, some registers may be RealRegs
/// that were provided when the instruction was created.
///
/// This struct should be used in a specific way: when matching on an
/// instruction, provide it the Regs in the same order as they were
/// provided to the OperandCollector.
#[derive(Clone)]
pub struct AllocationConsumer<'a> {
    allocs: std::slice::Iter<'a, Allocation>,
}

impl<'a> AllocationConsumer<'a> {
    pub fn new(allocs: &'a [Allocation]) -> Self {
        Self {
            allocs: allocs.iter(),
        }
    }

    pub fn next_fixed_nonallocatable(&mut self, preg: PReg) {
        let alloc = self.allocs.next();
        let alloc = alloc.map(|alloc| {
            Reg::from(
                alloc
                    .as_reg()
                    .expect("Should not have gotten a stack allocation"),
            )
        });

        assert_eq!(preg, alloc.unwrap().to_real_reg().unwrap().into());
    }

    pub fn next(&mut self, pre_regalloc_reg: Reg) -> Reg {
        let alloc = self.allocs.next();
        let alloc = alloc.map(|alloc| {
            Reg::from(
                alloc
                    .as_reg()
                    .expect("Should not have gotten a stack allocation"),
            )
        });

        match (pre_regalloc_reg.to_real_reg(), alloc) {
            (Some(rreg), None) => rreg.into(),
            (Some(rreg), Some(alloc)) => {
                debug_assert_eq!(Reg::from(rreg), alloc);
                alloc
            }
            (None, Some(alloc)) => alloc,
            _ => pre_regalloc_reg,
        }
    }

    pub fn next_writable(&mut self, pre_regalloc_reg: Writable<Reg>) -> Writable<Reg> {
        Writable::from_reg(self.next(pre_regalloc_reg.to_reg()))
    }
}

impl<'a> std::default::Default for AllocationConsumer<'a> {
    fn default() -> Self {
        Self { allocs: [].iter() }
    }
}
