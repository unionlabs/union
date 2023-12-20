//! S390x ISA definitions: registers.

use crate::settings;
use regalloc::{RealRegUniverse, Reg, RegClass, RegClassInfo, Writable, NUM_REG_CLASSES};

//=============================================================================
// Registers, the Universe thereof, and printing

#[rustfmt::skip]
const GPR_INDICES: [u8; 16] = [
    // r0 and r1 reserved
    30, 31,
    // r2 - r5 call-clobbered
    16, 17, 18, 19,
    // r6 - r14 call-saved (order reversed)
    28, 27, 26, 25, 24, 23, 22, 21, 20,
    // r15 (SP)
    29,
];

#[rustfmt::skip]
const FPR_INDICES: [u8; 16] = [
    // f0 - f7 as pairs
    0, 4, 1, 5, 2, 6, 3, 7,
    // f8 - f15 as pairs
    8, 12, 9, 13, 10, 14, 11, 15,
];

/// Get a reference to a GPR (integer register).
pub fn gpr(num: u8) -> Reg {
    assert!(num < 16);
    Reg::new_real(
        RegClass::I64,
        /* enc = */ num,
        /* index = */ GPR_INDICES[num as usize],
    )
}

/// Get a writable reference to a GPR.
pub fn writable_gpr(num: u8) -> Writable<Reg> {
    Writable::from_reg(gpr(num))
}

/// Get a reference to a FPR (floating-point register).
pub fn fpr(num: u8) -> Reg {
    assert!(num < 16);
    Reg::new_real(
        RegClass::F64,
        /* enc = */ num,
        /* index = */ FPR_INDICES[num as usize],
    )
}

/// Get a writable reference to a V-register.
pub fn writable_fpr(num: u8) -> Writable<Reg> {
    Writable::from_reg(fpr(num))
}

/// Get a reference to the stack-pointer register.
pub fn stack_reg() -> Reg {
    gpr(15)
}

/// Get a writable reference to the stack-pointer register.
pub fn writable_stack_reg() -> Writable<Reg> {
    Writable::from_reg(stack_reg())
}

/// Get a reference to the first temporary, sometimes "spill temporary", register. This register is
/// used to compute the address of a spill slot when a direct offset addressing mode from FP is not
/// sufficient (+/- 2^11 words). We exclude this register from regalloc and reserve it for this
/// purpose for simplicity; otherwise we need a multi-stage analysis where we first determine how
/// many spill slots we have, then perhaps remove the reg from the pool and recompute regalloc.
///
/// We use r1 for this because it's a scratch register but is slightly special (used for linker
/// veneers). We're free to use it as long as we don't expect it to live through call instructions.
pub fn spilltmp_reg() -> Reg {
    gpr(1)
}

/// Get a writable reference to the spilltmp reg.
pub fn writable_spilltmp_reg() -> Writable<Reg> {
    Writable::from_reg(spilltmp_reg())
}

pub fn zero_reg() -> Reg {
    gpr(0)
}

/// Create the register universe for AArch64.
pub fn create_reg_universe(_flags: &settings::Flags) -> RealRegUniverse {
    let mut regs = vec![];
    let mut allocable_by_class = [None; NUM_REG_CLASSES];

    // Numbering Scheme: we put FPRs first, then GPRs. The GPRs exclude several registers:
    // r0 (we cannot use this for addressing // FIXME regalloc)
    // r1 (spilltmp)
    // r15 (stack pointer)

    // FPRs.
    let mut base = regs.len();
    regs.push((fpr(0).to_real_reg(), "%f0".into()));
    regs.push((fpr(2).to_real_reg(), "%f2".into()));
    regs.push((fpr(4).to_real_reg(), "%f4".into()));
    regs.push((fpr(6).to_real_reg(), "%f6".into()));
    regs.push((fpr(1).to_real_reg(), "%f1".into()));
    regs.push((fpr(3).to_real_reg(), "%f3".into()));
    regs.push((fpr(5).to_real_reg(), "%f5".into()));
    regs.push((fpr(7).to_real_reg(), "%f7".into()));
    regs.push((fpr(8).to_real_reg(), "%f8".into()));
    regs.push((fpr(10).to_real_reg(), "%f10".into()));
    regs.push((fpr(12).to_real_reg(), "%f12".into()));
    regs.push((fpr(14).to_real_reg(), "%f14".into()));
    regs.push((fpr(9).to_real_reg(), "%f9".into()));
    regs.push((fpr(11).to_real_reg(), "%f11".into()));
    regs.push((fpr(13).to_real_reg(), "%f13".into()));
    regs.push((fpr(15).to_real_reg(), "%f15".into()));

    allocable_by_class[RegClass::F64.rc_to_usize()] = Some(RegClassInfo {
        first: base,
        last: regs.len() - 1,
        suggested_scratch: Some(fpr(1).get_index()),
    });

    // Caller-saved GPRs in the SystemV s390x ABI.
    base = regs.len();
    regs.push((gpr(2).to_real_reg(), "%r2".into()));
    regs.push((gpr(3).to_real_reg(), "%r3".into()));
    regs.push((gpr(4).to_real_reg(), "%r4".into()));
    regs.push((gpr(5).to_real_reg(), "%r5".into()));

    // Callee-saved GPRs in the SystemV s390x ABI.
    // We start from r14 downwards in an attempt to allow the
    // prolog to use as short a STMG as possible.
    regs.push((gpr(14).to_real_reg(), "%r14".into()));
    regs.push((gpr(13).to_real_reg(), "%r13".into()));
    regs.push((gpr(12).to_real_reg(), "%r12".into()));
    regs.push((gpr(11).to_real_reg(), "%r11".into()));
    regs.push((gpr(10).to_real_reg(), "%r10".into()));
    regs.push((gpr(9).to_real_reg(), "%r9".into()));
    regs.push((gpr(8).to_real_reg(), "%r8".into()));
    regs.push((gpr(7).to_real_reg(), "%r7".into()));
    regs.push((gpr(6).to_real_reg(), "%r6".into()));

    allocable_by_class[RegClass::I64.rc_to_usize()] = Some(RegClassInfo {
        first: base,
        last: regs.len() - 1,
        suggested_scratch: Some(gpr(13).get_index()),
    });

    // Other regs, not available to the allocator.
    let allocable = regs.len();
    regs.push((gpr(15).to_real_reg(), "%r15".into()));
    regs.push((gpr(0).to_real_reg(), "%r0".into()));
    regs.push((gpr(1).to_real_reg(), "%r1".into()));

    // Assert sanity: the indices in the register structs must match their
    // actual indices in the array.
    for (i, reg) in regs.iter().enumerate() {
        assert_eq!(i, reg.0.get_index());
    }

    RealRegUniverse {
        regs,
        allocable,
        allocable_by_class,
    }
}
