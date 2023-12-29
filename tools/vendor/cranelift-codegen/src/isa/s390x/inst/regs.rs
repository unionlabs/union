//! S390x ISA definitions: registers.

use alloc::string::String;
use regalloc2::MachineEnv;
use regalloc2::PReg;
use regalloc2::VReg;

use crate::isa::s390x::inst::{RegPair, WritableRegPair};
use crate::machinst::*;
use crate::settings;

//=============================================================================
// Registers, the Universe thereof, and printing

/// Get a reference to a GPR (integer register).
pub fn gpr(num: u8) -> Reg {
    let preg = gpr_preg(num);
    Reg::from(VReg::new(preg.index(), RegClass::Int))
}

pub(crate) const fn gpr_preg(num: u8) -> PReg {
    assert!(num < 16);
    PReg::new(num as usize, RegClass::Int)
}

/// Get a writable reference to a GPR.
pub fn writable_gpr(num: u8) -> Writable<Reg> {
    Writable::from_reg(gpr(num))
}

/// Get a reference to a VR (vector register).
pub fn vr(num: u8) -> Reg {
    let preg = vr_preg(num);
    Reg::from(VReg::new(preg.index(), RegClass::Float))
}

pub(crate) const fn vr_preg(num: u8) -> PReg {
    assert!(num < 32);
    PReg::new(num as usize, RegClass::Float)
}

/// Get a writable reference to a VR.
#[allow(dead_code)] // used by tests.
pub fn writable_vr(num: u8) -> Writable<Reg> {
    Writable::from_reg(vr(num))
}

/// Test whether a vector register is overlapping an FPR.
pub fn is_fpr(r: Reg) -> bool {
    let r = r.to_real_reg().unwrap();
    assert!(r.class() == RegClass::Float);
    return r.hw_enc() < 16;
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
pub fn create_machine_env(_flags: &settings::Flags) -> MachineEnv {
    fn preg(r: Reg) -> PReg {
        r.to_real_reg().unwrap().into()
    }

    MachineEnv {
        preferred_regs_by_class: [
            vec![
                // no r0; can't use for addressing?
                // no r1; it is our spilltmp.
                preg(gpr(2)),
                preg(gpr(3)),
                preg(gpr(4)),
                preg(gpr(5)),
            ],
            vec![
                preg(vr(0)),
                preg(vr(1)),
                preg(vr(2)),
                preg(vr(3)),
                preg(vr(4)),
                preg(vr(5)),
                preg(vr(6)),
                preg(vr(7)),
                preg(vr(16)),
                preg(vr(17)),
                preg(vr(18)),
                preg(vr(19)),
                preg(vr(20)),
                preg(vr(21)),
                preg(vr(22)),
                preg(vr(23)),
                preg(vr(24)),
                preg(vr(25)),
                preg(vr(26)),
                preg(vr(27)),
                preg(vr(28)),
                preg(vr(29)),
                preg(vr(30)),
                preg(vr(31)),
            ],
        ],
        non_preferred_regs_by_class: [
            vec![
                preg(gpr(6)),
                preg(gpr(7)),
                preg(gpr(8)),
                preg(gpr(9)),
                preg(gpr(10)),
                preg(gpr(11)),
                preg(gpr(12)),
                preg(gpr(13)),
                preg(gpr(14)),
                // no r15; it is the stack pointer.
            ],
            vec![
                preg(vr(8)),
                preg(vr(9)),
                preg(vr(10)),
                preg(vr(11)),
                preg(vr(12)),
                preg(vr(13)),
                preg(vr(14)),
                preg(vr(15)),
            ],
        ],
        fixed_stack_slots: vec![],
    }
}

pub fn show_reg(reg: Reg) -> String {
    if let Some(rreg) = reg.to_real_reg() {
        match rreg.class() {
            RegClass::Int => format!("%r{}", rreg.hw_enc()),
            RegClass::Float => format!("%v{}", rreg.hw_enc()),
        }
    } else {
        format!("%{:?}", reg)
    }
}

pub fn maybe_show_fpr(reg: Reg) -> Option<String> {
    if let Some(rreg) = reg.to_real_reg() {
        if is_fpr(reg) {
            return Some(format!("%f{}", rreg.hw_enc()));
        }
    }
    None
}

pub fn pretty_print_reg(reg: Reg, allocs: &mut AllocationConsumer<'_>) -> String {
    let reg = allocs.next(reg);
    show_reg(reg)
}

pub fn pretty_print_regpair(pair: RegPair, allocs: &mut AllocationConsumer<'_>) -> String {
    let hi = allocs.next(pair.hi);
    let lo = allocs.next(pair.lo);
    if let Some(hi_reg) = hi.to_real_reg() {
        if let Some(lo_reg) = lo.to_real_reg() {
            assert!(
                hi_reg.hw_enc() + 1 == lo_reg.hw_enc(),
                "Invalid regpair: {} {}",
                show_reg(hi),
                show_reg(lo)
            );
            return show_reg(hi);
        }
    }

    format!("{}/{}", show_reg(hi), show_reg(lo))
}

pub fn pretty_print_reg_mod(
    rd: Writable<Reg>,
    ri: Reg,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let output = allocs.next_writable(rd).to_reg();
    let input = allocs.next(ri);
    if output == input {
        show_reg(output)
    } else {
        format!("{}<-{}", show_reg(output), show_reg(input))
    }
}

pub fn pretty_print_regpair_mod(
    rd: WritableRegPair,
    ri: RegPair,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let rd_hi = allocs.next(rd.hi.to_reg());
    let rd_lo = allocs.next(rd.lo.to_reg());
    let ri_hi = allocs.next(ri.hi);
    let ri_lo = allocs.next(ri.lo);
    if rd_hi == ri_hi {
        show_reg(rd_hi)
    } else {
        format!(
            "{}/{}<-{}/{}",
            show_reg(rd_hi),
            show_reg(rd_lo),
            show_reg(ri_hi),
            show_reg(ri_lo)
        )
    }
}

pub fn pretty_print_regpair_mod_lo(
    rd: WritableRegPair,
    ri: Reg,
    allocs: &mut AllocationConsumer<'_>,
) -> String {
    let rd_hi = allocs.next(rd.hi.to_reg());
    let rd_lo = allocs.next(rd.lo.to_reg());
    let ri = allocs.next(ri);
    if rd_lo == ri {
        show_reg(rd_hi)
    } else {
        format!(
            "{}/{}<-_/{}",
            show_reg(rd_hi),
            show_reg(rd_lo),
            show_reg(ri),
        )
    }
}

pub fn pretty_print_fpr(reg: Reg, allocs: &mut AllocationConsumer<'_>) -> (String, Option<String>) {
    let reg = allocs.next(reg);
    (show_reg(reg), maybe_show_fpr(reg))
}
