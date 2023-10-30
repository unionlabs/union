//! Registers, the Universe thereof, and printing.
//!
//! These are ordered by sequence number, as required in the Universe.
//!
//! The caller-saved registers are placed first in order to prefer not to clobber (requiring
//! saves/restores in prologue/epilogue code) when possible. Note that there is no other heuristic
//! in the backend that will apply such pressure; the register allocator's cost heuristics are not
//! aware of the cost of clobber-save/restore code.
//!
//! One might worry that this pessimizes code with many callsites, where using caller-saves causes
//! us to have to save them (as we are the caller) frequently. However, the register allocator
//! *should be* aware of *this* cost, because it sees that the call instruction modifies all of the
//! caller-saved (i.e., callee-clobbered) registers.
//!
//! Hence, this ordering encodes pressure in one direction (prefer not to clobber registers that we
//! ourselves have to save) and this is balanaced against the RA's pressure in the other direction
//! at callsites.

use crate::settings;
use alloc::vec::Vec;
use regalloc::{
    PrettyPrint, RealReg, RealRegUniverse, Reg, RegClass, RegClassInfo, NUM_REG_CLASSES,
};
use std::string::String;

// Hardware encodings (note the special rax, rcx, rdx, rbx order).

pub const ENC_RAX: u8 = 0;
pub const ENC_RCX: u8 = 1;
pub const ENC_RDX: u8 = 2;
pub const ENC_RBX: u8 = 3;
pub const ENC_RSP: u8 = 4;
pub const ENC_RBP: u8 = 5;
pub const ENC_RSI: u8 = 6;
pub const ENC_RDI: u8 = 7;
pub const ENC_R8: u8 = 8;
pub const ENC_R9: u8 = 9;
pub const ENC_R10: u8 = 10;
pub const ENC_R11: u8 = 11;
pub const ENC_R12: u8 = 12;
pub const ENC_R13: u8 = 13;
pub const ENC_R14: u8 = 14;
pub const ENC_R15: u8 = 15;

fn gpr(enc: u8, index: u8) -> Reg {
    Reg::new_real(RegClass::I64, enc, index)
}

pub(crate) fn rsi() -> Reg {
    gpr(ENC_RSI, 16)
}
pub(crate) fn rdi() -> Reg {
    gpr(ENC_RDI, 17)
}
pub(crate) fn rax() -> Reg {
    gpr(ENC_RAX, 18)
}
pub(crate) fn rcx() -> Reg {
    gpr(ENC_RCX, 19)
}
pub(crate) fn rdx() -> Reg {
    gpr(ENC_RDX, 20)
}
pub(crate) fn r8() -> Reg {
    gpr(ENC_R8, 21)
}
pub(crate) fn r9() -> Reg {
    gpr(ENC_R9, 22)
}
pub(crate) fn r10() -> Reg {
    gpr(ENC_R10, 23)
}
pub(crate) fn r11() -> Reg {
    gpr(ENC_R11, 24)
}
pub(crate) fn r12() -> Reg {
    gpr(ENC_R12, 25)
}
pub(crate) fn r13() -> Reg {
    gpr(ENC_R13, 26)
}
pub(crate) fn r14() -> Reg {
    gpr(ENC_R14, 27)
}
pub(crate) fn rbx() -> Reg {
    gpr(ENC_RBX, 28)
}

pub(crate) fn r15() -> Reg {
    // r15 is put aside since this is the pinned register.
    gpr(ENC_R15, 29)
}

/// The pinned register on this architecture.
/// It must be the same as Spidermonkey's HeapReg, as found in this file.
/// https://searchfox.org/mozilla-central/source/js/src/jit/x64/Assembler-x64.h#99
pub(crate) fn pinned_reg() -> Reg {
    r15()
}

fn fpr(enc: u8, index: u8) -> Reg {
    Reg::new_real(RegClass::V128, enc, index)
}

pub(crate) fn xmm0() -> Reg {
    fpr(0, 0)
}
pub(crate) fn xmm1() -> Reg {
    fpr(1, 1)
}
pub(crate) fn xmm2() -> Reg {
    fpr(2, 2)
}
pub(crate) fn xmm3() -> Reg {
    fpr(3, 3)
}
pub(crate) fn xmm4() -> Reg {
    fpr(4, 4)
}
pub(crate) fn xmm5() -> Reg {
    fpr(5, 5)
}
pub(crate) fn xmm6() -> Reg {
    fpr(6, 6)
}
pub(crate) fn xmm7() -> Reg {
    fpr(7, 7)
}
pub(crate) fn xmm8() -> Reg {
    fpr(8, 8)
}
pub(crate) fn xmm9() -> Reg {
    fpr(9, 9)
}
pub(crate) fn xmm10() -> Reg {
    fpr(10, 10)
}
pub(crate) fn xmm11() -> Reg {
    fpr(11, 11)
}
pub(crate) fn xmm12() -> Reg {
    fpr(12, 12)
}
pub(crate) fn xmm13() -> Reg {
    fpr(13, 13)
}
pub(crate) fn xmm14() -> Reg {
    fpr(14, 14)
}
pub(crate) fn xmm15() -> Reg {
    fpr(15, 15)
}

pub(crate) fn rsp() -> Reg {
    gpr(ENC_RSP, 30)
}
pub(crate) fn rbp() -> Reg {
    gpr(ENC_RBP, 31)
}

/// Create the register universe for X64.
///
/// The ordering of registers matters, as commented in the file doc comment: assumes the
/// calling-convention is SystemV, at the moment.
pub(crate) fn create_reg_universe_systemv(flags: &settings::Flags) -> RealRegUniverse {
    let mut regs = Vec::<(RealReg, String)>::new();
    let mut allocable_by_class = [None; NUM_REG_CLASSES];

    let use_pinned_reg = flags.enable_pinned_reg();

    // XMM registers
    let first_fpr = regs.len();
    regs.push((xmm0().to_real_reg(), "%xmm0".into()));
    regs.push((xmm1().to_real_reg(), "%xmm1".into()));
    regs.push((xmm2().to_real_reg(), "%xmm2".into()));
    regs.push((xmm3().to_real_reg(), "%xmm3".into()));
    regs.push((xmm4().to_real_reg(), "%xmm4".into()));
    regs.push((xmm5().to_real_reg(), "%xmm5".into()));
    regs.push((xmm6().to_real_reg(), "%xmm6".into()));
    regs.push((xmm7().to_real_reg(), "%xmm7".into()));
    regs.push((xmm8().to_real_reg(), "%xmm8".into()));
    regs.push((xmm9().to_real_reg(), "%xmm9".into()));
    regs.push((xmm10().to_real_reg(), "%xmm10".into()));
    regs.push((xmm11().to_real_reg(), "%xmm11".into()));
    regs.push((xmm12().to_real_reg(), "%xmm12".into()));
    regs.push((xmm13().to_real_reg(), "%xmm13".into()));
    regs.push((xmm14().to_real_reg(), "%xmm14".into()));
    regs.push((xmm15().to_real_reg(), "%xmm15".into()));
    let last_fpr = regs.len() - 1;

    // Integer regs.
    let first_gpr = regs.len();

    // Caller-saved, in the SystemV x86_64 ABI.
    regs.push((rsi().to_real_reg(), "%rsi".into()));
    regs.push((rdi().to_real_reg(), "%rdi".into()));
    regs.push((rax().to_real_reg(), "%rax".into()));
    regs.push((rcx().to_real_reg(), "%rcx".into()));
    regs.push((rdx().to_real_reg(), "%rdx".into()));
    regs.push((r8().to_real_reg(), "%r8".into()));
    regs.push((r9().to_real_reg(), "%r9".into()));
    regs.push((r10().to_real_reg(), "%r10".into()));
    regs.push((r11().to_real_reg(), "%r11".into()));

    // Callee-saved, in the SystemV x86_64 ABI.
    regs.push((r12().to_real_reg(), "%r12".into()));
    regs.push((r13().to_real_reg(), "%r13".into()));
    regs.push((r14().to_real_reg(), "%r14".into()));

    regs.push((rbx().to_real_reg(), "%rbx".into()));

    // Other regs, not available to the allocator.
    debug_assert_eq!(r15(), pinned_reg());
    let allocable = if use_pinned_reg {
        // The pinned register is not allocatable in this case, so record the length before adding
        // it.
        let len = regs.len();
        regs.push((r15().to_real_reg(), "%r15/pinned".into()));
        len
    } else {
        regs.push((r15().to_real_reg(), "%r15".into()));
        regs.len()
    };
    let last_gpr = allocable - 1;

    regs.push((rsp().to_real_reg(), "%rsp".into()));
    regs.push((rbp().to_real_reg(), "%rbp".into()));

    allocable_by_class[RegClass::I64.rc_to_usize()] = Some(RegClassInfo {
        first: first_gpr,
        last: last_gpr,
        suggested_scratch: Some(r12().get_index()),
    });
    allocable_by_class[RegClass::V128.rc_to_usize()] = Some(RegClassInfo {
        first: first_fpr,
        last: last_fpr,
        suggested_scratch: Some(xmm15().get_index()),
    });

    // Sanity-check: the index passed to the Reg ctor must match the order in the register list.
    for (i, reg) in regs.iter().enumerate() {
        assert_eq!(i, reg.0.get_index());
    }

    RealRegUniverse {
        regs,
        allocable,
        allocable_by_class,
    }
}

/// If `ireg` denotes an I64-classed reg, make a best-effort attempt to show its name at some
/// smaller size (4, 2 or 1 bytes).
pub fn show_ireg_sized(reg: Reg, mb_rru: Option<&RealRegUniverse>, size: u8) -> String {
    let mut s = reg.show_rru(mb_rru);

    if reg.get_class() != RegClass::I64 || size == 8 {
        // We can't do any better.
        return s;
    }

    if reg.is_real() {
        // Change (eg) "rax" into "eax", "ax" or "al" as appropriate.  This is something one could
        // describe diplomatically as "a kludge", but it's only debug code.
        let remapper = match s.as_str() {
            "%rax" => Some(["%eax", "%ax", "%al"]),
            "%rbx" => Some(["%ebx", "%bx", "%bl"]),
            "%rcx" => Some(["%ecx", "%cx", "%cl"]),
            "%rdx" => Some(["%edx", "%dx", "%dl"]),
            "%rsi" => Some(["%esi", "%si", "%sil"]),
            "%rdi" => Some(["%edi", "%di", "%dil"]),
            "%rbp" => Some(["%ebp", "%bp", "%bpl"]),
            "%rsp" => Some(["%esp", "%sp", "%spl"]),
            "%r8" => Some(["%r8d", "%r8w", "%r8b"]),
            "%r9" => Some(["%r9d", "%r9w", "%r9b"]),
            "%r10" => Some(["%r10d", "%r10w", "%r10b"]),
            "%r11" => Some(["%r11d", "%r11w", "%r11b"]),
            "%r12" => Some(["%r12d", "%r12w", "%r12b"]),
            "%r13" => Some(["%r13d", "%r13w", "%r13b"]),
            "%r14" => Some(["%r14d", "%r14w", "%r14b"]),
            "%r15" => Some(["%r15d", "%r15w", "%r15b"]),
            _ => None,
        };
        if let Some(smaller_names) = remapper {
            match size {
                4 => s = smaller_names[0].into(),
                2 => s = smaller_names[1].into(),
                1 => s = smaller_names[2].into(),
                _ => panic!("show_ireg_sized: real"),
            }
        }
    } else {
        // Add a "l", "w" or "b" suffix to RegClass::I64 vregs used at narrower widths.
        let suffix = match size {
            4 => "l",
            2 => "w",
            1 => "b",
            _ => panic!("show_ireg_sized: virtual"),
        };
        s = s + suffix;
    }

    s
}
