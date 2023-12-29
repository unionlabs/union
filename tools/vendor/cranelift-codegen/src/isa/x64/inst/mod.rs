//! This module defines x86_64-specific machine instruction types.

use crate::binemit::{Addend, CodeOffset, Reloc, StackMap};
use crate::ir::{types, ExternalName, LibCall, Opcode, RelSourceLoc, TrapCode, Type};
use crate::isa::x64::abi::X64ABIMachineSpec;
use crate::isa::x64::inst::regs::{pretty_print_reg, show_ireg_sized};
use crate::isa::x64::settings as x64_settings;
use crate::isa::CallConv;
use crate::{machinst::*, trace};
use crate::{settings, CodegenError, CodegenResult};
use alloc::boxed::Box;
use alloc::vec::Vec;
use regalloc2::{Allocation, PRegSet, VReg};
use smallvec::{smallvec, SmallVec};
use std::fmt;
use std::string::{String, ToString};

pub mod args;
mod emit;
#[cfg(test)]
mod emit_tests;
pub mod regs;
pub mod unwind;

use args::*;

//=============================================================================
// Instructions (top level): definition

// `Inst` is defined inside ISLE as `MInst`. We publicly re-export it here.
pub use super::lower::isle::generated_code::MInst as Inst;

// Out-of-line data for calls, to keep the size of `Inst` downn.
#[derive(Clone, Debug)]
pub struct CallInfo {
    /// Register uses of this call.
    pub uses: CallArgList,
    /// Register defs of this call.
    pub defs: CallRetList,
    /// Registers clobbered by this call, as per its calling convention.
    pub clobbers: PRegSet,
    /// The opcode of this call.
    pub opcode: Opcode,
}

#[test]
#[cfg(target_pointer_width = "64")]
fn inst_size_test() {
    // This test will help with unintentionally growing the size
    // of the Inst enum.
    assert_eq!(40, std::mem::size_of::<Inst>());
}

pub(crate) fn low32_will_sign_extend_to_64(x: u64) -> bool {
    let xs = x as i64;
    xs == ((xs << 32) >> 32)
}

impl Inst {
    /// Retrieve a list of ISA feature sets in which the instruction is available. An empty list
    /// indicates that the instruction is available in the baseline feature set (i.e. SSE2 and
    /// below); more than one `InstructionSet` in the list indicates that the instruction is present
    /// *any* of the included ISA feature sets.
    fn available_in_any_isa(&self) -> SmallVec<[InstructionSet; 2]> {
        match self {
            // These instructions are part of SSE2, which is a basic requirement in Cranelift, and
            // don't have to be checked.
            Inst::AluRmiR { .. }
            | Inst::AluRM { .. }
            | Inst::AtomicRmwSeq { .. }
            | Inst::Bswap { .. }
            | Inst::CallKnown { .. }
            | Inst::CallUnknown { .. }
            | Inst::CheckedDivOrRemSeq { .. }
            | Inst::Cmove { .. }
            | Inst::CmpRmiR { .. }
            | Inst::CvtFloatToSintSeq { .. }
            | Inst::CvtFloatToUintSeq { .. }
            | Inst::CvtUint64ToFloatSeq { .. }
            | Inst::Div { .. }
            | Inst::Fence { .. }
            | Inst::Hlt
            | Inst::Imm { .. }
            | Inst::JmpCond { .. }
            | Inst::JmpIf { .. }
            | Inst::JmpKnown { .. }
            | Inst::JmpTableSeq { .. }
            | Inst::JmpUnknown { .. }
            | Inst::LoadEffectiveAddress { .. }
            | Inst::LoadExtName { .. }
            | Inst::LockCmpxchg { .. }
            | Inst::Mov64MR { .. }
            | Inst::MovRM { .. }
            | Inst::MovRR { .. }
            | Inst::MovFromPReg { .. }
            | Inst::MovToPReg { .. }
            | Inst::MovsxRmR { .. }
            | Inst::MovzxRmR { .. }
            | Inst::MulHi { .. }
            | Inst::Neg { .. }
            | Inst::Not { .. }
            | Inst::Nop { .. }
            | Inst::Pop64 { .. }
            | Inst::Push64 { .. }
            | Inst::StackProbeLoop { .. }
            | Inst::Args { .. }
            | Inst::Ret { .. }
            | Inst::Setcc { .. }
            | Inst::ShiftR { .. }
            | Inst::SignExtendData { .. }
            | Inst::TrapIf { .. }
            | Inst::TrapIfAnd { .. }
            | Inst::TrapIfOr { .. }
            | Inst::Ud2 { .. }
            | Inst::VirtualSPOffsetAdj { .. }
            | Inst::XmmCmove { .. }
            | Inst::XmmCmpRmR { .. }
            | Inst::XmmMinMaxSeq { .. }
            | Inst::XmmUninitializedValue { .. }
            | Inst::ElfTlsGetAddr { .. }
            | Inst::MachOTlsGetAddr { .. }
            | Inst::CoffTlsGetAddr { .. }
            | Inst::Unwind { .. }
            | Inst::DummyUse { .. } => smallvec![],

            Inst::UnaryRmR { op, .. } => op.available_from(),

            // These use dynamic SSE opcodes.
            Inst::GprToXmm { op, .. }
            | Inst::XmmMovRM { op, .. }
            | Inst::XmmRmiReg { opcode: op, .. }
            | Inst::XmmRmR { op, .. }
            | Inst::XmmRmRBlend { op, .. }
            | Inst::XmmRmRImm { op, .. }
            | Inst::XmmToGpr { op, .. }
            | Inst::XmmUnaryRmRImm { op, .. }
            | Inst::XmmUnaryRmR { op, .. } => smallvec![op.available_from()],

            Inst::XmmUnaryRmREvex { op, .. }
            | Inst::XmmRmREvex { op, .. }
            | Inst::XmmRmREvex3 { op, .. } => op.available_from(),

            Inst::XmmRmRVex { op, .. } => op.available_from(),
        }
    }
}

// Handy constructors for Insts.

impl Inst {
    pub(crate) fn nop(len: u8) -> Self {
        debug_assert!(len <= 15);
        Self::Nop { len }
    }

    pub(crate) fn alu_rmi_r(
        size: OperandSize,
        op: AluRmiROpcode,
        src: RegMemImm,
        dst: Writable<Reg>,
    ) -> Self {
        debug_assert!(size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        src.assert_regclass_is(RegClass::Int);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        Self::AluRmiR {
            size,
            op,
            src1: Gpr::new(dst.to_reg()).unwrap(),
            src2: GprMemImm::new(src).unwrap(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn unary_rm_r(
        size: OperandSize,
        op: UnaryRmROpcode,
        src: RegMem,
        dst: Writable<Reg>,
    ) -> Self {
        src.assert_regclass_is(RegClass::Int);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        debug_assert!(size.is_one_of(&[
            OperandSize::Size16,
            OperandSize::Size32,
            OperandSize::Size64
        ]));
        Self::UnaryRmR {
            size,
            op,
            src: GprMem::new(src).unwrap(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn not(size: OperandSize, src: Writable<Reg>) -> Inst {
        debug_assert_eq!(src.to_reg().class(), RegClass::Int);
        Inst::Not {
            size,
            src: Gpr::new(src.to_reg()).unwrap(),
            dst: WritableGpr::from_writable_reg(src).unwrap(),
        }
    }

    pub(crate) fn div(
        size: OperandSize,
        signed: bool,
        divisor: RegMem,
        dividend_lo: Gpr,
        dividend_hi: Gpr,
        dst_quotient: WritableGpr,
        dst_remainder: WritableGpr,
    ) -> Inst {
        divisor.assert_regclass_is(RegClass::Int);
        Inst::Div {
            size,
            signed,
            divisor: GprMem::new(divisor).unwrap(),
            dividend_lo,
            dividend_hi,
            dst_quotient,
            dst_remainder,
        }
    }

    pub(crate) fn checked_div_or_rem_seq(
        kind: DivOrRemKind,
        size: OperandSize,
        divisor: Reg,
        dividend_lo: Gpr,
        dividend_hi: Gpr,
        dst_quotient: WritableGpr,
        dst_remainder: WritableGpr,
        tmp: Option<Writable<Reg>>,
    ) -> Inst {
        debug_assert!(divisor.class() == RegClass::Int);
        debug_assert!(tmp
            .map(|tmp| tmp.to_reg().class() == RegClass::Int)
            .unwrap_or(true));
        Inst::CheckedDivOrRemSeq {
            kind,
            size,
            divisor: Gpr::new(divisor).unwrap(),
            dividend_lo,
            dividend_hi,
            dst_quotient,
            dst_remainder,
            tmp: tmp.map(|tmp| WritableGpr::from_writable_reg(tmp).unwrap()),
        }
    }

    pub(crate) fn sign_extend_data(size: OperandSize, src: Gpr, dst: WritableGpr) -> Inst {
        Inst::SignExtendData { size, src, dst }
    }

    pub(crate) fn imm(dst_size: OperandSize, simm64: u64, dst: Writable<Reg>) -> Inst {
        debug_assert!(dst_size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        // Try to generate a 32-bit immediate when the upper high bits are zeroed (which matches
        // the semantics of movl).
        let dst_size = match dst_size {
            OperandSize::Size64 if simm64 > u32::max_value() as u64 => OperandSize::Size64,
            _ => OperandSize::Size32,
        };
        Inst::Imm {
            dst_size,
            simm64,
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn mov_r_r(size: OperandSize, src: Reg, dst: Writable<Reg>) -> Inst {
        debug_assert!(size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        debug_assert!(src.class() == RegClass::Int);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        let src = Gpr::new(src).unwrap();
        let dst = WritableGpr::from_writable_reg(dst).unwrap();
        Inst::MovRR { size, src, dst }
    }

    /// Convenient helper for unary float operations.
    pub(crate) fn xmm_unary_rm_r(op: SseOpcode, src: RegMem, dst: Writable<Reg>) -> Inst {
        src.assert_regclass_is(RegClass::Float);
        debug_assert!(dst.to_reg().class() == RegClass::Float);
        Inst::XmmUnaryRmR {
            op,
            src: XmmMem::new(src).unwrap(),
            dst: WritableXmm::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn xmm_rm_r(op: SseOpcode, src: RegMem, dst: Writable<Reg>) -> Self {
        src.assert_regclass_is(RegClass::Float);
        debug_assert!(dst.to_reg().class() == RegClass::Float);
        Inst::XmmRmR {
            op,
            src1: Xmm::new(dst.to_reg()).unwrap(),
            src2: XmmMem::new(src).unwrap(),
            dst: WritableXmm::from_writable_reg(dst).unwrap(),
        }
    }

    #[cfg(test)]
    pub(crate) fn xmm_rm_r_vex(op: AvxOpcode, src3: RegMem, src2: Reg, dst: Writable<Reg>) -> Self {
        src3.assert_regclass_is(RegClass::Float);
        debug_assert!(src2.class() == RegClass::Float);
        debug_assert!(dst.to_reg().class() == RegClass::Float);
        Inst::XmmRmRVex {
            op,
            src3: XmmMem::new(src3).unwrap(),
            src2: Xmm::new(src2).unwrap(),
            src1: Xmm::new(dst.to_reg()).unwrap(),
            dst: WritableXmm::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn xmm_mov_r_m(op: SseOpcode, src: Reg, dst: impl Into<SyntheticAmode>) -> Inst {
        debug_assert!(src.class() == RegClass::Float);
        Inst::XmmMovRM {
            op,
            src,
            dst: dst.into(),
        }
    }

    pub(crate) fn xmm_to_gpr(
        op: SseOpcode,
        src: Reg,
        dst: Writable<Reg>,
        dst_size: OperandSize,
    ) -> Inst {
        debug_assert!(src.class() == RegClass::Float);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        debug_assert!(dst_size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        Inst::XmmToGpr {
            op,
            src: Xmm::new(src).unwrap(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
            dst_size,
        }
    }

    pub(crate) fn gpr_to_xmm(
        op: SseOpcode,
        src: RegMem,
        src_size: OperandSize,
        dst: Writable<Reg>,
    ) -> Inst {
        src.assert_regclass_is(RegClass::Int);
        debug_assert!(src_size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        debug_assert!(dst.to_reg().class() == RegClass::Float);
        Inst::GprToXmm {
            op,
            src: GprMem::new(src).unwrap(),
            dst: WritableXmm::from_writable_reg(dst).unwrap(),
            src_size,
        }
    }

    pub(crate) fn xmm_cmp_rm_r(op: SseOpcode, src: RegMem, dst: Reg) -> Inst {
        src.assert_regclass_is(RegClass::Float);
        debug_assert!(dst.class() == RegClass::Float);
        let src = XmmMem::new(src).unwrap();
        let dst = Xmm::new(dst).unwrap();
        Inst::XmmCmpRmR { op, src, dst }
    }

    #[allow(dead_code)]
    pub(crate) fn xmm_min_max_seq(
        size: OperandSize,
        is_min: bool,
        lhs: Reg,
        rhs: Reg,
        dst: Writable<Reg>,
    ) -> Inst {
        debug_assert!(size.is_one_of(&[OperandSize::Size32, OperandSize::Size64]));
        debug_assert_eq!(lhs.class(), RegClass::Float);
        debug_assert_eq!(rhs.class(), RegClass::Float);
        debug_assert_eq!(dst.to_reg().class(), RegClass::Float);
        Inst::XmmMinMaxSeq {
            size,
            is_min,
            lhs: Xmm::new(lhs).unwrap(),
            rhs: Xmm::new(rhs).unwrap(),
            dst: WritableXmm::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn movzx_rm_r(ext_mode: ExtMode, src: RegMem, dst: Writable<Reg>) -> Inst {
        src.assert_regclass_is(RegClass::Int);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        let src = GprMem::new(src).unwrap();
        let dst = WritableGpr::from_writable_reg(dst).unwrap();
        Inst::MovzxRmR { ext_mode, src, dst }
    }

    pub(crate) fn movsx_rm_r(ext_mode: ExtMode, src: RegMem, dst: Writable<Reg>) -> Inst {
        src.assert_regclass_is(RegClass::Int);
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        let src = GprMem::new(src).unwrap();
        let dst = WritableGpr::from_writable_reg(dst).unwrap();
        Inst::MovsxRmR { ext_mode, src, dst }
    }

    pub(crate) fn mov64_m_r(src: impl Into<SyntheticAmode>, dst: Writable<Reg>) -> Inst {
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        Inst::Mov64MR {
            src: src.into(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn mov_r_m(size: OperandSize, src: Reg, dst: impl Into<SyntheticAmode>) -> Inst {
        debug_assert!(src.class() == RegClass::Int);
        Inst::MovRM {
            size,
            src: Gpr::new(src).unwrap(),
            dst: dst.into(),
        }
    }

    pub(crate) fn lea(addr: impl Into<SyntheticAmode>, dst: Writable<Reg>) -> Inst {
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        Inst::LoadEffectiveAddress {
            addr: addr.into(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn shift_r(
        size: OperandSize,
        kind: ShiftKind,
        num_bits: Imm8Gpr,
        src: Reg,
        dst: Writable<Reg>,
    ) -> Inst {
        if let Imm8Reg::Imm8 { imm: num_bits } = num_bits.clone().to_imm8_reg() {
            debug_assert!(num_bits < size.to_bits());
        }
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        Inst::ShiftR {
            size,
            kind,
            src: Gpr::new(src).unwrap(),
            num_bits,
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    /// Does a comparison of dst - src for operands of size `size`, as stated by the machine
    /// instruction semantics. Be careful with the order of parameters!
    pub(crate) fn cmp_rmi_r(size: OperandSize, src: RegMemImm, dst: Reg) -> Inst {
        src.assert_regclass_is(RegClass::Int);
        debug_assert_eq!(dst.class(), RegClass::Int);
        Inst::CmpRmiR {
            size,
            src: GprMemImm::new(src).unwrap(),
            dst: Gpr::new(dst).unwrap(),
            opcode: CmpOpcode::Cmp,
        }
    }

    pub(crate) fn trap(trap_code: TrapCode) -> Inst {
        Inst::Ud2 { trap_code }
    }

    pub(crate) fn cmove(size: OperandSize, cc: CC, src: RegMem, dst: Writable<Reg>) -> Inst {
        debug_assert!(size.is_one_of(&[
            OperandSize::Size16,
            OperandSize::Size32,
            OperandSize::Size64
        ]));
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        Inst::Cmove {
            size,
            cc,
            consequent: GprMem::new(src).unwrap(),
            alternative: Gpr::new(dst.to_reg()).unwrap(),
            dst: WritableGpr::from_writable_reg(dst).unwrap(),
        }
    }

    pub(crate) fn push64(src: RegMemImm) -> Inst {
        src.assert_regclass_is(RegClass::Int);
        let src = GprMemImm::new(src).unwrap();
        Inst::Push64 { src }
    }

    pub(crate) fn pop64(dst: Writable<Reg>) -> Inst {
        debug_assert!(dst.to_reg().class() == RegClass::Int);
        let dst = WritableGpr::from_writable_reg(dst).unwrap();
        Inst::Pop64 { dst }
    }

    pub(crate) fn call_known(
        dest: ExternalName,
        uses: CallArgList,
        defs: CallRetList,
        clobbers: PRegSet,
        opcode: Opcode,
    ) -> Inst {
        Inst::CallKnown {
            dest,
            info: Box::new(CallInfo {
                uses,
                defs,
                clobbers,
                opcode,
            }),
        }
    }

    pub(crate) fn call_unknown(
        dest: RegMem,
        uses: CallArgList,
        defs: CallRetList,
        clobbers: PRegSet,
        opcode: Opcode,
    ) -> Inst {
        dest.assert_regclass_is(RegClass::Int);
        Inst::CallUnknown {
            dest,
            info: Box::new(CallInfo {
                uses,
                defs,
                clobbers,
                opcode,
            }),
        }
    }

    pub(crate) fn ret(rets: Vec<RetPair>) -> Inst {
        Inst::Ret { rets }
    }

    pub(crate) fn jmp_known(dst: MachLabel) -> Inst {
        Inst::JmpKnown { dst }
    }

    pub(crate) fn jmp_unknown(target: RegMem) -> Inst {
        target.assert_regclass_is(RegClass::Int);
        Inst::JmpUnknown { target }
    }

    pub(crate) fn trap_if(cc: CC, trap_code: TrapCode) -> Inst {
        Inst::TrapIf { cc, trap_code }
    }

    /// Choose which instruction to use for loading a register value from memory. For loads smaller
    /// than 64 bits, this method expects a way to extend the value (i.e. [ExtKind::SignExtend],
    /// [ExtKind::ZeroExtend]); loads with no extension necessary will ignore this.
    pub(crate) fn load(
        ty: Type,
        from_addr: impl Into<SyntheticAmode>,
        to_reg: Writable<Reg>,
        ext_kind: ExtKind,
    ) -> Inst {
        let rc = to_reg.to_reg().class();
        match rc {
            RegClass::Int => {
                let ext_mode = match ty.bytes() {
                    1 => Some(ExtMode::BQ),
                    2 => Some(ExtMode::WQ),
                    4 => Some(ExtMode::LQ),
                    8 => None,
                    _ => unreachable!("the type should never use a scalar load: {}", ty),
                };
                if let Some(ext_mode) = ext_mode {
                    // Values smaller than 64 bits must be extended in some way.
                    match ext_kind {
                        ExtKind::SignExtend => {
                            Inst::movsx_rm_r(ext_mode, RegMem::mem(from_addr), to_reg)
                        }
                        ExtKind::ZeroExtend => {
                            Inst::movzx_rm_r(ext_mode, RegMem::mem(from_addr), to_reg)
                        }
                        ExtKind::None => panic!(
                            "expected an extension kind for extension mode: {:?}",
                            ext_mode
                        ),
                    }
                } else {
                    // 64-bit values can be moved directly.
                    Inst::mov64_m_r(from_addr, to_reg)
                }
            }
            RegClass::Float => {
                let opcode = match ty {
                    types::F32 => SseOpcode::Movss,
                    types::F64 => SseOpcode::Movsd,
                    types::F32X4 => SseOpcode::Movups,
                    types::F64X2 => SseOpcode::Movupd,
                    _ if ty.is_vector() && ty.bits() == 128 => SseOpcode::Movdqu,
                    _ => unimplemented!("unable to load type: {}", ty),
                };
                Inst::xmm_unary_rm_r(opcode, RegMem::mem(from_addr), to_reg)
            }
        }
    }

    /// Choose which instruction to use for storing a register value to memory.
    pub(crate) fn store(ty: Type, from_reg: Reg, to_addr: impl Into<SyntheticAmode>) -> Inst {
        let rc = from_reg.class();
        match rc {
            RegClass::Int => Inst::mov_r_m(OperandSize::from_ty(ty), from_reg, to_addr),
            RegClass::Float => {
                let opcode = match ty {
                    types::F32 => SseOpcode::Movss,
                    types::F64 => SseOpcode::Movsd,
                    types::F32X4 => SseOpcode::Movups,
                    types::F64X2 => SseOpcode::Movupd,
                    _ if ty.is_vector() && ty.bits() == 128 => SseOpcode::Movdqu,
                    _ => unimplemented!("unable to store type: {}", ty),
                };
                Inst::xmm_mov_r_m(opcode, from_reg, to_addr)
            }
        }
    }
}

// Inst helpers.

impl Inst {
    /// In certain cases, instructions of this format can act as a definition of an XMM register,
    /// producing a value that is independent of its initial value.
    ///
    /// For example, a vector equality comparison (`cmppd` or `cmpps`) that compares a register to
    /// itself will generate all ones as a result, regardless of its value. From the register
    /// allocator's point of view, we should (i) record the first register, which is normally a
    /// mod, as a def instead; and (ii) not record the second register as a use, because it is the
    /// same as the first register (already handled).
    fn produces_const(&self) -> bool {
        match self {
            Self::AluRmiR { op, src1, src2, .. } => {
                src2.clone().to_reg_mem_imm().to_reg() == Some(src1.to_reg())
                    && (*op == AluRmiROpcode::Xor || *op == AluRmiROpcode::Sub)
            }

            Self::XmmRmR { op, src1, src2, .. } => {
                src2.clone().to_reg_mem().to_reg() == Some(src1.to_reg())
                    && (*op == SseOpcode::Xorps
                        || *op == SseOpcode::Xorpd
                        || *op == SseOpcode::Pxor
                        || *op == SseOpcode::Pcmpeqb
                        || *op == SseOpcode::Pcmpeqw
                        || *op == SseOpcode::Pcmpeqd
                        || *op == SseOpcode::Pcmpeqq)
            }

            _ => false,
        }
    }
}

//=============================================================================
// Instructions: printing

impl PrettyPrint for Inst {
    fn pretty_print(&self, _size: u8, allocs: &mut AllocationConsumer<'_>) -> String {
        fn ljustify(s: String) -> String {
            let w = 7;
            if s.len() >= w {
                s
            } else {
                let need = usize::min(w, w - s.len());
                s + &format!("{nil: <width$}", nil = "", width = need)
            }
        }

        fn ljustify2(s1: String, s2: String) -> String {
            ljustify(s1 + &s2)
        }

        fn suffix_lq(size: OperandSize) -> String {
            match size {
                OperandSize::Size32 => "l",
                OperandSize::Size64 => "q",
                _ => unreachable!(),
            }
            .to_string()
        }

        fn suffix_lqb(size: OperandSize) -> String {
            match size {
                OperandSize::Size32 => "l",
                OperandSize::Size64 => "q",
                _ => unreachable!(),
            }
            .to_string()
        }

        fn suffix_bwlq(size: OperandSize) -> String {
            match size {
                OperandSize::Size8 => "b".to_string(),
                OperandSize::Size16 => "w".to_string(),
                OperandSize::Size32 => "l".to_string(),
                OperandSize::Size64 => "q".to_string(),
            }
        }

        match self {
            Inst::Nop { len } => format!("{} len={}", ljustify("nop".to_string()), len),

            Inst::AluRmiR { size, op, dst, .. } if self.produces_const() => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify2(op.to_string(), suffix_lqb(*size)),
                    dst,
                    dst,
                    dst
                )
            }
            Inst::AluRmiR {
                size,
                op,
                src1,
                src2,
                dst,
            } => {
                let size_bytes = size.to_bytes();
                let src1 = pretty_print_reg(src1.to_reg(), size_bytes, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size_bytes, allocs);
                let src2 = src2.pretty_print(size_bytes, allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify2(op.to_string(), suffix_lqb(*size)),
                    src1,
                    src2,
                    dst
                )
            }
            Inst::AluRM {
                size,
                op,
                src1_dst,
                src2,
            } => {
                let size_bytes = size.to_bytes();
                let src2 = pretty_print_reg(src2.to_reg(), size_bytes, allocs);
                let src1_dst = src1_dst.pretty_print(size_bytes, allocs);
                format!(
                    "{} {}, {}",
                    ljustify2(op.to_string(), suffix_lqb(*size)),
                    src2,
                    src1_dst,
                )
            }
            Inst::UnaryRmR { src, dst, op, size } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                let src = src.pretty_print(size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2(op.to_string(), suffix_bwlq(*size)),
                    src,
                    dst,
                )
            }

            Inst::Not { size, src, dst } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("not".to_string(), suffix_bwlq(*size)),
                    src,
                    dst,
                )
            }

            Inst::Neg { size, src, dst } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("neg".to_string(), suffix_bwlq(*size)),
                    src,
                    dst,
                )
            }

            Inst::Div {
                size,
                signed,
                divisor,
                dividend_lo,
                dividend_hi,
                dst_quotient,
                dst_remainder,
            } => {
                let dividend_lo = pretty_print_reg(dividend_lo.to_reg(), size.to_bytes(), allocs);
                let dst_quotient =
                    pretty_print_reg(dst_quotient.to_reg().to_reg(), size.to_bytes(), allocs);
                let dst_remainder = if size.to_bits() > 8 {
                    pretty_print_reg(dst_remainder.to_reg().to_reg(), size.to_bytes(), allocs)
                } else {
                    "(none)".to_string()
                };
                let dividend_hi = if size.to_bits() > 8 {
                    pretty_print_reg(dividend_hi.to_reg(), size.to_bytes(), allocs)
                } else {
                    "(none)".to_string()
                };
                let divisor = divisor.pretty_print(size.to_bytes(), allocs);
                format!(
                    "{} {}, {}, {}, {}, {}",
                    ljustify(if *signed {
                        "idiv".to_string()
                    } else {
                        "div".into()
                    }),
                    dividend_lo,
                    dividend_hi,
                    divisor,
                    dst_quotient,
                    dst_remainder,
                )
            }

            Inst::MulHi {
                size,
                signed,
                src1,
                src2,
                dst_lo,
                dst_hi,
            } => {
                let src1 = pretty_print_reg(src1.to_reg(), size.to_bytes(), allocs);
                let dst_lo = pretty_print_reg(dst_lo.to_reg().to_reg(), size.to_bytes(), allocs);
                let dst_hi = pretty_print_reg(dst_hi.to_reg().to_reg(), size.to_bytes(), allocs);
                let src2 = src2.pretty_print(size.to_bytes(), allocs);
                format!(
                    "{} {}, {}, {}, {}",
                    ljustify(if *signed {
                        "imul".to_string()
                    } else {
                        "mul".to_string()
                    }),
                    src1,
                    src2,
                    dst_lo,
                    dst_hi,
                )
            }

            Inst::CheckedDivOrRemSeq {
                kind,
                size,
                divisor,
                dividend_lo,
                dividend_hi,
                dst_quotient,
                dst_remainder,
                tmp,
            } => {
                let dividend_lo = pretty_print_reg(dividend_lo.to_reg(), size.to_bytes(), allocs);
                let dividend_hi = pretty_print_reg(dividend_hi.to_reg(), size.to_bytes(), allocs);
                let divisor = pretty_print_reg(divisor.to_reg(), size.to_bytes(), allocs);
                let dst_quotient =
                    pretty_print_reg(dst_quotient.to_reg().to_reg(), size.to_bytes(), allocs);
                let dst_remainder =
                    pretty_print_reg(dst_remainder.to_reg().to_reg(), size.to_bytes(), allocs);
                let tmp = tmp
                    .map(|tmp| pretty_print_reg(tmp.to_reg().to_reg(), size.to_bytes(), allocs))
                    .unwrap_or("(none)".to_string());
                format!(
                    "{} {}, {}, {}, {}, {}, tmp={}",
                    match kind {
                        DivOrRemKind::SignedDiv => "sdiv_seq",
                        DivOrRemKind::UnsignedDiv => "udiv_seq",
                        DivOrRemKind::SignedRem => "srem_seq",
                        DivOrRemKind::UnsignedRem => "urem_seq",
                    },
                    dividend_lo,
                    dividend_hi,
                    divisor,
                    dst_quotient,
                    dst_remainder,
                    tmp,
                )
            }

            Inst::SignExtendData { size, src, dst } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    match size {
                        OperandSize::Size8 => "cbw",
                        OperandSize::Size16 => "cwd",
                        OperandSize::Size32 => "cdq",
                        OperandSize::Size64 => "cqo",
                    },
                    src,
                    dst,
                )
            }

            Inst::XmmUnaryRmR { op, src, dst, .. } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), op.src_size(), allocs);
                let src = src.pretty_print(op.src_size(), allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::XmmUnaryRmRImm {
                op, src, dst, imm, ..
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), op.src_size(), allocs);
                let src = src.pretty_print(op.src_size(), allocs);
                format!("{} ${}, {}, {}", ljustify(op.to_string()), imm, src, dst)
            }

            Inst::XmmUnaryRmREvex { op, src, dst, .. } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src = src.pretty_print(8, allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::XmmMovRM { op, src, dst, .. } => {
                let src = pretty_print_reg(*src, 8, allocs);
                let dst = dst.pretty_print(8, allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::XmmRmR { op, dst, .. } if self.produces_const() => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} {}, {}, {}", ljustify(op.to_string()), dst, dst, dst)
            }

            Inst::XmmRmR {
                op,
                src1,
                src2,
                dst,
                ..
            } => {
                let src1 = pretty_print_reg(src1.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = src2.pretty_print(8, allocs);
                format!("{} {}, {}, {}", ljustify(op.to_string()), src1, src2, dst)
            }

            Inst::XmmRmRBlend {
                op,
                src1,
                src2,
                mask,
                dst,
            } => {
                let src1 = pretty_print_reg(src1.to_reg(), 8, allocs);
                let mask = allocs.next(mask.to_reg());
                let mask = if mask.is_virtual() {
                    format!(" <{}>", show_ireg_sized(mask, 8))
                } else {
                    debug_assert_eq!(mask, regs::xmm0());
                    String::new()
                };
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = src2.pretty_print(8, allocs);
                format!(
                    "{} {}, {}, {}{}",
                    ljustify(op.to_string()),
                    src1,
                    src2,
                    dst,
                    mask
                )
            }

            Inst::XmmRmRVex {
                op,
                src1,
                src2,
                src3,
                dst,
                ..
            } => {
                let src1 = pretty_print_reg(src1.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = pretty_print_reg(src2.to_reg(), 8, allocs);
                let src3 = src3.pretty_print(8, allocs);

                format!(
                    "{} {}, {}, {}, {}",
                    ljustify(op.to_string()),
                    src1,
                    src2,
                    src3,
                    dst
                )
            }

            Inst::XmmRmREvex {
                op,
                src1,
                src2,
                dst,
                ..
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = pretty_print_reg(src2.to_reg(), 8, allocs);
                let src1 = src1.pretty_print(8, allocs);
                format!("{} {}, {}, {}", ljustify(op.to_string()), src1, src2, dst)
            }

            Inst::XmmRmREvex3 {
                op,
                src1,
                src2,
                src3,
                dst,
                ..
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = pretty_print_reg(src2.to_reg(), 8, allocs);
                let src3 = pretty_print_reg(src3.to_reg(), 8, allocs);
                let src1 = src1.pretty_print(8, allocs);
                format!(
                    "{} {}, {}, {}, {}",
                    ljustify(op.to_string()),
                    src1,
                    src2,
                    src3,
                    dst
                )
            }

            Inst::XmmMinMaxSeq {
                lhs,
                rhs,
                dst,
                is_min,
                size,
            } => {
                let rhs = pretty_print_reg(rhs.to_reg(), 8, allocs);
                let lhs = pretty_print_reg(lhs.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify2(
                        if *is_min {
                            "xmm min seq ".to_string()
                        } else {
                            "xmm max seq ".to_string()
                        },
                        format!("f{}", size.to_bits())
                    ),
                    lhs,
                    rhs,
                    dst
                )
            }

            Inst::XmmRmRImm {
                op, dst, imm, size, ..
            } if self.produces_const() => {
                let dst = pretty_print_reg(dst.to_reg(), 8, allocs);
                format!(
                    "{} ${}, {}, {}, {}",
                    ljustify(format!(
                        "{}{}",
                        op.to_string(),
                        if *size == OperandSize::Size64 {
                            ".w"
                        } else {
                            ""
                        }
                    )),
                    imm,
                    dst,
                    dst,
                    dst,
                )
            }

            Inst::XmmRmRImm {
                op,
                src1,
                src2,
                dst,
                imm,
                size,
                ..
            } => {
                let src1 = if op.uses_src1() {
                    pretty_print_reg(*src1, 8, allocs) + ", "
                } else {
                    "".into()
                };
                let dst = pretty_print_reg(dst.to_reg(), 8, allocs);
                let src2 = src2.pretty_print(8, allocs);
                format!(
                    "{} ${}, {}{}, {}",
                    ljustify(format!(
                        "{}{}",
                        op.to_string(),
                        if *size == OperandSize::Size64 {
                            ".w"
                        } else {
                            ""
                        }
                    )),
                    imm,
                    src1,
                    src2,
                    dst,
                )
            }

            Inst::XmmUninitializedValue { dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} {}", ljustify("uninit".into()), dst)
            }

            Inst::XmmToGpr {
                op,
                src,
                dst,
                dst_size,
            } => {
                let dst_size = dst_size.to_bytes();
                let src = pretty_print_reg(src.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size, allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::GprToXmm {
                op,
                src,
                src_size,
                dst,
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src = src.pretty_print(src_size.to_bytes(), allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::XmmCmpRmR { op, src, dst } => {
                let dst = pretty_print_reg(dst.to_reg(), 8, allocs);
                let src = src.pretty_print(8, allocs);
                format!("{} {}, {}", ljustify(op.to_string()), src, dst)
            }

            Inst::CvtUint64ToFloatSeq {
                src,
                dst,
                dst_size,
                tmp_gpr1,
                tmp_gpr2,
                ..
            } => {
                let src = pretty_print_reg(src.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size.to_bytes(), allocs);
                let tmp_gpr1 = pretty_print_reg(tmp_gpr1.to_reg().to_reg(), 8, allocs);
                let tmp_gpr2 = pretty_print_reg(tmp_gpr2.to_reg().to_reg(), 8, allocs);
                format!(
                    "{} {}, {}, {}, {}",
                    ljustify(format!(
                        "u64_to_{}_seq",
                        if *dst_size == OperandSize::Size64 {
                            "f64"
                        } else {
                            "f32"
                        }
                    )),
                    src,
                    dst,
                    tmp_gpr1,
                    tmp_gpr2
                )
            }

            Inst::CvtFloatToSintSeq {
                src,
                dst,
                src_size,
                dst_size,
                tmp_xmm,
                tmp_gpr,
                is_saturating,
            } => {
                let src = pretty_print_reg(src.to_reg(), src_size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size.to_bytes(), allocs);
                let tmp_gpr = pretty_print_reg(tmp_gpr.to_reg().to_reg(), 8, allocs);
                let tmp_xmm = pretty_print_reg(tmp_xmm.to_reg().to_reg(), 8, allocs);
                format!(
                    "{} {}, {}, {}, {}",
                    ljustify(format!(
                        "cvt_float{}_to_sint{}{}_seq",
                        src_size.to_bits(),
                        dst_size.to_bits(),
                        if *is_saturating { "_sat" } else { "" },
                    )),
                    src,
                    dst,
                    tmp_gpr,
                    tmp_xmm,
                )
            }

            Inst::CvtFloatToUintSeq {
                src,
                dst,
                src_size,
                dst_size,
                tmp_gpr,
                tmp_xmm,
                tmp_xmm2,
                is_saturating,
            } => {
                let src = pretty_print_reg(src.to_reg(), src_size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size.to_bytes(), allocs);
                let tmp_gpr = pretty_print_reg(tmp_gpr.to_reg().to_reg(), 8, allocs);
                let tmp_xmm = pretty_print_reg(tmp_xmm.to_reg().to_reg(), 8, allocs);
                let tmp_xmm2 = pretty_print_reg(tmp_xmm2.to_reg().to_reg(), 8, allocs);
                format!(
                    "{} {}, {}, {}, {}, {}",
                    ljustify(format!(
                        "cvt_float{}_to_uint{}{}_seq",
                        src_size.to_bits(),
                        dst_size.to_bits(),
                        if *is_saturating { "_sat" } else { "" },
                    )),
                    src,
                    dst,
                    tmp_gpr,
                    tmp_xmm,
                    tmp_xmm2,
                )
            }

            Inst::Imm {
                dst_size,
                simm64,
                dst,
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size.to_bytes(), allocs);
                if *dst_size == OperandSize::Size64 {
                    format!(
                        "{} ${}, {}",
                        ljustify("movabsq".to_string()),
                        *simm64 as i64,
                        dst,
                    )
                } else {
                    format!(
                        "{} ${}, {}",
                        ljustify("movl".to_string()),
                        (*simm64 as u32) as i32,
                        dst,
                    )
                }
            }

            Inst::MovRR { size, src, dst } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("mov".to_string(), suffix_lq(*size)),
                    src,
                    dst
                )
            }

            Inst::MovFromPReg { src, dst } => {
                allocs.next_fixed_nonallocatable(*src);
                let src: Reg = (*src).into();
                let src = regs::show_ireg_sized(src, 8);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} {}, {}", ljustify("movq".to_string()), src, dst)
            }

            Inst::MovToPReg { src, dst } => {
                let src = pretty_print_reg(src.to_reg(), 8, allocs);
                allocs.next_fixed_nonallocatable(*dst);
                let dst: Reg = (*dst).into();
                let dst = regs::show_ireg_sized(dst, 8);
                format!("{} {}, {}", ljustify("movq".to_string()), src, dst)
            }

            Inst::MovzxRmR {
                ext_mode, src, dst, ..
            } => {
                let dst_size = if *ext_mode == ExtMode::LQ {
                    4
                } else {
                    ext_mode.dst_size()
                };
                let dst = pretty_print_reg(dst.to_reg().to_reg(), dst_size, allocs);
                let src = src.pretty_print(ext_mode.src_size(), allocs);
                if *ext_mode == ExtMode::LQ {
                    format!("{} {}, {}", ljustify("movl".to_string()), src, dst)
                } else {
                    format!(
                        "{} {}, {}",
                        ljustify2("movz".to_string(), ext_mode.to_string()),
                        src,
                        dst,
                    )
                }
            }

            Inst::Mov64MR { src, dst, .. } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src = src.pretty_print(8, allocs);
                format!("{} {}, {}", ljustify("movq".to_string()), src, dst)
            }

            Inst::LoadEffectiveAddress { addr, dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let addr = addr.pretty_print(8, allocs);
                format!("{} {}, {}", ljustify("lea".to_string()), addr, dst)
            }

            Inst::MovsxRmR {
                ext_mode, src, dst, ..
            } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), ext_mode.dst_size(), allocs);
                let src = src.pretty_print(ext_mode.src_size(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("movs".to_string(), ext_mode.to_string()),
                    src,
                    dst
                )
            }

            Inst::MovRM { size, src, dst, .. } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = dst.pretty_print(size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("mov".to_string(), suffix_bwlq(*size)),
                    src,
                    dst
                )
            }

            Inst::ShiftR {
                size,
                kind,
                num_bits,
                src,
                dst,
                ..
            } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                match num_bits.clone().to_imm8_reg() {
                    Imm8Reg::Reg { reg } => {
                        let reg = pretty_print_reg(reg, 1, allocs);
                        format!(
                            "{} {}, {}, {}",
                            ljustify2(kind.to_string(), suffix_bwlq(*size)),
                            reg,
                            src,
                            dst,
                        )
                    }

                    Imm8Reg::Imm8 { imm: num_bits } => format!(
                        "{} ${}, {}, {}",
                        ljustify2(kind.to_string(), suffix_bwlq(*size)),
                        num_bits,
                        src,
                        dst,
                    ),
                }
            }

            Inst::XmmRmiReg {
                opcode,
                src1,
                src2,
                dst,
                ..
            } => {
                let src1 = pretty_print_reg(src1.to_reg(), 8, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let src2 = src2.pretty_print(8, allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify(opcode.to_string()),
                    src1,
                    src2,
                    dst,
                )
            }

            Inst::CmpRmiR {
                size,
                src,
                dst,
                opcode,
            } => {
                let dst = pretty_print_reg(dst.to_reg(), size.to_bytes(), allocs);
                let src = src.pretty_print(size.to_bytes(), allocs);
                let op = match opcode {
                    CmpOpcode::Cmp => "cmp",
                    CmpOpcode::Test => "test",
                };
                format!(
                    "{} {}, {}",
                    ljustify2(op.to_string(), suffix_bwlq(*size)),
                    src,
                    dst,
                )
            }

            Inst::Setcc { cc, dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 1, allocs);
                format!("{} {}", ljustify2("set".to_string(), cc.to_string()), dst)
            }

            Inst::Bswap { size, src, dst } => {
                let src = pretty_print_reg(src.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                format!(
                    "{} {}, {}",
                    ljustify2("bswap".to_string(), suffix_bwlq(*size)),
                    src,
                    dst
                )
            }

            Inst::Cmove {
                size,
                cc,
                consequent,
                alternative,
                dst,
            } => {
                let alternative = pretty_print_reg(alternative.to_reg(), size.to_bytes(), allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size.to_bytes(), allocs);
                let consequent = consequent.pretty_print(size.to_bytes(), allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify(format!("cmov{}{}", cc.to_string(), suffix_bwlq(*size))),
                    consequent,
                    alternative,
                    dst,
                )
            }

            Inst::XmmCmove {
                ty,
                cc,
                consequent,
                alternative,
                dst,
                ..
            } => {
                let size = u8::try_from(ty.bytes()).unwrap();
                let alternative = pretty_print_reg(alternative.to_reg(), size, allocs);
                let dst = pretty_print_reg(dst.to_reg().to_reg(), size, allocs);
                let consequent = consequent.pretty_print(size, allocs);
                format!(
                    "mov {}, {}; j{} $next; mov{} {}, {}; $next: ",
                    cc.invert().to_string(),
                    match *ty {
                        types::F64 => "sd",
                        types::F32 => "ss",
                        types::F32X4 => "aps",
                        types::F64X2 => "apd",
                        _ => "dqa",
                    },
                    consequent,
                    dst,
                    alternative,
                    dst,
                )
            }

            Inst::Push64 { src } => {
                let src = src.pretty_print(8, allocs);
                format!("{} {}", ljustify("pushq".to_string()), src)
            }

            Inst::StackProbeLoop {
                tmp,
                frame_size,
                guard_size,
            } => {
                let tmp = pretty_print_reg(tmp.to_reg(), 8, allocs);
                format!(
                    "{} {}, frame_size={}, guard_size={}",
                    ljustify("stack_probe_loop".to_string()),
                    tmp,
                    frame_size,
                    guard_size
                )
            }

            Inst::Pop64 { dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} {}", ljustify("popq".to_string()), dst)
            }

            Inst::CallKnown { dest, .. } => {
                format!("{} {:?}", ljustify("call".to_string()), dest)
            }

            Inst::CallUnknown { dest, .. } => {
                let dest = dest.pretty_print(8, allocs);
                format!("{} *{}", ljustify("call".to_string()), dest)
            }

            Inst::Args { args } => {
                let mut s = "args".to_string();
                for arg in args {
                    use std::fmt::Write;
                    let preg = regs::show_reg(arg.preg);
                    let def = pretty_print_reg(arg.vreg.to_reg(), 8, allocs);
                    write!(&mut s, " {}={}", def, preg).unwrap();
                }
                s
            }

            Inst::Ret { .. } => "ret".to_string(),

            Inst::JmpKnown { dst } => {
                format!("{} {}", ljustify("jmp".to_string()), dst.to_string())
            }

            Inst::JmpIf { cc, taken } => format!(
                "{} {}",
                ljustify2("j".to_string(), cc.to_string()),
                taken.to_string(),
            ),

            Inst::JmpCond {
                cc,
                taken,
                not_taken,
            } => format!(
                "{} {}; j {}",
                ljustify2("j".to_string(), cc.to_string()),
                taken.to_string(),
                not_taken.to_string()
            ),

            Inst::JmpTableSeq {
                idx, tmp1, tmp2, ..
            } => {
                let idx = pretty_print_reg(*idx, 8, allocs);
                let tmp1 = pretty_print_reg(tmp1.to_reg(), 8, allocs);
                let tmp2 = pretty_print_reg(tmp2.to_reg(), 8, allocs);
                format!(
                    "{} {}, {}, {}",
                    ljustify("br_table".into()),
                    idx,
                    tmp1,
                    tmp2
                )
            }

            Inst::JmpUnknown { target } => {
                let target = target.pretty_print(8, allocs);
                format!("{} *{}", ljustify("jmp".to_string()), target)
            }

            Inst::TrapIf { cc, trap_code, .. } => {
                format!("j{} ; ud2 {} ;", cc.invert().to_string(), trap_code)
            }

            Inst::TrapIfAnd {
                cc1,
                cc2,
                trap_code,
                ..
            } => {
                format!(
                    "trap_if_and {}, {}, {}",
                    cc1.invert().to_string(),
                    cc2.invert().to_string(),
                    trap_code
                )
            }

            Inst::TrapIfOr {
                cc1,
                cc2,
                trap_code,
                ..
            } => {
                format!(
                    "trap_if_or {}, {}, {}",
                    cc1.to_string(),
                    cc2.invert().to_string(),
                    trap_code
                )
            }

            Inst::LoadExtName {
                dst, name, offset, ..
            } => {
                let dst = pretty_print_reg(dst.to_reg(), 8, allocs);
                format!(
                    "{} {}+{}, {}",
                    ljustify("load_ext_name".into()),
                    name.display(None),
                    offset,
                    dst,
                )
            }

            Inst::LockCmpxchg {
                ty,
                replacement,
                expected,
                mem,
                dst_old,
                ..
            } => {
                let size = ty.bytes() as u8;
                let replacement = pretty_print_reg(*replacement, size, allocs);
                let expected = pretty_print_reg(*expected, size, allocs);
                let dst_old = pretty_print_reg(dst_old.to_reg(), size, allocs);
                let mem = mem.pretty_print(size, allocs);
                format!(
                    "lock cmpxchg{} {}, {}, expected={}, dst_old={}",
                    suffix_bwlq(OperandSize::from_bytes(size as u32)),
                    replacement,
                    mem,
                    expected,
                    dst_old,
                )
            }

            Inst::AtomicRmwSeq { ty, op, .. } => {
                format!(
                    "atomically {{ {}_bits_at_[%r9]) {:?}= %r10; %rax = old_value_at_[%r9]; %r11, %rflags = trash }}",
                    ty.bits(), op)
            }

            Inst::Fence { kind } => match kind {
                FenceKind::MFence => "mfence".to_string(),
                FenceKind::LFence => "lfence".to_string(),
                FenceKind::SFence => "sfence".to_string(),
            },

            Inst::VirtualSPOffsetAdj { offset } => format!("virtual_sp_offset_adjust {}", offset),

            Inst::Hlt => "hlt".into(),

            Inst::Ud2 { trap_code } => format!("ud2 {}", trap_code),

            Inst::ElfTlsGetAddr { ref symbol, dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} = elf_tls_get_addr {:?}", dst, symbol)
            }

            Inst::MachOTlsGetAddr { ref symbol, dst } => {
                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                format!("{} = macho_tls_get_addr {:?}", dst, symbol)
            }

            Inst::CoffTlsGetAddr {
                ref symbol,
                dst,
                tmp,
            } => {
                use std::fmt::Write;

                let dst = pretty_print_reg(dst.to_reg().to_reg(), 8, allocs);
                let tmp = allocs.next(tmp.to_reg().to_reg());

                let mut s = format!("{} = coff_tls_get_addr {:?}", dst, symbol);
                if tmp.is_virtual() {
                    write!(&mut s, ", {}", show_ireg_sized(tmp, 8)).unwrap();
                };

                s
            }

            Inst::Unwind { inst } => {
                format!("unwind {:?}", inst)
            }

            Inst::DummyUse { reg } => {
                let reg = pretty_print_reg(*reg, 8, allocs);
                format!("dummy_use {}", reg)
            }
        }
    }
}

impl fmt::Debug for Inst {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}",
            self.pretty_print_inst(&[], &mut Default::default())
        )
    }
}

fn x64_get_operands<F: Fn(VReg) -> VReg>(inst: &Inst, collector: &mut OperandCollector<'_, F>) {
    // FIXME: remove all remaining `mod` operands here to get to pure
    // SSA.

    // Note: because we need to statically know the indices of each
    // reg in the operands list in order to fetch its allocation
    // later, we put the variable-operand-count bits (the RegMem,
    // RegMemImm, etc args) last. regalloc2 doesn't care what order
    // the operands come in; they can be freely reordered.

    // N.B.: we MUST keep the below in careful sync with (i) emission,
    // in `emit.rs`, and (ii) pretty-printing, in the `pretty_print`
    // method above.
    match inst {
        Inst::AluRmiR {
            src1, src2, dst, ..
        } => {
            if inst.produces_const() {
                collector.reg_def(dst.to_writable_reg());
            } else {
                collector.reg_use(src1.to_reg());
                collector.reg_reuse_def(dst.to_writable_reg(), 0);
                src2.get_operands(collector);
            }
        }
        Inst::AluRM { src1_dst, src2, .. } => {
            collector.reg_use(src2.to_reg());
            src1_dst.get_operands(collector);
        }
        Inst::Not { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
        }
        Inst::Neg { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
        }
        Inst::Div {
            divisor,
            dividend_lo,
            dividend_hi,
            dst_quotient,
            dst_remainder,
            size,
            ..
        } => {
            collector.reg_fixed_use(dividend_lo.to_reg(), regs::rax());
            collector.reg_fixed_def(dst_quotient.to_writable_reg(), regs::rax());
            if size.to_bits() > 8 {
                collector.reg_fixed_def(dst_remainder.to_writable_reg(), regs::rdx());
                collector.reg_fixed_use(dividend_hi.to_reg(), regs::rdx());
            }
            divisor.get_operands(collector);
        }
        Inst::MulHi {
            src1,
            src2,
            dst_lo,
            dst_hi,
            ..
        } => {
            collector.reg_fixed_use(src1.to_reg(), regs::rax());
            collector.reg_fixed_def(dst_lo.to_writable_reg(), regs::rax());
            collector.reg_fixed_def(dst_hi.to_writable_reg(), regs::rdx());
            src2.get_operands(collector);
        }
        Inst::CheckedDivOrRemSeq {
            divisor,
            dividend_lo,
            dividend_hi,
            dst_quotient,
            dst_remainder,
            tmp,
            ..
        } => {
            collector.reg_fixed_use(dividend_lo.to_reg(), regs::rax());
            collector.reg_fixed_use(dividend_hi.to_reg(), regs::rdx());
            collector.reg_use(divisor.to_reg());
            collector.reg_fixed_def(dst_quotient.to_writable_reg(), regs::rax());
            collector.reg_fixed_def(dst_remainder.to_writable_reg(), regs::rdx());
            if let Some(tmp) = tmp {
                // Early def so that the temporary register does not
                // conflict with inputs or outputs.
                collector.reg_early_def(tmp.to_writable_reg());
            }
        }
        Inst::SignExtendData { size, src, dst } => {
            match size {
                OperandSize::Size8 => {
                    // Note `rax` on both src and dest: 8->16 extend
                    // does AL -> AX.
                    collector.reg_fixed_use(src.to_reg(), regs::rax());
                    collector.reg_fixed_def(dst.to_writable_reg(), regs::rax());
                }
                _ => {
                    // All other widths do RAX -> RDX (AX -> DX:AX,
                    // EAX -> EDX:EAX).
                    collector.reg_fixed_use(src.to_reg(), regs::rax());
                    collector.reg_fixed_def(dst.to_writable_reg(), regs::rdx());
                }
            }
        }
        Inst::UnaryRmR { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::XmmUnaryRmR { src, dst, .. }
        | Inst::XmmUnaryRmREvex { src, dst, .. }
        | Inst::XmmUnaryRmRImm { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::XmmRmR {
            src1, src2, dst, ..
        } => {
            if inst.produces_const() {
                collector.reg_def(dst.to_writable_reg());
            } else {
                collector.reg_use(src1.to_reg());
                collector.reg_reuse_def(dst.to_writable_reg(), 0);
                src2.get_operands(collector);
            }
        }
        Inst::XmmRmRBlend {
            src1,
            src2,
            mask,
            dst,
            op,
        } => {
            assert!(
                *op == SseOpcode::Blendvpd
                    || *op == SseOpcode::Blendvps
                    || *op == SseOpcode::Pblendvb
            );
            collector.reg_use(src1.to_reg());
            collector.reg_fixed_use(mask.to_reg(), regs::xmm0());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
            src2.get_operands(collector);
        }
        Inst::XmmRmRVex {
            op,
            src1,
            src2,
            src3,
            dst,
            ..
        } => {
            // Vfmadd uses and defs the dst reg, that is not the case with all
            // AVX's ops, if you're adding a new op, make sure to correctly define
            // register uses.
            assert!(
                *op == AvxOpcode::Vfmadd213ss
                    || *op == AvxOpcode::Vfmadd213sd
                    || *op == AvxOpcode::Vfmadd213ps
                    || *op == AvxOpcode::Vfmadd213pd
            );

            collector.reg_use(src1.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
            collector.reg_use(src2.to_reg());
            src3.get_operands(collector);
        }
        Inst::XmmRmREvex {
            op,
            src1,
            src2,
            dst,
            ..
        } => {
            assert_ne!(*op, Avx512Opcode::Vpermi2b);
            collector.reg_def(dst.to_writable_reg());
            collector.reg_use(src2.to_reg());
            src1.get_operands(collector);
        }
        Inst::XmmRmREvex3 {
            op,
            src1,
            src2,
            src3,
            dst,
            ..
        } => {
            assert_eq!(*op, Avx512Opcode::Vpermi2b);
            collector.reg_reuse_def(dst.to_writable_reg(), 2); // Reuse `src3`.
            collector.reg_use(src2.to_reg());
            collector.reg_use(src3.to_reg());
            src1.get_operands(collector);
        }
        Inst::XmmRmRImm {
            op,
            src1,
            src2,
            dst,
            ..
        } => {
            if inst.produces_const() {
                collector.reg_def(*dst);
            } else if !op.uses_src1() {
                // FIXME: split this instruction into two, so we don't
                // need this awkward src1-is-only-sometimes-an-arg
                // behavior.
                collector.reg_def(*dst);
                src2.get_operands(collector);
            } else {
                collector.reg_use(*src1);
                collector.reg_reuse_def(*dst, 0);
                src2.get_operands(collector);
            }
        }
        Inst::XmmUninitializedValue { dst } => collector.reg_def(dst.to_writable_reg()),
        Inst::XmmMinMaxSeq { lhs, rhs, dst, .. } => {
            collector.reg_use(rhs.to_reg());
            collector.reg_use(lhs.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0); // Reuse RHS.
        }
        Inst::XmmRmiReg {
            src1, src2, dst, ..
        } => {
            collector.reg_use(src1.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0); // Reuse RHS.
            src2.get_operands(collector);
        }
        Inst::XmmMovRM { src, dst, .. } => {
            collector.reg_use(*src);
            dst.get_operands(collector);
        }
        Inst::XmmCmpRmR { src, dst, .. } => {
            collector.reg_use(dst.to_reg());
            src.get_operands(collector);
        }
        Inst::Imm { dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::MovRR { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::MovFromPReg { dst, src } => {
            debug_assert!(dst.to_reg().to_reg().is_virtual());
            collector.reg_fixed_nonallocatable(*src);
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::MovToPReg { dst, src } => {
            debug_assert!(src.to_reg().is_virtual());
            collector.reg_use(src.to_reg());
            collector.reg_fixed_nonallocatable(*dst);
        }
        Inst::XmmToGpr { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::GprToXmm { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::CvtUint64ToFloatSeq {
            src,
            dst,
            tmp_gpr1,
            tmp_gpr2,
            ..
        } => {
            collector.reg_use(src.to_reg());
            collector.reg_early_def(dst.to_writable_reg());
            collector.reg_early_def(tmp_gpr1.to_writable_reg());
            collector.reg_early_def(tmp_gpr2.to_writable_reg());
        }
        Inst::CvtFloatToSintSeq {
            src,
            dst,
            tmp_xmm,
            tmp_gpr,
            ..
        } => {
            collector.reg_use(src.to_reg());
            collector.reg_early_def(dst.to_writable_reg());
            collector.reg_early_def(tmp_gpr.to_writable_reg());
            collector.reg_early_def(tmp_xmm.to_writable_reg());
        }
        Inst::CvtFloatToUintSeq {
            src,
            dst,
            tmp_gpr,
            tmp_xmm,
            tmp_xmm2,
            ..
        } => {
            collector.reg_use(src.to_reg());
            collector.reg_early_def(dst.to_writable_reg());
            collector.reg_early_def(tmp_gpr.to_writable_reg());
            collector.reg_early_def(tmp_xmm.to_writable_reg());
            collector.reg_early_def(tmp_xmm2.to_writable_reg());
        }
        Inst::MovzxRmR { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::Mov64MR { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::LoadEffectiveAddress { addr: src, dst } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::MovsxRmR { src, dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
            src.get_operands(collector);
        }
        Inst::MovRM { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            dst.get_operands(collector);
        }
        Inst::ShiftR {
            num_bits, src, dst, ..
        } => {
            collector.reg_use(src.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
            if let Imm8Reg::Reg { reg } = num_bits.clone().to_imm8_reg() {
                collector.reg_fixed_use(reg, regs::rcx());
            }
        }
        Inst::CmpRmiR { src, dst, .. } => {
            // N.B.: use, not def (cmp doesn't write its result).
            collector.reg_use(dst.to_reg());
            src.get_operands(collector);
        }
        Inst::Setcc { dst, .. } => {
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::Bswap { src, dst, .. } => {
            collector.reg_use(src.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
        }
        Inst::Cmove {
            consequent,
            alternative,
            dst,
            ..
        } => {
            collector.reg_use(alternative.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
            consequent.get_operands(collector);
        }
        Inst::XmmCmove {
            consequent,
            alternative,
            dst,
            ..
        } => {
            collector.reg_use(alternative.to_reg());
            collector.reg_reuse_def(dst.to_writable_reg(), 0);
            consequent.get_operands(collector);
        }
        Inst::Push64 { src } => {
            src.get_operands(collector);
        }
        Inst::Pop64 { dst } => {
            collector.reg_def(dst.to_writable_reg());
        }
        Inst::StackProbeLoop { tmp, .. } => {
            collector.reg_early_def(*tmp);
        }

        Inst::CallKnown { dest, ref info, .. } => {
            // Probestack is special and is only inserted after
            // regalloc, so we do not need to represent its ABI to the
            // register allocator. Assert that we don't alter that
            // arrangement.
            debug_assert_ne!(*dest, ExternalName::LibCall(LibCall::Probestack));
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }

        Inst::CallUnknown { ref info, dest, .. } => {
            dest.get_operands(collector);
            for u in &info.uses {
                collector.reg_fixed_use(u.vreg, u.preg);
            }
            for d in &info.defs {
                collector.reg_fixed_def(d.vreg, d.preg);
            }
            collector.reg_clobbers(info.clobbers);
        }

        Inst::JmpTableSeq {
            ref idx,
            ref tmp1,
            ref tmp2,
            ..
        } => {
            collector.reg_use(*idx);
            collector.reg_early_def(*tmp1);
            collector.reg_early_def(*tmp2);
        }

        Inst::JmpUnknown { target } => {
            target.get_operands(collector);
        }

        Inst::LoadExtName { dst, .. } => {
            collector.reg_def(*dst);
        }

        Inst::LockCmpxchg {
            replacement,
            expected,
            mem,
            dst_old,
            ..
        } => {
            collector.reg_use(*replacement);
            collector.reg_fixed_use(*expected, regs::rax());
            collector.reg_fixed_def(*dst_old, regs::rax());
            mem.get_operands(collector);
        }

        Inst::AtomicRmwSeq {
            operand,
            temp,
            dst_old,
            mem,
            ..
        } => {
            collector.reg_late_use(*operand);
            collector.reg_early_def(*temp);
            // This `fixed_def` is needed because `CMPXCHG` always uses this
            // register implicitly.
            collector.reg_fixed_def(*dst_old, regs::rax());
            mem.get_operands_late(collector)
        }

        Inst::Args { args } => {
            for arg in args {
                collector.reg_fixed_def(arg.vreg, arg.preg);
            }
        }

        Inst::Ret { rets } => {
            // The return value(s) are live-out; we represent this
            // with register uses on the return instruction.
            for ret in rets.iter() {
                collector.reg_fixed_use(ret.vreg, ret.preg);
            }
        }

        Inst::JmpKnown { .. }
        | Inst::JmpIf { .. }
        | Inst::JmpCond { .. }
        | Inst::Nop { .. }
        | Inst::TrapIf { .. }
        | Inst::TrapIfAnd { .. }
        | Inst::TrapIfOr { .. }
        | Inst::VirtualSPOffsetAdj { .. }
        | Inst::Hlt
        | Inst::Ud2 { .. }
        | Inst::Fence { .. } => {
            // No registers are used.
        }

        Inst::ElfTlsGetAddr { dst, .. } | Inst::MachOTlsGetAddr { dst, .. } => {
            collector.reg_fixed_def(dst.to_writable_reg(), regs::rax());
            // All caller-saves are clobbered.
            //
            // We use the SysV calling convention here because the
            // pseudoinstruction (and relocation that it emits) is specific to
            // ELF systems; other x86-64 targets with other conventions (i.e.,
            // Windows) use different TLS strategies.
            let mut clobbers = X64ABIMachineSpec::get_regs_clobbered_by_call(CallConv::SystemV);
            clobbers.remove(regs::gpr_preg(regs::ENC_RAX));
            collector.reg_clobbers(clobbers);
        }

        Inst::CoffTlsGetAddr { dst, tmp, .. } => {
            // We also use the gs register. But that register is not allocatable by the
            // register allocator, so we don't need to mark it as used here.

            // We use %rax to set the address
            collector.reg_fixed_def(dst.to_writable_reg(), regs::rax());

            // We use %rcx as a temporary variable to load the _tls_index
            collector.reg_fixed_def(tmp.to_writable_reg(), regs::rcx());
        }

        Inst::Unwind { .. } => {}

        Inst::DummyUse { reg } => {
            collector.reg_use(*reg);
        }
    }
}

//=============================================================================
// Instructions: misc functions and external interface

impl MachInst for Inst {
    type ABIMachineSpec = X64ABIMachineSpec;

    fn get_operands<F: Fn(VReg) -> VReg>(&self, collector: &mut OperandCollector<'_, F>) {
        x64_get_operands(&self, collector)
    }

    fn is_move(&self) -> Option<(Writable<Reg>, Reg)> {
        match self {
            // Note (carefully!) that a 32-bit mov *isn't* a no-op since it zeroes
            // out the upper 32 bits of the destination.  For example, we could
            // conceivably use `movl %reg, %reg` to zero out the top 32 bits of
            // %reg.
            Self::MovRR { size, src, dst, .. } if *size == OperandSize::Size64 => {
                Some((dst.to_writable_reg(), src.to_reg()))
            }
            // Note as well that MOVS[S|D] when used in the `XmmUnaryRmR` context are pure moves of
            // scalar floating-point values (and annotate `dst` as `def`s to the register allocator)
            // whereas the same operation in a packed context, e.g. `XMM_RM_R`, is used to merge a
            // value into the lowest lane of a vector (not a move).
            Self::XmmUnaryRmR { op, src, dst, .. }
                if *op == SseOpcode::Movss
                    || *op == SseOpcode::Movsd
                    || *op == SseOpcode::Movaps
                    || *op == SseOpcode::Movapd
                    || *op == SseOpcode::Movups
                    || *op == SseOpcode::Movupd
                    || *op == SseOpcode::Movdqa
                    || *op == SseOpcode::Movdqu =>
            {
                if let RegMem::Reg { reg } = src.clone().to_reg_mem() {
                    Some((dst.to_writable_reg(), reg))
                } else {
                    None
                }
            }
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
            // Interesting cases.
            &Self::Ret { .. } => MachTerminator::Ret,
            &Self::JmpKnown { .. } => MachTerminator::Uncond,
            &Self::JmpCond { .. } => MachTerminator::Cond,
            &Self::JmpTableSeq { .. } => MachTerminator::Indirect,
            // All other cases are boring.
            _ => MachTerminator::None,
        }
    }

    fn gen_move(dst_reg: Writable<Reg>, src_reg: Reg, ty: Type) -> Inst {
        trace!(
            "Inst::gen_move {:?} -> {:?} (type: {:?})",
            src_reg,
            dst_reg.to_reg(),
            ty
        );
        let rc_dst = dst_reg.to_reg().class();
        let rc_src = src_reg.class();
        // If this isn't true, we have gone way off the rails.
        debug_assert!(rc_dst == rc_src);
        match rc_dst {
            RegClass::Int => Inst::mov_r_r(OperandSize::Size64, src_reg, dst_reg),
            RegClass::Float => {
                // The Intel optimization manual, in "3.5.1.13 Zero-Latency MOV Instructions",
                // doesn't include MOVSS/MOVSD as instructions with zero-latency. Use movaps for
                // those, which may write more lanes that we need, but are specified to have
                // zero-latency.
                let opcode = match ty {
                    types::F32 | types::F64 | types::F32X4 => SseOpcode::Movaps,
                    types::F64X2 => SseOpcode::Movapd,
                    _ if ty.is_vector() && ty.bits() == 128 => SseOpcode::Movdqa,
                    _ => unimplemented!("unable to move type: {}", ty),
                };
                Inst::xmm_unary_rm_r(opcode, RegMem::reg(src_reg), dst_reg)
            }
        }
    }

    fn gen_nop(preferred_size: usize) -> Inst {
        Inst::nop(std::cmp::min(preferred_size, 15) as u8)
    }

    fn rc_for_type(ty: Type) -> CodegenResult<(&'static [RegClass], &'static [Type])> {
        match ty {
            types::I8 => Ok((&[RegClass::Int], &[types::I8])),
            types::I16 => Ok((&[RegClass::Int], &[types::I16])),
            types::I32 => Ok((&[RegClass::Int], &[types::I32])),
            types::I64 => Ok((&[RegClass::Int], &[types::I64])),
            types::R32 => panic!("32-bit reftype pointer should never be seen on x86-64"),
            types::R64 => Ok((&[RegClass::Int], &[types::R64])),
            types::F32 => Ok((&[RegClass::Float], &[types::F32])),
            types::F64 => Ok((&[RegClass::Float], &[types::F64])),
            types::I128 => Ok((&[RegClass::Int, RegClass::Int], &[types::I64, types::I64])),
            _ if ty.is_vector() => {
                assert!(ty.bits() <= 128);
                Ok((&[RegClass::Float], &[types::I8X16]))
            }
            types::IFLAGS | types::FFLAGS => Ok((&[RegClass::Int], &[types::I64])),
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

    fn gen_jump(label: MachLabel) -> Inst {
        Inst::jmp_known(label)
    }

    fn gen_constant<F: FnMut(Type) -> Writable<Reg>>(
        to_regs: ValueRegs<Writable<Reg>>,
        value: u128,
        ty: Type,
        mut alloc_tmp: F,
    ) -> SmallVec<[Self; 4]> {
        let mut ret = SmallVec::new();
        if ty == types::I128 {
            let lo = value as u64;
            let hi = (value >> 64) as u64;
            let lo_reg = to_regs.regs()[0];
            let hi_reg = to_regs.regs()[1];
            if lo == 0 {
                ret.push(Inst::alu_rmi_r(
                    OperandSize::Size64,
                    AluRmiROpcode::Xor,
                    RegMemImm::reg(lo_reg.to_reg()),
                    lo_reg,
                ));
            } else {
                ret.push(Inst::imm(OperandSize::Size64, lo, lo_reg));
            }
            if hi == 0 {
                ret.push(Inst::alu_rmi_r(
                    OperandSize::Size64,
                    AluRmiROpcode::Xor,
                    RegMemImm::reg(hi_reg.to_reg()),
                    hi_reg,
                ));
            } else {
                ret.push(Inst::imm(OperandSize::Size64, hi, hi_reg));
            }
        } else {
            let to_reg = to_regs
                .only_reg()
                .expect("multi-reg values not supported on x64");
            if ty == types::F32 {
                if value == 0 {
                    ret.push(Inst::xmm_rm_r(
                        SseOpcode::Xorps,
                        RegMem::reg(to_reg.to_reg()),
                        to_reg,
                    ));
                } else {
                    let tmp = alloc_tmp(types::I32);
                    ret.push(Inst::imm(OperandSize::Size32, value as u64, tmp));

                    ret.push(Inst::gpr_to_xmm(
                        SseOpcode::Movd,
                        RegMem::reg(tmp.to_reg()),
                        OperandSize::Size32,
                        to_reg,
                    ));
                }
            } else if ty == types::F64 {
                if value == 0 {
                    ret.push(Inst::xmm_rm_r(
                        SseOpcode::Xorpd,
                        RegMem::reg(to_reg.to_reg()),
                        to_reg,
                    ));
                } else {
                    let tmp = alloc_tmp(types::I64);
                    ret.push(Inst::imm(OperandSize::Size64, value as u64, tmp));

                    ret.push(Inst::gpr_to_xmm(
                        SseOpcode::Movq,
                        RegMem::reg(tmp.to_reg()),
                        OperandSize::Size64,
                        to_reg,
                    ));
                }
            } else {
                // Must be an integer type.
                debug_assert!(
                    ty == types::I8
                        || ty == types::I16
                        || ty == types::I32
                        || ty == types::I64
                        || ty == types::R32
                        || ty == types::R64
                );
                // Immediates must be 32 or 64 bits.
                // Smaller types are widened.
                let size = match OperandSize::from_ty(ty) {
                    OperandSize::Size64 => OperandSize::Size64,
                    _ => OperandSize::Size32,
                };
                if value == 0 {
                    ret.push(Inst::alu_rmi_r(
                        size,
                        AluRmiROpcode::Xor,
                        RegMemImm::reg(to_reg.to_reg()),
                        to_reg,
                    ));
                } else {
                    let value = value as u64;
                    ret.push(Inst::imm(size, value.into(), to_reg));
                }
            }
        }
        ret
    }

    fn gen_dummy_use(reg: Reg) -> Self {
        Inst::DummyUse { reg }
    }

    fn worst_case_size() -> CodeOffset {
        15
    }

    fn ref_type_regclass(_: &settings::Flags) -> RegClass {
        RegClass::Int
    }

    fn is_safepoint(&self) -> bool {
        match self {
            Inst::CallKnown { .. }
            | Inst::CallUnknown { .. }
            | Inst::TrapIf { .. }
            | Inst::Ud2 { .. } => true,
            _ => false,
        }
    }

    type LabelUse = LabelUse;
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
    /// Current source location.
    cur_srcloc: RelSourceLoc,
}

/// Constant state used during emissions of a sequence of instructions.
pub struct EmitInfo {
    pub(super) flags: settings::Flags,
    isa_flags: x64_settings::Flags,
}

impl EmitInfo {
    pub(crate) fn new(flags: settings::Flags, isa_flags: x64_settings::Flags) -> Self {
        Self { flags, isa_flags }
    }
}

impl MachInstEmit for Inst {
    type State = EmitState;
    type Info = EmitInfo;

    fn emit(
        &self,
        allocs: &[Allocation],
        sink: &mut MachBuffer<Inst>,
        info: &Self::Info,
        state: &mut Self::State,
    ) {
        let mut allocs = AllocationConsumer::new(allocs);
        emit::emit(self, &mut allocs, sink, info, state);
    }

    fn pretty_print_inst(&self, allocs: &[Allocation], _: &mut Self::State) -> String {
        PrettyPrint::pretty_print(self, 0, &mut AllocationConsumer::new(allocs))
    }
}

impl MachInstEmitState<Inst> for EmitState {
    fn new(abi: &Callee<X64ABIMachineSpec>) -> Self {
        EmitState {
            virtual_sp_offset: 0,
            nominal_sp_to_fp: abi.frame_size() as i64,
            stack_map: None,
            cur_srcloc: Default::default(),
        }
    }

    fn pre_safepoint(&mut self, stack_map: StackMap) {
        self.stack_map = Some(stack_map);
    }

    fn pre_sourceloc(&mut self, srcloc: RelSourceLoc) {
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
}

/// A label-use (internal relocation) in generated code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUse {
    /// A 32-bit offset from location of relocation itself, added to the existing value at that
    /// location. Used for control flow instructions which consider an offset from the start of the
    /// next instruction (so the size of the payload -- 4 bytes -- is subtracted from the payload).
    JmpRel32,

    /// A 32-bit offset from location of relocation itself, added to the existing value at that
    /// location.
    PCRel32,
}

impl MachInstLabelUse for LabelUse {
    const ALIGN: CodeOffset = 1;

    fn max_pos_range(self) -> CodeOffset {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => 0x7fff_ffff,
        }
    }

    fn max_neg_range(self) -> CodeOffset {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => 0x8000_0000,
        }
    }

    fn patch_size(self) -> CodeOffset {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => 4,
        }
    }

    fn patch(self, buffer: &mut [u8], use_offset: CodeOffset, label_offset: CodeOffset) {
        let pc_rel = (label_offset as i64) - (use_offset as i64);
        debug_assert!(pc_rel <= self.max_pos_range() as i64);
        debug_assert!(pc_rel >= -(self.max_neg_range() as i64));
        let pc_rel = pc_rel as u32;
        match self {
            LabelUse::JmpRel32 => {
                let addend = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                let value = pc_rel.wrapping_add(addend).wrapping_sub(4);
                buffer.copy_from_slice(&value.to_le_bytes()[..]);
            }
            LabelUse::PCRel32 => {
                let addend = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                let value = pc_rel.wrapping_add(addend);
                buffer.copy_from_slice(&value.to_le_bytes()[..]);
            }
        }
    }

    fn supports_veneer(self) -> bool {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => false,
        }
    }

    fn veneer_size(self) -> CodeOffset {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => 0,
        }
    }

    fn generate_veneer(self, _: &mut [u8], _: CodeOffset) -> (CodeOffset, LabelUse) {
        match self {
            LabelUse::JmpRel32 | LabelUse::PCRel32 => {
                panic!("Veneer not supported for JumpRel32 label-use.");
            }
        }
    }

    fn from_reloc(reloc: Reloc, addend: Addend) -> Option<Self> {
        match (reloc, addend) {
            (Reloc::X86CallPCRel4, -4) => Some(LabelUse::JmpRel32),
            _ => None,
        }
    }
}
