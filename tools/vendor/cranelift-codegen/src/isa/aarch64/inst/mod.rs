//! This module defines aarch64-specific machine instruction types.

use crate::binemit::{Addend, CodeOffset, Reloc};
use crate::ir::types::{F32, F64, FFLAGS, I128, I16, I32, I64, I8, I8X16, IFLAGS, R32, R64};
use crate::ir::{types, ExternalName, MemFlags, Opcode, Type};
use crate::isa::CallConv;
use crate::machinst::*;
use crate::{settings, CodegenError, CodegenResult};

use crate::machinst::{PrettyPrint, Reg, RegClass, Writable};

use alloc::vec::Vec;
use core::convert::TryFrom;
use regalloc2::{PRegSet, VReg};
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
    ALUOp, ALUOp3, AMode, APIKey, AtomicRMWLoopOp, AtomicRMWOp, BitOp, BranchTargetType, FPUOp1,
    FPUOp2, FPUOp3, FpuRoundMode, FpuToIntOp, IntToFpuOp, MInst as Inst, MoveWideOp, VecALUModOp,
    VecALUOp, VecExtendOp, VecLanesOp, VecMisc2, VecPairOp, VecRRLongOp, VecRRNarrowOp,
    VecRRPairLongOp, VecRRRLongModOp, VecRRRLongOp, VecShiftImmModOp, VecShiftImmOp,
};

/// A floating-point unit (FPU) operation with two args, a register and an immediate.
#[derive(Copy, Clone, Debug)]
pub enum FPUOpRI {
    /// Unsigned right shift. Rd = Rn << #imm
    UShr32(FPURightShiftImm),
    /// Unsigned right shift. Rd = Rn << #imm
    UShr64(FPURightShiftImm),
}

/// A floating-point unit (FPU) operation with two args, a register and
/// an immediate that modifies its dest (so takes that input value as a
/// separate virtual register).
#[derive(Copy, Clone, Debug)]
pub enum FPUOpRIMod {
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
            BitOp::Rev16 => "rev16",
            BitOp::Rev32 => "rev32",
            BitOp::Rev64 => "rev64",
        }
    }
}

/// Additional information for (direct) Call instructions, left out of line to lower the size of
/// the Inst enum.
#[derive(Clone, Debug)]
pub struct CallInfo {
    pub dest: ExternalName,
    pub uses: CallArgList,
    pub defs: CallRetList,
    pub clobbers: PRegSet,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
}

/// Additional information for CallInd instructions, left out of line to lower the size of the Inst
/// enum.
#[derive(Clone, Debug)]
pub struct CallIndInfo {
    pub rn: Reg,
    pub uses: SmallVec<[CallArgPair; 8]>,
    pub defs: SmallVec<[CallRetPair; 8]>,
    pub clobbers: PRegSet,
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
    pub fn load_constant<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        value: u64,
        alloc_tmp: &mut F,
    ) -> SmallVec<[Inst; 4]> {
        // NB: this is duplicated in `lower/isle.rs` and `inst.isle` right now,
        // if modifications are made here before this is deleted after moving to
        // ISLE then those locations should be updated as well.

        if let Some(imm) = MoveWideConst::maybe_from_u64(value) {
            // 16-bit immediate (shifted by 0, 16, 32 or 48 bits) in MOVZ
            smallvec![Inst::MovWide {
                op: MoveWideOp::MovZ,
                rd,
                imm,
                size: OperandSize::Size64
            }]
        } else if let Some(imm) = MoveWideConst::maybe_from_u64(!value) {
            // 16-bit immediate (shifted by 0, 16, 32 or 48 bits) in MOVN
            smallvec![Inst::MovWide {
                op: MoveWideOp::MovN,
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

            let halfwords: SmallVec<[_; 4]> = (0..num_half_words)
                .filter_map(|i| {
                    let imm16 = (value >> (16 * i)) & 0xffff;
                    if imm16 == ignored_halfword {
                        None
                    } else {
                        Some((i, imm16))
                    }
                })
                .collect();

            let mut prev_result = None;
            let last_index = halfwords.last().unwrap().0;
            for (i, imm16) in halfwords {
                let shift = i * 16;
                let rd = if i == last_index { rd } else { alloc_tmp(I16) };

                if let Some(rn) = prev_result {
                    let imm = MoveWideConst::maybe_with_shift(imm16 as u16, shift).unwrap();
                    insts.push(Inst::MovK { rd, rn, imm, size });
                } else {
                    if first_is_inverted {
                        let imm =
                            MoveWideConst::maybe_with_shift(((!imm16) & 0xffff) as u16, shift)
                                .unwrap();
                        insts.push(Inst::MovWide {
                            op: MoveWideOp::MovN,
                            rd,
                            imm,
                            size,
                        });
                    } else {
                        let imm = MoveWideConst::maybe_with_shift(imm16 as u16, shift).unwrap();
                        insts.push(Inst::MovWide {
                            op: MoveWideOp::MovZ,
                            rd,
                            imm,
                            size,
                        });
                    }
                }

                prev_result = Some(rd.to_reg());
            }

            assert!(prev_result.is_some());

            insts
        }
    }

    /// Create instructions that load a 128-bit constant.
    pub fn load_constant128<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        assert_eq!(to_regs.len(), 2, "Expected to load i128 into two registers");

        let lower = value as u64;
        let upper = (value >> 64) as u64;

        let lower_reg = to_regs.regs()[0];
        let upper_reg = to_regs.regs()[1];

        let mut load_ins = Inst::load_constant(lower_reg, lower, &mut alloc_tmp);
        let load_upper = Inst::load_constant(upper_reg, upper, &mut alloc_tmp);

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
            let mut insts = Inst::load_constant(tmp, const_data as u64, &mut alloc_tmp);

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
            let mut insts = Inst::load_constant(tmp, const_data, &mut alloc_tmp);

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
            let mut insts = SmallVec::from(&Inst::load_constant(tmp, pattern, &mut alloc_tmp)[..]);

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
            I8 => Inst::ULoad8 {
                rd: into_reg,
                mem,
                flags,
            },
            I16 => Inst::ULoad16 {
                rd: into_reg,
                mem,
                flags,
            },
            I32 | R32 => Inst::ULoad32 {
                rd: into_reg,
                mem,
                flags,
            },
            I64 | R64 => Inst::ULoad64 {
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
            I8 => Inst::Store8 {
                rd: from_reg,
                mem,
                flags,
            },
            I16 => Inst::Store16 {
                rd: from_reg,
                mem,
                flags,
            },
            I32 | R32 => Inst::Store32 {
                rd: from_reg,
                mem,
                flags,
            },
            I64 | R64 => Inst::Store64 {
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
}

//=============================================================================
// Instructions: get_regs

fn memarg_operands<F: Fn(VReg) -> VReg>(memarg: &AMode, collector: &mut OperandCollector<'_, F>) {
    // This should match `AMode::with_allocs()`.
    match memarg {
        &AMode::Unscaled { rn, .. } | &AMode::UnsignedOffset { rn, .. } => {
            collector.reg_use(rn);
        }
        &AMode::RegReg { rn, rm, .. }
        | &AMode::RegScaled { rn, rm, .. }
        | &AMode::RegScaledExtended { rn, rm, .. }
        | &AMode::RegExtended { rn, rm, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &AMode::Label { .. } => {}
        &AMode::SPPreIndexed { .. } | &AMode::SPPostIndexed { .. } => {}
        &AMode::FPOffset { .. } => {}
        &AMode::SPOffset { .. } | &AMode::NominalSPOffset { .. } => {}
        &AMode::RegOffset { rn, .. } => {
            collector.reg_use(rn);
        }
    }
}

fn pairmemarg_operands<F: Fn(VReg) -> VReg>(
    pairmemarg: &PairAMode,
    collector: &mut OperandCollector<'_, F>,
) {
    // This should match `PairAMode::with_allocs()`.
    match pairmemarg {
        &PairAMode::SignedOffset(reg, ..) => {
            collector.reg_use(reg);
        }
        &PairAMode::SPPreIndexed(..) | &PairAMode::SPPostIndexed(..) => {}
    }
}

fn aarch64_get_operands<F: Fn(VReg) -> VReg>(inst: &Inst, collector: &mut OperandCollector<'_, F>) {
    match inst {
        &Inst::AluRRR { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::AluRRRR { rd, rn, rm, ra, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_use(ra);
        }
        &Inst::AluRRImm12 { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::AluRRImmLogic { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::AluRRImmShift { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::AluRRRShift { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::AluRRRExtend { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::BitRR { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::ULoad8 { rd, ref mem, .. }
        | &Inst::SLoad8 { rd, ref mem, .. }
        | &Inst::ULoad16 { rd, ref mem, .. }
        | &Inst::SLoad16 { rd, ref mem, .. }
        | &Inst::ULoad32 { rd, ref mem, .. }
        | &Inst::SLoad32 { rd, ref mem, .. }
        | &Inst::ULoad64 { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::Store8 { rd, ref mem, .. }
        | &Inst::Store16 { rd, ref mem, .. }
        | &Inst::Store32 { rd, ref mem, .. }
        | &Inst::Store64 { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::StoreP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_use(rt);
            collector.reg_use(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::LoadP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_def(rt);
            collector.reg_def(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::Mov { rd, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rm);
        }
        &Inst::MovFromPReg { rd, rm } => {
            debug_assert!(rd.to_reg().is_virtual());
            collector.reg_def(rd);
            collector.reg_fixed_nonallocatable(rm);
        }
        &Inst::MovToPReg { rd, rm } => {
            debug_assert!(rm.is_virtual());
            collector.reg_fixed_nonallocatable(rd);
            collector.reg_use(rm);
        }
        &Inst::MovK { rd, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_reuse_def(rd, 0); // `rn` == `rd`.
        }
        &Inst::MovWide { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::CSel { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::CSNeg { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::CSet { rd, .. } | &Inst::CSetm { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::CCmp { rn, rm, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::CCmpImm { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::AtomicRMWLoop {
            op,
            addr,
            operand,
            oldval,
            scratch1,
            scratch2,
            ..
        } => {
            collector.reg_fixed_use(addr, xreg(25));
            collector.reg_fixed_use(operand, xreg(26));
            collector.reg_fixed_def(oldval, xreg(27));
            collector.reg_fixed_def(scratch1, xreg(24));
            if op != AtomicRMWLoopOp::Xchg {
                collector.reg_fixed_def(scratch2, xreg(28));
            }
        }
        &Inst::AtomicRMW { rs, rt, rn, .. } => {
            collector.reg_use(rs);
            collector.reg_def(rt);
            collector.reg_use(rn);
        }
        &Inst::AtomicCAS { rd, rs, rt, rn, .. } => {
            collector.reg_reuse_def(rd, 1); // reuse `rs`.
            collector.reg_use(rs);
            collector.reg_use(rt);
            collector.reg_use(rn);
        }
        &Inst::AtomicCASLoop {
            addr,
            expected,
            replacement,
            oldval,
            scratch,
            ..
        } => {
            collector.reg_fixed_use(addr, xreg(25));
            collector.reg_fixed_use(expected, xreg(26));
            collector.reg_fixed_use(replacement, xreg(28));
            collector.reg_fixed_def(oldval, xreg(27));
            collector.reg_fixed_def(scratch, xreg(24));
        }
        &Inst::LoadAcquire { rt, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_def(rt);
        }
        &Inst::StoreRelease { rt, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rt);
        }
        &Inst::Fence {} | &Inst::Csdb {} => {}
        &Inst::FpuMove64 { rd, rn } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuMove128 { rd, rn } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuMoveFromVec { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuExtend { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuRR { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuRRR { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::FpuRRI { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuRRIMod { rd, ri, rn, .. } => {
            collector.reg_reuse_def(rd, 1); // reuse `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
        }
        &Inst::FpuRRRR { rd, rn, rm, ra, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_use(ra);
        }
        &Inst::VecMisc { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }

        &Inst::VecLanes { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecShiftImm { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecShiftImmMod { rd, ri, rn, .. } => {
            collector.reg_reuse_def(rd, 1); // `rd` == `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
        }
        &Inst::VecExtract { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecTbl { rd, rn, rm } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_def(rd);
        }
        &Inst::VecTblExt { rd, ri, rn, rm } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_reuse_def(rd, 3); // `rd` == `ri`.
            collector.reg_use(ri);
        }

        &Inst::VecTbl2 { rd, rn, rn2, rm } => {
            // Constrain to v30 / v31 so that we satisfy the "adjacent
            // registers" constraint without use of pinned vregs in
            // lowering.
            collector.reg_fixed_use(rn, vreg(30));
            collector.reg_fixed_use(rn2, vreg(31));
            collector.reg_use(rm);
            collector.reg_def(rd);
        }
        &Inst::VecTbl2Ext {
            rd,
            ri,
            rn,
            rn2,
            rm,
        } => {
            // Constrain to v30 / v31 so that we satisfy the "adjacent
            // registers" constraint without use of pinned vregs in
            // lowering.
            collector.reg_fixed_use(rn, vreg(30));
            collector.reg_fixed_use(rn2, vreg(31));
            collector.reg_use(rm);
            collector.reg_reuse_def(rd, 4); // `rd` == `ri`.
            collector.reg_use(ri);
        }
        &Inst::VecLoadReplicate { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecCSel { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::FpuCmp { rn, rm, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::FpuLoad32 { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuLoad64 { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuLoad128 { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuStore32 { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuStore64 { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuStore128 { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::FpuLoadP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_def(rt);
            collector.reg_def(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::FpuStoreP64 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_use(rt);
            collector.reg_use(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::FpuLoadP128 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_def(rt);
            collector.reg_def(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::FpuStoreP128 {
            rt, rt2, ref mem, ..
        } => {
            collector.reg_use(rt);
            collector.reg_use(rt2);
            pairmemarg_operands(mem, collector);
        }
        &Inst::LoadFpuConst64 { rd, .. } | &Inst::LoadFpuConst128 { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::FpuToInt { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::IntToFpu { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuCSel32 { rd, rn, rm, .. } | &Inst::FpuCSel64 { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::FpuRound { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::MovToFpu { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuMoveFPImm { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::MovToVec { rd, ri, rn, .. } => {
            collector.reg_reuse_def(rd, 1); // `rd` == `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
        }
        &Inst::MovFromVec { rd, rn, .. } | &Inst::MovFromVecSigned { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecDup { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecDupFromFpu { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecDupFPImm { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::VecDupImm { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::VecExtend { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecMovElement { rd, ri, rn, .. } => {
            collector.reg_reuse_def(rd, 1); // `rd` == `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
        }
        &Inst::VecRRLong { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecRRNarrowLow { rd, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_def(rd);
        }
        &Inst::VecRRNarrowHigh { rd, ri, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_reuse_def(rd, 2); // `rd` == `ri`.
            collector.reg_use(ri);
        }
        &Inst::VecRRPair { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecRRRLong { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecRRRLongMod { rd, ri, rn, rm, .. } => {
            collector.reg_reuse_def(rd, 1); // `rd` == `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecRRPairLong { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecRRR { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecRRRMod { rd, ri, rn, rm, .. } => {
            collector.reg_reuse_def(rd, 1); // `rd` == `ri`.
            collector.reg_use(ri);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::MovToNZCV { rn } => {
            collector.reg_use(rn);
        }
        &Inst::MovFromNZCV { rd } => {
            collector.reg_def(rd);
        }
        &Inst::Extend { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::Args { ref args } => {
            for arg in args {
                collector.reg_fixed_def(arg.vreg, arg.preg);
            }
        }
        &Inst::Ret { ref rets } | &Inst::AuthenticatedRet { ref rets, .. } => {
            for ret in rets {
                collector.reg_fixed_use(ret.vreg, ret.preg);
            }
        }
        &Inst::Jump { .. } => {}
        &Inst::Call { ref info, .. } => {
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }
        &Inst::CallInd { ref info, .. } => {
            collector.reg_use(info.rn);
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }
        &Inst::CondBr { ref kind, .. } => match kind {
            CondBrKind::Zero(rt) | CondBrKind::NotZero(rt) => {
                collector.reg_use(*rt);
            }
            CondBrKind::Cond(_) => {}
        },
        &Inst::IndirectBr { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::Nop0 | Inst::Nop4 => {}
        &Inst::Brk => {}
        &Inst::Udf { .. } => {}
        &Inst::TrapIf { ref kind, .. } => match kind {
            CondBrKind::Zero(rt) | CondBrKind::NotZero(rt) => {
                collector.reg_use(*rt);
            }
            CondBrKind::Cond(_) => {}
        },
        &Inst::Adr { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::Word4 { .. } | &Inst::Word8 { .. } => {}
        &Inst::JTSequence {
            ridx, rtmp1, rtmp2, ..
        } => {
            collector.reg_use(ridx);
            collector.reg_early_def(rtmp1);
            collector.reg_early_def(rtmp2);
        }
        &Inst::LoadExtName { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::LoadAddr { rd, ref mem } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::Pacisp { .. } | &Inst::Xpaclri => {
            // Neither LR nor SP is an allocatable register, so there is no need
            // to do anything.
        }
        &Inst::Bti { .. } => {}
        &Inst::VirtualSPOffsetAdj { .. } => {}

        &Inst::ElfTlsGetAddr { rd, .. } => {
            collector.reg_fixed_def(rd, regs::xreg(0));
            let mut clobbers = AArch64MachineDeps::get_regs_clobbered_by_call(CallConv::SystemV);
            clobbers.remove(regs::xreg_preg(0));
            collector.reg_clobbers(clobbers);
        }
        &Inst::Unwind { .. } => {}
        &Inst::EmitIsland { .. } => {}
        &Inst::DummyUse { reg } => {
            collector.reg_use(reg);
        }
        &Inst::StackProbeLoop { start, end, .. } => {
            collector.reg_early_def(start);
            collector.reg_use(end);
        }
    }
}

//=============================================================================
// Instructions: misc functions and external interface

impl MachInst for Inst {
    type ABIMachineSpec = AArch64MachineDeps;
    type LabelUse = LabelUse;

    fn get_operands<F: Fn(VReg) -> VReg>(&self, collector: &mut OperandCollector<'_, F>) {
        aarch64_get_operands(self, collector);
    }

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        match self {
            &Inst::Mov {
                size: OperandSize::Size64,
                rd,
                rm,
            } => Some((rd, rm)),
            &Inst::FpuMove64 { rd, rn } => Some((rd, rn)),
            &Inst::FpuMove128 { rd, rn } => Some((rd, rn)),
            _ => None,
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
            &Inst::Args { .. } => false,
            &Inst::Call { ref info } => info.caller_callconv != info.callee_callconv,
            &Inst::CallInd { ref info } => info.caller_callconv != info.callee_callconv,
            _ => true,
        }
    }

    fn is_args(&self) -> bool {
        match self {
            Self::Args { .. } => true,
            _ => false,
        }
    }

    fn is_term(&self) -> MachTerminator {
        match self {
            &Inst::Ret { .. } | &Inst::AuthenticatedRet { .. } => MachTerminator::Ret,
            &Inst::Jump { .. } => MachTerminator::Uncond,
            &Inst::CondBr { .. } => MachTerminator::Cond,
            &Inst::IndirectBr { .. } => MachTerminator::Indirect,
            &Inst::JTSequence { .. } => MachTerminator::Indirect,
            _ => MachTerminator::None,
        }
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Inst {
        let bits = ty.bits();

        assert!(bits <= 128);
        assert!(to_reg.to_reg().class() == from_reg.class());
        match from_reg.class() {
            RegClass::Int => Inst::Mov {
                size: OperandSize::Size64,
                rd: to_reg,
                rm: from_reg,
            },
            RegClass::Float => {
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
            }
        }
    }

    fn is_safepoint(&self) -> bool {
        match self {
            &Inst::Call { .. }
            | &Inst::CallInd { .. }
            | &Inst::TrapIf { .. }
            | &Inst::Udf { .. } => true,
            _ => false,
        }
    }

    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        let to_reg = to_regs.only_reg();
        match ty {
            F64 => Inst::load_fp_constant64(to_reg.unwrap(), value as u64, alloc_tmp),
            F32 => Inst::load_fp_constant32(to_reg.unwrap(), value as u32, alloc_tmp),
            I8 | I16 | I32 | I64 | R32 | R64 => {
                Inst::load_constant(to_reg.unwrap(), value as u64, &mut alloc_tmp)
            }
            I128 => Inst::load_constant128(to_regs, value, alloc_tmp),
            _ => panic!("Cannot generate constant for type: {}", ty),
        }
    }

    fn gen_dummy_use(reg: Reg) -> Inst {
        Inst::DummyUse { reg }
    }

    fn gen_nop(preferred_size: usize) -> Inst {
        if preferred_size == 0 {
            return Inst::Nop0;
        }
        // We can't give a NOP (or any insn) < 4 bytes.
        assert!(preferred_size >= 4);
        Inst::Nop4
    }

    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
        match ty {
            I8 => Ok((&[RegClass::Int], &[I8])),
            I16 => Ok((&[RegClass::Int], &[I16])),
            I32 => Ok((&[RegClass::Int], &[I32])),
            I64 => Ok((&[RegClass::Int], &[I64])),
            R32 => panic!("32-bit reftype pointer should never be seen on AArch64"),
            R64 => Ok((&[RegClass::Int], &[R64])),
            F32 => Ok((&[RegClass::Float], &[F32])),
            F64 => Ok((&[RegClass::Float], &[F64])),
            I128 => Ok((&[RegClass::Int, RegClass::Int], &[I64, I64])),
            _ if ty.is_vector() => {
                assert!(ty.bits() <= 128);
                Ok((&[RegClass::Float], &[I8X16]))
            }
            _ if ty.is_dynamic_vector() => Ok((&[RegClass::Float], &[I8X16])),
            IFLAGS | FFLAGS => Ok((&[RegClass::Int], &[I64])),
            _ => Err(CodegenError::Unsupported(format!(
                "Unexpected SSA-value type: {}",
                ty
            ))),
        }
    }

    fn canonical_type_for_rc(rc: RegClass) -> Type {
        match rc {
            RegClass::Float => types::I8X16,
            RegClass::Int => types::I64,
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
        RegClass::Int
    }

    fn gen_block_start(
        is_indirect_branch_target: bool,
        is_forward_edge_cfi_enabled: bool,
    ) -> Option<Self> {
        if is_indirect_branch_target && is_forward_edge_cfi_enabled {
            Some(Inst::Bti {
                targets: BranchTargetType::J,
            })
        } else {
            None
        }
    }
}

//=============================================================================
// Pretty-printing of instructions.

fn mem_finalize_for_show(mem: &AMode, state: &EmitState) -> (String, AMode) {
    let (mem_insts, mem) = mem_finalize(0, mem, state);
    let mut mem_str = mem_insts
        .into_iter()
        .map(|inst| {
            inst.print_with_state(&mut EmitState::default(), &mut AllocationConsumer::new(&[]))
        })
        .collect::<Vec<_>>()
        .join(" ; ");
    if !mem_str.is_empty() {
        mem_str += " ; ";
    }

    (mem_str, mem)
}

impl Inst {
    fn print_with_state(&self, state: &mut EmitState, allocs: &mut AllocationConsumer) -> String {
        let mut empty_allocs = AllocationConsumer::default();

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

        // N.B.: order of `allocs` consumption (via register
        // pretty-printing or memarg.with_allocs()) needs to match the
        // order in `aarch64_get_operands` above.
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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::AluRRRR {
                alu_op,
                size,
                rd,
                rn,
                rm,
                ra,
            } => {
                let op = match alu_op {
                    ALUOp3::MAdd => "madd",
                    ALUOp3::MSub => "msub",
                };
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                let ra = pretty_print_ireg(ra, size, allocs);

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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);

                if imm12.bits == 0 && alu_op == ALUOp::Add && size.is64() {
                    // special-case MOV (used for moving into SP).
                    format!("mov {}, {}", rd, rn)
                } else {
                    let imm12 = imm12.pretty_print(0, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let imml = imml.pretty_print(0, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let immshift = immshift.pretty_print(0, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                let shiftop = shiftop.pretty_print(0, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                let extendop = extendop.pretty_print(0, allocs);
                format!("{} {}, {}, {}, {}", op, rd, rn, rm, extendop)
            }
            &Inst::BitRR { op, size, rd, rn } => {
                let op = op.op_str();
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::ULoad8 { rd, ref mem, .. }
            | &Inst::SLoad8 { rd, ref mem, .. }
            | &Inst::ULoad16 { rd, ref mem, .. }
            | &Inst::SLoad16 { rd, ref mem, .. }
            | &Inst::ULoad32 { rd, ref mem, .. }
            | &Inst::SLoad32 { rd, ref mem, .. }
            | &Inst::ULoad64 { rd, ref mem, .. } => {
                let is_unscaled = match &mem {
                    &AMode::Unscaled { .. } => true,
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

                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op, rd, mem)
            }
            &Inst::Store8 { rd, ref mem, .. }
            | &Inst::Store16 { rd, ref mem, .. }
            | &Inst::Store32 { rd, ref mem, .. }
            | &Inst::Store64 { rd, ref mem, .. } => {
                let is_unscaled = match &mem {
                    &AMode::Unscaled { .. } => true,
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

                let rd = pretty_print_ireg(rd, size, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op, rd, mem)
            }
            &Inst::StoreP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_ireg(rt, OperandSize::Size64, allocs);
                let rt2 = pretty_print_ireg(rt2, OperandSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();
                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::LoadP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_ireg(rt.to_reg(), OperandSize::Size64, allocs);
                let rt2 = pretty_print_ireg(rt2.to_reg(), OperandSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();
                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::Mov { size, rd, rm } => {
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                format!("mov {}, {}", rd, rm)
            }
            &Inst::MovFromPReg { rd, rm } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size64, allocs);
                allocs.next_fixed_nonallocatable(rm);
                let rm = show_ireg_sized(rm.into(), OperandSize::Size64);
                format!("mov {}, {}", rd, rm)
            }
            &Inst::MovToPReg { rd, rm } => {
                allocs.next_fixed_nonallocatable(rd);
                let rd = show_ireg_sized(rd.into(), OperandSize::Size64);
                let rm = pretty_print_ireg(rm, OperandSize::Size64, allocs);
                format!("mov {}, {}", rd, rm)
            }
            &Inst::MovWide {
                op,
                rd,
                ref imm,
                size,
            } => {
                let op_str = match op {
                    MoveWideOp::MovZ => "movz",
                    MoveWideOp::MovN => "movn",
                };
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let imm = imm.pretty_print(0, allocs);
                format!("{} {}, {}", op_str, rd, imm)
            }
            &Inst::MovK {
                rd,
                rn,
                ref imm,
                size,
            } => {
                let rn = pretty_print_ireg(rn, size, allocs);
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let imm = imm.pretty_print(0, allocs);
                format!("movk {}, {}, {}", rd, rn, imm)
            }
            &Inst::CSel { rd, rn, rm, cond } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size64, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);
                let rm = pretty_print_ireg(rm, OperandSize::Size64, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("csel {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::CSNeg { rd, rn, rm, cond } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size64, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);
                let rm = pretty_print_ireg(rm, OperandSize::Size64, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("csneg {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::CSet { rd, cond } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size64, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("cset {}, {}", rd, cond)
            }
            &Inst::CSetm { rd, cond } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size64, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("csetm {}, {}", rd, cond)
            }
            &Inst::CCmp {
                size,
                rn,
                rm,
                nzcv,
                cond,
            } => {
                let rn = pretty_print_ireg(rn, size, allocs);
                let rm = pretty_print_ireg(rm, size, allocs);
                let nzcv = nzcv.pretty_print(0, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("ccmp {}, {}, {}, {}", rn, rm, nzcv, cond)
            }
            &Inst::CCmpImm {
                size,
                rn,
                imm,
                nzcv,
                cond,
            } => {
                let rn = pretty_print_ireg(rn, size, allocs);
                let imm = imm.pretty_print(0, allocs);
                let nzcv = nzcv.pretty_print(0, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("ccmp {}, {}, {}, {}", rn, imm, nzcv, cond)
            }
            &Inst::AtomicRMW {
                rs, rt, rn, ty, op, ..
            } => {
                let op = match op {
                    AtomicRMWOp::Add => "ldaddal",
                    AtomicRMWOp::Clr => "ldclral",
                    AtomicRMWOp::Eor => "ldeoral",
                    AtomicRMWOp::Set => "ldsetal",
                    AtomicRMWOp::Smax => "ldsmaxal",
                    AtomicRMWOp::Umax => "ldumaxal",
                    AtomicRMWOp::Smin => "ldsminal",
                    AtomicRMWOp::Umin => "lduminal",
                    AtomicRMWOp::Swp => "swpal",
                };

                let size = OperandSize::from_ty(ty);
                let rs = pretty_print_ireg(rs, size, allocs);
                let rt = pretty_print_ireg(rt.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);

                let ty_suffix = match ty {
                    I8 => "b",
                    I16 => "h",
                    _ => "",
                };
                format!("{}{} {}, {}, [{}]", op, ty_suffix, rs, rt, rn)
            }
            &Inst::AtomicRMWLoop {
                ty,
                op,
                addr,
                operand,
                oldval,
                scratch1,
                scratch2,
                ..
            } => {
                let op = match op {
                    AtomicRMWLoopOp::Add => "add",
                    AtomicRMWLoopOp::Sub => "sub",
                    AtomicRMWLoopOp::Eor => "eor",
                    AtomicRMWLoopOp::Orr => "orr",
                    AtomicRMWLoopOp::And => "and",
                    AtomicRMWLoopOp::Nand => "nand",
                    AtomicRMWLoopOp::Smin => "smin",
                    AtomicRMWLoopOp::Smax => "smax",
                    AtomicRMWLoopOp::Umin => "umin",
                    AtomicRMWLoopOp::Umax => "umax",
                    AtomicRMWLoopOp::Xchg => "xchg",
                };
                let addr = pretty_print_ireg(addr, OperandSize::Size64, allocs);
                let operand = pretty_print_ireg(operand, OperandSize::Size64, allocs);
                let oldval = pretty_print_ireg(oldval.to_reg(), OperandSize::Size64, allocs);
                let scratch1 = pretty_print_ireg(scratch1.to_reg(), OperandSize::Size64, allocs);
                let scratch2 = pretty_print_ireg(scratch2.to_reg(), OperandSize::Size64, allocs);
                format!(
                    "atomic_rmw_loop_{}_{} addr={} operand={} oldval={} scratch1={} scratch2={}",
                    op,
                    ty.bits(),
                    addr,
                    operand,
                    oldval,
                    scratch1,
                    scratch2,
                )
            }
            &Inst::AtomicCAS {
                rd, rs, rt, rn, ty, ..
            } => {
                let op = match ty {
                    I8 => "casalb",
                    I16 => "casalh",
                    I32 | I64 => "casal",
                    _ => panic!("Unsupported type: {}", ty),
                };
                let size = OperandSize::from_ty(ty);
                let rd = pretty_print_ireg(rd.to_reg(), size, allocs);
                let rs = pretty_print_ireg(rs, size, allocs);
                let rt = pretty_print_ireg(rt, size, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);

                format!("{} {}, {}, {}, [{}]", op, rd, rs, rt, rn)
            }
            &Inst::AtomicCASLoop {
                ty,
                addr,
                expected,
                replacement,
                oldval,
                scratch,
                ..
            } => {
                let addr = pretty_print_ireg(addr, OperandSize::Size64, allocs);
                let expected = pretty_print_ireg(expected, OperandSize::Size64, allocs);
                let replacement = pretty_print_ireg(replacement, OperandSize::Size64, allocs);
                let oldval = pretty_print_ireg(oldval.to_reg(), OperandSize::Size64, allocs);
                let scratch = pretty_print_ireg(scratch.to_reg(), OperandSize::Size64, allocs);
                format!(
                    "atomic_cas_loop_{} addr={}, expect={}, replacement={}, oldval={}, scratch={}",
                    ty.bits(),
                    addr,
                    expected,
                    replacement,
                    oldval,
                    scratch,
                )
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
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);
                let rt = pretty_print_ireg(rt.to_reg(), size, allocs);
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
                let rn = pretty_print_ireg(rn, OperandSize::Size64, allocs);
                let rt = pretty_print_ireg(rt, size, allocs);
                format!("{} {}, [{}]", op, rt, rn)
            }
            &Inst::Fence {} => {
                format!("dmb ish")
            }
            &Inst::Csdb {} => {
                format!("csdb")
            }
            &Inst::FpuMove64 { rd, rn } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs);
                let rn = pretty_print_vreg_scalar(rn, ScalarSize::Size64, allocs);
                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuMove128 { rd, rn } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!("mov {}.16b, {}.16b", rd, rn)
            }
            &Inst::FpuMoveFromVec { rd, rn, idx, size } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size.lane_size(), allocs);
                let rn = pretty_print_vreg_element(rn, idx as usize, size.lane_size(), allocs);
                format!("mov {}, {}", rd, rn)
            }
            &Inst::FpuExtend { rd, rn, size } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuRR {
                fpu_op,
                size,
                rd,
                rn,
            } => {
                let op = match fpu_op {
                    FPUOp1::Abs => "fabs",
                    FPUOp1::Neg => "fneg",
                    FPUOp1::Sqrt => "fsqrt",
                    FPUOp1::Cvt32To64 | FPUOp1::Cvt64To32 => "fcvt",
                };
                let dst_size = match fpu_op {
                    FPUOp1::Cvt32To64 => ScalarSize::Size64,
                    FPUOp1::Cvt64To32 => ScalarSize::Size32,
                    _ => size,
                };
                let rd = pretty_print_vreg_scalar(rd.to_reg(), dst_size, allocs);
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::FpuRRR {
                fpu_op,
                size,
                rd,
                rn,
                rm,
            } => {
                let op = match fpu_op {
                    FPUOp2::Add => "fadd",
                    FPUOp2::Sub => "fsub",
                    FPUOp2::Mul => "fmul",
                    FPUOp2::Div => "fdiv",
                    FPUOp2::Max => "fmax",
                    FPUOp2::Min => "fmin",
                };
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                let rm = pretty_print_vreg_scalar(rm, size, allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::FpuRRI { fpu_op, rd, rn } => {
                let (op, imm, vector) = match fpu_op {
                    FPUOpRI::UShr32(imm) => ("ushr", imm.pretty_print(0, allocs), true),
                    FPUOpRI::UShr64(imm) => ("ushr", imm.pretty_print(0, allocs), false),
                };

                let (rd, rn) = if vector {
                    (
                        pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size32x2, allocs),
                        pretty_print_vreg_vector(rn, VectorSize::Size32x2, allocs),
                    )
                } else {
                    (
                        pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs),
                        pretty_print_vreg_scalar(rn, ScalarSize::Size64, allocs),
                    )
                };
                format!("{} {}, {}, {}", op, rd, rn, imm)
            }
            &Inst::FpuRRIMod { fpu_op, rd, ri, rn } => {
                let (op, imm, vector) = match fpu_op {
                    FPUOpRIMod::Sli32(imm) => ("sli", imm.pretty_print(0, allocs), true),
                    FPUOpRIMod::Sli64(imm) => ("sli", imm.pretty_print(0, allocs), false),
                };

                let (rd, ri, rn) = if vector {
                    (
                        pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size32x2, allocs),
                        pretty_print_vreg_vector(ri, VectorSize::Size32x2, allocs),
                        pretty_print_vreg_vector(rn, VectorSize::Size32x2, allocs),
                    )
                } else {
                    (
                        pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs),
                        pretty_print_vreg_scalar(ri, ScalarSize::Size64, allocs),
                        pretty_print_vreg_scalar(rn, ScalarSize::Size64, allocs),
                    )
                };
                format!("{} {}, {}, {}, {}", op, rd, ri, rn, imm)
            }
            &Inst::FpuRRRR {
                fpu_op,
                size,
                rd,
                rn,
                rm,
                ra,
            } => {
                let op = match fpu_op {
                    FPUOp3::MAdd => "fmadd",
                };
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                let rm = pretty_print_vreg_scalar(rm, size, allocs);
                let ra = pretty_print_vreg_scalar(ra, size, allocs);
                format!("{} {}, {}, {}, {}", op, rd, rn, rm, ra)
            }
            &Inst::FpuCmp { size, rn, rm } => {
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                let rm = pretty_print_vreg_scalar(rm, size, allocs);
                format!("fcmp {}, {}", rn, rm)
            }
            &Inst::FpuLoad32 { rd, ref mem, .. } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size32, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoad64 { rd, ref mem, .. } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoad128 { rd, ref mem, .. } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rd = "q".to_string() + &rd[1..];
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}ldr {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore32 { rd, ref mem, .. } => {
                let rd = pretty_print_vreg_scalar(rd, ScalarSize::Size32, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore64 { rd, ref mem, .. } => {
                let rd = pretty_print_vreg_scalar(rd, ScalarSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuStore128 { rd, ref mem, .. } => {
                let rd = pretty_print_reg(rd, allocs);
                let rd = "q".to_string() + &rd[1..];
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(&mem, state);
                let mem = mem.pretty_print_default();
                format!("{}str {}, {}", mem_str, rd, mem)
            }
            &Inst::FpuLoadP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_vreg_scalar(rt.to_reg(), ScalarSize::Size64, allocs);
                let rt2 = pretty_print_vreg_scalar(rt2.to_reg(), ScalarSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();

                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuStoreP64 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_vreg_scalar(rt, ScalarSize::Size64, allocs);
                let rt2 = pretty_print_vreg_scalar(rt2, ScalarSize::Size64, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();

                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuLoadP128 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_vreg_scalar(rt.to_reg(), ScalarSize::Size128, allocs);
                let rt2 = pretty_print_vreg_scalar(rt2.to_reg(), ScalarSize::Size128, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();

                format!("ldp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::FpuStoreP128 {
                rt, rt2, ref mem, ..
            } => {
                let rt = pretty_print_vreg_scalar(rt, ScalarSize::Size128, allocs);
                let rt2 = pretty_print_vreg_scalar(rt2, ScalarSize::Size128, allocs);
                let mem = mem.with_allocs(allocs);
                let mem = mem.pretty_print_default();

                format!("stp {}, {}, {}", rt, rt2, mem)
            }
            &Inst::LoadFpuConst64 { rd, const_data } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs);
                format!(
                    "ldr {}, pc+8 ; b 12 ; data.f64 {}",
                    rd,
                    f64::from_bits(const_data)
                )
            }
            &Inst::LoadFpuConst128 { rd, const_data } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size128, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), sizedest, allocs);
                let rn = pretty_print_vreg_scalar(rn, sizesrc, allocs);
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
                let rd = pretty_print_vreg_scalar(rd.to_reg(), sizedest, allocs);
                let rn = pretty_print_ireg(rn, sizesrc, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::FpuCSel32 { rd, rn, rm, cond } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size32, allocs);
                let rn = pretty_print_vreg_scalar(rn, ScalarSize::Size32, allocs);
                let rm = pretty_print_vreg_scalar(rm, ScalarSize::Size32, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!("fcsel {}, {}, {}, {}", rd, rn, rm, cond)
            }
            &Inst::FpuCSel64 { rd, rn, rm, cond } => {
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs);
                let rn = pretty_print_vreg_scalar(rn, ScalarSize::Size64, allocs);
                let rm = pretty_print_vreg_scalar(rm, ScalarSize::Size64, allocs);
                let cond = cond.pretty_print(0, allocs);
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
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_scalar(rn, size, allocs);
                format!("{} {}, {}", inst, rd, rn)
            }
            &Inst::MovToFpu { rd, rn, size } => {
                let operand_size = size.operand_size();
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, operand_size, allocs);
                format!("fmov {}, {}", rd, rn)
            }
            &Inst::FpuMoveFPImm { rd, imm, size } => {
                let imm = imm.pretty_print(0, allocs);
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size, allocs);

                format!("fmov {}, {}", rd, imm)
            }
            &Inst::MovToVec {
                rd,
                ri,
                rn,
                idx,
                size,
            } => {
                let rd =
                    pretty_print_vreg_element(rd.to_reg(), idx as usize, size.lane_size(), allocs);
                let ri = pretty_print_vreg_element(ri, idx as usize, size.lane_size(), allocs);
                let rn = pretty_print_ireg(rn, size.operand_size(), allocs);
                format!("mov {}, {}, {}", rd, ri, rn)
            }
            &Inst::MovFromVec { rd, rn, idx, size } => {
                let op = match size {
                    ScalarSize::Size8 => "umov",
                    ScalarSize::Size16 => "umov",
                    ScalarSize::Size32 => "mov",
                    ScalarSize::Size64 => "mov",
                    _ => unimplemented!(),
                };
                let rd = pretty_print_ireg(rd.to_reg(), size.operand_size(), allocs);
                let rn = pretty_print_vreg_element(rn, idx as usize, size, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::MovFromVecSigned {
                rd,
                rn,
                idx,
                size,
                scalar_size,
            } => {
                let rd = pretty_print_ireg(rd.to_reg(), scalar_size, allocs);
                let rn = pretty_print_vreg_element(rn, idx as usize, size.lane_size(), allocs);
                format!("smov {}, {}", rd, rn)
            }
            &Inst::VecDup { rd, rn, size } => {
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_ireg(rn, size.operand_size(), allocs);
                format!("dup {}, {}", rd, rn)
            }
            &Inst::VecDupFromFpu { rd, rn, size } => {
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_element(rn, 0, size.lane_size(), allocs);
                format!("dup {}, {}", rd, rn)
            }
            &Inst::VecDupFPImm { rd, imm, size } => {
                let imm = imm.pretty_print(0, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);

                format!("fmov {}, {}", rd, imm)
            }
            &Inst::VecDupImm {
                rd,
                imm,
                invert,
                size,
            } => {
                let imm = imm.pretty_print(0, allocs);
                let op = if invert { "mvni" } else { "movi" };
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);

                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::VecExtend {
                t,
                rd,
                rn,
                high_half,
                lane_size,
            } => {
                let vec64 = VectorSize::from_lane_size(lane_size.narrow(), false);
                let vec128 = VectorSize::from_lane_size(lane_size.narrow(), true);
                let rd_size = VectorSize::from_lane_size(lane_size, true);
                let (op, rn_size) = match (t, high_half) {
                    (VecExtendOp::Sxtl, false) => ("sxtl", vec64),
                    (VecExtendOp::Sxtl, true) => ("sxtl2", vec128),
                    (VecExtendOp::Uxtl, false) => ("uxtl", vec64),
                    (VecExtendOp::Uxtl, true) => ("uxtl2", vec128),
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), rd_size, allocs);
                let rn = pretty_print_vreg_vector(rn, rn_size, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecMovElement {
                rd,
                ri,
                rn,
                dest_idx,
                src_idx,
                size,
            } => {
                let rd = pretty_print_vreg_element(
                    rd.to_reg(),
                    dest_idx as usize,
                    size.lane_size(),
                    allocs,
                );
                let ri = pretty_print_vreg_element(ri, dest_idx as usize, size.lane_size(), allocs);
                let rn = pretty_print_vreg_element(rn, src_idx as usize, size.lane_size(), allocs);
                format!("mov {}, {}, {}", rd, ri, rn)
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
                let rd = pretty_print_vreg_vector(rd.to_reg(), rd_size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);

                format!("{} {}, {}{}", op, rd, rn, suffix)
            }
            &Inst::VecRRNarrowLow {
                op,
                rd,
                rn,
                lane_size,
                ..
            }
            | &Inst::VecRRNarrowHigh {
                op,
                rd,
                rn,
                lane_size,
                ..
            } => {
                let vec64 = VectorSize::from_lane_size(lane_size, false);
                let vec128 = VectorSize::from_lane_size(lane_size, true);
                let rn_size = VectorSize::from_lane_size(lane_size.widen(), true);
                let high_half = match self {
                    &Inst::VecRRNarrowLow { .. } => false,
                    &Inst::VecRRNarrowHigh { .. } => true,
                    _ => unreachable!(),
                };
                let (op, rd_size) = match (op, high_half) {
                    (VecRRNarrowOp::Xtn, false) => ("xtn", vec64),
                    (VecRRNarrowOp::Xtn, true) => ("xtn2", vec128),
                    (VecRRNarrowOp::Sqxtn, false) => ("sqxtn", vec64),
                    (VecRRNarrowOp::Sqxtn, true) => ("sqxtn2", vec128),
                    (VecRRNarrowOp::Sqxtun, false) => ("sqxtun", vec64),
                    (VecRRNarrowOp::Sqxtun, true) => ("sqxtun2", vec128),
                    (VecRRNarrowOp::Uqxtn, false) => ("uqxtn", vec64),
                    (VecRRNarrowOp::Uqxtn, true) => ("uqxtn2", vec128),
                    (VecRRNarrowOp::Fcvtn, false) => ("fcvtn", vec64),
                    (VecRRNarrowOp::Fcvtn, true) => ("fcvtn2", vec128),
                };
                let rn = pretty_print_vreg_vector(rn, rn_size, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), rd_size, allocs);
                let ri = match self {
                    &Inst::VecRRNarrowLow { .. } => "".to_string(),
                    &Inst::VecRRNarrowHigh { ri, .. } => {
                        format!("{}, ", pretty_print_vreg_vector(ri, rd_size, allocs))
                    }
                    _ => unreachable!(),
                };

                format!("{} {}, {}{}", op, rd, ri, rn)
            }
            &Inst::VecRRPair { op, rd, rn } => {
                let op = match op {
                    VecPairOp::Addp => "addp",
                };
                let rd = pretty_print_vreg_scalar(rd.to_reg(), ScalarSize::Size64, allocs);
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size64x2, allocs);

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
                let rd = pretty_print_vreg_vector(rd.to_reg(), dest, allocs);
                let rn = pretty_print_vreg_vector(rn, src, allocs);

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
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
                let rm = pretty_print_vreg_vector(rm, size, allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::VecRRRMod {
                rd,
                ri,
                rn,
                rm,
                alu_op,
                size,
            } => {
                let (op, size) = match alu_op {
                    VecALUModOp::Bsl => ("bsl", VectorSize::Size8x16),
                    VecALUModOp::Fmla => ("fmla", size),
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let ri = pretty_print_vreg_vector(ri, size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
                let rm = pretty_print_vreg_vector(rm, size, allocs);
                format!("{} {}, {}, {}, {}", op, rd, ri, rn, rm)
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
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), dest_size, allocs);
                let rn = pretty_print_vreg_vector(rn, src_size, allocs);
                let rm = pretty_print_vreg_vector(rm, src_size, allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::VecRRRLongMod {
                rd,
                ri,
                rn,
                rm,
                alu_op,
                high_half,
            } => {
                let (op, dest_size, src_size) = match (alu_op, high_half) {
                    (VecRRRLongModOp::Umlal8, false) => {
                        ("umlal", VectorSize::Size16x8, VectorSize::Size8x8)
                    }
                    (VecRRRLongModOp::Umlal8, true) => {
                        ("umlal2", VectorSize::Size16x8, VectorSize::Size8x16)
                    }
                    (VecRRRLongModOp::Umlal16, false) => {
                        ("umlal", VectorSize::Size32x4, VectorSize::Size16x4)
                    }
                    (VecRRRLongModOp::Umlal16, true) => {
                        ("umlal2", VectorSize::Size32x4, VectorSize::Size16x8)
                    }
                    (VecRRRLongModOp::Umlal32, false) => {
                        ("umlal", VectorSize::Size64x2, VectorSize::Size32x2)
                    }
                    (VecRRRLongModOp::Umlal32, true) => {
                        ("umlal2", VectorSize::Size64x2, VectorSize::Size32x4)
                    }
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), dest_size, allocs);
                let ri = pretty_print_vreg_vector(ri, dest_size, allocs);
                let rn = pretty_print_vreg_vector(rn, src_size, allocs);
                let rm = pretty_print_vreg_vector(rm, src_size, allocs);
                format!("{} {}, {}, {}, {}", op, rd, ri, rn, rm)
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
                    VecMisc2::Cmge0 => ("cmge", size, ", #0"),
                    VecMisc2::Cmgt0 => ("cmgt", size, ", #0"),
                    VecMisc2::Cmle0 => ("cmle", size, ", #0"),
                    VecMisc2::Cmlt0 => ("cmlt", size, ", #0"),
                    VecMisc2::Fcmeq0 => ("fcmeq", size, ", #0.0"),
                    VecMisc2::Fcmge0 => ("fcmge", size, ", #0.0"),
                    VecMisc2::Fcmgt0 => ("fcmgt", size, ", #0.0"),
                    VecMisc2::Fcmle0 => ("fcmle", size, ", #0.0"),
                    VecMisc2::Fcmlt0 => ("fcmlt", size, ", #0.0"),
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
                format!("{} {}, {}{}", op, rd, rn, suffix)
            }
            &Inst::VecLanes { op, rd, rn, size } => {
                let op = match op {
                    VecLanesOp::Uminv => "uminv",
                    VecLanesOp::Addv => "addv",
                };
                let rd = pretty_print_vreg_scalar(rd.to_reg(), size.lane_size(), allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
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
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
                format!("{} {}, {}, #{}", op, rd, rn, imm)
            }
            &Inst::VecShiftImmMod {
                op,
                rd,
                ri,
                rn,
                size,
                imm,
            } => {
                let op = match op {
                    VecShiftImmModOp::Sli => "sli",
                };
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let ri = pretty_print_vreg_vector(ri, size, allocs);
                let rn = pretty_print_vreg_vector(rn, size, allocs);
                format!("{} {}, {}, {}, #{}", op, rd, ri, rn, imm)
            }
            &Inst::VecExtract { rd, rn, rm, imm4 } => {
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                format!("ext {}, {}, {}, #{}", rd, rn, rm, imm4)
            }
            &Inst::VecTbl { rd, rn, rm } => {
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                format!("tbl {}, {{ {} }}, {}", rd, rn, rm)
            }
            &Inst::VecTblExt { rd, ri, rn, rm } => {
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                let ri = pretty_print_vreg_vector(ri, VectorSize::Size8x16, allocs);
                format!("tbx {}, {}, {{ {} }}, {}", rd, ri, rn, rm)
            }
            &Inst::VecTbl2 { rd, rn, rn2, rm } => {
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rn2 = pretty_print_vreg_vector(rn2, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                format!("tbl {}, {{ {}, {} }}, {}", rd, rn, rn2, rm)
            }
            &Inst::VecTbl2Ext {
                rd,
                ri,
                rn,
                rn2,
                rm,
            } => {
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rn2 = pretty_print_vreg_vector(rn2, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                let ri = pretty_print_vreg_vector(ri, VectorSize::Size8x16, allocs);
                format!("tbx {}, {}, {{ {}, {} }}, {}", rd, ri, rn, rn2, rm)
            }
            &Inst::VecLoadReplicate { rd, rn, size, .. } => {
                let rd = pretty_print_vreg_vector(rd.to_reg(), size, allocs);
                let rn = pretty_print_reg(rn, allocs);

                format!("ld1r {{ {} }}, [{}]", rd, rn)
            }
            &Inst::VecCSel { rd, rn, rm, cond } => {
                let rd = pretty_print_vreg_vector(rd.to_reg(), VectorSize::Size8x16, allocs);
                let rn = pretty_print_vreg_vector(rn, VectorSize::Size8x16, allocs);
                let rm = pretty_print_vreg_vector(rm, VectorSize::Size8x16, allocs);
                let cond = cond.pretty_print(0, allocs);
                format!(
                    "vcsel {}, {}, {}, {} (if-then-else diamond)",
                    rd, rn, rm, cond
                )
            }
            &Inst::MovToNZCV { rn } => {
                let rn = pretty_print_reg(rn, allocs);
                format!("msr nzcv, {}", rn)
            }
            &Inst::MovFromNZCV { rd } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("mrs {}, nzcv", rd)
            }
            &Inst::Extend {
                rd,
                rn,
                signed: false,
                from_bits: 1,
                ..
            } => {
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size32, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size32, allocs);
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
                let rd = pretty_print_ireg(rd.to_reg(), OperandSize::Size32, allocs);
                let rn = pretty_print_ireg(rn, OperandSize::Size32, allocs);
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
                    let rd = pretty_print_ireg(rd.to_reg(), dest_size, allocs);
                    let rn = pretty_print_ireg(rn, dest_size, allocs);
                    format!("{} {}, {}, #0, #{}", op, rd, rn, from_bits)
                } else {
                    let dest_size = if signed {
                        OperandSize::from_bits(to_bits)
                    } else {
                        OperandSize::Size32
                    };
                    let rd = pretty_print_ireg(rd.to_reg(), dest_size, allocs);
                    let rn = pretty_print_ireg(rn, OperandSize::from_bits(from_bits), allocs);
                    format!("{} {}, {}", op, rd, rn)
                }
            }
            &Inst::Call { .. } => format!("bl 0"),
            &Inst::CallInd { ref info, .. } => {
                let rn = pretty_print_reg(info.rn, allocs);
                format!("blr {}", rn)
            }
            &Inst::Args { ref args } => {
                let mut s = "args".to_string();
                for arg in args {
                    use std::fmt::Write;
                    let preg = pretty_print_reg(arg.preg, &mut empty_allocs);
                    let def = pretty_print_reg(arg.vreg.to_reg(), allocs);
                    write!(&mut s, " {}={}", def, preg).unwrap();
                }
                s
            }
            &Inst::Ret { .. } => "ret".to_string(),
            &Inst::AuthenticatedRet { key, is_hint, .. } => {
                let key = match key {
                    APIKey::A => "a",
                    APIKey::B => "b",
                };

                if is_hint {
                    "auti".to_string() + key + "sp ; ret"
                } else {
                    "reta".to_string() + key
                }
            }
            &Inst::Jump { ref dest } => {
                let dest = dest.pretty_print(0, allocs);
                format!("b {}", dest)
            }
            &Inst::CondBr {
                ref taken,
                ref not_taken,
                ref kind,
            } => {
                let taken = taken.pretty_print(0, allocs);
                let not_taken = not_taken.pretty_print(0, allocs);
                match kind {
                    &CondBrKind::Zero(reg) => {
                        let reg = pretty_print_reg(reg, allocs);
                        format!("cbz {}, {} ; b {}", reg, taken, not_taken)
                    }
                    &CondBrKind::NotZero(reg) => {
                        let reg = pretty_print_reg(reg, allocs);
                        format!("cbnz {}, {} ; b {}", reg, taken, not_taken)
                    }
                    &CondBrKind::Cond(c) => {
                        let c = c.pretty_print(0, allocs);
                        format!("b.{} {} ; b {}", c, taken, not_taken)
                    }
                }
            }
            &Inst::IndirectBr { rn, .. } => {
                let rn = pretty_print_reg(rn, allocs);
                format!("br {}", rn)
            }
            &Inst::Brk => "brk #0".to_string(),
            &Inst::Udf { .. } => "udf #0xc11f".to_string(),
            &Inst::TrapIf { ref kind, .. } => match kind {
                &CondBrKind::Zero(reg) => {
                    let reg = pretty_print_reg(reg, allocs);
                    format!("cbnz {}, 8 ; udf", reg)
                }
                &CondBrKind::NotZero(reg) => {
                    let reg = pretty_print_reg(reg, allocs);
                    format!("cbz {}, 8 ; udf", reg)
                }
                &CondBrKind::Cond(c) => {
                    let c = c.invert().pretty_print(0, allocs);
                    format!("b.{} 8 ; udf", c)
                }
            },
            &Inst::Adr { rd, off } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
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
                let ridx = pretty_print_reg(ridx, allocs);
                let rtmp1 = pretty_print_reg(rtmp1.to_reg(), allocs);
                let rtmp2 = pretty_print_reg(rtmp2.to_reg(), allocs);
                let default_target = info.default_target.pretty_print(0, allocs);
                format!(
                    concat!(
                        "b.hs {} ; ",
                        "csel {}, xzr, {}, hs ; ",
                        "csdb ; ",
                        "adr {}, pc+16 ; ",
                        "ldrsw {}, [{}, {}, uxtw #2] ; ",
                        "add {}, {}, {} ; ",
                        "br {} ; ",
                        "jt_entries {:?}"
                    ),
                    default_target,
                    rtmp2,
                    ridx,
                    rtmp1,
                    rtmp2,
                    rtmp1,
                    rtmp2,
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
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("ldr {}, 8 ; b 12 ; data {:?} + {}", rd, name, offset)
            }
            &Inst::LoadAddr { rd, ref mem } => {
                // TODO: we really should find a better way to avoid duplication of
                // this logic between `emit()` and `show_rru()` -- a separate 1-to-N
                // expansion stage (i.e., legalization, but without the slow edit-in-place
                // of the existing legalization framework).
                let rd = allocs.next_writable(rd);
                let mem = mem.with_allocs(allocs);
                let (mem_insts, mem) = mem_finalize(0, &mem, state);
                let mut ret = String::new();
                for inst in mem_insts.into_iter() {
                    ret.push_str(
                        &inst.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                    );
                }
                let (reg, index_reg, offset) = match mem {
                    AMode::RegExtended { rn, rm, extendop } => (rn, Some((rm, extendop)), 0),
                    AMode::Unscaled { rn, simm9 } => (rn, None, simm9.value()),
                    AMode::UnsignedOffset { rn, uimm12 } => (rn, None, uimm12.value() as i32),
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

                    ret.push_str(
                        &add.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                    );
                } else if offset == 0 {
                    let mov = Inst::gen_move(rd, reg, I64);
                    ret.push_str(
                        &mov.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                    );
                } else if let Some(imm12) = Imm12::maybe_from_u64(abs_offset) {
                    let add = Inst::AluRRImm12 {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        imm12,
                    };
                    ret.push_str(
                        &add.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                    );
                } else {
                    let tmp = writable_spilltmp_reg();
                    for inst in Inst::load_constant(tmp, abs_offset, &mut |_| tmp).into_iter() {
                        ret.push_str(
                            &inst.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                        );
                    }
                    let add = Inst::AluRRR {
                        alu_op,
                        size: OperandSize::Size64,
                        rd,
                        rn: reg,
                        rm: tmp.to_reg(),
                    };
                    ret.push_str(
                        &add.print_with_state(&mut EmitState::default(), &mut empty_allocs),
                    );
                }
                ret
            }
            &Inst::Pacisp { key } => {
                let key = match key {
                    APIKey::A => "a",
                    APIKey::B => "b",
                };

                "paci".to_string() + key + "sp"
            }
            &Inst::Xpaclri => "xpaclri".to_string(),
            &Inst::Bti { targets } => {
                let targets = match targets {
                    BranchTargetType::None => "",
                    BranchTargetType::C => " c",
                    BranchTargetType::J => " j",
                    BranchTargetType::JC => " jc",
                };

                "bti".to_string() + targets
            }
            &Inst::VirtualSPOffsetAdj { offset } => {
                state.virtual_sp_offset += offset;
                format!("virtual_sp_offset_adjust {}", offset)
            }
            &Inst::EmitIsland { needed_space } => format!("emit_island {}", needed_space),

            &Inst::ElfTlsGetAddr { ref symbol, rd } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("elf_tls_get_addr {}, {}", rd, symbol.display(None))
            }
            &Inst::Unwind { ref inst } => {
                format!("unwind {:?}", inst)
            }
            &Inst::DummyUse { reg } => {
                let reg = pretty_print_reg(reg, allocs);
                format!("dummy_use {}", reg)
            }
            &Inst::StackProbeLoop { start, end, step } => {
                let start = pretty_print_reg(start.to_reg(), allocs);
                let end = pretty_print_reg(end, allocs);
                let step = step.pretty_print(0, allocs);
                format!("stack_probe_loop {start}, {end}, {step}")
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
    #[allow(dead_code)]
    /// 19-bit offset for LDR (load literal). PC-rel, offset is imm << 2. Immediate is 19 signed bits,
    /// in bits 23:5.
    Ldr19,
    #[allow(dead_code)]
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
