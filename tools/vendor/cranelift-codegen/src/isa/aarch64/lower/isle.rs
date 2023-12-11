//! ISLE integration glue code for aarch64 lowering.

// Pull in the ISLE generated code.
pub mod generated_code;
use generated_code::Context;
use smallvec::SmallVec;

// Types that the generated ISLE code uses via `use super::*`.
use super::{
    fp_reg, lower_condcode, lower_constant_f128, lower_constant_f32, lower_constant_f64,
    lower_fp_condcode, stack_reg, writable_link_reg, writable_zero_reg, zero_reg, AMode,
    ASIMDFPModImm, ASIMDMovModImm, BranchTarget, CallIndInfo, CallInfo, Cond, CondBrKind, ExtendOp,
    FPUOpRI, FPUOpRIMod, FloatCC, Imm12, ImmLogic, ImmShift, Inst as MInst, IntCC, JTSequenceInfo,
    MachLabel, MemLabel, MoveWideConst, MoveWideOp, NarrowValueMode, Opcode, OperandSize,
    PairAMode, Reg, SImm9, ScalarSize, ShiftOpAndAmt, UImm12Scaled, UImm5, VecMisc2, VectorSize,
    NZCV,
};
use crate::ir::condcodes;
use crate::isa::aarch64::inst::{FPULeftShiftImm, FPURightShiftImm};
use crate::isa::aarch64::lower::{lower_address, lower_pair_address, lower_splat_const};
use crate::isa::aarch64::settings::Flags as IsaFlags;
use crate::machinst::valueregs;
use crate::machinst::{isle::*, InputSourceInst};
use crate::settings::Flags;
use crate::{
    binemit::CodeOffset,
    ir::{
        immediates::*, types::*, AtomicRmwOp, ExternalName, Inst, InstructionData, MemFlags,
        TrapCode, Value, ValueList,
    },
    isa::aarch64::abi::AArch64Caller,
    isa::aarch64::inst::args::{ShiftOp, ShiftOpShiftImm},
    isa::unwind::UnwindInst,
    machinst::{
        abi::ArgPair, ty_bits, InsnOutput, Lower, MachInst, VCodeConstant, VCodeConstantData,
    },
};
use crate::{isle_common_prelude_methods, isle_lower_prelude_methods};
use regalloc2::PReg;
use std::boxed::Box;
use std::convert::TryFrom;
use std::vec::Vec;
use target_lexicon::Triple;

type BoxCallInfo = Box<CallInfo>;
type BoxCallIndInfo = Box<CallIndInfo>;
type VecMachLabel = Vec<MachLabel>;
type BoxJTSequenceInfo = Box<JTSequenceInfo>;
type BoxExternalName = Box<ExternalName>;
type VecArgPair = Vec<ArgPair>;

/// The main entry point for lowering with ISLE.
pub(crate) fn lower(
    lower_ctx: &mut Lower<MInst>,
    triple: &Triple,
    flags: &Flags,
    isa_flags: &IsaFlags,
    outputs: &[InsnOutput],
    inst: Inst,
) -> Result<(), ()> {
    lower_common(
        lower_ctx,
        triple,
        flags,
        isa_flags,
        outputs,
        inst,
        |cx, insn| generated_code::constructor_lower(cx, insn),
    )
}

pub(crate) fn lower_branch(
    lower_ctx: &mut Lower<MInst>,
    triple: &Triple,
    flags: &Flags,
    isa_flags: &IsaFlags,
    branch: Inst,
    targets: &[MachLabel],
) -> Result<(), ()> {
    lower_common(
        lower_ctx,
        triple,
        flags,
        isa_flags,
        &[],
        branch,
        |cx, insn| generated_code::constructor_lower_branch(cx, insn, &targets.to_vec()),
    )
}

pub struct ExtendedValue {
    val: Value,
    extend: ExtendOp,
}

impl IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_prelude_method_helpers!(AArch64Caller);
}

impl Context for IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_lower_prelude_methods!();
    isle_prelude_caller_methods!(crate::isa::aarch64::abi::AArch64MachineDeps, AArch64Caller);

    fn sign_return_address_disabled(&mut self) -> Option<()> {
        if self.isa_flags.sign_return_address() {
            None
        } else {
            Some(())
        }
    }

    fn use_lse(&mut self, _: Inst) -> Option<()> {
        if self.isa_flags.has_lse() {
            Some(())
        } else {
            None
        }
    }

    fn imm_logic_from_u64(&mut self, ty: Type, n: u64) -> Option<ImmLogic> {
        ImmLogic::maybe_from_u64(n, ty)
    }

    fn imm_logic_from_imm64(&mut self, ty: Type, n: Imm64) -> Option<ImmLogic> {
        let ty = if ty.bits() < 32 { I32 } else { ty };
        self.imm_logic_from_u64(ty, n.bits() as u64)
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

    fn lshr_from_u64(&mut self, ty: Type, n: u64) -> Option<ShiftOpAndAmt> {
        let shiftimm = ShiftOpShiftImm::maybe_from_shift(n)?;
        if let Ok(bits) = u8::try_from(ty_bits(ty)) {
            let shiftimm = shiftimm.mask(bits);
            Some(ShiftOpAndAmt::new(ShiftOp::LSR, shiftimm))
        } else {
            None
        }
    }

    fn lshl_from_imm64(&mut self, ty: Type, n: Imm64) -> Option<ShiftOpAndAmt> {
        self.lshl_from_u64(ty, n.bits() as u64)
    }

    fn lshl_from_u64(&mut self, ty: Type, n: u64) -> Option<ShiftOpAndAmt> {
        let shiftimm = ShiftOpShiftImm::maybe_from_shift(n)?;
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
            _ => None,
        }
    }

    fn is_zero_simm9(&mut self, imm: &SImm9) -> Option<()> {
        if imm.value() == 0 {
            Some(())
        } else {
            None
        }
    }

    fn is_zero_uimm12(&mut self, imm: &UImm12Scaled) -> Option<()> {
        if imm.value() == 0 {
            Some(())
        } else {
            None
        }
    }

    /// This is target-word-size dependent.  And it excludes booleans and reftypes.
    fn valid_atomic_transaction(&mut self, ty: Type) -> Option<Type> {
        match ty {
            I8 | I16 | I32 | I64 => Some(ty),
            _ => None,
        }
    }

    /// This is the fallback case for loading a 64-bit integral constant into a
    /// register.
    ///
    /// The logic here is nontrivial enough that it's not really worth porting
    /// this over to ISLE.
    fn load_constant64_full(
        &mut self,
        ty: Type,
        extend: &generated_code::ImmExtend,
        value: u64,
    ) -> Reg {
        let bits = ty.bits();
        let value = if bits < 64 {
            if *extend == generated_code::ImmExtend::Sign {
                let shift = 64 - bits;
                let value = value as i64;

                ((value << shift) >> shift) as u64
            } else {
                value & !(u64::MAX << bits)
            }
        } else {
            value
        };
        let size = OperandSize::Size64;

        // If the top 32 bits are zero, use 32-bit `mov` operations.
        if value >> 32 == 0 {
            let size = OperandSize::Size32;
            let lower_halfword = value as u16;
            let upper_halfword = (value >> 16) as u16;

            let rd = self.temp_writable_reg(I64);
            if upper_halfword == u16::MAX {
                self.emit(&MInst::MovWide {
                    op: MoveWideOp::MovN,
                    rd,
                    imm: MoveWideConst::maybe_with_shift(!lower_halfword, 0).unwrap(),
                    size,
                });
            } else {
                self.emit(&MInst::MovWide {
                    op: MoveWideOp::MovZ,
                    rd,
                    imm: MoveWideConst::maybe_with_shift(lower_halfword, 0).unwrap(),
                    size,
                });

                if upper_halfword != 0 {
                    let tmp = self.temp_writable_reg(I64);
                    self.emit(&MInst::MovK {
                        rd: tmp,
                        rn: rd.to_reg(),
                        imm: MoveWideConst::maybe_with_shift(upper_halfword, 16).unwrap(),
                        size,
                    });
                    return tmp.to_reg();
                }
            };

            return rd.to_reg();
        } else if value == u64::MAX {
            let rd = self.temp_writable_reg(I64);
            self.emit(&MInst::MovWide {
                op: MoveWideOp::MovN,
                rd,
                imm: MoveWideConst::zero(),
                size,
            });
            return rd.to_reg();
        };

        // If the number of 0xffff half words is greater than the number of 0x0000 half words
        // it is more efficient to use `movn` for the first instruction.
        let first_is_inverted = count_zero_half_words(!value) > count_zero_half_words(value);

        // Either 0xffff or 0x0000 half words can be skipped, depending on the first
        // instruction used.
        let ignored_halfword = if first_is_inverted { 0xffff } else { 0 };

        let halfwords: SmallVec<[_; 4]> = (0..4)
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
        for (i, imm16) in halfwords {
            let shift = i * 16;
            let rd = self.temp_writable_reg(I64);

            if let Some(rn) = prev_result {
                let imm = MoveWideConst::maybe_with_shift(imm16 as u16, shift).unwrap();
                self.emit(&MInst::MovK { rd, rn, imm, size });
            } else {
                if first_is_inverted {
                    let imm =
                        MoveWideConst::maybe_with_shift(((!imm16) & 0xffff) as u16, shift).unwrap();
                    self.emit(&MInst::MovWide {
                        op: MoveWideOp::MovN,
                        rd,
                        imm,
                        size,
                    });
                } else {
                    let imm = MoveWideConst::maybe_with_shift(imm16 as u16, shift).unwrap();
                    self.emit(&MInst::MovWide {
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

        return prev_result.unwrap();

        fn count_zero_half_words(mut value: u64) -> usize {
            let mut count = 0;
            for _ in 0..4 {
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

    fn stack_reg(&mut self) -> Reg {
        stack_reg()
    }

    fn fp_reg(&mut self) -> Reg {
        fp_reg()
    }

    fn writable_link_reg(&mut self) -> WritableReg {
        writable_link_reg()
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
        self.lower_ctx.emit(inst.clone());
    }

    fn cond_br_zero(&mut self, reg: Reg) -> CondBrKind {
        CondBrKind::Zero(reg)
    }

    fn cond_br_not_zero(&mut self, reg: Reg) -> CondBrKind {
        CondBrKind::NotZero(reg)
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

    fn shift_mask(&mut self, ty: Type) -> ImmLogic {
        debug_assert!(ty.lane_bits().is_power_of_two());

        let mask = (ty.lane_bits() - 1) as u64;
        ImmLogic::maybe_from_u64(mask, I32).unwrap()
    }

    fn imm_shift_from_imm64(&mut self, ty: Type, val: Imm64) -> Option<ImmShift> {
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

    fn icmp_zero_cond(&mut self, cond: &IntCC) -> Option<IntCC> {
        match cond {
            &IntCC::Equal
            | &IntCC::SignedGreaterThanOrEqual
            | &IntCC::SignedGreaterThan
            | &IntCC::SignedLessThanOrEqual
            | &IntCC::SignedLessThan => Some(*cond),
            _ => None,
        }
    }

    fn fcmp_zero_cond(&mut self, cond: &FloatCC) -> Option<FloatCC> {
        match cond {
            &FloatCC::Equal
            | &FloatCC::GreaterThanOrEqual
            | &FloatCC::GreaterThan
            | &FloatCC::LessThanOrEqual
            | &FloatCC::LessThan => Some(*cond),
            _ => None,
        }
    }

    fn fcmp_zero_cond_not_eq(&mut self, cond: &FloatCC) -> Option<FloatCC> {
        match cond {
            &FloatCC::NotEqual => Some(FloatCC::NotEqual),
            _ => None,
        }
    }

    fn icmp_zero_cond_not_eq(&mut self, cond: &IntCC) -> Option<IntCC> {
        match cond {
            &IntCC::NotEqual => Some(IntCC::NotEqual),
            _ => None,
        }
    }

    fn float_cc_cmp_zero_to_vec_misc_op(&mut self, cond: &FloatCC) -> VecMisc2 {
        match cond {
            &FloatCC::Equal => VecMisc2::Fcmeq0,
            &FloatCC::GreaterThanOrEqual => VecMisc2::Fcmge0,
            &FloatCC::LessThanOrEqual => VecMisc2::Fcmle0,
            &FloatCC::GreaterThan => VecMisc2::Fcmgt0,
            &FloatCC::LessThan => VecMisc2::Fcmlt0,
            _ => panic!(),
        }
    }

    fn int_cc_cmp_zero_to_vec_misc_op(&mut self, cond: &IntCC) -> VecMisc2 {
        match cond {
            &IntCC::Equal => VecMisc2::Cmeq0,
            &IntCC::SignedGreaterThanOrEqual => VecMisc2::Cmge0,
            &IntCC::SignedLessThanOrEqual => VecMisc2::Cmle0,
            &IntCC::SignedGreaterThan => VecMisc2::Cmgt0,
            &IntCC::SignedLessThan => VecMisc2::Cmlt0,
            _ => panic!(),
        }
    }

    fn float_cc_cmp_zero_to_vec_misc_op_swap(&mut self, cond: &FloatCC) -> VecMisc2 {
        match cond {
            &FloatCC::Equal => VecMisc2::Fcmeq0,
            &FloatCC::GreaterThanOrEqual => VecMisc2::Fcmle0,
            &FloatCC::LessThanOrEqual => VecMisc2::Fcmge0,
            &FloatCC::GreaterThan => VecMisc2::Fcmlt0,
            &FloatCC::LessThan => VecMisc2::Fcmgt0,
            _ => panic!(),
        }
    }

    fn int_cc_cmp_zero_to_vec_misc_op_swap(&mut self, cond: &IntCC) -> VecMisc2 {
        match cond {
            &IntCC::Equal => VecMisc2::Cmeq0,
            &IntCC::SignedGreaterThanOrEqual => VecMisc2::Cmle0,
            &IntCC::SignedLessThanOrEqual => VecMisc2::Cmge0,
            &IntCC::SignedGreaterThan => VecMisc2::Cmlt0,
            &IntCC::SignedLessThan => VecMisc2::Cmgt0,
            _ => panic!(),
        }
    }

    fn amode(&mut self, ty: Type, addr: Value, offset: u32) -> AMode {
        lower_address(self.lower_ctx, ty, addr, offset as i32)
    }

    fn pair_amode(&mut self, addr: Value, offset: u32) -> PairAMode {
        lower_pair_address(self.lower_ctx, addr, offset as i32)
    }

    fn constant_f64(&mut self, value: u64) -> Reg {
        let rd = self.temp_writable_reg(I8X16);

        lower_constant_f64(self.lower_ctx, rd, f64::from_bits(value));

        rd.to_reg()
    }

    fn constant_f128(&mut self, value: u128) -> Reg {
        let rd = self.temp_writable_reg(I8X16);

        lower_constant_f128(self.lower_ctx, rd, value);

        rd.to_reg()
    }

    fn splat_const(&mut self, value: u64, size: &VectorSize) -> Reg {
        let rd = self.temp_writable_reg(I8X16);

        lower_splat_const(self.lower_ctx, rd, value, *size);

        rd.to_reg()
    }

    fn fp_cond_code(&mut self, cc: &condcodes::FloatCC) -> Cond {
        lower_fp_condcode(*cc)
    }

    fn cond_code(&mut self, cc: &condcodes::IntCC) -> Cond {
        lower_condcode(*cc)
    }

    fn invert_cond(&mut self, cond: &Cond) -> Cond {
        (*cond).invert()
    }
    fn preg_sp(&mut self) -> PReg {
        super::regs::stack_reg().to_real_reg().unwrap().into()
    }

    fn preg_fp(&mut self) -> PReg {
        super::regs::fp_reg().to_real_reg().unwrap().into()
    }

    fn preg_link(&mut self) -> PReg {
        super::regs::link_reg().to_real_reg().unwrap().into()
    }

    fn preg_pinned(&mut self) -> PReg {
        super::regs::pinned_reg().to_real_reg().unwrap().into()
    }

    fn branch_target(&mut self, elements: &VecMachLabel, idx: u8) -> BranchTarget {
        BranchTarget::Label(elements[idx as usize])
    }

    fn targets_jt_size(&mut self, elements: &VecMachLabel) -> u32 {
        (elements.len() - 1) as u32
    }

    fn targets_jt_space(&mut self, elements: &VecMachLabel) -> CodeOffset {
        // calculate the number of bytes needed for the jumptable sequence:
        // 4 bytes per instruction, with 8 instructions base + the size of
        // the jumptable more.
        4 * (8 + self.targets_jt_size(elements))
    }

    fn targets_jt_info(&mut self, elements: &VecMachLabel) -> BoxJTSequenceInfo {
        let targets: Vec<BranchTarget> = elements
            .iter()
            .skip(1)
            .map(|bix| BranchTarget::Label(*bix))
            .collect();
        let default_target = BranchTarget::Label(elements[0]);
        Box::new(JTSequenceInfo {
            targets,
            default_target,
        })
    }

    fn min_fp_value(&mut self, signed: bool, in_bits: u8, out_bits: u8) -> Reg {
        let tmp = self.lower_ctx.alloc_tmp(I8X16).only_reg().unwrap();

        if in_bits == 32 {
            // From float32.
            let min = match (signed, out_bits) {
                (true, 8) => i8::MIN as f32 - 1.,
                (true, 16) => i16::MIN as f32 - 1.,
                (true, 32) => i32::MIN as f32, // I32_MIN - 1 isn't precisely representable as a f32.
                (true, 64) => i64::MIN as f32, // I64_MIN - 1 isn't precisely representable as a f32.

                (false, _) => -1.,
                _ => unimplemented!(
                    "unexpected {} output size of {} bits for 32-bit input",
                    if signed { "signed" } else { "unsigned" },
                    out_bits
                ),
            };

            lower_constant_f32(self.lower_ctx, tmp, min);
        } else if in_bits == 64 {
            // From float64.
            let min = match (signed, out_bits) {
                (true, 8) => i8::MIN as f64 - 1.,
                (true, 16) => i16::MIN as f64 - 1.,
                (true, 32) => i32::MIN as f64 - 1.,
                (true, 64) => i64::MIN as f64,

                (false, _) => -1.,
                _ => unimplemented!(
                    "unexpected {} output size of {} bits for 64-bit input",
                    if signed { "signed" } else { "unsigned" },
                    out_bits
                ),
            };

            lower_constant_f64(self.lower_ctx, tmp, min);
        } else {
            unimplemented!(
                "unexpected input size for min_fp_value: {} (signed: {}, output size: {})",
                in_bits,
                signed,
                out_bits
            );
        }

        tmp.to_reg()
    }

    fn max_fp_value(&mut self, signed: bool, in_bits: u8, out_bits: u8) -> Reg {
        let tmp = self.lower_ctx.alloc_tmp(I8X16).only_reg().unwrap();

        if in_bits == 32 {
            // From float32.
            let max = match (signed, out_bits) {
                (true, 8) => i8::MAX as f32 + 1.,
                (true, 16) => i16::MAX as f32 + 1.,
                (true, 32) => (i32::MAX as u64 + 1) as f32,
                (true, 64) => (i64::MAX as u64 + 1) as f32,

                (false, 8) => u8::MAX as f32 + 1.,
                (false, 16) => u16::MAX as f32 + 1.,
                (false, 32) => (u32::MAX as u64 + 1) as f32,
                (false, 64) => (u64::MAX as u128 + 1) as f32,
                _ => unimplemented!(
                    "unexpected {} output size of {} bits for 32-bit input",
                    if signed { "signed" } else { "unsigned" },
                    out_bits
                ),
            };

            lower_constant_f32(self.lower_ctx, tmp, max);
        } else if in_bits == 64 {
            // From float64.
            let max = match (signed, out_bits) {
                (true, 8) => i8::MAX as f64 + 1.,
                (true, 16) => i16::MAX as f64 + 1.,
                (true, 32) => i32::MAX as f64 + 1.,
                (true, 64) => (i64::MAX as u64 + 1) as f64,

                (false, 8) => u8::MAX as f64 + 1.,
                (false, 16) => u16::MAX as f64 + 1.,
                (false, 32) => u32::MAX as f64 + 1.,
                (false, 64) => (u64::MAX as u128 + 1) as f64,
                _ => unimplemented!(
                    "unexpected {} output size of {} bits for 64-bit input",
                    if signed { "signed" } else { "unsigned" },
                    out_bits
                ),
            };

            lower_constant_f64(self.lower_ctx, tmp, max);
        } else {
            unimplemented!(
                "unexpected input size for max_fp_value: {} (signed: {}, output size: {})",
                in_bits,
                signed,
                out_bits
            );
        }

        tmp.to_reg()
    }

    fn fpu_op_ri_ushr(&mut self, ty_bits: u8, shift: u8) -> FPUOpRI {
        if ty_bits == 32 {
            FPUOpRI::UShr32(FPURightShiftImm::maybe_from_u8(shift, ty_bits).unwrap())
        } else if ty_bits == 64 {
            FPUOpRI::UShr64(FPURightShiftImm::maybe_from_u8(shift, ty_bits).unwrap())
        } else {
            unimplemented!(
                "unexpected input size for fpu_op_ri_ushr: {} (shift: {})",
                ty_bits,
                shift
            );
        }
    }

    fn fpu_op_ri_sli(&mut self, ty_bits: u8, shift: u8) -> FPUOpRIMod {
        if ty_bits == 32 {
            FPUOpRIMod::Sli32(FPULeftShiftImm::maybe_from_u8(shift, ty_bits).unwrap())
        } else if ty_bits == 64 {
            FPUOpRIMod::Sli64(FPULeftShiftImm::maybe_from_u8(shift, ty_bits).unwrap())
        } else {
            unimplemented!(
                "unexpected input size for fpu_op_ri_sli: {} (shift: {})",
                ty_bits,
                shift
            );
        }
    }
}
