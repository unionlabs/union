//! S390x ISA: binary code emission.

use crate::binemit::{Reloc, StackMap};
use crate::ir::MemFlags;
use crate::ir::{SourceLoc, TrapCode};
use crate::isa::s390x::inst::*;
use crate::isa::s390x::settings as s390x_settings;
use core::convert::TryFrom;
use regalloc::{Reg, RegClass};

/// Memory addressing mode finalization: convert "special" modes (e.g.,
/// generic arbitrary stack offset) into real addressing modes, possibly by
/// emitting some helper instructions that come immediately before the use
/// of this amode.
pub fn mem_finalize(
    mem: &MemArg,
    state: &EmitState,
    have_d12: bool,
    have_d20: bool,
    have_pcrel: bool,
    have_index: bool,
) -> (SmallVec<[Inst; 4]>, MemArg) {
    let mut insts = SmallVec::new();

    // Resolve virtual addressing modes.
    let mem = match mem {
        &MemArg::RegOffset { off, .. }
        | &MemArg::InitialSPOffset { off }
        | &MemArg::NominalSPOffset { off } => {
            let base = match mem {
                &MemArg::RegOffset { reg, .. } => reg,
                &MemArg::InitialSPOffset { .. } | &MemArg::NominalSPOffset { .. } => stack_reg(),
                _ => unreachable!(),
            };
            let adj = match mem {
                &MemArg::InitialSPOffset { .. } => {
                    state.initial_sp_offset + state.virtual_sp_offset
                }
                &MemArg::NominalSPOffset { .. } => state.virtual_sp_offset,
                _ => 0,
            };
            let off = off + adj;

            if let Some(disp) = UImm12::maybe_from_u64(off as u64) {
                MemArg::BXD12 {
                    base,
                    index: zero_reg(),
                    disp,
                    flags: mem.get_flags(),
                }
            } else if let Some(disp) = SImm20::maybe_from_i64(off) {
                MemArg::BXD20 {
                    base,
                    index: zero_reg(),
                    disp,
                    flags: mem.get_flags(),
                }
            } else {
                let tmp = writable_spilltmp_reg();
                assert!(base != tmp.to_reg());
                insts.extend(Inst::load_constant64(tmp, off as u64));
                MemArg::reg_plus_reg(base, tmp.to_reg(), mem.get_flags())
            }
        }
        _ => mem.clone(),
    };

    // If this addressing mode cannot be handled by the instruction, use load-address.
    let need_load_address = match &mem {
        &MemArg::Label { .. } | &MemArg::Symbol { .. } if !have_pcrel => true,
        &MemArg::BXD20 { .. } if !have_d20 => true,
        &MemArg::BXD12 { index, .. } | &MemArg::BXD20 { index, .. } if !have_index => {
            index != zero_reg()
        }
        _ => false,
    };
    let mem = if need_load_address {
        let flags = mem.get_flags();
        let tmp = writable_spilltmp_reg();
        insts.push(Inst::LoadAddr { rd: tmp, mem });
        MemArg::reg(tmp.to_reg(), flags)
    } else {
        mem
    };

    // Convert 12-bit displacement to 20-bit if required.
    let mem = match &mem {
        &MemArg::BXD12 {
            base,
            index,
            disp,
            flags,
        } if !have_d12 => {
            assert!(have_d20);
            MemArg::BXD20 {
                base,
                index,
                disp: SImm20::from_uimm12(disp),
                flags,
            }
        }
        _ => mem,
    };

    (insts, mem)
}

pub fn mem_emit(
    rd: Reg,
    mem: &MemArg,
    opcode_rx: Option<u16>,
    opcode_rxy: Option<u16>,
    opcode_ril: Option<u16>,
    add_trap: bool,
    sink: &mut MachBuffer<Inst>,
    emit_info: &EmitInfo,
    state: &mut EmitState,
) {
    let (mem_insts, mem) = mem_finalize(
        mem,
        state,
        opcode_rx.is_some(),
        opcode_rxy.is_some(),
        opcode_ril.is_some(),
        true,
    );
    for inst in mem_insts.into_iter() {
        inst.emit(sink, emit_info, state);
    }

    if add_trap && mem.can_trap() {
        let srcloc = state.cur_srcloc();
        if srcloc != SourceLoc::default() {
            sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
        }
    }

    match &mem {
        &MemArg::BXD12 {
            base, index, disp, ..
        } => {
            put(
                sink,
                &enc_rx(opcode_rx.unwrap(), rd, base, index, disp.bits()),
            );
        }
        &MemArg::BXD20 {
            base, index, disp, ..
        } => {
            put(
                sink,
                &enc_rxy(opcode_rxy.unwrap(), rd, base, index, disp.bits()),
            );
        }
        &MemArg::Label { target } => {
            sink.use_label_at_offset(sink.cur_offset(), target, LabelUse::BranchRIL);
            put(sink, &enc_ril_b(opcode_ril.unwrap(), rd, 0));
        }
        &MemArg::Symbol {
            ref name, offset, ..
        } => {
            let reloc = Reloc::S390xPCRel32Dbl;
            let srcloc = state.cur_srcloc();
            put_with_reloc(
                sink,
                &enc_ril_b(opcode_ril.unwrap(), rd, 0),
                2,
                srcloc,
                reloc,
                name,
                offset.into(),
            );
        }
        _ => unreachable!(),
    }
}

pub fn mem_rs_emit(
    rd: Reg,
    rn: Reg,
    mem: &MemArg,
    opcode_rs: Option<u16>,
    opcode_rsy: Option<u16>,
    add_trap: bool,
    sink: &mut MachBuffer<Inst>,
    emit_info: &EmitInfo,
    state: &mut EmitState,
) {
    let (mem_insts, mem) = mem_finalize(
        mem,
        state,
        opcode_rs.is_some(),
        opcode_rsy.is_some(),
        false,
        false,
    );
    for inst in mem_insts.into_iter() {
        inst.emit(sink, emit_info, state);
    }

    if add_trap && mem.can_trap() {
        let srcloc = state.cur_srcloc();
        if srcloc != SourceLoc::default() {
            sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
        }
    }

    match &mem {
        &MemArg::BXD12 {
            base, index, disp, ..
        } => {
            assert!(index == zero_reg());
            put(sink, &enc_rs(opcode_rs.unwrap(), rd, rn, base, disp.bits()));
        }
        &MemArg::BXD20 {
            base, index, disp, ..
        } => {
            assert!(index == zero_reg());
            put(
                sink,
                &enc_rsy(opcode_rsy.unwrap(), rd, rn, base, disp.bits()),
            );
        }
        _ => unreachable!(),
    }
}

pub fn mem_imm8_emit(
    imm: u8,
    mem: &MemArg,
    opcode_si: u16,
    opcode_siy: u16,
    add_trap: bool,
    sink: &mut MachBuffer<Inst>,
    emit_info: &EmitInfo,
    state: &mut EmitState,
) {
    let (mem_insts, mem) = mem_finalize(mem, state, true, true, false, false);
    for inst in mem_insts.into_iter() {
        inst.emit(sink, emit_info, state);
    }

    if add_trap && mem.can_trap() {
        let srcloc = state.cur_srcloc();
        if srcloc != SourceLoc::default() {
            sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
        }
    }

    match &mem {
        &MemArg::BXD12 {
            base, index, disp, ..
        } => {
            assert!(index == zero_reg());
            put(sink, &enc_si(opcode_si, base, disp.bits(), imm));
        }
        &MemArg::BXD20 {
            base, index, disp, ..
        } => {
            assert!(index == zero_reg());
            put(sink, &enc_siy(opcode_siy, base, disp.bits(), imm));
        }
        _ => unreachable!(),
    }
}

pub fn mem_imm16_emit(
    imm: i16,
    mem: &MemArg,
    opcode_sil: u16,
    add_trap: bool,
    sink: &mut MachBuffer<Inst>,
    emit_info: &EmitInfo,
    state: &mut EmitState,
) {
    let (mem_insts, mem) = mem_finalize(mem, state, true, false, false, false);
    for inst in mem_insts.into_iter() {
        inst.emit(sink, emit_info, state);
    }

    if add_trap && mem.can_trap() {
        let srcloc = state.cur_srcloc();
        if srcloc != SourceLoc::default() {
            sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
        }
    }

    match &mem {
        &MemArg::BXD12 {
            base, index, disp, ..
        } => {
            assert!(index == zero_reg());
            put(sink, &enc_sil(opcode_sil, base, disp.bits(), imm));
        }
        _ => unreachable!(),
    }
}

//=============================================================================
// Instructions and subcomponents: emission

fn machreg_to_gpr(m: Reg) -> u8 {
    assert_eq!(m.get_class(), RegClass::I64);
    u8::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

fn machreg_to_fpr(m: Reg) -> u8 {
    assert_eq!(m.get_class(), RegClass::F64);
    u8::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

fn machreg_to_gpr_or_fpr(m: Reg) -> u8 {
    u8::try_from(m.to_real_reg().get_hw_encoding()).unwrap()
}

/// E-type instructions.
///
///   15
///   opcode
///        0
///
fn enc_e(opcode: u16) -> [u8; 2] {
    let mut enc: [u8; 2] = [0; 2];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;

    enc[0] = opcode1;
    enc[1] = opcode2;
    enc
}

/// RIa-type instructions.
///
///   31      23 19      15
///   opcode1 r1 opcode2 i2
///        24 20      16  0
///
fn enc_ri_a(opcode: u16, r1: Reg, i2: u16) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | opcode2;
    enc[2..].copy_from_slice(&i2.to_be_bytes());
    enc
}

/// RIb-type instructions.
///
///   31      23 19      15
///   opcode1 r1 opcode2 ri2
///        24 20      16   0
///
fn enc_ri_b(opcode: u16, r1: Reg, ri2: i32) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;
    let ri2 = ((ri2 >> 1) & 0xffff) as u16;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | opcode2;
    enc[2..].copy_from_slice(&ri2.to_be_bytes());
    enc
}

/// RIc-type instructions.
///
///   31      23 19      15
///   opcode1 m1 opcode2 ri2
///        24 20      16   0
///
fn enc_ri_c(opcode: u16, m1: u8, ri2: i32) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let m1 = m1 & 0x0f;
    let ri2 = ((ri2 >> 1) & 0xffff) as u16;

    enc[0] = opcode1;
    enc[1] = m1 << 4 | opcode2;
    enc[2..].copy_from_slice(&ri2.to_be_bytes());
    enc
}

/// RIEa-type instructions.
///
///   47      39 35 31 15 11 7
///   opcode1 r1 -- i2 m3 -- opcode2
///        40 36 32 16 12 8       0
///
fn enc_rie_a(opcode: u16, r1: Reg, i2: u16, m3: u8) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;
    let m3 = m3 & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4;
    enc[2..4].copy_from_slice(&i2.to_be_bytes());
    enc[4] = m3 << 4;
    enc[5] = opcode2;
    enc
}

/// RIEd-type instructions.
///
///   47      39 35 31 15 7
///   opcode1 r1 r3 i2 -- opcode2
///        40 36 32 16  8       0
///
fn enc_rie_d(opcode: u16, r1: Reg, r3: Reg, i2: u16) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;
    let r3 = machreg_to_gpr(r3) & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | r3;
    enc[2..4].copy_from_slice(&i2.to_be_bytes());
    enc[5] = opcode2;
    enc
}

/// RIEf-type instructions.
///
///   47      39 35 31 23 15 7
///   opcode1 r1 r2 i3 i4 i5 opcode2
///        40 36 32 24 16  8       0
///
fn enc_rie_f(opcode: u16, r1: Reg, r2: Reg, i3: u8, i4: u8, i5: u8) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;
    let r2 = machreg_to_gpr(r2) & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | r2;
    enc[2] = i3;
    enc[3] = i4;
    enc[4] = i5;
    enc[5] = opcode2;
    enc
}

/// RIEg-type instructions.
///
///   47      39 35 31 15 7
///   opcode1 r1 m3 i2 -- opcode2
///        40 36 32 16  8       0
///
fn enc_rie_g(opcode: u16, r1: Reg, i2: u16, m3: u8) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;
    let m3 = m3 & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | m3;
    enc[2..4].copy_from_slice(&i2.to_be_bytes());
    enc[5] = opcode2;
    enc
}

/// RILa-type instructions.
///
///   47      39 35      31
///   opcode1 r1 opcode2 i2
///        40 36      32  0
///
fn enc_ril_a(opcode: u16, r1: Reg, i2: u32) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | opcode2;
    enc[2..].copy_from_slice(&i2.to_be_bytes());
    enc
}

/// RILb-type instructions.
///
///   47      39 35      31
///   opcode1 r1 opcode2 ri2
///        40 36      32   0
///
fn enc_ril_b(opcode: u16, r1: Reg, ri2: u32) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let r1 = machreg_to_gpr(r1) & 0x0f;

    enc[0] = opcode1;
    enc[1] = r1 << 4 | opcode2;
    enc[2..].copy_from_slice(&ri2.to_be_bytes());
    enc
}

/// RILc-type instructions.
///
///   47      39 35      31
///   opcode1 m1 opcode2 i2
///        40 36      32  0
///
fn enc_ril_c(opcode: u16, m1: u8, ri2: u32) -> [u8; 6] {
    let mut enc: [u8; 6] = [0; 6];
    let opcode1 = ((opcode >> 4) & 0xff) as u8;
    let opcode2 = (opcode & 0xf) as u8;
    let m1 = m1 & 0x0f;

    enc[0] = opcode1;
    enc[1] = m1 << 4 | opcode2;
    enc[2..].copy_from_slice(&ri2.to_be_bytes());
    enc
}

/// RR-type instructions.
///
///   15     7  3
///   opcode r1 r2
///        8  4  0
///
fn enc_rr(opcode: u16, r1: Reg, r2: Reg) -> [u8; 2] {
    let mut enc: [u8; 2] = [0; 2];
    let opcode = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r2 = machreg_to_gpr_or_fpr(r2) & 0x0f;

    enc[0] = opcode;
    enc[1] = r1 << 4 | r2;
    enc
}

/// RRD-type instructions.
///
///   31     15 11 7  3
///   opcode r1 -- r3 r2
///       16 12  8 4  0
///
fn enc_rrd(opcode: u16, r1: Reg, r2: Reg, r3: Reg) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_fpr(r1) & 0x0f;
    let r2 = machreg_to_fpr(r2) & 0x0f;
    let r3 = machreg_to_fpr(r3) & 0x0f;

    enc[0] = opcode1;
    enc[1] = opcode2;
    enc[2] = r1 << 4;
    enc[3] = r3 << 4 | r2;
    enc
}

/// RRE-type instructions.
///
///   31     15 7  3
///   opcode -- r1 r2
///       16  8  4  0
///
fn enc_rre(opcode: u16, r1: Reg, r2: Reg) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r2 = machreg_to_gpr_or_fpr(r2) & 0x0f;

    enc[0] = opcode1;
    enc[1] = opcode2;
    enc[3] = r1 << 4 | r2;
    enc
}

/// RRFa/b-type instructions.
///
///   31     15 11 7  3
///   opcode r3 m4 r1 r2
///       16 12  8  4  0
///
fn enc_rrf_ab(opcode: u16, r1: Reg, r2: Reg, r3: Reg, m4: u8) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r2 = machreg_to_gpr_or_fpr(r2) & 0x0f;
    let r3 = machreg_to_gpr_or_fpr(r3) & 0x0f;
    let m4 = m4 & 0x0f;

    enc[0] = opcode1;
    enc[1] = opcode2;
    enc[2] = r3 << 4 | m4;
    enc[3] = r1 << 4 | r2;
    enc
}

/// RRFc/d/e-type instructions.
///
///   31     15 11 7  3
///   opcode m3 m4 r1 r2
///       16 12  8  4  0
///
fn enc_rrf_cde(opcode: u16, r1: Reg, r2: Reg, m3: u8, m4: u8) -> [u8; 4] {
    let mut enc: [u8; 4] = [0; 4];
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r2 = machreg_to_gpr_or_fpr(r2) & 0x0f;
    let m3 = m3 & 0x0f;
    let m4 = m4 & 0x0f;

    enc[0] = opcode1;
    enc[1] = opcode2;
    enc[2] = m3 << 4 | m4;
    enc[3] = r1 << 4 | r2;
    enc
}

/// RS-type instructions.
///
///   31     23 19 15 11
///   opcode r1 r3 b2 d2
///       24 20 16 12  0
///
fn enc_rs(opcode: u16, r1: Reg, r3: Reg, b2: Reg, d2: u32) -> [u8; 4] {
    let opcode = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r3 = machreg_to_gpr_or_fpr(r3) & 0x0f;
    let b2 = machreg_to_gpr(b2) & 0x0f;
    let d2_lo = (d2 & 0xff) as u8;
    let d2_hi = ((d2 >> 8) & 0x0f) as u8;

    let mut enc: [u8; 4] = [0; 4];
    enc[0] = opcode;
    enc[1] = r1 << 4 | r3;
    enc[2] = b2 << 4 | d2_hi;
    enc[3] = d2_lo;
    enc
}

/// RSY-type instructions.
///
///   47      39 35 31 27  15  7
///   opcode1 r1 r3 b2 dl2 dh2 opcode2
///        40 36 32 28  16   8       0
///
fn enc_rsy(opcode: u16, r1: Reg, r3: Reg, b2: Reg, d2: u32) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let r3 = machreg_to_gpr_or_fpr(r3) & 0x0f;
    let b2 = machreg_to_gpr(b2) & 0x0f;
    let dl2_lo = (d2 & 0xff) as u8;
    let dl2_hi = ((d2 >> 8) & 0x0f) as u8;
    let dh2 = ((d2 >> 12) & 0xff) as u8;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = r1 << 4 | r3;
    enc[2] = b2 << 4 | dl2_hi;
    enc[3] = dl2_lo;
    enc[4] = dh2;
    enc[5] = opcode2;
    enc
}

/// RX-type instructions.
///
///   31     23 19 15 11
///   opcode r1 x2 b2 d2
///       24 20 16 12  0
///
fn enc_rx(opcode: u16, r1: Reg, b2: Reg, x2: Reg, d2: u32) -> [u8; 4] {
    let opcode = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let b2 = machreg_to_gpr(b2) & 0x0f;
    let x2 = machreg_to_gpr(x2) & 0x0f;
    let d2_lo = (d2 & 0xff) as u8;
    let d2_hi = ((d2 >> 8) & 0x0f) as u8;

    let mut enc: [u8; 4] = [0; 4];
    enc[0] = opcode;
    enc[1] = r1 << 4 | x2;
    enc[2] = b2 << 4 | d2_hi;
    enc[3] = d2_lo;
    enc
}

/// RXY-type instructions.
///
///   47      39 35 31 27  15  7
///   opcode1 r1 x2 b2 dl2 dh2 opcode2
///        40 36 32 28  16   8       0
///
fn enc_rxy(opcode: u16, r1: Reg, b2: Reg, x2: Reg, d2: u32) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let r1 = machreg_to_gpr_or_fpr(r1) & 0x0f;
    let b2 = machreg_to_gpr(b2) & 0x0f;
    let x2 = machreg_to_gpr(x2) & 0x0f;
    let dl2_lo = (d2 & 0xff) as u8;
    let dl2_hi = ((d2 >> 8) & 0x0f) as u8;
    let dh2 = ((d2 >> 12) & 0xff) as u8;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = r1 << 4 | x2;
    enc[2] = b2 << 4 | dl2_hi;
    enc[3] = dl2_lo;
    enc[4] = dh2;
    enc[5] = opcode2;
    enc
}

/// SI-type instructions.
///
///   31     23 15 11
///   opcode i2 b1 d1
///       24 16 12  0
///
fn enc_si(opcode: u16, b1: Reg, d1: u32, i2: u8) -> [u8; 4] {
    let opcode = (opcode & 0xff) as u8;
    let b1 = machreg_to_gpr(b1) & 0x0f;
    let d1_lo = (d1 & 0xff) as u8;
    let d1_hi = ((d1 >> 8) & 0x0f) as u8;

    let mut enc: [u8; 4] = [0; 4];
    enc[0] = opcode;
    enc[1] = i2;
    enc[2] = b1 << 4 | d1_hi;
    enc[3] = d1_lo;
    enc
}

/// SIL-type instructions.
///
///   47     31 27 15
///   opcode b1 d1 i2
///       32 28 16  0
///
fn enc_sil(opcode: u16, b1: Reg, d1: u32, i2: i16) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let b1 = machreg_to_gpr(b1) & 0x0f;
    let d1_lo = (d1 & 0xff) as u8;
    let d1_hi = ((d1 >> 8) & 0x0f) as u8;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = opcode2;
    enc[2] = b1 << 4 | d1_hi;
    enc[3] = d1_lo;
    enc[4..].copy_from_slice(&i2.to_be_bytes());
    enc
}

/// SIY-type instructions.
///
///   47      39 31 27  15  7
///   opcode1 i2 b1 dl1 dh1 opcode2
///        40 32 28  16   8       0
///
fn enc_siy(opcode: u16, b1: Reg, d1: u32, i2: u8) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let b1 = machreg_to_gpr(b1) & 0x0f;
    let dl1_lo = (d1 & 0xff) as u8;
    let dl1_hi = ((d1 >> 8) & 0x0f) as u8;
    let dh1 = ((d1 >> 12) & 0xff) as u8;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = i2;
    enc[2] = b1 << 4 | dl1_hi;
    enc[3] = dl1_lo;
    enc[4] = dh1;
    enc[5] = opcode2;
    enc
}

/// VRR-type instructions.
///
///   47      39 35 31 27 23 19 15 11  7
///   opcode1 v1 v2 v3 -  m6 m5 m4 rxb opcode2
///        40 36 32 28 24 20 16 12   8       0
///
fn enc_vrr(opcode: u16, v1: Reg, v2: Reg, v3: Reg, m4: u8, m5: u8, m6: u8) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let rxb = 0; // FIXME
    let v1 = machreg_to_fpr(v1) & 0x0f; // FIXME
    let v2 = machreg_to_fpr(v2) & 0x0f; // FIXME
    let v3 = machreg_to_fpr(v3) & 0x0f; // FIXME
    let m4 = m4 & 0x0f;
    let m5 = m5 & 0x0f;
    let m6 = m6 & 0x0f;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = v1 << 4 | v2;
    enc[2] = v3 << 4;
    enc[3] = m6 << 4 | m5;
    enc[4] = m4 << 4 | rxb;
    enc[5] = opcode2;
    enc
}

/// VRX-type instructions.
///
///   47      39 35 31 27 15 11  7
///   opcode1 v1 x2 b2 d2 m3 rxb opcode2
///        40 36 32 28 16 12   8       0
///
fn enc_vrx(opcode: u16, v1: Reg, b2: Reg, x2: Reg, d2: u32, m3: u8) -> [u8; 6] {
    let opcode1 = ((opcode >> 8) & 0xff) as u8;
    let opcode2 = (opcode & 0xff) as u8;
    let rxb = 0; // FIXME
    let v1 = machreg_to_fpr(v1) & 0x0f; // FIXME
    let b2 = machreg_to_gpr(b2) & 0x0f;
    let x2 = machreg_to_gpr(x2) & 0x0f;
    let d2_lo = (d2 & 0xff) as u8;
    let d2_hi = ((d2 >> 8) & 0x0f) as u8;
    let m3 = m3 & 0x0f;

    let mut enc: [u8; 6] = [0; 6];
    enc[0] = opcode1;
    enc[1] = v1 << 4 | x2;
    enc[2] = b2 << 4 | d2_hi;
    enc[3] = d2_lo;
    enc[4] = m3 << 4 | rxb;
    enc[5] = opcode2;
    enc
}

/// Emit encoding to sink.
fn put(sink: &mut MachBuffer<Inst>, enc: &[u8]) {
    for byte in enc {
        sink.put1(*byte);
    }
}

/// Emit encoding to sink, adding a trap on the last byte.
fn put_with_trap(sink: &mut MachBuffer<Inst>, enc: &[u8], srcloc: SourceLoc, trap_code: TrapCode) {
    let len = enc.len();
    for i in 0..len - 1 {
        sink.put1(enc[i]);
    }
    sink.add_trap(srcloc, trap_code);
    sink.put1(enc[len - 1]);
}

/// Emit encoding to sink, adding a relocation at byte offset.
fn put_with_reloc(
    sink: &mut MachBuffer<Inst>,
    enc: &[u8],
    offset: usize,
    ri2_srcloc: SourceLoc,
    ri2_reloc: Reloc,
    ri2_name: &ExternalName,
    ri2_offset: i64,
) {
    let len = enc.len();
    for i in 0..offset {
        sink.put1(enc[i]);
    }
    sink.add_reloc(ri2_srcloc, ri2_reloc, ri2_name, ri2_offset + offset as i64);
    for i in offset..len {
        sink.put1(enc[i]);
    }
}

/// State carried between emissions of a sequence of instructions.
#[derive(Default, Clone, Debug)]
pub struct EmitState {
    pub(crate) initial_sp_offset: i64,
    pub(crate) virtual_sp_offset: i64,
    /// Safepoint stack map for upcoming instruction, as provided to `pre_safepoint()`.
    stack_map: Option<StackMap>,
    /// Current source-code location corresponding to instruction to be emitted.
    cur_srcloc: SourceLoc,
}

impl MachInstEmitState<Inst> for EmitState {
    fn new(abi: &dyn ABICallee<I = Inst>) -> Self {
        EmitState {
            virtual_sp_offset: 0,
            initial_sp_offset: abi.frame_size() as i64,
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
pub struct EmitInfo {
    flags: settings::Flags,
    isa_flags: s390x_settings::Flags,
}

impl EmitInfo {
    pub(crate) fn new(flags: settings::Flags, isa_flags: s390x_settings::Flags) -> Self {
        Self { flags, isa_flags }
    }
}

impl MachInstEmit for Inst {
    type State = EmitState;
    type Info = EmitInfo;

    fn emit(&self, sink: &mut MachBuffer<Inst>, emit_info: &Self::Info, state: &mut EmitState) {
        // Verify that we can emit this Inst in the current ISA
        let matches_isa_flags = |iset_requirement: &InstructionSet| -> bool {
            match iset_requirement {
                // Baseline ISA is z14
                InstructionSet::Base => true,
                // Miscellaneous-Instruction-Extensions Facility 2 (z15)
                InstructionSet::MIE2 => emit_info.isa_flags.has_mie2(),
                // Vector-Enhancements Facility 2 (z15)
                InstructionSet::VXRS_EXT2 => emit_info.isa_flags.has_vxrs_ext2(),
            }
        };
        let isa_requirements = self.available_in_isa();
        if !matches_isa_flags(&isa_requirements) {
            panic!(
                "Cannot emit inst '{:?}' for target; failed to match ISA requirements: {:?}",
                self, isa_requirements
            )
        }

        // N.B.: we *must* not exceed the "worst-case size" used to compute
        // where to insert islands, except when islands are explicitly triggered
        // (with an `EmitIsland`). We check this in debug builds. This is `mut`
        // to allow disabling the check for `JTSequence`, which is always
        // emitted following an `EmitIsland`.
        let mut start_off = sink.cur_offset();

        match self {
            &Inst::AluRRR { alu_op, rd, rn, rm } => {
                let (opcode, have_rr) = match alu_op {
                    ALUOp::Add32 => (0xb9f8, true),        // ARK
                    ALUOp::Add64 => (0xb9e8, true),        // AGRK
                    ALUOp::AddLogical32 => (0xb9fa, true), // ALRK
                    ALUOp::AddLogical64 => (0xb9ea, true), // ALGRK
                    ALUOp::Sub32 => (0xb9f9, true),        // SRK
                    ALUOp::Sub64 => (0xb9e9, true),        // SGRK
                    ALUOp::SubLogical32 => (0xb9fb, true), // SLRK
                    ALUOp::SubLogical64 => (0xb9eb, true), // SLGRK
                    ALUOp::Mul32 => (0xb9fd, true),        // MSRKC
                    ALUOp::Mul64 => (0xb9ed, true),        // MSGRKC
                    ALUOp::And32 => (0xb9f4, true),        // NRK
                    ALUOp::And64 => (0xb9e4, true),        // NGRK
                    ALUOp::Orr32 => (0xb9f6, true),        // ORK
                    ALUOp::Orr64 => (0xb9e6, true),        // OGRK
                    ALUOp::Xor32 => (0xb9f7, true),        // XRK
                    ALUOp::Xor64 => (0xb9e7, true),        // XGRK
                    ALUOp::AndNot32 => (0xb974, false),    // NNRK
                    ALUOp::AndNot64 => (0xb964, false),    // NNGRK
                    ALUOp::OrrNot32 => (0xb976, false),    // NORK
                    ALUOp::OrrNot64 => (0xb966, false),    // NOGRK
                    ALUOp::XorNot32 => (0xb977, false),    // NXRK
                    ALUOp::XorNot64 => (0xb967, false),    // NXGRK
                    _ => unreachable!(),
                };
                if have_rr && rd.to_reg() == rn {
                    let inst = Inst::AluRR { alu_op, rd, rm };
                    inst.emit(sink, emit_info, state);
                } else {
                    put(sink, &enc_rrf_ab(opcode, rd.to_reg(), rn, rm, 0));
                }
            }
            &Inst::AluRRSImm16 {
                alu_op,
                rd,
                rn,
                imm,
            } => {
                if rd.to_reg() == rn {
                    let inst = Inst::AluRSImm16 { alu_op, rd, imm };
                    inst.emit(sink, emit_info, state);
                } else {
                    let opcode = match alu_op {
                        ALUOp::Add32 => 0xecd8, // AHIK
                        ALUOp::Add64 => 0xecd9, // AGHIK
                        _ => unreachable!(),
                    };
                    put(sink, &enc_rie_d(opcode, rd.to_reg(), rn, imm as u16));
                }
            }
            &Inst::AluRR { alu_op, rd, rm } => {
                let (opcode, is_rre) = match alu_op {
                    ALUOp::Add32 => (0x1a, false),              // AR
                    ALUOp::Add64 => (0xb908, true),             // AGR
                    ALUOp::Add64Ext32 => (0xb918, true),        // AGFR
                    ALUOp::AddLogical32 => (0x1e, false),       // ALR
                    ALUOp::AddLogical64 => (0xb90a, true),      // ALGR
                    ALUOp::AddLogical64Ext32 => (0xb91a, true), // ALGFR
                    ALUOp::Sub32 => (0x1b, false),              // SR
                    ALUOp::Sub64 => (0xb909, true),             // SGR
                    ALUOp::Sub64Ext32 => (0xb919, true),        // SGFR
                    ALUOp::SubLogical32 => (0x1f, false),       // SLR
                    ALUOp::SubLogical64 => (0xb90b, true),      // SLGR
                    ALUOp::SubLogical64Ext32 => (0xb91b, true), // SLGFR
                    ALUOp::Mul32 => (0xb252, true),             // MSR
                    ALUOp::Mul64 => (0xb90c, true),             // MSGR
                    ALUOp::Mul64Ext32 => (0xb91c, true),        // MSGFR
                    ALUOp::And32 => (0x14, false),              // NR
                    ALUOp::And64 => (0xb980, true),             // NGR
                    ALUOp::Orr32 => (0x16, false),              // OR
                    ALUOp::Orr64 => (0xb981, true),             // OGR
                    ALUOp::Xor32 => (0x17, false),              // XR
                    ALUOp::Xor64 => (0xb982, true),             // XGR
                    _ => unreachable!(),
                };
                if is_rre {
                    put(sink, &enc_rre(opcode, rd.to_reg(), rm));
                } else {
                    put(sink, &enc_rr(opcode, rd.to_reg(), rm));
                }
            }
            &Inst::AluRX {
                alu_op,
                rd,
                ref mem,
            } => {
                let (opcode_rx, opcode_rxy) = match alu_op {
                    ALUOp::Add32 => (Some(0x5a), Some(0xe35a)),        // A(Y)
                    ALUOp::Add32Ext16 => (Some(0x4a), Some(0xe34a)),   // AH(Y)
                    ALUOp::Add64 => (None, Some(0xe308)),              // AG
                    ALUOp::Add64Ext16 => (None, Some(0xe338)),         // AGH
                    ALUOp::Add64Ext32 => (None, Some(0xe318)),         // AGF
                    ALUOp::AddLogical32 => (Some(0x5e), Some(0xe35e)), // AL(Y)
                    ALUOp::AddLogical64 => (None, Some(0xe30a)),       // ALG
                    ALUOp::AddLogical64Ext32 => (None, Some(0xe31a)),  // ALGF
                    ALUOp::Sub32 => (Some(0x5b), Some(0xe35b)),        // S(Y)
                    ALUOp::Sub32Ext16 => (Some(0x4b), Some(0xe37b)),   // SH(Y)
                    ALUOp::Sub64 => (None, Some(0xe309)),              // SG
                    ALUOp::Sub64Ext16 => (None, Some(0xe339)),         // SGH
                    ALUOp::Sub64Ext32 => (None, Some(0xe319)),         // SGF
                    ALUOp::SubLogical32 => (Some(0x5f), Some(0xe35f)), // SL(Y)
                    ALUOp::SubLogical64 => (None, Some(0xe30b)),       // SLG
                    ALUOp::SubLogical64Ext32 => (None, Some(0xe31b)),  // SLGF
                    ALUOp::Mul32 => (Some(0x71), Some(0xe351)),        // MS(Y)
                    ALUOp::Mul32Ext16 => (Some(0x4c), Some(0xe37c)),   // MH(Y)
                    ALUOp::Mul64 => (None, Some(0xe30c)),              // MSG
                    ALUOp::Mul64Ext16 => (None, Some(0xe33c)),         // MSH
                    ALUOp::Mul64Ext32 => (None, Some(0xe31c)),         // MSGF
                    ALUOp::And32 => (Some(0x54), Some(0xe354)),        // N(Y)
                    ALUOp::And64 => (None, Some(0xe380)),              // NG
                    ALUOp::Orr32 => (Some(0x56), Some(0xe356)),        // O(Y)
                    ALUOp::Orr64 => (None, Some(0xe381)),              // OG
                    ALUOp::Xor32 => (Some(0x57), Some(0xe357)),        // X(Y)
                    ALUOp::Xor64 => (None, Some(0xe382)),              // XG
                    _ => unreachable!(),
                };
                let rd = rd.to_reg();
                mem_emit(
                    rd, mem, opcode_rx, opcode_rxy, None, true, sink, emit_info, state,
                );
            }
            &Inst::AluRSImm16 { alu_op, rd, imm } => {
                let opcode = match alu_op {
                    ALUOp::Add32 => 0xa7a, // AHI
                    ALUOp::Add64 => 0xa7b, // AGHI
                    ALUOp::Mul32 => 0xa7c, // MHI
                    ALUOp::Mul64 => 0xa7d, // MGHI
                    _ => unreachable!(),
                };
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm as u16));
            }
            &Inst::AluRSImm32 { alu_op, rd, imm } => {
                let opcode = match alu_op {
                    ALUOp::Add32 => 0xc29, // AFI
                    ALUOp::Add64 => 0xc28, // AGFI
                    ALUOp::Mul32 => 0xc21, // MSFI
                    ALUOp::Mul64 => 0xc20, // MSGFI
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm as u32));
            }
            &Inst::AluRUImm32 { alu_op, rd, imm } => {
                let opcode = match alu_op {
                    ALUOp::AddLogical32 => 0xc2b, // ALFI
                    ALUOp::AddLogical64 => 0xc2a, // ALGFI
                    ALUOp::SubLogical32 => 0xc25, // SLFI
                    ALUOp::SubLogical64 => 0xc24, // SLGFI
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm));
            }
            &Inst::AluRUImm16Shifted { alu_op, rd, imm } => {
                let opcode = match (alu_op, imm.shift) {
                    (ALUOp::And32, 0) => 0xa57, // NILL
                    (ALUOp::And32, 1) => 0xa56, // NILH
                    (ALUOp::And64, 0) => 0xa57, // NILL
                    (ALUOp::And64, 1) => 0xa56, // NILH
                    (ALUOp::And64, 2) => 0xa55, // NIHL
                    (ALUOp::And64, 3) => 0xa54, // NIHL
                    (ALUOp::Orr32, 0) => 0xa5b, // OILL
                    (ALUOp::Orr32, 1) => 0xa5a, // OILH
                    (ALUOp::Orr64, 0) => 0xa5b, // OILL
                    (ALUOp::Orr64, 1) => 0xa5a, // OILH
                    (ALUOp::Orr64, 2) => 0xa59, // OIHL
                    (ALUOp::Orr64, 3) => 0xa58, // OIHH
                    _ => unreachable!(),
                };
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm.bits));
            }
            &Inst::AluRUImm32Shifted { alu_op, rd, imm } => {
                let opcode = match (alu_op, imm.shift) {
                    (ALUOp::And32, 0) => 0xc0b, // NILF
                    (ALUOp::And64, 0) => 0xc0b, // NILF
                    (ALUOp::And64, 1) => 0xc0a, // NIHF
                    (ALUOp::Orr32, 0) => 0xc0d, // OILF
                    (ALUOp::Orr64, 0) => 0xc0d, // OILF
                    (ALUOp::Orr64, 1) => 0xc0c, // OILF
                    (ALUOp::Xor32, 0) => 0xc07, // XILF
                    (ALUOp::Xor64, 0) => 0xc07, // XILF
                    (ALUOp::Xor64, 1) => 0xc06, // XILH
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm.bits));
            }

            &Inst::SMulWide { rn, rm } => {
                let opcode = 0xb9ec; // MGRK
                put(sink, &enc_rrf_ab(opcode, gpr(0), rn, rm, 0));
            }
            &Inst::UMulWide { rn } => {
                let opcode = 0xb986; // MLGR
                put(sink, &enc_rre(opcode, gpr(0), rn));
            }
            &Inst::SDivMod32 { rn } => {
                let opcode = 0xb91d; // DSGFR
                let srcloc = state.cur_srcloc();
                let trap_code = TrapCode::IntegerDivisionByZero;
                put_with_trap(sink, &enc_rre(opcode, gpr(0), rn), srcloc, trap_code);
            }
            &Inst::SDivMod64 { rn } => {
                let opcode = 0xb90d; // DSGR
                let srcloc = state.cur_srcloc();
                let trap_code = TrapCode::IntegerDivisionByZero;
                put_with_trap(sink, &enc_rre(opcode, gpr(0), rn), srcloc, trap_code);
            }
            &Inst::UDivMod32 { rn } => {
                let opcode = 0xb997; // DLR
                let srcloc = state.cur_srcloc();
                let trap_code = TrapCode::IntegerDivisionByZero;
                put_with_trap(sink, &enc_rre(opcode, gpr(0), rn), srcloc, trap_code);
            }
            &Inst::UDivMod64 { rn } => {
                let opcode = 0xb987; // DLGR
                let srcloc = state.cur_srcloc();
                let trap_code = TrapCode::IntegerDivisionByZero;
                put_with_trap(sink, &enc_rre(opcode, gpr(0), rn), srcloc, trap_code);
            }
            &Inst::Flogr { rn } => {
                let opcode = 0xb983; // FLOGR
                put(sink, &enc_rre(opcode, gpr(0), rn));
            }

            &Inst::ShiftRR {
                shift_op,
                rd,
                rn,
                shift_imm,
                shift_reg,
            } => {
                let opcode = match shift_op {
                    ShiftOp::RotL32 => 0xeb1d, // RLL
                    ShiftOp::RotL64 => 0xeb1c, // RLLG
                    ShiftOp::LShL32 => 0xebdf, // SLLK  (SLL ?)
                    ShiftOp::LShL64 => 0xeb0d, // SLLG
                    ShiftOp::LShR32 => 0xebde, // SRLK  (SRL ?)
                    ShiftOp::LShR64 => 0xeb0c, // SRLG
                    ShiftOp::AShR32 => 0xebdc, // SRAK  (SRA ?)
                    ShiftOp::AShR64 => 0xeb0a, // SRAG
                };
                put(
                    sink,
                    &enc_rsy(opcode, rd.to_reg(), rn, shift_reg, shift_imm.into()),
                );
            }

            &Inst::RxSBG {
                op,
                rd,
                rn,
                start_bit,
                end_bit,
                rotate_amt,
            } => {
                let opcode = match op {
                    RxSBGOp::Insert => 0xec59, // RISBGN
                    RxSBGOp::And => 0xec54,    // RNSBG
                    RxSBGOp::Or => 0xec56,     // ROSBG
                    RxSBGOp::Xor => 0xec57,    // RXSBG
                };
                put(
                    sink,
                    &enc_rie_f(
                        opcode,
                        rd.to_reg(),
                        rn,
                        start_bit,
                        end_bit,
                        (rotate_amt as u8) & 63,
                    ),
                );
            }

            &Inst::RxSBGTest {
                op,
                rd,
                rn,
                start_bit,
                end_bit,
                rotate_amt,
            } => {
                let opcode = match op {
                    RxSBGOp::And => 0xec54, // RNSBG
                    RxSBGOp::Or => 0xec56,  // ROSBG
                    RxSBGOp::Xor => 0xec57, // RXSBG
                    _ => unreachable!(),
                };
                put(
                    sink,
                    &enc_rie_f(
                        opcode,
                        rd,
                        rn,
                        start_bit | 0x80,
                        end_bit,
                        (rotate_amt as u8) & 63,
                    ),
                );
            }

            &Inst::UnaryRR { op, rd, rn } => {
                match op {
                    UnaryOp::Abs32 => {
                        let opcode = 0x10; // LPR
                        put(sink, &enc_rr(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::Abs64 => {
                        let opcode = 0xb900; // LPGR
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::Abs64Ext32 => {
                        let opcode = 0xb910; // LPGFR
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::Neg32 => {
                        let opcode = 0x13; // LCR
                        put(sink, &enc_rr(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::Neg64 => {
                        let opcode = 0xb903; // LCGR
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::Neg64Ext32 => {
                        let opcode = 0xb913; // LCGFR
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::PopcntByte => {
                        let opcode = 0xb9e1; // POPCNT
                        put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rn, 0, 0));
                    }
                    UnaryOp::PopcntReg => {
                        let opcode = 0xb9e1; // POPCNT
                        put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rn, 8, 0));
                    }
                    UnaryOp::BSwap32 => {
                        let opcode = 0xb91f; // LRVR
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                    UnaryOp::BSwap64 => {
                        let opcode = 0xb90f; // LRVRG
                        put(sink, &enc_rre(opcode, rd.to_reg(), rn));
                    }
                }
            }

            &Inst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits,
            } => {
                let opcode = match (signed, from_bits, to_bits) {
                    (_, 1, 32) => 0xb926,      // LBR
                    (_, 1, 64) => 0xb906,      // LGBR
                    (false, 8, 32) => 0xb994,  // LLCR
                    (false, 8, 64) => 0xb984,  // LLGCR
                    (true, 8, 32) => 0xb926,   // LBR
                    (true, 8, 64) => 0xb906,   // LGBR
                    (false, 16, 32) => 0xb995, // LLHR
                    (false, 16, 64) => 0xb985, // LLGHR
                    (true, 16, 32) => 0xb927,  // LHR
                    (true, 16, 64) => 0xb907,  // LGHR
                    (false, 32, 64) => 0xb916, // LLGFR
                    (true, 32, 64) => 0xb914,  // LGFR
                    _ => panic!(
                        "Unsupported extend combination: signed = {}, from_bits = {}, to_bits = {}",
                        signed, from_bits, to_bits
                    ),
                };
                put(sink, &enc_rre(opcode, rd.to_reg(), rn));
            }

            &Inst::CmpRR { op, rn, rm } => {
                let (opcode, is_rre) = match op {
                    CmpOp::CmpS32 => (0x19, false),       // CR
                    CmpOp::CmpS64 => (0xb920, true),      // CGR
                    CmpOp::CmpS64Ext32 => (0xb930, true), // CGFR
                    CmpOp::CmpL32 => (0x15, false),       // CLR
                    CmpOp::CmpL64 => (0xb921, true),      // CLGR
                    CmpOp::CmpL64Ext32 => (0xb931, true), // CLGFR
                    _ => unreachable!(),
                };
                if is_rre {
                    put(sink, &enc_rre(opcode, rn, rm));
                } else {
                    put(sink, &enc_rr(opcode, rn, rm));
                }
            }
            &Inst::CmpRX { op, rn, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match op {
                    CmpOp::CmpS32 => (Some(0x59), Some(0xe359), Some(0xc6d)), // C(Y), CRL
                    CmpOp::CmpS32Ext16 => (Some(0x49), Some(0xe379), Some(0xc65)), // CH(Y), CHRL
                    CmpOp::CmpS64 => (None, Some(0xe320), Some(0xc68)),       // CG, CGRL
                    CmpOp::CmpS64Ext16 => (None, Some(0xe334), Some(0xc64)),  // CGH, CGHRL
                    CmpOp::CmpS64Ext32 => (None, Some(0xe330), Some(0xc6c)),  // CGF, CGFRL
                    CmpOp::CmpL32 => (Some(0x55), Some(0xe355), Some(0xc6f)), // CL(Y), CLRL
                    CmpOp::CmpL32Ext16 => (None, None, Some(0xc67)),          // CLHRL
                    CmpOp::CmpL64 => (None, Some(0xe321), Some(0xc6a)),       // CLG, CLGRL
                    CmpOp::CmpL64Ext16 => (None, None, Some(0xc66)),          // CLGHRL
                    CmpOp::CmpL64Ext32 => (None, Some(0xe331), Some(0xc6e)),  // CLGF, CLGFRL
                };
                mem_emit(
                    rn, mem, opcode_rx, opcode_rxy, opcode_ril, true, sink, emit_info, state,
                );
            }
            &Inst::CmpRSImm16 { op, rn, imm } => {
                let opcode = match op {
                    CmpOp::CmpS32 => 0xa7e, // CHI
                    CmpOp::CmpS64 => 0xa7f, // CGHI
                    _ => unreachable!(),
                };
                put(sink, &enc_ri_a(opcode, rn, imm as u16));
            }
            &Inst::CmpRSImm32 { op, rn, imm } => {
                let opcode = match op {
                    CmpOp::CmpS32 => 0xc2d, // CFI
                    CmpOp::CmpS64 => 0xc2c, // CGFI
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rn, imm as u32));
            }
            &Inst::CmpRUImm32 { op, rn, imm } => {
                let opcode = match op {
                    CmpOp::CmpL32 => 0xc2f, // CLFI
                    CmpOp::CmpL64 => 0xc2e, // CLGFI
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rn, imm));
            }
            &Inst::CmpTrapRR {
                op,
                rn,
                rm,
                cond,
                trap_code,
            } => {
                let opcode = match op {
                    CmpOp::CmpS32 => 0xb972, // CRT
                    CmpOp::CmpS64 => 0xb960, // CGRT
                    CmpOp::CmpL32 => 0xb973, // CLRT
                    CmpOp::CmpL64 => 0xb961, // CLGRT
                    _ => unreachable!(),
                };
                let srcloc = state.cur_srcloc();
                put_with_trap(
                    sink,
                    &enc_rrf_cde(opcode, rn, rm, cond.bits(), 0),
                    srcloc,
                    trap_code,
                );
            }
            &Inst::CmpTrapRSImm16 {
                op,
                rn,
                imm,
                cond,
                trap_code,
            } => {
                let opcode = match op {
                    CmpOp::CmpS32 => 0xec72, // CIT
                    CmpOp::CmpS64 => 0xec70, // CGIT
                    _ => unreachable!(),
                };
                let srcloc = state.cur_srcloc();
                put_with_trap(
                    sink,
                    &enc_rie_a(opcode, rn, imm as u16, cond.bits()),
                    srcloc,
                    trap_code,
                );
            }
            &Inst::CmpTrapRUImm16 {
                op,
                rn,
                imm,
                cond,
                trap_code,
            } => {
                let opcode = match op {
                    CmpOp::CmpL32 => 0xec73, // CLFIT
                    CmpOp::CmpL64 => 0xec71, // CLGIT
                    _ => unreachable!(),
                };
                let srcloc = state.cur_srcloc();
                put_with_trap(
                    sink,
                    &enc_rie_a(opcode, rn, imm, cond.bits()),
                    srcloc,
                    trap_code,
                );
            }

            &Inst::AtomicRmw {
                alu_op,
                rd,
                rn,
                ref mem,
            } => {
                let opcode = match alu_op {
                    ALUOp::Add32 => 0xebf8,        // LAA
                    ALUOp::Add64 => 0xebe8,        // LAAG
                    ALUOp::AddLogical32 => 0xebfa, // LAAL
                    ALUOp::AddLogical64 => 0xebea, // LAALG
                    ALUOp::And32 => 0xebf4,        // LAN
                    ALUOp::And64 => 0xebe4,        // LANG
                    ALUOp::Orr32 => 0xebf6,        // LAO
                    ALUOp::Orr64 => 0xebe6,        // LAOG
                    ALUOp::Xor32 => 0xebf7,        // LAX
                    ALUOp::Xor64 => 0xebe7,        // LAXG
                    _ => unreachable!(),
                };

                let rd = rd.to_reg();
                mem_rs_emit(
                    rd,
                    rn,
                    mem,
                    None,
                    Some(opcode),
                    true,
                    sink,
                    emit_info,
                    state,
                );
            }
            &Inst::Loop { ref body, cond } => {
                // This sequence is *one* instruction in the vcode, and is expanded only here at
                // emission time, because it requires branching to internal labels.
                let loop_label = sink.get_label();
                let done_label = sink.get_label();

                // Emit label at the start of the loop.
                sink.bind_label(loop_label);

                for inst in (&body).into_iter() {
                    match &inst {
                        // Replace a CondBreak with a branch to done_label.
                        &Inst::CondBreak { cond } => {
                            let inst = Inst::OneWayCondBr {
                                target: done_label,
                                cond: *cond,
                            };
                            inst.emit(sink, emit_info, state);
                        }
                        _ => inst.emit(sink, emit_info, state),
                    };
                }

                let inst = Inst::OneWayCondBr {
                    target: loop_label,
                    cond,
                };
                inst.emit(sink, emit_info, state);

                // Emit label at the end of the loop.
                sink.bind_label(done_label);
            }
            &Inst::CondBreak { .. } => unreachable!(), // Only valid inside a Loop.
            &Inst::AtomicCas32 { rd, rn, ref mem } | &Inst::AtomicCas64 { rd, rn, ref mem } => {
                let (opcode_rs, opcode_rsy) = match self {
                    &Inst::AtomicCas32 { .. } => (Some(0xba), Some(0xeb14)), // CS(Y)
                    &Inst::AtomicCas64 { .. } => (None, Some(0xeb30)),       // CSG
                    _ => unreachable!(),
                };

                let rd = rd.to_reg();
                mem_rs_emit(
                    rd, rn, mem, opcode_rs, opcode_rsy, true, sink, emit_info, state,
                );
            }
            &Inst::Fence => {
                put(sink, &enc_e(0x07e0));
            }

            &Inst::Load32 { rd, ref mem }
            | &Inst::Load32ZExt8 { rd, ref mem }
            | &Inst::Load32SExt8 { rd, ref mem }
            | &Inst::Load32ZExt16 { rd, ref mem }
            | &Inst::Load32SExt16 { rd, ref mem }
            | &Inst::Load64 { rd, ref mem }
            | &Inst::Load64ZExt8 { rd, ref mem }
            | &Inst::Load64SExt8 { rd, ref mem }
            | &Inst::Load64ZExt16 { rd, ref mem }
            | &Inst::Load64SExt16 { rd, ref mem }
            | &Inst::Load64ZExt32 { rd, ref mem }
            | &Inst::Load64SExt32 { rd, ref mem }
            | &Inst::LoadRev16 { rd, ref mem }
            | &Inst::LoadRev32 { rd, ref mem }
            | &Inst::LoadRev64 { rd, ref mem }
            | &Inst::FpuLoad32 { rd, ref mem }
            | &Inst::FpuLoad64 { rd, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match self {
                    &Inst::Load32 { .. } => (Some(0x58), Some(0xe358), Some(0xc4d)), // L(Y), LRL
                    &Inst::Load32ZExt8 { .. } => (None, Some(0xe394), None),         // LLC
                    &Inst::Load32SExt8 { .. } => (None, Some(0xe376), None),         // LB
                    &Inst::Load32ZExt16 { .. } => (None, Some(0xe395), Some(0xc42)), // LLH, LLHRL
                    &Inst::Load32SExt16 { .. } => (Some(0x48), Some(0xe378), Some(0xc45)), // LH(Y), LHRL
                    &Inst::Load64 { .. } => (None, Some(0xe304), Some(0xc48)), // LG, LGRL
                    &Inst::Load64ZExt8 { .. } => (None, Some(0xe390), None),   // LLGC
                    &Inst::Load64SExt8 { .. } => (None, Some(0xe377), None),   // LGB
                    &Inst::Load64ZExt16 { .. } => (None, Some(0xe391), Some(0xc46)), // LLGH, LLGHRL
                    &Inst::Load64SExt16 { .. } => (None, Some(0xe315), Some(0xc44)), // LGH, LGHRL
                    &Inst::Load64ZExt32 { .. } => (None, Some(0xe316), Some(0xc4e)), // LLGF, LLGFRL
                    &Inst::Load64SExt32 { .. } => (None, Some(0xe314), Some(0xc4c)), // LGF, LGFRL
                    &Inst::LoadRev16 { .. } => (None, Some(0xe31f), None),     // LRVH
                    &Inst::LoadRev32 { .. } => (None, Some(0xe31e), None),     // LRV
                    &Inst::LoadRev64 { .. } => (None, Some(0xe30f), None),     // LRVG
                    &Inst::FpuLoad32 { .. } => (Some(0x78), Some(0xed64), None), // LE(Y)
                    &Inst::FpuLoad64 { .. } => (Some(0x68), Some(0xed65), None), // LD(Y)
                    _ => unreachable!(),
                };
                let rd = rd.to_reg();
                mem_emit(
                    rd, mem, opcode_rx, opcode_rxy, opcode_ril, true, sink, emit_info, state,
                );
            }
            &Inst::FpuLoadRev32 { rd, ref mem } | &Inst::FpuLoadRev64 { rd, ref mem } => {
                let opcode = match self {
                    &Inst::FpuLoadRev32 { .. } => 0xe603, // VLEBRF
                    &Inst::FpuLoadRev64 { .. } => 0xe602, // VLEBRG
                    _ => unreachable!(),
                };

                let (mem_insts, mem) = mem_finalize(mem, state, true, false, false, true);
                for inst in mem_insts.into_iter() {
                    inst.emit(sink, emit_info, state);
                }

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && mem.can_trap() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                match &mem {
                    &MemArg::BXD12 {
                        base, index, disp, ..
                    } => {
                        put(
                            sink,
                            &enc_vrx(opcode, rd.to_reg(), base, index, disp.bits(), 0),
                        );
                    }
                    _ => unreachable!(),
                }
            }

            &Inst::Store8 { rd, ref mem }
            | &Inst::Store16 { rd, ref mem }
            | &Inst::Store32 { rd, ref mem }
            | &Inst::Store64 { rd, ref mem }
            | &Inst::StoreRev16 { rd, ref mem }
            | &Inst::StoreRev32 { rd, ref mem }
            | &Inst::StoreRev64 { rd, ref mem }
            | &Inst::FpuStore32 { rd, ref mem }
            | &Inst::FpuStore64 { rd, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match self {
                    &Inst::Store8 { .. } => (Some(0x42), Some(0xe372), None), // STC(Y)
                    &Inst::Store16 { .. } => (Some(0x40), Some(0xe370), Some(0xc47)), // STH(Y), STHRL
                    &Inst::Store32 { .. } => (Some(0x50), Some(0xe350), Some(0xc4f)), // ST(Y), STRL
                    &Inst::Store64 { .. } => (None, Some(0xe324), Some(0xc4b)),       // STG, STGRL
                    &Inst::StoreRev16 { .. } => (None, Some(0xe33f), None),           // STRVH
                    &Inst::StoreRev32 { .. } => (None, Some(0xe33e), None),           // STRV
                    &Inst::StoreRev64 { .. } => (None, Some(0xe32f), None),           // STRVG
                    &Inst::FpuStore32 { .. } => (Some(0x70), Some(0xed66), None),     // STE(Y)
                    &Inst::FpuStore64 { .. } => (Some(0x60), Some(0xed67), None),     // STD(Y)
                    _ => unreachable!(),
                };
                mem_emit(
                    rd, mem, opcode_rx, opcode_rxy, opcode_ril, true, sink, emit_info, state,
                );
            }
            &Inst::StoreImm8 { imm, ref mem } => {
                let opcode_si = 0x92; // MVI
                let opcode_siy = 0xeb52; // MVIY
                mem_imm8_emit(
                    imm, mem, opcode_si, opcode_siy, true, sink, emit_info, state,
                );
            }
            &Inst::StoreImm16 { imm, ref mem }
            | &Inst::StoreImm32SExt16 { imm, ref mem }
            | &Inst::StoreImm64SExt16 { imm, ref mem } => {
                let opcode = match self {
                    &Inst::StoreImm16 { .. } => 0xe544,       // MVHHI
                    &Inst::StoreImm32SExt16 { .. } => 0xe54c, // MVHI
                    &Inst::StoreImm64SExt16 { .. } => 0xe548, // MVGHI
                    _ => unreachable!(),
                };
                mem_imm16_emit(imm, mem, opcode, true, sink, emit_info, state);
            }
            &Inst::FpuStoreRev32 { rd, ref mem } | &Inst::FpuStoreRev64 { rd, ref mem } => {
                let opcode = match self {
                    &Inst::FpuStoreRev32 { .. } => 0xe60b, // VSTEBRF
                    &Inst::FpuStoreRev64 { .. } => 0xe60a, // VSTEBRG
                    _ => unreachable!(),
                };

                let (mem_insts, mem) = mem_finalize(mem, state, true, false, false, true);
                for inst in mem_insts.into_iter() {
                    inst.emit(sink, emit_info, state);
                }

                let srcloc = state.cur_srcloc();
                if srcloc != SourceLoc::default() && mem.can_trap() {
                    sink.add_trap(srcloc, TrapCode::HeapOutOfBounds);
                }

                match &mem {
                    &MemArg::BXD12 {
                        base, index, disp, ..
                    } => {
                        put(sink, &enc_vrx(opcode, rd, base, index, disp.bits(), 0));
                    }
                    _ => unreachable!(),
                }
            }

            &Inst::LoadMultiple64 { rt, rt2, ref mem } => {
                let opcode = 0xeb04; // LMG
                let rt = rt.to_reg();
                let rt2 = rt2.to_reg();
                mem_rs_emit(
                    rt,
                    rt2,
                    &mem,
                    None,
                    Some(opcode),
                    true,
                    sink,
                    emit_info,
                    state,
                );
            }
            &Inst::StoreMultiple64 { rt, rt2, ref mem } => {
                let opcode = 0xeb24; // STMG
                mem_rs_emit(
                    rt,
                    rt2,
                    &mem,
                    None,
                    Some(opcode),
                    true,
                    sink,
                    emit_info,
                    state,
                );
            }

            &Inst::LoadAddr { rd, ref mem } => {
                let opcode_rx = Some(0x41); // LA
                let opcode_rxy = Some(0xe371); // LAY
                let opcode_ril = Some(0xc00); // LARL
                let rd = rd.to_reg();
                mem_emit(
                    rd, mem, opcode_rx, opcode_rxy, opcode_ril, false, sink, emit_info, state,
                );
            }

            &Inst::Mov64 { rd, rm } => {
                let opcode = 0xb904; // LGR
                put(sink, &enc_rre(opcode, rd.to_reg(), rm));
            }
            &Inst::Mov32 { rd, rm } => {
                let opcode = 0x18; // LR
                put(sink, &enc_rr(opcode, rd.to_reg(), rm));
            }
            &Inst::Mov32Imm { rd, imm } => {
                let opcode = 0xc09; // IILF
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm));
            }
            &Inst::Mov32SImm16 { rd, imm } => {
                let opcode = 0xa78; // LHI
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm as u16));
            }
            &Inst::Mov64SImm16 { rd, imm } => {
                let opcode = 0xa79; // LGHI
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm as u16));
            }
            &Inst::Mov64SImm32 { rd, imm } => {
                let opcode = 0xc01; // LGFI
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm as u32));
            }
            &Inst::CMov32 { rd, cond, rm } => {
                let opcode = 0xb9f2; // LOCR
                put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rm, cond.bits(), 0));
            }
            &Inst::CMov64 { rd, cond, rm } => {
                let opcode = 0xb9e2; // LOCGR
                put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rm, cond.bits(), 0));
            }
            &Inst::CMov32SImm16 { rd, cond, imm } => {
                let opcode = 0xec42; // LOCHI
                put(
                    sink,
                    &enc_rie_g(opcode, rd.to_reg(), imm as u16, cond.bits()),
                );
            }
            &Inst::CMov64SImm16 { rd, cond, imm } => {
                let opcode = 0xec46; // LOCGHI
                put(
                    sink,
                    &enc_rie_g(opcode, rd.to_reg(), imm as u16, cond.bits()),
                );
            }
            &Inst::Mov64UImm16Shifted { rd, imm } => {
                let opcode = match imm.shift {
                    0 => 0xa5f, // LLILL
                    1 => 0xa5e, // LLILH
                    2 => 0xa5d, // LLIHL
                    3 => 0xa5c, // LLIHH
                    _ => unreachable!(),
                };
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm.bits));
            }
            &Inst::Mov64UImm32Shifted { rd, imm } => {
                let opcode = match imm.shift {
                    0 => 0xc0f, // LLILF
                    1 => 0xc0e, // LLIHF
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm.bits));
            }
            &Inst::Insert64UImm16Shifted { rd, imm } => {
                let opcode = match imm.shift {
                    0 => 0xa53, // IILL
                    1 => 0xa52, // IILH
                    2 => 0xa51, // IIHL
                    3 => 0xa50, // IIHH
                    _ => unreachable!(),
                };
                put(sink, &enc_ri_a(opcode, rd.to_reg(), imm.bits));
            }
            &Inst::Insert64UImm32Shifted { rd, imm } => {
                let opcode = match imm.shift {
                    0 => 0xc09, // IILF
                    1 => 0xc08, // IIHF
                    _ => unreachable!(),
                };
                put(sink, &enc_ril_a(opcode, rd.to_reg(), imm.bits));
            }
            &Inst::LoadExtNameFar {
                rd,
                ref name,
                offset,
            } => {
                let opcode = 0xa75; // BRAS
                let srcloc = state.cur_srcloc();
                let reg = writable_spilltmp_reg().to_reg();
                put(sink, &enc_ri_b(opcode, reg, 12));
                sink.add_reloc(srcloc, Reloc::Abs8, name, offset);
                if emit_info.flags.emit_all_ones_funcaddrs() {
                    sink.put8(u64::max_value());
                } else {
                    sink.put8(0);
                }
                let inst = Inst::Load64 {
                    rd,
                    mem: MemArg::reg(reg, MemFlags::trusted()),
                };
                inst.emit(sink, emit_info, state);
            }

            &Inst::FpuMove32 { rd, rn } => {
                let opcode = 0x38; // LER
                put(sink, &enc_rr(opcode, rd.to_reg(), rn));
            }
            &Inst::FpuMove64 { rd, rn } => {
                let opcode = 0x28; // LDR
                put(sink, &enc_rr(opcode, rd.to_reg(), rn));
            }
            &Inst::FpuCMov32 { rd, cond, rm } => {
                let opcode = 0xa74; // BCR
                put(sink, &enc_ri_c(opcode, cond.invert().bits(), 4 + 2));
                let opcode = 0x38; // LER
                put(sink, &enc_rr(opcode, rd.to_reg(), rm));
            }
            &Inst::FpuCMov64 { rd, cond, rm } => {
                let opcode = 0xa74; // BCR
                put(sink, &enc_ri_c(opcode, cond.invert().bits(), 4 + 2));
                let opcode = 0x28; // LDR
                put(sink, &enc_rr(opcode, rd.to_reg(), rm));
            }
            &Inst::MovToFpr { rd, rn } => {
                let opcode = 0xb3c1; // LDGR
                put(sink, &enc_rre(opcode, rd.to_reg(), rn));
            }
            &Inst::MovFromFpr { rd, rn } => {
                let opcode = 0xb3cd; // LGDR
                put(sink, &enc_rre(opcode, rd.to_reg(), rn));
            }
            &Inst::LoadFpuConst32 { rd, const_data } => {
                let opcode = 0xa75; // BRAS
                let reg = writable_spilltmp_reg().to_reg();
                put(sink, &enc_ri_b(opcode, reg, 8));
                sink.put4(const_data.swap_bytes());
                let inst = Inst::FpuLoad32 {
                    rd,
                    mem: MemArg::reg(reg, MemFlags::trusted()),
                };
                inst.emit(sink, emit_info, state);
            }
            &Inst::LoadFpuConst64 { rd, const_data } => {
                let opcode = 0xa75; // BRAS
                let reg = writable_spilltmp_reg().to_reg();
                put(sink, &enc_ri_b(opcode, reg, 12));
                sink.put8(const_data.swap_bytes());
                let inst = Inst::FpuLoad64 {
                    rd,
                    mem: MemArg::reg(reg, MemFlags::trusted()),
                };
                inst.emit(sink, emit_info, state);
            }

            &Inst::FpuCopysign { rd, rn, rm } => {
                let opcode = 0xb372; // CPSDR
                put(sink, &enc_rrf_ab(opcode, rd.to_reg(), rn, rm, 0));
            }
            &Inst::FpuRR { fpu_op, rd, rn } => {
                let opcode = match fpu_op {
                    FPUOp1::Abs32 => 0xb300,     // LPEBR
                    FPUOp1::Abs64 => 0xb310,     // LPDBR
                    FPUOp1::Neg32 => 0xb303,     // LCEBR
                    FPUOp1::Neg64 => 0xb313,     // LCDBR
                    FPUOp1::NegAbs32 => 0xb301,  // LNEBR
                    FPUOp1::NegAbs64 => 0xb311,  // LNDBR
                    FPUOp1::Sqrt32 => 0xb314,    // SQEBR
                    FPUOp1::Sqrt64 => 0xb315,    // SQDBR
                    FPUOp1::Cvt32To64 => 0xb304, // LDEBR
                    FPUOp1::Cvt64To32 => 0xb344, // LEDBR
                };
                put(sink, &enc_rre(opcode, rd.to_reg(), rn));
            }
            &Inst::FpuRRR { fpu_op, rd, rm } => {
                let opcode = match fpu_op {
                    FPUOp2::Add32 => 0xb30a, // AEBR
                    FPUOp2::Add64 => 0xb31a, // ADBR
                    FPUOp2::Sub32 => 0xb30b, // SEBR
                    FPUOp2::Sub64 => 0xb31b, // SDBR
                    FPUOp2::Mul32 => 0xb317, // MEEBR
                    FPUOp2::Mul64 => 0xb31c, // MDBR
                    FPUOp2::Div32 => 0xb30d, // DEBR
                    FPUOp2::Div64 => 0xb31d, // DDBR
                    _ => unimplemented!(),
                };
                put(sink, &enc_rre(opcode, rd.to_reg(), rm));
            }
            &Inst::FpuRRRR { fpu_op, rd, rn, rm } => {
                let opcode = match fpu_op {
                    FPUOp3::MAdd32 => 0xb30e, // MAEBR
                    FPUOp3::MAdd64 => 0xb31e, // MADBR
                    FPUOp3::MSub32 => 0xb30f, // MSEBR
                    FPUOp3::MSub64 => 0xb31f, // MSDBR
                };
                put(sink, &enc_rrd(opcode, rd.to_reg(), rm, rn));
            }
            &Inst::FpuToInt { op, rd, rn } => {
                let opcode = match op {
                    FpuToIntOp::F32ToI32 => 0xb398, // CFEBRA
                    FpuToIntOp::F32ToU32 => 0xb39c, // CLFEBR
                    FpuToIntOp::F32ToI64 => 0xb3a8, // CGEBRA
                    FpuToIntOp::F32ToU64 => 0xb3ac, // CLGEBR
                    FpuToIntOp::F64ToI32 => 0xb399, // CFDBRA
                    FpuToIntOp::F64ToU32 => 0xb39d, // CLFDBR
                    FpuToIntOp::F64ToI64 => 0xb3a9, // CGDBRA
                    FpuToIntOp::F64ToU64 => 0xb3ad, // CLGDBR
                };
                put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rn, 5, 0));
            }
            &Inst::IntToFpu { op, rd, rn } => {
                let opcode = match op {
                    IntToFpuOp::I32ToF32 => 0xb394, // CEFBRA
                    IntToFpuOp::U32ToF32 => 0xb390, // CELFBR
                    IntToFpuOp::I64ToF32 => 0xb3a4, // CEGBRA
                    IntToFpuOp::U64ToF32 => 0xb3a0, // CELGBR
                    IntToFpuOp::I32ToF64 => 0xb395, // CDFBRA
                    IntToFpuOp::U32ToF64 => 0xb391, // CDLFBR
                    IntToFpuOp::I64ToF64 => 0xb3a5, // CDGBRA
                    IntToFpuOp::U64ToF64 => 0xb3a1, // CDLGBR
                };
                put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rn, 0, 0));
            }
            &Inst::FpuRound { op, rd, rn } => {
                let (opcode, m3) = match op {
                    FpuRoundMode::Minus32 => (0xb357, 7),   // FIEBR
                    FpuRoundMode::Minus64 => (0xb35f, 7),   // FIDBR
                    FpuRoundMode::Plus32 => (0xb357, 6),    // FIEBR
                    FpuRoundMode::Plus64 => (0xb35f, 6),    // FIDBR
                    FpuRoundMode::Zero32 => (0xb357, 5),    // FIEBR
                    FpuRoundMode::Zero64 => (0xb35f, 5),    // FIDBR
                    FpuRoundMode::Nearest32 => (0xb357, 4), // FIEBR
                    FpuRoundMode::Nearest64 => (0xb35f, 4), // FIDBR
                };
                put(sink, &enc_rrf_cde(opcode, rd.to_reg(), rn, m3, 0));
            }
            &Inst::FpuVecRRR { fpu_op, rd, rn, rm } => {
                let (opcode, m4) = match fpu_op {
                    FPUOp2::Max32 => (0xe7ef, 2), // VFMAX
                    FPUOp2::Max64 => (0xe7ef, 3), // VFMAX
                    FPUOp2::Min32 => (0xe7ee, 2), // VFMIN
                    FPUOp2::Min64 => (0xe7ee, 3), // VFMIN
                    _ => unimplemented!(),
                };
                put(sink, &enc_vrr(opcode, rd.to_reg(), rn, rm, m4, 8, 1));
            }
            &Inst::FpuCmp32 { rn, rm } => {
                let opcode = 0xb309; // CEBR
                put(sink, &enc_rre(opcode, rn, rm));
            }
            &Inst::FpuCmp64 { rn, rm } => {
                let opcode = 0xb319; // CDBR
                put(sink, &enc_rre(opcode, rn, rm));
            }

            &Inst::Call { link, ref info } => {
                let opcode = 0xc05; // BRASL
                let reloc = Reloc::S390xPCRel32Dbl;
                let srcloc = state.cur_srcloc();
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(6), s);
                }
                put_with_reloc(
                    sink,
                    &enc_ril_b(opcode, link.to_reg(), 0),
                    2,
                    srcloc,
                    reloc,
                    &info.dest,
                    0,
                );
                if info.opcode.is_call() {
                    sink.add_call_site(srcloc, info.opcode);
                }
            }
            &Inst::CallInd { link, ref info } => {
                let opcode = 0x0d; // BASR
                let srcloc = state.cur_srcloc();
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(2), s);
                }
                put(sink, &enc_rr(opcode, link.to_reg(), info.rn));
                if info.opcode.is_call() {
                    sink.add_call_site(srcloc, info.opcode);
                }
            }
            &Inst::Ret { link } => {
                let opcode = 0x07; // BCR
                put(sink, &enc_rr(opcode, gpr(15), link));
            }
            &Inst::EpiloguePlaceholder => {
                // Noop; this is just a placeholder for epilogues.
            }
            &Inst::Jump { dest } => {
                let off = sink.cur_offset();
                // Indicate that the jump uses a label, if so, so that a fixup can occur later.
                sink.use_label_at_offset(off, dest, LabelUse::BranchRIL);
                sink.add_uncond_branch(off, off + 6, dest);
                // Emit the jump itself.
                let opcode = 0xc04; // BCRL
                put(sink, &enc_ril_c(opcode, 15, 0));
            }
            &Inst::IndirectBr { rn, .. } => {
                let opcode = 0x07; // BCR
                put(sink, &enc_rr(opcode, gpr(15), rn));
            }
            &Inst::CondBr {
                taken,
                not_taken,
                cond,
            } => {
                let opcode = 0xc04; // BCRL

                // Conditional part first.
                let cond_off = sink.cur_offset();
                sink.use_label_at_offset(cond_off, taken, LabelUse::BranchRIL);
                let inverted = &enc_ril_c(opcode, cond.invert().bits(), 0);
                sink.add_cond_branch(cond_off, cond_off + 6, taken, inverted);
                put(sink, &enc_ril_c(opcode, cond.bits(), 0));

                // Unconditional part next.
                let uncond_off = sink.cur_offset();
                sink.use_label_at_offset(uncond_off, not_taken, LabelUse::BranchRIL);
                sink.add_uncond_branch(uncond_off, uncond_off + 6, not_taken);
                put(sink, &enc_ril_c(opcode, 15, 0));
            }
            &Inst::OneWayCondBr { target, cond } => {
                let opcode = 0xc04; // BCRL
                sink.use_label_at_offset(sink.cur_offset(), target, LabelUse::BranchRIL);
                put(sink, &enc_ril_c(opcode, cond.bits(), 0));
            }
            &Inst::Nop0 => {}
            &Inst::Nop2 => {
                put(sink, &enc_e(0x0707));
            }
            &Inst::Debugtrap => {
                put(sink, &enc_e(0x0001));
            }
            &Inst::Trap { trap_code } => {
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(2), s);
                }
                let srcloc = state.cur_srcloc();
                put_with_trap(sink, &enc_e(0x0000), srcloc, trap_code);
            }
            &Inst::TrapIf { cond, trap_code } => {
                // Branch over trap if condition is false.
                let opcode = 0xa74; // BCR
                put(sink, &enc_ri_c(opcode, cond.invert().bits(), 4 + 2));
                // Now emit the actual trap.
                if let Some(s) = state.take_stack_map() {
                    sink.add_stack_map(StackMapExtent::UpcomingBytes(2), s);
                }
                let srcloc = state.cur_srcloc();
                put_with_trap(sink, &enc_e(0x0000), srcloc, trap_code);
            }
            &Inst::JTSequence { ridx, ref targets } => {
                let table_label = sink.get_label();

                // This sequence is *one* instruction in the vcode, and is expanded only here at
                // emission time, because we cannot allow the regalloc to insert spills/reloads in
                // the middle; we depend on hardcoded PC-rel addressing below.

                // Set temp register to address of jump table.
                let rtmp = writable_spilltmp_reg();
                let inst = Inst::LoadAddr {
                    rd: rtmp,
                    mem: MemArg::Label {
                        target: table_label,
                    },
                };
                inst.emit(sink, emit_info, state);

                // Set temp to target address by adding the value of the jump table entry.
                let inst = Inst::AluRX {
                    alu_op: ALUOp::Add64Ext32,
                    rd: rtmp,
                    mem: MemArg::reg_plus_reg(rtmp.to_reg(), ridx, MemFlags::trusted()),
                };
                inst.emit(sink, emit_info, state);

                // Branch to computed address. (`targets` here is only used for successor queries
                // and is not needed for emission.)
                let inst = Inst::IndirectBr {
                    rn: rtmp.to_reg(),
                    targets: vec![],
                };
                inst.emit(sink, emit_info, state);

                // Emit jump table (table of 32-bit offsets).
                // The first entry is the default target, which is not emitted
                // into the jump table, so we skip it here.  It is only in the
                // list so MachTerminator will see the potential target.
                sink.bind_label(table_label);
                let jt_off = sink.cur_offset();
                for &target in targets.iter().skip(1) {
                    let word_off = sink.cur_offset();
                    let off_into_table = word_off - jt_off;
                    sink.use_label_at_offset(word_off, target, LabelUse::PCRel32);
                    sink.put4(off_into_table.swap_bytes());
                }

                // Lowering produces an EmitIsland before using a JTSequence, so we can safely
                // disable the worst-case-size check in this case.
                start_off = sink.cur_offset();
            }

            &Inst::VirtualSPOffsetAdj { offset } => {
                log::trace!(
                    "virtual sp offset adjusted by {} -> {}",
                    offset,
                    state.virtual_sp_offset + offset
                );
                state.virtual_sp_offset += offset;
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
