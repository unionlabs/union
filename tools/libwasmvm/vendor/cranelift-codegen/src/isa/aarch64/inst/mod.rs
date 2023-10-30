//! This module defines aarch64-specific machine instruction types.

// Some variants are not constructed, but we still want them as options in the future.
#![allow(dead_code)]

use crate::binemit::{Addend, CodeOffset, Reloc};
use crate::ir::types::{
    B1, B128, B16, B32, B64, B8, F32, F64, FFLAGS, I128, I16, I32, I64, I8, I8X16, IFLAGS, R32, R64,
};
use crate::ir::{ExternalName, MemFlags, Opcode, SourceLoc, Type, ValueLabel};
use crate::isa::CallConv;
use crate::machinst::*;
use crate::{settings, CodegenError, CodegenResult};

use regalloc::RegUsageCollector;
use regalloc::{PrettyPrint, RealRegUniverse, Reg, RegClass, SpillSlot, VirtualReg, Writable};

use alloc::vec::Vec;
use core::convert::TryFrom;
use smallvec::{smallvec, SmallVec};
use std::string::{String, ToString};

pub mod regs;
pub use self::regs::*;
pub mod imms;
pub use self::imms::*;
pub mod args;
pub use self::args::*;
pub mod emit;
pub use self::emit::*;
use crate::isa::aarch64::abi::AArch64MachineDeps;

pub mod unwind;

#[cfg(test)]
mod emit_tests;

//=============================================================================
// Instructions (top level): definition

pub use crate::isa::aarch64::lower::isle::generated_code::{
    ALUOp, ALUOp3, AtomicRMWOp, BitOp, FPUOp1, FPUOp2, FPUOp3, FpuRoundMode, FpuToIntOp,
    IntToFpuOp, MInst as Inst, VecALUOp, VecExtendOp, VecLanesOp, VecMisc2, VecPairOp, VecRRLongOp,
    VecRRNarrowOp, VecRRPairLongOp, VecRRRLongOp, VecShiftImmOp,
};

/// A floating-point unit (FPU) operation with two args, a register and an immediate.
#[derive(Copy, Clone, Debug)]
pub enum FPUOpRI {
    /// Unsigned right shift. Rd = Rn << #imm
    UShr32(FPURightShiftImm),
    /// Unsigned right shift. Rd = Rn << #imm
    UShr64(FPURightShiftImm),
    /// Shift left and insert. Rd |= Rn << #imm
    Sli32(FPULeftShiftImm),
    /// Shift left and insert. Rd |= Rn << #imm
    Sli64(FPULeftShiftImm),
}

impl BitOp {
    /// Get the assembly mnemonic for this opcode.
    pub fn op_str(&self) -> &'static str {
        match self {
            BitOp::RBit => "rbit",
            BitOp::Clz => "clz",
            BitOp::Cls => "cls",
        }
    }
}

/// Additional information for (direct) Call instructions, left out of line to lower the size of
/// the Inst enum.
#[derive(Clone, Debug)]
pub struct CallInfo {
    pub dest: ExternalName,
    pub uses: Vec<Reg>,
    pub defs: Vec<Writable<Reg>>,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
}

/// Additional information for CallInd instructions, left out of line to lower the size of the Inst
/// enum.
#[derive(Clone, Debug)]
pub struct CallIndInfo {
    pub rn: Reg,
    pub uses: Vec<Reg>,
    pub defs: Vec<Writable<Reg>>,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
}

/// Additional information for JTSequence instructions, left out of line to lower the size of the Inst
/// enum.
#[derive(Clone, Debug)]
pub struct JTSequenceInfo {
    pub targets: Vec<BranchTarget>,
    pub default_target: BranchTarget,
    pub targets_for_term: Vec<MachLabel>, // needed for MachTerminator.
}

fn count_zero_half_words(mut value: u64, num_half_words: u8) -> usize {
    let mut count = 0;
    for _ in 0..num_half_words {
        if value & 0xffff == 0 {
            count += 1;
        }
        value >>= 16;
    }

    count
}

#[test]
fn inst_size_test() {
    // This test will help with unintentionally growing the size
    // of the Inst enum.
    assert_eq!(32, std::mem::size_of::<Inst>());
}

impl Inst {
    /// Create an instruction that loads a constant, using one of serveral options (MOVZ, MOVN,
    /// logical immediate, or constant pool).
    pub fn load_constant(rd: Writable<Reg>, value: u64) -> SmallVec<[Inst; 4]> {
        // NB: this is duplicated in `lower/isle.rs` and `inst.isle` right now,
        // if modifications are made here before this is deleted after moving to
        // ISLE then those locations should be updated as well.

        if let Some(imm) = MoveWideConst::maybe_from_u64(value) {
            // 16-bit immediate (shifted by 0, 16, 32 or 48 bits) in MOVZ
            smallvec![Inst::MovZ {
                rd,
                imm,
                size: OperandSize::Size64
            }]
        } else if let Some(imm) = MoveWideConst::maybe_from_u64(!value) {
            // 16-bit immediate (shifted by 0, 16, 32 or 48 bits) in MOVN
            smallvec![Inst::MovN {
                rd,
                imm,
                size: OperandSize::Size64
            }]
        } else if let Some(imml) = ImmLogic::maybe_from_u64(value, I64) {
            // Weird logical-instruction immediate in ORI using zero register
            smallvec![Inst::AluRRImmLogic {
                alu_op: ALUOp::Orr,
                size: OperandSize::Size64,
                rd,
                rn: zero_reg(),
                imml,
            }]
        } else {
            let mut insts = smallvec![];

            // If the top 32 bits are zero, use 32-bit `mov` operations.
            let (num_half_words, size, negated) = if value >> 32 == 0 {
                (2, OperandSize::Size32, (!value << 32) >> 32)
            } else {
                (4, OperandSize::Size64, !value)
            };
            // If the number of 0xffff half words is greater than the number of 0x0000 half words
            // it is more efficient to use `movn` for the first instruction.
            let first_is_inverted = count_zero_half_words(negated, num_half_words)
                > count_zero_half_words(value, num_half_words);
            // Either 0xffff or 0x0000 half words can be skipped, depending on the first
            // instruction used.
            let ignored_halfword = if first_is_inverted { 0xffff } else { 0 };
            let mut first_mov_emitted = false;

            for i in 0..num_half_words {
                let imm16 = (value >> (16 * i)) & 0xffff;
                if imm16 != ignored_halfword {
                    if !first_mov_emitted {
                        first_mov_emitted = true;
                        if first_is_inverted {
                            let imm =
                                MoveWideConst::maybe_with_shift(((!imm16) & 0xffff) as u16, i * 16)
                                    .unwrap();
                            insts.push(Inst::MovN { rd, imm, size });
                        } else {
                            let imm =
                                MoveWideConst::maybe_with_shift(imm16 as u16, i * 16).unwrap();
                            insts.push(Inst::MovZ { rd, imm, size });
                        }
                    } else {
                        let imm = MoveWideConst::maybe_with_shift(imm16 as u16, i * 16).unwrap();
                        insts.push(Inst::MovK { rd, imm, size });
                    }
                }
            }

            assert!(first_mov_emitted);

            insts
        }
    }

    /// Create instructions that load a 128-bit constant.
    pub fn load_constant128(to_regs: ValueRegs<Writable<Reg>>, value: u128) -> SmallVec<[Inst; 4]> {
        assert_eq!(to_regs.len(), 2, "Expected to load i128 into two registers");

        let lower = value as u64;
        let upper = (value >> 64) as u64;

        let lower_reg = to_regs.regs()[0];
        let upper_reg = to_regs.regs()[1];

        let mut load_ins = Inst::load_constant(lower_reg, lower);
        let load_upper = Inst::load_constant(upper_reg, upper);

        load_ins.extend(load_upper.into_iter());
        load_ins
    }

    /// Create instructions that load a 32-bit floating-point constant.
    pub fn load_fp_constant32<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        const_data: u32,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        // Note that we must make sure that all bits outside the lowest 32 are set to 0
        // because this function is also used to load wider constants (that have zeros
        // in their most significant bits).
        if const_data == 0 {
            smallvec![Inst::VecDupImm {
                rd,
                imm: ASIMDMovModImm::zero(ScalarSize::Size32),
                invert: false,
                size: VectorSize::Size32x2,
            }]
        } else if let Some(imm) =
            ASIMDFPModImm::maybe_from_u64(const_data.into(), ScalarSize::Size32)
        {
            smallvec![Inst::FpuMoveFPImm {
                rd,
                imm,
                size: ScalarSize::Size32,
            }]
        } else {
            let tmp = alloc_tmp(I32);
            let mut insts = Inst::load_constant(tmp, const_data as u64);

            insts.push(Inst::MovToFpu {
                rd,
                rn: tmp.to_reg(),
                size: ScalarSize::Size32,
            });

            insts
        }
    }

    /// Create instructions that load a 64-bit floating-point constant.
    pub fn load_fp_constant64<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        const_data: u64,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        // Note that we must make sure that all bits outside the lowest 64 are set to 0
        // because this function is also used to load wider constants (that have zeros
        // in their most significant bits).
        // TODO: Treat as half of a 128 bit vector and consider replicated patterns.
        // Scalar MOVI might also be an option.
        if const_data == 0 {
            smallvec![Inst::VecDupImm {
                rd,
                imm: ASIMDMovModImm::zero(ScalarSize::Size32),
                invert: false,
                size: VectorSize::Size32x2,
            }]
        } else if let Some(imm) = ASIMDFPModImm::maybe_from_u64(const_data, ScalarSize::Size64) {
            smallvec![Inst::FpuMoveFPImm {
                rd,
                imm,
                size: ScalarSize::Size64,
            }]
        } else if let Ok(const_data) = u32::try_from(const_data) {
            Inst::load_fp_constant32(rd, const_data, alloc_tmp)
        } else if const_data & (u32::MAX as u64) == 0 {
            let tmp = alloc_tmp(I64);
            let mut insts = Inst::load_constant(tmp, const_data);

            insts.push(Inst::MovToFpu {
                rd,
                rn: tmp.to_reg(),
                size: ScalarSize::Size64,
            });

            insts
        } else {
            smallvec![Inst::LoadFpuConst64 { rd, const_data }]
        }
    }

    /// Create instructions that load a 128-bit vector constant.
    pub fn load_fp_constant128<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        const_data: u128,
        alloc_tmp: F,
    ) -> SmallVec<[Inst; 5]> {
        if let Ok(const_data) = u64::try_from(const_data) {
            SmallVec::from(&Inst::load_fp_constant64(rd, const_data, alloc_tmp)[..])
        } else if let Some((pattern, size)) =
            Inst::get_replicated_vector_pattern(const_data, ScalarSize::Size64)
        {
            Inst::load_replicated_vector_pattern(
                rd,
                pattern,
                VectorSize::from_lane_size(size, true),
                alloc_tmp,
            )
        } else {
            smallvec![Inst::LoadFpuConst128 { rd, const_data }]
        }
    }

    /// Determine whether a 128-bit constant represents a vector consisting of elements with
    /// the same value.
    pub fn get_replicated_vector_pattern(
        value: u128,
        size: ScalarSize,
    ) -> Option<(u64, ScalarSize)> {
        let (mask, shift, next_size) = match size {
            ScalarSize::Size8 => (u8::MAX as u128, 8, ScalarSize::Size128),
            ScalarSize::Size16 => (u16::MAX as u128, 16, ScalarSize::Size8),
            ScalarSize::Size32 => (u32::MAX as u128, 32, ScalarSize::Size16),
            ScalarSize::Size64 => (u64::MAX as u128, 64, ScalarSize::Size32),
            _ => return None,
        };
        let mut r = None;
        let v = value & mask;

        if (value >> shift) & mask == v {
            r = Inst::get_replicated_vector_pattern(v, next_size);

            if r.is_none() {
                r = Some((v as u64, size));
            }
        }

        r
    }

    /// Create instructions that load a vector constant consisting of elements with
    /// the same value.
    pub fn load_replicated_vector_pattern<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        pattern: u64,
        size: VectorSize,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 5]> {
        let lane_size = size.lane_size();
        let widen_32_bit_pattern = |pattern, lane_size| {
            if lane_size == ScalarSize::Size32 {
                let pattern = pattern as u32 as u64;

                ASIMDMovModImm::maybe_from_u64(pattern | (pattern << 32), ScalarSize::Size64)
            } else {
                None
            }
        };

        if let Some(imm) = ASIMDMovModImm::maybe_from_u64(pattern, lane_size) {
            smallvec![Inst::VecDupImm {
                rd,
                imm,
                invert: false,
                size
            }]
        } else if let Some(imm) = ASIMDMovModImm::maybe_from_u64(!pattern, lane_size) {
            debug_assert_ne!(lane_size, ScalarSize::Size8);
            debug_assert_ne!(lane_size, ScalarSize::Size64);

            smallvec![Inst::VecDupImm {
                rd,
                imm,
                invert: true,
                size
            }]
        } else if let Some(imm) = widen_32_bit_pattern(pattern, lane_size) {
            let mut insts = smallvec![Inst::VecDupImm {
                rd,
                imm,
                invert: false,
                size: VectorSize::Size64x2,
            }];

            // TODO: Implement support for 64-bit scalar MOVI; we zero-extend the
            // lower 64 bits instead.
            if !size.is_128bits() {
                insts.push(Inst::FpuExtend {
                    rd,
                    rn: rd.to_reg(),
                    size: ScalarSize::Size64,
                });
            }

            insts
        } else if let Some(imm) = ASIMDFPModImm::maybe_from_u64(pattern, lane_size) {
            smallvec![Inst::VecDupFPImm { rd, imm, size }]
        } else {
            let tmp = alloc_tmp(I64);
            let mut insts = SmallVec::from(&Inst::load_constant(tmp, pattern)[..]);

            insts.push(Inst::VecDup {
                rd,
                rn: tmp.to_reg(),
                size,
            });

            insts
        }
    }

    /// Generic constructor for a load (zero-extending where appropriate).
    pub fn gen_load(into_reg: Writable<Reg>, mem: AMode, ty: Type, flags: MemFlags) -> Inst {
        match ty {
            B1 | B8 | I8 => Inst::ULoad8 {
                rd: into_reg,
                mem,
                flags,
            },
            B16 | I16 => Inst::ULoad16 {
                rd: into_reg,
                mem,
                flags,
            },
            B32 | I32 | R32 => Inst::ULoad32 {
                rd: into_reg,
                mem,
                flags,
            },
            B64 | I64 | R64 => Inst::ULoad64 {
                rd: into_reg,
                mem,
                flags,
            },
            F32 => Inst::FpuLoad32 {
                rd: into_reg,
                mem,
                flags,
            },
            F64 => Inst::FpuLoad64 {
                rd: into_reg,
                mem,
                flags,
            },
            _ => {
                if ty.is_vector() {
                    let bits = ty_bits(ty);
                    let rd = into_reg;

                    if bits == 128 {
                        Inst::FpuLoad128 { rd, mem, flags }
                    } else {
                        assert_eq!(bits, 64);
                        Inst::FpuLoad64 { rd, mem, flags }
                    }
                } else {
                    unimplemented!("gen_load({})", ty);
                }
            }
        }
    }

    /// Generic constructor for a store.
    pub fn gen_store(mem: AMode, from_reg: Reg, ty: Type, flags: MemFlags) -> Inst {
        match ty {
            B1 | B8 | I8 => Inst::Store8 {
                rd: from_reg,
                mem,
                flags,
            },
            B16 | I16 => Inst::Store16 {
                rd: from_reg,
                mem,
                flags,
            },
            B32 | I32 | R32 => Inst::Store32 {
                rd: from_reg,
                mem,
                flags,
            },
            B64 | I64 | R64 => Inst::Store64 {
                rd: from_reg,
                mem,
                flags,
            },
            F32 => Inst::FpuStore32 {
                rd: from_reg,
                mem,
                flags,
            },
            F64 => Inst::FpuStore64 {
                rd: from_reg,
                mem,
                flags,
            },
            _ => {
                if ty.is_vector() {
                    let bits = ty_bits(ty);
                    let rd = from_reg;

                    if bits == 128 {
                        Inst::FpuStore128 { rd, mem, flags }
                    } else {
                        assert_eq!(bits, 64);
                        Inst::FpuStore64 { rd, mem, flags }
                    }
                } else {
                    unimplemented!("gen_store({})", ty);
                }
            }
        }
    }

    /// Generate a LoadAddr instruction (load address of an amode into
    /// register). Elides when possible (when amode is just a register). Returns
    /// destination register: either `rd` or a register directly from the amode.
    pub fn gen_load_addr(rd: Writable<Reg>, mem: AMode) -> (Reg, Option<Inst>) {
        if let Some(r) = mem.is_reg() {
            (r, None)
        } else {
            (rd.to_reg(), Some(Inst::LoadAddr { rd, mem }))
        }
    }
}

//=============================================================================
// Instructions: get_regs

fn memarg_regs(memarg: &AMode, collector: &mut RegUsageCollector) {
    match memarg {
        &AMode::Unscaled(reg, ..) | &AMode::UnsignedOffset(reg, ..) => {
            collector.add_use(reg);
        }
        &AMode::RegReg(r1, r2, ..)
        | &AMode::RegScaled(r1, r2, ..)
        | &AMode::RegScaledExtended(r1, r2, ..)
        | &AMode::RegExtended(r1, r2, ..) => {
            collector.add_use(r1);
            collector.add_use(r2);
        }
        &AMode::Label(..) => {}
        &AMode::PreIndexed(reg, ..) | &AMode::PostIndexed(reg, ..) => {
            collector.add_mod(reg);
        }
        &AMode::FPOffset(..) => {
            collector.add_use(fp_reg());
        }
        &AMode::SPOffset(..) | &AMode::NominalSPOffset(..) => {
            collector.add_use(stack_reg());
        }
        &AMode::RegOffset(r, ..) => {
            collector.add_use(r);
        }
    }
}

fn pairmemarg_regs(pairmemarg: &PairAMode, collector: &mut RegUsageCollector) {
    match pairmemarg {
        &PairAMode::SignedOffset(reg, ..) => {
            collector.add_use(reg);
        }
        &PairAMode::PreIndexed(reg, ..) | &PairAMode::PostIndexed(reg, ..) => {
            collector.add_mod(reg);
        }
    }
}

fn aarch64_get_regs(inst: &Inst, collector: &mut RegUsageCollector) {
    match inst {
        &Inst::AluRRR { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::AluRRRR { rd, rn, rm, ra, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
            collector.add_use(ra);
        }
        &Inst::AluRRImm12 { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::AluRRImmLogic { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::AluRRImmShift { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::AluRRRShift { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::AluRRRExtend { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::BitRR { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::ULoad8 { rd, ref mem, .. }
        | &Inst::SLoad8 { rd, ref mem, .. }
        | &Inst::ULoad16 { rd, ref mem, .. }
        | &Inst::SLoad16 { rd, ref mem, .. }
        | &Inst::ULoad32 { rd, ref mem, .. }
        | &Inst::SLoad32 { rd, ref mem, .. }
        | &Inst::ULoad64 { rd, ref mem, .. } => {
            collector.add_def(rd);
            memarg_regs(mem, collector);
        }
        &Inst::Store8 { rd, ref mem, .. }
        | &Inst::Store16 { rd, ref mem, .. }
        | &Inst::Store32 { rd, ref mem, .. }
        | &Inst::Store64 { rd, ref mem, .. } => {
            collector.add_use(rd);
            memarg_regs(mem, collector);
        }
        &Inst::StoreP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_use(rt);
            collector.add_use(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::LoadP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_def(rt);
            collector.add_def(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::Mov64 { rd, rm } => {
            collector.add_def(rd);
            collector.add_use(rm);
        }
        &Inst::Mov32 { rd, rm } => {
            collector.add_def(rd);
            collector.add_use(rm);
        }
        &Inst::MovZ { rd, .. } | &Inst::MovN { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::MovK { rd, .. } => {
            collector.add_mod(rd);
        }
        &Inst::CSel { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::CSet { rd, .. } | &Inst::CSetm { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::CCmpImm { rn, .. } => {
            collector.add_use(rn);
        }
        &Inst::AtomicRMWLoop { op, .. } => {
            collector.add_use(xreg(25));
            collector.add_use(xreg(26));
            collector.add_def(writable_xreg(24));
            collector.add_def(writable_xreg(27));
            if op != AtomicRmwOp::Xchg {
                collector.add_def(writable_xreg(28));
            }
        }
        &Inst::AtomicRMW { rs, rt, rn, .. } => {
            collector.add_use(rs);
            collector.add_def(rt);
            collector.add_use(rn);
        }
        &Inst::AtomicCAS { rs, rt, rn, .. } => {
            collector.add_mod(rs);
            collector.add_use(rt);
            collector.add_use(rn);
        }
        &Inst::AtomicCASLoop { .. } => {
            collector.add_use(xreg(25));
            collector.add_use(xreg(26));
            collector.add_use(xreg(28));
            collector.add_def(writable_xreg(24));
            collector.add_def(writable_xreg(27));
        }
        &Inst::LoadAcquire { rt, rn, .. } => {
            collector.add_use(rn);
            collector.add_def(rt);
        }
        &Inst::StoreRelease { rt, rn, .. } => {
            collector.add_use(rn);
            collector.add_use(rt);
        }
        &Inst::Fence {} => {}
        &Inst::FpuMove64 { rd, rn } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuMove128 { rd, rn } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuMoveFromVec { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuExtend { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuRR { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuRRR { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::FpuRRI { fpu_op, rd, rn, .. } => {
            match fpu_op {
                FPUOpRI::UShr32(..) | FPUOpRI::UShr64(..) => collector.add_def(rd),
                FPUOpRI::Sli32(..) | FPUOpRI::Sli64(..) => collector.add_mod(rd),
            }
            collector.add_use(rn);
        }
        &Inst::FpuRRRR { rd, rn, rm, ra, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
            collector.add_use(ra);
        }
        &Inst::VecMisc { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }

        &Inst::VecLanes { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecShiftImm { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecExtract { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::VecTbl {
            rd,
            rn,
            rm,
            is_extension,
        } => {
            collector.add_use(rn);
            collector.add_use(rm);

            if is_extension {
                collector.add_mod(rd);
            } else {
                collector.add_def(rd);
            }
        }
        &Inst::VecTbl2 {
            rd,
            rn,
            rn2,
            rm,
            is_extension,
        } => {
            collector.add_use(rn);
            collector.add_use(rn2);
            collector.add_use(rm);

            if is_extension {
                collector.add_mod(rd);
            } else {
                collector.add_def(rd);
            }
        }
        &Inst::VecLoadReplicate { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecCSel { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::FpuCmp32 { rn, rm } | &Inst::FpuCmp64 { rn, rm } => {
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::FpuLoad32 { rd, ref mem, .. } => {
            collector.add_def(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuLoad64 { rd, ref mem, .. } => {
            collector.add_def(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuLoad128 { rd, ref mem, .. } => {
            collector.add_def(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuStore32 { rd, ref mem, .. } => {
            collector.add_use(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuStore64 { rd, ref mem, .. } => {
            collector.add_use(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuStore128 { rd, ref mem, .. } => {
            collector.add_use(rd);
            memarg_regs(mem, collector);
        }
        &Inst::FpuLoadP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_def(rt);
            collector.add_def(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::FpuStoreP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_use(rt);
            collector.add_use(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::FpuLoadP128 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_def(rt);
            collector.add_def(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::FpuStoreP128 {
            rt, rt2, ref mem, ..
        } => {
            collector.add_use(rt);
            collector.add_use(rt2);
            pairmemarg_regs(mem, collector);
        }
        &Inst::LoadFpuConst64 { rd, .. } | &Inst::LoadFpuConst128 { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::FpuToInt { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::IntToFpu { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuCSel32 { rd, rn, rm, .. } | &Inst::FpuCSel64 { rd, rn, rm, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::FpuRound { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::MovToFpu { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::FpuMoveFPImm { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::MovToVec { rd, rn, .. } => {
            collector.add_mod(rd);
            collector.add_use(rn);
        }
        &Inst::MovFromVec { rd, rn, .. } | &Inst::MovFromVecSigned { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecDup { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecDupFromFpu { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecDupFPImm { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::VecDupImm { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::VecExtend { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecMovElement { rd, rn, .. } => {
            collector.add_mod(rd);
            collector.add_use(rn);
        }
        &Inst::VecRRLong { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecRRNarrow {
            rd, rn, high_half, ..
        } => {
            collector.add_use(rn);

            if high_half {
                collector.add_mod(rd);
            } else {
                collector.add_def(rd);
            }
        }
        &Inst::VecRRPair { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecRRRLong {
            alu_op, rd, rn, rm, ..
        } => {
            match alu_op {
                VecRRRLongOp::Umlal8 | VecRRRLongOp::Umlal16 | VecRRRLongOp::Umlal32 => {
                    collector.add_mod(rd)
                }
                _ => collector.add_def(rd),
            };
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::VecRRPairLong { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::VecRRR {
            alu_op, rd, rn, rm, ..
        } => {
            if alu_op == VecALUOp::Bsl {
                collector.add_mod(rd);
            } else {
                collector.add_def(rd);
            }
            collector.add_use(rn);
            collector.add_use(rm);
        }
        &Inst::MovToNZCV { rn } => {
            collector.add_use(rn);
        }
        &Inst::MovFromNZCV { rd } => {
            collector.add_def(rd);
        }
        &Inst::Extend { rd, rn, .. } => {
            collector.add_def(rd);
            collector.add_use(rn);
        }
        &Inst::Jump { .. } | &Inst::Ret | &Inst::EpiloguePlaceholder => {}
        &Inst::Call { ref info, .. } => {
            collector.add_uses(&*info.uses);
            collector.add_defs(&*info.defs);
        }
        &Inst::CallInd { ref info, .. } => {
            collector.add_uses(&*info.uses);
            collector.add_defs(&*info.defs);
            collector.add_use(info.rn);
        }
        &Inst::CondBr { ref kind, .. } => match kind {
            CondBrKind::Zero(rt) | CondBrKind::NotZero(rt) => {
                collector.add_use(*rt);
            }
            CondBrKind::Cond(_) => {}
        },
        &Inst::IndirectBr { rn, .. } => {
            collector.add_use(rn);
        }
        &Inst::Nop0 | Inst::Nop4 => {}
        &Inst::Brk => {}
        &Inst::Udf { .. } => {}
        &Inst::TrapIf { ref kind, .. } => match kind {
            CondBrKind::Zero(rt) | CondBrKind::NotZero(rt) => {
                collector.add_use(*rt);
            }
            CondBrKind::Cond(_) => {}
        },
        &Inst::Adr { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::Word4 { .. } | &Inst::Word8 { .. } => {}
        &Inst::JTSequence {
            ridx, rtmp1, rtmp2, ..
        } => {
            collector.add_use(ridx);
            collector.add_def(rtmp1);
            collector.add_def(rtmp2);
        }
        &Inst::LoadExtName { rd, .. } => {
            collector.add_def(rd);
        }
        &Inst::LoadAddr { rd, ref mem } => {
            collector.add_def(rd);
            memarg_regs(mem, collector);
        }
        &Inst::VirtualSPOffsetAdj { .. } => {}
        &Inst::ValueLabelMarker { reg, .. } => {
            collector.add_use(reg);
        }

        &Inst::ElfTlsGetAddr { .. } => {
            for reg in AArch64MachineDeps::get_regs_clobbered_by_call(CallConv::SystemV) {
                collector.add_def(reg);
            }
        }
        &Inst::Unwind { .. } => {}
        &Inst::EmitIsland { .. } => {}
    }
}

//=============================================================================
// Instructions: map_regs

pub fn aarch64_map_regs<RM: RegMapper>(inst: &mut Inst, mapper: &RM) {
    fn map_mem<RM: RegMapper>(m: &RM, mem: &mut AMode) {
        // N.B.: we take only the pre-map here, but this is OK because the
        // only addressing modes that update registers (pre/post-increment on
        // AArch64) both read and write registers, so they are "mods" rather
        // than "defs", so must be the same in both the pre- and post-map.
        match mem {
            &mut AMode::Unscaled(ref mut reg, ..) => m.map_use(reg),
            &mut AMode::UnsignedOffset(ref mut reg, ..) => m.map_use(reg),
            &mut AMode::RegReg(ref mut r1, ref mut r2)
            | &mut AMode::RegScaled(ref mut r1, ref mut r2, ..)
            | &mut AMode::RegScaledExtended(ref mut r1, ref mut r2, ..)
            | &mut AMode::RegExtended(ref mut r1, ref mut r2, ..) => {
                m.map_use(r1);
                m.map_use(r2);
            }
            &mut AMode::Label(..) => {}
            &mut AMode::PreIndexed(ref mut r, ..) => m.map_mod(r),
            &mut AMode::PostIndexed(ref mut r, ..) => m.map_mod(r),
            &mut AMode::FPOffset(..)
            | &mut AMode::SPOffset(..)
            | &mut AMode::NominalSPOffset(..) => {}
            &mut AMode::RegOffset(ref mut r, ..) => m.map_use(r),
        };
    }

    fn map_pairmem<RM: RegMapper>(m: &RM, mem: &mut PairAMode) {
        match mem {
            &mut PairAMode::SignedOffset(ref mut reg, ..) => m.map_use(reg),
            &mut PairAMode::PreIndexed(ref mut reg, ..) => m.map_def(reg),
            &mut PairAMode::PostIndexed(ref mut reg, ..) => m.map_def(reg),
        }
    }

    fn map_br<RM: RegMapper>(m: &RM, br: &mut CondBrKind) {
        match br {
            &mut CondBrKind::Zero(ref mut reg) => m.map_use(reg),
            &mut CondBrKind::NotZero(ref mut reg) => m.map_use(reg),
            &mut CondBrKind::Cond(..) => {}
        };
    }

    match inst {
        &mut Inst::AluRRR {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::AluRRRR {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ref mut ra,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
            mapper.map_use(ra);
        }
        &mut Inst::AluRRImm12 {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::AluRRImmLogic {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::AluRRImmShift {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::AluRRRShift {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::AluRRRExtend {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::BitRR {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::ULoad8 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::SLoad8 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::ULoad16 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::SLoad16 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::ULoad32 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::SLoad32 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }

        &mut Inst::ULoad64 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::Store8 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::Store16 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::Store32 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::Store64 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }

        &mut Inst::StoreP64 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rt);
            mapper.map_use(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::LoadP64 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rt);
            mapper.map_def(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::Mov64 {
            ref mut rd,
            ref mut rm,
        } => {
            mapper.map_def(rd);
            mapper.map_use(rm);
        }
        &mut Inst::Mov32 {
            ref mut rd,
            ref mut rm,
        } => {
            mapper.map_def(rd);
            mapper.map_use(rm);
        }
        &mut Inst::MovZ { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::MovN { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::MovK { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::CSel {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::CSet { ref mut rd, .. } | &mut Inst::CSetm { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::CCmpImm { ref mut rn, .. } => {
            mapper.map_use(rn);
        }
        &mut Inst::AtomicRMWLoop { .. } => {
            // There are no vregs to map in this insn.
        }
        &mut Inst::AtomicRMW {
            ref mut rs,
            ref mut rt,
            ref mut rn,
            ..
        } => {
            mapper.map_use(rs);
            mapper.map_def(rt);
            mapper.map_use(rn);
        }
        &mut Inst::AtomicCAS {
            ref mut rs,
            ref mut rt,
            ref mut rn,
            ..
        } => {
            mapper.map_mod(rs);
            mapper.map_use(rt);
            mapper.map_use(rn);
        }
        &mut Inst::AtomicCASLoop { .. } => {
            // There are no vregs to map in this insn.
        }
        &mut Inst::LoadAcquire {
            ref mut rt,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rt);
            mapper.map_use(rn);
        }
        &mut Inst::StoreRelease {
            ref mut rt,
            ref mut rn,
            ..
        } => {
            mapper.map_use(rt);
            mapper.map_use(rn);
        }
        &mut Inst::Fence {} => {}
        &mut Inst::FpuMove64 {
            ref mut rd,
            ref mut rn,
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuMove128 {
            ref mut rd,
            ref mut rn,
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuMoveFromVec {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuExtend {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuRR {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuRRR {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuRRI {
            fpu_op,
            ref mut rd,
            ref mut rn,
            ..
        } => {
            match fpu_op {
                FPUOpRI::UShr32(..) | FPUOpRI::UShr64(..) => mapper.map_def(rd),
                FPUOpRI::Sli32(..) | FPUOpRI::Sli64(..) => mapper.map_mod(rd),
            }
            mapper.map_use(rn);
        }
        &mut Inst::FpuRRRR {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ref mut ra,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
            mapper.map_use(ra);
        }
        &mut Inst::VecMisc {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecLanes {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecShiftImm {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecExtract {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::VecTbl {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            is_extension,
        } => {
            mapper.map_use(rn);
            mapper.map_use(rm);

            if is_extension {
                mapper.map_mod(rd);
            } else {
                mapper.map_def(rd);
            }
        }
        &mut Inst::VecTbl2 {
            ref mut rd,
            ref mut rn,
            ref mut rn2,
            ref mut rm,
            is_extension,
        } => {
            mapper.map_use(rn);
            mapper.map_use(rn2);
            mapper.map_use(rm);

            if is_extension {
                mapper.map_mod(rd);
            } else {
                mapper.map_def(rd);
            }
        }
        &mut Inst::VecLoadReplicate {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecCSel {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuCmp32 {
            ref mut rn,
            ref mut rm,
        } => {
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuCmp64 {
            ref mut rn,
            ref mut rm,
        } => {
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuLoad32 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuLoad64 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuLoad128 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuStore32 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuStore64 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuStore128 {
            ref mut rd,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::FpuLoadP64 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rt);
            mapper.map_def(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::FpuStoreP64 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rt);
            mapper.map_use(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::FpuLoadP128 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_def(rt);
            mapper.map_def(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::FpuStoreP128 {
            ref mut rt,
            ref mut rt2,
            ref mut mem,
            ..
        } => {
            mapper.map_use(rt);
            mapper.map_use(rt2);
            map_pairmem(mapper, mem);
        }
        &mut Inst::LoadFpuConst64 { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::LoadFpuConst128 { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::FpuToInt {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::IntToFpu {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuCSel32 {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuCSel64 {
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::FpuRound {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::MovToFpu {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::FpuMoveFPImm { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::MovToVec {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_mod(rd);
            mapper.map_use(rn);
        }
        &mut Inst::MovFromVec {
            ref mut rd,
            ref mut rn,
            ..
        }
        | &mut Inst::MovFromVecSigned {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecDup {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecDupFromFpu {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecDupFPImm { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::VecDupImm { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::VecExtend {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecMovElement {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_mod(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecRRLong {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecRRNarrow {
            ref mut rd,
            ref mut rn,
            high_half,
            ..
        } => {
            mapper.map_use(rn);

            if high_half {
                mapper.map_mod(rd);
            } else {
                mapper.map_def(rd);
            }
        }
        &mut Inst::VecRRPair {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecRRRLong {
            alu_op,
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            match alu_op {
                VecRRRLongOp::Umlal8 | VecRRRLongOp::Umlal16 | VecRRRLongOp::Umlal32 => {
                    mapper.map_mod(rd)
                }
                _ => mapper.map_def(rd),
            };
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::VecRRPairLong {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::VecRRR {
            alu_op,
            ref mut rd,
            ref mut rn,
            ref mut rm,
            ..
        } => {
            if alu_op == VecALUOp::Bsl {
                mapper.map_mod(rd);
            } else {
                mapper.map_def(rd);
            }
            mapper.map_use(rn);
            mapper.map_use(rm);
        }
        &mut Inst::MovToNZCV { ref mut rn } => {
            mapper.map_use(rn);
        }
        &mut Inst::MovFromNZCV { ref mut rd } => {
            mapper.map_def(rd);
        }
        &mut Inst::Extend {
            ref mut rd,
            ref mut rn,
            ..
        } => {
            mapper.map_def(rd);
            mapper.map_use(rn);
        }
        &mut Inst::Jump { .. } => {}
        &mut Inst::Call { ref mut info } => {
            for r in info.uses.iter_mut() {
                mapper.map_use(r);
            }
            for r in info.defs.iter_mut() {
                mapper.map_def(r);
            }
        }
        &mut Inst::Ret | &mut Inst::EpiloguePlaceholder => {}
        &mut Inst::CallInd { ref mut info, .. } => {
            for r in info.uses.iter_mut() {
                mapper.map_use(r);
            }
            for r in info.defs.iter_mut() {
                mapper.map_def(r);
            }
            mapper.map_use(&mut info.rn);
        }
        &mut Inst::CondBr { ref mut kind, .. } => {
            map_br(mapper, kind);
        }
        &mut Inst::IndirectBr { ref mut rn, .. } => {
            mapper.map_use(rn);
        }
        &mut Inst::Nop0 | &mut Inst::Nop4 | &mut Inst::Brk | &mut Inst::Udf { .. } => {}
        &mut Inst::TrapIf { ref mut kind, .. } => {
            map_br(mapper, kind);
        }
        &mut Inst::Adr { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::Word4 { .. } | &mut Inst::Word8 { .. } => {}
        &mut Inst::JTSequence {
            ref mut ridx,
            ref mut rtmp1,
            ref mut rtmp2,
            ..
        } => {
            mapper.map_use(ridx);
            mapper.map_def(rtmp1);
            mapper.map_def(rtmp2);
        }
        &mut Inst::LoadExtName { ref mut rd, .. } => {
            mapper.map_def(rd);
        }
        &mut Inst::LoadAddr {
            ref mut rd,
            ref mut mem,
        } => {
            mapper.map_def(rd);
            map_mem(mapper, mem);
        }
        &mut Inst::VirtualSPOffsetAdj { .. } => {}
        &mut Inst::EmitIsland { .. } => {}
        &mut Inst::ElfTlsGetAddr { .. } => {}
        &mut Inst::ValueLabelMarker { ref mut reg, .. } => {
            mapper.map_use(reg);
        }
        &mut Inst::Unwind { .. } => {}
    }
}

//=============================================================================
// Instructions: misc functions and external interface

impl MachInst for Inst {
    type LabelUse = LabelUse;

    fn get_regs(&self, collector: &mut RegUsageCollector) {
        aarch64_get_regs(self, collector)
    }

    fn map_regs<RM: RegMapper>(&mut self, mapper: &RM) {
        aarch64_map_regs(self, mapper);
    }

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        match self {
            &Inst::Mov64 { rd, rm } => Some((rd, rm)),
            &Inst::FpuMove64 { rd, rn } => Some((rd, rn)),
            &Inst::FpuMove128 { rd, rn } => Some((rd, rn)),
            _ => None,
        }
    }

    fn is_epilogue_placeholder(&self) -> bool {
        if let Inst::EpiloguePlaceholder = self {
            true
        } else {
            false
        }
    }

    fn is_included_in_clobbers(&self) -> bool {
        // We exclude call instructions from the clobber-set when they are calls
        // from caller to callee with the same ABI. Such calls cannot possibly
        // force any new registers to be saved in the prologue, because anything
        // that the callee clobbers, the caller is also allowed to clobber. This
        // both saves work and enables us to more precisely follow the
        // half-caller-save, half-callee-save SysV ABI for some vector
        // registers.
        //
        // See the note in [crate::isa::aarch64::abi::is_caller_save_reg] for
        // more information on this ABI-implementation hack.
        match self {
            &Inst::Call { ref info } => info.caller_callconv != info.callee_callconv,
            &Inst::CallInd { ref info } => info.caller_callconv != info.callee_callconv,
            _ => true,
        }
    }

    fn is_term<'a>(&'a self) -> MachTerminator<'a> {
        match self {
            &Inst::Ret | &Inst::EpiloguePlaceholder => MachTerminator::Ret,
            &Inst::Jump { dest } => MachTerminator::Uncond(dest.as_label().unwrap()),
            &Inst::CondBr {
                taken, not_taken, ..
            } => MachTerminator::Cond(taken.as_label().unwrap(), not_taken.as_label().unwrap()),
            &Inst::IndirectBr { ref targets, .. } => MachTerminator::Indirect(&targets[..]),
            &Inst::JTSequence { ref info, .. } => {
                MachTerminator::Indirect(&info.targets_for_term[..])
            }
            _ => MachTerminator::None,
        }
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Inst {
        let bits = ty.bits();

        assert!(bits <= 128);
        assert!(to_reg.to_reg().get_class() == from_reg.get_class());

        if from_reg.get_class() == RegClass::I64 {
            Inst::Mov64 {
                rd: to_reg,
                rm: from_reg,
            }
        } else if from_reg.get_class() == RegClass::V128 {
            if bits > 64 {
                Inst::FpuMove128 {
                    rd: to_reg,
                    rn: from_reg,
                }
            } else {
                Inst::FpuMove64 {
                    rd: to_reg,
                    rn: from_reg,
                }
            }
        } else {
            panic!("Unexpected register class: {:?}", from_reg.get_class());
        }
    }

    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        let to_reg = to_regs.only_reg();
        match ty {
            F64 => Inst::load_fp_constant64(to_reg.unwrap(), value as u64, alloc_tmp),
            F32 => Inst::load_fp_constant32(to_reg.unwrap(), value as u32, alloc_tmp),
            B1 | B8 | B16 | B32 | B64 | I8 | I16 | I32 | I64 | R32 | R64 => {
                Inst::load_constant(to_reg.unwrap(), value as u64)
            }
            I128 => Inst::load_constant128(to_regs, value),
            _ => panic!("Cannot generate constant for type: {}", ty),
        }
    }

    fn gen_nop(preferred_size: usize) -> Inst {
        if preferred_size == 0 {
            return Inst::Nop0;
        }
        // We can't give a NOP (or any insn) < 4 bytes.
        assert!(preferred_size >= 4);
        Inst::Nop4
    }

    fn maybe_direct_reload(&self, _reg: VirtualReg, _slot: SpillSlot) -> Option<Inst> {
        None
    }

    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
        match ty {
            I8 => Ok((&[RegClass::I64], &[I8])),
            I16 => Ok((&[RegClass::I64], &[I16])),
            I32 => Ok((&[RegClass::I64], &[I32])),
            I64 => Ok((&[RegClass::I64], &[I64])),
            B1 => Ok((&[RegClass::I64], &[B1])),
            B8 => Ok((&[RegClass::I64], &[B8])),
            B16 => Ok((&[RegClass::I64], &[B16])),
            B32 => Ok((&[RegClass::I64], &[B32])),
            B64 => Ok((&[RegClass::I64], &[B64])),
            R32 => panic!("32-bit reftype pointer should never be seen on AArch64"),
            R64 => Ok((&[RegClass::I64], &[R64])),
            F32 => Ok((&[RegClass::V128], &[F32])),
            F64 => Ok((&[RegClass::V128], &[F64])),
            I128 => Ok((&[RegClass::I64, RegClass::I64], &[I64, I64])),
            B128 => Ok((&[RegClass::I64, RegClass::I64], &[B64, B64])),
            _ if ty.is_vector() => {
                assert!(ty.bits() <= 128);
                Ok((&[RegClass::V128], &[I8X16]))
            }
            IFLAGS | FFLAGS => Ok((&[RegClass::I64], &[I64])),
            _ => Err(CodegenError::Unsupported(format!(
                "Unexpected SSA-value type: {}",
                ty
            ))),
        }
    }

    fn gen_jump(target: MachLabel) -> Inst {
        Inst::Jump {
            dest: BranchTarget::Label(target),
        }
    }

    fn worst_case_size() -> CodeOffset {
        // The maximum size, in bytes, of any `Inst`'s emitted code. We have at least one case of
        // an 8-instruction sequence (saturating int-to-float conversions) with three embedded
        // 64-bit f64 constants.
        //
        // Note that inline jump-tables handle island/pool insertion separately, so we do not need
        // to account for them here (otherwise the worst case would be 2^31 * 4, clearly not
        // feasible for other reasons).
        44
    }

    fn ref_type_regclass(_: &settings::Flags) -> RegClass {
        RegClass::I64
    }

    fn gen_value_label_marker(label: ValueLabel, reg: Reg) -> Self {
        Inst::ValueLabelMarker { label, reg }
    }

    fn defines_value_label(&self) -> Option<(ValueLabel, Reg)> {
        match self {
            Inst::ValueLabelMarker { label, reg } => Some((*label, *reg)),
            _ => None,
        }
    }
}

//=============================================================================
// Pretty-printing of instructions.

fn mem_finalize_for_show(
    mem: &AMode,
    mb_rru: Option<&RealRegUniverse>,
    state: &EmitState,
) -> (String, AMode) {
    let (mem_insts, mem) = mem_finalize(0, mem, state);
    let mut mem_str = mem_insts
        .into_iter()
        .map(|inst| inst.show_rru(mb_rru))
        .collect::<Vec<_>>()
        .join(" ; ");
    if !mem_str.is_empty() {
        mem_str += " ; ";
    }

    (mem_str, mem)
}

impl PrettyPrint for Inst {
    fn show_rru(&self, mb_rru: Option<&RealRegUniverse>) -> String {
        self.pretty_print(mb_rru, &mut EmitState::default())
    }
}

impl Inst {
    fn print_with_state(&self, mb_rru: Option<&RealRegUniverse>, state: &mut EmitState) -> String {
        fn op_name(alu_op: ALUOp) -> &'static str {
            match alu_op {
                ALUOp::Add => "add",
                ALUOp::Sub => "sub",
                ALUOp::Orr => "orr",
                ALUOp::And => "and",
                ALUOp::AndS => "ands",
                ALUOp::Eor => "eor",
                ALUOp::AddS => "adds",
                ALUOp::SubS => "subs",
                ALUOp::SMulH => "smulh",
                ALUOp::UMulH => "umulh",
                ALUOp::SDiv => "sdiv",
                ALUOp::UDiv => "udiv",
                ALUOp::AndNot => "bic",
                ALUOp::OrrNot => "orn",
                ALUOp::EorNot => "eon",
                ALUOp::RotR => "ror",
                ALUOp::Lsr => "lsr",
                ALUOp::Asr => "asr",
                ALUOp::Lsl => "lsl",
                ALUOp::Adc => "adc",
                ALUOp::AdcS => "adcs",
                ALUOp::Sbc => "sbc",
                ALUOp::SbcS => "sbcs",
            }
        }

        match self {
            &Inst::Nop0 => "nop-zero-len".to_string(),
            &Inst::Nop4 => "nop".to_string(),
            &Inst::AluRRR {
                alu_op,
                size,
                rd,
                rn,
                rm,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let rm = show_ireg_sized(rm, mb_rru, size);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::AluRRRR {
                alu_op,
                rd,
                rn,
                rm,
                ra,
            } => {
                let (op, size) = match alu_op {
                    ALUOp3::MAdd32 => ("madd", OperandSize::Size32),
                    ALUOp3::MAdd64 => ("madd", OperandSize::Size64),
                    ALUOp3::MSub32 => ("msub", OperandSize::Size32),
                    ALUOp3::MSub64 => ("msub", OperandSize::Size64),
                };
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let rm = show_ireg_sized(rm, mb_rru, size);
                let ra = show_ireg_sized(ra, mb_rru, size);

                format!("{} {}, {}, {}, {}", op, rd, rn, rm, ra)
            }
            &Inst::AluRRImm12 {
                alu_op,
                size,
                rd,
                rn,
                ref imm12,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);

                if imm12.bits == 0 && alu_op == ALUOp::Add && size.is64() {
                    // special-case MOV (used for moving into SP).
                    format!("mov {}, {}", rd, rn)
                } else {
                    let imm12 = imm12.show_rru(mb_rru);
                    format!("{} {}, {}, {}", op, rd, rn, imm12)
                }
            }
            &Inst::AluRRImmLogic {
                alu_op,
                size,
                rd,
                rn,
                ref imml,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let imml = imml.show_rru(mb_rru);
                format!("{} {}, {}, {}", op, rd, rn, imml)
            }
            &Inst::AluRRImmShift {
                alu_op,
                size,
                rd,
                rn,
                ref immshift,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let immshift = immshift.show_rru(mb_rru);
                format!("{} {}, {}, {}", op, rd, rn, immshift)
            }
            &Inst::AluRRRShift {
                alu_op,
                size,
                rd,
                rn,
                rm,
                ref shiftop,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let rm = show_ireg_sized(rm, mb_rru, size);
                let shiftop = shiftop.show_rru(mb_rru);
                format!("{} {}, {}, {}, {}", op, rd, rn, rm, shiftop)
            }
            &Inst::AluRRRExtend {
                alu_op,
                size,
                rd,
                rn,
                rm,
                ref extendop,
            } => {
                let op = op_name(alu_op);
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                let rm = show_ireg_sized(rm, mb_rru, size);
                let extendop = extendop.show_rru(mb_rru);
                format!("{} {}, {}, {}, {}", op, rd, rn, rm, extendop)
            }
            &Inst::BitRR { op, size, rd, rn } => {
                let op = op.op_str();
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::ULoad8 { rd, ref mem, .. }
            | &Inst::SLoad8 { rd, ref mem, .. }
            | &Inst::ULoad16 { rd, ref mem, .. }
            | &Inst::SLoad16 { rd, ref mem, .. }
            | &Inst::ULoad32 { rd, ref mem, .. }
            | &Inst::SLoad32 { rd, ref mem, .. }
            | &Inst::ULoad64 { rd, ref mem, .. } => {
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);

                let is_unscaled = match &mem {
                    &AMode::Unscaled(..) => true,
                    _ => false,
                };
                let (op, size) = match (self, is_unscaled) {
                    (&Inst::ULoad8 { .. }, false) => ("ldrb", OperandSize::Size32),
                    (&Inst::ULoad8 { .. }, true) => ("ldurb", OperandSize::Size32),
                    (&Inst::SLoad8 { .. }, false) => ("ldrsb", OperandSize::Size64),
                    (&Inst::SLoad8 { .. }, true) => ("ldursb", OperandSize::Size64),
                    (&Inst::ULoad16 { .. }, false) => ("ldrh", OperandSize::Size32),
                    (&Inst::ULoad16 { .. }, true) => ("ldurh", OperandSize::Size32),
                    (&Inst::SLoad16 { .. }, false) => ("ldrsh", OperandSize::Size64),
                    (&Inst::SLoad16 { .. }, true) => ("ldursh", OperandSize::Size64),
                    (&Inst::ULoad32 { .. }, false) => ("ldr", OperandSize::Size32),
                    (&Inst::ULoad32 { .. }, true) => ("ldur", OperandSize::Size32),
                    (&Inst::SLoad32 { .. }, false) => ("ldrsw", OperandSize::Size64),
                    (&Inst::SLoad32 { .. }, true) => ("ldursw", OperandSize::Size64),
                    (&Inst::ULoad64 { .. }, false) => ("ldr", OperandSize::Size64),
                    (&Inst::ULoad64 { .. }, true) => ("ldur", OperandSize::Size64),
                    _ => unreachable!(),
                };
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let mem = mem.show_rru(mb_rru);
                format!("{}{} {}, {}", mem_str, op, rd, mem)
            }
            &Inst::Store8 { rd, ref mem, .. }
            | &Inst::Store16 { rd, ref mem, .. }
            | &Inst::Store32 { rd, ref mem, .. }
            | &Inst::Store64 { rd, ref mem, .. } => {
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);

                let is_unscaled = match &mem {
                    &AMode::Unscaled(..) => true,
                    _ => false,
                };
                let (op, size) = match (self, is_unscaled) {
                    (&Inst::Store8 { .. }, false) => ("strb", OperandSize::Size32),
                    (&Inst::Store8 { .. }, true) => ("sturb", OperandSize::Size32),
                    (&Inst::Store16 { .. }, false) => ("strh", OperandSize::Size32),
                    (&Inst::Store16 { .. }, true) => ("sturh", OperandSize::Size32),
                    (&Inst::Store32 { .. }, false) => ("str", OperandSize::Size32),
                    (&Inst::Store32 { .. }, true) => ("stur", OperandSize::Size32),
                    (&Inst::Store64 { .. }, false) => ("str", OperandSize::Size64),
                    (&Inst::Store64 { .. }, true) => ("stur", OperandSize::Size64),
                    _ => unreachable!(),
                };
                let rd = show_ireg_sized(rd, mb_rru, size);
                let mem = mem.show_rru(mb_rru);
                format!("{}{} {}, {}", mem_str, op, rd, mem)
            }
            &Inst::StoreP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = rt.show_rru(mb_rru);
                let rt2 = rt2.show_rru(mb_rru);
                let mem = mem.show_rru(mb_rru);
                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::LoadP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = rt.to_reg().show_rru(mb_rru);
                let rt2 = rt2.to_reg().show_rru(mb_rru);
                let mem = mem.show_rru(mb_rru);
                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::Mov64 { rd, rm } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let rm = rm.show_rru(mb_rru);
                format!("mov {}, {}", rd, rm)
            }
            &Inst::Mov32 { rd, rm } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, OperandSize::Size32);
                let rm = show_ireg_sized(rm, mb_rru, OperandSize::Size32);
                format!("mov {}, {}", rd, rm)
            }
            &Inst::MovZ { rd, ref imm, size } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let imm = imm.show_rru(mb_rru);
                format!("movz {}, {}", rd, imm)
            }
            &Inst::MovN { rd, ref imm, size } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let imm = imm.show_rru(mb_rru);
                format!("movn {}, {}", rd, imm)
            }
            &Inst::MovK { rd, ref imm, size } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size);
                let imm = imm.show_rru(mb_rru);
                format!("movk {}, {}", rd, imm)
            }
            &Inst::CSel { rd, rn, rm, cond } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let rn = rn.show_rru(mb_rru);
                let rm = rm.show_rru(mb_rru);
                let cond = cond.show_rru(mb_rru);
                format!("csel {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::CSet { rd, cond } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let cond = cond.show_rru(mb_rru);
                format!("cset {}, {}", rd, cond)
            }
            &Inst::CSetm { rd, cond } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let cond = cond.show_rru(mb_rru);
                format!("csetm {}, {}", rd, cond)
            }
            &Inst::CCmpImm {
                size,
                rn,
                imm,
                nzcv,
                cond,
            } => {
                let rn = show_ireg_sized(rn, mb_rru, size);
                let imm = imm.show_rru(mb_rru);
                let nzcv = nzcv.show_rru(mb_rru);
                let cond = cond.show_rru(mb_rru);
                format!("ccmp {}, {}, {}, {}", rn, imm, nzcv, cond)
            }
            &Inst::AtomicRMW { rs, rt, rn, ty, op } => {
                let op = match op {
                    AtomicRMWOp::Add => "ldaddal",
                    AtomicRMWOp::Clr => "ldclral",
                    AtomicRMWOp::Eor => "ldeoral",
                    AtomicRMWOp::Set => "ldsetal",
                    AtomicRMWOp::Smax => "ldsmaxal",
                    AtomicRMWOp::Umax => "ldumaxal",
                    AtomicRMWOp::Smin => "ldsminal",
                    AtomicRMWOp::Umin => "lduminal",
                };

                let size = OperandSize::from_ty(ty);
                let rs = show_ireg_sized(rs, mb_rru, size);
                let rt = show_ireg_sized(rt.to_reg(), mb_rru, size);
                let rn = rn.show_rru(mb_rru);

                let ty_suffix = match ty {
                    I8 => "b",
                    I16 => "h",
                    _ => "",
                };
                format!("{}{} {}, {}, [{}]", op, ty_suffix, rs, rt, rn)
            }
            &Inst::AtomicRMWLoop { ty, op, .. } => {
                let ty_suffix = match ty {
                    I8 => "b",
                    I16 => "h",
                    _ => "",
                };
                let size = OperandSize::from_ty(ty);
                let r_status = show_ireg_sized(xreg(24), mb_rru, OperandSize::Size32);
                let r_arg2 = show_ireg_sized(xreg(26), mb_rru, size);
                let r_tmp = show_ireg_sized(xreg(27), mb_rru, size);
                let mut r_dst = show_ireg_sized(xreg(28), mb_rru, size);

                let mut loop_str: String = "1: ".to_string();
                loop_str.push_str(&format!("ldaxr{} {}, [x25]; ", ty_suffix, r_tmp));

                let op_str = match op {
                    inst_common::AtomicRmwOp::Add => "add",
                    inst_common::AtomicRmwOp::Sub => "sub",
                    inst_common::AtomicRmwOp::Xor => "eor",
                    inst_common::AtomicRmwOp::Or => "orr",
                    inst_common::AtomicRmwOp::And => "and",
                    _ => "",
                };

                if op_str.is_empty() {
                    match op {
                        inst_common::AtomicRmwOp::Xchg => r_dst = r_arg2,
                        inst_common::AtomicRmwOp::Nand => {
                            loop_str.push_str(&format!("and {}, {}, {}; ", r_dst, r_tmp, r_arg2));
                            loop_str.push_str(&format!("mvn {}, {}; ", r_dst, r_dst));
                        }
                        _ => {
                            loop_str.push_str(&format!("cmp {}, {}; ", r_tmp, r_arg2));
                            let cond = match op {
                                inst_common::AtomicRmwOp::Smin => "lt",
                                inst_common::AtomicRmwOp::Smax => "gt",
                                inst_common::AtomicRmwOp::Umin => "lo",
                                inst_common::AtomicRmwOp::Umax => "hi",
                                _ => unreachable!(),
                            };
                            loop_str.push_str(&format!(
                                "csel {}, {}, {}, {}; ",
                                r_dst, r_tmp, r_arg2, cond
                            ));
                        }
                    };
                } else {
                    loop_str.push_str(&format!("{} {}, {}, {}; ", op_str, r_dst, r_tmp, r_arg2));
                }
                loop_str.push_str(&format!(
                    "stlxr{} {}, {}, [x25]; ",
                    ty_suffix, r_status, r_dst
                ));
                loop_str.push_str(&format!("cbnz {}, 1b", r_status));
                loop_str
            }
            &Inst::AtomicCAS { rs, rt, rn, ty } => {
                let op = match ty {
                    I8 => "casalb",
                    I16 => "casalh",
                    I32 | I64 => "casal",
                    _ => panic!("Unsupported type: {}", ty),
                };
                let size = OperandSize::from_ty(ty);
                let rs = show_ireg_sized(rs.to_reg(), mb_rru, size);
                let rt = show_ireg_sized(rt, mb_rru, size);
                let rn = rn.show_rru(mb_rru);

                format!("{} {}, {}, [{}]", op, rs, rt, rn)
            }
            &Inst::AtomicCASLoop { ty } => {
                format!(
                    "atomically {{ compare-and-swap({}_bits_at_[x25], x26 -> x28), x27 = old_value_at_[x25]; x24 = trash }}",
                    ty.bits())
            }
            &Inst::LoadAcquire {
                access_ty, rt, rn, ..
            } => {
                let (op, ty) = match access_ty {
                    I8 => ("ldarb", I32),
                    I16 => ("ldarh", I32),
                    I32 => ("ldar", I32),
                    I64 => ("ldar", I64),
                    _ => panic!("Unsupported type: {}", access_ty),
                };
                let size = OperandSize::from_ty(ty);
                let rt = show_ireg_sized(rt.to_reg(), mb_rru, size);
                let rn = rn.show_rru(mb_rru);
                format!("{} {}, [{}]", op, rt, rn)
            }
            &Inst::StoreRelease {
                access_ty, rt, rn, ..
            } => {
                let (op, ty) = match access_ty {
                    I8 => ("stlrb", I32),
                    I16 => ("stlrh", I32),
                    I32 => ("stlr", I32),
                    I64 => ("stlr", I64),
                    _ => panic!("Unsupported type: {}", access_ty),
                };
                let size = OperandSize::from_ty(ty);
                let rt = show_ireg_sized(rt, mb_rru, size);
                let rn = rn.show_rru(mb_rru);
                format!("{} {}, [{}]", op, rt, rn)
            }
            &Inst::Fence {} => {
                format!("dmb ish")
            }
            &Inst::FpuMove64 { rd, rn } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size64);
                let rn = show_vreg_scalar(rn, mb_rru, ScalarSize::Size64);
                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuMove128 { rd, rn } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let rn = rn.show_rru(mb_rru);
                format!("mov {}.16b, {}.16b", rd, rn)
            }
            &Inst::FpuMoveFromVec { rd, rn, idx, size } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size.lane_size());
                let rn = show_vreg_element(rn, mb_rru, idx, size);
                format!("mov {}, {}", rd, rn)
            }
            &Inst::FpuExtend { rd, rn, size } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_scalar(rn, mb_rru, size);

                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuRR { fpu_op, rd, rn } => {
                let (op, sizesrc, sizedest) = match fpu_op {
                    FPUOp1::Abs32 => ("fabs", ScalarSize::Size32, ScalarSize::Size32),
                    FPUOp1::Abs64 => ("fabs", ScalarSize::Size64, ScalarSize::Size64),
                    FPUOp1::Neg32 => ("fneg", ScalarSize::Size32, ScalarSize::Size32),
                    FPUOp1::Neg64 => ("fneg", ScalarSize::Size64, ScalarSize::Size64),
                    FPUOp1::Sqrt32 => ("fsqrt", ScalarSize::Size32, ScalarSize::Size32),
                    FPUOp1::Sqrt64 => ("fsqrt", ScalarSize::Size64, ScalarSize::Size64),
                    FPUOp1::Cvt32To64 => ("fcvt", ScalarSize::Size32, ScalarSize::Size64),
                    FPUOp1::Cvt64To32 => ("fcvt", ScalarSize::Size64, ScalarSize::Size32),
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, sizedest);
                let rn = show_vreg_scalar(rn, mb_rru, sizesrc);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::FpuRRR { fpu_op, rd, rn, rm } => {
                let (op, size) = match fpu_op {
                    FPUOp2::Add32 => ("fadd", ScalarSize::Size32),
                    FPUOp2::Add64 => ("fadd", ScalarSize::Size64),
                    FPUOp2::Sub32 => ("fsub", ScalarSize::Size32),
                    FPUOp2::Sub64 => ("fsub", ScalarSize::Size64),
                    FPUOp2::Mul32 => ("fmul", ScalarSize::Size32),
                    FPUOp2::Mul64 => ("fmul", ScalarSize::Size64),
                    FPUOp2::Div32 => ("fdiv", ScalarSize::Size32),
                    FPUOp2::Div64 => ("fdiv", ScalarSize::Size64),
                    FPUOp2::Max32 => ("fmax", ScalarSize::Size32),
                    FPUOp2::Max64 => ("fmax", ScalarSize::Size64),
                    FPUOp2::Min32 => ("fmin", ScalarSize::Size32),
                    FPUOp2::Min64 => ("fmin", ScalarSize::Size64),
                    FPUOp2::Sqadd64 => ("sqadd", ScalarSize::Size64),
                    FPUOp2::Uqadd64 => ("uqadd", ScalarSize::Size64),
                    FPUOp2::Sqsub64 => ("sqsub", ScalarSize::Size64),
                    FPUOp2::Uqsub64 => ("uqsub", ScalarSize::Size64),
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_scalar(rn, mb_rru, size);
                let rm = show_vreg_scalar(rm, mb_rru, size);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::FpuRRI { fpu_op, rd, rn } => {
                let (op, imm, vector) = match fpu_op {
                    FPUOpRI::UShr32(imm) => ("ushr", imm.show_rru(mb_rru), true),
                    FPUOpRI::UShr64(imm) => ("ushr", imm.show_rru(mb_rru), false),
                    FPUOpRI::Sli32(imm) => ("sli", imm.show_rru(mb_rru), true),
                    FPUOpRI::Sli64(imm) => ("sli", imm.show_rru(mb_rru), false),
                };

                let show_vreg_fn: fn(Reg, Option<&RealRegUniverse>) -> String = if vector {
                    |reg, mb_rru| show_vreg_vector(reg, mb_rru, VectorSize::Size32x2)
                } else {
                    |reg, mb_rru| show_vreg_scalar(reg, mb_rru, ScalarSize::Size64)
                };
                let rd = show_vreg_fn(rd.to_reg(), mb_rru);
                let rn = show_vreg_fn(rn, mb_rru);
                format!("{} {}, {}, {}", op, rd, rn, imm)
            }
            &Inst::FpuRRRR {
                fpu_op,
                rd,
                rn,
                rm,
                ra,
            } => {
                let (op, size) = match fpu_op {
                    FPUOp3::MAdd32 => ("fmadd", ScalarSize::Size32),
                    FPUOp3::MAdd64 => ("fmadd", ScalarSize::Size64),
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_scalar(rn, mb_rru, size);
                let rm = show_vreg_scalar(rm, mb_rru, size);
                let ra = show_vreg_scalar(ra, mb_rru, size);
                format!("{} {}, {}, {}, {}", op, rd, rn, rm, ra)
            }
            &Inst::FpuCmp32 { rn, rm } => {
                let rn = show_vreg_scalar(rn, mb_rru, ScalarSize::Size32);
                let rm = show_vreg_scalar(rm, mb_rru, ScalarSize::Size32);
                format!("fcmp {}, {}", rn, rm)
            }
            &Inst::FpuCmp64 { rn, rm } => {
                let rn = show_vreg_scalar(rn, mb_rru, ScalarSize::Size64);
                let rm = show_vreg_scalar(rm, mb_rru, ScalarSize::Size64);
                format!("fcmp {}, {}", rn, rm)
            }
            &Inst::FpuLoad32 { rd, ref mem, .. } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size32);
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoad64 { rd, ref mem, .. } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size64);
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoad128 { rd, ref mem, .. } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                let rd = "q".to_string() + &rd[1..];
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore32 { rd, ref mem, .. } => {
                let rd = show_vreg_scalar(rd, mb_rru, ScalarSize::Size32);
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore64 { rd, ref mem, .. } => {
                let rd = show_vreg_scalar(rd, mb_rru, ScalarSize::Size64);
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore128 { rd, ref mem, .. } => {
                let rd = rd.show_rru(mb_rru);
                let rd = "q".to_string() + &rd[1..];
                let (mem_str, mem) = mem_finalize_for_show(mem, mb_rru, state);
                let mem = mem.show_rru(mb_rru);
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoadP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = show_vreg_scalar(rt.to_reg(), mb_rru, ScalarSize::Size64);
                let rt2 = show_vreg_scalar(rt2.to_reg(), mb_rru, ScalarSize::Size64);
                let mem = mem.show_rru(mb_rru);

                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuStoreP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = show_vreg_scalar(rt, mb_rru, ScalarSize::Size64);
                let rt2 = show_vreg_scalar(rt2, mb_rru, ScalarSize::Size64);
                let mem = mem.show_rru(mb_rru);

                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuLoadP128 {
                rt, rt2, ref mem, ..
            } => {
                let rt = show_vreg_scalar(rt.to_reg(), mb_rru, ScalarSize::Size128);
                let rt2 = show_vreg_scalar(rt2.to_reg(), mb_rru, ScalarSize::Size128);
                let mem = mem.show_rru(mb_rru);

                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuStoreP128 {
                rt, rt2, ref mem, ..
            } => {
                let rt = show_vreg_scalar(rt, mb_rru, ScalarSize::Size128);
                let rt2 = show_vreg_scalar(rt2, mb_rru, ScalarSize::Size128);
                let mem = mem.show_rru(mb_rru);

                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::LoadFpuConst64 { rd, const_data } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size64);
                format!(
                    "ldr {}, pc+8 ; b 12 ; data.f64 {}",
                    rd,
                    f64::from_bits(const_data)
                )
            }
            &Inst::LoadFpuConst128 { rd, const_data } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size128);
                format!("ldr {}, pc+8 ; b 20 ; data.f128 0x{:032x}", rd, const_data)
            }
            &Inst::FpuToInt { op, rd, rn } => {
                let (op, sizesrc, sizedest) = match op {
                    FpuToIntOp::F32ToI32 => ("fcvtzs", ScalarSize::Size32, OperandSize::Size32),
                    FpuToIntOp::F32ToU32 => ("fcvtzu", ScalarSize::Size32, OperandSize::Size32),
                    FpuToIntOp::F32ToI64 => ("fcvtzs", ScalarSize::Size32, OperandSize::Size64),
                    FpuToIntOp::F32ToU64 => ("fcvtzu", ScalarSize::Size32, OperandSize::Size64),
                    FpuToIntOp::F64ToI32 => ("fcvtzs", ScalarSize::Size64, OperandSize::Size32),
                    FpuToIntOp::F64ToU32 => ("fcvtzu", ScalarSize::Size64, OperandSize::Size32),
                    FpuToIntOp::F64ToI64 => ("fcvtzs", ScalarSize::Size64, OperandSize::Size64),
                    FpuToIntOp::F64ToU64 => ("fcvtzu", ScalarSize::Size64, OperandSize::Size64),
                };
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, sizedest);
                let rn = show_vreg_scalar(rn, mb_rru, sizesrc);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::IntToFpu { op, rd, rn } => {
                let (op, sizesrc, sizedest) = match op {
                    IntToFpuOp::I32ToF32 => ("scvtf", OperandSize::Size32, ScalarSize::Size32),
                    IntToFpuOp::U32ToF32 => ("ucvtf", OperandSize::Size32, ScalarSize::Size32),
                    IntToFpuOp::I64ToF32 => ("scvtf", OperandSize::Size64, ScalarSize::Size32),
                    IntToFpuOp::U64ToF32 => ("ucvtf", OperandSize::Size64, ScalarSize::Size32),
                    IntToFpuOp::I32ToF64 => ("scvtf", OperandSize::Size32, ScalarSize::Size64),
                    IntToFpuOp::U32ToF64 => ("ucvtf", OperandSize::Size32, ScalarSize::Size64),
                    IntToFpuOp::I64ToF64 => ("scvtf", OperandSize::Size64, ScalarSize::Size64),
                    IntToFpuOp::U64ToF64 => ("ucvtf", OperandSize::Size64, ScalarSize::Size64),
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, sizedest);
                let rn = show_ireg_sized(rn, mb_rru, sizesrc);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::FpuCSel32 { rd, rn, rm, cond } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size32);
                let rn = show_vreg_scalar(rn, mb_rru, ScalarSize::Size32);
                let rm = show_vreg_scalar(rm, mb_rru, ScalarSize::Size32);
                let cond = cond.show_rru(mb_rru);
                format!("fcsel {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::FpuCSel64 { rd, rn, rm, cond } => {
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size64);
                let rn = show_vreg_scalar(rn, mb_rru, ScalarSize::Size64);
                let rm = show_vreg_scalar(rm, mb_rru, ScalarSize::Size64);
                let cond = cond.show_rru(mb_rru);
                format!("fcsel {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::FpuRound { op, rd, rn } => {
                let (inst, size) = match op {
                    FpuRoundMode::Minus32 => ("frintm", ScalarSize::Size32),
                    FpuRoundMode::Minus64 => ("frintm", ScalarSize::Size64),
                    FpuRoundMode::Plus32 => ("frintp", ScalarSize::Size32),
                    FpuRoundMode::Plus64 => ("frintp", ScalarSize::Size64),
                    FpuRoundMode::Zero32 => ("frintz", ScalarSize::Size32),
                    FpuRoundMode::Zero64 => ("frintz", ScalarSize::Size64),
                    FpuRoundMode::Nearest32 => ("frintn", ScalarSize::Size32),
                    FpuRoundMode::Nearest64 => ("frintn", ScalarSize::Size64),
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_scalar(rn, mb_rru, size);
                format!("{} {}, {}", inst, rd, rn)
            }
            &Inst::MovToFpu { rd, rn, size } => {
                let operand_size = size.operand_size();
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, operand_size);
                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuMoveFPImm { rd, imm, size } => {
                let imm = imm.show_rru(mb_rru);
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size);

                format!("fmov {}, {}", rd, imm)
            }
            &Inst::MovToVec { rd, rn, idx, size } => {
                let rd = show_vreg_element(rd.to_reg(), mb_rru, idx, size);
                let rn = show_ireg_sized(rn, mb_rru, size.operand_size());
                format!("mov {}, {}", rd, rn)
            }
            &Inst::MovFromVec { rd, rn, idx, size } => {
                let op = match size {
                    VectorSize::Size8x16 => "umov",
                    VectorSize::Size16x8 => "umov",
                    VectorSize::Size32x4 => "mov",
                    VectorSize::Size64x2 => "mov",
                    _ => unimplemented!(),
                };
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, size.operand_size());
                let rn = show_vreg_element(rn, mb_rru, idx, size);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::MovFromVecSigned {
                rd,
                rn,
                idx,
                size,
                scalar_size,
            } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, scalar_size);
                let rn = show_vreg_element(rn, mb_rru, idx, size);
                format!("smov {}, {}", rd, rn)
            }
            &Inst::VecDup { rd, rn, size } => {
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = show_ireg_sized(rn, mb_rru, size.operand_size());
                format!("dup {}, {}", rd, rn)
            }
            &Inst::VecDupFromFpu { rd, rn, size } => {
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_element(rn, mb_rru, 0, size);
                format!("dup {}, {}", rd, rn)
            }
            &Inst::VecDupFPImm { rd, imm, size } => {
                let imm = imm.show_rru(mb_rru);
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);

                format!("fmov {}, {}", rd, imm)
            }
            &Inst::VecDupImm {
                rd,
                imm,
                invert,
                size,
            } => {
                let imm = imm.show_rru(mb_rru);
                let op = if invert { "mvni" } else { "movi" };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);

                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::VecExtend {
                t,
                rd,
                rn,
                high_half,
            } => {
                let (op, dest, src) = match (t, high_half) {
                    (VecExtendOp::Sxtl8, false) => {
                        ("sxtl", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecExtendOp::Sxtl8, true) => {
                        ("sxtl2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecExtendOp::Sxtl16, false) => {
                        ("sxtl", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecExtendOp::Sxtl16, true) => {
                        ("sxtl2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecExtendOp::Sxtl32, false) => {
                        ("sxtl", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecExtendOp::Sxtl32, true) => {
                        ("sxtl2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                    (VecExtendOp::Uxtl8, false) => {
                        ("uxtl", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecExtendOp::Uxtl8, true) => {
                        ("uxtl2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecExtendOp::Uxtl16, false) => {
                        ("uxtl", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecExtendOp::Uxtl16, true) => {
                        ("uxtl2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecExtendOp::Uxtl32, false) => {
                        ("uxtl", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecExtendOp::Uxtl32, true) => {
                        ("uxtl2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, dest);
                let rn = show_vreg_vector(rn, mb_rru, src);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecMovElement {
                rd,
                rn,
                dest_idx,
                src_idx,
                size,
            } => {
                let rd = show_vreg_element(rd.to_reg(), mb_rru, dest_idx, size);
                let rn = show_vreg_element(rn, mb_rru, src_idx, size);
                format!("mov {}, {}", rd, rn)
            }
            &Inst::VecRRLong {
                op,
                rd,
                rn,
                high_half,
            } => {
                let (op, rd_size, size, suffix) = match (op, high_half) {
                    (VecRRLongOp::Fcvtl16, false) => {
                        ("fcvtl", VectorSize::Size32x4, VectorSize::Size16x4, "")
                    }
                    (VecRRLongOp::Fcvtl16, true) => {
                        ("fcvtl2", VectorSize::Size32x4, VectorSize::Size16x8, "")
                    }
                    (VecRRLongOp::Fcvtl32, false) => {
                        ("fcvtl", VectorSize::Size64x2, VectorSize::Size32x2, "")
                    }
                    (VecRRLongOp::Fcvtl32, true) => {
                        ("fcvtl2", VectorSize::Size64x2, VectorSize::Size32x4, "")
                    }
                    (VecRRLongOp::Shll8, false) => {
                        ("shll", VectorSize::Size16x8, VectorSize::Size8x8, ", #8")
                    }
                    (VecRRLongOp::Shll8, true) => {
                        ("shll2", VectorSize::Size16x8, VectorSize::Size8x16, ", #8")
                    }
                    (VecRRLongOp::Shll16, false) => {
                        ("shll", VectorSize::Size32x4, VectorSize::Size16x4, ", #16")
                    }
                    (VecRRLongOp::Shll16, true) => {
                        ("shll2", VectorSize::Size32x4, VectorSize::Size16x8, ", #16")
                    }
                    (VecRRLongOp::Shll32, false) => {
                        ("shll", VectorSize::Size64x2, VectorSize::Size32x2, ", #32")
                    }
                    (VecRRLongOp::Shll32, true) => {
                        ("shll2", VectorSize::Size64x2, VectorSize::Size32x4, ", #32")
                    }
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, rd_size);
                let rn = show_vreg_vector(rn, mb_rru, size);

                format!("{} {}, {}{}", op, rd, rn, suffix)
            }
            &Inst::VecRRNarrow {
                op,
                rd,
                rn,
                high_half,
            } => {
                let (op, rd_size, size) = match (op, high_half) {
                    (VecRRNarrowOp::Xtn16, false) => {
                        ("xtn", VectorSize::Size8x8, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Xtn16, true) => {
                        ("xtn2", VectorSize::Size8x16, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Xtn32, false) => {
                        ("xtn", VectorSize::Size16x4, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Xtn32, true) => {
                        ("xtn2", VectorSize::Size16x8, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Xtn64, false) => {
                        ("xtn", VectorSize::Size32x2, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Xtn64, true) => {
                        ("xtn2", VectorSize::Size32x4, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Sqxtn16, false) => {
                        ("sqxtn", VectorSize::Size8x8, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Sqxtn16, true) => {
                        ("sqxtn2", VectorSize::Size8x16, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Sqxtn32, false) => {
                        ("sqxtn", VectorSize::Size16x4, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Sqxtn32, true) => {
                        ("sqxtn2", VectorSize::Size16x8, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Sqxtn64, false) => {
                        ("sqxtn", VectorSize::Size32x2, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Sqxtn64, true) => {
                        ("sqxtn2", VectorSize::Size32x4, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Sqxtun16, false) => {
                        ("sqxtun", VectorSize::Size8x8, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Sqxtun16, true) => {
                        ("sqxtun2", VectorSize::Size8x16, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Sqxtun32, false) => {
                        ("sqxtun", VectorSize::Size16x4, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Sqxtun32, true) => {
                        ("sqxtun2", VectorSize::Size16x8, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Sqxtun64, false) => {
                        ("sqxtun", VectorSize::Size32x2, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Sqxtun64, true) => {
                        ("sqxtun2", VectorSize::Size32x4, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Uqxtn16, false) => {
                        ("uqxtn", VectorSize::Size8x8, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Uqxtn16, true) => {
                        ("uqxtn2", VectorSize::Size8x16, VectorSize::Size16x8)
                    }
                    (VecRRNarrowOp::Uqxtn32, false) => {
                        ("uqxtn", VectorSize::Size16x4, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Uqxtn32, true) => {
                        ("uqxtn2", VectorSize::Size16x8, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Uqxtn64, false) => {
                        ("uqxtn", VectorSize::Size32x2, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Uqxtn64, true) => {
                        ("uqxtn2", VectorSize::Size32x4, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Fcvtn32, false) => {
                        ("fcvtn", VectorSize::Size16x4, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Fcvtn32, true) => {
                        ("fcvtn2", VectorSize::Size16x8, VectorSize::Size32x4)
                    }
                    (VecRRNarrowOp::Fcvtn64, false) => {
                        ("fcvtn", VectorSize::Size32x2, VectorSize::Size64x2)
                    }
                    (VecRRNarrowOp::Fcvtn64, true) => {
                        ("fcvtn2", VectorSize::Size32x4, VectorSize::Size64x2)
                    }
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, rd_size);
                let rn = show_vreg_vector(rn, mb_rru, size);

                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecRRPair { op, rd, rn } => {
                let op = match op {
                    VecPairOp::Addp => "addp",
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, ScalarSize::Size64);
                let rn = show_vreg_vector(rn, mb_rru, VectorSize::Size64x2);

                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecRRPairLong { op, rd, rn } => {
                let (op, dest, src) = match op {
                    VecRRPairLongOp::Saddlp8 => {
                        ("saddlp", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    VecRRPairLongOp::Saddlp16 => {
                        ("saddlp", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    VecRRPairLongOp::Uaddlp8 => {
                        ("uaddlp", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    VecRRPairLongOp::Uaddlp16 => {
                        ("uaddlp", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, dest);
                let rn = show_vreg_vector(rn, mb_rru, src);

                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecRRR {
                rd,
                rn,
                rm,
                alu_op,
                size,
            } => {
                let (op, size) = match alu_op {
                    VecALUOp::Sqadd => ("sqadd", size),
                    VecALUOp::Uqadd => ("uqadd", size),
                    VecALUOp::Sqsub => ("sqsub", size),
                    VecALUOp::Uqsub => ("uqsub", size),
                    VecALUOp::Cmeq => ("cmeq", size),
                    VecALUOp::Cmge => ("cmge", size),
                    VecALUOp::Cmgt => ("cmgt", size),
                    VecALUOp::Cmhs => ("cmhs", size),
                    VecALUOp::Cmhi => ("cmhi", size),
                    VecALUOp::Fcmeq => ("fcmeq", size),
                    VecALUOp::Fcmgt => ("fcmgt", size),
                    VecALUOp::Fcmge => ("fcmge", size),
                    VecALUOp::And => ("and", VectorSize::Size8x16),
                    VecALUOp::Bic => ("bic", VectorSize::Size8x16),
                    VecALUOp::Orr => ("orr", VectorSize::Size8x16),
                    VecALUOp::Eor => ("eor", VectorSize::Size8x16),
                    VecALUOp::Bsl => ("bsl", VectorSize::Size8x16),
                    VecALUOp::Umaxp => ("umaxp", size),
                    VecALUOp::Add => ("add", size),
                    VecALUOp::Sub => ("sub", size),
                    VecALUOp::Mul => ("mul", size),
                    VecALUOp::Sshl => ("sshl", size),
                    VecALUOp::Ushl => ("ushl", size),
                    VecALUOp::Umin => ("umin", size),
                    VecALUOp::Smin => ("smin", size),
                    VecALUOp::Umax => ("umax", size),
                    VecALUOp::Smax => ("smax", size),
                    VecALUOp::Urhadd => ("urhadd", size),
                    VecALUOp::Fadd => ("fadd", size),
                    VecALUOp::Fsub => ("fsub", size),
                    VecALUOp::Fdiv => ("fdiv", size),
                    VecALUOp::Fmax => ("fmax", size),
                    VecALUOp::Fmin => ("fmin", size),
                    VecALUOp::Fmul => ("fmul", size),
                    VecALUOp::Addp => ("addp", size),
                    VecALUOp::Zip1 => ("zip1", size),
                    VecALUOp::Sqrdmulh => ("sqrdmulh", size),
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_vector(rn, mb_rru, size);
                let rm = show_vreg_vector(rm, mb_rru, size);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::VecRRRLong {
                rd,
                rn,
                rm,
                alu_op,
                high_half,
            } => {
                let (op, dest_size, src_size) = match (alu_op, high_half) {
                    (VecRRRLongOp::Smull8, false) => {
                        ("smull", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecRRRLongOp::Smull8, true) => {
                        ("smull2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecRRRLongOp::Smull16, false) => {
                        ("smull", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecRRRLongOp::Smull16, true) => {
                        ("smull2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecRRRLongOp::Smull32, false) => {
                        ("smull", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecRRRLongOp::Smull32, true) => {
                        ("smull2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                    (VecRRRLongOp::Umull8, false) => {
                        ("umull", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecRRRLongOp::Umull8, true) => {
                        ("umull2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecRRRLongOp::Umull16, false) => {
                        ("umull", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecRRRLongOp::Umull16, true) => {
                        ("umull2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecRRRLongOp::Umull32, false) => {
                        ("umull", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecRRRLongOp::Umull32, true) => {
                        ("umull2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                    (VecRRRLongOp::Umlal8, false) => {
                        ("umlal", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecRRRLongOp::Umlal8, true) => {
                        ("umlal2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecRRRLongOp::Umlal16, false) => {
                        ("umlal", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecRRRLongOp::Umlal16, true) => {
                        ("umlal2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecRRRLongOp::Umlal32, false) => {
                        ("umlal", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecRRRLongOp::Umlal32, true) => {
                        ("umlal2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, dest_size);
                let rn = show_vreg_vector(rn, mb_rru, src_size);
                let rm = show_vreg_vector(rm, mb_rru, src_size);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::VecMisc { op, rd, rn, size } => {
                let (op, size, suffix) = match op {
                    VecMisc2::Not => (
                        "mvn",
                        if size.is_128bits() {
                            VectorSize::Size8x16
                        } else {
                            VectorSize::Size8x8
                        },
                        "",
                    ),
                    VecMisc2::Neg => ("neg", size, ""),
                    VecMisc2::Abs => ("abs", size, ""),
                    VecMisc2::Fabs => ("fabs", size, ""),
                    VecMisc2::Fneg => ("fneg", size, ""),
                    VecMisc2::Fsqrt => ("fsqrt", size, ""),
                    VecMisc2::Rev64 => ("rev64", size, ""),
                    VecMisc2::Fcvtzs => ("fcvtzs", size, ""),
                    VecMisc2::Fcvtzu => ("fcvtzu", size, ""),
                    VecMisc2::Scvtf => ("scvtf", size, ""),
                    VecMisc2::Ucvtf => ("ucvtf", size, ""),
                    VecMisc2::Frintn => ("frintn", size, ""),
                    VecMisc2::Frintz => ("frintz", size, ""),
                    VecMisc2::Frintm => ("frintm", size, ""),
                    VecMisc2::Frintp => ("frintp", size, ""),
                    VecMisc2::Cnt => ("cnt", size, ""),
                    VecMisc2::Cmeq0 => ("cmeq", size, ", #0"),
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_vector(rn, mb_rru, size);
                format!("{} {}, {}{}", op, rd, rn, suffix)
            }
            &Inst::VecLanes { op, rd, rn, size } => {
                let op = match op {
                    VecLanesOp::Uminv => "uminv",
                    VecLanesOp::Addv => "addv",
                };
                let rd = show_vreg_scalar(rd.to_reg(), mb_rru, size.lane_size());
                let rn = show_vreg_vector(rn, mb_rru, size);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecShiftImm {
                op,
                rd,
                rn,
                size,
                imm,
            } => {
                let op = match op {
                    VecShiftImmOp::Shl => "shl",
                    VecShiftImmOp::Ushr => "ushr",
                    VecShiftImmOp::Sshr => "sshr",
                };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = show_vreg_vector(rn, mb_rru, size);
                format!("{} {}, {}, #{}", op, rd, rn, imm)
            }
            &Inst::VecExtract { rd, rn, rm, imm4 } => {
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, VectorSize::Size8x16);
                let rn = show_vreg_vector(rn, mb_rru, VectorSize::Size8x16);
                let rm = show_vreg_vector(rm, mb_rru, VectorSize::Size8x16);
                format!("ext {}, {}, {}, #{}", rd, rn, rm, imm4)
            }
            &Inst::VecTbl {
                rd,
                rn,
                rm,
                is_extension,
            } => {
                let op = if is_extension { "tbx" } else { "tbl" };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, VectorSize::Size8x16);
                let rn = show_vreg_vector(rn, mb_rru, VectorSize::Size8x16);
                let rm = show_vreg_vector(rm, mb_rru, VectorSize::Size8x16);
                format!("{} {}, {{ {} }}, {}", op, rd, rn, rm)
            }
            &Inst::VecTbl2 {
                rd,
                rn,
                rn2,
                rm,
                is_extension,
            } => {
                let op = if is_extension { "tbx" } else { "tbl" };
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, VectorSize::Size8x16);
                let rn = show_vreg_vector(rn, mb_rru, VectorSize::Size8x16);
                let rn2 = show_vreg_vector(rn2, mb_rru, VectorSize::Size8x16);
                let rm = show_vreg_vector(rm, mb_rru, VectorSize::Size8x16);
                format!("{} {}, {{ {}, {} }}, {}", op, rd, rn, rn2, rm)
            }
            &Inst::VecLoadReplicate { rd, rn, size, .. } => {
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, size);
                let rn = rn.show_rru(mb_rru);

                format!("ld1r {{ {} }}, [{}]", rd, rn)
            }
            &Inst::VecCSel { rd, rn, rm, cond } => {
                let rd = show_vreg_vector(rd.to_reg(), mb_rru, VectorSize::Size8x16);
                let rn = show_vreg_vector(rn, mb_rru, VectorSize::Size8x16);
                let rm = show_vreg_vector(rm, mb_rru, VectorSize::Size8x16);
                let cond = cond.show_rru(mb_rru);
                format!(
                    "vcsel {}, {}, {}, {} (if-then-else diamond)",
                    rd, rn, rm, cond
                )
            }
            &Inst::MovToNZCV { rn } => {
                let rn = rn.show_rru(mb_rru);
                format!("msr nzcv, {}", rn)
            }
            &Inst::MovFromNZCV { rd } => {
                let rd = rd.to_reg().show_rru(mb_rru);
                format!("mrs {}, nzcv", rd)
            }
            &Inst::Extend {
                rd,
                rn,
                signed: false,
                from_bits: 1,
                ..
            } => {
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, OperandSize::Size32);
                let rn = show_ireg_sized(rn, mb_rru, OperandSize::Size32);
                format!("and {}, {}, #1", rd, rn)
            }
            &Inst::Extend {
                rd,
                rn,
                signed: false,
                from_bits: 32,
                to_bits: 64,
            } => {
                // The case of a zero extension from 32 to 64 bits, is implemented
                // with a "mov" to a 32-bit (W-reg) dest, because this zeroes
                // the top 32 bits.
                let rd = show_ireg_sized(rd.to_reg(), mb_rru, OperandSize::Size32);
                let rn = show_ireg_sized(rn, mb_rru, OperandSize::Size32);
                format!("mov {}, {}", rd, rn)
            }
            &Inst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits,
            } => {
                assert!(from_bits <= to_bits);
                let op = match (signed, from_bits) {
                    (false, 8) => "uxtb",
                    (true, 8) => "sxtb",
                    (false, 16) => "uxth",
                    (true, 16) => "sxth",
                    (true, 32) => "sxtw",
                    (true, _) => "sbfx",
                    (false, _) => "ubfx",
                };
                if op == "sbfx" || op == "ubfx" {
                    let dest_size = OperandSize::from_bits(to_bits);
                    let rd = show_ireg_sized(rd.to_reg(), mb_rru, dest_size);
                    let rn = show_ireg_sized(rn, mb_rru, dest_size);
                    format!("{} {}, {}, #0, #{}", op, rd, rn, from_bits)
                } else {
                    let dest_size = if signed {
                        OperandSize::from_bits(to_bits)
                    } else {
                        OperandSize::Size32
                    };
                    let rd = show_ireg_sized(rd.to_reg(), mb_rru, dest_size);
                    let rn = show_ireg_sized(rn, mb_rru, OperandSize::from_bits(from_bits));
                    format!("{} {}, {}", op, rd, rn)
                }
            }
            &Inst::Call { .. } => format!("bl 0"),
            &Inst::CallInd { ref info, .. } => {
                let rn = info.rn.show_rru(mb_rru);
                format!("blr {}", rn)
            }
            &Inst::Ret => "ret".to_string(),
            &Inst::EpiloguePlaceholder => "epilogue placeholder".to_string(),
            &Inst::Jump { ref dest } => {
                let dest = dest.show_rru(mb_rru);
                format!("b {}", dest)
            }
            &Inst::CondBr {
                ref taken,
                ref not_taken,
                ref kind,
            } => {
                let taken = taken.show_rru(mb_rru);
                let not_taken = not_taken.show_rru(mb_rru);
                match kind {
                    &CondBrKind::Zero(reg) => {
                        let reg = reg.show_rru(mb_rru);
                        format!("cbz {}, {} ; b {}", reg, taken, not_taken)
                    }
                    &CondBrKind::NotZero(reg) => {
                        let reg = reg.show_rru(mb_rru);
                        format!("cbnz {}, {} ; b {}", reg, taken, not_taken)
                    }
                    &CondBrKind::Cond(c) => {
                        let c = c.show_rru(mb_rru);
                        format!("b.{} {} ; b {}", c, taken, not_taken)
                    }
                }
            }
            &Inst::IndirectBr { rn, .. } => {
                let rn = rn.show_rru(mb_rru);
                format!("br {}", rn)
            }
            &Inst::Brk => "brk #0".to_string(),
            &Inst::Udf { .. } => "udf".to_string(),
            &Inst::TrapIf { ref kind, .. } => match kind {
                &CondBrKind::Zero(reg) => {
                    let reg = reg.show_rru(mb_rru);
                    format!("cbnz {}, 8 ; udf", reg)
                }
                &CondBrKind::NotZero(reg) => {
                    let reg = reg.show_rru(mb_rru);
                    format!("cbz {}, 8 ; udf", reg)
                }
                &CondBrKind::Cond(c) => {
                    let c = c.invert().show_rru(mb_rru);
                    format!("b.{} 8 ; udf", c)
                }
            },
            &Inst::Adr { rd, off } => {
                let rd = rd.show_rru(mb_rru);
                format!("adr {}, pc+{}", rd, off)
            }
            &Inst::Word4 { data } => format!("data.i32 {}", data),
            &Inst::Word8 { data } => format!("data.i64 {}", data),
            &Inst::JTSequence {
                ref info,
                ridx,
                rtmp1,
                rtmp2,
                ..
            } => {
                let ridx = ridx.show_rru(mb_rru);
                let rtmp1 = rtmp1.show_rru(mb_rru);
                let rtmp2 = rtmp2.show_rru(mb_rru);
                let default_target = info.default_target.show_rru(mb_rru);
                format!(
                    concat!(
                        "b.hs {} ; ",
                        "adr {}, pc+16 ; ",
                        "ldrsw {}, [{}, {}, LSL 2] ; ",
                        "add {}, {}, {} ; ",
                        "br {} ; ",
                        "jt_entries {:?}"
                    ),
                    default_target,
                    rtmp1,
                    rtmp2,
                    rtmp1,
                    ridx,
                    rtmp1,
                    rtmp1,
                    rtmp2,
                    rtmp1,
                    info.targets
                )
            }
            &Inst::LoadExtName {
                rd,
                ref name,
                offset,
            } => {
                let rd = rd.show_rru(mb_rru);
                format!("ldr {}, 8 ; b 12 ; data {:?} + {}", rd, name, offset)
            }
            &Inst::LoadAddr { rd, ref mem } => {
                // TODO: we really should find a better way to avoid duplication of
                // this logic between `emit()` and `show_rru()` -- a separate 1-to-N
                // expansion stage (i.e., legalization, but without the slow edit-in-place
                // of the existing legalization framework).
                let (mem_insts, mem) = mem_finalize(0, mem, state);
                let mut ret = String::new();
                for inst in mem_insts.into_iter() {
                    ret.push_str(&inst.show_rru(mb_rru));
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

                    ret.push_str(&add.show_rru(mb_rru));
                } else if offset == 0 {
                    let mov = Inst::gen_move(rd, reg, I64);
                    ret.push_str(&mov.show_rru(mb_rru));
                } else if let Some(imm12) = Imm12::maybe_from_u64(abs_offset) {
                    let add = Inst::AluRRImm12 {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        imm12,
                    };
                    ret.push_str(&add.show_rru(mb_rru));
                } else {
                    let tmp = writable_spilltmp_reg();
                    for inst in Inst::load_constant(tmp, abs_offset).into_iter() {
                        ret.push_str(&inst.show_rru(mb_rru));
                    }
                    let add = Inst::AluRRR {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        rm: tmp.to_reg(),
                    };
                    ret.push_str(&add.show_rru(mb_rru));
                }
                ret
            }
            &Inst::VirtualSPOffsetAdj { offset } => {
                state.virtual_sp_offset += offset;
                format!("virtual_sp_offset_adjust {}", offset)
            }
            &Inst::EmitIsland { needed_space } => format!("emit_island {}", needed_space),

            &Inst::ElfTlsGetAddr { ref symbol } => {
                format!("elf_tls_get_addr {}", symbol)
            }

            &Inst::ValueLabelMarker { label, reg } => {
                format!("value_label {:?}, {}", label, reg.show_rru(mb_rru))
            }

            &Inst::Unwind { ref inst } => {
                format!("unwind {:?}", inst)
            }
        }
    }
}

//=============================================================================
// Label fixups and jump veneers.

/// Different forms of label references for different instruction formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUse {
    /// 19-bit branch offset (conditional branches). PC-rel, offset is imm << 2. Immediate is 19
    /// signed bits, in bits 23:5. Used by cbz, cbnz, b.cond.
    Branch19,
    /// 26-bit branch offset (unconditional branches). PC-rel, offset is imm << 2. Immediate is 26
    /// signed bits, in bits 25:0. Used by b, bl.
    Branch26,
    /// 19-bit offset for LDR (load literal). PC-rel, offset is imm << 2. Immediate is 19 signed bits,
    /// in bits 23:5.
    Ldr19,
    /// 21-bit offset for ADR (get address of label). PC-rel, offset is not shifted. Immediate is
    /// 21 signed bits, with high 19 bits in bits 23:5 and low 2 bits in bits 30:29.
    Adr21,
    /// 32-bit PC relative constant offset (from address of constant itself),
    /// signed. Used in jump tables.
    PCRel32,
}

impl MachInstLabelUse for LabelUse {
    /// Alignment for veneer code. Every AArch64 instruction must be 4-byte-aligned.
    const ALIGN: CodeOffset = 4;

    /// Maximum PC-relative range (positive), inclusive.
    fn max_pos_range(self) -> CodeOffset {
        match self {
            // 19-bit immediate, left-shifted by 2, for 21 bits of total range. Signed, so +2^20
            // from zero. Likewise for two other shifted cases below.
            LabelUse::Branch19 => (1 << 20) - 1,
            LabelUse::Branch26 => (1 << 27) - 1,
            LabelUse::Ldr19 => (1 << 20) - 1,
            // Adr does not shift its immediate, so the 21-bit immediate gives 21 bits of total
            // range.
            LabelUse::Adr21 => (1 << 20) - 1,
            LabelUse::PCRel32 => 0x7fffffff,
        }
    }

    /// Maximum PC-relative range (negative).
    fn max_neg_range(self) -> CodeOffset {
        // All forms are twos-complement signed offsets, so negative limit is one more than
        // positive limit.
        self.max_pos_range() + 1
    }

    /// Size of window into code needed to do the patch.
    fn patch_size(self) -> CodeOffset {
        // Patch is on one instruction only for all of these label reference types.
        4
    }

    /// Perform the patch.
    fn patch(self, buffer: &mut [u8], use_offset: CodeOffset, label_offset: CodeOffset) {
        let pc_rel = (label_offset as i64) - (use_offset as i64);
        debug_assert!(pc_rel <= self.max_pos_range() as i64);
        debug_assert!(pc_rel >= -(self.max_neg_range() as i64));
        let pc_rel = pc_rel as u32;
        let insn_word = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let mask = match self {
            LabelUse::Branch19 => 0x00ffffe0, // bits 23..5 inclusive
            LabelUse::Branch26 => 0x03ffffff, // bits 25..0 inclusive
            LabelUse::Ldr19 => 0x00ffffe0,    // bits 23..5 inclusive
            LabelUse::Adr21 => 0x60ffffe0,    // bits 30..29, 25..5 inclusive
            LabelUse::PCRel32 => 0xffffffff,
        };
        let pc_rel_shifted = match self {
            LabelUse::Adr21 | LabelUse::PCRel32 => pc_rel,
            _ => {
                debug_assert!(pc_rel & 3 == 0);
                pc_rel >> 2
            }
        };
        let pc_rel_inserted = match self {
            LabelUse::Branch19 | LabelUse::Ldr19 => (pc_rel_shifted & 0x7ffff) << 5,
            LabelUse::Branch26 => pc_rel_shifted & 0x3ffffff,
            LabelUse::Adr21 => (pc_rel_shifted & 0x7ffff) << 5 | (pc_rel_shifted & 0x180000) << 10,
            LabelUse::PCRel32 => pc_rel_shifted,
        };
        let is_add = match self {
            LabelUse::PCRel32 => true,
            _ => false,
        };
        let insn_word = if is_add {
            insn_word.wrapping_add(pc_rel_inserted)
        } else {
            (insn_word & !mask) | pc_rel_inserted
        };
        buffer[0..4].clone_from_slice(&u32::to_le_bytes(insn_word));
    }

    /// Is a veneer supported for this label reference type?
    fn supports_veneer(self) -> bool {
        match self {
            LabelUse::Branch19 => true, // veneer is a Branch26
            LabelUse::Branch26 => true, // veneer is a PCRel32
            _ => false,
        }
    }

    /// How large is the veneer, if supported?
    fn veneer_size(self) -> CodeOffset {
        match self {
            LabelUse::Branch19 => 4,
            LabelUse::Branch26 => 20,
            _ => unreachable!(),
        }
    }

    /// Generate a veneer into the buffer, given that this veneer is at `veneer_offset`, and return
    /// an offset and label-use for the veneer's use of the original label.
    fn generate_veneer(
        self,
        buffer: &mut [u8],
        veneer_offset: CodeOffset,
    ) -> (CodeOffset, LabelUse) {
        match self {
            LabelUse::Branch19 => {
                // veneer is a Branch26 (unconditional branch). Just encode directly here -- don't
                // bother with constructing an Inst.
                let insn_word = 0b000101 << 26;
                buffer[0..4].clone_from_slice(&u32::to_le_bytes(insn_word));
                (veneer_offset, LabelUse::Branch26)
            }

            // This is promoting a 26-bit call/jump to a 32-bit call/jump to
            // get a further range. This jump translates to a jump to a
            // relative location based on the address of the constant loaded
            // from here.
            //
            // If this path is taken from a call instruction then caller-saved
            // registers are available (minus arguments), so x16/x17 are
            // available. Otherwise for intra-function jumps we also reserve
            // x16/x17 as spill-style registers. In both cases these are
            // available for us to use.
            LabelUse::Branch26 => {
                let tmp1 = regs::spilltmp_reg();
                let tmp1_w = regs::writable_spilltmp_reg();
                let tmp2 = regs::tmp2_reg();
                let tmp2_w = regs::writable_tmp2_reg();
                // ldrsw x16, 16
                let ldr = emit::enc_ldst_imm19(0b1001_1000, 16 / 4, tmp1);
                // adr x17, 12
                let adr = emit::enc_adr(12, tmp2_w);
                // add x16, x16, x17
                let add = emit::enc_arith_rrr(0b10001011_000, 0, tmp1_w, tmp1, tmp2);
                // br x16
                let br = emit::enc_br(tmp1);
                buffer[0..4].clone_from_slice(&u32::to_le_bytes(ldr));
                buffer[4..8].clone_from_slice(&u32::to_le_bytes(adr));
                buffer[8..12].clone_from_slice(&u32::to_le_bytes(add));
                buffer[12..16].clone_from_slice(&u32::to_le_bytes(br));
                // the 4-byte signed immediate we'll load is after these
                // instructions, 16-bytes in.
                (veneer_offset + 16, LabelUse::PCRel32)
            }

            _ => panic!("Unsupported label-reference type for veneer generation!"),
        }
    }

    fn from_reloc(reloc: Reloc, addend: Addend) -> Option<LabelUse> {
        match (reloc, addend) {
            (Reloc::Arm64Call, 0) => Some(LabelUse::Branch26),
            _ => None,
        }
    }
}
