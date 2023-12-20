//! This module defines riscv64-specific machine instruction types.

// Some variants are not constructed, but we still want them as options in the future.
#![allow(dead_code)]
#![allow(non_camel_case_types)]

use crate::binemit::{Addend, CodeOffset, Reloc};
pub use crate::ir::condcodes::IntCC;
use crate::ir::types::{F32, F64, FFLAGS, I128, I16, I32, I64, I8, IFLAGS, R32, R64};

pub use crate::ir::{ExternalName, MemFlags, Opcode, SourceLoc, Type, ValueLabel};
use crate::isa::CallConv;
use crate::machinst::isle::WritableReg;
use crate::machinst::*;
use crate::{settings, CodegenError, CodegenResult};

pub use crate::ir::condcodes::FloatCC;

use alloc::vec::Vec;
use regalloc2::{PRegSet, VReg};
use smallvec::SmallVec;
use std::boxed::Box;
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

use crate::isa::riscv64::abi::Riscv64MachineDeps;

#[cfg(test)]
mod emit_tests;

use std::fmt::{Display, Formatter};

pub(crate) type OptionReg = Option<Reg>;
pub(crate) type OptionImm12 = Option<Imm12>;
pub(crate) type VecBranchTarget = Vec<BranchTarget>;
pub(crate) type OptionUimm5 = Option<Uimm5>;
pub(crate) type OptionFloatRoundingMode = Option<FRM>;
pub(crate) type VecU8 = Vec<u8>;
pub(crate) type VecWritableReg = Vec<Writable<Reg>>;
//=============================================================================
// Instructions (top level): definition

use crate::isa::riscv64::lower::isle::generated_code::MInst;
pub use crate::isa::riscv64::lower::isle::generated_code::{
    AluOPRRI, AluOPRRR, AtomicOP, CsrOP, FClassResult, FFlagsException, FenceFm, FloatRoundOP,
    FloatSelectOP, FpuOPRR, FpuOPRRR, FpuOPRRRR, IntSelectOP, LoadOP, MInst as Inst,
    ReferenceCheckOP, StoreOP, FRM,
};

type BoxCallInfo = Box<CallInfo>;
type BoxCallIndInfo = Box<CallIndInfo>;

/// Additional information for (direct) Call instructions, left out of line to lower the size of
/// the Inst enum.
#[derive(Clone, Debug)]
pub struct CallInfo {
    pub dest: ExternalName,
    pub uses: CallArgList,
    pub defs: CallRetList,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
    pub clobbers: PRegSet,
}

/// Additional information for CallInd instructions, left out of line to lower the size of the Inst
/// enum.
#[derive(Clone, Debug)]
pub struct CallIndInfo {
    pub rn: Reg,
    pub uses: CallArgList,
    pub defs: CallRetList,
    pub opcode: Opcode,
    pub caller_callconv: CallConv,
    pub callee_callconv: CallConv,
    pub clobbers: PRegSet,
}

/// A branch target. Either unresolved (basic-block index) or resolved (offset
/// from end of current instruction).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BranchTarget {
    /// An unresolved reference to a Label, as passed into
    /// `lower_branch_group()`.
    Label(MachLabel),
    /// A fixed PC offset.
    ResolvedOffset(i32),
}

impl BranchTarget {
    /// Return the target's label, if it is a label-based target.
    pub(crate) fn as_label(self) -> Option<MachLabel> {
        match self {
            BranchTarget::Label(l) => Some(l),
            _ => None,
        }
    }
    /// offset zero.
    #[inline]
    pub(crate) fn zero() -> Self {
        Self::ResolvedOffset(0)
    }
    #[inline]
    pub(crate) fn offset(off: i32) -> Self {
        Self::ResolvedOffset(off)
    }
    #[inline]
    pub(crate) fn is_zero(self) -> bool {
        match self {
            BranchTarget::Label(_) => false,
            BranchTarget::ResolvedOffset(off) => off == 0,
        }
    }
    #[inline]
    pub(crate) fn as_offset(self) -> Option<i32> {
        match self {
            BranchTarget::Label(_) => None,
            BranchTarget::ResolvedOffset(off) => Some(off),
        }
    }
}

impl Display for BranchTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchTarget::Label(l) => write!(f, "{}", l.to_string()),
            BranchTarget::ResolvedOffset(off) => write!(f, "{}", off),
        }
    }
}

pub(crate) fn enc_auipc(rd: Writable<Reg>, imm: Imm20) -> u32 {
    let x = 0b0010111 | reg_to_gpr_num(rd.to_reg()) << 7 | imm.as_u32() << 12;
    x
}

pub(crate) fn enc_jalr(rd: Writable<Reg>, base: Reg, offset: Imm12) -> u32 {
    let x = 0b1100111
        | reg_to_gpr_num(rd.to_reg()) << 7
        | 0b000 << 12
        | reg_to_gpr_num(base) << 15
        | offset.as_u32() << 20;
    x
}

/// rd and src must have the same length.
pub(crate) fn gen_moves(rd: &[Writable<Reg>], src: &[Reg]) -> SmallInstVec<Inst> {
    assert!(rd.len() == src.len());
    assert!(rd.len() > 0);
    let mut insts = SmallInstVec::new();
    for (dst, src) in rd.iter().zip(src.iter()) {
        let out_ty = Inst::canonical_type_for_rc(dst.to_reg().class());
        let in_ty = Inst::canonical_type_for_rc(src.class());
        insts.push(gen_move(*dst, out_ty, *src, in_ty));
    }
    insts
}

/// if input or output is float,
/// you should use special instruction.
/// generate a move and re-interpret the data.
pub(crate) fn gen_move(rd: Writable<Reg>, oty: Type, rm: Reg, ity: Type) -> Inst {
    match (ity.is_float(), oty.is_float()) {
        (false, false) => Inst::gen_move(rd, rm, oty),
        (true, true) => Inst::gen_move(rd, rm, oty),
        (false, true) => Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::move_x_to_f_op(oty),
            rd: rd,
            rs: rm,
        },
        (true, false) => Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::move_f_to_x_op(ity),
            rd: rd,
            rs: rm,
        },
    }
}

impl Inst {
    const INSTRUCTION_SIZE: i32 = 4;

    #[inline]
    pub(crate) fn load_imm12(rd: Writable<Reg>, imm: Imm12) -> Inst {
        Inst::AluRRImm12 {
            alu_op: AluOPRRI::Addi,
            rd: rd,
            rs: zero_reg(),
            imm12: imm,
        }
    }

    /// Immediates can be loaded using lui and addi instructions.
    fn load_const_imm(rd: Writable<Reg>, value: u64) -> Option<SmallInstVec<Inst>> {
        Inst::generate_imm(value, |imm20, imm12| {
            let mut insts = SmallVec::new();
            imm20.map(|x| insts.push(Inst::Lui { rd, imm: x }));
            imm12.map(|x| {
                let imm20_is_none = imm20.is_none();
                let rs = if imm20_is_none {
                    zero_reg()
                } else {
                    rd.to_reg()
                };
                insts.push(Inst::AluRRImm12 {
                    alu_op: AluOPRRI::Addi,
                    rd,
                    rs,
                    imm12: x,
                })
            });

            insts
        })
    }

    pub(crate) fn load_constant_u32(rd: Writable<Reg>, value: u64) -> SmallInstVec<Inst> {
        let insts = Inst::load_const_imm(rd, value);
        insts.unwrap_or(LoadConstant::U32(value as u32).load_constant(rd))
    }

    pub fn load_constant_u64(rd: Writable<Reg>, value: u64) -> SmallInstVec<Inst> {
        let insts = Inst::load_const_imm(rd, value);
        insts.unwrap_or(LoadConstant::U64(value).load_constant(rd))
    }

    pub(crate) fn construct_auipc_and_jalr(
        link: Option<Writable<Reg>>,
        tmp: Writable<Reg>,
        offset: i64,
    ) -> [Inst; 2] {
        Inst::generate_imm(offset as u64, |imm20, imm12| {
            let a = Inst::Auipc {
                rd: tmp,
                imm: imm20.unwrap_or_default(),
            };
            let b = Inst::Jalr {
                rd: link.unwrap_or(writable_zero_reg()),
                base: tmp.to_reg(),
                offset: imm12.unwrap_or_default(),
            };
            [a, b]
        })
        .expect("code range is too big.")
    }

    /// Create instructions that load a 32-bit floating-point constant.
    pub fn load_fp_constant32(
        rd: Writable<Reg>,
        const_data: u32,
        tmp: Writable<Reg>,
    ) -> SmallVec<[Inst; 4]> {
        let mut insts = SmallVec::new();
        insts.extend(Self::load_constant_u32(tmp, const_data as u64));
        insts.push(Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::move_x_to_f_op(F32),
            rd,
            rs: tmp.to_reg(),
        });
        insts
    }

    /// Create instructions that load a 64-bit floating-point constant.
    pub fn load_fp_constant64(
        rd: Writable<Reg>,
        const_data: u64,
        tmp: WritableReg,
    ) -> SmallVec<[Inst; 4]> {
        let mut insts = SmallInstVec::new();
        insts.extend(Self::load_constant_u64(tmp, const_data));
        insts.push(Inst::FpuRR {
            frm: None,
            alu_op: FpuOPRR::move_x_to_f_op(F64),
            rd,
            rs: tmp.to_reg(),
        });
        insts
    }

    /// Generic constructor for a load (zero-extending where appropriate).
    pub fn gen_load(into_reg: Writable<Reg>, mem: AMode, ty: Type, flags: MemFlags) -> Inst {
        Inst::Load {
            rd: into_reg,
            op: LoadOP::from_type(ty),
            from: mem,
            flags,
        }
    }

    /// Generic constructor for a store.
    pub fn gen_store(mem: AMode, from_reg: Reg, ty: Type, flags: MemFlags) -> Inst {
        Inst::Store {
            src: from_reg,
            op: StoreOP::from_type(ty),
            to: mem,
            flags,
        }
    }
}

//=============================================================================
fn riscv64_get_operands<F: Fn(VReg) -> VReg>(inst: &Inst, collector: &mut OperandCollector<'_, F>) {
    match inst {
        &Inst::Nop0 => {}
        &Inst::Nop4 => {}
        &Inst::BrTable { index, tmp1, .. } => {
            collector.reg_use(index);
            collector.reg_early_def(tmp1);
        }
        &Inst::BrTableCheck { index, .. } => {
            collector.reg_use(index);
        }
        &Inst::Auipc { rd, .. } => collector.reg_def(rd),
        &Inst::Lui { rd, .. } => collector.reg_def(rd),
        &Inst::AluRRR { rd, rs1, rs2, .. } => {
            collector.reg_use(rs1);
            collector.reg_use(rs2);
            collector.reg_def(rd);
        }
        &Inst::FpuRRR { rd, rs1, rs2, .. } => {
            collector.reg_use(rs1);
            collector.reg_use(rs2);
            collector.reg_def(rd);
        }
        &Inst::AluRRImm12 { rd, rs, .. } => {
            collector.reg_use(rs);
            collector.reg_def(rd);
        }
        &Inst::Load { rd, from, .. } => {
            collector.reg_use(from.get_base_register());
            collector.reg_def(rd);
        }
        &Inst::Store { to, src, .. } => {
            collector.reg_use(to.get_base_register());
            collector.reg_use(src);
        }

        &Inst::Args { ref args } => {
            for arg in args {
                collector.reg_fixed_def(arg.vreg, arg.preg);
            }
        }
        &Inst::Ret { ref rets } => {
            for ret in rets {
                collector.reg_fixed_use(ret.vreg, ret.preg);
            }
        }

        &Inst::Extend { rd, rn, .. } => {
            collector.reg_use(rn);
            collector.reg_def(rd);
        }
        &Inst::AjustSp { .. } => {}
        &Inst::Call { ref info } => {
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }
        &Inst::CallInd { ref info } => {
            collector.reg_use(info.rn);
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }
        &Inst::TrapIf { test, .. } => {
            collector.reg_use(test);
        }
        &Inst::TrapFf { x, y, tmp, .. } => {
            collector.reg_use(x);
            collector.reg_use(y);
            collector.reg_early_def(tmp);
        }

        &Inst::Jal { .. } => {}
        &Inst::CondBr { kind, .. } => {
            collector.reg_use(kind.rs1);
            collector.reg_use(kind.rs2);
        }
        &Inst::LoadExtName { rd, .. } => {
            collector.reg_def(rd);
        }
        &Inst::LoadAddr { rd, mem } => {
            collector.reg_use(mem.get_base_register());
            collector.reg_early_def(rd);
        }

        &Inst::VirtualSPOffsetAdj { .. } => {}
        &Inst::Mov { rd, rm, .. } => {
            collector.reg_use(rm);
            collector.reg_def(rd);
        }
        &Inst::Fence { .. } => {}
        &Inst::FenceI => {}
        &Inst::ECall => {}
        &Inst::EBreak => {}
        &Inst::Udf { .. } => {}
        &Inst::FpuRR { rd, rs, .. } => {
            collector.reg_use(rs);
            collector.reg_def(rd);
        }
        &Inst::FpuRRRR {
            rd, rs1, rs2, rs3, ..
        } => {
            collector.reg_uses(&[rs1, rs2, rs3]);
            collector.reg_def(rd);
        }

        &Inst::Jalr { rd, base, .. } => {
            collector.reg_use(base);
            collector.reg_def(rd);
        }
        &Inst::Atomic { rd, addr, src, .. } => {
            collector.reg_use(addr);
            collector.reg_use(src);
            collector.reg_def(rd);
        }
        &Inst::Fcmp { rd, rs1, rs2, .. } => {
            collector.reg_use(rs1);
            collector.reg_use(rs2);
            collector.reg_early_def(rd);
        }
        &Inst::Select {
            ref dst,
            condition,
            x,
            y,
            ..
        } => {
            collector.reg_use(condition);
            collector.reg_uses(x.regs());
            collector.reg_uses(y.regs());
            collector.reg_defs(&dst[..]);
        }
        &Inst::ReferenceCheck { rd, x, .. } => {
            collector.reg_use(x);
            collector.reg_def(rd);
        }
        &Inst::AtomicCas {
            offset,
            t0,
            dst,
            e,
            addr,
            v,
            ..
        } => {
            collector.reg_uses(&[offset, e, addr, v]);
            collector.reg_early_def(t0);
            collector.reg_early_def(dst);
        }
        &Inst::IntSelect {
            ref dst,
            ref x,
            ref y,
            ..
        } => {
            collector.reg_uses(x.regs());
            collector.reg_uses(y.regs());
            collector.reg_defs(&dst[..]);
        }

        &Inst::Csr { rd, rs, .. } => {
            if let Some(rs) = rs {
                collector.reg_use(rs);
            }
            collector.reg_def(rd);
        }

        &Inst::Icmp { rd, a, b, .. } => {
            collector.reg_uses(a.regs());
            collector.reg_uses(b.regs());
            collector.reg_def(rd);
        }

        &Inst::SelectReg {
            rd,
            rs1,
            rs2,
            condition,
        } => {
            collector.reg_use(condition.rs1);
            collector.reg_use(condition.rs2);
            collector.reg_use(rs1);
            collector.reg_use(rs2);
            collector.reg_def(rd);
        }
        &Inst::FcvtToInt { rd, rs, tmp, .. } => {
            collector.reg_use(rs);
            collector.reg_early_def(tmp);
            collector.reg_def(rd);
        }
        &Inst::SelectIf {
            ref rd,
            test,
            ref x,
            ref y,
            ..
        } => {
            collector.reg_use(test);
            collector.reg_uses(x.regs());
            collector.reg_uses(y.regs());
            rd.iter().for_each(|r| collector.reg_def(*r));
        }
        &Inst::RawData { .. } => {}
        &Inst::AtomicStore { src, p, .. } => {
            collector.reg_use(src);
            collector.reg_use(p);
        }
        &Inst::AtomicLoad { rd, p, .. } => {
            collector.reg_use(p);
            collector.reg_def(rd);
        }
        &Inst::AtomicRmwLoop {
            offset,
            dst,
            p,
            x,
            t0,
            ..
        } => {
            collector.reg_uses(&[offset, p, x]);
            collector.reg_early_def(t0);
            collector.reg_early_def(dst);
        }
        &Inst::TrapIfC { rs1, rs2, .. } => {
            collector.reg_use(rs1);
            collector.reg_use(rs2);
        }
        &Inst::Unwind { .. } => {}
        &Inst::DummyUse { reg } => {
            collector.reg_use(reg);
        }
        &Inst::FloatRound {
            rd,
            int_tmp,
            f_tmp,
            rs,
            ..
        } => {
            collector.reg_use(rs);
            collector.reg_early_def(int_tmp);
            collector.reg_early_def(f_tmp);
            collector.reg_early_def(rd);
        }
        &Inst::FloatSelect {
            rd, tmp, rs1, rs2, ..
        } => {
            collector.reg_uses(&[rs1, rs2]);
            collector.reg_early_def(tmp);
            collector.reg_early_def(rd);
        }
        &Inst::FloatSelectPseudo {
            rd, tmp, rs1, rs2, ..
        } => {
            collector.reg_uses(&[rs1, rs2]);
            collector.reg_early_def(tmp);
            collector.reg_early_def(rd);
        }
        &Inst::Popcnt {
            sum, step, rs, tmp, ..
        } => {
            collector.reg_use(rs);
            collector.reg_early_def(tmp);
            collector.reg_early_def(step);
            collector.reg_early_def(sum);
        }
        &Inst::Rev8 { rs, rd, tmp, step } => {
            collector.reg_use(rs);
            collector.reg_early_def(tmp);
            collector.reg_early_def(step);
            collector.reg_early_def(rd);
        }
        &Inst::Cltz {
            sum, step, tmp, rs, ..
        } => {
            collector.reg_use(rs);
            collector.reg_early_def(tmp);
            collector.reg_early_def(step);
            collector.reg_early_def(sum);
        }
        &Inst::Brev8 {
            rs,
            rd,
            step,
            tmp,
            tmp2,
            ..
        } => {
            collector.reg_use(rs);
            collector.reg_early_def(step);
            collector.reg_early_def(tmp);
            collector.reg_early_def(tmp2);
            collector.reg_early_def(rd);
        }
        &Inst::StackProbeLoop { .. } => {
            // StackProbeLoop has a tmp register and StackProbeLoop used at gen_prologue.
            // t3 will do the job. (t3 is caller-save register and not used directly by compiler like writable_spilltmp_reg)
            // gen_prologue is called at emit stage.
            // no need let reg alloc know.
        }
    }
}

impl MachInst for Inst {
    type LabelUse = LabelUse;
    type ABIMachineSpec = Riscv64MachineDeps;

    fn gen_dummy_use(reg: Reg) -> Self {
        Inst::DummyUse { reg }
    }

    fn canonical_type_for_rc(rc: RegClass) -> Type {
        match rc {
            regalloc2::RegClass::Int => I64,
            regalloc2::RegClass::Float => F64,
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

    fn get_operands<F: Fn(VReg) -> VReg>(&self, collector: &mut OperandCollector<'_, F>) {
        riscv64_get_operands(self, collector);
    }

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        match self {
            Inst::Mov { rd, rm, .. } => Some((rd.clone(), rm.clone())),
            _ => None,
        }
    }

    fn is_included_in_clobbers(&self) -> bool {
        match self {
            &Inst::Args { .. } => false,
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
            &Inst::Jal { .. } => MachTerminator::Uncond,
            &Inst::CondBr { .. } => MachTerminator::Cond,
            &Inst::Jalr { .. } => MachTerminator::Uncond,
            &Inst::Ret { .. } => MachTerminator::Ret,
            // BrTableCheck is a check before BrTable
            // can lead transfer to default_.
            &Inst::BrTable { .. } | &Inst::BrTableCheck { .. } => MachTerminator::Indirect,
            _ => MachTerminator::None,
        }
    }

    fn gen_move(to_reg: Writable<Reg>, from_reg: Reg, ty: Type) -> Inst {
        let x = Inst::Mov {
            rd: to_reg,
            rm: from_reg,
            ty,
        };
        x
    }

    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        mut alloc_tmp: F,
    ) -> SmallVec<[Inst; 4]> {
        if (ty.bits() <= 64 && ty.is_int()) || ty == R32 || ty == R64 {
            return Inst::load_constant_u64(to_regs.only_reg().unwrap(), value as u64);
        };
        match ty {
            F32 => {
                Inst::load_fp_constant32(to_regs.only_reg().unwrap(), value as u32, alloc_tmp(I64))
            }
            F64 => {
                Inst::load_fp_constant64(to_regs.only_reg().unwrap(), value as u64, alloc_tmp(I64))
            }
            I128 => {
                let mut insts = SmallInstVec::new();
                insts.extend(Inst::load_constant_u64(
                    to_regs.regs()[0],
                    (value >> 64) as u64,
                ));
                insts.extend(Inst::load_constant_u64(to_regs.regs()[1], value as u64));
                return insts;
            }
            _ => unreachable!("vector type not implemented now."),
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

    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
        match ty {
            I8 => Ok((&[RegClass::Int], &[I8])),
            I16 => Ok((&[RegClass::Int], &[I16])),
            I32 => Ok((&[RegClass::Int], &[I32])),
            I64 => Ok((&[RegClass::Int], &[I64])),
            R32 => panic!("32-bit reftype pointer should never be seen on riscv64"),
            R64 => Ok((&[RegClass::Int], &[R64])),
            F32 => Ok((&[RegClass::Float], &[F32])),
            F64 => Ok((&[RegClass::Float], &[F64])),
            I128 => Ok((&[RegClass::Int, RegClass::Int], &[I64, I64])),
            IFLAGS => Ok((&[RegClass::Int], &[IFLAGS])),
            FFLAGS => Ok((&[RegClass::Int], &[FFLAGS])),
            _ => Err(CodegenError::Unsupported(format!(
                "Unexpected SSA-value type: {}",
                ty
            ))),
        }
    }

    fn gen_jump(target: MachLabel) -> Inst {
        Inst::Jal {
            dest: BranchTarget::Label(target),
        }
    }

    fn worst_case_size() -> CodeOffset {
        // calculate by test function riscv64_worst_case_instruction_size()
        100
    }

    fn ref_type_regclass(_settings: &settings::Flags) -> RegClass {
        RegClass::Int
    }
}

//=============================================================================
// Pretty-printing of instructions.
pub fn reg_name(reg: Reg) -> String {
    match reg.to_real_reg() {
        Some(real) => match real.class() {
            RegClass::Int => match real.hw_enc() {
                0 => "zero".into(),
                1 => "ra".into(),
                2 => "sp".into(),
                3 => "gp".into(),
                4 => "tp".into(),
                5 => "t0".into(),
                6..=7 => format!("t{}", real.hw_enc() - 5),
                8 => "fp".into(),
                9 => "s1".into(),
                10..=17 => format!("a{}", real.hw_enc() - 10),
                18..=27 => format!("s{}", real.hw_enc() - 16),
                28..=31 => format!("t{}", real.hw_enc() - 25),
                _ => unreachable!(),
            },
            RegClass::Float => match real.hw_enc() {
                0..=7 => format!("ft{}", real.hw_enc() - 0),
                8..=9 => format!("fs{}", real.hw_enc() - 8),
                10..=17 => format!("fa{}", real.hw_enc() - 10),
                18..=27 => format!("fs{}", real.hw_enc() - 16),
                28..=31 => format!("ft{}", real.hw_enc() - 20),
                _ => unreachable!(),
            },
        },
        None => {
            format!("{:?}", reg)
        }
    }
}

impl Inst {
    fn print_with_state(
        &self,
        _state: &mut EmitState,
        allocs: &mut AllocationConsumer<'_>,
    ) -> String {
        let format_reg = |reg: Reg, allocs: &mut AllocationConsumer<'_>| -> String {
            let reg = allocs.next(reg);
            reg_name(reg)
        };

        let format_regs = |regs: &[Reg], allocs: &mut AllocationConsumer<'_>| -> String {
            let mut x = if regs.len() > 1 {
                String::from("[")
            } else {
                String::default()
            };
            regs.iter().for_each(|i| {
                x.push_str(format_reg(i.clone(), allocs).as_str());
                if *i != *regs.last().unwrap() {
                    x.push_str(",");
                }
            });
            if regs.len() > 1 {
                x.push_str("]");
            }
            x
        };
        let format_labels = |labels: &[MachLabel]| -> String {
            if labels.len() == 0 {
                return String::from("[_]");
            }
            let mut x = String::from("[");
            labels.iter().for_each(|l| {
                x.push_str(
                    format!(
                        "{:?}{}",
                        l,
                        if l != labels.last().unwrap() { "," } else { "" },
                    )
                    .as_str(),
                );
            });
            x.push_str("]");
            x
        };

        fn format_extend_op(signed: bool, from_bits: u8, _to_bits: u8) -> String {
            let type_name = match from_bits {
                1 => "b1",
                8 => "b",
                16 => "h",
                32 => "w",
                _ => unreachable!("from_bits:{:?}", from_bits),
            };
            format!("{}ext.{}", if signed { "s" } else { "u" }, type_name)
        }
        fn format_frm(rounding_mode: Option<FRM>) -> String {
            if let Some(r) = rounding_mode {
                format!(",{}", r.to_static_str(),)
            } else {
                "".into()
            }
        }
        match self {
            &Inst::Nop0 => {
                format!("##zero length nop")
            }
            &Inst::Nop4 => {
                format!("##fixed 4-size nop")
            }
            &Inst::StackProbeLoop {
                guard_size,
                probe_count,
                tmp,
            } => {
                let tmp = format_reg(tmp.to_reg(), allocs);
                format!(
                    "inline_stack_probe##guard_size={} probe_count={} tmp={}",
                    guard_size, probe_count, tmp
                )
            }
            &Inst::FloatRound {
                op,
                rd,
                int_tmp,
                f_tmp,
                rs,
                ty,
            } => {
                let rs = format_reg(rs, allocs);
                let int_tmp = format_reg(int_tmp.to_reg(), allocs);
                let f_tmp = format_reg(f_tmp.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "{} {},{}##int_tmp={} f_tmp={} ty={}",
                    op.op_name(),
                    rd,
                    rs,
                    int_tmp,
                    f_tmp,
                    ty
                )
            }
            &Inst::FloatSelectPseudo {
                op,
                rd,
                tmp,
                rs1,
                rs2,
                ty,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "f{}.{}.pseudo {},{},{}##tmp={} ty={}",
                    op.op_name(),
                    if ty == F32 { "s" } else { "d" },
                    rd,
                    rs1,
                    rs2,
                    tmp,
                    ty
                )
            }
            &Inst::FloatSelect {
                op,
                rd,
                tmp,
                rs1,
                rs2,
                ty,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "f{}.{} {},{},{}##tmp={} ty={}",
                    op.op_name(),
                    if ty == F32 { "s" } else { "d" },
                    rd,
                    rs1,
                    rs2,
                    tmp,
                    ty
                )
            }
            &Inst::AtomicStore { src, ty, p } => {
                let src = format_reg(src, allocs);
                let p = format_reg(p, allocs);
                format!("atomic_store.{} {},({})", ty, src, p)
            }
            &Inst::DummyUse { reg } => {
                let reg = format_reg(reg, allocs);
                format!("dummy_use {}", reg)
            }

            &Inst::AtomicLoad { rd, ty, p } => {
                let p = format_reg(p, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("atomic_load.{} {},({})", ty, rd, p)
            }
            &Inst::AtomicRmwLoop {
                offset,
                op,
                dst,
                ty,
                p,
                x,
                t0,
            } => {
                let offset = format_reg(offset, allocs);
                let p = format_reg(p, allocs);
                let x = format_reg(x, allocs);
                let t0 = format_reg(t0.to_reg(), allocs);
                let dst = format_reg(dst.to_reg(), allocs);
                format!(
                    "atomic_rmw.{} {} {},{},({})##t0={} offset={}",
                    ty, op, dst, x, p, t0, offset
                )
            }

            &Inst::RawData { ref data } => match data.len() {
                4 => {
                    let mut bytes = [0; 4];
                    for i in 0..bytes.len() {
                        bytes[i] = data[i];
                    }
                    format!(".4byte 0x{:x}", u32::from_le_bytes(bytes))
                }
                8 => {
                    let mut bytes = [0; 8];
                    for i in 0..bytes.len() {
                        bytes[i] = data[i];
                    }
                    format!(".8byte 0x{:x}", u64::from_le_bytes(bytes))
                }
                _ => {
                    format!(".data {:?}", data)
                }
            },
            &Inst::Unwind { ref inst } => {
                format!("unwind {:?}", inst)
            }
            &Inst::Brev8 {
                rs,
                ty,
                step,
                tmp,
                tmp2,
                rd,
            } => {
                let rs = format_reg(rs, allocs);
                let step = format_reg(step.to_reg(), allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let tmp2 = format_reg(tmp2.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "brev8 {},{}##tmp={} tmp2={} step={} ty={}",
                    rd, rs, tmp, tmp2, step, ty
                )
            }
            &Inst::SelectIf {
                if_spectre_guard,
                ref rd,
                test,
                ref x,
                ref y,
            } => {
                let test = format_reg(test, allocs);
                let x = format_regs(x.regs(), allocs);
                let y = format_regs(y.regs(), allocs);
                let rd: Vec<_> = rd.iter().map(|r| r.to_reg()).collect();
                let rd = format_regs(&rd[..], allocs);
                format!(
                    "selectif{} {},{},{}##test={}",
                    if if_spectre_guard {
                        "_spectre_guard"
                    } else {
                        ""
                    },
                    rd,
                    x,
                    y,
                    test
                )
            }
            &Inst::Popcnt {
                sum,
                step,
                rs,
                tmp,
                ty,
            } => {
                let rs = format_reg(rs, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let step = format_reg(step.to_reg(), allocs);
                let sum = format_reg(sum.to_reg(), allocs);
                format!("popcnt {},{}##ty={} tmp={} step={}", sum, rs, ty, tmp, step)
            }
            &Inst::Rev8 { rs, rd, tmp, step } => {
                let rs = format_reg(rs, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let step = format_reg(step.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("rev8 {},{}##step={} tmp={}", rd, rs, step, tmp)
            }
            &Inst::Cltz {
                sum,
                step,
                rs,
                tmp,
                ty,
                leading,
            } => {
                let rs = format_reg(rs, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let step = format_reg(step.to_reg(), allocs);
                let sum = format_reg(sum.to_reg(), allocs);
                format!(
                    "{} {},{}##ty={} tmp={} step={}",
                    if leading { "clz" } else { "ctz" },
                    sum,
                    rs,
                    ty,
                    tmp,
                    step
                )
            }
            &Inst::FcvtToInt {
                is_sat,
                rd,
                rs,
                is_signed,
                in_type,
                out_type,
                tmp,
            } => {
                let rs = format_reg(rs, allocs);
                let tmp = format_reg(tmp.to_reg(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "fcvt_to_{}int{}.{} {},{}##in_ty={} tmp={}",
                    if is_signed { "s" } else { "u" },
                    if is_sat { "_sat" } else { "" },
                    out_type,
                    rd,
                    rs,
                    in_type,
                    tmp
                )
            }
            &Inst::SelectReg {
                rd,
                rs1,
                rs2,
                ref condition,
            } => {
                let c_rs1 = format_reg(condition.rs1, allocs);
                let c_rs2 = format_reg(condition.rs2, allocs);
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "select_reg {},{},{}##condition={}",
                    rd,
                    rs1,
                    rs2,
                    format!("({} {} {})", c_rs1, condition.kind.to_static_str(), c_rs2),
                )
            }
            &Inst::AtomicCas {
                offset,
                t0,
                dst,
                e,
                addr,
                v,
                ty,
            } => {
                let offset = format_reg(offset, allocs);
                let e = format_reg(e, allocs);
                let addr = format_reg(addr, allocs);
                let v = format_reg(v, allocs);
                let t0 = format_reg(t0.to_reg(), allocs);
                let dst = format_reg(dst.to_reg(), allocs);
                format!(
                    "atomic_cas.{} {},{},{},({})##t0={} offset={}",
                    ty, dst, e, v, addr, t0, offset,
                )
            }
            &Inst::Icmp { cc, rd, a, b, ty } => {
                let a = format_regs(a.regs(), allocs);
                let b = format_regs(b.regs(), allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{},{}##ty={}", cc.to_static_str(), rd, a, b, ty)
            }
            &Inst::IntSelect {
                op,
                ref dst,
                x,
                y,
                ty,
            } => {
                let x = format_regs(x.regs(), allocs);
                let y = format_regs(y.regs(), allocs);
                let dst: Vec<_> = dst.iter().map(|r| r.to_reg()).collect();
                let dst = format_regs(&dst[..], allocs);
                format!("{} {},{},{}##ty={}", op.op_name(), dst, x, y, ty,)
            }
            &Inst::BrTableCheck {
                index,
                targets_len,
                default_,
            } => {
                let index = format_reg(index, allocs);
                format!(
                    "br_table_check {}##targets_len={} default_={}",
                    index, targets_len, default_
                )
            }
            &Inst::BrTable {
                index,
                tmp1,
                ref targets,
            } => {
                let targets: Vec<_> = targets.iter().map(|x| x.as_label().unwrap()).collect();
                format!(
                    "{} {},{}##tmp1={}",
                    "br_table",
                    format_reg(index, allocs),
                    format_labels(&targets[..]),
                    format_reg(tmp1.to_reg(), allocs),
                )
            }
            &Inst::Auipc { rd, imm } => {
                format!(
                    "{} {},{}",
                    "auipc",
                    format_reg(rd.to_reg(), allocs),
                    imm.bits
                )
            }

            &Inst::ReferenceCheck { rd, op, x } => {
                let x = format_reg(x, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{}", op.op_name(), rd, x)
            }
            &Inst::Jalr { rd, base, offset } => {
                let base = format_reg(base, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{}({})", "jalr", rd, offset.bits, base)
            }
            &Inst::Lui { rd, ref imm } => {
                format!("{} {},{}", "lui", format_reg(rd.to_reg(), allocs), imm.bits)
            }

            &Inst::AluRRR {
                alu_op,
                rd,
                rs1,
                rs2,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{},{}", alu_op.op_name(), rd, rs1, rs2,)
            }
            &Inst::FpuRR {
                frm,
                alu_op,
                rd,
                rs,
            } => {
                let rs = format_reg(rs, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{}{}", alu_op.op_name(), rd, rs, format_frm(frm))
            }
            &Inst::FpuRRR {
                alu_op,
                rd,
                rs1,
                rs2,
                frm,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                let rs1_is_rs2 = rs1 == rs2;
                if rs1_is_rs2 && alu_op.is_copy_sign() {
                    // this is move instruction.
                    format!(
                        "fmv.{} {},{}",
                        if alu_op.is_32() { "s" } else { "d" },
                        rd,
                        rs1
                    )
                } else if rs1_is_rs2 && alu_op.is_copy_neg_sign() {
                    format!(
                        "fneg.{} {},{}",
                        if alu_op.is_32() { "s" } else { "d" },
                        rd,
                        rs1
                    )
                } else if rs1_is_rs2 && alu_op.is_copy_xor_sign() {
                    format!(
                        "fabs.{} {},{}",
                        if alu_op.is_32() { "s" } else { "d" },
                        rd,
                        rs1
                    )
                } else {
                    format!(
                        "{} {},{},{}{}",
                        alu_op.op_name(),
                        rd,
                        rs1,
                        rs2,
                        format_frm(frm)
                    )
                }
            }
            &Inst::Csr {
                csr_op,
                rd,
                rs,
                imm,
                csr,
            } => {
                let rs = rs.map_or("".into(), |r| format_reg(r, allocs));
                let rd = format_reg(rd.to_reg(), allocs);
                if csr_op.need_rs() {
                    format!("{} {},{},{}", csr_op.op_name(), rd, csr, rs)
                } else {
                    format!("{} {},{},{}", csr_op.op_name(), rd, csr, imm.unwrap())
                }
            }
            &Inst::FpuRRRR {
                alu_op,
                rd,
                rs1,
                rs2,
                rs3,
                frm,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let rs3 = format_reg(rs3, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "{} {},{},{},{}{}",
                    alu_op.op_name(),
                    rd,
                    rs1,
                    rs2,
                    rs3,
                    format_frm(frm)
                )
            }
            &Inst::AluRRImm12 {
                alu_op,
                rd,
                rs,
                ref imm12,
            } => {
                let rs_s = format_reg(rs, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                // check if it is a load constant.
                if alu_op == AluOPRRI::Addi && rs == zero_reg() {
                    format!("li {},{}", rd, imm12.as_i16())
                } else if alu_op == AluOPRRI::Xori && imm12.as_i16() == -1 {
                    format!("not {},{}", rd, rs_s)
                } else {
                    if alu_op.option_funct12().is_some() {
                        format!("{} {},{}", alu_op.op_name(), rd, rs_s)
                    } else {
                        format!("{} {},{},{}", alu_op.op_name(), rd, rs_s, imm12.as_i16())
                    }
                }
            }
            &Inst::Load {
                rd,
                op,
                from,
                flags: _flags,
            } => {
                let base = from.to_string_with_alloc(allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("{} {},{}", op.op_name(), rd, base,)
            }
            &Inst::Fcmp {
                rd,
                cc,
                ty,
                rs1,
                rs2,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!(
                    "f{}.{} {},{},{}",
                    cc,
                    if ty == F32 { "s" } else { "d" },
                    rd,
                    rs1,
                    rs2,
                )
            }
            &Inst::Store {
                to,
                src,
                op,
                flags: _flags,
            } => {
                let base = to.to_string_with_alloc(allocs);
                let src = format_reg(src, allocs);
                format!("{} {},{}", op.op_name(), src, base,)
            }
            &Inst::Args { ref args } => {
                let mut s = "args".to_string();
                let mut empty_allocs = AllocationConsumer::default();
                for arg in args {
                    use std::fmt::Write;
                    let preg = format_reg(arg.preg, &mut empty_allocs);
                    let def = format_reg(arg.vreg.to_reg(), allocs);
                    write!(&mut s, " {}={}", def, preg).unwrap();
                }
                s
            }
            &Inst::Ret { .. } => {
                format!("ret")
            }

            &MInst::Extend {
                rd,
                rn,
                signed,
                from_bits,
                to_bits,
            } => {
                let rn = format_reg(rn, allocs);
                let rm = format_reg(rd.to_reg(), allocs);
                format!(
                    "{} {},{}",
                    format_extend_op(signed, from_bits, to_bits),
                    rm,
                    rn
                )
            }
            &MInst::AjustSp { amount } => {
                format!("{} sp,{:+}", "add", amount)
            }
            &MInst::Call { ref info } => format!("call {}", info.dest.display(None)),
            &MInst::CallInd { ref info } => {
                let rd = format_reg(info.rn, allocs);
                format!("callind {}", rd)
            }
            &MInst::TrapIf { test, trap_code } => {
                format!("trap_if {},{}", format_reg(test, allocs), trap_code,)
            }
            &MInst::TrapIfC {
                rs1,
                rs2,
                cc,
                trap_code,
            } => {
                let rs1 = format_reg(rs1, allocs);
                let rs2 = format_reg(rs2, allocs);
                format!("trap_ifc {}##({} {} {})", trap_code, rs1, cc, rs2)
            }
            &MInst::TrapFf {
                cc,
                x,
                y,
                ty,
                trap_code,
                tmp,
            } => format!(
                "trap_ff_{} {} {},{}##tmp={} ty={}",
                cc,
                trap_code,
                format_reg(x, allocs),
                format_reg(y, allocs),
                format_reg(tmp.to_reg(), allocs),
                ty,
            ),
            &MInst::Jal { dest, .. } => {
                format!("{} {}", "j", dest)
            }
            &MInst::CondBr {
                taken,
                not_taken,
                kind,
                ..
            } => {
                let rs1 = format_reg(kind.rs1, allocs);
                let rs2 = format_reg(kind.rs2, allocs);
                if not_taken.is_zero() && taken.as_label().is_none() {
                    let off = taken.as_offset().unwrap();
                    format!("{} {},{},{}", kind.op_name(), rs1, rs2, off)
                } else {
                    let x = format!(
                        "{} {},{},taken({}),not_taken({})",
                        kind.op_name(),
                        rs1,
                        rs2,
                        taken,
                        not_taken
                    );
                    x
                }
            }
            &MInst::Atomic {
                op,
                rd,
                addr,
                src,
                amo,
            } => {
                let op_name = op.op_name(amo);
                let addr = format_reg(addr, allocs);
                let src = format_reg(src, allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                if op.is_load() {
                    format!("{} {},({})", op_name, rd, addr)
                } else {
                    format!("{} {},{},({})", op_name, rd, src, addr)
                }
            }
            &MInst::LoadExtName {
                rd,
                ref name,
                offset,
            } => {
                let rd = format_reg(rd.to_reg(), allocs);
                format!("load_sym {},{}{:+}", rd, name.display(None), offset)
            }
            &MInst::LoadAddr { ref rd, ref mem } => {
                let rs = mem.to_addr(allocs);
                let rd = format_reg(rd.to_reg(), allocs);
                format!("load_addr {},{}", rd, rs)
            }
            &MInst::VirtualSPOffsetAdj { amount } => {
                format!("virtual_sp_offset_adj {:+}", amount)
            }
            &MInst::Mov { rd, rm, ty } => {
                let rd = format_reg(rd.to_reg(), allocs);
                let rm = format_reg(rm, allocs);
                let v = if ty == F32 {
                    "fmv.s"
                } else if ty == F64 {
                    "fmv.d"
                } else {
                    "mv"
                };
                format!("{} {},{}", v, rd, rm)
            }
            &MInst::Fence { pred, succ } => {
                format!(
                    "fence {},{}",
                    Inst::fence_req_to_string(pred),
                    Inst::fence_req_to_string(succ),
                )
            }
            &MInst::FenceI => "fence.i".into(),
            &MInst::Select {
                ref dst,
                condition,
                ref x,
                ref y,
                ty,
            } => {
                let condition = format_reg(condition, allocs);
                let x = format_regs(x.regs(), allocs);
                let y = format_regs(y.regs(), allocs);
                let dst: Vec<_> = dst.clone().into_iter().map(|r| r.to_reg()).collect();
                let dst = format_regs(&dst[..], allocs);
                format!("select_{} {},{},{}##condition={}", ty, dst, x, y, condition)
            }
            &MInst::Udf { trap_code } => format!("udf##trap_code={}", trap_code),
            &MInst::EBreak {} => String::from("ebreak"),
            &MInst::ECall {} => String::from("ecall"),
        }
    }
}

/// Different forms of label references for different instruction formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUse {
    /// 20-bit branch offset (unconditional branches). PC-rel, offset is
    /// imm << 1. Immediate is 20 signed bits. Use in Jal instructions.
    Jal20,

    /// The unconditional jump instructions all use PC-relative
    /// addressing to help support position independent code. The JALR
    /// instruction was defined to enable a two-instruction sequence to
    /// jump anywhere in a 32-bit absolute address range. A LUI
    /// instruction can first load rs1 with the upper 20 bits of a
    /// target address, then JALR can add in the lower bits. Similarly,
    /// AUIPC then JALR can jump anywhere in a 32-bit pc-relative
    /// address range.
    PCRel32,

    /// All branch instructions use the B-type instruction format. The
    /// 12-bit B-immediate encodes signed offsets in multiples of 2, and
    /// is added to the current pc to give the target address. The
    /// conditional branch range is ±4 KiB.
    B12,
}

impl MachInstLabelUse for LabelUse {
    /// Alignment for veneer code. Every Riscv64 instruction must be
    /// 4-byte-aligned.
    const ALIGN: CodeOffset = 4;

    /// Maximum PC-relative range (positive), inclusive.
    fn max_pos_range(self) -> CodeOffset {
        match self {
            LabelUse::Jal20 => ((1 << 19) - 1) * 2,
            LabelUse::PCRel32 => Inst::imm_max() as CodeOffset,
            LabelUse::B12 => ((1 << 11) - 1) * 2,
        }
    }

    /// Maximum PC-relative range (negative).
    fn max_neg_range(self) -> CodeOffset {
        match self {
            LabelUse::PCRel32 => Inst::imm_min().abs() as CodeOffset,
            _ => self.max_pos_range() + 2,
        }
    }

    /// Size of window into code needed to do the patch.
    fn patch_size(self) -> CodeOffset {
        match self {
            LabelUse::Jal20 => 4,
            LabelUse::PCRel32 => 8,
            LabelUse::B12 => 4,
        }
    }

    /// Perform the patch.
    fn patch(self, buffer: &mut [u8], use_offset: CodeOffset, label_offset: CodeOffset) {
        assert!(use_offset % 4 == 0);
        assert!(label_offset % 4 == 0);
        let offset = (label_offset as i64) - (use_offset as i64);

        // re-check range
        assert!(
            offset >= -(self.max_neg_range() as i64) && offset <= (self.max_pos_range() as i64),
            "{:?} offset '{}' use_offset:'{}' label_offset:'{}'  must not exceed max range.",
            self,
            offset,
            use_offset,
            label_offset,
        );
        self.patch_raw_offset(buffer, offset);
    }

    /// Is a veneer supported for this label reference type?
    fn supports_veneer(self) -> bool {
        match self {
            Self::B12 => true,
            Self::Jal20 => true,
            _ => false,
        }
    }

    /// How large is the veneer, if supported?
    fn veneer_size(self) -> CodeOffset {
        match self {
            Self::B12 => 8,
            Self::Jal20 => 8,
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
        let base = writable_spilltmp_reg();
        {
            let x = enc_auipc(base, Imm20::from_bits(0)).to_le_bytes();
            buffer[0] = x[0];
            buffer[1] = x[1];
            buffer[2] = x[2];
            buffer[3] = x[3];
        }
        {
            let x = enc_jalr(writable_zero_reg(), base.to_reg(), Imm12::from_bits(0)).to_le_bytes();
            buffer[4] = x[0];
            buffer[5] = x[1];
            buffer[6] = x[2];
            buffer[7] = x[3];
        }
        (veneer_offset, Self::PCRel32)
    }

    fn from_reloc(reloc: Reloc, addend: Addend) -> Option<LabelUse> {
        match (reloc, addend) {
            (Reloc::RiscvCall, _) => Some(Self::PCRel32),
            _ => None,
        }
    }
}

impl LabelUse {
    fn offset_in_range(self, offset: i64) -> bool {
        let min = -(self.max_neg_range() as i64);
        let max = self.max_pos_range() as i64;
        offset >= min && offset <= max
    }

    fn patch_raw_offset(self, buffer: &mut [u8], offset: i64) {
        let insn = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        match self {
            LabelUse::Jal20 => {
                let offset = offset as u32;
                let v = ((offset >> 12 & 0b1111_1111) << 12)
                    | ((offset >> 11 & 0b1) << 20)
                    | ((offset >> 1 & 0b11_1111_1111) << 21)
                    | ((offset >> 20 & 0b1) << 31);
                buffer[0..4].clone_from_slice(&u32::to_le_bytes(insn | v));
            }
            LabelUse::PCRel32 => {
                let insn2 = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
                Inst::generate_imm(offset as u64, |imm20, imm12| {
                    let imm20 = imm20.unwrap_or_default();
                    let imm12 = imm12.unwrap_or_default();
                    // Encode the OR-ed-in value with zero_reg(). The
                    // register parameter must be in the original
                    // encoded instruction and or'ing in zeroes does not
                    // change it.
                    buffer[0..4].clone_from_slice(&u32::to_le_bytes(
                        insn | enc_auipc(writable_zero_reg(), imm20),
                    ));
                    buffer[4..8].clone_from_slice(&u32::to_le_bytes(
                        insn2 | enc_jalr(writable_zero_reg(), zero_reg(), imm12),
                    ));
                })
                // expect make sure we handled.
                .expect("we have check the range before,this is a compiler error.");
            }

            LabelUse::B12 => {
                let offset = offset as u32;
                let v = ((offset >> 11 & 0b1) << 7)
                    | ((offset >> 1 & 0b1111) << 8)
                    | ((offset >> 5 & 0b11_1111) << 25)
                    | ((offset >> 12 & 0b1) << 31);
                buffer[0..4].clone_from_slice(&u32::to_le_bytes(insn | v));
            }
        }
    }
}

pub(crate) fn overflow_already_lowerd() -> ! {
    unreachable!("overflow and nof should be lowered at early phase.")
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn label_use_max_range() {
        assert!(LabelUse::B12.max_neg_range() == LabelUse::B12.max_pos_range() + 2);
        assert!(LabelUse::Jal20.max_neg_range() == LabelUse::Jal20.max_pos_range() + 2);
        assert!(LabelUse::PCRel32.max_pos_range() == (Inst::imm_max() as CodeOffset));
        assert!(LabelUse::PCRel32.max_neg_range() == (Inst::imm_min().abs() as CodeOffset));
        assert!(LabelUse::B12.max_pos_range() == ((1 << 11) - 1) * 2);
    }
}
