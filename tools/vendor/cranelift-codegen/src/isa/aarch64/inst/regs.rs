//! AArch64 ISA definitions: registers.

use crate::isa::aarch64::inst::OperandSize;
use crate::isa::aarch64::inst::ScalarSize;
use crate::isa::aarch64::inst::VectorSize;
use crate::settings;

use regalloc::{
    PrettyPrint, RealRegUniverse, Reg, RegClass, RegClassInfo, Writable, NUM_REG_CLASSES,
};

use std::string::{String, ToString};

//=============================================================================
// Registers, the Universe thereof, and printing

/// The pinned register on this architecture.
/// It must be the same as Spidermonkey's HeapReg, as found in this file.
/// https://searchfox.org/mozilla-central/source/js/src/jit/arm64/Assembler-arm64.h#103
pub const PINNED_REG: u8 = 21;

#[rustfmt::skip]
const XREG_INDICES: [u8; 31] = [
    // X0 - X7
    32, 33, 34, 35, 36, 37, 38, 39,
    // X8 - X15
    40, 41, 42, 43, 44, 45, 46, 47,
    // X16, X17
    58, 59,
    // X18
    60,
    // X19, X20
    48, 49,
    // X21, put aside because it's the pinned register.
    57,
    // X22 - X28
    50, 51, 52, 53, 54, 55, 56,
    // X29 (FP)
    61,
    // X30 (LR)
    62,
];

const ZERO_REG_INDEX: u8 = 63;

const SP_REG_INDEX: u8 = 64;

/// Get a reference to an X-register (integer register).
pub fn xreg(num: u8) -> Reg {
    assert!(num < 31);
    Reg::new_real(
        RegClass::I64,
        /* enc = */ num,
        /* index = */ XREG_INDICES[num as usize],
    )
}

/// Get a writable reference to an X-register.
pub fn writable_xreg(num: u8) -> Writable<Reg> {
    Writable::from_reg(xreg(num))
}

/// Get a reference to a V-register (vector/FP register).
pub fn vreg(num: u8) -> Reg {
    assert!(num < 32);
    Reg::new_real(RegClass::V128, /* enc = */ num, /* index = */ num)
}

/// Get a writable reference to a V-register.
pub fn writable_vreg(num: u8) -> Writable<Reg> {
    Writable::from_reg(vreg(num))
}

/// Get a reference to the zero-register.
pub fn zero_reg() -> Reg {
    // This should be the same as what xreg(31) returns, except that
    // we use the special index into the register index space.
    Reg::new_real(
        RegClass::I64,
        /* enc = */ 31,
        /* index = */ ZERO_REG_INDEX,
    )
}

/// Get a writable reference to the zero-register (this discards a result).
pub fn writable_zero_reg() -> Writable<Reg> {
    Writable::from_reg(zero_reg())
}

/// Get a reference to the stack-pointer register.
pub fn stack_reg() -> Reg {
    // XSP (stack) and XZR (zero) are logically different registers which have
    // the same hardware encoding, and whose meaning, in real aarch64
    // instructions, is context-dependent.  For convenience of
    // universe-construction and for correct printing, we make them be two
    // different real registers.
    Reg::new_real(
        RegClass::I64,
        /* enc = */ 31,
        /* index = */ SP_REG_INDEX,
    )
}

/// Get a writable reference to the stack-pointer register.
pub fn writable_stack_reg() -> Writable<Reg> {
    Writable::from_reg(stack_reg())
}

/// Get a reference to the link register (x30).
pub fn link_reg() -> Reg {
    xreg(30)
}

/// Get a writable reference to the link register.
pub fn writable_link_reg() -> Writable<Reg> {
    Writable::from_reg(link_reg())
}

/// Get a reference to the frame pointer (x29).
pub fn fp_reg() -> Reg {
    xreg(29)
}

/// Get a writable reference to the frame pointer.
pub fn writable_fp_reg() -> Writable<Reg> {
    Writable::from_reg(fp_reg())
}

/// Get a reference to the first temporary, sometimes "spill temporary", register. This register is
/// used to compute the address of a spill slot when a direct offset addressing mode from FP is not
/// sufficient (+/- 2^11 words). We exclude this register from regalloc and reserve it for this
/// purpose for simplicity; otherwise we need a multi-stage analysis where we first determine how
/// many spill slots we have, then perhaps remove the reg from the pool and recompute regalloc.
///
/// We use x16 for this (aka IP0 in the AArch64 ABI) because it's a scratch register but is
/// slightly special (used for linker veneers). We're free to use it as long as we don't expect it
/// to live through call instructions.
pub fn spilltmp_reg() -> Reg {
    xreg(16)
}

/// Get a writable reference to the spilltmp reg.
pub fn writable_spilltmp_reg() -> Writable<Reg> {
    Writable::from_reg(spilltmp_reg())
}

/// Get a reference to the second temp register. We need this in some edge cases
/// where we need both the spilltmp and another temporary.
///
/// We use x17 (aka IP1), the other "interprocedural"/linker-veneer scratch reg that is
/// free to use otherwise.
pub fn tmp2_reg() -> Reg {
    xreg(17)
}

/// Get a writable reference to the tmp2 reg.
pub fn writable_tmp2_reg() -> Writable<Reg> {
    Writable::from_reg(tmp2_reg())
}

/// Create the register universe for AArch64.
pub fn create_reg_universe(flags: &settings::Flags) -> RealRegUniverse {
    let mut regs = vec![];
    let mut allocable_by_class = [None; NUM_REG_CLASSES];

    // Numbering Scheme: we put V-regs first, then X-regs. The X-regs exclude several registers:
    // x18 (globally reserved for platform-specific purposes), x29 (frame pointer), x30 (link
    // register), x31 (stack pointer or zero register, depending on context).

    let v_reg_base = 0u8; // in contiguous real-register index space
    let v_reg_count = 32;
    for i in 0u8..v_reg_count {
        let reg = Reg::new_real(
            RegClass::V128,
            /* enc = */ i,
            /* index = */ v_reg_base + i,
        )
        .to_real_reg();
        let name = format!("v{}", i);
        regs.push((reg, name));
    }
    let v_reg_last = v_reg_base + v_reg_count - 1;

    // Add the X registers. N.B.: the order here must match the order implied
    // by XREG_INDICES, ZERO_REG_INDEX, and SP_REG_INDEX above.

    let x_reg_base = 32u8; // in contiguous real-register index space
    let mut x_reg_count = 0;

    let uses_pinned_reg = flags.enable_pinned_reg();

    for i in 0u8..32u8 {
        // See above for excluded registers.
        if i == 16 || i == 17 || i == 18 || i == 29 || i == 30 || i == 31 || i == PINNED_REG {
            continue;
        }
        let reg = Reg::new_real(
            RegClass::I64,
            /* enc = */ i,
            /* index = */ x_reg_base + x_reg_count,
        )
        .to_real_reg();
        let name = format!("x{}", i);
        regs.push((reg, name));
        x_reg_count += 1;
    }
    let x_reg_last = x_reg_base + x_reg_count - 1;

    allocable_by_class[RegClass::I64.rc_to_usize()] = Some(RegClassInfo {
        first: x_reg_base as usize,
        last: x_reg_last as usize,
        suggested_scratch: Some(XREG_INDICES[19] as usize),
    });
    allocable_by_class[RegClass::V128.rc_to_usize()] = Some(RegClassInfo {
        first: v_reg_base as usize,
        last: v_reg_last as usize,
        suggested_scratch: Some(/* V31: */ 31),
    });

    // Other regs, not available to the allocator.
    let allocable = if uses_pinned_reg {
        // The pinned register is not allocatable in this case, so record the length before adding
        // it.
        let len = regs.len();
        regs.push((xreg(PINNED_REG).to_real_reg(), "x21/pinned_reg".to_string()));
        len
    } else {
        regs.push((xreg(PINNED_REG).to_real_reg(), "x21".to_string()));
        regs.len()
    };

    regs.push((xreg(16).to_real_reg(), "x16".to_string()));
    regs.push((xreg(17).to_real_reg(), "x17".to_string()));
    regs.push((xreg(18).to_real_reg(), "x18".to_string()));
    regs.push((fp_reg().to_real_reg(), "fp".to_string()));
    regs.push((link_reg().to_real_reg(), "lr".to_string()));
    regs.push((zero_reg().to_real_reg(), "xzr".to_string()));
    regs.push((stack_reg().to_real_reg(), "sp".to_string()));

    // FIXME JRS 2020Feb06: unfortunately this pushes the number of real regs
    // to 65, which is potentially inconvenient from a compiler performance
    // standpoint.  We could possibly drop back to 64 by "losing" a vector
    // register in future.

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

/// If `ireg` denotes an I64-classed reg, make a best-effort attempt to show
/// its name at the 32-bit size.
pub fn show_ireg_sized(reg: Reg, mb_rru: Option<&RealRegUniverse>, size: OperandSize) -> String {
    let mut s = reg.show_rru(mb_rru);
    if reg.get_class() != RegClass::I64 || !size.is32() {
        // We can't do any better.
        return s;
    }

    if reg.is_real() {
        // Change (eg) "x42" into "w42" as appropriate
        if reg.get_class() == RegClass::I64 && size.is32() && s.starts_with("x") {
            s = "w".to_string() + &s[1..];
        }
    } else {
        // Add a "w" suffix to RegClass::I64 vregs used in a 32-bit role
        if reg.get_class() == RegClass::I64 && size.is32() {
            s.push('w');
        }
    }
    s
}

/// Show a vector register used in a scalar context.
pub fn show_vreg_scalar(reg: Reg, mb_rru: Option<&RealRegUniverse>, size: ScalarSize) -> String {
    let mut s = reg.show_rru(mb_rru);
    if reg.get_class() != RegClass::V128 {
        // We can't do any better.
        return s;
    }

    if reg.is_real() {
        // Change (eg) "v0" into "d0".
        if s.starts_with("v") {
            let replacement = match size {
                ScalarSize::Size8 => "b",
                ScalarSize::Size16 => "h",
                ScalarSize::Size32 => "s",
                ScalarSize::Size64 => "d",
                ScalarSize::Size128 => "q",
            };
            s.replace_range(0..1, replacement);
        }
    } else {
        // Add a "d" suffix to RegClass::V128 vregs.
        if reg.get_class() == RegClass::V128 {
            s.push('d');
        }
    }
    s
}

/// Show a vector register.
pub fn show_vreg_vector(reg: Reg, mb_rru: Option<&RealRegUniverse>, size: VectorSize) -> String {
    assert_eq!(RegClass::V128, reg.get_class());
    let mut s = reg.show_rru(mb_rru);

    let suffix = match size {
        VectorSize::Size8x8 => ".8b",
        VectorSize::Size8x16 => ".16b",
        VectorSize::Size16x4 => ".4h",
        VectorSize::Size16x8 => ".8h",
        VectorSize::Size32x2 => ".2s",
        VectorSize::Size32x4 => ".4s",
        VectorSize::Size64x2 => ".2d",
    };

    s.push_str(suffix);
    s
}

/// Show an indexed vector element.
pub fn show_vreg_element(
    reg: Reg,
    mb_rru: Option<&RealRegUniverse>,
    idx: u8,
    size: VectorSize,
) -> String {
    assert_eq!(RegClass::V128, reg.get_class());
    let mut s = reg.show_rru(mb_rru);

    let suffix = match size {
        VectorSize::Size8x8 => "b",
        VectorSize::Size8x16 => "b",
        VectorSize::Size16x4 => "h",
        VectorSize::Size16x8 => "h",
        VectorSize::Size32x2 => "s",
        VectorSize::Size32x4 => "s",
        VectorSize::Size64x2 => "d",
    };

    s.push_str(&format!(".{}[{}]", suffix, idx));
    s
}
