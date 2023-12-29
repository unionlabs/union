//! This module defines s390x-specific machine instruction types.

use crate::binemit::{Addend, CodeOffset, Reloc};
use crate::ir::{types, ExternalName, Opcode, Type};
use crate::isa::s390x::abi::S390xMachineDeps;
use crate::isa::CallConv;
use crate::machinst::*;
use crate::{settings, CodegenError, CodegenResult};
use alloc::boxed::Box;
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
pub mod unwind;

#[cfg(test)]
mod emit_tests;

//=============================================================================
// Instructions (top level): definition

pub use crate::isa::s390x::lower::isle::generated_code::{
    ALUOp, CmpOp, FPUOp1, FPUOp2, FPUOp3, FpuRoundMode, FpuRoundOp, LaneOrder, MInst as Inst,
    RxSBGOp, ShiftOp, SymbolReloc, UnaryOp, VecBinaryOp, VecFloatCmpOp, VecIntCmpOp, VecShiftOp,
    VecUnaryOp,
};

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
    pub tls_symbol: Option<SymbolReloc>,
}

/// Additional information for CallInd instructions, left out of line to lower the size of the Inst
/// enum.
#[derive(Clone, Debug)]
pub struct CallIndInfo {
    pub rn: Reg,
    pub uses: CallArgList,
    pub defs: CallRetList,
    pub clobbers: PRegSet,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
}

#[test]
fn inst_size_test() {
    // This test will help with unintentionally growing the size
    // of the Inst enum.
    assert_eq!(32, std::mem::size_of::<Inst>());
}

/// A register pair. Enum so it can be destructured in ISLE.
#[derive(Clone, Copy, Debug)]
pub struct RegPair {
    pub hi: Reg,
    pub lo: Reg,
}

/// A writable register pair. Enum so it can be destructured in ISLE.
#[derive(Clone, Copy, Debug)]
pub struct WritableRegPair {
    pub hi: Writable<Reg>,
    pub lo: Writable<Reg>,
}

impl WritableRegPair {
    pub fn to_regpair(&self) -> RegPair {
        RegPair {
            hi: self.hi.to_reg(),
            lo: self.lo.to_reg(),
        }
    }
}

/// Supported instruction sets
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub(crate) enum InstructionSet {
    /// Baseline ISA for cranelift is z14.
    Base,
    /// Miscellaneous-Instruction-Extensions Facility 2 (z15)
    MIE2,
    /// Vector-Enhancements Facility 2 (z15)
    VXRS_EXT2,
}

impl Inst {
    /// Retrieve the ISA feature set in which the instruction is available.
    fn available_in_isa(&self) -> InstructionSet {
        match self {
            // These instructions are part of the baseline ISA for cranelift (z14)
            Inst::Nop0
            | Inst::Nop2
            | Inst::AluRRSImm16 { .. }
            | Inst::AluRR { .. }
            | Inst::AluRX { .. }
            | Inst::AluRSImm16 { .. }
            | Inst::AluRSImm32 { .. }
            | Inst::AluRUImm32 { .. }
            | Inst::AluRUImm16Shifted { .. }
            | Inst::AluRUImm32Shifted { .. }
            | Inst::ShiftRR { .. }
            | Inst::RxSBG { .. }
            | Inst::RxSBGTest { .. }
            | Inst::SMulWide { .. }
            | Inst::UMulWide { .. }
            | Inst::SDivMod32 { .. }
            | Inst::SDivMod64 { .. }
            | Inst::UDivMod32 { .. }
            | Inst::UDivMod64 { .. }
            | Inst::Flogr { .. }
            | Inst::CmpRR { .. }
            | Inst::CmpRX { .. }
            | Inst::CmpRSImm16 { .. }
            | Inst::CmpRSImm32 { .. }
            | Inst::CmpRUImm32 { .. }
            | Inst::CmpTrapRR { .. }
            | Inst::CmpTrapRSImm16 { .. }
            | Inst::CmpTrapRUImm16 { .. }
            | Inst::AtomicRmw { .. }
            | Inst::AtomicCas32 { .. }
            | Inst::AtomicCas64 { .. }
            | Inst::Fence
            | Inst::Load32 { .. }
            | Inst::Load32ZExt8 { .. }
            | Inst::Load32SExt8 { .. }
            | Inst::Load32ZExt16 { .. }
            | Inst::Load32SExt16 { .. }
            | Inst::Load64 { .. }
            | Inst::Load64ZExt8 { .. }
            | Inst::Load64SExt8 { .. }
            | Inst::Load64ZExt16 { .. }
            | Inst::Load64SExt16 { .. }
            | Inst::Load64ZExt32 { .. }
            | Inst::Load64SExt32 { .. }
            | Inst::LoadRev16 { .. }
            | Inst::LoadRev32 { .. }
            | Inst::LoadRev64 { .. }
            | Inst::Store8 { .. }
            | Inst::Store16 { .. }
            | Inst::Store32 { .. }
            | Inst::Store64 { .. }
            | Inst::StoreImm8 { .. }
            | Inst::StoreImm16 { .. }
            | Inst::StoreImm32SExt16 { .. }
            | Inst::StoreImm64SExt16 { .. }
            | Inst::StoreRev16 { .. }
            | Inst::StoreRev32 { .. }
            | Inst::StoreRev64 { .. }
            | Inst::Mvc { .. }
            | Inst::LoadMultiple64 { .. }
            | Inst::StoreMultiple64 { .. }
            | Inst::Mov32 { .. }
            | Inst::Mov64 { .. }
            | Inst::MovPReg { .. }
            | Inst::Mov32Imm { .. }
            | Inst::Mov32SImm16 { .. }
            | Inst::Mov64SImm16 { .. }
            | Inst::Mov64SImm32 { .. }
            | Inst::Mov64UImm16Shifted { .. }
            | Inst::Mov64UImm32Shifted { .. }
            | Inst::Insert64UImm16Shifted { .. }
            | Inst::Insert64UImm32Shifted { .. }
            | Inst::LoadAR { .. }
            | Inst::InsertAR { .. }
            | Inst::Extend { .. }
            | Inst::CMov32 { .. }
            | Inst::CMov64 { .. }
            | Inst::CMov32SImm16 { .. }
            | Inst::CMov64SImm16 { .. }
            | Inst::FpuMove32 { .. }
            | Inst::FpuMove64 { .. }
            | Inst::FpuCMov32 { .. }
            | Inst::FpuCMov64 { .. }
            | Inst::FpuRR { .. }
            | Inst::FpuRRR { .. }
            | Inst::FpuRRRR { .. }
            | Inst::FpuCmp32 { .. }
            | Inst::FpuCmp64 { .. }
            | Inst::LoadFpuConst32 { .. }
            | Inst::LoadFpuConst64 { .. }
            | Inst::VecRRR { .. }
            | Inst::VecRR { .. }
            | Inst::VecShiftRR { .. }
            | Inst::VecSelect { .. }
            | Inst::VecPermute { .. }
            | Inst::VecPermuteDWImm { .. }
            | Inst::VecIntCmp { .. }
            | Inst::VecIntCmpS { .. }
            | Inst::VecFloatCmp { .. }
            | Inst::VecFloatCmpS { .. }
            | Inst::VecInt128SCmpHi { .. }
            | Inst::VecInt128UCmpHi { .. }
            | Inst::VecLoad { .. }
            | Inst::VecStore { .. }
            | Inst::VecLoadReplicate { .. }
            | Inst::VecMov { .. }
            | Inst::VecCMov { .. }
            | Inst::MovToVec128 { .. }
            | Inst::VecLoadConst { .. }
            | Inst::VecLoadConstReplicate { .. }
            | Inst::VecImmByteMask { .. }
            | Inst::VecImmBitMask { .. }
            | Inst::VecImmReplicate { .. }
            | Inst::VecLoadLane { .. }
            | Inst::VecLoadLaneUndef { .. }
            | Inst::VecStoreLane { .. }
            | Inst::VecInsertLane { .. }
            | Inst::VecInsertLaneUndef { .. }
            | Inst::VecExtractLane { .. }
            | Inst::VecInsertLaneImm { .. }
            | Inst::VecReplicateLane { .. }
            | Inst::Call { .. }
            | Inst::CallInd { .. }
            | Inst::Args { .. }
            | Inst::Ret { .. }
            | Inst::Jump { .. }
            | Inst::CondBr { .. }
            | Inst::TrapIf { .. }
            | Inst::OneWayCondBr { .. }
            | Inst::IndirectBr { .. }
            | Inst::Debugtrap
            | Inst::Trap { .. }
            | Inst::JTSequence { .. }
            | Inst::LoadSymbolReloc { .. }
            | Inst::LoadAddr { .. }
            | Inst::Loop { .. }
            | Inst::CondBreak { .. }
            | Inst::VirtualSPOffsetAdj { .. }
            | Inst::Unwind { .. } => InstructionSet::Base,

            // These depend on the opcode
            Inst::AluRRR { alu_op, .. } => match alu_op {
                ALUOp::NotAnd32 | ALUOp::NotAnd64 => InstructionSet::MIE2,
                ALUOp::NotOrr32 | ALUOp::NotOrr64 => InstructionSet::MIE2,
                ALUOp::NotXor32 | ALUOp::NotXor64 => InstructionSet::MIE2,
                ALUOp::AndNot32 | ALUOp::AndNot64 => InstructionSet::MIE2,
                ALUOp::OrrNot32 | ALUOp::OrrNot64 => InstructionSet::MIE2,
                _ => InstructionSet::Base,
            },
            Inst::UnaryRR { op, .. } => match op {
                UnaryOp::PopcntReg => InstructionSet::MIE2,
                _ => InstructionSet::Base,
            },
            Inst::FpuRound { op, .. } => match op {
                FpuRoundOp::ToSInt32 | FpuRoundOp::FromSInt32 => InstructionSet::MIE2,
                FpuRoundOp::ToUInt32 | FpuRoundOp::FromUInt32 => InstructionSet::MIE2,
                FpuRoundOp::ToSInt32x4 | FpuRoundOp::FromSInt32x4 => InstructionSet::MIE2,
                FpuRoundOp::ToUInt32x4 | FpuRoundOp::FromUInt32x4 => InstructionSet::MIE2,
                _ => InstructionSet::Base,
            },

            // These are all part of VXRS_EXT2
            Inst::VecLoadRev { .. }
            | Inst::VecLoadByte16Rev { .. }
            | Inst::VecLoadByte32Rev { .. }
            | Inst::VecLoadByte64Rev { .. }
            | Inst::VecLoadElt16Rev { .. }
            | Inst::VecLoadElt32Rev { .. }
            | Inst::VecLoadElt64Rev { .. }
            | Inst::VecStoreRev { .. }
            | Inst::VecStoreByte16Rev { .. }
            | Inst::VecStoreByte32Rev { .. }
            | Inst::VecStoreByte64Rev { .. }
            | Inst::VecStoreElt16Rev { .. }
            | Inst::VecStoreElt32Rev { .. }
            | Inst::VecStoreElt64Rev { .. }
            | Inst::VecLoadReplicateRev { .. }
            | Inst::VecLoadLaneRev { .. }
            | Inst::VecLoadLaneRevUndef { .. }
            | Inst::VecStoreLaneRev { .. } => InstructionSet::VXRS_EXT2,

            Inst::DummyUse { .. } => InstructionSet::Base,
        }
    }

    /// Create a 128-bit move instruction.
    pub fn mov128(to_reg: Writable<Reg>, from_reg: Reg) -> Inst {
        assert!(to_reg.to_reg().class() == RegClass::Float);
        assert!(from_reg.class() == RegClass::Float);
        Inst::VecMov {
            rd: to_reg,
            rn: from_reg,
        }
    }

    /// Create a 64-bit move instruction.
    pub fn mov64(to_reg: Writable<Reg>, from_reg: Reg) -> Inst {
        assert!(to_reg.to_reg().class() == from_reg.class());
        if from_reg.class() == RegClass::Int {
            Inst::Mov64 {
                rd: to_reg,
                rm: from_reg,
            }
        } else {
            Inst::FpuMove64 {
                rd: to_reg,
                rn: from_reg,
            }
        }
    }

    /// Create a 32-bit move instruction.
    pub fn mov32(to_reg: Writable<Reg>, from_reg: Reg) -> Inst {
        if from_reg.class() == RegClass::Int {
            Inst::Mov32 {
                rd: to_reg,
                rm: from_reg,
            }
        } else {
            Inst::FpuMove32 {
                rd: to_reg,
                rn: from_reg,
            }
        }
    }

    /// Create an instruction that loads a 64-bit integer constant.
    pub fn load_constant64<F: FnMut(Type) -> Writable<Reg>>(
        rd: Writable<Reg>,
        value: u64,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        if let Ok(imm) = i16::try_from(value as i64) {
            // 16-bit signed immediate
            smallvec![Inst::Mov64SImm16 { rd, imm }]
        } else if let Ok(imm) = i32::try_from(value as i64) {
            // 32-bit signed immediate
            smallvec![Inst::Mov64SImm32 { rd, imm }]
        } else if let Some(imm) = UImm16Shifted::maybe_from_u64(value) {
            // 16-bit shifted immediate
            smallvec![Inst::Mov64UImm16Shifted { rd, imm }]
        } else if let Some(imm) = UImm32Shifted::maybe_from_u64(value) {
            // 32-bit shifted immediate
            smallvec![Inst::Mov64UImm32Shifted { rd, imm }]
        } else {
            let mut insts = smallvec![];
            let hi = value & 0xffff_ffff_0000_0000u64;
            let lo = value & 0x0000_0000_ffff_ffffu64;

            let hi_rd = alloc_tmp(types::I64);
            if let Some(imm) = UImm16Shifted::maybe_from_u64(hi) {
                // 16-bit shifted immediate
                insts.push(Inst::Mov64UImm16Shifted { rd: hi_rd, imm });
            } else if let Some(imm) = UImm32Shifted::maybe_from_u64(hi) {
                // 32-bit shifted immediate
                insts.push(Inst::Mov64UImm32Shifted { rd: hi_rd, imm });
            } else {
                unreachable!();
            }

            if let Some(imm) = UImm16Shifted::maybe_from_u64(lo) {
                // 16-bit shifted immediate
                insts.push(Inst::Insert64UImm16Shifted {
                    rd,
                    ri: hi_rd.to_reg(),
                    imm,
                });
            } else if let Some(imm) = UImm32Shifted::maybe_from_u64(lo) {
                // 32-bit shifted immediate
                insts.push(Inst::Insert64UImm32Shifted {
                    rd,
                    ri: hi_rd.to_reg(),
                    imm,
                });
            } else {
                unreachable!();
            }

            insts
        }
    }

    /// Create an instruction that loads a 32-bit integer constant.
    pub fn load_constant32(rd: Writable<Reg>, value: u32) -> SmallVec<[Inst; 4]> {
        if let Ok(imm) = i16::try_from(value as i32) {
            // 16-bit signed immediate
            smallvec![Inst::Mov32SImm16 { rd, imm }]
        } else {
            // 32-bit full immediate
            smallvec![Inst::Mov32Imm { rd, imm: value }]
        }
    }

    /// Create an instruction that loads a 32-bit floating-point constant.
    pub fn load_fp_constant32(rd: Writable<Reg>, value: f32) -> Inst {
        // TODO: use LZER to load 0.0
        Inst::LoadFpuConst32 {
            rd,
            const_data: value.to_bits(),
        }
    }

    /// Create an instruction that loads a 64-bit floating-point constant.
    pub fn load_fp_constant64(rd: Writable<Reg>, value: f64) -> Inst {
        // TODO: use LZDR to load 0.0
        Inst::LoadFpuConst64 {
            rd,
            const_data: value.to_bits(),
        }
    }

    /// Create an instruction that loads a 128-bit floating-point constant.
    pub fn load_vec_constant(rd: Writable<Reg>, value: u128) -> Inst {
        // FIXME: This doesn't special-case constants that can be loaded
        // without a constant pool, like the ISLE lowering does.  Ideally,
        // we should not have to duplicate the logic here.
        Inst::VecLoadConst {
            rd,
            const_data: value,
        }
    }

    /// Generic constructor for a load (zero-extending where appropriate).
    pub fn gen_load(into_reg: Writable<Reg>, mem: MemArg, ty: Type) -> Inst {
        match ty {
            types::I8 => Inst::Load64ZExt8 { rd: into_reg, mem },
            types::I16 => Inst::Load64ZExt16 { rd: into_reg, mem },
            types::I32 => Inst::Load64ZExt32 { rd: into_reg, mem },
            types::I64 | types::R64 => Inst::Load64 { rd: into_reg, mem },
            types::F32 => Inst::VecLoadLaneUndef {
                size: 32,
                rd: into_reg,
                mem,
                lane_imm: 0,
            },
            types::F64 => Inst::VecLoadLaneUndef {
                size: 64,
                rd: into_reg,
                mem,
                lane_imm: 0,
            },
            _ if ty.is_vector() && ty.bits() == 128 => Inst::VecLoad { rd: into_reg, mem },
            types::I128 => Inst::VecLoad { rd: into_reg, mem },
            _ => unimplemented!("gen_load({})", ty),
        }
    }

    /// Generic constructor for a store.
    pub fn gen_store(mem: MemArg, from_reg: Reg, ty: Type) -> Inst {
        match ty {
            types::I8 => Inst::Store8 { rd: from_reg, mem },
            types::I16 => Inst::Store16 { rd: from_reg, mem },
            types::I32 => Inst::Store32 { rd: from_reg, mem },
            types::I64 | types::R64 => Inst::Store64 { rd: from_reg, mem },
            types::F32 => Inst::VecStoreLane {
                size: 32,
                rd: from_reg,
                mem,
                lane_imm: 0,
            },
            types::F64 => Inst::VecStoreLane {
                size: 64,
                rd: from_reg,
                mem,
                lane_imm: 0,
            },
            _ if ty.is_vector() && ty.bits() == 128 => Inst::VecStore { rd: from_reg, mem },
            types::I128 => Inst::VecStore { rd: from_reg, mem },
            _ => unimplemented!("gen_store({})", ty),
        }
    }
}

//=============================================================================
// Instructions: get_regs

fn memarg_operands<F: Fn(VReg) -> VReg>(memarg: &MemArg, collector: &mut OperandCollector<'_, F>) {
    match memarg {
        &MemArg::BXD12 { base, index, .. } | &MemArg::BXD20 { base, index, .. } => {
            collector.reg_use(base);
            collector.reg_use(index);
        }
        &MemArg::Label { .. } | &MemArg::Symbol { .. } => {}
        &MemArg::RegOffset { reg, .. } => {
            collector.reg_use(reg);
        }
        &MemArg::InitialSPOffset { .. } | &MemArg::NominalSPOffset { .. } => {}
    }
    // mem_finalize might require %r1 to hold (part of) the address.
    // Conservatively assume this will always be necessary here.
    collector.reg_early_def(writable_gpr(1));
}

fn s390x_get_operands<F: Fn(VReg) -> VReg>(inst: &Inst, collector: &mut OperandCollector<'_, F>) {
    match inst {
        &Inst::AluRRR { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::AluRRSImm16 { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::AluRR { rd, ri, rm, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rm);
        }
        &Inst::AluRX {
            rd, ri, ref mem, ..
        } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            memarg_operands(mem, collector);
        }
        &Inst::AluRSImm16 { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::AluRSImm32 { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::AluRUImm32 { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::AluRUImm16Shifted { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::AluRUImm32Shifted { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::SMulWide { rd, rn, rm } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
            // FIXME: The pair is hard-coded as %r2/%r3 because regalloc cannot handle pairs. If
            // that changes, all the hard-coded uses of %r2/%r3 can be changed.
            collector.reg_fixed_def(rd.hi, gpr(2));
            collector.reg_fixed_def(rd.lo, gpr(3));
        }
        &Inst::UMulWide { rd, ri, rn } => {
            collector.reg_use(rn);
            collector.reg_fixed_def(rd.hi, gpr(2));
            collector.reg_fixed_def(rd.lo, gpr(3));
            collector.reg_fixed_use(ri, gpr(3));
        }
        &Inst::SDivMod32 { rd, ri, rn } | &Inst::SDivMod64 { rd, ri, rn } => {
            collector.reg_use(rn);
            collector.reg_fixed_def(rd.hi, gpr(2));
            collector.reg_fixed_def(rd.lo, gpr(3));
            collector.reg_fixed_use(ri, gpr(3));
        }
        &Inst::UDivMod32 { rd, ri, rn } | &Inst::UDivMod64 { rd, ri, rn } => {
            collector.reg_use(rn);
            collector.reg_fixed_def(rd.hi, gpr(2));
            collector.reg_fixed_def(rd.lo, gpr(3));
            collector.reg_fixed_use(ri.hi, gpr(2));
            collector.reg_fixed_use(ri.lo, gpr(3));
        }
        &Inst::Flogr { rd, rn } => {
            collector.reg_use(rn);
            collector.reg_fixed_def(rd.hi, gpr(2));
            collector.reg_fixed_def(rd.lo, gpr(3));
        }
        &Inst::ShiftRR {
            rd, rn, shift_reg, ..
        } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            if shift_reg != zero_reg() {
                collector.reg_use(shift_reg);
            }
        }
        &Inst::RxSBG { rd, ri, rn, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rn);
        }
        &Inst::RxSBGTest { rd, rn, .. } => {
            collector.reg_use(rd);
            collector.reg_use(rn);
        }
        &Inst::UnaryRR { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::CmpRR { rn, rm, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::CmpRX { rn, ref mem, .. } => {
            collector.reg_use(rn);
            memarg_operands(mem, collector);
        }
        &Inst::CmpRSImm16 { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::CmpRSImm32 { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::CmpRUImm32 { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::CmpTrapRR { rn, rm, .. } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::CmpTrapRSImm16 { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::CmpTrapRUImm16 { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::AtomicRmw {
            rd, rn, ref mem, ..
        } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            memarg_operands(mem, collector);
        }
        &Inst::AtomicCas32 {
            rd,
            ri,
            rn,
            ref mem,
            ..
        }
        | &Inst::AtomicCas64 {
            rd,
            ri,
            rn,
            ref mem,
            ..
        } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rn);
            memarg_operands(mem, collector);
        }
        &Inst::Fence => {}
        &Inst::Load32 { rd, ref mem, .. }
        | &Inst::Load32ZExt8 { rd, ref mem, .. }
        | &Inst::Load32SExt8 { rd, ref mem, .. }
        | &Inst::Load32ZExt16 { rd, ref mem, .. }
        | &Inst::Load32SExt16 { rd, ref mem, .. }
        | &Inst::Load64 { rd, ref mem, .. }
        | &Inst::Load64ZExt8 { rd, ref mem, .. }
        | &Inst::Load64SExt8 { rd, ref mem, .. }
        | &Inst::Load64ZExt16 { rd, ref mem, .. }
        | &Inst::Load64SExt16 { rd, ref mem, .. }
        | &Inst::Load64ZExt32 { rd, ref mem, .. }
        | &Inst::Load64SExt32 { rd, ref mem, .. }
        | &Inst::LoadRev16 { rd, ref mem, .. }
        | &Inst::LoadRev32 { rd, ref mem, .. }
        | &Inst::LoadRev64 { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::Store8 { rd, ref mem, .. }
        | &Inst::Store16 { rd, ref mem, .. }
        | &Inst::Store32 { rd, ref mem, .. }
        | &Inst::Store64 { rd, ref mem, .. }
        | &Inst::StoreRev16 { rd, ref mem, .. }
        | &Inst::StoreRev32 { rd, ref mem, .. }
        | &Inst::StoreRev64 { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::StoreImm8 { ref mem, .. }
        | &Inst::StoreImm16 { ref mem, .. }
        | &Inst::StoreImm32SExt16 { ref mem, .. }
        | &Inst::StoreImm64SExt16 { ref mem, .. } => {
            memarg_operands(mem, collector);
        }
        &Inst::Mvc {
            ref dst, ref src, ..
        } => {
            collector.reg_use(dst.base);
            collector.reg_use(src.base);
        }
        &Inst::LoadMultiple64 {
            rt, rt2, ref mem, ..
        } => {
            memarg_operands(mem, collector);
            let first_regnum = rt.to_reg().to_real_reg().unwrap().hw_enc();
            let last_regnum = rt2.to_reg().to_real_reg().unwrap().hw_enc();
            for regnum in first_regnum..last_regnum + 1 {
                collector.reg_def(writable_gpr(regnum));
            }
        }
        &Inst::StoreMultiple64 {
            rt, rt2, ref mem, ..
        } => {
            memarg_operands(mem, collector);
            let first_regnum = rt.to_real_reg().unwrap().hw_enc();
            let last_regnum = rt2.to_real_reg().unwrap().hw_enc();
            for regnum in first_regnum..last_regnum + 1 {
                collector.reg_use(gpr(regnum));
            }
        }
        &Inst::Mov64 { rd, rm } => {
            collector.reg_def(rd);
            collector.reg_use(rm);
        }
        &Inst::MovPReg { rd, rm } => {
            debug_assert!([regs::gpr(0), regs::gpr(14), regs::gpr(15)].contains(&rm.into()));
            debug_assert!(rd.to_reg().is_virtual());
            collector.reg_def(rd);
        }
        &Inst::Mov32 { rd, rm } => {
            collector.reg_def(rd);
            collector.reg_use(rm);
        }
        &Inst::Mov32Imm { rd, .. }
        | &Inst::Mov32SImm16 { rd, .. }
        | &Inst::Mov64SImm16 { rd, .. }
        | &Inst::Mov64SImm32 { rd, .. }
        | &Inst::Mov64UImm16Shifted { rd, .. }
        | &Inst::Mov64UImm32Shifted { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::CMov32 { rd, ri, rm, .. } | &Inst::CMov64 { rd, ri, rm, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rm);
        }
        &Inst::CMov32SImm16 { rd, ri, .. } | &Inst::CMov64SImm16 { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::Insert64UImm16Shifted { rd, ri, .. }
        | &Inst::Insert64UImm32Shifted { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::LoadAR { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::InsertAR { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::FpuMove32 { rd, rn } | &Inst::FpuMove64 { rd, rn } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::FpuCMov32 { rd, ri, rm, .. } | &Inst::FpuCMov64 { rd, ri, rm, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rm);
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
        &Inst::FpuRRRR { rd, rn, rm, ra, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_use(ra);
        }
        &Inst::FpuCmp32 { rn, rm } | &Inst::FpuCmp64 { rn, rm } => {
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::LoadFpuConst32 { rd, .. } | &Inst::LoadFpuConst64 { rd, .. } => {
            collector.reg_def(rd);
            collector.reg_def(writable_gpr(1));
        }
        &Inst::FpuRound { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecRRR { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecRR { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecShiftRR {
            rd, rn, shift_reg, ..
        } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            if shift_reg != zero_reg() {
                collector.reg_use(shift_reg);
            }
        }
        &Inst::VecSelect { rd, rn, rm, ra, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_use(ra);
        }
        &Inst::VecPermute { rd, rn, rm, ra, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
            collector.reg_use(ra);
        }
        &Inst::VecPermuteDWImm { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecIntCmp { rd, rn, rm, .. } | &Inst::VecIntCmpS { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecFloatCmp { rd, rn, rm, .. } | &Inst::VecFloatCmpS { rd, rn, rm, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecInt128SCmpHi { tmp, rn, rm, .. } | &Inst::VecInt128UCmpHi { tmp, rn, rm, .. } => {
            collector.reg_def(tmp);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecLoad { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadRev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadByte16Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadByte32Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadByte64Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadElt16Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadElt32Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadElt64Rev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStore { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreRev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreByte16Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreByte32Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreByte64Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreElt16Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreElt32Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreElt64Rev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadReplicate { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadReplicateRev { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecMov { rd, rn } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::VecCMov { rd, ri, rm, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rm);
        }
        &Inst::MovToVec128 { rd, rn, rm } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            collector.reg_use(rm);
        }
        &Inst::VecLoadConst { rd, .. } | &Inst::VecLoadConstReplicate { rd, .. } => {
            collector.reg_def(rd);
            collector.reg_def(writable_gpr(1));
        }
        &Inst::VecImmByteMask { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::VecImmBitMask { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::VecImmReplicate { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::VecLoadLane {
            rd, ri, ref mem, ..
        } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadLaneUndef { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreLaneRev { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadLaneRevUndef { rd, ref mem, .. } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecStoreLane { rd, ref mem, .. } => {
            collector.reg_use(rd);
            memarg_operands(mem, collector);
        }
        &Inst::VecLoadLaneRev {
            rd, ri, ref mem, ..
        } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            memarg_operands(mem, collector);
        }
        &Inst::VecInsertLane {
            rd,
            ri,
            rn,
            lane_reg,
            ..
        } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
            collector.reg_use(rn);
            if lane_reg != zero_reg() {
                collector.reg_use(lane_reg);
            }
        }
        &Inst::VecInsertLaneUndef {
            rd, rn, lane_reg, ..
        } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            if lane_reg != zero_reg() {
                collector.reg_use(lane_reg);
            }
        }
        &Inst::VecExtractLane {
            rd, rn, lane_reg, ..
        } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
            if lane_reg != zero_reg() {
                collector.reg_use(lane_reg);
            }
        }
        &Inst::VecInsertLaneImm { rd, ri, .. } => {
            collector.reg_reuse_def(rd, 1);
            collector.reg_use(ri);
        }
        &Inst::VecReplicateLane { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::Extend { rd, rn, .. } => {
            collector.reg_def(rd);
            collector.reg_use(rn);
        }
        &Inst::Call { link, ref info } => {
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            let mut clobbers = info.clobbers.clone();
            clobbers.add(link.to_reg().to_real_reg().unwrap().into());
            collector.reg_clobbers(clobbers);
        }
        &Inst::CallInd { link, ref info } => {
            collector.reg_use(info.rn);
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            let mut clobbers = info.clobbers.clone();
            clobbers.add(link.to_reg().to_real_reg().unwrap().into());
            collector.reg_clobbers(clobbers);
        }
        &Inst::Args { ref args } => {
            for arg in args {
                collector.reg_fixed_def(arg.vreg, arg.preg);
            }
        }
        &Inst::Ret { ref rets, .. } => {
            // NOTE: we explicitly don't mark the link register as used here, as the use is only in
            // the epilog where callee-save registers are restored.
            for ret in rets {
                collector.reg_fixed_use(ret.vreg, ret.preg);
            }
        }
        &Inst::Jump { .. } => {}
        &Inst::IndirectBr { rn, .. } => {
            collector.reg_use(rn);
        }
        &Inst::CondBr { .. } | &Inst::OneWayCondBr { .. } => {}
        &Inst::Nop0 | Inst::Nop2 => {}
        &Inst::Debugtrap => {}
        &Inst::Trap { .. } => {}
        &Inst::TrapIf { .. } => {}
        &Inst::JTSequence { ridx, .. } => {
            collector.reg_use(ridx);
            collector.reg_early_def(writable_gpr(1));
        }
        &Inst::LoadSymbolReloc { rd, .. } => {
            collector.reg_def(rd);
            collector.reg_def(writable_gpr(1));
        }
        &Inst::LoadAddr { rd, ref mem } => {
            collector.reg_def(rd);
            memarg_operands(mem, collector);
        }
        &Inst::Loop { ref body, .. } => {
            for inst in body.iter() {
                s390x_get_operands(inst, collector);
            }

            // `reuse_def` constraints can't be permitted in a Loop instruction because the operand
            // index will always be relative to the Loop instruction, not the individual
            // instruction in the loop body. However, fixed-nonallocatable registers used with
            // instructions that would have emitted `reuse_def` constraints are fine.
            debug_assert!(collector.no_reuse_def());
        }
        &Inst::CondBreak { .. } => {}
        &Inst::VirtualSPOffsetAdj { .. } => {}
        &Inst::Unwind { .. } => {}
        &Inst::DummyUse { reg } => {
            collector.reg_use(reg);
        }
    }
}

//=============================================================================
// Instructions: misc functions and external interface

impl MachInst for Inst {
    type ABIMachineSpec = S390xMachineDeps;
    type LabelUse = LabelUse;

    fn get_operands<F: Fn(VReg) -> VReg>(&self, collector: &mut OperandCollector<'_, F>) {
        s390x_get_operands(self, collector);
    }

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        match self {
            &Inst::Mov32 { rd, rm } => Some((rd, rm)),
            &Inst::Mov64 { rd, rm } => Some((rd, rm)),
            &Inst::FpuMove32 { rd, rn } => Some((rd, rn)),
            &Inst::FpuMove64 { rd, rn } => Some((rd, rn)),
            &Inst::VecMov { rd, rn } => Some((rd, rn)),
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
        match self {
            &Inst::Args { .. } => false,
            &Inst::Call { ref info, .. } => info.caller_callconv != info.callee_callconv,
            &Inst::CallInd { ref info, .. } => info.caller_callconv != info.callee_callconv,
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
            &Inst::Ret { .. } => MachTerminator::Ret,
            &Inst::Jump { .. } => MachTerminator::Uncond,
            &Inst::CondBr { .. } => MachTerminator::Cond,
            &Inst::OneWayCondBr { .. } => {
                // Explicitly invisible to CFG processing.
                MachTerminator::None
            }
            &Inst::IndirectBr { .. } => MachTerminator::Indirect,
            &Inst::JTSequence { .. } => MachTerminator::Indirect,
            _ => MachTerminator::None,
        }
    }

    fn is_safepoint(&self) -> bool {
        match self {
            &Inst::Call { .. }
            | &Inst::CallInd { .. }
            | &Inst::Trap { .. }
            | Inst::TrapIf { .. }
            | &Inst::CmpTrapRR { .. }
            | &Inst::CmpTrapRSImm16 { .. }
            | &Inst::CmpTrapRUImm16 { .. } => true,
            _ => false,
        }
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Inst {
        assert!(ty.bits() <= 128);
        if ty.bits() <= 32 {
            Inst::mov32(to_reg, from_reg)
        } else if ty.bits() <= 64 {
            Inst::mov64(to_reg, from_reg)
        } else {
            Inst::mov128(to_reg, from_reg)
        }
    }

    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        let to_reg = to_regs
            .only_reg()
            .expect("multi-reg values not supported yet");
        match ty {
            types::I128 => {
                let mut ret = SmallVec::new();
                ret.push(Inst::load_vec_constant(to_reg, value));
                ret
            }
            _ if ty.is_vector() && ty.bits() == 128 => {
                let mut ret = SmallVec::new();
                ret.push(Inst::load_vec_constant(to_reg, value));
                ret
            }
            types::F64 => {
                let mut ret = SmallVec::new();
                ret.push(Inst::load_fp_constant64(
                    to_reg,
                    f64::from_bits(value as u64),
                ));
                ret
            }
            types::F32 => {
                let mut ret = SmallVec::new();
                ret.push(Inst::load_fp_constant32(
                    to_reg,
                    f32::from_bits(value as u32),
                ));
                ret
            }
            types::I64 | types::R64 => Inst::load_constant64(to_reg, value as u64, alloc_tmp),
            types::I8 | types::I16 | types::I32 => Inst::load_constant32(to_reg, value as u32),
            _ => unreachable!(),
        }
    }

    fn gen_nop(preferred_size: usize) -> Inst {
        if preferred_size == 0 {
            Inst::Nop0
        } else {
            // We can't give a NOP (or any insn) < 2 bytes.
            assert!(preferred_size >= 2);
            Inst::Nop2
        }
    }

    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
        match ty {
            types::I8 => Ok((&[RegClass::Int], &[types::I8])),
            types::I16 => Ok((&[RegClass::Int], &[types::I16])),
            types::I32 => Ok((&[RegClass::Int], &[types::I32])),
            types::I64 => Ok((&[RegClass::Int], &[types::I64])),
            types::R32 => panic!("32-bit reftype pointer should never be seen on s390x"),
            types::R64 => Ok((&[RegClass::Int], &[types::R64])),
            types::F32 => Ok((&[RegClass::Float], &[types::F32])),
            types::F64 => Ok((&[RegClass::Float], &[types::F64])),
            types::I128 => Ok((&[RegClass::Float], &[types::I128])),
            _ if ty.is_vector() && ty.bits() == 128 => Ok((&[RegClass::Float], &[types::I8X16])),
            // FIXME: We don't really have IFLAGS, but need to allow it here
            // for now to support the SelectifSpectreGuard instruction.
            types::IFLAGS => Ok((&[RegClass::Int], &[types::I64])),
            _ => Err(CodegenError::Unsupported(format!(
                "Unexpected SSA-value type: {}",
                ty
            ))),
        }
    }

    fn canonical_type_for_rc(rc: RegClass) -> Type {
        match rc {
            RegClass::Int => types::I64,
            RegClass::Float => types::I8X16,
        }
    }

    fn gen_jump(target: MachLabel) -> Inst {
        Inst::Jump { dest: target }
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

    fn gen_dummy_use(reg: Reg) -> Inst {
        Inst::DummyUse { reg }
    }
}

//=============================================================================
// Pretty-printing of instructions.

fn mem_finalize_for_show(mem: &MemArg, state: &EmitState, mi: MemInstType) -> (String, MemArg) {
    let (mem_insts, mem) = mem_finalize(mem, state, mi);
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
    fn print_with_state(
        &self,
        state: &mut EmitState,
        allocs: &mut AllocationConsumer<'_>,
    ) -> String {
        // N.B.: order of consumption of `allocs` must match the order
        // in `s390x_get_operands()`.

        let mut empty_allocs = AllocationConsumer::new(&[]);

        match self {
            &Inst::Nop0 => "nop-zero-len".to_string(),
            &Inst::Nop2 => "nop".to_string(),
            &Inst::AluRRR { alu_op, rd, rn, rm } => {
                let rd = allocs.next_writable(rd);
                let rn = allocs.next(rn);
                let rm = allocs.next(rm);

                let (op, have_rr) = match alu_op {
                    ALUOp::Add32 => ("ark", true),
                    ALUOp::Add64 => ("agrk", true),
                    ALUOp::AddLogical32 => ("alrk", true),
                    ALUOp::AddLogical64 => ("algrk", true),
                    ALUOp::Sub32 => ("srk", true),
                    ALUOp::Sub64 => ("sgrk", true),
                    ALUOp::SubLogical32 => ("slrk", true),
                    ALUOp::SubLogical64 => ("slgrk", true),
                    ALUOp::Mul32 => ("msrkc", true),
                    ALUOp::Mul64 => ("msgrkc", true),
                    ALUOp::And32 => ("nrk", true),
                    ALUOp::And64 => ("ngrk", true),
                    ALUOp::Orr32 => ("ork", true),
                    ALUOp::Orr64 => ("ogrk", true),
                    ALUOp::Xor32 => ("xrk", true),
                    ALUOp::Xor64 => ("xgrk", true),
                    ALUOp::NotAnd32 => ("nnrk", false),
                    ALUOp::NotAnd64 => ("nngrk", false),
                    ALUOp::NotOrr32 => ("nork", false),
                    ALUOp::NotOrr64 => ("nogrk", false),
                    ALUOp::NotXor32 => ("nxrk", false),
                    ALUOp::NotXor64 => ("nxgrk", false),
                    ALUOp::AndNot32 => ("ncrk", false),
                    ALUOp::AndNot64 => ("ncgrk", false),
                    ALUOp::OrrNot32 => ("ocrk", false),
                    ALUOp::OrrNot64 => ("ocgrk", false),
                    _ => unreachable!(),
                };
                if have_rr && rd.to_reg() == rn {
                    let inst = Inst::AluRR {
                        alu_op,
                        rd,
                        ri: rd.to_reg(),
                        rm,
                    };
                    return inst.print_with_state(state, &mut empty_allocs);
                }
                let rd = pretty_print_reg(rd.to_reg(), &mut empty_allocs);
                let rn = pretty_print_reg(rn, &mut empty_allocs);
                let rm = pretty_print_reg(rm, &mut empty_allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::AluRRSImm16 {
                alu_op,
                rd,
                rn,
                imm,
            } => {
                let rd = allocs.next_writable(rd);
                let rn = allocs.next(rn);

                if rd.to_reg() == rn {
                    let inst = Inst::AluRSImm16 {
                        alu_op,
                        rd,
                        ri: rd.to_reg(),
                        imm,
                    };
                    return inst.print_with_state(state, &mut empty_allocs);
                }
                let op = match alu_op {
                    ALUOp::Add32 => "ahik",
                    ALUOp::Add64 => "aghik",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd.to_reg(), &mut empty_allocs);
                let rn = pretty_print_reg(rn, &mut empty_allocs);
                format!("{} {}, {}, {}", op, rd, rn, imm)
            }
            &Inst::AluRR { alu_op, rd, ri, rm } => {
                let op = match alu_op {
                    ALUOp::Add32 => "ar",
                    ALUOp::Add64 => "agr",
                    ALUOp::Add64Ext32 => "agfr",
                    ALUOp::AddLogical32 => "alr",
                    ALUOp::AddLogical64 => "algr",
                    ALUOp::AddLogical64Ext32 => "algfr",
                    ALUOp::Sub32 => "sr",
                    ALUOp::Sub64 => "sgr",
                    ALUOp::Sub64Ext32 => "sgfr",
                    ALUOp::SubLogical32 => "slr",
                    ALUOp::SubLogical64 => "slgr",
                    ALUOp::SubLogical64Ext32 => "slgfr",
                    ALUOp::Mul32 => "msr",
                    ALUOp::Mul64 => "msgr",
                    ALUOp::Mul64Ext32 => "msgfr",
                    ALUOp::And32 => "nr",
                    ALUOp::And64 => "ngr",
                    ALUOp::Orr32 => "or",
                    ALUOp::Orr64 => "ogr",
                    ALUOp::Xor32 => "xr",
                    ALUOp::Xor64 => "xgr",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("{} {}, {}", op, rd, rm)
            }
            &Inst::AluRX {
                alu_op,
                rd,
                ri,
                ref mem,
            } => {
                let (opcode_rx, opcode_rxy) = match alu_op {
                    ALUOp::Add32 => (Some("a"), Some("ay")),
                    ALUOp::Add32Ext16 => (Some("ah"), Some("ahy")),
                    ALUOp::Add64 => (None, Some("ag")),
                    ALUOp::Add64Ext16 => (None, Some("agh")),
                    ALUOp::Add64Ext32 => (None, Some("agf")),
                    ALUOp::AddLogical32 => (Some("al"), Some("aly")),
                    ALUOp::AddLogical64 => (None, Some("alg")),
                    ALUOp::AddLogical64Ext32 => (None, Some("algf")),
                    ALUOp::Sub32 => (Some("s"), Some("sy")),
                    ALUOp::Sub32Ext16 => (Some("sh"), Some("shy")),
                    ALUOp::Sub64 => (None, Some("sg")),
                    ALUOp::Sub64Ext16 => (None, Some("sgh")),
                    ALUOp::Sub64Ext32 => (None, Some("sgf")),
                    ALUOp::SubLogical32 => (Some("sl"), Some("sly")),
                    ALUOp::SubLogical64 => (None, Some("slg")),
                    ALUOp::SubLogical64Ext32 => (None, Some("slgf")),
                    ALUOp::Mul32 => (Some("ms"), Some("msy")),
                    ALUOp::Mul32Ext16 => (Some("mh"), Some("mhy")),
                    ALUOp::Mul64 => (None, Some("msg")),
                    ALUOp::Mul64Ext16 => (None, Some("mgh")),
                    ALUOp::Mul64Ext32 => (None, Some("msgf")),
                    ALUOp::And32 => (Some("n"), Some("ny")),
                    ALUOp::And64 => (None, Some("ng")),
                    ALUOp::Orr32 => (Some("o"), Some("oy")),
                    ALUOp::Orr64 => (None, Some("og")),
                    ALUOp::Xor32 => (Some("x"), Some("xy")),
                    ALUOp::Xor64 => (None, Some("xg")),
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: opcode_rx.is_some(),
                        have_d20: opcode_rxy.is_some(),
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => opcode_rx,
                    &MemArg::BXD20 { .. } => opcode_rxy,
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op.unwrap(), rd, mem)
            }
            &Inst::AluRSImm16 {
                alu_op,
                rd,
                ri,
                imm,
            } => {
                let op = match alu_op {
                    ALUOp::Add32 => "ahi",
                    ALUOp::Add64 => "aghi",
                    ALUOp::Mul32 => "mhi",
                    ALUOp::Mul64 => "mghi",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::AluRSImm32 {
                alu_op,
                rd,
                ri,
                imm,
            } => {
                let op = match alu_op {
                    ALUOp::Add32 => "afi",
                    ALUOp::Add64 => "agfi",
                    ALUOp::Mul32 => "msfi",
                    ALUOp::Mul64 => "msgfi",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::AluRUImm32 {
                alu_op,
                rd,
                ri,
                imm,
            } => {
                let op = match alu_op {
                    ALUOp::AddLogical32 => "alfi",
                    ALUOp::AddLogical64 => "algfi",
                    ALUOp::SubLogical32 => "slfi",
                    ALUOp::SubLogical64 => "slgfi",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::AluRUImm16Shifted {
                alu_op,
                rd,
                ri,
                imm,
            } => {
                let op = match (alu_op, imm.shift) {
                    (ALUOp::And32, 0) => "nill",
                    (ALUOp::And32, 1) => "nilh",
                    (ALUOp::And64, 0) => "nill",
                    (ALUOp::And64, 1) => "nilh",
                    (ALUOp::And64, 2) => "nihl",
                    (ALUOp::And64, 3) => "nihh",
                    (ALUOp::Orr32, 0) => "oill",
                    (ALUOp::Orr32, 1) => "oilh",
                    (ALUOp::Orr64, 0) => "oill",
                    (ALUOp::Orr64, 1) => "oilh",
                    (ALUOp::Orr64, 2) => "oihl",
                    (ALUOp::Orr64, 3) => "oihh",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::AluRUImm32Shifted {
                alu_op,
                rd,
                ri,
                imm,
            } => {
                let op = match (alu_op, imm.shift) {
                    (ALUOp::And32, 0) => "nilf",
                    (ALUOp::And64, 0) => "nilf",
                    (ALUOp::And64, 1) => "nihf",
                    (ALUOp::Orr32, 0) => "oilf",
                    (ALUOp::Orr64, 0) => "oilf",
                    (ALUOp::Orr64, 1) => "oihf",
                    (ALUOp::Xor32, 0) => "xilf",
                    (ALUOp::Xor64, 0) => "xilf",
                    (ALUOp::Xor64, 1) => "xihf",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::SMulWide { rd, rn, rm } => {
                let op = "mgrk";
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let rd = pretty_print_regpair(rd.to_regpair(), allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::UMulWide { rd, ri, rn } => {
                let op = "mlgr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair_mod_lo(rd, ri, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::SDivMod32 { rd, ri, rn } => {
                let op = "dsgfr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair_mod_lo(rd, ri, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::SDivMod64 { rd, ri, rn } => {
                let op = "dsgr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair_mod_lo(rd, ri, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::UDivMod32 { rd, ri, rn } => {
                let op = "dlr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::UDivMod64 { rd, ri, rn } => {
                let op = "dlgr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair_mod(rd, ri, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::Flogr { rd, rn } => {
                let op = "flogr";
                let rn = pretty_print_reg(rn, allocs);
                let rd = pretty_print_regpair(rd.to_regpair(), allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::ShiftRR {
                shift_op,
                rd,
                rn,
                shift_imm,
                shift_reg,
            } => {
                let op = match shift_op {
                    ShiftOp::RotL32 => "rll",
                    ShiftOp::RotL64 => "rllg",
                    ShiftOp::LShL32 => "sllk",
                    ShiftOp::LShL64 => "sllg",
                    ShiftOp::LShR32 => "srlk",
                    ShiftOp::LShR64 => "srlg",
                    ShiftOp::AShR32 => "srak",
                    ShiftOp::AShR64 => "srag",
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let shift_reg = if shift_reg != zero_reg() {
                    format!("({})", pretty_print_reg(shift_reg, allocs))
                } else {
                    "".to_string()
                };
                format!("{} {}, {}, {}{}", op, rd, rn, shift_imm, shift_reg)
            }
            &Inst::RxSBG {
                op,
                rd,
                ri,
                rn,
                start_bit,
                end_bit,
                rotate_amt,
            } => {
                let op = match op {
                    RxSBGOp::Insert => "risbgn",
                    RxSBGOp::And => "rnsbg",
                    RxSBGOp::Or => "rosbg",
                    RxSBGOp::Xor => "rxsbg",
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!(
                    "{} {}, {}, {}, {}, {}",
                    op,
                    rd,
                    rn,
                    start_bit,
                    end_bit,
                    (rotate_amt as u8) & 63
                )
            }
            &Inst::RxSBGTest {
                op,
                rd,
                rn,
                start_bit,
                end_bit,
                rotate_amt,
            } => {
                let op = match op {
                    RxSBGOp::And => "rnsbg",
                    RxSBGOp::Or => "rosbg",
                    RxSBGOp::Xor => "rxsbg",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd, allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!(
                    "{} {}, {}, {}, {}, {}",
                    op,
                    rd,
                    rn,
                    start_bit | 0x80,
                    end_bit,
                    (rotate_amt as u8) & 63
                )
            }
            &Inst::UnaryRR { op, rd, rn } => {
                let (op, extra) = match op {
                    UnaryOp::Abs32 => ("lpr", ""),
                    UnaryOp::Abs64 => ("lpgr", ""),
                    UnaryOp::Abs64Ext32 => ("lpgfr", ""),
                    UnaryOp::Neg32 => ("lcr", ""),
                    UnaryOp::Neg64 => ("lcgr", ""),
                    UnaryOp::Neg64Ext32 => ("lcgfr", ""),
                    UnaryOp::PopcntByte => ("popcnt", ""),
                    UnaryOp::PopcntReg => ("popcnt", ", 8"),
                    UnaryOp::BSwap32 => ("lrvr", ""),
                    UnaryOp::BSwap64 => ("lrvgr", ""),
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}{}", op, rd, rn, extra)
            }
            &Inst::CmpRR { op, rn, rm } => {
                let op = match op {
                    CmpOp::CmpS32 => "cr",
                    CmpOp::CmpS64 => "cgr",
                    CmpOp::CmpS64Ext32 => "cgfr",
                    CmpOp::CmpL32 => "clr",
                    CmpOp::CmpL64 => "clgr",
                    CmpOp::CmpL64Ext32 => "clgfr",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("{} {}, {}", op, rn, rm)
            }
            &Inst::CmpRX { op, rn, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match op {
                    CmpOp::CmpS32 => (Some("c"), Some("cy"), Some("crl")),
                    CmpOp::CmpS32Ext16 => (Some("ch"), Some("chy"), Some("chrl")),
                    CmpOp::CmpS64 => (None, Some("cg"), Some("cgrl")),
                    CmpOp::CmpS64Ext16 => (None, Some("cgh"), Some("cghrl")),
                    CmpOp::CmpS64Ext32 => (None, Some("cgf"), Some("cgfrl")),
                    CmpOp::CmpL32 => (Some("cl"), Some("cly"), Some("clrl")),
                    CmpOp::CmpL32Ext16 => (None, None, Some("clhrl")),
                    CmpOp::CmpL64 => (None, Some("clg"), Some("clgrl")),
                    CmpOp::CmpL64Ext16 => (None, None, Some("clghrl")),
                    CmpOp::CmpL64Ext32 => (None, Some("clgf"), Some("clgfrl")),
                };

                let rn = pretty_print_reg(rn, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: opcode_rx.is_some(),
                        have_d20: opcode_rxy.is_some(),
                        have_pcrel: opcode_ril.is_some(),
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => opcode_rx,
                    &MemArg::BXD20 { .. } => opcode_rxy,
                    &MemArg::Label { .. } | &MemArg::Symbol { .. } => opcode_ril,
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op.unwrap(), rn, mem)
            }
            &Inst::CmpRSImm16 { op, rn, imm } => {
                let op = match op {
                    CmpOp::CmpS32 => "chi",
                    CmpOp::CmpS64 => "cghi",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}", op, rn, imm)
            }
            &Inst::CmpRSImm32 { op, rn, imm } => {
                let op = match op {
                    CmpOp::CmpS32 => "cfi",
                    CmpOp::CmpS64 => "cgfi",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}", op, rn, imm)
            }
            &Inst::CmpRUImm32 { op, rn, imm } => {
                let op = match op {
                    CmpOp::CmpL32 => "clfi",
                    CmpOp::CmpL64 => "clgfi",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}", op, rn, imm)
            }
            &Inst::CmpTrapRR {
                op, rn, rm, cond, ..
            } => {
                let op = match op {
                    CmpOp::CmpS32 => "crt",
                    CmpOp::CmpS64 => "cgrt",
                    CmpOp::CmpL32 => "clrt",
                    CmpOp::CmpL64 => "clgrt",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let cond = cond.pretty_print_default();
                format!("{}{} {}, {}", op, cond, rn, rm)
            }
            &Inst::CmpTrapRSImm16 {
                op, rn, imm, cond, ..
            } => {
                let op = match op {
                    CmpOp::CmpS32 => "cit",
                    CmpOp::CmpS64 => "cgit",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                let cond = cond.pretty_print_default();
                format!("{}{} {}, {}", op, cond, rn, imm)
            }
            &Inst::CmpTrapRUImm16 {
                op, rn, imm, cond, ..
            } => {
                let op = match op {
                    CmpOp::CmpL32 => "clfit",
                    CmpOp::CmpL64 => "clgit",
                    _ => unreachable!(),
                };
                let rn = pretty_print_reg(rn, allocs);
                let cond = cond.pretty_print_default();
                format!("{}{} {}, {}", op, cond, rn, imm)
            }
            &Inst::AtomicRmw {
                alu_op,
                rd,
                rn,
                ref mem,
            } => {
                let op = match alu_op {
                    ALUOp::Add32 => "laa",
                    ALUOp::Add64 => "laag",
                    ALUOp::AddLogical32 => "laal",
                    ALUOp::AddLogical64 => "laalg",
                    ALUOp::And32 => "lan",
                    ALUOp::And64 => "lang",
                    ALUOp::Orr32 => "lao",
                    ALUOp::Orr64 => "laog",
                    ALUOp::Xor32 => "lax",
                    ALUOp::Xor64 => "laxg",
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: false,
                        have_d20: true,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}, {}", mem_str, op, rd, rn, mem)
            }
            &Inst::AtomicCas32 {
                rd,
                ri,
                rn,
                ref mem,
            }
            | &Inst::AtomicCas64 {
                rd,
                ri,
                rn,
                ref mem,
            } => {
                let (opcode_rs, opcode_rsy) = match self {
                    &Inst::AtomicCas32 { .. } => (Some("cs"), Some("csy")),
                    &Inst::AtomicCas64 { .. } => (None, Some("csg")),
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rn = pretty_print_reg(rn, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: opcode_rs.is_some(),
                        have_d20: opcode_rsy.is_some(),
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => opcode_rs,
                    &MemArg::BXD20 { .. } => opcode_rsy,
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}, {}", mem_str, op.unwrap(), rd, rn, mem)
            }
            &Inst::Fence => "bcr 14, 0".to_string(),
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
            | &Inst::LoadRev64 { rd, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match self {
                    &Inst::Load32 { .. } => (Some("l"), Some("ly"), Some("lrl")),
                    &Inst::Load32ZExt8 { .. } => (None, Some("llc"), None),
                    &Inst::Load32SExt8 { .. } => (None, Some("lb"), None),
                    &Inst::Load32ZExt16 { .. } => (None, Some("llh"), Some("llhrl")),
                    &Inst::Load32SExt16 { .. } => (Some("lh"), Some("lhy"), Some("lhrl")),
                    &Inst::Load64 { .. } => (None, Some("lg"), Some("lgrl")),
                    &Inst::Load64ZExt8 { .. } => (None, Some("llgc"), None),
                    &Inst::Load64SExt8 { .. } => (None, Some("lgb"), None),
                    &Inst::Load64ZExt16 { .. } => (None, Some("llgh"), Some("llghrl")),
                    &Inst::Load64SExt16 { .. } => (None, Some("lgh"), Some("lghrl")),
                    &Inst::Load64ZExt32 { .. } => (None, Some("llgf"), Some("llgfrl")),
                    &Inst::Load64SExt32 { .. } => (None, Some("lgf"), Some("lgfrl")),
                    &Inst::LoadRev16 { .. } => (None, Some("lrvh"), None),
                    &Inst::LoadRev32 { .. } => (None, Some("lrv"), None),
                    &Inst::LoadRev64 { .. } => (None, Some("lrvg"), None),
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: opcode_rx.is_some(),
                        have_d20: opcode_rxy.is_some(),
                        have_pcrel: opcode_ril.is_some(),
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => opcode_rx,
                    &MemArg::BXD20 { .. } => opcode_rxy,
                    &MemArg::Label { .. } | &MemArg::Symbol { .. } => opcode_ril,
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}", mem_str, op.unwrap(), rd, mem)
            }
            &Inst::Store8 { rd, ref mem }
            | &Inst::Store16 { rd, ref mem }
            | &Inst::Store32 { rd, ref mem }
            | &Inst::Store64 { rd, ref mem }
            | &Inst::StoreRev16 { rd, ref mem }
            | &Inst::StoreRev32 { rd, ref mem }
            | &Inst::StoreRev64 { rd, ref mem } => {
                let (opcode_rx, opcode_rxy, opcode_ril) = match self {
                    &Inst::Store8 { .. } => (Some("stc"), Some("stcy"), None),
                    &Inst::Store16 { .. } => (Some("sth"), Some("sthy"), Some("sthrl")),
                    &Inst::Store32 { .. } => (Some("st"), Some("sty"), Some("strl")),
                    &Inst::Store64 { .. } => (None, Some("stg"), Some("stgrl")),
                    &Inst::StoreRev16 { .. } => (None, Some("strvh"), None),
                    &Inst::StoreRev32 { .. } => (None, Some("strv"), None),
                    &Inst::StoreRev64 { .. } => (None, Some("strvg"), None),
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: opcode_rx.is_some(),
                        have_d20: opcode_rxy.is_some(),
                        have_pcrel: opcode_ril.is_some(),
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => opcode_rx,
                    &MemArg::BXD20 { .. } => opcode_rxy,
                    &MemArg::Label { .. } | &MemArg::Symbol { .. } => opcode_ril,
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op.unwrap(), rd, mem)
            }
            &Inst::StoreImm8 { imm, ref mem } => {
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: true,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => "mvi",
                    &MemArg::BXD20 { .. } => "mviy",
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op, mem, imm)
            }
            &Inst::StoreImm16 { imm, ref mem }
            | &Inst::StoreImm32SExt16 { imm, ref mem }
            | &Inst::StoreImm64SExt16 { imm, ref mem } => {
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: false,
                        have_d20: true,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let op = match self {
                    &Inst::StoreImm16 { .. } => "mvhhi",
                    &Inst::StoreImm32SExt16 { .. } => "mvhi",
                    &Inst::StoreImm64SExt16 { .. } => "mvghi",
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op, mem, imm)
            }
            &Inst::Mvc {
                ref dst,
                ref src,
                len_minus_one,
            } => {
                let dst = dst.with_allocs(allocs);
                let src = src.with_allocs(allocs);
                format!(
                    "mvc {}({},{}), {}({})",
                    dst.disp.pretty_print_default(),
                    len_minus_one,
                    show_reg(dst.base),
                    src.disp.pretty_print_default(),
                    show_reg(src.base)
                )
            }
            &Inst::LoadMultiple64 { rt, rt2, ref mem } => {
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: false,
                        have_d20: true,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let rt = pretty_print_reg(rt.to_reg(), &mut empty_allocs);
                let rt2 = pretty_print_reg(rt2.to_reg(), &mut empty_allocs);
                let mem = mem.pretty_print_default();
                format!("{}lmg {}, {}, {}", mem_str, rt, rt2, mem)
            }
            &Inst::StoreMultiple64 { rt, rt2, ref mem } => {
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: false,
                        have_d20: true,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: false,
                    },
                );
                let rt = pretty_print_reg(rt, &mut empty_allocs);
                let rt2 = pretty_print_reg(rt2, &mut empty_allocs);
                let mem = mem.pretty_print_default();
                format!("{}stmg {}, {}, {}", mem_str, rt, rt2, mem)
            }
            &Inst::Mov64 { rd, rm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("lgr {}, {}", rd, rm)
            }
            &Inst::MovPReg { rd, rm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rm = show_reg(rm.into());
                format!("lgr {}, {}", rd, rm)
            }
            &Inst::Mov32 { rd, rm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("lr {}, {}", rd, rm)
            }
            &Inst::Mov32Imm { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("iilf {}, {}", rd, imm)
            }
            &Inst::Mov32SImm16 { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("lhi {}, {}", rd, imm)
            }
            &Inst::Mov64SImm16 { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("lghi {}, {}", rd, imm)
            }
            &Inst::Mov64SImm32 { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("lgfi {}, {}", rd, imm)
            }
            &Inst::Mov64UImm16Shifted { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let op = match imm.shift {
                    0 => "llill",
                    1 => "llilh",
                    2 => "llihl",
                    3 => "llihh",
                    _ => unreachable!(),
                };
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::Mov64UImm32Shifted { rd, ref imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let op = match imm.shift {
                    0 => "llilf",
                    1 => "llihf",
                    _ => unreachable!(),
                };
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::Insert64UImm16Shifted { rd, ri, ref imm } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let op = match imm.shift {
                    0 => "iill",
                    1 => "iilh",
                    2 => "iihl",
                    3 => "iihh",
                    _ => unreachable!(),
                };
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::Insert64UImm32Shifted { rd, ri, ref imm } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let op = match imm.shift {
                    0 => "iilf",
                    1 => "iihf",
                    _ => unreachable!(),
                };
                format!("{} {}, {}", op, rd, imm.bits)
            }
            &Inst::LoadAR { rd, ar } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("ear {}, %a{}", rd, ar)
            }
            &Inst::InsertAR { rd, ri, ar } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("ear {}, %a{}", rd, ar)
            }
            &Inst::CMov32 { rd, cond, ri, rm } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let cond = cond.pretty_print_default();
                format!("locr{} {}, {}", cond, rd, rm)
            }
            &Inst::CMov64 { rd, cond, ri, rm } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let cond = cond.pretty_print_default();
                format!("locgr{} {}, {}", cond, rd, rm)
            }
            &Inst::CMov32SImm16 {
                rd,
                cond,
                ri,
                ref imm,
            } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let cond = cond.pretty_print_default();
                format!("lochi{} {}, {}", cond, rd, imm)
            }
            &Inst::CMov64SImm16 {
                rd,
                cond,
                ri,
                ref imm,
            } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let cond = cond.pretty_print_default();
                format!("locghi{} {}, {}", cond, rd, imm)
            }
            &Inst::FpuMove32 { rd, rn } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                if rd_fpr.is_some() && rn_fpr.is_some() {
                    format!("ler {}, {}", rd_fpr.unwrap(), rn_fpr.unwrap())
                } else {
                    format!("vlr {}, {}", rd, rn)
                }
            }
            &Inst::FpuMove64 { rd, rn } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                if rd_fpr.is_some() && rn_fpr.is_some() {
                    format!("ldr {}, {}", rd_fpr.unwrap(), rn_fpr.unwrap())
                } else {
                    format!("vlr {}, {}", rd, rn)
                }
            }
            &Inst::FpuCMov32 { rd, cond, ri, rm } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let _ri = allocs.next(ri);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                if rd_fpr.is_some() && rm_fpr.is_some() {
                    let cond = cond.invert().pretty_print_default();
                    format!("j{} 6 ; ler {}, {}", cond, rd_fpr.unwrap(), rm_fpr.unwrap())
                } else {
                    let cond = cond.invert().pretty_print_default();
                    format!("j{} 10 ; vlr {}, {}", cond, rd, rm)
                }
            }
            &Inst::FpuCMov64 { rd, cond, ri, rm } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let _ri = allocs.next(ri);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                if rd_fpr.is_some() && rm_fpr.is_some() {
                    let cond = cond.invert().pretty_print_default();
                    format!("j{} 6 ; ldr {}, {}", cond, rd_fpr.unwrap(), rm_fpr.unwrap())
                } else {
                    let cond = cond.invert().pretty_print_default();
                    format!("j{} 10 ; vlr {}, {}", cond, rd, rm)
                }
            }
            &Inst::FpuRR { fpu_op, rd, rn } => {
                let (op, op_fpr) = match fpu_op {
                    FPUOp1::Abs32 => ("wflpsb", Some("lpebr")),
                    FPUOp1::Abs64 => ("wflpdb", Some("lpdbr")),
                    FPUOp1::Abs32x4 => ("vflpsb", None),
                    FPUOp1::Abs64x2 => ("vflpdb", None),
                    FPUOp1::Neg32 => ("wflcsb", Some("lcebr")),
                    FPUOp1::Neg64 => ("wflcdb", Some("lcdbr")),
                    FPUOp1::Neg32x4 => ("vflcsb", None),
                    FPUOp1::Neg64x2 => ("vflcdb", None),
                    FPUOp1::NegAbs32 => ("wflnsb", Some("lnebr")),
                    FPUOp1::NegAbs64 => ("wflndb", Some("lndbr")),
                    FPUOp1::NegAbs32x4 => ("vflnsb", None),
                    FPUOp1::NegAbs64x2 => ("vflndb", None),
                    FPUOp1::Sqrt32 => ("wfsqsb", Some("sqebr")),
                    FPUOp1::Sqrt64 => ("wfsqdb", Some("sqdbr")),
                    FPUOp1::Sqrt32x4 => ("vfsqsb", None),
                    FPUOp1::Sqrt64x2 => ("vfsqdb", None),
                    FPUOp1::Cvt32To64 => ("wldeb", Some("ldebr")),
                    FPUOp1::Cvt32x4To64x2 => ("vldeb", None),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                if op_fpr.is_some() && rd_fpr.is_some() && rn_fpr.is_some() {
                    format!(
                        "{} {}, {}",
                        op_fpr.unwrap(),
                        rd_fpr.unwrap(),
                        rn_fpr.unwrap()
                    )
                } else if op.starts_with('w') {
                    format!("{} {}, {}", op, rd_fpr.unwrap_or(rd), rn_fpr.unwrap_or(rn))
                } else {
                    format!("{} {}, {}", op, rd, rn)
                }
            }
            &Inst::FpuRRR { fpu_op, rd, rn, rm } => {
                let (op, opt_m6, op_fpr) = match fpu_op {
                    FPUOp2::Add32 => ("wfasb", "", Some("aebr")),
                    FPUOp2::Add64 => ("wfadb", "", Some("adbr")),
                    FPUOp2::Add32x4 => ("vfasb", "", None),
                    FPUOp2::Add64x2 => ("vfadb", "", None),
                    FPUOp2::Sub32 => ("wfssb", "", Some("sebr")),
                    FPUOp2::Sub64 => ("wfsdb", "", Some("sdbr")),
                    FPUOp2::Sub32x4 => ("vfssb", "", None),
                    FPUOp2::Sub64x2 => ("vfsdb", "", None),
                    FPUOp2::Mul32 => ("wfmsb", "", Some("meebr")),
                    FPUOp2::Mul64 => ("wfmdb", "", Some("mdbr")),
                    FPUOp2::Mul32x4 => ("vfmsb", "", None),
                    FPUOp2::Mul64x2 => ("vfmdb", "", None),
                    FPUOp2::Div32 => ("wfdsb", "", Some("debr")),
                    FPUOp2::Div64 => ("wfddb", "", Some("ddbr")),
                    FPUOp2::Div32x4 => ("vfdsb", "", None),
                    FPUOp2::Div64x2 => ("vfddb", "", None),
                    FPUOp2::Max32 => ("wfmaxsb", ", 1", None),
                    FPUOp2::Max64 => ("wfmaxdb", ", 1", None),
                    FPUOp2::Max32x4 => ("vfmaxsb", ", 1", None),
                    FPUOp2::Max64x2 => ("vfmaxdb", ", 1", None),
                    FPUOp2::Min32 => ("wfminsb", ", 1", None),
                    FPUOp2::Min64 => ("wfmindb", ", 1", None),
                    FPUOp2::Min32x4 => ("vfminsb", ", 1", None),
                    FPUOp2::Min64x2 => ("vfmindb", ", 1", None),
                    FPUOp2::MaxPseudo32 => ("wfmaxsb", ", 3", None),
                    FPUOp2::MaxPseudo64 => ("wfmaxdb", ", 3", None),
                    FPUOp2::MaxPseudo32x4 => ("vfmaxsb", ", 3", None),
                    FPUOp2::MaxPseudo64x2 => ("vfmaxdb", ", 3", None),
                    FPUOp2::MinPseudo32 => ("wfminsb", ", 3", None),
                    FPUOp2::MinPseudo64 => ("wfmindb", ", 3", None),
                    FPUOp2::MinPseudo32x4 => ("vfminsb", ", 3", None),
                    FPUOp2::MinPseudo64x2 => ("vfmindb", ", 3", None),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                if op_fpr.is_some() && rd == rn && rd_fpr.is_some() && rm_fpr.is_some() {
                    format!(
                        "{} {}, {}",
                        op_fpr.unwrap(),
                        rd_fpr.unwrap(),
                        rm_fpr.unwrap()
                    )
                } else if op.starts_with('w') {
                    format!(
                        "{} {}, {}, {}{}",
                        op,
                        rd_fpr.unwrap_or(rd),
                        rn_fpr.unwrap_or(rn),
                        rm_fpr.unwrap_or(rm),
                        opt_m6
                    )
                } else {
                    format!("{} {}, {}, {}{}", op, rd, rn, rm, opt_m6)
                }
            }
            &Inst::FpuRRRR {
                fpu_op,
                rd,
                rn,
                rm,
                ra,
            } => {
                let (op, op_fpr) = match fpu_op {
                    FPUOp3::MAdd32 => ("wfmasb", Some("maebr")),
                    FPUOp3::MAdd64 => ("wfmadb", Some("madbr")),
                    FPUOp3::MAdd32x4 => ("vfmasb", None),
                    FPUOp3::MAdd64x2 => ("vfmadb", None),
                    FPUOp3::MSub32 => ("wfmssb", Some("msebr")),
                    FPUOp3::MSub64 => ("wfmsdb", Some("msdbr")),
                    FPUOp3::MSub32x4 => ("vfmssb", None),
                    FPUOp3::MSub64x2 => ("vfmsdb", None),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                let (ra, ra_fpr) = pretty_print_fpr(ra, allocs);
                if op_fpr.is_some()
                    && rd == ra
                    && rd_fpr.is_some()
                    && rn_fpr.is_some()
                    && rm_fpr.is_some()
                {
                    format!(
                        "{} {}, {}, {}",
                        op_fpr.unwrap(),
                        rd_fpr.unwrap(),
                        rn_fpr.unwrap(),
                        rm_fpr.unwrap()
                    )
                } else if op.starts_with('w') {
                    format!(
                        "{} {}, {}, {}, {}",
                        op,
                        rd_fpr.unwrap_or(rd),
                        rn_fpr.unwrap_or(rn),
                        rm_fpr.unwrap_or(rm),
                        ra_fpr.unwrap_or(ra)
                    )
                } else {
                    format!("{} {}, {}, {}, {}", op, rd, rn, rm, ra)
                }
            }
            &Inst::FpuCmp32 { rn, rm } => {
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                if rn_fpr.is_some() && rm_fpr.is_some() {
                    format!("cebr {}, {}", rn_fpr.unwrap(), rm_fpr.unwrap())
                } else {
                    format!("wfcsb {}, {}", rn_fpr.unwrap_or(rn), rm_fpr.unwrap_or(rm))
                }
            }
            &Inst::FpuCmp64 { rn, rm } => {
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                let (rm, rm_fpr) = pretty_print_fpr(rm, allocs);
                if rn_fpr.is_some() && rm_fpr.is_some() {
                    format!("cdbr {}, {}", rn_fpr.unwrap(), rm_fpr.unwrap())
                } else {
                    format!("wfcdb {}, {}", rn_fpr.unwrap_or(rn), rm_fpr.unwrap_or(rm))
                }
            }
            &Inst::LoadFpuConst32 { rd, const_data } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let tmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                if rd_fpr.is_some() {
                    format!(
                        "bras {}, 8 ; data.f32 {} ; le {}, 0({})",
                        tmp,
                        f32::from_bits(const_data),
                        rd_fpr.unwrap(),
                        tmp
                    )
                } else {
                    format!(
                        "bras {}, 8 ; data.f32 {} ; vlef {}, 0({}), 0",
                        tmp,
                        f32::from_bits(const_data),
                        rd,
                        tmp
                    )
                }
            }
            &Inst::LoadFpuConst64 { rd, const_data } => {
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let tmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                if rd_fpr.is_some() {
                    format!(
                        "bras {}, 12 ; data.f64 {} ; ld {}, 0({})",
                        tmp,
                        f64::from_bits(const_data),
                        rd_fpr.unwrap(),
                        tmp
                    )
                } else {
                    format!(
                        "bras {}, 12 ; data.f64 {} ; vleg {}, 0({}), 0",
                        tmp,
                        f64::from_bits(const_data),
                        rd,
                        tmp
                    )
                }
            }
            &Inst::FpuRound { op, mode, rd, rn } => {
                let mode = match mode {
                    FpuRoundMode::Current => 0,
                    FpuRoundMode::ToNearest => 1,
                    FpuRoundMode::ShorterPrecision => 3,
                    FpuRoundMode::ToNearestTiesToEven => 4,
                    FpuRoundMode::ToZero => 5,
                    FpuRoundMode::ToPosInfinity => 6,
                    FpuRoundMode::ToNegInfinity => 7,
                };
                let (opcode, opcode_fpr) = match op {
                    FpuRoundOp::Cvt64To32 => ("wledb", Some("ledbra")),
                    FpuRoundOp::Cvt64x2To32x4 => ("vledb", None),
                    FpuRoundOp::Round32 => ("wfisb", Some("fiebr")),
                    FpuRoundOp::Round64 => ("wfidb", Some("fidbr")),
                    FpuRoundOp::Round32x4 => ("vfisb", None),
                    FpuRoundOp::Round64x2 => ("vfidb", None),
                    FpuRoundOp::ToSInt32 => ("wcfeb", None),
                    FpuRoundOp::ToSInt64 => ("wcgdb", None),
                    FpuRoundOp::ToUInt32 => ("wclfeb", None),
                    FpuRoundOp::ToUInt64 => ("wclgdb", None),
                    FpuRoundOp::ToSInt32x4 => ("vcfeb", None),
                    FpuRoundOp::ToSInt64x2 => ("vcgdb", None),
                    FpuRoundOp::ToUInt32x4 => ("vclfeb", None),
                    FpuRoundOp::ToUInt64x2 => ("vclgdb", None),
                    FpuRoundOp::FromSInt32 => ("wcefb", None),
                    FpuRoundOp::FromSInt64 => ("wcdgb", None),
                    FpuRoundOp::FromUInt32 => ("wcelfb", None),
                    FpuRoundOp::FromUInt64 => ("wcdlgb", None),
                    FpuRoundOp::FromSInt32x4 => ("vcefb", None),
                    FpuRoundOp::FromSInt64x2 => ("vcdgb", None),
                    FpuRoundOp::FromUInt32x4 => ("vcelfb", None),
                    FpuRoundOp::FromUInt64x2 => ("vcdlgb", None),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                if opcode_fpr.is_some() && rd_fpr.is_some() && rn_fpr.is_some() {
                    format!(
                        "{} {}, {}, {}",
                        opcode_fpr.unwrap(),
                        rd_fpr.unwrap(),
                        rn_fpr.unwrap(),
                        mode
                    )
                } else if opcode.starts_with('w') {
                    format!(
                        "{} {}, {}, 0, {}",
                        opcode,
                        rd_fpr.unwrap_or(rd),
                        rn_fpr.unwrap_or(rn),
                        mode
                    )
                } else {
                    format!("{} {}, {}, 0, {}", opcode, rd, rn, mode)
                }
            }
            &Inst::VecRRR { op, rd, rn, rm } => {
                let op = match op {
                    VecBinaryOp::Add8x16 => "vab",
                    VecBinaryOp::Add16x8 => "vah",
                    VecBinaryOp::Add32x4 => "vaf",
                    VecBinaryOp::Add64x2 => "vag",
                    VecBinaryOp::Add128 => "vaq",
                    VecBinaryOp::Sub8x16 => "vsb",
                    VecBinaryOp::Sub16x8 => "vsh",
                    VecBinaryOp::Sub32x4 => "vsf",
                    VecBinaryOp::Sub64x2 => "vsg",
                    VecBinaryOp::Sub128 => "vsq",
                    VecBinaryOp::Mul8x16 => "vmlb",
                    VecBinaryOp::Mul16x8 => "vmlhw",
                    VecBinaryOp::Mul32x4 => "vmlf",
                    VecBinaryOp::UMulHi8x16 => "vmlhb",
                    VecBinaryOp::UMulHi16x8 => "vmlhh",
                    VecBinaryOp::UMulHi32x4 => "vmlhf",
                    VecBinaryOp::SMulHi8x16 => "vmhb",
                    VecBinaryOp::SMulHi16x8 => "vmhh",
                    VecBinaryOp::SMulHi32x4 => "vmhf",
                    VecBinaryOp::UMulEven8x16 => "vmleb",
                    VecBinaryOp::UMulEven16x8 => "vmleh",
                    VecBinaryOp::UMulEven32x4 => "vmlef",
                    VecBinaryOp::SMulEven8x16 => "vmeb",
                    VecBinaryOp::SMulEven16x8 => "vmeh",
                    VecBinaryOp::SMulEven32x4 => "vmef",
                    VecBinaryOp::UMulOdd8x16 => "vmlob",
                    VecBinaryOp::UMulOdd16x8 => "vmloh",
                    VecBinaryOp::UMulOdd32x4 => "vmlof",
                    VecBinaryOp::SMulOdd8x16 => "vmob",
                    VecBinaryOp::SMulOdd16x8 => "vmoh",
                    VecBinaryOp::SMulOdd32x4 => "vmof",
                    VecBinaryOp::UMax8x16 => "vmxlb",
                    VecBinaryOp::UMax16x8 => "vmxlh",
                    VecBinaryOp::UMax32x4 => "vmxlf",
                    VecBinaryOp::UMax64x2 => "vmxlg",
                    VecBinaryOp::SMax8x16 => "vmxb",
                    VecBinaryOp::SMax16x8 => "vmxh",
                    VecBinaryOp::SMax32x4 => "vmxf",
                    VecBinaryOp::SMax64x2 => "vmxg",
                    VecBinaryOp::UMin8x16 => "vmnlb",
                    VecBinaryOp::UMin16x8 => "vmnlh",
                    VecBinaryOp::UMin32x4 => "vmnlf",
                    VecBinaryOp::UMin64x2 => "vmnlg",
                    VecBinaryOp::SMin8x16 => "vmnb",
                    VecBinaryOp::SMin16x8 => "vmnh",
                    VecBinaryOp::SMin32x4 => "vmnf",
                    VecBinaryOp::SMin64x2 => "vmng",
                    VecBinaryOp::UAvg8x16 => "vavglb",
                    VecBinaryOp::UAvg16x8 => "vavglh",
                    VecBinaryOp::UAvg32x4 => "vavglf",
                    VecBinaryOp::UAvg64x2 => "vavglg",
                    VecBinaryOp::SAvg8x16 => "vavgb",
                    VecBinaryOp::SAvg16x8 => "vavgh",
                    VecBinaryOp::SAvg32x4 => "vavgf",
                    VecBinaryOp::SAvg64x2 => "vavgg",
                    VecBinaryOp::And128 => "vn",
                    VecBinaryOp::Orr128 => "vo",
                    VecBinaryOp::Xor128 => "vx",
                    VecBinaryOp::NotAnd128 => "vnn",
                    VecBinaryOp::NotOrr128 => "vno",
                    VecBinaryOp::NotXor128 => "vnx",
                    VecBinaryOp::AndNot128 => "vnc",
                    VecBinaryOp::OrrNot128 => "voc",
                    VecBinaryOp::BitPermute128 => "vbperm",
                    VecBinaryOp::LShLByByte128 => "vslb",
                    VecBinaryOp::LShRByByte128 => "vsrlb",
                    VecBinaryOp::AShRByByte128 => "vsrab",
                    VecBinaryOp::LShLByBit128 => "vsl",
                    VecBinaryOp::LShRByBit128 => "vsrl",
                    VecBinaryOp::AShRByBit128 => "vsra",
                    VecBinaryOp::Pack16x8 => "vpkh",
                    VecBinaryOp::Pack32x4 => "vpkf",
                    VecBinaryOp::Pack64x2 => "vpkg",
                    VecBinaryOp::PackUSat16x8 => "vpklsh",
                    VecBinaryOp::PackUSat32x4 => "vpklsf",
                    VecBinaryOp::PackUSat64x2 => "vpklsg",
                    VecBinaryOp::PackSSat16x8 => "vpksh",
                    VecBinaryOp::PackSSat32x4 => "vpksf",
                    VecBinaryOp::PackSSat64x2 => "vpksg",
                    VecBinaryOp::MergeLow8x16 => "vmrlb",
                    VecBinaryOp::MergeLow16x8 => "vmrlh",
                    VecBinaryOp::MergeLow32x4 => "vmrlf",
                    VecBinaryOp::MergeLow64x2 => "vmrlg",
                    VecBinaryOp::MergeHigh8x16 => "vmrhb",
                    VecBinaryOp::MergeHigh16x8 => "vmrhh",
                    VecBinaryOp::MergeHigh32x4 => "vmrhf",
                    VecBinaryOp::MergeHigh64x2 => "vmrhg",
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("{} {}, {}, {}", op, rd, rn, rm)
            }
            &Inst::VecRR { op, rd, rn } => {
                let op = match op {
                    VecUnaryOp::Abs8x16 => "vlpb",
                    VecUnaryOp::Abs16x8 => "vlph",
                    VecUnaryOp::Abs32x4 => "vlpf",
                    VecUnaryOp::Abs64x2 => "vlpg",
                    VecUnaryOp::Neg8x16 => "vlcb",
                    VecUnaryOp::Neg16x8 => "vlch",
                    VecUnaryOp::Neg32x4 => "vlcf",
                    VecUnaryOp::Neg64x2 => "vlcg",
                    VecUnaryOp::Popcnt8x16 => "vpopctb",
                    VecUnaryOp::Popcnt16x8 => "vpopcth",
                    VecUnaryOp::Popcnt32x4 => "vpopctf",
                    VecUnaryOp::Popcnt64x2 => "vpopctg",
                    VecUnaryOp::Clz8x16 => "vclzb",
                    VecUnaryOp::Clz16x8 => "vclzh",
                    VecUnaryOp::Clz32x4 => "vclzf",
                    VecUnaryOp::Clz64x2 => "vclzg",
                    VecUnaryOp::Ctz8x16 => "vctzb",
                    VecUnaryOp::Ctz16x8 => "vctzh",
                    VecUnaryOp::Ctz32x4 => "vctzf",
                    VecUnaryOp::Ctz64x2 => "vctzg",
                    VecUnaryOp::UnpackULow8x16 => "vupllb",
                    VecUnaryOp::UnpackULow16x8 => "vupllh",
                    VecUnaryOp::UnpackULow32x4 => "vupllf",
                    VecUnaryOp::UnpackUHigh8x16 => "vuplhb",
                    VecUnaryOp::UnpackUHigh16x8 => "vuplhh",
                    VecUnaryOp::UnpackUHigh32x4 => "vuplhf",
                    VecUnaryOp::UnpackSLow8x16 => "vuplb",
                    VecUnaryOp::UnpackSLow16x8 => "vuplh",
                    VecUnaryOp::UnpackSLow32x4 => "vuplf",
                    VecUnaryOp::UnpackSHigh8x16 => "vuphb",
                    VecUnaryOp::UnpackSHigh16x8 => "vuphh",
                    VecUnaryOp::UnpackSHigh32x4 => "vuphf",
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::VecShiftRR {
                shift_op,
                rd,
                rn,
                shift_imm,
                shift_reg,
            } => {
                let op = match shift_op {
                    VecShiftOp::RotL8x16 => "verllb",
                    VecShiftOp::RotL16x8 => "verllh",
                    VecShiftOp::RotL32x4 => "verllf",
                    VecShiftOp::RotL64x2 => "verllg",
                    VecShiftOp::LShL8x16 => "veslb",
                    VecShiftOp::LShL16x8 => "veslh",
                    VecShiftOp::LShL32x4 => "veslf",
                    VecShiftOp::LShL64x2 => "veslg",
                    VecShiftOp::LShR8x16 => "vesrlb",
                    VecShiftOp::LShR16x8 => "vesrlh",
                    VecShiftOp::LShR32x4 => "vesrlf",
                    VecShiftOp::LShR64x2 => "vesrlg",
                    VecShiftOp::AShR8x16 => "vesrab",
                    VecShiftOp::AShR16x8 => "vesrah",
                    VecShiftOp::AShR32x4 => "vesraf",
                    VecShiftOp::AShR64x2 => "vesrag",
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let shift_reg = if shift_reg != zero_reg() {
                    format!("({})", pretty_print_reg(shift_reg, allocs))
                } else {
                    "".to_string()
                };
                format!("{} {}, {}, {}{}", op, rd, rn, shift_imm, shift_reg)
            }
            &Inst::VecSelect { rd, rn, rm, ra } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let ra = pretty_print_reg(ra, allocs);
                format!("vsel {}, {}, {}, {}", rd, rn, rm, ra)
            }
            &Inst::VecPermute { rd, rn, rm, ra } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let ra = pretty_print_reg(ra, allocs);
                format!("vperm {}, {}, {}, {}", rd, rn, rm, ra)
            }
            &Inst::VecPermuteDWImm {
                rd,
                rn,
                rm,
                idx1,
                idx2,
            } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let m4 = (idx1 & 1) * 4 + (idx2 & 1);
                format!("vpdi {}, {}, {}, {}", rd, rn, rm, m4)
            }
            &Inst::VecIntCmp { op, rd, rn, rm } | &Inst::VecIntCmpS { op, rd, rn, rm } => {
                let op = match op {
                    VecIntCmpOp::CmpEq8x16 => "vceqb",
                    VecIntCmpOp::CmpEq16x8 => "vceqh",
                    VecIntCmpOp::CmpEq32x4 => "vceqf",
                    VecIntCmpOp::CmpEq64x2 => "vceqg",
                    VecIntCmpOp::SCmpHi8x16 => "vchb",
                    VecIntCmpOp::SCmpHi16x8 => "vchh",
                    VecIntCmpOp::SCmpHi32x4 => "vchf",
                    VecIntCmpOp::SCmpHi64x2 => "vchg",
                    VecIntCmpOp::UCmpHi8x16 => "vchlb",
                    VecIntCmpOp::UCmpHi16x8 => "vchlh",
                    VecIntCmpOp::UCmpHi32x4 => "vchlf",
                    VecIntCmpOp::UCmpHi64x2 => "vchlg",
                };
                let s = match self {
                    &Inst::VecIntCmp { .. } => "",
                    &Inst::VecIntCmpS { .. } => "s",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("{}{} {}, {}, {}", op, s, rd, rn, rm)
            }
            &Inst::VecFloatCmp { op, rd, rn, rm } | &Inst::VecFloatCmpS { op, rd, rn, rm } => {
                let op = match op {
                    VecFloatCmpOp::CmpEq32x4 => "vfcesb",
                    VecFloatCmpOp::CmpEq64x2 => "vfcedb",
                    VecFloatCmpOp::CmpHi32x4 => "vfchsb",
                    VecFloatCmpOp::CmpHi64x2 => "vfchdb",
                    VecFloatCmpOp::CmpHiEq32x4 => "vfchesb",
                    VecFloatCmpOp::CmpHiEq64x2 => "vfchedb",
                };
                let s = match self {
                    &Inst::VecFloatCmp { .. } => "",
                    &Inst::VecFloatCmpS { .. } => "s",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("{}{} {}, {}, {}", op, s, rd, rn, rm)
            }
            &Inst::VecInt128SCmpHi { tmp, rn, rm } | &Inst::VecInt128UCmpHi { tmp, rn, rm } => {
                let op = match self {
                    &Inst::VecInt128SCmpHi { .. } => "vecg",
                    &Inst::VecInt128UCmpHi { .. } => "veclg",
                    _ => unreachable!(),
                };
                let tmp = pretty_print_reg(tmp.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!(
                    "{} {}, {} ; jne 10 ; vchlgs {}, {}, {}",
                    op, rm, rn, tmp, rn, rm
                )
            }
            &Inst::VecLoad { rd, ref mem }
            | &Inst::VecLoadRev { rd, ref mem }
            | &Inst::VecLoadByte16Rev { rd, ref mem }
            | &Inst::VecLoadByte32Rev { rd, ref mem }
            | &Inst::VecLoadByte64Rev { rd, ref mem }
            | &Inst::VecLoadElt16Rev { rd, ref mem }
            | &Inst::VecLoadElt32Rev { rd, ref mem }
            | &Inst::VecLoadElt64Rev { rd, ref mem } => {
                let opcode = match self {
                    &Inst::VecLoad { .. } => "vl",
                    &Inst::VecLoadRev { .. } => "vlbrq",
                    &Inst::VecLoadByte16Rev { .. } => "vlbrh",
                    &Inst::VecLoadByte32Rev { .. } => "vlbrf",
                    &Inst::VecLoadByte64Rev { .. } => "vlbrg",
                    &Inst::VecLoadElt16Rev { .. } => "vlerh",
                    &Inst::VecLoadElt32Rev { .. } => "vlerf",
                    &Inst::VecLoadElt64Rev { .. } => "vlerg",
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: false,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}", mem_str, opcode, rd, mem)
            }
            &Inst::VecStore { rd, ref mem }
            | &Inst::VecStoreRev { rd, ref mem }
            | &Inst::VecStoreByte16Rev { rd, ref mem }
            | &Inst::VecStoreByte32Rev { rd, ref mem }
            | &Inst::VecStoreByte64Rev { rd, ref mem }
            | &Inst::VecStoreElt16Rev { rd, ref mem }
            | &Inst::VecStoreElt32Rev { rd, ref mem }
            | &Inst::VecStoreElt64Rev { rd, ref mem } => {
                let opcode = match self {
                    &Inst::VecStore { .. } => "vst",
                    &Inst::VecStoreRev { .. } => "vstbrq",
                    &Inst::VecStoreByte16Rev { .. } => "vstbrh",
                    &Inst::VecStoreByte32Rev { .. } => "vstbrf",
                    &Inst::VecStoreByte64Rev { .. } => "vstbrg",
                    &Inst::VecStoreElt16Rev { .. } => "vsterh",
                    &Inst::VecStoreElt32Rev { .. } => "vsterf",
                    &Inst::VecStoreElt64Rev { .. } => "vsterg",
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd, allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: false,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}", mem_str, opcode, rd, mem)
            }
            &Inst::VecLoadReplicate { size, rd, ref mem }
            | &Inst::VecLoadReplicateRev { size, rd, ref mem } => {
                let opcode = match (self, size) {
                    (&Inst::VecLoadReplicate { .. }, 8) => "vlrepb",
                    (&Inst::VecLoadReplicate { .. }, 16) => "vlreph",
                    (&Inst::VecLoadReplicate { .. }, 32) => "vlrepf",
                    (&Inst::VecLoadReplicate { .. }, 64) => "vlrepg",
                    (&Inst::VecLoadReplicateRev { .. }, 16) => "vlbrreph",
                    (&Inst::VecLoadReplicateRev { .. }, 32) => "vlbrrepf",
                    (&Inst::VecLoadReplicateRev { .. }, 64) => "vlbrrepg",
                    _ => unreachable!(),
                };

                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: false,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}", mem_str, opcode, rd, mem)
            }
            &Inst::VecMov { rd, rn } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!("vlr {}, {}", rd, rn)
            }
            &Inst::VecCMov { rd, cond, ri, rm } => {
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rm = pretty_print_reg(rm, allocs);
                let cond = cond.invert().pretty_print_default();
                format!("j{} 10 ; vlr {}, {}", cond, rd, rm)
            }
            &Inst::MovToVec128 { rd, rn, rm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let rm = pretty_print_reg(rm, allocs);
                format!("vlvgp {}, {}, {}", rd, rn, rm)
            }
            &Inst::VecLoadConst { rd, const_data } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let tmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                format!(
                    "bras {}, 20 ; data.u128 0x{:032x} ; vl {}, 0({})",
                    tmp, const_data, rd, tmp
                )
            }
            &Inst::VecLoadConstReplicate {
                size,
                rd,
                const_data,
            } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let tmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                let (opcode, data) = match size {
                    32 => ("vlrepf", format!("0x{:08x}", const_data as u32)),
                    64 => ("vlrepg", format!("0x{:016x}", const_data)),
                    _ => unreachable!(),
                };
                format!(
                    "bras {}, {} ; data.u{} {} ; {} {}, 0({})",
                    tmp,
                    4 + size / 8,
                    size,
                    data,
                    opcode,
                    rd,
                    tmp
                )
            }
            &Inst::VecImmByteMask { rd, mask } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                format!("vgbm {}, {}", rd, mask)
            }
            &Inst::VecImmBitMask {
                size,
                rd,
                start_bit,
                end_bit,
            } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let op = match size {
                    8 => "vgmb",
                    16 => "vgmh",
                    32 => "vgmf",
                    64 => "vgmg",
                    _ => unreachable!(),
                };
                format!("{} {}, {}, {}", op, rd, start_bit, end_bit)
            }
            &Inst::VecImmReplicate { size, rd, imm } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let op = match size {
                    8 => "vrepib",
                    16 => "vrepih",
                    32 => "vrepif",
                    64 => "vrepig",
                    _ => unreachable!(),
                };
                format!("{} {}, {}", op, rd, imm)
            }
            &Inst::VecLoadLane {
                size,
                rd,
                ri,
                ref mem,
                lane_imm,
            }
            | &Inst::VecLoadLaneRev {
                size,
                rd,
                ri,
                ref mem,
                lane_imm,
            } => {
                let opcode_vrx = match (self, size) {
                    (&Inst::VecLoadLane { .. }, 8) => "vleb",
                    (&Inst::VecLoadLane { .. }, 16) => "vleh",
                    (&Inst::VecLoadLane { .. }, 32) => "vlef",
                    (&Inst::VecLoadLane { .. }, 64) => "vleg",
                    (&Inst::VecLoadLaneRev { .. }, 16) => "vlebrh",
                    (&Inst::VecLoadLaneRev { .. }, 32) => "vlebrf",
                    (&Inst::VecLoadLaneRev { .. }, 64) => "vlebrg",
                    _ => unreachable!(),
                };

                let (rd, _) = pretty_print_fpr(rd.to_reg(), allocs);
                let _ri = allocs.next(ri);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: false,
                        have_pcrel: false,
                        have_unaligned_pcrel: false,
                        have_index: true,
                    },
                );
                let mem = mem.pretty_print_default();
                format!("{}{} {}, {}, {}", mem_str, opcode_vrx, rd, mem, lane_imm)
            }
            &Inst::VecLoadLaneUndef {
                size,
                rd,
                ref mem,
                lane_imm,
            }
            | &Inst::VecLoadLaneRevUndef {
                size,
                rd,
                ref mem,
                lane_imm,
            } => {
                let (opcode_vrx, opcode_rx, opcode_rxy) = match (self, size) {
                    (&Inst::VecLoadLaneUndef { .. }, 8) => ("vleb", None, None),
                    (&Inst::VecLoadLaneUndef { .. }, 16) => ("vleh", None, None),
                    (&Inst::VecLoadLaneUndef { .. }, 32) => ("vlef", Some("le"), Some("ley")),
                    (&Inst::VecLoadLaneUndef { .. }, 64) => ("vleg", Some("ld"), Some("ldy")),
                    (&Inst::VecLoadLaneRevUndef { .. }, 16) => ("vlebrh", None, None),
                    (&Inst::VecLoadLaneRevUndef { .. }, 32) => ("vlebrf", None, None),
                    (&Inst::VecLoadLaneRevUndef { .. }, 64) => ("vlebrg", None, None),
                    _ => unreachable!(),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let mem = mem.with_allocs(allocs);
                if lane_imm == 0 && rd_fpr.is_some() && opcode_rx.is_some() {
                    let (mem_str, mem) = mem_finalize_for_show(
                        &mem,
                        state,
                        MemInstType {
                            have_d12: true,
                            have_d20: true,
                            have_pcrel: false,
                            have_unaligned_pcrel: false,
                            have_index: true,
                        },
                    );
                    let op = match &mem {
                        &MemArg::BXD12 { .. } => opcode_rx,
                        &MemArg::BXD20 { .. } => opcode_rxy,
                        _ => unreachable!(),
                    };
                    let mem = mem.pretty_print_default();
                    format!("{}{} {}, {}", mem_str, op.unwrap(), rd_fpr.unwrap(), mem)
                } else {
                    let (mem_str, mem) = mem_finalize_for_show(
                        &mem,
                        state,
                        MemInstType {
                            have_d12: true,
                            have_d20: false,
                            have_pcrel: false,
                            have_unaligned_pcrel: false,
                            have_index: true,
                        },
                    );
                    let mem = mem.pretty_print_default();
                    format!("{}{} {}, {}, {}", mem_str, opcode_vrx, rd, mem, lane_imm)
                }
            }
            &Inst::VecStoreLane {
                size,
                rd,
                ref mem,
                lane_imm,
            }
            | &Inst::VecStoreLaneRev {
                size,
                rd,
                ref mem,
                lane_imm,
            } => {
                let (opcode_vrx, opcode_rx, opcode_rxy) = match (self, size) {
                    (&Inst::VecStoreLane { .. }, 8) => ("vsteb", None, None),
                    (&Inst::VecStoreLane { .. }, 16) => ("vsteh", None, None),
                    (&Inst::VecStoreLane { .. }, 32) => ("vstef", Some("ste"), Some("stey")),
                    (&Inst::VecStoreLane { .. }, 64) => ("vsteg", Some("std"), Some("stdy")),
                    (&Inst::VecStoreLaneRev { .. }, 16) => ("vstebrh", None, None),
                    (&Inst::VecStoreLaneRev { .. }, 32) => ("vstebrf", None, None),
                    (&Inst::VecStoreLaneRev { .. }, 64) => ("vstebrg", None, None),
                    _ => unreachable!(),
                };

                let (rd, rd_fpr) = pretty_print_fpr(rd, allocs);
                let mem = mem.with_allocs(allocs);
                if lane_imm == 0 && rd_fpr.is_some() && opcode_rx.is_some() {
                    let (mem_str, mem) = mem_finalize_for_show(
                        &mem,
                        state,
                        MemInstType {
                            have_d12: true,
                            have_d20: true,
                            have_pcrel: false,
                            have_unaligned_pcrel: false,
                            have_index: true,
                        },
                    );
                    let op = match &mem {
                        &MemArg::BXD12 { .. } => opcode_rx,
                        &MemArg::BXD20 { .. } => opcode_rxy,
                        _ => unreachable!(),
                    };
                    let mem = mem.pretty_print_default();
                    format!("{}{} {}, {}", mem_str, op.unwrap(), rd_fpr.unwrap(), mem)
                } else {
                    let (mem_str, mem) = mem_finalize_for_show(
                        &mem,
                        state,
                        MemInstType {
                            have_d12: true,
                            have_d20: false,
                            have_pcrel: false,
                            have_unaligned_pcrel: false,
                            have_index: true,
                        },
                    );
                    let mem = mem.pretty_print_default();
                    format!("{}{} {}, {}, {}", mem_str, opcode_vrx, rd, mem, lane_imm,)
                }
            }
            &Inst::VecInsertLane {
                size,
                rd,
                ri,
                rn,
                lane_imm,
                lane_reg,
            } => {
                let op = match size {
                    8 => "vlvgb",
                    16 => "vlvgh",
                    32 => "vlvgf",
                    64 => "vlvgg",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                let rn = pretty_print_reg(rn, allocs);
                let lane_reg = if lane_reg != zero_reg() {
                    format!("({})", pretty_print_reg(lane_reg, allocs))
                } else {
                    "".to_string()
                };
                format!("{} {}, {}, {}{}", op, rd, rn, lane_imm, lane_reg)
            }
            &Inst::VecInsertLaneUndef {
                size,
                rd,
                rn,
                lane_imm,
                lane_reg,
            } => {
                let (opcode_vrs, opcode_rre) = match size {
                    8 => ("vlvgb", None),
                    16 => ("vlvgh", None),
                    32 => ("vlvgf", None),
                    64 => ("vlvgg", Some("ldgr")),
                    _ => unreachable!(),
                };
                let (rd, rd_fpr) = pretty_print_fpr(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let lane_reg = if lane_reg != zero_reg() {
                    format!("({})", pretty_print_reg(lane_reg, allocs))
                } else {
                    "".to_string()
                };
                if opcode_rre.is_some() && lane_imm == 0 && lane_reg.is_empty() && rd_fpr.is_some()
                {
                    format!("{} {}, {}", opcode_rre.unwrap(), rd_fpr.unwrap(), rn)
                } else {
                    format!("{} {}, {}, {}{}", opcode_vrs, rd, rn, lane_imm, lane_reg)
                }
            }
            &Inst::VecExtractLane {
                size,
                rd,
                rn,
                lane_imm,
                lane_reg,
            } => {
                let (opcode_vrs, opcode_rre) = match size {
                    8 => ("vlgvb", None),
                    16 => ("vlgvh", None),
                    32 => ("vlgvf", None),
                    64 => ("vlgvg", Some("lgdr")),
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let (rn, rn_fpr) = pretty_print_fpr(rn, allocs);
                let lane_reg = if lane_reg != zero_reg() {
                    format!("({})", pretty_print_reg(lane_reg, allocs))
                } else {
                    "".to_string()
                };
                if opcode_rre.is_some() && lane_imm == 0 && lane_reg.is_empty() && rn_fpr.is_some()
                {
                    format!("{} {}, {}", opcode_rre.unwrap(), rd, rn_fpr.unwrap())
                } else {
                    format!("{} {}, {}, {}{}", opcode_vrs, rd, rn, lane_imm, lane_reg)
                }
            }
            &Inst::VecInsertLaneImm {
                size,
                rd,
                ri,
                imm,
                lane_imm,
            } => {
                let op = match size {
                    8 => "vleib",
                    16 => "vleih",
                    32 => "vleif",
                    64 => "vleig",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg_mod(rd, ri, allocs);
                format!("{} {}, {}, {}", op, rd, imm, lane_imm)
            }
            &Inst::VecReplicateLane {
                size,
                rd,
                rn,
                lane_imm,
            } => {
                let op = match size {
                    8 => "vrepb",
                    16 => "vreph",
                    32 => "vrepf",
                    64 => "vrepg",
                    _ => unreachable!(),
                };
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                format!("{} {}, {}, {}", op, rd, rn, lane_imm)
            }
            &Inst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits,
            } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let rn = pretty_print_reg(rn, allocs);
                let op = match (signed, from_bits, to_bits) {
                    (_, 1, 32) => "llcr",
                    (_, 1, 64) => "llgcr",
                    (false, 8, 32) => "llcr",
                    (false, 8, 64) => "llgcr",
                    (true, 8, 32) => "lbr",
                    (true, 8, 64) => "lgbr",
                    (false, 16, 32) => "llhr",
                    (false, 16, 64) => "llghr",
                    (true, 16, 32) => "lhr",
                    (true, 16, 64) => "lghr",
                    (false, 32, 64) => "llgfr",
                    (true, 32, 64) => "lgfr",
                    _ => panic!("Unsupported Extend case: {:?}", self),
                };
                format!("{} {}, {}", op, rd, rn)
            }
            &Inst::Call { link, ref info, .. } => {
                let link = link.to_reg();
                let tls_symbol = match &info.tls_symbol {
                    None => "".to_string(),
                    Some(SymbolReloc::TlsGd { name }) => {
                        format!(":tls_gdcall:{}", name.display(None))
                    }
                    _ => unreachable!(),
                };
                debug_assert_eq!(link, gpr(14));
                format!(
                    "brasl {}, {}{}",
                    show_reg(link),
                    info.dest.display(None),
                    tls_symbol
                )
            }
            &Inst::CallInd { link, ref info, .. } => {
                let link = link.to_reg();
                let rn = pretty_print_reg(info.rn, allocs);
                debug_assert_eq!(link, gpr(14));
                format!("basr {}, {}", show_reg(link), rn)
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
            &Inst::Ret { link, .. } => {
                debug_assert_eq!(link, gpr(14));
                format!("br {}", show_reg(link))
            }
            &Inst::Jump { dest } => {
                let dest = dest.to_string();
                format!("jg {}", dest)
            }
            &Inst::IndirectBr { rn, .. } => {
                let rn = pretty_print_reg(rn, allocs);
                format!("br {}", rn)
            }
            &Inst::CondBr {
                taken,
                not_taken,
                cond,
            } => {
                let taken = taken.to_string();
                let not_taken = not_taken.to_string();
                let cond = cond.pretty_print_default();
                format!("jg{} {} ; jg {}", cond, taken, not_taken)
            }
            &Inst::OneWayCondBr { target, cond } => {
                let target = target.to_string();
                let cond = cond.pretty_print_default();
                format!("jg{} {}", cond, target)
            }
            &Inst::Debugtrap => "debugtrap".to_string(),
            &Inst::Trap { .. } => "trap".to_string(),
            &Inst::TrapIf { cond, .. } => {
                let cond = cond.invert().pretty_print_default();
                format!("j{} 6 ; trap", cond)
            }
            &Inst::JTSequence { ridx, ref targets } => {
                let ridx = pretty_print_reg(ridx, allocs);
                let rtmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                // The first entry is the default target, which is not emitted
                // into the jump table, so we skip it here.  It is only in the
                // list so MachTerminator will see the potential target.
                let jt_entries: String = targets
                    .iter()
                    .skip(1)
                    .map(|label| format!(" {}", label.to_string()))
                    .collect();
                format!(
                    concat!(
                        "larl {}, 14 ; ",
                        "agf {}, 0({}, {}) ; ",
                        "br {} ; ",
                        "jt_entries{}"
                    ),
                    rtmp, rtmp, rtmp, ridx, rtmp, jt_entries,
                )
            }
            &Inst::LoadSymbolReloc {
                rd,
                ref symbol_reloc,
            } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let tmp = pretty_print_reg(writable_spilltmp_reg().to_reg(), &mut empty_allocs);
                let symbol = match &**symbol_reloc {
                    SymbolReloc::Absolute { name, offset } => {
                        format!("{} + {}", name.display(None), offset)
                    }
                    SymbolReloc::TlsGd { name } => format!("{}@tlsgd", name.display(None)),
                };
                format!("bras {}, 12 ; data {} ; lg {}, 0({})", tmp, symbol, rd, tmp)
            }
            &Inst::LoadAddr { rd, ref mem } => {
                let rd = pretty_print_reg(rd.to_reg(), allocs);
                let mem = mem.with_allocs(allocs);
                let (mem_str, mem) = mem_finalize_for_show(
                    &mem,
                    state,
                    MemInstType {
                        have_d12: true,
                        have_d20: true,
                        have_pcrel: true,
                        have_unaligned_pcrel: true,
                        have_index: true,
                    },
                );
                let op = match &mem {
                    &MemArg::BXD12 { .. } => "la",
                    &MemArg::BXD20 { .. } => "lay",
                    &MemArg::Label { .. } | &MemArg::Symbol { .. } => "larl",
                    _ => unreachable!(),
                };
                let mem = mem.pretty_print_default();

                format!("{}{} {}, {}", mem_str, op, rd, mem)
            }
            &Inst::Loop { ref body, cond } => {
                let body = body
                    .into_iter()
                    .map(|inst| inst.print_with_state(state, allocs))
                    .collect::<Vec<_>>()
                    .join(" ; ");
                let cond = cond.pretty_print_default();
                format!("0: {} ; jg{} 0b ; 1:", body, cond)
            }
            &Inst::CondBreak { cond } => {
                let cond = cond.pretty_print_default();
                format!("jg{} 1f", cond)
            }
            &Inst::VirtualSPOffsetAdj { offset } => {
                state.virtual_sp_offset += offset;
                format!("virtual_sp_offset_adjust {}", offset)
            }
            &Inst::Unwind { ref inst } => {
                format!("unwind {:?}", inst)
            }
            &Inst::DummyUse { reg } => {
                let reg = pretty_print_reg(reg, allocs);
                format!("dummy_use {}", reg)
            }
        }
    }
}

//=============================================================================
// Label fixups and jump veneers.

/// Different forms of label references for different instruction formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUse {
    #[allow(dead_code)]
    /// RI-format branch.  16-bit signed offset.  PC-relative, offset is imm << 1.
    BranchRI,
    /// RIL-format branch.  32-bit signed offset.  PC-relative, offset is imm << 1.
    BranchRIL,
    /// 32-bit PC relative constant offset (from address of constant itself),
    /// signed. Used in jump tables.
    PCRel32,
    /// 32-bit PC relative constant offset (from address of call instruction),
    /// signed. Offset is imm << 1.  Used for call relocations.
    PCRel32Dbl,
}

impl MachInstLabelUse for LabelUse {
    /// Alignment for veneer code.
    const ALIGN: CodeOffset = 2;

    /// Maximum PC-relative range (positive), inclusive.
    fn max_pos_range(self) -> CodeOffset {
        match self {
            // 16-bit signed immediate, left-shifted by 1.
            LabelUse::BranchRI => ((1 << 15) - 1) << 1,
            // 32-bit signed immediate, left-shifted by 1.
            LabelUse::BranchRIL => 0xffff_fffe,
            // 32-bit signed immediate.
            LabelUse::PCRel32 => 0x7fff_ffff,
            // 32-bit signed immediate, left-shifted by 1, offset by 2.
            LabelUse::PCRel32Dbl => 0xffff_fffc,
        }
    }

    /// Maximum PC-relative range (negative).
    fn max_neg_range(self) -> CodeOffset {
        match self {
            // 16-bit signed immediate, left-shifted by 1.
            LabelUse::BranchRI => (1 << 15) << 1,
            // 32-bit signed immediate, left-shifted by 1.
            // NOTE: This should be 4GB, but CodeOffset is only u32.
            LabelUse::BranchRIL => 0xffff_ffff,
            // 32-bit signed immediate.
            LabelUse::PCRel32 => 0x8000_0000,
            // 32-bit signed immediate, left-shifted by 1, offset by 2.
            // NOTE: This should be 4GB + 2, but CodeOffset is only u32.
            LabelUse::PCRel32Dbl => 0xffff_ffff,
        }
    }

    /// Size of window into code needed to do the patch.
    fn patch_size(self) -> CodeOffset {
        match self {
            LabelUse::BranchRI => 4,
            LabelUse::BranchRIL => 6,
            LabelUse::PCRel32 => 4,
            LabelUse::PCRel32Dbl => 4,
        }
    }

    /// Perform the patch.
    fn patch(self, buffer: &mut [u8], use_offset: CodeOffset, label_offset: CodeOffset) {
        let pc_rel = (label_offset as i64) - (use_offset as i64);
        debug_assert!(pc_rel <= self.max_pos_range() as i64);
        debug_assert!(pc_rel >= -(self.max_neg_range() as i64));
        debug_assert!(pc_rel & 1 == 0);
        let pc_rel_shifted = pc_rel >> 1;

        match self {
            LabelUse::BranchRI => {
                buffer[2..4].clone_from_slice(&u16::to_be_bytes(pc_rel_shifted as u16));
            }
            LabelUse::BranchRIL => {
                buffer[2..6].clone_from_slice(&u32::to_be_bytes(pc_rel_shifted as u32));
            }
            LabelUse::PCRel32 => {
                let insn_word = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                let insn_word = insn_word.wrapping_add(pc_rel as u32);
                buffer[0..4].clone_from_slice(&u32::to_be_bytes(insn_word));
            }
            LabelUse::PCRel32Dbl => {
                let insn_word = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                let insn_word = insn_word.wrapping_add((pc_rel_shifted + 1) as u32);
                buffer[0..4].clone_from_slice(&u32::to_be_bytes(insn_word));
            }
        }
    }

    /// Is a veneer supported for this label reference type?
    fn supports_veneer(self) -> bool {
        false
    }

    /// How large is the veneer, if supported?
    fn veneer_size(self) -> CodeOffset {
        0
    }

    /// Generate a veneer into the buffer, given that this veneer is at `veneer_offset`, and return
    /// an offset and label-use for the veneer's use of the original label.
    fn generate_veneer(
        self,
        _buffer: &mut [u8],
        _veneer_offset: CodeOffset,
    ) -> (CodeOffset, LabelUse) {
        unreachable!();
    }

    fn from_reloc(reloc: Reloc, addend: Addend) -> Option<Self> {
        match (reloc, addend) {
            (Reloc::S390xPCRel32Dbl, 2) => Some(LabelUse::PCRel32Dbl),
            (Reloc::S390xPLTRel32Dbl, 2) => Some(LabelUse::PCRel32Dbl),
            _ => None,
        }
    }
}
