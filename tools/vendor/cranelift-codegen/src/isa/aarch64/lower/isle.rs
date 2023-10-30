//! ISLE integration glue code for aarch64 lowering.

// Pull in the ISLE generated code.
pub mod generated_code;

// Types that the generated ISLE code uses via `use super::*`.
use super::{
    writable_zero_reg, zero_reg, AMode, ASIMDFPModImm, ASIMDMovModImm, AtomicRmwOp, BranchTarget,
    CallIndInfo, CallInfo, Cond, CondBrKind, ExtendOp, FPUOpRI, Imm12, ImmLogic, ImmShift,
    Inst as MInst, JTSequenceInfo, MachLabel, MoveWideConst, NarrowValueMode, Opcode, OperandSize,
    PairAMode, Reg, ScalarSize, ShiftOpAndAmt, UImm5, VectorSize, NZCV,
};
use crate::isa::aarch64::settings::Flags as IsaFlags;
use crate::machinst::isle::*;
use crate::settings::Flags;
use crate::{
    binemit::CodeOffset,
    ir::{
        immediates::*, types::*, ExternalName, Inst, InstructionData, MemFlags, TrapCode, Value,
        ValueLabel, ValueList,
    },
    isa::aarch64::inst::aarch64_map_regs,
    isa::aarch64::inst::args::{ShiftOp, ShiftOpShiftImm},
    isa::unwind::UnwindInst,
    machinst::{ty_bits, InsnOutput, LowerCtx},
};
use std::boxed::Box;
use std::convert::TryFrom;
use std::vec::Vec;

type BoxCallInfo = Box<CallInfo>;
type BoxCallIndInfo = Box<CallIndInfo>;
type VecMachLabel = Vec<MachLabel>;
type BoxJTSequenceInfo = Box<JTSequenceInfo>;
type BoxExternalName = Box<ExternalName>;

/// The main entry point for lowering with ISLE.
pub(crate) fn lower<C>(
    lower_ctx: &mut C,
    flags: &Flags,
    isa_flags: &IsaFlags,
    outputs: &[InsnOutput],
    inst: Inst,
) -> Result<(), ()>
where
    C: LowerCtx<I = MInst>,
{
    lower_common(
        lower_ctx,
        flags,
        isa_flags,
        outputs,
        inst,
        |cx, insn| generated_code::constructor_lower(cx, insn),
        aarch64_map_regs,
    )
}

pub struct ExtendedValue {
    val: Value,
    extend: ExtendOp,
}

pub struct SinkableAtomicLoad {
    atomic_load: Inst,
    atomic_addr: Value,
}

impl<C> generated_code::Context for IsleContext<'_, C, Flags, IsaFlags, 6>
where
    C: LowerCtx<I = MInst>,
{
    isle_prelude_methods!();

    fn move_wide_const_from_u64(&mut self, n: u64) -> Option<MoveWideConst> {
        MoveWideConst::maybe_from_u64(n)
    }

    fn move_wide_const_from_negated_u64(&mut self, n: u64) -> Option<MoveWideConst> {
        MoveWideConst::maybe_from_u64(!n)
    }

    fn imm_logic_from_u64(&mut self, n: u64, ty: Type) -> Option<ImmLogic> {
        let ty = if ty.bits() < 32 { I32 } else { ty };
        ImmLogic::maybe_from_u64(n, ty)
    }

    fn imm_logic_from_imm64(&mut self, n: Imm64, ty: Type) -> Option<ImmLogic> {
        self.imm_logic_from_u64(n.bits() as u64, ty)
    }

    fn imm12_from_u64(&mut self, n: u64) -> Option<Imm12> {
        Imm12::maybe_from_u64(n)
    }

    fn imm12_from_negated_u64(&mut self, n: u64) -> Option<Imm12> {
        Imm12::maybe_from_u64((n as i64).wrapping_neg() as u64)
    }

    fn imm_shift_from_u8(&mut self, n: u8) -> ImmShift {
        ImmShift::maybe_from_u64(n.into()).unwrap()
    }

    fn lshl_from_imm64(&mut self, n: Imm64, ty: Type) -> Option<ShiftOpAndAmt> {
        let shiftimm = ShiftOpShiftImm::maybe_from_shift(n.bits() as u64)?;
        let shiftee_bits = ty_bits(ty);
        if shiftee_bits <= std::u8::MAX as usize {
            let shiftimm = shiftimm.mask(shiftee_bits as u8);
            Some(ShiftOpAndAmt::new(ShiftOp::LSL, shiftimm))
        } else {
            None
        }
    }

    fn integral_ty(&mut self, ty: Type) -> Option<Type> {
        match ty {
            I8 | I16 | I32 | I64 | R64 => Some(ty),
            ty if ty.is_bool() => Some(ty),
            _ => None,
        }
    }

    /// This is the fallback case for loading a 64-bit integral constant into a
    /// register.
    ///
    /// The logic here is nontrivial enough that it's not really worth porting
    /// this over to ISLE.
    fn load_constant64_full(&mut self, value: u64) -> Reg {
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

        let rd = self.temp_writable_reg(I64);

        for i in 0..num_half_words {
            let imm16 = (value >> (16 * i)) & 0xffff;
            if imm16 != ignored_halfword {
                if !first_mov_emitted {
                    first_mov_emitted = true;
                    if first_is_inverted {
                        let imm =
                            MoveWideConst::maybe_with_shift(((!imm16) & 0xffff) as u16, i * 16)
                                .unwrap();
                        self.emit(&MInst::MovN { rd, imm, size });
                    } else {
                        let imm = MoveWideConst::maybe_with_shift(imm16 as u16, i * 16).unwrap();
                        self.emit(&MInst::MovZ { rd, imm, size });
                    }
                } else {
                    let imm = MoveWideConst::maybe_with_shift(imm16 as u16, i * 16).unwrap();
                    self.emit(&MInst::MovK { rd, imm, size });
                }
            }
        }

        assert!(first_mov_emitted);

        return self.writable_reg_to_reg(rd);

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
    }

    fn zero_reg(&mut self) -> Reg {
        zero_reg()
    }

    fn extended_value_from_value(&mut self, val: Value) -> Option<ExtendedValue> {
        let (val, extend) =
            super::get_as_extended_value(self.lower_ctx, val, NarrowValueMode::None)?;
        Some(ExtendedValue { val, extend })
    }

    fn put_extended_in_reg(&mut self, reg: &ExtendedValue) -> Reg {
        self.put_in_reg(reg.val)
    }

    fn get_extended_op(&mut self, reg: &ExtendedValue) -> ExtendOp {
        reg.extend
    }

    fn emit(&mut self, inst: &MInst) -> Unit {
        self.emitted_insts.push((inst.clone(), false));
    }

    fn emit_safepoint(&mut self, inst: &MInst) -> Unit {
        self.emitted_insts.push((inst.clone(), true));
    }

    fn cond_br_zero(&mut self, reg: Reg) -> CondBrKind {
        CondBrKind::Zero(reg)
    }

    fn cond_br_cond(&mut self, cond: &Cond) -> CondBrKind {
        CondBrKind::Cond(*cond)
    }

    fn nzcv(&mut self, n: bool, z: bool, c: bool, v: bool) -> NZCV {
        NZCV::new(n, z, c, v)
    }

    fn u8_into_uimm5(&mut self, x: u8) -> UImm5 {
        UImm5::maybe_from_u8(x).unwrap()
    }

    fn u8_into_imm12(&mut self, x: u8) -> Imm12 {
        Imm12::maybe_from_u64(x.into()).unwrap()
    }

    fn writable_zero_reg(&mut self) -> WritableReg {
        writable_zero_reg()
    }

    fn safe_divisor_from_imm64(&mut self, val: Imm64) -> Option<u64> {
        match val.bits() {
            0 | -1 => None,
            n => Some(n as u64),
        }
    }

    fn sinkable_atomic_load(&mut self, val: Value) -> Option<SinkableAtomicLoad> {
        let input = self.lower_ctx.get_value_as_source_or_const(val);
        if let Some((atomic_load, 0)) = input.inst {
            if self.lower_ctx.data(atomic_load).opcode() == Opcode::AtomicLoad {
                let atomic_addr = self.lower_ctx.input_as_value(atomic_load, 0);
                return Some(SinkableAtomicLoad {
                    atomic_load,
                    atomic_addr,
                });
            }
        }
        None
    }

    fn sink_atomic_load(&mut self, load: &SinkableAtomicLoad) -> Reg {
        self.lower_ctx.sink_inst(load.atomic_load);
        self.put_in_reg(load.atomic_addr)
    }

    fn shift_mask(&mut self, ty: Type) -> ImmLogic {
        let mask = (ty.bits() - 1) as u64;
        ImmLogic::maybe_from_u64(mask, I32).unwrap()
    }

    fn imm_shift_from_imm64(&mut self, val: Imm64, ty: Type) -> Option<ImmShift> {
        let imm_value = (val.bits() as u64) & ((ty.bits() - 1) as u64);
        ImmShift::maybe_from_u64(imm_value)
    }

    fn u64_into_imm_logic(&mut self, ty: Type, val: u64) -> ImmLogic {
        ImmLogic::maybe_from_u64(val, ty).unwrap()
    }

    fn negate_imm_shift(&mut self, ty: Type, mut imm: ImmShift) -> ImmShift {
        let size = u8::try_from(ty.bits()).unwrap();
        imm.imm = size.wrapping_sub(imm.value());
        imm.imm &= size - 1;
        imm
    }

    fn rotr_mask(&mut self, ty: Type) -> ImmLogic {
        ImmLogic::maybe_from_u64((ty.bits() - 1) as u64, I32).unwrap()
    }

    fn rotr_opposite_amount(&mut self, ty: Type, val: ImmShift) -> ImmShift {
        let amount = val.value() & u8::try_from(ty.bits() - 1).unwrap();
        ImmShift::maybe_from_u64(u64::from(ty.bits()) - u64::from(amount)).unwrap()
    }
}
