//! AArch64 ISA: binary code emission.

use crate::binemit::{CodeOffset, Reloc, StackMap};
use crate::ir::constant::ConstantData;
use crate::ir::types::*;
use crate::ir::{LibCall, MemFlags, TrapCode};
use crate::isa::aarch64::inst::*;
use crate::machinst::ty_bits;

use regalloc::{Reg, RegClass, Writable};

use core::convert::TryFrom;

/// Memory label/reference finalization: convert a MemLabel to a PC-relative
/// offset, possibly emitting relocation(s) as necessary.
pub fn memlabel_finalize(_insn_off: CodeOffset, label: &MemLabel) -> i32 {
    match label {
        &MemLabel::PCRel(rel) => rel,
    }
}

/// Memory addressing mode finalization: convert "special" modes (e.g.,
/// generic arbitrary stack offset) into real addressing modes, possibly by
/// emitting some helper instructions that come immediately before the use
/// of this amode.
pub fn mem_finalize(
    insn_off: CodeOffset,
    mem: &AMode,
    state: &EmitState,
) -> (SmallVec<[Inst; 4]>, AMode) {
    match mem {
        &AMode::RegOffset(_, off, ty)
        | &AMode::SPOffset(off, ty)
        | &AMode::FPOffset(off, ty)
        | &AMode::NominalSPOffset(off, ty) => {
            let basereg = match mem {
                &AMode::RegOffset(reg, _, _) => reg,
                &AMode::SPOffset(..) | &AMode::NominalSPOffset(..) => stack_reg(),
                &AMode::FPOffset(..) => fp_reg(),
                _ => unreachable!(),
            };
            let adj = match mem {
                &AMode::NominalSPOffset(..) => {
                    log::trace!(
                        "mem_finalize: nominal SP offset {} + adj {} -> {}",
                        off,
                        state.virtual_sp_offset,
                        off + state.virtual_sp_offset
                    );
                    state.virtual_sp_offset
                }
                _ => 0,
            };
            let off = off + adj;

            if let Some(simm9) = SImm9::maybe_from_i64(off) {
                let mem = AMode::Unscaled(basereg, simm9);
                (smallvec![], mem)
            } else if let Some(uimm12s) = UImm12Scaled::maybe_from_i64(off, ty) {
                let mem = AMode::UnsignedOffset(basereg, uimm12s);
                (smallvec![], mem)
            } else {
                let tmp = writable_spilltmp_reg();
                let mut const_insts = Inst::load_constant(tmp, off as u64);
                // N.B.: we must use AluRRRExtend because AluRRR uses the "shifted register" form
                // (AluRRRShift) instead, which interprets register 31 as the zero reg, not SP. SP
                // is a valid base (for SPOffset) which we must handle here.
                // Also, SP needs to be the first arg, not second.
                let add_inst = Inst::AluRRRExtend {
                    alu_op: ALUOp::Add,
                    size: OperandSize::Size64,
                    rd: tmp,
                    rn: basereg,
                    rm: tmp.to_reg(),
                    extendop: ExtendOp::UXTX,
                };
                const_insts.push(add_inst);
                (const_insts, AMode::reg(tmp.to_reg()))
            }
        }

        &AMode::Label(ref label) => {
            let off = memlabel_finalize(insn_off, label);
            (smallvec![], AMode::Label(MemLabel::PCRel(off)))
        }

        _ => (smallvec![], mem.clone()),
    }
}

/// Helper: get a ConstantData from a u64.
pub fn u64_constant(bits: u64) -> ConstantData {
    let data = bits.to_le_bytes();
    ConstantData::from(&data[..])
}

//=============================================================================
// Instructions and subcomponents: emission

fn machreg_to_gpr(m: Reg) -> u32 {
    assert_eq!(m.get_class(), RegClass::I64);
    u32::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

fn machreg_to_vec(m: Reg) -> u32 {
    assert_eq!(m.get_class(), RegClass::V128);
    u32::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

fn machreg_to_gpr_or_vec(m: Reg) -> u32 {
    u32::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

pub(crate) fn enc_arith_rrr(
    bits_31_21: u32,
    bits_15_10: u32,
    rd: Writable<Reg>,
    rn: Reg,
    rm: Reg,
) -> u32 {
    (bits_31_21 << 21)
        | (bits_15_10 << 10)
        | machreg_to_gpr(rd.to_reg())
        | (machreg_to_gpr(rn) << 5)
        | (machreg_to_gpr(rm) << 16)
}

fn enc_arith_rr_imm12(
    bits_31_24: u32,
    immshift: u32,
    imm12: u32,
    rn: Reg,
    rd: Writable<Reg>,
) -> u32 {
    (bits_31_24 << 24)
        | (immshift << 22)
        | (imm12 << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rd.to_reg())
}

fn enc_arith_rr_imml(bits_31_23: u32, imm_bits: u32, rn: Reg, rd: Writable<Reg>) -> u32 {
    (bits_31_23 << 23) | (imm_bits << 10) | (machreg_to_gpr(rn) << 5) | machreg_to_gpr(rd.to_reg())
}

fn enc_arith_rrrr(top11: u32, rm: Reg, bit15: u32, ra: Reg, rn: Reg, rd: Writable<Reg>) -> u32 {
    (top11 << 21)
        | (machreg_to_gpr(rm) << 16)
        | (bit15 << 15)
        | (machreg_to_gpr(ra) << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rd.to_reg())
}

fn enc_jump26(op_31_26: u32, off_26_0: u32) -> u32 {
    assert!(off_26_0 < (1 << 26));
    (op_31_26 << 26) | off_26_0
}

fn enc_cmpbr(op_31_24: u32, off_18_0: u32, reg: Reg) -> u32 {
    assert!(off_18_0 < (1 << 19));
    (op_31_24 << 24) | (off_18_0 << 5) | machreg_to_gpr(reg)
}

fn enc_cbr(op_31_24: u32, off_18_0: u32, op_4: u32, cond: u32) -> u32 {
    assert!(off_18_0 < (1 << 19));
    assert!(cond < (1 << 4));
    (op_31_24 << 24) | (off_18_0 << 5) | (op_4 << 4) | cond
}

fn enc_conditional_br(taken: BranchTarget, kind: CondBrKind) -> u32 {
    match kind {
        CondBrKind::Zero(reg) => enc_cmpbr(0b1_011010_0, taken.as_offset19_or_zero(), reg),
        CondBrKind::NotZero(reg) => enc_cmpbr(0b1_011010_1, taken.as_offset19_or_zero(), reg),
        CondBrKind::Cond(c) => enc_cbr(0b01010100, taken.as_offset19_or_zero(), 0b0, c.bits()),
    }
}

const MOVE_WIDE_FIXED: u32 = 0x12800000;

#[repr(u32)]
enum MoveWideOpcode {
    MOVN = 0b00,
    MOVZ = 0b10,
    MOVK = 0b11,
}

fn enc_move_wide(
    op: MoveWideOpcode,
    rd: Writable<Reg>,
    imm: MoveWideConst,
    size: OperandSize,
) -> u32 {
    assert!(imm.shift <= 0b11);
    MOVE_WIDE_FIXED
        | size.sf_bit() << 31
        | (op as u32) << 29
        | u32::from(imm.shift) << 21
        | u32::from(imm.bits) << 5
        | machreg_to_gpr(rd.to_reg())
}

fn enc_ldst_pair(op_31_22: u32, simm7: SImm7Scaled, rn: Reg, rt: Reg, rt2: Reg) -> u32 {
    (op_31_22 << 22)
        | (simm7.bits() << 15)
        | (machreg_to_gpr(rt2) << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt)
}

fn enc_ldst_simm9(op_31_22: u32, simm9: SImm9, op_11_10: u32, rn: Reg, rd: Reg) -> u32 {
    (op_31_22 << 22)
        | (simm9.bits() << 12)
        | (op_11_10 << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr_or_vec(rd)
}

fn enc_ldst_uimm12(op_31_22: u32, uimm12: UImm12Scaled, rn: Reg, rd: Reg) -> u32 {
    (op_31_22 << 22)
        | (0b1 << 24)
        | (uimm12.bits() << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr_or_vec(rd)
}

fn enc_ldst_reg(
    op_31_22: u32,
    rn: Reg,
    rm: Reg,
    s_bit: bool,
    extendop: Option<ExtendOp>,
    rd: Reg,
) -> u32 {
    let s_bit = if s_bit { 1 } else { 0 };
    let extend_bits = match extendop {
        Some(ExtendOp::UXTW) => 0b010,
        Some(ExtendOp::SXTW) => 0b110,
        Some(ExtendOp::SXTX) => 0b111,
        None => 0b011, // LSL
        _ => panic!("bad extend mode for ld/st AMode"),
    };
    (op_31_22 << 22)
        | (1 << 21)
        | (machreg_to_gpr(rm) << 16)
        | (extend_bits << 13)
        | (s_bit << 12)
        | (0b10 << 10)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr_or_vec(rd)
}

pub(crate) fn enc_ldst_imm19(op_31_24: u32, imm19: u32, rd: Reg) -> u32 {
    (op_31_24 << 24) | (imm19 << 5) | machreg_to_gpr_or_vec(rd)
}

fn enc_ldst_vec(q: u32, size: u32, rn: Reg, rt: Writable<Reg>) -> u32 {
    debug_assert_eq!(q & 0b1, q);
    debug_assert_eq!(size & 0b11, size);
    0b0_0_0011010_10_00000_110_0_00_00000_00000
        | q << 30
        | size << 10
        | machreg_to_gpr(rn) << 5
        | machreg_to_vec(rt.to_reg())
}

fn enc_ldst_vec_pair(
    opc: u32,
    amode: u32,
    is_load: bool,
    simm7: SImm7Scaled,
    rn: Reg,
    rt: Reg,
    rt2: Reg,
) -> u32 {
    debug_assert_eq!(opc & 0b11, opc);
    debug_assert_eq!(amode & 0b11, amode);

    0b00_10110_00_0_0000000_00000_00000_00000
        | opc << 30
        | amode << 23
        | (is_load as u32) << 22
        | simm7.bits() << 15
        | machreg_to_vec(rt2) << 10
        | machreg_to_gpr(rn) << 5
        | machreg_to_vec(rt)
}

fn enc_vec_rrr(top11: u32, rm: Reg, bit15_10: u32, rn: Reg, rd: Writable<Reg>) -> u32 {
    (top11 << 21)
        | (machreg_to_vec(rm) << 16)
        | (bit15_10 << 10)
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
}

fn enc_vec_rrr_long(
    q: u32,
    u: u32,
    size: u32,
    bit14: u32,
    rm: Reg,
    rn: Reg,
    rd: Writable<Reg>,
) -> u32 {
    debug_assert_eq!(q & 0b1, q);
    debug_assert_eq!(u & 0b1, u);
    debug_assert_eq!(size & 0b11, size);
    debug_assert_eq!(bit14 & 0b1, bit14);

    0b0_0_0_01110_00_1_00000_100000_00000_00000
        | q << 30
        | u << 29
        | size << 22
        | bit14 << 14
        | (machreg_to_vec(rm) << 16)
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
}

fn enc_bit_rr(size: u32, opcode2: u32, opcode1: u32, rn: Reg, rd: Writable<Reg>) -> u32 {
    (0b01011010110 << 21)
        | size << 31
        | opcode2 << 16
        | opcode1 << 10
        | machreg_to_gpr(rn) << 5
        | machreg_to_gpr(rd.to_reg())
}

pub(crate) fn enc_br(rn: Reg) -> u32 {
    0b1101011_0000_11111_000000_00000_00000 | (machreg_to_gpr(rn) << 5)
}

pub(crate) fn enc_adr(off: i32, rd: Writable<Reg>) -> u32 {
    let off = u32::try_from(off).unwrap();
    let immlo = off & 3;
    let immhi = (off >> 2) & ((1 << 19) - 1);
    (0b00010000 << 24) | (immlo << 29) | (immhi << 5) | machreg_to_gpr(rd.to_reg())
}

fn enc_csel(rd: Writable<Reg>, rn: Reg, rm: Reg, cond: Cond) -> u32 {
    0b100_11010100_00000_0000_00_00000_00000
        | (machreg_to_gpr(rm) << 16)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rd.to_reg())
        | (cond.bits() << 12)
}

fn enc_fcsel(rd: Writable<Reg>, rn: Reg, rm: Reg, cond: Cond, size: ScalarSize) -> u32 {
    0b000_11110_00_1_00000_0000_11_00000_00000
        | (size.ftype() << 22)
        | (machreg_to_vec(rm) << 16)
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
        | (cond.bits() << 12)
}

fn enc_cset(rd: Writable<Reg>, cond: Cond) -> u32 {
    0b100_11010100_11111_0000_01_11111_00000
        | machreg_to_gpr(rd.to_reg())
        | (cond.invert().bits() << 12)
}

fn enc_csetm(rd: Writable<Reg>, cond: Cond) -> u32 {
    0b110_11010100_11111_0000_00_11111_00000
        | machreg_to_gpr(rd.to_reg())
        | (cond.invert().bits() << 12)
}

fn enc_ccmp_imm(size: OperandSize, rn: Reg, imm: UImm5, nzcv: NZCV, cond: Cond) -> u32 {
    0b0_1_1_11010010_00000_0000_10_00000_0_0000
        | size.sf_bit() << 31
        | imm.bits() << 16
        | cond.bits() << 12
        | machreg_to_gpr(rn) << 5
        | nzcv.bits()
}

fn enc_bfm(opc: u8, size: OperandSize, rd: Writable<Reg>, rn: Reg, immr: u8, imms: u8) -> u32 {
    match size {
        OperandSize::Size64 => {
            debug_assert!(immr <= 63);
            debug_assert!(imms <= 63);
        }
        OperandSize::Size32 => {
            debug_assert!(immr <= 31);
            debug_assert!(imms <= 31);
        }
    }
    debug_assert_eq!(opc & 0b11, opc);
    let n_bit = size.sf_bit();
    0b0_00_100110_0_000000_000000_00000_00000
        | size.sf_bit() << 31
        | u32::from(opc) << 29
        | n_bit << 22
        | u32::from(immr) << 16
        | u32::from(imms) << 10
        | machreg_to_gpr(rn) << 5
        | machreg_to_gpr(rd.to_reg())
}

fn enc_vecmov(is_16b: bool, rd: Writable<Reg>, rn: Reg) -> u32 {
    0b00001110_101_00000_00011_1_00000_00000
        | ((is_16b as u32) << 30)
        | machreg_to_vec(rd.to_reg())
        | (machreg_to_vec(rn) << 16)
        | (machreg_to_vec(rn) << 5)
}

fn enc_fpurr(top22: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    (top22 << 10) | (machreg_to_vec(rn) << 5) | machreg_to_vec(rd.to_reg())
}

fn enc_fpurrr(top22: u32, rd: Writable<Reg>, rn: Reg, rm: Reg) -> u32 {
    (top22 << 10)
        | (machreg_to_vec(rm) << 16)
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
}

fn enc_fpurrrr(top17: u32, rd: Writable<Reg>, rn: Reg, rm: Reg, ra: Reg) -> u32 {
    (top17 << 15)
        | (machreg_to_vec(rm) << 16)
        | (machreg_to_vec(ra) << 10)
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
}

fn enc_fcmp(size: ScalarSize, rn: Reg, rm: Reg) -> u32 {
    0b000_11110_00_1_00000_00_1000_00000_00000
        | (size.ftype() << 22)
        | (machreg_to_vec(rm) << 16)
        | (machreg_to_vec(rn) << 5)
}

fn enc_fputoint(top16: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    (top16 << 16) | (machreg_to_vec(rn) << 5) | machreg_to_gpr(rd.to_reg())
}

fn enc_inttofpu(top16: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    (top16 << 16) | (machreg_to_gpr(rn) << 5) | machreg_to_vec(rd.to_reg())
}

fn enc_fround(top22: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    (top22 << 10) | (machreg_to_vec(rn) << 5) | machreg_to_vec(rd.to_reg())
}

fn enc_vec_rr_misc(qu: u32, size: u32, bits_12_16: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    debug_assert_eq!(qu & 0b11, qu);
    debug_assert_eq!(size & 0b11, size);
    debug_assert_eq!(bits_12_16 & 0b11111, bits_12_16);
    let bits = 0b0_00_01110_00_10000_00000_10_00000_00000;
    bits | qu << 29
        | size << 22
        | bits_12_16 << 12
        | machreg_to_vec(rn) << 5
        | machreg_to_vec(rd.to_reg())
}

fn enc_vec_rr_pair(bits_12_16: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    debug_assert_eq!(bits_12_16 & 0b11111, bits_12_16);

    0b010_11110_11_11000_11011_10_00000_00000
        | bits_12_16 << 12
        | machreg_to_vec(rn) << 5
        | machreg_to_vec(rd.to_reg())
}

fn enc_vec_rr_pair_long(u: u32, enc_size: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    debug_assert_eq!(u & 0b1, u);
    debug_assert_eq!(enc_size & 0b1, enc_size);

    0b0_1_0_01110_00_10000_00_0_10_10_00000_00000
        | u << 29
        | enc_size << 22
        | machreg_to_vec(rn) << 5
        | machreg_to_vec(rd.to_reg())
}

fn enc_vec_lanes(q: u32, u: u32, size: u32, opcode: u32, rd: Writable<Reg>, rn: Reg) -> u32 {
    debug_assert_eq!(q & 0b1, q);
    debug_assert_eq!(u & 0b1, u);
    debug_assert_eq!(size & 0b11, size);
    debug_assert_eq!(opcode & 0b11111, opcode);
    0b0_0_0_01110_00_11000_0_0000_10_00000_00000
        | q << 30
        | u << 29
        | size << 22
        | opcode << 12
        | machreg_to_vec(rn) << 5
        | machreg_to_vec(rd.to_reg())
}

fn enc_tbl(is_extension: bool, len: u32, rd: Writable<Reg>, rn: Reg, rm: Reg) -> u32 {
    debug_assert_eq!(len & 0b11, len);
    0b0_1_001110_000_00000_0_00_0_00_00000_00000
        | (machreg_to_vec(rm) << 16)
        | len << 13
        | (is_extension as u32) << 12
        | (machreg_to_vec(rn) << 5)
        | machreg_to_vec(rd.to_reg())
}

fn enc_dmb_ish() -> u32 {
    0xD5033BBF
}

fn enc_ldal(ty: Type, op: AtomicRMWOp, rs: Reg, rt: Writable<Reg>, rn: Reg) -> u32 {
    assert!(machreg_to_gpr(rt.to_reg()) != 31);
    let sz = match ty {
        I64 => 0b11,
        I32 => 0b10,
        I16 => 0b01,
        I8 => 0b00,
        _ => unreachable!(),
    };
    let op = match op {
        AtomicRMWOp::Add => 0b000,
        AtomicRMWOp::Clr => 0b001,
        AtomicRMWOp::Eor => 0b010,
        AtomicRMWOp::Set => 0b011,
        AtomicRMWOp::Smax => 0b100,
        AtomicRMWOp::Smin => 0b101,
        AtomicRMWOp::Umax => 0b110,
        AtomicRMWOp::Umin => 0b111,
    };
    0b00_111_000_111_00000_0_000_00_00000_00000
        | (sz << 30)
        | (machreg_to_gpr(rs) << 16)
        | (op << 12)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt.to_reg())
}

fn enc_ldar(ty: Type, rt: Writable<Reg>, rn: Reg) -> u32 {
    let sz = match ty {
        I64 => 0b11,
        I32 => 0b10,
        I16 => 0b01,
        I8 => 0b00,
        _ => unreachable!(),
    };
    0b00_001000_1_1_0_11111_1_11111_00000_00000
        | (sz << 30)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt.to_reg())
}

fn enc_stlr(ty: Type, rt: Reg, rn: Reg) -> u32 {
    let sz = match ty {
        I64 => 0b11,
        I32 => 0b10,
        I16 => 0b01,
        I8 => 0b00,
        _ => unreachable!(),
    };
    0b00_001000_100_11111_1_11111_00000_00000
        | (sz << 30)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt)
}

fn enc_ldaxr(ty: Type, rt: Writable<Reg>, rn: Reg) -> u32 {
    let sz = match ty {
        I64 => 0b11,
        I32 => 0b10,
        I16 => 0b01,
        I8 => 0b00,
        _ => unreachable!(),
    };
    0b00_001000_0_1_0_11111_1_11111_00000_00000
        | (sz << 30)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt.to_reg())
}

fn enc_stlxr(ty: Type, rs: Writable<Reg>, rt: Reg, rn: Reg) -> u32 {
    let sz = match ty {
        I64 => 0b11,
        I32 => 0b10,
        I16 => 0b01,
        I8 => 0b00,
        _ => unreachable!(),
    };
    0b00_001000_000_00000_1_11111_00000_00000
        | (sz << 30)
        | (machreg_to_gpr(rs.to_reg()) << 16)
        | (machreg_to_gpr(rn) << 5)
        | machreg_to_gpr(rt)
}

fn enc_cas(size: u32, rs: Writable<Reg>, rt: Reg, rn: Reg) -> u32 {
    debug_assert_eq!(size & 0b11, size);

    0b00_0010001_1_1_00000_1_11111_00000_00000
        | size << 30
        | machreg_to_gpr(rs.to_reg()) << 16
        | machreg_to_gpr(rn) << 5
        | machreg_to_gpr(rt)
}

fn enc_asimd_mod_imm(rd: Writable<Reg>, q_op: u32, cmode: u32, imm: u8) -> u32 {
    let abc = (imm >> 5) as u32;
    let defgh = (imm & 0b11111) as u32;

    debug_assert_eq!(cmode & 0b1111, cmode);
    debug_assert_eq!(q_op & 0b11, q_op);

    0b0_0_0_0111100000_000_0000_01_00000_00000
        | (q_op << 29)
        | (abc << 16)
        | (cmode << 12)
        | (defgh << 5)
        | machreg_to_vec(rd.to_reg())
}

/// State carried between emissions of a sequence of instructions.
#[derive(Default, Clone, Debug)]
pub struct EmitState {
    /// Addend to convert nominal-SP offsets to real-SP offsets at the current
    /// program point.
    pub(crate) virtual_sp_offset: i64,
    /// Offset of FP from nominal-SP.
    pub(crate) nominal_sp_to_fp: i64,
    /// Safepoint stack map for upcoming instruction, as provided to `pre_safepoint()`.
    stack_map: Option<StackMap>,
    /// Current source-code location corresponding to instruction to be emitted.
    cur_srcloc: SourceLoc,
}

impl MachInstEmitState<Inst> for EmitState {
    fn new(abi: &dyn ABICallee<I = Inst>) -> Self {
        EmitState {
            virtual_sp_offset: 0,
            nominal_sp_to_fp: abi.frame_size() as i64,
            stack_map: None,
            cur_srcloc: SourceLoc::default(),
        }
    }

    fn pre_safepoint(&mut self, stack_map: StackMap) {
        self.stack_map = Some(stack_map);
    }

    fn pre_sourceloc(&mut self, srcloc: SourceLoc) {
        self.cur_srcloc = srcloc;
    }
}

impl EmitState {
    fn take_stack_map(&mut self) -> Option<StackMap> {
        self.stack_map.take()
    }

    fn clear_post_insn(&mut self) {
        self.stack_map = None;
    }

    fn cur_srcloc(&self) -> SourceLoc {
        self.cur_srcloc
    }
}

/// Constant state used during function compilation.
pub struct EmitInfo(settings::Flags);

impl EmitInfo {
    pub(crate) fn new(flags: settings::Flags) -> Self {
        Self(flags)
    }
}

impl MachInstEmit for Inst {
    type State = EmitState;
    type Info = EmitInfo;

    fn emit(&self, sink: &mut MachBuffer<Inst>, emit_info: &Self::Info, state: &mut EmitState) {
        // N.B.: we *must* not exceed the "worst-case size" used to compute
        // where to insert islands, except when islands are explicitly triggered
        // (with an `EmitIsland`). We check this in debug builds. This is `mut`
        // to allow disabling the check for `JTSequence`, which is always
        // emitted following an `EmitIsland`.
        let mut start_off = sink.cur_offset();

        match self {
            &Inst::AluRRR {
                alu_op,
                size,
                rd,
                rn,
                rm,
            } => {
                debug_assert!(match alu_op {
                    ALUOp::SDiv | ALUOp::UDiv | ALUOp::SMulH | ALUOp::UMulH =>
                        size == OperandSize::Size64,
                    _ => true,
                });
                let top11 = match alu_op {
                    ALUOp::Add => 0b00001011_000,
                    ALUOp::Adc => 0b00011010_000,
                    ALUOp::AdcS => 0b00111010_000,
                    ALUOp::Sub => 0b01001011_000,
                    ALUOp::Sbc => 0b01011010_000,
                    ALUOp::SbcS => 0b01111010_000,
                    ALUOp::Orr => 0b00101010_000,
                    ALUOp::And => 0b00001010_000,
                    ALUOp::AndS => 0b01101010_000,
                    ALUOp::Eor => 0b01001010_000,
                    ALUOp::OrrNot => 0b00101010_001,
                    ALUOp::AndNot => 0b00001010_001,
                    ALUOp::EorNot => 0b01001010_001,
                    ALUOp::AddS => 0b00101011_000,
                    ALUOp::SubS => 0b01101011_000,
                    ALUOp::SDiv => 0b10011010_110,
                    ALUOp::UDiv => 0b10011010_110,
                    ALUOp::RotR | ALUOp::Lsr | ALUOp::Asr | ALUOp::Lsl => 0b00011010_110,
                    ALUOp::SMulH => 0b10011011_010,
                    ALUOp::UMulH => 0b10011011_110,
                };
                let top11 = top11 | size.sf_bit() << 10;
                let bit15_10 = match alu_op {
                    ALUOp::SDiv => 0b000011,
                    ALUOp::UDiv => 0b000010,
                    ALUOp::RotR => 0b001011,
                    ALUOp::Lsr => 0b001001,
                    ALUOp::Asr => 0b001010,
                    ALUOp::Lsl => 0b001000,
                    ALUOp::SMulH | ALUOp::UMulH => 0b011111,
                    _ => 0b000000,
                };
                debug_assert_ne!(writable_stack_reg(), rd);
                // The stack pointer is the zero register in this context, so this might be an
                // indication that something is wrong.
                debug_assert_ne!(stack_reg(), rn);
                debug_assert_ne!(stack_reg(), rm);
                sink.put4(enc_arith_rrr(top11, bit15_10, rd, rn, rm));
            }
            &Inst::AluRRRR {
                alu_op,
                rd,
                rm,
                rn,
                ra,
            } => {
                let (top11, bit15) = match alu_op {
                    ALUOp3::MAdd32 => (0b0_00_11011_000, 0),
                    ALUOp3::MSub32 => (0b0_00_11011_000, 1),
                    ALUOp3::MAdd64 => (0b1_00_11011_000, 0),
                    ALUOp3::MSub64 => (0b1_00_11011_000, 1),
                };
                sink.put4(enc_arith_rrrr(top11, rm, bit15, ra, rn, rd));
            }
            &Inst::AluRRImm12 {
                alu_op,
                size,
                rd,
                rn,
                ref imm12,
            } => {
                let top8 = match alu_op {
                    ALUOp::Add => 0b000_10001,
                    ALUOp::Sub => 0b010_10001,
                    ALUOp::AddS => 0b001_10001,
                    ALUOp::SubS => 0b011_10001,
                    _ => unimplemented!("{:?}", alu_op),
                };
                let top8 = top8 | size.sf_bit() << 7;
                sink.put4(enc_arith_rr_imm12(
                    top8,
                    imm12.shift_bits(),
                    imm12.imm_bits(),
                    rn,
                    rd,
                ));
            }
            &Inst::AluRRImmLogic {
                alu_op,
                size,
                rd,
                rn,
                ref imml,
            } => {
                let (top9, inv) = match alu_op {
                    ALUOp::Orr => (0b001_100100, false),
                    ALUOp::And => (0b000_100100, false),
                    ALUOp::AndS => (0b011_100100, false),
                    ALUOp::Eor => (0b010_100100, false),
                    ALUOp::OrrNot => (0b001_100100, true),
                    ALUOp::AndNot => (0b000_100100, true),
                    ALUOp::EorNot => (0b010_100100, true),
                    _ => unimplemented!("{:?}", alu_op),
                };
                let top9 = top9 | size.sf_bit() << 8;
                let imml = if inv { imml.invert() } else { imml.clone() };
                sink.put4(enc_arith_rr_imml(top9, imml.enc_bits(), rn, rd));
            }

            &Inst::AluRRImmShift {
                alu_op,
                size,
                rd,
                rn,
                ref immshift,
            } => {
                let amt = immshift.value();
                let (top10, immr, imms) = match alu_op {
                    ALUOp::RotR => (0b0001001110, machreg_to_gpr(rn), u32::from(amt)),
                    ALUOp::Lsr => (0b0101001100, u32::from(amt), 0b011111),
                    ALUOp::Asr => (0b0001001100, u32::from(amt), 0b011111),
                    ALUOp::Lsl => {
                        let bits = if size.is64() { 64 } else { 32 };
                        (
                            0b0101001100,
                            u32::from((bits - amt) % bits),
                            u32::from(bits - 1 - amt),
                        )
                    }
                    _ => unimplemented!("{:?}", alu_op),
                };
                let top10 = top10 | size.sf_bit() << 9 | size.sf_bit();
                let imms = match alu_op {
                    ALUOp::Lsr | ALUOp::Asr => imms | size.sf_bit() << 5,
                    _ => imms,
                };
                sink.put4(
                    (top10 << 22)
                        | (immr << 16)
                        | (imms << 10)
                        | (machreg_to_gpr(rn) << 5)
                        | machreg_to_gpr(rd.to_reg()),
                );
            }

            &Inst::AluRRRShift {
                alu_op,
                size,
                rd,
                rn,
                rm,
                ref shiftop,
            } => {
                let top11: u32 = match alu_op {
                    ALUOp::Add => 0b000_01011000,
                    ALUOp::AddS => 0b001_01011000,
                    ALUOp::Sub => 0b010_01011000,
                    ALUOp::SubS => 0b011_01011000,
                    ALUOp::Orr => 0b001_01010000,
                    ALUOp::And => 0b000_01010000,
                    ALUOp::AndS => 0b011_01010000,
                    ALUOp::Eor => 0b010_01010000,
                    ALUOp::OrrNot => 0b001_01010001,
                    ALUOp::EorNot => 0b010_01010001,
                    ALUOp::AndNot => 0b000_01010001,
                    _ => unimplemented!("{:?}", alu_op),
                };
                let top11 = top11 | size.sf_bit() << 10;
                let top11 = top11 | (u32::from(shiftop.op().bits()) << 1);
                let bits_15_10 = u32::from(shiftop.amt().value());
                sink.put4(enc_arith_rrr(top11, bits_15_10, rd, rn, rm));
            }

            &Inst::AluRRRExtend {
                alu_op,
                size,
                rd,
                rn,
                rm,
                extendop,
            } => {
                let top11: u32 = match alu_op {
                    ALUOp::Add => 0b00001011001,
                    ALUOp::Sub => 0b01001011001,
                    ALUOp::AddS => 0b00101011001,
                    ALUOp::SubS => 0b01101011001,
                    _ => unimplemented!("{:?}", alu_op),
                };
                let top11 = top11 | size.sf_bit() << 10;
                let bits_15_10 = u32::from(extendop.bits()) << 3;
                sink.put4(enc_arith_rrr(top11, bits_15_10, rd, rn, rm));
            }

            &Inst::BitRR {
                op, size, rd, rn, ..
            } => {
                let (op1, op2) = match op {
                    BitOp::RBit => (0b00000, 0b000000),
                    BitOp::Clz => (0b00000, 0b000100),
                    BitOp::Cls => (0b00000, 0b000101),
                };
                sink.put4(enc_bit_rr(size.sf_bit(), op1, op2, rn, rd))
            }

            &Inst::ULoad8 { rd, ref mem, flags }
            | &Inst::SLoad8 { rd, ref mem, flags }
            | &Inst::ULoad16 { rd, ref mem, flags }
            | &Inst::SLoad16 { rd, ref mem, flags }
            | &Inst::ULoad32 { rd, ref mem, flags }
            | &Inst::SLoad32 { rd, ref mem, flags }
            | &Inst::ULoad64 {
                rd, ref mem, flags, ..
            }
            | &Inst::FpuLoad32 { rd, ref mem, flags }
            | &Inst::FpuLoad64 { rd, ref mem, flags }
            | &Inst::FpuLoad128 { rd, ref mem, flags } => {
                let (mem_insts, mem) = mem_finalize(sink.cur_offset(), mem, state);

                for inst in mem_insts.into_iter() {
                    inst.emit(sink, emit_info, state);
                }

                // ldst encoding helpers take Reg, not Writable<Reg>.
                let rd = rd.to_reg();

                // This is the base opcode (top 10 bits) for the "unscaled
                // immediate" form (Unscaled). Other addressing modes will OR in
                // other values for bits 24/25 (bits 1/2 of this constant).
                let (op, bits) = match self {
                    &Inst::ULoad8 { .. } => (0b0011100001, 8),
                    &Inst::SLoad8 { .. } => (0b0011100010, 8),
                    &Inst::ULoad16 { .. } => (0b0111100001, 16),
                    &Inst::SLoad16 { .. } => (0b0111100010, 16),
                    &Inst::ULoad32 { .. } => (0b1011100001, 32),
                    &Inst::SLoad32 { .. } => (0b1011100010, 32),
                    &Inst::ULoad64 { .. } => (0b1111100001, 64),
                    &Inst::FpuLoad32 { .. } => (0b1011110001, 32),
                    &Inst::FpuLoad64 { .. } => (0b1111110001, 64),
                    &Inst::FpuLoad128 { .. } => (0b0011110011, 128),
                    _ => unreachable!(),
                };

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual load instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                match &mem {
                    &AMode::Unscaled(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b00, reg, rd));
                    }
                    &AMode::UnsignedOffset(reg, uimm12scaled) => {
                        if uimm12scaled.value() != 0 {
                            assert_eq!(bits, ty_bits(uimm12scaled.scale_ty()));
                        }
                        sink.put4(enc_ldst_uimm12(op, uimm12scaled, reg, rd));
                    }
                    &AMode::RegReg(r1, r2) => {
                        sink.put4(enc_ldst_reg(
                            op, r1, r2, /* scaled = */ false, /* extendop = */ None, rd,
                        ));
                    }
                    &AMode::RegScaled(r1, r2, ty) | &AMode::RegScaledExtended(r1, r2, ty, _) => {
                        assert_eq!(bits, ty_bits(ty));
                        let extendop = match &mem {
                            &AMode::RegScaled(..) => None,
                            &AMode::RegScaledExtended(_, _, _, op) => Some(op),
                            _ => unreachable!(),
                        };
                        sink.put4(enc_ldst_reg(
                            op, r1, r2, /* scaled = */ true, extendop, rd,
                        ));
                    }
                    &AMode::RegExtended(r1, r2, extendop) => {
                        sink.put4(enc_ldst_reg(
                            op,
                            r1,
                            r2,
                            /* scaled = */ false,
                            Some(extendop),
                            rd,
                        ));
                    }
                    &AMode::Label(ref label) => {
                        let offset = match label {
                            // cast i32 to u32 (two's-complement)
                            &MemLabel::PCRel(off) => off as u32,
                        } / 4;
                        assert!(offset < (1 << 19));
                        match self {
                            &Inst::ULoad32 { .. } => {
                                sink.put4(enc_ldst_imm19(0b00011000, offset, rd));
                            }
                            &Inst::SLoad32 { .. } => {
                                sink.put4(enc_ldst_imm19(0b10011000, offset, rd));
                            }
                            &Inst::FpuLoad32 { .. } => {
                                sink.put4(enc_ldst_imm19(0b00011100, offset, rd));
                            }
                            &Inst::ULoad64 { .. } => {
                                sink.put4(enc_ldst_imm19(0b01011000, offset, rd));
                            }
                            &Inst::FpuLoad64 { .. } => {
                                sink.put4(enc_ldst_imm19(0b01011100, offset, rd));
                            }
                            &Inst::FpuLoad128 { .. } => {
                                sink.put4(enc_ldst_imm19(0b10011100, offset, rd));
                            }
                            _ => panic!("Unspported size for LDR from constant pool!"),
                        }
                    }
                    &AMode::PreIndexed(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b11, reg.to_reg(), rd));
                    }
                    &AMode::PostIndexed(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b01, reg.to_reg(), rd));
                    }
                    // Eliminated by `mem_finalize()` above.
                    &AMode::SPOffset(..) | &AMode::FPOffset(..) | &AMode::NominalSPOffset(..) => {
                        panic!("Should not see stack-offset here!")
                    }
                    &AMode::RegOffset(..) => panic!("SHould not see generic reg-offset here!"),
                }
            }

            &Inst::Store8 { rd, ref mem, flags }
            | &Inst::Store16 { rd, ref mem, flags }
            | &Inst::Store32 { rd, ref mem, flags }
            | &Inst::Store64 { rd, ref mem, flags }
            | &Inst::FpuStore32 { rd, ref mem, flags }
            | &Inst::FpuStore64 { rd, ref mem, flags }
            | &Inst::FpuStore128 { rd, ref mem, flags } => {
                let (mem_insts, mem) = mem_finalize(sink.cur_offset(), mem, state);

                for inst in mem_insts.into_iter() {
                    inst.emit(sink, emit_info, state);
                }

                let (op, bits) = match self {
                    &Inst::Store8 { .. } => (0b0011100000, 8),
                    &Inst::Store16 { .. } => (0b0111100000, 16),
                    &Inst::Store32 { .. } => (0b1011100000, 32),
                    &Inst::Store64 { .. } => (0b1111100000, 64),
                    &Inst::FpuStore32 { .. } => (0b1011110000, 32),
                    &Inst::FpuStore64 { .. } => (0b1111110000, 64),
                    &Inst::FpuStore128 { .. } => (0b0011110010, 128),
                    _ => unreachable!(),
                };

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual store instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                match &mem {
                    &AMode::Unscaled(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b00, reg, rd));
                    }
                    &AMode::UnsignedOffset(reg, uimm12scaled) => {
                        if uimm12scaled.value() != 0 {
                            assert_eq!(bits, ty_bits(uimm12scaled.scale_ty()));
                        }
                        sink.put4(enc_ldst_uimm12(op, uimm12scaled, reg, rd));
                    }
                    &AMode::RegReg(r1, r2) => {
                        sink.put4(enc_ldst_reg(
                            op, r1, r2, /* scaled = */ false, /* extendop = */ None, rd,
                        ));
                    }
                    &AMode::RegScaled(r1, r2, _ty) | &AMode::RegScaledExtended(r1, r2, _ty, _) => {
                        let extendop = match &mem {
                            &AMode::RegScaled(..) => None,
                            &AMode::RegScaledExtended(_, _, _, op) => Some(op),
                            _ => unreachable!(),
                        };
                        sink.put4(enc_ldst_reg(
                            op, r1, r2, /* scaled = */ true, extendop, rd,
                        ));
                    }
                    &AMode::RegExtended(r1, r2, extendop) => {
                        sink.put4(enc_ldst_reg(
                            op,
                            r1,
                            r2,
                            /* scaled = */ false,
                            Some(extendop),
                            rd,
                        ));
                    }
                    &AMode::Label(..) => {
                        panic!("Store to a MemLabel not implemented!");
                    }
                    &AMode::PreIndexed(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b11, reg.to_reg(), rd));
                    }
                    &AMode::PostIndexed(reg, simm9) => {
                        sink.put4(enc_ldst_simm9(op, simm9, 0b01, reg.to_reg(), rd));
                    }
                    // Eliminated by `mem_finalize()` above.
                    &AMode::SPOffset(..) | &AMode::FPOffset(..) | &AMode::NominalSPOffset(..) => {
                        panic!("Should not see stack-offset here!")
                    }
                    &AMode::RegOffset(..) => panic!("SHould not see generic reg-offset here!"),
                }
            }

            &Inst::StoreP64 {
                rt,
                rt2,
                ref mem,
                flags,
            } => {
                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual store instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }
                match mem {
                    &PairAMode::SignedOffset(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100100, simm7, reg, rt, rt2));
                    }
                    &PairAMode::PreIndexed(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100110, simm7, reg.to_reg(), rt, rt2));
                    }
                    &PairAMode::PostIndexed(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100010, simm7, reg.to_reg(), rt, rt2));
                    }
                }
            }
            &Inst::LoadP64 {
                rt,
                rt2,
                ref mem,
                flags,
            } => {
                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual load instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                let rt = rt.to_reg();
                let rt2 = rt2.to_reg();
                match mem {
                    &PairAMode::SignedOffset(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100101, simm7, reg, rt, rt2));
                    }
                    &PairAMode::PreIndexed(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100111, simm7, reg.to_reg(), rt, rt2));
                    }
                    &PairAMode::PostIndexed(reg, simm7) => {
                        assert_eq!(simm7.scale_ty, I64);
                        sink.put4(enc_ldst_pair(0b1010100011, simm7, reg.to_reg(), rt, rt2));
                    }
                }
            }
            &Inst::FpuLoadP64 {
                rt,
                rt2,
                ref mem,
                flags,
            }
            | &Inst::FpuLoadP128 {
                rt,
                rt2,
                ref mem,
                flags,
            } => {
                let srcloc = state.cur_srcloc();

                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual load instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                let opc = match self {
                    &Inst::FpuLoadP64 { .. } => 0b01,
                    &Inst::FpuLoadP128 { .. } => 0b10,
                    _ => unreachable!(),
                };
                let rt = rt.to_reg();
                let rt2 = rt2.to_reg();

                match mem {
                    &PairAMode::SignedOffset(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(opc, 0b10, true, simm7, reg, rt, rt2));
                    }
                    &PairAMode::PreIndexed(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(
                            opc,
                            0b11,
                            true,
                            simm7,
                            reg.to_reg(),
                            rt,
                            rt2,
                        ));
                    }
                    &PairAMode::PostIndexed(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(
                            opc,
                            0b01,
                            true,
                            simm7,
                            reg.to_reg(),
                            rt,
                            rt2,
                        ));
                    }
                }
            }
            &Inst::FpuStoreP64 {
                rt,
                rt2,
                ref mem,
                flags,
            }
            | &Inst::FpuStoreP128 {
                rt,
                rt2,
                ref mem,
                flags,
            } => {
                let srcloc = state.cur_srcloc();

                if srcloc != SourceLoc::default() && !flags.notrap() {
                    // Register the offset at which the actual store instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                let opc = match self {
                    &Inst::FpuStoreP64 { .. } => 0b01,
                    &Inst::FpuStoreP128 { .. } => 0b10,
                    _ => unreachable!(),
                };

                match mem {
                    &PairAMode::SignedOffset(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(opc, 0b10, false, simm7, reg, rt, rt2));
                    }
                    &PairAMode::PreIndexed(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(
                            opc,
                            0b11,
                            false,
                            simm7,
                            reg.to_reg(),
                            rt,
                            rt2,
                        ));
                    }
                    &PairAMode::PostIndexed(reg, simm7) => {
                        assert!(simm7.scale_ty == F64 || simm7.scale_ty == I8X16);
                        sink.put4(enc_ldst_vec_pair(
                            opc,
                            0b01,
                            false,
                            simm7,
                            reg.to_reg(),
                            rt,
                            rt2,
                        ));
                    }
                }
            }
            &Inst::Mov64 { rd, rm } => {
                assert!(rd.to_reg().get_class() == rm.get_class());
                assert!(rm.get_class() == RegClass::I64);

                // MOV to SP is interpreted as MOV to XZR instead. And our codegen
                // should never MOV to XZR.
                assert!(rd.to_reg() != stack_reg());

                if rm == stack_reg() {
                    // We can't use ORR here, so use an `add rd, sp, #0` instead.
                    let imm12 = Imm12::maybe_from_u64(0).unwrap();
                    sink.put4(enc_arith_rr_imm12(
                        0b100_10001,
                        imm12.shift_bits(),
                        imm12.imm_bits(),
                        rm,
                        rd,
                    ));
                } else {
                    // Encoded as ORR rd, rm, zero.
                    sink.put4(enc_arith_rrr(0b10101010_000, 0b000_000, rd, zero_reg(), rm));
                }
            }
            &Inst::Mov32 { rd, rm } => {
                // MOV to SP is interpreted as MOV to XZR instead. And our codegen
                // should never MOV to XZR.
                assert!(machreg_to_gpr(rd.to_reg()) != 31);
                // Encoded as ORR rd, rm, zero.
                sink.put4(enc_arith_rrr(0b00101010_000, 0b000_000, rd, zero_reg(), rm));
            }
            &Inst::MovZ { rd, imm, size } => {
                sink.put4(enc_move_wide(MoveWideOpcode::MOVZ, rd, imm, size))
            }
            &Inst::MovN { rd, imm, size } => {
                sink.put4(enc_move_wide(MoveWideOpcode::MOVN, rd, imm, size))
            }
            &Inst::MovK { rd, imm, size } => {
                sink.put4(enc_move_wide(MoveWideOpcode::MOVK, rd, imm, size))
            }
            &Inst::CSel { rd, rn, rm, cond } => {
                sink.put4(enc_csel(rd, rn, rm, cond));
            }
            &Inst::CSet { rd, cond } => {
                sink.put4(enc_cset(rd, cond));
            }
            &Inst::CSetm { rd, cond } => {
                sink.put4(enc_csetm(rd, cond));
            }
            &Inst::CCmpImm {
                size,
                rn,
                imm,
                nzcv,
                cond,
            } => {
                sink.put4(enc_ccmp_imm(size, rn, imm, nzcv, cond));
            }
            &Inst::AtomicRMW { ty, op, rs, rt, rn } => {
                sink.put4(enc_ldal(ty, op, rs, rt, rn));
            }
            &Inst::AtomicRMWLoop { ty, op } => {
                /* Emit this:
                     again:
                      ldaxr{,b,h}  x/w27, [x25]
                      op          x28, x27, x26 // op is add,sub,and,orr,eor
                      stlxr{,b,h}  w24, x/w28, [x25]
                      cbnz        x24, again

                   Operand conventions:
                      IN:  x25 (addr), x26 (2nd arg for op)
                      OUT: x27 (old value), x24 (trashed), x28 (trashed)

                   It is unfortunate that, per the ARM documentation, x28 cannot be used for
                   both the store-data and success-flag operands of stlxr.  This causes the
                   instruction's behaviour to be "CONSTRAINED UNPREDICTABLE", so we use x24
                   instead for the success-flag.
                */
                // TODO: We should not hardcode registers here, a better idea would be to
                // pass some scratch registers in the AtomicRMWLoop pseudo-instruction, and use those
                let xzr = zero_reg();
                let x24 = xreg(24);
                let x25 = xreg(25);
                let x26 = xreg(26);
                let x27 = xreg(27);
                let x28 = xreg(28);
                let x24wr = writable_xreg(24);
                let x27wr = writable_xreg(27);
                let x28wr = writable_xreg(28);
                let again_label = sink.get_label();

                // again:
                sink.bind_label(again_label);
                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }
                sink.put4(enc_ldaxr(ty, x27wr, x25)); // ldaxr x27, [x25]
                let size = OperandSize::from_ty(ty);

                match op {
                    AtomicRmwOp::Xchg => {} // do nothing
                    AtomicRmwOp::Nand => {
                        // and x28, x27, x26
                        // mvn x28, x28

                        Inst::AluRRR {
                            alu_op: ALUOp::And,
                            size,
                            rd: x28wr,
                            rn: x27,
                            rm: x26,
                        }
                        .emit(sink, emit_info, state);

                        Inst::AluRRR {
                            alu_op: ALUOp::OrrNot,
                            size,
                            rd: x28wr,
                            rn: xzr,
                            rm: x28,
                        }
                        .emit(sink, emit_info, state);
                    }
                    AtomicRmwOp::Umin
                    | AtomicRmwOp::Umax
                    | AtomicRmwOp::Smin
                    | AtomicRmwOp::Smax => {
                        // cmp x27, x26
                        // csel.op x28, x27, x26

                        let cond = match op {
                            AtomicRmwOp::Umin => Cond::Lo,
                            AtomicRmwOp::Umax => Cond::Hi,
                            AtomicRmwOp::Smin => Cond::Lt,
                            AtomicRmwOp::Smax => Cond::Gt,
                            _ => unreachable!(),
                        };

                        Inst::AluRRR {
                            alu_op: ALUOp::SubS,
                            size,
                            rd: writable_zero_reg(),
                            rn: x27,
                            rm: x26,
                        }
                        .emit(sink, emit_info, state);

                        Inst::CSel {
                            cond,
                            rd: x28wr,
                            rn: x27,
                            rm: x26,
                        }
                        .emit(sink, emit_info, state);
                    }
                    _ => {
                        // add/sub/and/orr/eor x28, x27, x26
                        let alu_op = match op {
                            AtomicRmwOp::Add => ALUOp::Add,
                            AtomicRmwOp::Sub => ALUOp::Sub,
                            AtomicRmwOp::And => ALUOp::And,
                            AtomicRmwOp::Or => ALUOp::Orr,
                            AtomicRmwOp::Xor => ALUOp::Eor,
                            AtomicRmwOp::Nand
                            | AtomicRmwOp::Umin
                            | AtomicRmwOp::Umax
                            | AtomicRmwOp::Smin
                            | AtomicRmwOp::Smax
                            | AtomicRmwOp::Xchg => unreachable!(),
                        };

                        Inst::AluRRR {
                            alu_op,
                            size,
                            rd: x28wr,
                            rn: x27,
                            rm: x26,
                        }
                        .emit(sink, emit_info, state);
                    }
                }

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }
                if op == AtomicRmwOp::Xchg {
                    sink.put4(enc_stlxr(ty, x24wr, x26, x25)); // stlxr w24, x26, [x25]
                } else {
                    sink.put4(enc_stlxr(ty, x24wr, x28, x25)); // stlxr w24, x28, [x25]
                }

                // cbnz w24, again
                // Note, we're actually testing x24, and relying on the default zero-high-half
                // rule in the assignment that `stlxr` does.
                let br_offset = sink.cur_offset();
                sink.put4(enc_conditional_br(
                    BranchTarget::Label(again_label),
                    CondBrKind::NotZero(x24),
                ));
                sink.use_label_at_offset(br_offset, again_label, LabelUse::Branch19);
            }
            &Inst::AtomicCAS { rs, rt, rn, ty } => {
                let size = match ty {
                    I8 => 0b00,
                    I16 => 0b01,
                    I32 => 0b10,
                    I64 => 0b11,
                    _ => panic!("Unsupported type: {}", ty),
                };

                sink.put4(enc_cas(size, rs, rt, rn));
            }
            &Inst::AtomicCASLoop { ty } => {
                /* Emit this:
                    again:
                     ldaxr{,b,h} x/w27, [x25]
                     cmp         x27, x/w26 uxt{b,h}
                     b.ne        out
                     stlxr{,b,h} w24, x/w28, [x25]
                     cbnz        x24, again
                    out:

                  Operand conventions:
                     IN:  x25 (addr), x26 (expected value), x28 (replacement value)
                     OUT: x27 (old value), x24 (trashed)
                */
                let x24 = xreg(24);
                let x25 = xreg(25);
                let x26 = xreg(26);
                let x27 = xreg(27);
                let x28 = xreg(28);
                let xzrwr = writable_zero_reg();
                let x24wr = writable_xreg(24);
                let x27wr = writable_xreg(27);
                let again_label = sink.get_label();
                let out_label = sink.get_label();

                // again:
                sink.bind_label(again_label);
                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }
                // ldaxr x27, [x25]
                sink.put4(enc_ldaxr(ty, x27wr, x25));

                // The top 32-bits are zero-extended by the ldaxr so we don't
                // have to use UXTW, just the x-form of the register.
                let (bit21, extend_op) = match ty {
                    I8 => (0b1, 0b000000),
                    I16 => (0b1, 0b001000),
                    _ => (0b0, 0b000000),
                };
                let bits_31_21 = 0b111_01011_000 | bit21;
                // cmp x27, x26 (== subs xzr, x27, x26)
                sink.put4(enc_arith_rrr(bits_31_21, extend_op, xzrwr, x27, x26));

                // b.ne out
                let br_out_offset = sink.cur_offset();
                sink.put4(enc_conditional_br(
                    BranchTarget::Label(out_label),
                    CondBrKind::Cond(Cond::Ne),
                ));
                sink.use_label_at_offset(br_out_offset, out_label, LabelUse::Branch19);

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }
                sink.put4(enc_stlxr(ty, x24wr, x28, x25)); // stlxr w24, x28, [x25]

                // cbnz w24, again.
                // Note, we're actually testing x24, and relying on the default zero-high-half
                // rule in the assignment that `stlxr` does.
                let br_again_offset = sink.cur_offset();
                sink.put4(enc_conditional_br(
                    BranchTarget::Label(again_label),
                    CondBrKind::NotZero(x24),
                ));
                sink.use_label_at_offset(br_again_offset, again_label, LabelUse::Branch19);

                // out:
                sink.bind_label(out_label);
            }
            &Inst::LoadAcquire { access_ty, rt, rn } => {
                sink.put4(enc_ldar(access_ty, rt, rn));
            }
            &Inst::StoreRelease { access_ty, rt, rn } => {
                sink.put4(enc_stlr(access_ty, rt, rn));
            }
            &Inst::Fence {} => {
                sink.put4(enc_dmb_ish()); // dmb ish
            }
            &Inst::FpuMove64 { rd, rn } => {
                sink.put4(enc_fpurr(0b000_11110_01_1_000000_10000, rd, rn));
            }
            &Inst::FpuMove128 { rd, rn } => {
                sink.put4(enc_vecmov(/* 16b = */ true, rd, rn));
            }
            &Inst::FpuMoveFromVec { rd, rn, idx, size } => {
                let (imm5, shift, mask) = match size.lane_size() {
                    ScalarSize::Size32 => (0b00100, 3, 0b011),
                    ScalarSize::Size64 => (0b01000, 4, 0b001),
                    _ => unimplemented!(),
                };
                debug_assert_eq!(idx & mask, idx);
                let imm5 = imm5 | ((idx as u32) << shift);
                sink.put4(
                    0b010_11110000_00000_000001_00000_00000
                        | (imm5 << 16)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::FpuExtend { rd, rn, size } => {
                sink.put4(enc_fpurr(
                    0b000_11110_00_1_000000_10000 | (size.ftype() << 13),
                    rd,
                    rn,
                ));
            }
            &Inst::FpuRR { fpu_op, rd, rn } => {
                let top22 = match fpu_op {
                    FPUOp1::Abs32 => 0b000_11110_00_1_000001_10000,
                    FPUOp1::Abs64 => 0b000_11110_01_1_000001_10000,
                    FPUOp1::Neg32 => 0b000_11110_00_1_000010_10000,
                    FPUOp1::Neg64 => 0b000_11110_01_1_000010_10000,
                    FPUOp1::Sqrt32 => 0b000_11110_00_1_000011_10000,
                    FPUOp1::Sqrt64 => 0b000_11110_01_1_000011_10000,
                    FPUOp1::Cvt32To64 => 0b000_11110_00_1_000101_10000,
                    FPUOp1::Cvt64To32 => 0b000_11110_01_1_000100_10000,
                };
                sink.put4(enc_fpurr(top22, rd, rn));
            }
            &Inst::FpuRRR { fpu_op, rd, rn, rm } => {
                let top22 = match fpu_op {
                    FPUOp2::Add32 => 0b000_11110_00_1_00000_001010,
                    FPUOp2::Add64 => 0b000_11110_01_1_00000_001010,
                    FPUOp2::Sub32 => 0b000_11110_00_1_00000_001110,
                    FPUOp2::Sub64 => 0b000_11110_01_1_00000_001110,
                    FPUOp2::Mul32 => 0b000_11110_00_1_00000_000010,
                    FPUOp2::Mul64 => 0b000_11110_01_1_00000_000010,
                    FPUOp2::Div32 => 0b000_11110_00_1_00000_000110,
                    FPUOp2::Div64 => 0b000_11110_01_1_00000_000110,
                    FPUOp2::Max32 => 0b000_11110_00_1_00000_010010,
                    FPUOp2::Max64 => 0b000_11110_01_1_00000_010010,
                    FPUOp2::Min32 => 0b000_11110_00_1_00000_010110,
                    FPUOp2::Min64 => 0b000_11110_01_1_00000_010110,
                    FPUOp2::Sqadd64 => 0b010_11110_11_1_00000_000011,
                    FPUOp2::Uqadd64 => 0b011_11110_11_1_00000_000011,
                    FPUOp2::Sqsub64 => 0b010_11110_11_1_00000_001011,
                    FPUOp2::Uqsub64 => 0b011_11110_11_1_00000_001011,
                };
                sink.put4(enc_fpurrr(top22, rd, rn, rm));
            }
            &Inst::FpuRRI { fpu_op, rd, rn } => match fpu_op {
                FPUOpRI::UShr32(imm) => {
                    debug_assert_eq!(32, imm.lane_size_in_bits);
                    sink.put4(
                        0b0_0_1_011110_0000000_00_0_0_0_1_00000_00000
                            | imm.enc() << 16
                            | machreg_to_vec(rn) << 5
                            | machreg_to_vec(rd.to_reg()),
                    )
                }
                FPUOpRI::UShr64(imm) => {
                    debug_assert_eq!(64, imm.lane_size_in_bits);
                    sink.put4(
                        0b01_1_111110_0000000_00_0_0_0_1_00000_00000
                            | imm.enc() << 16
                            | machreg_to_vec(rn) << 5
                            | machreg_to_vec(rd.to_reg()),
                    )
                }
                FPUOpRI::Sli64(imm) => {
                    debug_assert_eq!(64, imm.lane_size_in_bits);
                    sink.put4(
                        0b01_1_111110_0000000_010101_00000_00000
                            | imm.enc() << 16
                            | machreg_to_vec(rn) << 5
                            | machreg_to_vec(rd.to_reg()),
                    )
                }
                FPUOpRI::Sli32(imm) => {
                    debug_assert_eq!(32, imm.lane_size_in_bits);
                    sink.put4(
                        0b0_0_1_011110_0000000_010101_00000_00000
                            | imm.enc() << 16
                            | machreg_to_vec(rn) << 5
                            | machreg_to_vec(rd.to_reg()),
                    )
                }
            },
            &Inst::FpuRRRR {
                fpu_op,
                rd,
                rn,
                rm,
                ra,
            } => {
                let top17 = match fpu_op {
                    FPUOp3::MAdd32 => 0b000_11111_00_0_00000_0,
                    FPUOp3::MAdd64 => 0b000_11111_01_0_00000_0,
                };
                sink.put4(enc_fpurrrr(top17, rd, rn, rm, ra));
            }
            &Inst::VecMisc { op, rd, rn, size } => {
                let (q, enc_size) = size.enc_size();
                let (u, bits_12_16, size) = match op {
                    VecMisc2::Not => (0b1, 0b00101, 0b00),
                    VecMisc2::Neg => (0b1, 0b01011, enc_size),
                    VecMisc2::Abs => (0b0, 0b01011, enc_size),
                    VecMisc2::Fabs => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b01111, enc_size)
                    }
                    VecMisc2::Fneg => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b1, 0b01111, enc_size)
                    }
                    VecMisc2::Fsqrt => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b1, 0b11111, enc_size)
                    }
                    VecMisc2::Rev64 => {
                        debug_assert_ne!(VectorSize::Size64x2, size);
                        (0b0, 0b00000, enc_size)
                    }
                    VecMisc2::Fcvtzs => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b11011, enc_size)
                    }
                    VecMisc2::Fcvtzu => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b1, 0b11011, enc_size)
                    }
                    VecMisc2::Scvtf => {
                        debug_assert!(size == VectorSize::Size32x4 || size == VectorSize::Size64x2);
                        (0b0, 0b11101, enc_size & 0b1)
                    }
                    VecMisc2::Ucvtf => {
                        debug_assert!(size == VectorSize::Size32x4 || size == VectorSize::Size64x2);
                        (0b1, 0b11101, enc_size & 0b1)
                    }
                    VecMisc2::Frintn => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b11000, enc_size & 0b01)
                    }
                    VecMisc2::Frintz => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b11001, enc_size)
                    }
                    VecMisc2::Frintm => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b11001, enc_size & 0b01)
                    }
                    VecMisc2::Frintp => {
                        debug_assert!(
                            size == VectorSize::Size32x2
                                || size == VectorSize::Size32x4
                                || size == VectorSize::Size64x2
                        );
                        (0b0, 0b11000, enc_size)
                    }
                    VecMisc2::Cnt => {
                        debug_assert!(size == VectorSize::Size8x8 || size == VectorSize::Size8x16);
                        (0b0, 0b00101, enc_size)
                    }
                    VecMisc2::Cmeq0 => (0b0, 0b01001, enc_size),
                };
                sink.put4(enc_vec_rr_misc((q << 1) | u, size, bits_12_16, rd, rn));
            }
            &Inst::VecLanes { op, rd, rn, size } => {
                let (q, size) = match size {
                    VectorSize::Size8x8 => (0b0, 0b00),
                    VectorSize::Size8x16 => (0b1, 0b00),
                    VectorSize::Size16x4 => (0b0, 0b01),
                    VectorSize::Size16x8 => (0b1, 0b01),
                    VectorSize::Size32x4 => (0b1, 0b10),
                    _ => unreachable!(),
                };
                let (u, opcode) = match op {
                    VecLanesOp::Uminv => (0b1, 0b11010),
                    VecLanesOp::Addv => (0b0, 0b11011),
                };
                sink.put4(enc_vec_lanes(q, u, size, opcode, rd, rn));
            }
            &Inst::VecShiftImm {
                op,
                rd,
                rn,
                size,
                imm,
            } => {
                let (is_shr, template) = match op {
                    VecShiftImmOp::Ushr => (true, 0b_011_011110_0000_000_000001_00000_00000_u32),
                    VecShiftImmOp::Sshr => (true, 0b_010_011110_0000_000_000001_00000_00000_u32),
                    VecShiftImmOp::Shl => (false, 0b_010_011110_0000_000_010101_00000_00000_u32),
                };
                let imm = imm as u32;
                // Deal with the somewhat strange encoding scheme for, and limits on,
                // the shift amount.
                let immh_immb = match (size, is_shr) {
                    (VectorSize::Size64x2, true) if imm >= 1 && imm <= 64 => {
                        0b_1000_000_u32 | (64 - imm)
                    }
                    (VectorSize::Size32x4, true) if imm >= 1 && imm <= 32 => {
                        0b_0100_000_u32 | (32 - imm)
                    }
                    (VectorSize::Size16x8, true) if imm >= 1 && imm <= 16 => {
                        0b_0010_000_u32 | (16 - imm)
                    }
                    (VectorSize::Size8x16, true) if imm >= 1 && imm <= 8 => {
                        0b_0001_000_u32 | (8 - imm)
                    }
                    (VectorSize::Size64x2, false) if imm <= 63 => 0b_1000_000_u32 | imm,
                    (VectorSize::Size32x4, false) if imm <= 31 => 0b_0100_000_u32 | imm,
                    (VectorSize::Size16x8, false) if imm <= 15 => 0b_0010_000_u32 | imm,
                    (VectorSize::Size8x16, false) if imm <= 7 => 0b_0001_000_u32 | imm,
                    _ => panic!(
                        "aarch64: Inst::VecShiftImm: emit: invalid op/size/imm {:?}, {:?}, {:?}",
                        op, size, imm
                    ),
                };
                let rn_enc = machreg_to_vec(rn);
                let rd_enc = machreg_to_vec(rd.to_reg());
                sink.put4(template | (immh_immb << 16) | (rn_enc << 5) | rd_enc);
            }
            &Inst::VecExtract { rd, rn, rm, imm4 } => {
                if imm4 < 16 {
                    let template = 0b_01_101110_000_00000_0_0000_0_00000_00000_u32;
                    let rm_enc = machreg_to_vec(rm);
                    let rn_enc = machreg_to_vec(rn);
                    let rd_enc = machreg_to_vec(rd.to_reg());
                    sink.put4(
                        template | (rm_enc << 16) | ((imm4 as u32) << 11) | (rn_enc << 5) | rd_enc,
                    );
                } else {
                    panic!(
                        "aarch64: Inst::VecExtract: emit: invalid extract index {}",
                        imm4
                    );
                }
            }
            &Inst::VecTbl {
                rd,
                rn,
                rm,
                is_extension,
            } => {
                sink.put4(enc_tbl(is_extension, 0b00, rd, rn, rm));
            }
            &Inst::VecTbl2 {
                rd,
                rn,
                rn2,
                rm,
                is_extension,
            } => {
                assert_eq!(machreg_to_vec(rn2), (machreg_to_vec(rn) + 1) % 32);
                sink.put4(enc_tbl(is_extension, 0b01, rd, rn, rm));
            }
            &Inst::FpuCmp32 { rn, rm } => {
                sink.put4(enc_fcmp(ScalarSize::Size32, rn, rm));
            }
            &Inst::FpuCmp64 { rn, rm } => {
                sink.put4(enc_fcmp(ScalarSize::Size64, rn, rm));
            }
            &Inst::FpuToInt { op, rd, rn } => {
                let top16 = match op {
                    // FCVTZS (32/32-bit)
                    FpuToIntOp::F32ToI32 => 0b000_11110_00_1_11_000,
                    // FCVTZU (32/32-bit)
                    FpuToIntOp::F32ToU32 => 0b000_11110_00_1_11_001,
                    // FCVTZS (32/64-bit)
                    FpuToIntOp::F32ToI64 => 0b100_11110_00_1_11_000,
                    // FCVTZU (32/64-bit)
                    FpuToIntOp::F32ToU64 => 0b100_11110_00_1_11_001,
                    // FCVTZS (64/32-bit)
                    FpuToIntOp::F64ToI32 => 0b000_11110_01_1_11_000,
                    // FCVTZU (64/32-bit)
                    FpuToIntOp::F64ToU32 => 0b000_11110_01_1_11_001,
                    // FCVTZS (64/64-bit)
                    FpuToIntOp::F64ToI64 => 0b100_11110_01_1_11_000,
                    // FCVTZU (64/64-bit)
                    FpuToIntOp::F64ToU64 => 0b100_11110_01_1_11_001,
                };
                sink.put4(enc_fputoint(top16, rd, rn));
            }
            &Inst::IntToFpu { op, rd, rn } => {
                let top16 = match op {
                    // SCVTF (32/32-bit)
                    IntToFpuOp::I32ToF32 => 0b000_11110_00_1_00_010,
                    // UCVTF (32/32-bit)
                    IntToFpuOp::U32ToF32 => 0b000_11110_00_1_00_011,
                    // SCVTF (64/32-bit)
                    IntToFpuOp::I64ToF32 => 0b100_11110_00_1_00_010,
                    // UCVTF (64/32-bit)
                    IntToFpuOp::U64ToF32 => 0b100_11110_00_1_00_011,
                    // SCVTF (32/64-bit)
                    IntToFpuOp::I32ToF64 => 0b000_11110_01_1_00_010,
                    // UCVTF (32/64-bit)
                    IntToFpuOp::U32ToF64 => 0b000_11110_01_1_00_011,
                    // SCVTF (64/64-bit)
                    IntToFpuOp::I64ToF64 => 0b100_11110_01_1_00_010,
                    // UCVTF (64/64-bit)
                    IntToFpuOp::U64ToF64 => 0b100_11110_01_1_00_011,
                };
                sink.put4(enc_inttofpu(top16, rd, rn));
            }
            &Inst::LoadFpuConst64 { rd, const_data } => {
                let inst = Inst::FpuLoad64 {
                    rd,
                    mem: AMode::Label(MemLabel::PCRel(8)),
                    flags: MemFlags::trusted(),
                };
                inst.emit(sink, emit_info, state);
                let inst = Inst::Jump {
                    dest: BranchTarget::ResolvedOffset(12),
                };
                inst.emit(sink, emit_info, state);
                sink.put8(const_data);
            }
            &Inst::LoadFpuConst128 { rd, const_data } => {
                let inst = Inst::FpuLoad128 {
                    rd,
                    mem: AMode::Label(MemLabel::PCRel(8)),
                    flags: MemFlags::trusted(),
                };
                inst.emit(sink, emit_info, state);
                let inst = Inst::Jump {
                    dest: BranchTarget::ResolvedOffset(20),
                };
                inst.emit(sink, emit_info, state);

                for i in const_data.to_le_bytes().iter() {
                    sink.put1(*i);
                }
            }
            &Inst::FpuCSel32 { rd, rn, rm, cond } => {
                sink.put4(enc_fcsel(rd, rn, rm, cond, ScalarSize::Size32));
            }
            &Inst::FpuCSel64 { rd, rn, rm, cond } => {
                sink.put4(enc_fcsel(rd, rn, rm, cond, ScalarSize::Size64));
            }
            &Inst::FpuRound { op, rd, rn } => {
                let top22 = match op {
                    FpuRoundMode::Minus32 => 0b000_11110_00_1_001_010_10000,
                    FpuRoundMode::Minus64 => 0b000_11110_01_1_001_010_10000,
                    FpuRoundMode::Plus32 => 0b000_11110_00_1_001_001_10000,
                    FpuRoundMode::Plus64 => 0b000_11110_01_1_001_001_10000,
                    FpuRoundMode::Zero32 => 0b000_11110_00_1_001_011_10000,
                    FpuRoundMode::Zero64 => 0b000_11110_01_1_001_011_10000,
                    FpuRoundMode::Nearest32 => 0b000_11110_00_1_001_000_10000,
                    FpuRoundMode::Nearest64 => 0b000_11110_01_1_001_000_10000,
                };
                sink.put4(enc_fround(top22, rd, rn));
            }
            &Inst::MovToFpu { rd, rn, size } => {
                let template = match size {
                    ScalarSize::Size32 => 0b000_11110_00_1_00_111_000000_00000_00000,
                    ScalarSize::Size64 => 0b100_11110_01_1_00_111_000000_00000_00000,
                    _ => unreachable!(),
                };
                sink.put4(template | (machreg_to_gpr(rn) << 5) | machreg_to_vec(rd.to_reg()));
            }
            &Inst::FpuMoveFPImm { rd, imm, size } => {
                let size_code = match size {
                    ScalarSize::Size32 => 0b00,
                    ScalarSize::Size64 => 0b01,
                    _ => unimplemented!(),
                };
                sink.put4(
                    0b000_11110_00_1_00_000_000100_00000_00000
                        | size_code << 22
                        | ((imm.enc_bits() as u32) << 13)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::MovToVec { rd, rn, idx, size } => {
                let (imm5, shift) = match size.lane_size() {
                    ScalarSize::Size8 => (0b00001, 1),
                    ScalarSize::Size16 => (0b00010, 2),
                    ScalarSize::Size32 => (0b00100, 3),
                    ScalarSize::Size64 => (0b01000, 4),
                    _ => unreachable!(),
                };
                debug_assert_eq!(idx & (0b11111 >> shift), idx);
                let imm5 = imm5 | ((idx as u32) << shift);
                sink.put4(
                    0b010_01110000_00000_0_0011_1_00000_00000
                        | (imm5 << 16)
                        | (machreg_to_gpr(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::MovFromVec { rd, rn, idx, size } => {
                let (q, imm5, shift, mask) = match size {
                    VectorSize::Size8x16 => (0b0, 0b00001, 1, 0b1111),
                    VectorSize::Size16x8 => (0b0, 0b00010, 2, 0b0111),
                    VectorSize::Size32x4 => (0b0, 0b00100, 3, 0b0011),
                    VectorSize::Size64x2 => (0b1, 0b01000, 4, 0b0001),
                    _ => unreachable!(),
                };
                debug_assert_eq!(idx & mask, idx);
                let imm5 = imm5 | ((idx as u32) << shift);
                sink.put4(
                    0b000_01110000_00000_0_0111_1_00000_00000
                        | (q << 30)
                        | (imm5 << 16)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_gpr(rd.to_reg()),
                );
            }
            &Inst::MovFromVecSigned {
                rd,
                rn,
                idx,
                size,
                scalar_size,
            } => {
                let (imm5, shift, half) = match size {
                    VectorSize::Size8x8 => (0b00001, 1, true),
                    VectorSize::Size8x16 => (0b00001, 1, false),
                    VectorSize::Size16x4 => (0b00010, 2, true),
                    VectorSize::Size16x8 => (0b00010, 2, false),
                    VectorSize::Size32x2 => {
                        debug_assert_ne!(scalar_size, OperandSize::Size32);
                        (0b00100, 3, true)
                    }
                    VectorSize::Size32x4 => {
                        debug_assert_ne!(scalar_size, OperandSize::Size32);
                        (0b00100, 3, false)
                    }
                    _ => panic!("Unexpected vector operand size"),
                };
                debug_assert_eq!(idx & (0b11111 >> (half as u32 + shift)), idx);
                let imm5 = imm5 | ((idx as u32) << shift);
                sink.put4(
                    0b000_01110000_00000_0_0101_1_00000_00000
                        | (scalar_size.is64() as u32) << 30
                        | (imm5 << 16)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_gpr(rd.to_reg()),
                );
            }
            &Inst::VecDup { rd, rn, size } => {
                let imm5 = match size {
                    VectorSize::Size8x16 => 0b00001,
                    VectorSize::Size16x8 => 0b00010,
                    VectorSize::Size32x4 => 0b00100,
                    VectorSize::Size64x2 => 0b01000,
                    _ => unimplemented!(),
                };
                sink.put4(
                    0b010_01110000_00000_000011_00000_00000
                        | (imm5 << 16)
                        | (machreg_to_gpr(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::VecDupFromFpu { rd, rn, size } => {
                let imm5 = match size {
                    VectorSize::Size32x4 => 0b00100,
                    VectorSize::Size64x2 => 0b01000,
                    _ => unimplemented!(),
                };
                sink.put4(
                    0b010_01110000_00000_000001_00000_00000
                        | (imm5 << 16)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::VecDupFPImm { rd, imm, size } => {
                let imm = imm.enc_bits();
                let op = match size.lane_size() {
                    ScalarSize::Size32 => 0,
                    ScalarSize::Size64 => 1,
                    _ => unimplemented!(),
                };
                let q_op = op | ((size.is_128bits() as u32) << 1);

                sink.put4(enc_asimd_mod_imm(rd, q_op, 0b1111, imm));
            }
            &Inst::VecDupImm {
                rd,
                imm,
                invert,
                size,
            } => {
                let (imm, shift, shift_ones) = imm.value();
                let (op, cmode) = match size.lane_size() {
                    ScalarSize::Size8 => {
                        assert!(!invert);
                        assert_eq!(shift, 0);

                        (0, 0b1110)
                    }
                    ScalarSize::Size16 => {
                        let s = shift & 8;

                        assert!(!shift_ones);
                        assert_eq!(s, shift);

                        (invert as u32, 0b1000 | (s >> 2))
                    }
                    ScalarSize::Size32 => {
                        if shift_ones {
                            assert!(shift == 8 || shift == 16);

                            (invert as u32, 0b1100 | (shift >> 4))
                        } else {
                            let s = shift & 24;

                            assert_eq!(s, shift);

                            (invert as u32, 0b0000 | (s >> 2))
                        }
                    }
                    ScalarSize::Size64 => {
                        assert!(!invert);
                        assert_eq!(shift, 0);

                        (1, 0b1110)
                    }
                    _ => unreachable!(),
                };
                let q_op = op | ((size.is_128bits() as u32) << 1);

                sink.put4(enc_asimd_mod_imm(rd, q_op, cmode, imm));
            }
            &Inst::VecExtend {
                t,
                rd,
                rn,
                high_half,
            } => {
                let (u, immh) = match t {
                    VecExtendOp::Sxtl8 => (0b0, 0b001),
                    VecExtendOp::Sxtl16 => (0b0, 0b010),
                    VecExtendOp::Sxtl32 => (0b0, 0b100),
                    VecExtendOp::Uxtl8 => (0b1, 0b001),
                    VecExtendOp::Uxtl16 => (0b1, 0b010),
                    VecExtendOp::Uxtl32 => (0b1, 0b100),
                };
                sink.put4(
                    0b000_011110_0000_000_101001_00000_00000
                        | ((high_half as u32) << 30)
                        | (u << 29)
                        | (immh << 19)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::VecRRLong {
                op,
                rd,
                rn,
                high_half,
            } => {
                let (u, size, bits_12_16) = match op {
                    VecRRLongOp::Fcvtl16 => (0b0, 0b00, 0b10111),
                    VecRRLongOp::Fcvtl32 => (0b0, 0b01, 0b10111),
                    VecRRLongOp::Shll8 => (0b1, 0b00, 0b10011),
                    VecRRLongOp::Shll16 => (0b1, 0b01, 0b10011),
                    VecRRLongOp::Shll32 => (0b1, 0b10, 0b10011),
                };

                sink.put4(enc_vec_rr_misc(
                    ((high_half as u32) << 1) | u,
                    size,
                    bits_12_16,
                    rd,
                    rn,
                ));
            }
            &Inst::VecRRNarrow {
                op,
                rd,
                rn,
                high_half,
            } => {
                let (u, size, bits_12_16) = match op {
                    VecRRNarrowOp::Xtn16 => (0b0, 0b00, 0b10010),
                    VecRRNarrowOp::Xtn32 => (0b0, 0b01, 0b10010),
                    VecRRNarrowOp::Xtn64 => (0b0, 0b10, 0b10010),
                    VecRRNarrowOp::Sqxtn16 => (0b0, 0b00, 0b10100),
                    VecRRNarrowOp::Sqxtn32 => (0b0, 0b01, 0b10100),
                    VecRRNarrowOp::Sqxtn64 => (0b0, 0b10, 0b10100),
                    VecRRNarrowOp::Sqxtun16 => (0b1, 0b00, 0b10010),
                    VecRRNarrowOp::Sqxtun32 => (0b1, 0b01, 0b10010),
                    VecRRNarrowOp::Sqxtun64 => (0b1, 0b10, 0b10010),
                    VecRRNarrowOp::Uqxtn16 => (0b1, 0b00, 0b10100),
                    VecRRNarrowOp::Uqxtn32 => (0b1, 0b01, 0b10100),
                    VecRRNarrowOp::Uqxtn64 => (0b1, 0b10, 0b10100),
                    VecRRNarrowOp::Fcvtn32 => (0b0, 0b00, 0b10110),
                    VecRRNarrowOp::Fcvtn64 => (0b0, 0b01, 0b10110),
                };

                sink.put4(enc_vec_rr_misc(
                    ((high_half as u32) << 1) | u,
                    size,
                    bits_12_16,
                    rd,
                    rn,
                ));
            }
            &Inst::VecMovElement {
                rd,
                rn,
                dest_idx,
                src_idx,
                size,
            } => {
                let (imm5, shift) = match size.lane_size() {
                    ScalarSize::Size8 => (0b00001, 1),
                    ScalarSize::Size16 => (0b00010, 2),
                    ScalarSize::Size32 => (0b00100, 3),
                    ScalarSize::Size64 => (0b01000, 4),
                    _ => unreachable!(),
                };
                let mask = 0b11111 >> shift;
                debug_assert_eq!(dest_idx & mask, dest_idx);
                debug_assert_eq!(src_idx & mask, src_idx);
                let imm4 = (src_idx as u32) << (shift - 1);
                let imm5 = imm5 | ((dest_idx as u32) << shift);
                sink.put4(
                    0b011_01110000_00000_0_0000_1_00000_00000
                        | (imm5 << 16)
                        | (imm4 << 11)
                        | (machreg_to_vec(rn) << 5)
                        | machreg_to_vec(rd.to_reg()),
                );
            }
            &Inst::VecRRPair { op, rd, rn } => {
                let bits_12_16 = match op {
                    VecPairOp::Addp => 0b11011,
                };

                sink.put4(enc_vec_rr_pair(bits_12_16, rd, rn));
            }
            &Inst::VecRRRLong {
                rd,
                rn,
                rm,
                alu_op,
                high_half,
            } => {
                let (u, size, bit14) = match alu_op {
                    VecRRRLongOp::Smull8 => (0b0, 0b00, 0b1),
                    VecRRRLongOp::Smull16 => (0b0, 0b01, 0b1),
                    VecRRRLongOp::Smull32 => (0b0, 0b10, 0b1),
                    VecRRRLongOp::Umull8 => (0b1, 0b00, 0b1),
                    VecRRRLongOp::Umull16 => (0b1, 0b01, 0b1),
                    VecRRRLongOp::Umull32 => (0b1, 0b10, 0b1),
                    VecRRRLongOp::Umlal8 => (0b1, 0b00, 0b0),
                    VecRRRLongOp::Umlal16 => (0b1, 0b01, 0b0),
                    VecRRRLongOp::Umlal32 => (0b1, 0b10, 0b0),
                };
                sink.put4(enc_vec_rrr_long(
                    high_half as u32,
                    u,
                    size,
                    bit14,
                    rm,
                    rn,
                    rd,
                ));
            }
            &Inst::VecRRPairLong { op, rd, rn } => {
                let (u, size) = match op {
                    VecRRPairLongOp::Saddlp8 => (0b0, 0b0),
                    VecRRPairLongOp::Uaddlp8 => (0b1, 0b0),
                    VecRRPairLongOp::Saddlp16 => (0b0, 0b1),
                    VecRRPairLongOp::Uaddlp16 => (0b1, 0b1),
                };

                sink.put4(enc_vec_rr_pair_long(u, size, rd, rn));
            }
            &Inst::VecRRR {
                rd,
                rn,
                rm,
                alu_op,
                size,
            } => {
                let (q, enc_size) = size.enc_size();
                let is_float = match alu_op {
                    VecALUOp::Fcmeq
                    | VecALUOp::Fcmgt
                    | VecALUOp::Fcmge
                    | VecALUOp::Fadd
                    | VecALUOp::Fsub
                    | VecALUOp::Fdiv
                    | VecALUOp::Fmax
                    | VecALUOp::Fmin
                    | VecALUOp::Fmul => true,
                    _ => false,
                };
                let enc_float_size = match (is_float, size) {
                    (true, VectorSize::Size32x2) => 0b0,
                    (true, VectorSize::Size32x4) => 0b0,
                    (true, VectorSize::Size64x2) => 0b1,
                    (true, _) => unimplemented!(),
                    _ => 0,
                };

                let (top11, bit15_10) = match alu_op {
                    VecALUOp::Sqadd => (0b000_01110_00_1 | enc_size << 1, 0b000011),
                    VecALUOp::Sqsub => (0b000_01110_00_1 | enc_size << 1, 0b001011),
                    VecALUOp::Uqadd => (0b001_01110_00_1 | enc_size << 1, 0b000011),
                    VecALUOp::Uqsub => (0b001_01110_00_1 | enc_size << 1, 0b001011),
                    VecALUOp::Cmeq => (0b001_01110_00_1 | enc_size << 1, 0b100011),
                    VecALUOp::Cmge => (0b000_01110_00_1 | enc_size << 1, 0b001111),
                    VecALUOp::Cmgt => (0b000_01110_00_1 | enc_size << 1, 0b001101),
                    VecALUOp::Cmhi => (0b001_01110_00_1 | enc_size << 1, 0b001101),
                    VecALUOp::Cmhs => (0b001_01110_00_1 | enc_size << 1, 0b001111),
                    VecALUOp::Fcmeq => (0b000_01110_00_1, 0b111001),
                    VecALUOp::Fcmgt => (0b001_01110_10_1, 0b111001),
                    VecALUOp::Fcmge => (0b001_01110_00_1, 0b111001),
                    // The following logical instructions operate on bytes, so are not encoded differently
                    // for the different vector types.
                    VecALUOp::And => (0b000_01110_00_1, 0b000111),
                    VecALUOp::Bic => (0b000_01110_01_1, 0b000111),
                    VecALUOp::Orr => (0b000_01110_10_1, 0b000111),
                    VecALUOp::Eor => (0b001_01110_00_1, 0b000111),
                    VecALUOp::Bsl => (0b001_01110_01_1, 0b000111),
                    VecALUOp::Umaxp => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b001_01110_00_1 | enc_size << 1, 0b101001)
                    }
                    VecALUOp::Add => (0b000_01110_00_1 | enc_size << 1, 0b100001),
                    VecALUOp::Sub => (0b001_01110_00_1 | enc_size << 1, 0b100001),
                    VecALUOp::Mul => {
                        debug_assert_ne!(size, VectorSize::Size64x2);
                        (0b000_01110_00_1 | enc_size << 1, 0b100111)
                    }
                    VecALUOp::Sshl => (0b000_01110_00_1 | enc_size << 1, 0b010001),
                    VecALUOp::Ushl => (0b001_01110_00_1 | enc_size << 1, 0b010001),
                    VecALUOp::Umin => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b001_01110_00_1 | enc_size << 1, 0b011011)
                    }
                    VecALUOp::Smin => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b000_01110_00_1 | enc_size << 1, 0b011011)
                    }
                    VecALUOp::Umax => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b001_01110_00_1 | enc_size << 1, 0b011001)
                    }
                    VecALUOp::Smax => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b000_01110_00_1 | enc_size << 1, 0b011001)
                    }
                    VecALUOp::Urhadd => {
                        debug_assert_ne!(size, VectorSize::Size64x2);

                        (0b001_01110_00_1 | enc_size << 1, 0b000101)
                    }
                    VecALUOp::Fadd => (0b000_01110_00_1, 0b110101),
                    VecALUOp::Fsub => (0b000_01110_10_1, 0b110101),
                    VecALUOp::Fdiv => (0b001_01110_00_1, 0b111111),
                    VecALUOp::Fmax => (0b000_01110_00_1, 0b111101),
                    VecALUOp::Fmin => (0b000_01110_10_1, 0b111101),
                    VecALUOp::Fmul => (0b001_01110_00_1, 0b110111),
                    VecALUOp::Addp => (0b000_01110_00_1 | enc_size << 1, 0b101111),
                    VecALUOp::Zip1 => (0b01001110_00_0 | enc_size << 1, 0b001110),
                    VecALUOp::Sqrdmulh => {
                        debug_assert!(
                            size.lane_size() == ScalarSize::Size16
                                || size.lane_size() == ScalarSize::Size32
                        );

                        (0b001_01110_00_1 | enc_size << 1, 0b101101)
                    }
                };
                let top11 = if is_float {
                    top11 | enc_float_size << 1
                } else {
                    top11
                };
                sink.put4(enc_vec_rrr(top11 | q << 9, rm, bit15_10, rn, rd));
            }
            &Inst::VecLoadReplicate { rd, rn, size } => {
                let (q, size) = size.enc_size();

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() {
                    // Register the offset at which the actual load instruction starts.
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                sink.put4(enc_ldst_vec(q, size, rn, rd));
            }
            &Inst::VecCSel { rd, rn, rm, cond } => {
                /* Emit this:
                      b.cond  else
                      mov     rd, rm
                      b       out
                     else:
                      mov     rd, rn
                     out:

                   Note, we could do better in the cases where rd == rn or rd == rm.
                */
                let else_label = sink.get_label();
                let out_label = sink.get_label();

                // b.cond else
                let br_else_offset = sink.cur_offset();
                sink.put4(enc_conditional_br(
                    BranchTarget::Label(else_label),
                    CondBrKind::Cond(cond),
                ));
                sink.use_label_at_offset(br_else_offset, else_label, LabelUse::Branch19);

                // mov rd, rm
                sink.put4(enc_vecmov(/* 16b = */ true, rd, rm));

                // b out
                let b_out_offset = sink.cur_offset();
                sink.use_label_at_offset(b_out_offset, out_label, LabelUse::Branch26);
                sink.add_uncond_branch(b_out_offset, b_out_offset + 4, out_label);
                sink.put4(enc_jump26(0b000101, 0 /* will be fixed up later */));

                // else:
                sink.bind_label(else_label);

                // mov rd, rn
                sink.put4(enc_vecmov(/* 16b = */ true, rd, rn));

                // out:
                sink.bind_label(out_label);
            }
            &Inst::MovToNZCV { rn } => {
                sink.put4(0xd51b4200 | machreg_to_gpr(rn));
            }
            &Inst::MovFromNZCV { rd } => {
                sink.put4(0xd53b4200 | machreg_to_gpr(rd.to_reg()));
            }
            &Inst::Extend {
                rd,
                rn,
                signed: false,
                from_bits: 1,
                to_bits,
            } => {
                assert!(to_bits <= 64);
                // Reduce zero-extend-from-1-bit to:
                // - and rd, rn, #1
                // Note: This is special cased as UBFX may take more cycles
                // than AND on smaller cores.
                let imml = ImmLogic::maybe_from_u64(1, I32).unwrap();
                Inst::AluRRImmLogic {
                    alu_op: ALUOp::And,
                    size: OperandSize::Size32,
                    rd,
                    rn,
                    imml,
                }
                .emit(sink, emit_info, state);
            }
            &Inst::Extend {
                rd,
                rn,
                signed: false,
                from_bits: 32,
                to_bits: 64,
            } => {
                let mov = Inst::Mov32 { rd, rm: rn };
                mov.emit(sink, emit_info, state);
            }
            &Inst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits,
            } => {
                let (opc, size) = if signed {
                    (0b00, OperandSize::from_bits(to_bits))
                } else {
                    (0b10, OperandSize::Size32)
                };
                sink.put4(enc_bfm(opc, size, rd, rn, 0, from_bits - 1));
            }
            &Inst::Jump { ref dest } => {
                let off = sink.cur_offset();
                // Indicate that the jump uses a label, if so, so that a fixup can occur later.
                if let Some(l) = dest.as_label() {
                    sink.use_label_at_offset(off, l, LabelUse::Branch26);
                    sink.add_uncond_branch(off, off + 4, l);
                }
                // Emit the jump itself.
                sink.put4(enc_jump26(0b000101, dest.as_offset26_or_zero()));
            }
            &Inst::Ret => {
                sink.put4(0xd65f03c0);
            }
            &Inst::EpiloguePlaceholder => {
                // Noop; this is just a placeholder for epilogues.
            }
            &Inst::Call { ref info } => {
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                }
                let loc = state.cur_srcloc();
                sink.add_reloc(loc, Reloc::Arm64Call, &info.dest, 0);
                sink.put4(enc_jump26(0b100101, 0));
                if info.opcode.is_call() {
                    sink.add_call_site(loc, info.opcode);
                }
            }
            &Inst::CallInd { ref info } => {
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                }
                sink.put4(0b1101011_0001_11111_000000_00000_00000 | (machreg_to_gpr(info.rn) << 5));
                let loc = state.cur_srcloc();
                if info.opcode.is_call() {
                    sink.add_call_site(loc, info.opcode);
                }
            }
            &Inst::CondBr {
                taken,
                not_taken,
                kind,
            } => {
                // Conditional part first.
                let cond_off = sink.cur_offset();
                if let Some(l) = taken.as_label() {
                    sink.use_label_at_offset(cond_off, l, LabelUse::Branch19);
                    let inverted = enc_conditional_br(taken, kind.invert()).to_le_bytes();
                    sink.add_cond_branch(cond_off, cond_off + 4, l, &inverted[..]);
                }
                sink.put4(enc_conditional_br(taken, kind));

                // Unconditional part next.
                let uncond_off = sink.cur_offset();
                if let Some(l) = not_taken.as_label() {
                    sink.use_label_at_offset(uncond_off, l, LabelUse::Branch26);
                    sink.add_uncond_branch(uncond_off, uncond_off + 4, l);
                }
                sink.put4(enc_jump26(0b000101, not_taken.as_offset26_or_zero()));
            }
            &Inst::TrapIf { kind, trap_code } => {
                // condbr KIND, LABEL
                let off = sink.cur_offset();
                let label = sink.get_label();
                sink.put4(enc_conditional_br(
                    BranchTarget::Label(label),
                    kind.invert(),
                ));
                sink.use_label_at_offset(off, label, LabelUse::Branch19);
                // udf
                let trap = Inst::Udf { trap_code };
                trap.emit(sink, emit_info, state);
                // LABEL:
                sink.bind_label(label);
            }
            &Inst::IndirectBr { rn, .. } => {
                sink.put4(enc_br(rn));
            }
            &Inst::Nop0 => {}
            &Inst::Nop4 => {
                sink.put4(0xd503201f);
            }
            &Inst::Brk => {
                sink.put4(0xd4200000);
            }
            &Inst::Udf { trap_code } => {
                let srcloc = state.cur_srcloc();
                sink.add_trap(srcloc, trap_code);
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(4), s);
                }
                sink.put4(0xd4a00000);
            }
            &Inst::Adr { rd, off } => {
                assert!(off > -(1 << 20));
                assert!(off < (1 << 20));
                sink.put4(enc_adr(off, rd));
            }
            &Inst::Word4 { data } => {
                sink.put4(data);
            }
            &Inst::Word8 { data } => {
                sink.put8(data);
            }
            &Inst::JTSequence {
                ridx,
                rtmp1,
                rtmp2,
                ref info,
                ..
            } => {
                // This sequence is *one* instruction in the vcode, and is expanded only here at
                // emission time, because we cannot allow the regalloc to insert spills/reloads in
                // the middle; we depend on hardcoded PC-rel addressing below.

                // Branch to default when condition code from prior comparison indicates.
                let br = enc_conditional_br(info.default_target, CondBrKind::Cond(Cond::Hs));
                // No need to inform the sink's branch folding logic about this branch, because it
                // will not be merged with any other branch, flipped, or elided (it is not preceded
                // or succeeded by any other branch). Just emit it with the label use.
                let default_br_offset = sink.cur_offset();
                if let BranchTarget::Label(l) = info.default_target {
                    sink.use_label_at_offset(default_br_offset, l, LabelUse::Branch19);
                }
                sink.put4(br);

                // Save index in a tmp (the live range of ridx only goes to start of this
                // sequence; rtmp1 or rtmp2 may overwrite it).
                let inst = Inst::gen_move(rtmp2, ridx, I64);
                inst.emit(sink, emit_info, state);
                // Load address of jump table
                let inst = Inst::Adr { rd: rtmp1, off: 16 };
                inst.emit(sink, emit_info, state);
                // Load value out of jump table
                let inst = Inst::SLoad32 {
                    rd: rtmp2,
                    mem: AMode::reg_plus_reg_scaled_extended(
                        rtmp1.to_reg(),
                        rtmp2.to_reg(),
                        I32,
                        ExtendOp::UXTW,
                    ),
                    flags: MemFlags::trusted(),
                };
                inst.emit(sink, emit_info, state);
                // Add base of jump table to jump-table-sourced block offset
                let inst = Inst::AluRRR {
                    alu_op: ALUOp::Add,
                    size: OperandSize::Size64,
                    rd: rtmp1,
                    rn: rtmp1.to_reg(),
                    rm: rtmp2.to_reg(),
                };
                inst.emit(sink, emit_info, state);
                // Branch to computed address. (`targets` here is only used for successor queries
                // and is not needed for emission.)
                let inst = Inst::IndirectBr {
                    rn: rtmp1.to_reg(),
                    targets: vec![],
                };
                inst.emit(sink, emit_info, state);
                // Emit jump table (table of 32-bit offsets).
                let jt_off = sink.cur_offset();
                for &target in info.targets.iter() {
                    let word_off = sink.cur_offset();
                    // off_into_table is an addend here embedded in the label to be later patched
                    // at the end of codegen. The offset is initially relative to this jump table
                    // entry; with the extra addend, it'll be relative to the jump table's start,
                    // after patching.
                    let off_into_table = word_off - jt_off;
                    sink.use_label_at_offset(
                        word_off,
                        target.as_label().unwrap(),
                        LabelUse::PCRel32,
                    );
                    sink.put4(off_into_table);
                }

                // Lowering produces an EmitIsland before using a JTSequence, so we can safely
                // disable the worst-case-size check in this case.
                start_off = sink.cur_offset();
            }
            &Inst::LoadExtName {
                rd,
                ref name,
                offset,
            } => {
                let inst = Inst::ULoad64 {
                    rd,
                    mem: AMode::Label(MemLabel::PCRel(8)),
                    flags: MemFlags::trusted(),
                };
                inst.emit(sink, emit_info, state);
                let inst = Inst::Jump {
                    dest: BranchTarget::ResolvedOffset(12),
                };
                inst.emit(sink, emit_info, state);
                let srcloc = state.cur_srcloc();
                sink.add_reloc(srcloc, Reloc::Abs8, name, offset);
                if emit_info.0.emit_all_ones_funcaddrs() {
                    sink.put8(u64::max_value());
                } else {
                    sink.put8(0);
                }
            }
            &Inst::LoadAddr { rd, ref mem } => {
                let (mem_insts, mem) = mem_finalize(sink.cur_offset(), mem, state);
                for inst in mem_insts.into_iter() {
                    inst.emit(sink, emit_info, state);
                }

                let (reg, index_reg, offset) = match mem {
                    AMode::RegExtended(r, idx, extendop) => (r, Some((idx, extendop)), 0),
                    AMode::Unscaled(r, simm9) => (r, None, simm9.value()),
                    AMode::UnsignedOffset(r, uimm12scaled) => {
                        (r, None, uimm12scaled.value() as i32)
                    }
                    _ => panic!("Unsupported case for LoadAddr: {:?}", mem),
                };
                let abs_offset = if offset < 0 {
                    -offset as u64
                } else {
                    offset as u64
                };
                let alu_op = if offset < 0 { ALUOp::Sub } else { ALUOp::Add };

                if let Some((idx, extendop)) = index_reg {
                    let add = Inst::AluRRRExtend {
                        alu_op: ALUOp::Add,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        rm: idx,
                        extendop,
                    };

                    add.emit(sink, emit_info, state);
                } else if offset == 0 {
                    if reg != rd.to_reg() {
                        let mov = Inst::Mov64 { rd, rm: reg };

                        mov.emit(sink, emit_info, state);
                    }
                } else if let Some(imm12) = Imm12::maybe_from_u64(abs_offset) {
                    let add = Inst::AluRRImm12 {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        imm12,
                    };
                    add.emit(sink, emit_info, state);
                } else {
                    // Use `tmp2` here: `reg` may be `spilltmp` if the `AMode` on this instruction
                    // was initially an `SPOffset`. Assert that `tmp2` is truly free to use. Note
                    // that no other instructions will be inserted here (we're emitting directly),
                    // and a live range of `tmp2` should not span this instruction, so this use
                    // should otherwise be correct.
                    debug_assert!(rd.to_reg() != tmp2_reg());
                    debug_assert!(reg != tmp2_reg());
                    let tmp = writable_tmp2_reg();
                    for insn in Inst::load_constant(tmp, abs_offset).into_iter() {
                        insn.emit(sink, emit_info, state);
                    }
                    let add = Inst::AluRRR {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        rm: tmp.to_reg(),
                    };
                    add.emit(sink, emit_info, state);
                }
            }
            &Inst::VirtualSPOffsetAdj { offset } => {
                log::trace!(
                    "virtual sp offset adjusted by {} -> {}",
                    offset,
                    state.virtual_sp_offset + offset,
                );
                state.virtual_sp_offset += offset;
            }
            &Inst::EmitIsland { needed_space } => {
                if sink.island_needed(needed_space + 4) {
                    let jump_around_label = sink.get_label();
                    let jmp = Inst::Jump {
                        dest: BranchTarget::Label(jump_around_label),
                    };
                    jmp.emit(sink, emit_info, state);
                    sink.emit_island(needed_space + 4);
                    sink.bind_label(jump_around_label);
                }
            }

            &Inst::ElfTlsGetAddr { ref symbol } => {
                // This is the instruction sequence that GCC emits for ELF GD TLS Relocations in aarch64
                // See: https://gcc.godbolt.org/z/KhMh5Gvra

                // adrp x0, <label>
                sink.add_reloc(state.cur_srcloc(), Reloc::Aarch64TlsGdAdrPage21, symbol, 0);
                sink.put4(0x90000000);

                // add x0, x0, <label>
                sink.add_reloc(state.cur_srcloc(), Reloc::Aarch64TlsGdAddLo12Nc, symbol, 0);
                sink.put4(0x91000000);

                // bl __tls_get_addr
                sink.add_reloc(
                    state.cur_srcloc(),
                    Reloc::Arm64Call,
                    &ExternalName::LibCall(LibCall::ElfTlsGetAddr),
                    0,
                );
                sink.put4(0x94000000);

                // nop
                sink.put4(0xd503201f);
            }

            &Inst::ValueLabelMarker { .. } => {
                // Nothing; this is only used to compute debug info.
            }

            &Inst::Unwind { ref inst } => {
                sink.add_unwind(inst.clone());
            }
        }

        let end_off = sink.cur_offset();
        debug_assert!((end_off - start_off) <= Inst::worst_case_size());

        state.clear_post_insn();
    }

    fn pretty_print(&self, mb_rru: Option<&RealRegUniverse>, state: &mut EmitState) -> String {
        self.print_with_state(mb_rru, state)
    }
}
