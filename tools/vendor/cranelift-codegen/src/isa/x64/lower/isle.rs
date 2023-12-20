//! ISLE integration glue code for x64 lowering.

// Pull in the ISLE generated code.
pub(crate) mod generated_code;
use crate::{
    ir::types,
    ir::AtomicRmwOp,
    machinst::{InputSourceInst, Reg, Writable},
};
use crate::{isle_common_prelude_methods, isle_lower_prelude_methods};
use generated_code::{Context, MInst, RegisterClass};

// Types that the generated ISLE code uses via `use super::*`.
use super::{is_int_or_ref_ty, is_mergeable_load, lower_to_amode};
use crate::ir::LibCall;
use crate::isa::x64::lower::emit_vm_call;
use crate::{
    ir::{
        condcodes::{CondCode, FloatCC, IntCC},
        immediates::*,
        types::*,
        Inst, InstructionData, MemFlags, Opcode, TrapCode, Value, ValueList,
    },
    isa::{
        settings::Flags,
        unwind::UnwindInst,
        x64::{
            abi::X64Caller,
            inst::{args::*, regs, CallInfo},
            settings::Flags as IsaFlags,
        },
    },
    machinst::{
        isle::*, valueregs, ArgPair, InsnInput, InsnOutput, Lower, MachAtomicRmwOp, MachInst,
        VCodeConstant, VCodeConstantData,
    },
};
use alloc::vec::Vec;
use regalloc2::PReg;
use smallvec::SmallVec;
use std::boxed::Box;
use std::convert::TryFrom;
use target_lexicon::Triple;

type BoxCallInfo = Box<CallInfo>;
type BoxVecMachLabel = Box<SmallVec<[MachLabel; 4]>>;
type MachLabelSlice = [MachLabel];
type VecArgPair = Vec<ArgPair>;

pub struct SinkableLoad {
    inst: Inst,
    addr_input: InsnInput,
    offset: i32,
}

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
        |cx, insn| generated_code::constructor_lower_branch(cx, insn, targets),
    )
}

impl Context for IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_lower_prelude_methods!();
    isle_prelude_caller_methods!(X64ABIMachineSpec, X64Caller);

    #[inline]
    fn operand_size_of_type_32_64(&mut self, ty: Type) -> OperandSize {
        if ty.bits() == 64 {
            OperandSize::Size64
        } else {
            OperandSize::Size32
        }
    }

    #[inline]
    fn raw_operand_size_of_type(&mut self, ty: Type) -> OperandSize {
        OperandSize::from_ty(ty)
    }

    fn put_in_reg_mem_imm(&mut self, val: Value) -> RegMemImm {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            if let Some(imm) = to_simm32(c as i64) {
                return imm.to_reg_mem_imm();
            }

            // A load from the constant pool is better than a
            // rematerialization into a register, because it reduces
            // register pressure.
            let vcode_constant = self.emit_u64_le_const(c);
            return RegMemImm::mem(SyntheticAmode::ConstantOffset(vcode_constant));
        }

        if let InputSourceInst::UniqueUse(src_insn, 0) = inputs.inst {
            if let Some((addr_input, offset)) = is_mergeable_load(self.lower_ctx, src_insn) {
                self.lower_ctx.sink_inst(src_insn);
                let amode = lower_to_amode(self.lower_ctx, addr_input, offset);
                return RegMemImm::mem(amode);
            }
        }

        RegMemImm::reg(self.put_in_reg(val))
    }

    fn put_in_xmm_mem_imm(&mut self, val: Value) -> XmmMemImm {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            if let Some(imm) = to_simm32(c as i64) {
                return XmmMemImm::new(imm.to_reg_mem_imm()).unwrap();
            }
        }

        let res = match self.put_in_xmm_mem(val).to_reg_mem() {
            RegMem::Reg { reg } => RegMemImm::Reg { reg },
            RegMem::Mem { addr } => RegMemImm::Mem { addr },
        };

        XmmMemImm::new(res).unwrap()
    }

    fn put_in_xmm_mem(&mut self, val: Value) -> XmmMem {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            // A load from the constant pool is better than a rematerialization into a register,
            // because it reduces register pressure.
            //
            // NOTE: this is where behavior differs from `put_in_reg_mem`, as we always force
            // constants to be 16 bytes when a constant will be used in place of an xmm register.
            let vcode_constant = self.emit_u128_le_const(c as u128);
            return XmmMem::new(RegMem::mem(SyntheticAmode::ConstantOffset(vcode_constant)))
                .unwrap();
        }

        XmmMem::new(RegMem::reg(self.put_in_reg(val))).unwrap()
    }

    fn put_in_reg_mem(&mut self, val: Value) -> RegMem {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            // A load from the constant pool is better than a
            // rematerialization into a register, because it reduces
            // register pressure.
            let vcode_constant = self.emit_u64_le_const(c);
            return RegMem::mem(SyntheticAmode::ConstantOffset(vcode_constant));
        }

        if let InputSourceInst::UniqueUse(src_insn, 0) = inputs.inst {
            if let Some((addr_input, offset)) = is_mergeable_load(self.lower_ctx, src_insn) {
                self.lower_ctx.sink_inst(src_insn);
                let amode = lower_to_amode(self.lower_ctx, addr_input, offset);
                return RegMem::mem(amode);
            }
        }

        RegMem::reg(self.put_in_reg(val))
    }

    #[inline]
    fn encode_fcmp_imm(&mut self, imm: &FcmpImm) -> u8 {
        imm.encode()
    }

    #[inline]
    fn encode_round_imm(&mut self, imm: &RoundImm) -> u8 {
        imm.encode()
    }

    #[inline]
    fn avx512vl_enabled(&mut self, _: Type) -> bool {
        self.isa_flags.use_avx512vl_simd()
    }

    #[inline]
    fn avx512dq_enabled(&mut self, _: Type) -> bool {
        self.isa_flags.use_avx512dq_simd()
    }

    #[inline]
    fn avx512f_enabled(&mut self, _: Type) -> bool {
        self.isa_flags.use_avx512f_simd()
    }

    #[inline]
    fn avx512bitalg_enabled(&mut self, _: Type) -> bool {
        self.isa_flags.use_avx512bitalg_simd()
    }

    #[inline]
    fn avx512vbmi_enabled(&mut self, _: Type) -> bool {
        self.isa_flags.use_avx512vbmi_simd()
    }

    #[inline]
    fn use_lzcnt(&mut self, _: Type) -> bool {
        self.isa_flags.use_lzcnt()
    }

    #[inline]
    fn use_bmi1(&mut self, _: Type) -> bool {
        self.isa_flags.use_bmi1()
    }

    #[inline]
    fn use_popcnt(&mut self, _: Type) -> bool {
        self.isa_flags.use_popcnt()
    }

    #[inline]
    fn use_fma(&mut self, _: Type) -> bool {
        self.isa_flags.use_fma()
    }

    #[inline]
    fn use_sse41(&mut self, _: Type) -> bool {
        self.isa_flags.use_sse41()
    }

    #[inline]
    fn imm8_from_value(&mut self, val: Value) -> Option<Imm8Reg> {
        let inst = self.lower_ctx.dfg().value_def(val).inst()?;
        let constant = self.lower_ctx.get_constant(inst)?;
        let imm = u8::try_from(constant).ok()?;
        Some(Imm8Reg::Imm8 { imm })
    }

    #[inline]
    fn const_to_type_masked_imm8(&mut self, c: u64, ty: Type) -> Imm8Gpr {
        let mask = self.shift_mask(ty) as u64;
        Imm8Gpr::new(Imm8Reg::Imm8 {
            imm: (c & mask) as u8,
        })
        .unwrap()
    }

    #[inline]
    fn shift_mask(&mut self, ty: Type) -> u32 {
        debug_assert!(ty.lane_bits().is_power_of_two());

        ty.lane_bits() - 1
    }

    #[inline]
    fn simm32_from_value(&mut self, val: Value) -> Option<GprMemImm> {
        let inst = self.lower_ctx.dfg().value_def(val).inst()?;
        let constant: u64 = self.lower_ctx.get_constant(inst)?;
        let constant = constant as i64;
        to_simm32(constant)
    }

    #[inline]
    fn simm32_from_imm64(&mut self, imm: Imm64) -> Option<GprMemImm> {
        to_simm32(imm.bits())
    }

    fn sinkable_load(&mut self, val: Value) -> Option<SinkableLoad> {
        let input = self.lower_ctx.get_value_as_source_or_const(val);
        if let InputSourceInst::UniqueUse(inst, 0) = input.inst {
            if let Some((addr_input, offset)) = is_mergeable_load(self.lower_ctx, inst) {
                return Some(SinkableLoad {
                    inst,
                    addr_input,
                    offset,
                });
            }
        }
        None
    }

    fn sink_load(&mut self, load: &SinkableLoad) -> RegMem {
        self.lower_ctx.sink_inst(load.inst);
        let addr = lower_to_amode(self.lower_ctx, load.addr_input, load.offset);
        RegMem::Mem {
            addr: SyntheticAmode::Real(addr),
        }
    }

    #[inline]
    fn ext_mode(&mut self, from_bits: u16, to_bits: u16) -> ExtMode {
        ExtMode::new(from_bits, to_bits).unwrap()
    }

    fn emit(&mut self, inst: &MInst) -> Unit {
        self.lower_ctx.emit(inst.clone());
    }

    #[inline]
    fn nonzero_u64_fits_in_u32(&mut self, x: u64) -> Option<u64> {
        if x != 0 && x < u64::from(u32::MAX) {
            Some(x)
        } else {
            None
        }
    }

    #[inline]
    fn sse_insertps_lane_imm(&mut self, lane: u8) -> u8 {
        // Insert 32-bits from replacement (at index 00, bits 7:8) to vector (lane
        // shifted into bits 5:6).
        0b00_00_00_00 | lane << 4
    }

    #[inline]
    fn synthetic_amode_to_reg_mem(&mut self, addr: &SyntheticAmode) -> RegMem {
        RegMem::mem(addr.clone())
    }

    #[inline]
    fn amode_imm_reg_reg_shift(&mut self, simm32: u32, base: Gpr, index: Gpr, shift: u8) -> Amode {
        Amode::imm_reg_reg_shift(simm32, base, index, shift)
    }

    #[inline]
    fn amode_imm_reg(&mut self, simm32: u32, base: Gpr) -> Amode {
        Amode::imm_reg(simm32, base.to_reg())
    }

    #[inline]
    fn amode_with_flags(&mut self, amode: &Amode, flags: MemFlags) -> Amode {
        amode.with_flags(flags)
    }

    #[inline]
    fn amode_to_synthetic_amode(&mut self, amode: &Amode) -> SyntheticAmode {
        amode.clone().into()
    }

    #[inline]
    fn const_to_synthetic_amode(&mut self, c: VCodeConstant) -> SyntheticAmode {
        SyntheticAmode::ConstantOffset(c)
    }

    #[inline]
    fn writable_gpr_to_reg(&mut self, r: WritableGpr) -> WritableReg {
        r.to_writable_reg()
    }

    #[inline]
    fn writable_xmm_to_reg(&mut self, r: WritableXmm) -> WritableReg {
        r.to_writable_reg()
    }

    fn ishl_i8x16_mask_for_const(&mut self, amt: u32) -> SyntheticAmode {
        // When the shift amount is known, we can statically (i.e. at compile
        // time) determine the mask to use and only emit that.
        debug_assert!(amt < 8);
        let mask_offset = amt as usize * 16;
        let mask_constant = self.lower_ctx.use_constant(VCodeConstantData::WellKnown(
            &I8X16_ISHL_MASKS[mask_offset..mask_offset + 16],
        ));
        SyntheticAmode::ConstantOffset(mask_constant)
    }

    fn ishl_i8x16_mask_table(&mut self) -> SyntheticAmode {
        let mask_table = self
            .lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&I8X16_ISHL_MASKS));
        SyntheticAmode::ConstantOffset(mask_table)
    }

    fn ushr_i8x16_mask_for_const(&mut self, amt: u32) -> SyntheticAmode {
        // When the shift amount is known, we can statically (i.e. at compile
        // time) determine the mask to use and only emit that.
        debug_assert!(amt < 8);
        let mask_offset = amt as usize * 16;
        let mask_constant = self.lower_ctx.use_constant(VCodeConstantData::WellKnown(
            &I8X16_USHR_MASKS[mask_offset..mask_offset + 16],
        ));
        SyntheticAmode::ConstantOffset(mask_constant)
    }

    fn ushr_i8x16_mask_table(&mut self) -> SyntheticAmode {
        let mask_table = self
            .lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&I8X16_USHR_MASKS));
        SyntheticAmode::ConstantOffset(mask_table)
    }

    fn popcount_4bit_table(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&POPCOUNT_4BIT_TABLE))
    }

    fn popcount_low_mask(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&POPCOUNT_LOW_MASK))
    }

    #[inline]
    fn writable_reg_to_xmm(&mut self, r: WritableReg) -> WritableXmm {
        Writable::from_reg(Xmm::new(r.to_reg()).unwrap())
    }

    #[inline]
    fn writable_xmm_to_xmm(&mut self, r: WritableXmm) -> Xmm {
        r.to_reg()
    }

    #[inline]
    fn writable_gpr_to_gpr(&mut self, r: WritableGpr) -> Gpr {
        r.to_reg()
    }

    #[inline]
    fn gpr_to_reg(&mut self, r: Gpr) -> Reg {
        r.into()
    }

    #[inline]
    fn xmm_to_reg(&mut self, r: Xmm) -> Reg {
        r.into()
    }

    #[inline]
    fn xmm_to_xmm_mem_imm(&mut self, r: Xmm) -> XmmMemImm {
        r.into()
    }

    #[inline]
    fn temp_writable_gpr(&mut self) -> WritableGpr {
        Writable::from_reg(Gpr::new(self.temp_writable_reg(I64).to_reg()).unwrap())
    }

    #[inline]
    fn temp_writable_xmm(&mut self) -> WritableXmm {
        Writable::from_reg(Xmm::new(self.temp_writable_reg(I8X16).to_reg()).unwrap())
    }

    #[inline]
    fn reg_to_reg_mem_imm(&mut self, reg: Reg) -> RegMemImm {
        RegMemImm::Reg { reg }
    }

    #[inline]
    fn reg_mem_to_xmm_mem(&mut self, rm: &RegMem) -> XmmMem {
        XmmMem::new(rm.clone()).unwrap()
    }

    #[inline]
    fn gpr_mem_imm_new(&mut self, rmi: &RegMemImm) -> GprMemImm {
        GprMemImm::new(rmi.clone()).unwrap()
    }

    #[inline]
    fn xmm_mem_imm_new(&mut self, rmi: &RegMemImm) -> XmmMemImm {
        XmmMemImm::new(rmi.clone()).unwrap()
    }

    #[inline]
    fn xmm_to_xmm_mem(&mut self, r: Xmm) -> XmmMem {
        r.into()
    }

    #[inline]
    fn xmm_mem_to_reg_mem(&mut self, xm: &XmmMem) -> RegMem {
        xm.clone().into()
    }

    #[inline]
    fn gpr_mem_to_reg_mem(&mut self, gm: &GprMem) -> RegMem {
        gm.clone().into()
    }

    #[inline]
    fn xmm_new(&mut self, r: Reg) -> Xmm {
        Xmm::new(r).unwrap()
    }

    #[inline]
    fn gpr_new(&mut self, r: Reg) -> Gpr {
        Gpr::new(r).unwrap()
    }

    #[inline]
    fn reg_mem_to_gpr_mem(&mut self, rm: &RegMem) -> GprMem {
        GprMem::new(rm.clone()).unwrap()
    }

    #[inline]
    fn reg_to_gpr_mem(&mut self, r: Reg) -> GprMem {
        GprMem::new(RegMem::reg(r)).unwrap()
    }

    #[inline]
    fn imm8_reg_to_imm8_gpr(&mut self, ir: &Imm8Reg) -> Imm8Gpr {
        Imm8Gpr::new(ir.clone()).unwrap()
    }

    #[inline]
    fn gpr_to_gpr_mem(&mut self, gpr: Gpr) -> GprMem {
        GprMem::from(gpr)
    }

    #[inline]
    fn gpr_to_gpr_mem_imm(&mut self, gpr: Gpr) -> GprMemImm {
        GprMemImm::from(gpr)
    }

    #[inline]
    fn gpr_to_imm8_gpr(&mut self, gpr: Gpr) -> Imm8Gpr {
        Imm8Gpr::from(gpr)
    }

    #[inline]
    fn imm8_to_imm8_gpr(&mut self, imm: u8) -> Imm8Gpr {
        Imm8Gpr::new(Imm8Reg::Imm8 { imm }).unwrap()
    }

    #[inline]
    fn type_register_class(&mut self, ty: Type) -> Option<RegisterClass> {
        if is_int_or_ref_ty(ty) || ty == I128 {
            Some(RegisterClass::Gpr {
                single_register: ty != I128,
            })
        } else if ty == F32 || ty == F64 || (ty.is_vector() && ty.bits() == 128) {
            Some(RegisterClass::Xmm)
        } else {
            None
        }
    }

    #[inline]
    fn ty_int_bool_or_ref(&mut self, ty: Type) -> Option<()> {
        match ty {
            types::I8 | types::I16 | types::I32 | types::I64 | types::R64 => Some(()),
            types::R32 => panic!("shouldn't have 32-bits refs on x64"),
            _ => None,
        }
    }

    #[inline]
    fn intcc_without_eq(&mut self, x: &IntCC) -> IntCC {
        x.without_equal()
    }

    #[inline]
    fn intcc_to_cc(&mut self, intcc: &IntCC) -> CC {
        CC::from_intcc(*intcc)
    }

    #[inline]
    fn cc_invert(&mut self, cc: &CC) -> CC {
        cc.invert()
    }

    #[inline]
    fn cc_nz_or_z(&mut self, cc: &CC) -> Option<CC> {
        match cc {
            CC::Z => Some(*cc),
            CC::NZ => Some(*cc),
            _ => None,
        }
    }

    #[inline]
    fn sum_extend_fits_in_32_bits(
        &mut self,
        extend_from_ty: Type,
        constant_value: Imm64,
        offset: Offset32,
    ) -> Option<u32> {
        let offset: i64 = offset.into();
        let constant_value: u64 = constant_value.bits() as u64;
        // If necessary, zero extend `constant_value` up to 64 bits.
        let shift = 64 - extend_from_ty.bits();
        let zero_extended_constant_value = (constant_value << shift) >> shift;
        // Sum up the two operands.
        let sum = offset.wrapping_add(zero_extended_constant_value as i64);
        // Check that the sum will fit in 32-bits.
        if sum == ((sum << 32) >> 32) {
            Some(sum as u32)
        } else {
            None
        }
    }

    #[inline]
    fn amode_offset(&mut self, addr: &Amode, offset: u32) -> Amode {
        addr.offset(offset)
    }

    #[inline]
    fn zero_offset(&mut self) -> Offset32 {
        Offset32::new(0)
    }

    #[inline]
    fn atomic_rmw_op_to_mach_atomic_rmw_op(&mut self, op: &AtomicRmwOp) -> MachAtomicRmwOp {
        MachAtomicRmwOp::from(*op)
    }

    #[inline]
    fn preg_rbp(&mut self) -> PReg {
        regs::rbp().to_real_reg().unwrap().into()
    }

    #[inline]
    fn preg_rsp(&mut self) -> PReg {
        regs::rsp().to_real_reg().unwrap().into()
    }

    #[inline]
    fn preg_pinned(&mut self) -> PReg {
        regs::pinned_reg().to_real_reg().unwrap().into()
    }

    fn libcall_1(&mut self, libcall: &LibCall, a: Reg) -> Reg {
        let call_conv = self.lower_ctx.abi().call_conv(self.lower_ctx.sigs());
        let ret_ty = libcall.signature(call_conv).returns[0].value_type;
        let output_reg = self.lower_ctx.alloc_tmp(ret_ty).only_reg().unwrap();

        emit_vm_call(
            self.lower_ctx,
            self.flags,
            self.triple,
            libcall.clone(),
            &[a],
            &[output_reg],
        )
        .expect("Failed to emit LibCall");

        output_reg.to_reg()
    }

    fn libcall_3(&mut self, libcall: &LibCall, a: Reg, b: Reg, c: Reg) -> Reg {
        let call_conv = self.lower_ctx.abi().call_conv(self.lower_ctx.sigs());
        let ret_ty = libcall.signature(call_conv).returns[0].value_type;
        let output_reg = self.lower_ctx.alloc_tmp(ret_ty).only_reg().unwrap();

        emit_vm_call(
            self.lower_ctx,
            self.flags,
            self.triple,
            libcall.clone(),
            &[a, b, c],
            &[output_reg],
        )
        .expect("Failed to emit LibCall");

        output_reg.to_reg()
    }

    #[inline]
    fn single_target(&mut self, targets: &MachLabelSlice) -> Option<MachLabel> {
        if targets.len() == 1 {
            Some(targets[0])
        } else {
            None
        }
    }

    #[inline]
    fn two_targets(&mut self, targets: &MachLabelSlice) -> Option<(MachLabel, MachLabel)> {
        if targets.len() == 2 {
            Some((targets[0], targets[1]))
        } else {
            None
        }
    }

    #[inline]
    fn jump_table_targets(
        &mut self,
        targets: &MachLabelSlice,
    ) -> Option<(MachLabel, BoxVecMachLabel)> {
        if targets.is_empty() {
            return None;
        }

        let default_label = targets[0];
        let jt_targets = Box::new(SmallVec::from(&targets[1..]));
        Some((default_label, jt_targets))
    }

    #[inline]
    fn jump_table_size(&mut self, targets: &BoxVecMachLabel) -> u32 {
        targets.len() as u32
    }

    #[inline]
    fn vconst_all_ones_or_all_zeros(&mut self, constant: Constant) -> Option<()> {
        let const_data = self.lower_ctx.get_constant_data(constant);
        if const_data.iter().all(|&b| b == 0 || b == 0xFF) {
            return Some(());
        }
        None
    }

    #[inline]
    fn fcvt_uint_mask_const(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&UINT_MASK))
    }

    #[inline]
    fn fcvt_uint_mask_high_const(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&UINT_MASK_HIGH))
    }

    #[inline]
    fn iadd_pairwise_mul_const_16(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&IADD_PAIRWISE_MUL_CONST_16))
    }

    #[inline]
    fn iadd_pairwise_mul_const_32(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&IADD_PAIRWISE_MUL_CONST_32))
    }

    #[inline]
    fn iadd_pairwise_xor_const_32(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&IADD_PAIRWISE_XOR_CONST_32))
    }

    #[inline]
    fn iadd_pairwise_addd_const_32(&mut self) -> VCodeConstant {
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&IADD_PAIRWISE_ADDD_CONST_32))
    }

    #[inline]
    fn snarrow_umax_mask(&mut self) -> VCodeConstant {
        // 2147483647.0 is equivalent to 0x41DFFFFFFFC00000
        static UMAX_MASK: [u8; 16] = [
            0x00, 0x00, 0xC0, 0xFF, 0xFF, 0xFF, 0xDF, 0x41, 0x00, 0x00, 0xC0, 0xFF, 0xFF, 0xFF,
            0xDF, 0x41,
        ];
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&UMAX_MASK))
    }

    #[inline]
    fn shuffle_0_31_mask(&mut self, mask: &VecMask) -> VCodeConstant {
        let mask = mask
            .iter()
            .map(|&b| if b > 15 { b.wrapping_sub(16) } else { b })
            .map(|b| if b > 15 { 0b10000000 } else { b })
            .collect();
        self.lower_ctx
            .use_constant(VCodeConstantData::Generated(mask))
    }

    #[inline]
    fn shuffle_0_15_mask(&mut self, mask: &VecMask) -> VCodeConstant {
        let mask = mask
            .iter()
            .map(|&b| if b > 15 { 0b10000000 } else { b })
            .collect();
        self.lower_ctx
            .use_constant(VCodeConstantData::Generated(mask))
    }

    #[inline]
    fn shuffle_16_31_mask(&mut self, mask: &VecMask) -> VCodeConstant {
        let mask = mask
            .iter()
            .map(|&b| b.wrapping_sub(16))
            .map(|b| if b > 15 { 0b10000000 } else { b })
            .collect();
        self.lower_ctx
            .use_constant(VCodeConstantData::Generated(mask))
    }

    #[inline]
    fn perm_from_mask_with_zeros(
        &mut self,
        mask: &VecMask,
    ) -> Option<(VCodeConstant, VCodeConstant)> {
        if !mask.iter().any(|&b| b > 31) {
            return None;
        }

        let zeros = mask
            .iter()
            .map(|&b| if b > 31 { 0x00 } else { 0xff })
            .collect();

        Some((
            self.perm_from_mask(mask),
            self.lower_ctx
                .use_constant(VCodeConstantData::Generated(zeros)),
        ))
    }

    #[inline]
    fn perm_from_mask(&mut self, mask: &VecMask) -> VCodeConstant {
        let mask = mask.iter().cloned().collect();
        self.lower_ctx
            .use_constant(VCodeConstantData::Generated(mask))
    }

    #[inline]
    fn swizzle_zero_mask(&mut self) -> VCodeConstant {
        static ZERO_MASK_VALUE: [u8; 16] = [0x70; 16];
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&ZERO_MASK_VALUE))
    }

    #[inline]
    fn sqmul_round_sat_mask(&mut self) -> VCodeConstant {
        static SAT_MASK: [u8; 16] = [
            0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
            0x00, 0x80,
        ];
        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&SAT_MASK))
    }

    #[inline]
    fn uunarrow_umax_mask(&mut self) -> VCodeConstant {
        // 4294967295.0 is equivalent to 0x41EFFFFFFFE00000
        static UMAX_MASK: [u8; 16] = [
            0x00, 0x00, 0xE0, 0xFF, 0xFF, 0xFF, 0xEF, 0x41, 0x00, 0x00, 0xE0, 0xFF, 0xFF, 0xFF,
            0xEF, 0x41,
        ];

        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&UMAX_MASK))
    }

    #[inline]
    fn uunarrow_uint_mask(&mut self) -> VCodeConstant {
        static UINT_MASK: [u8; 16] = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x30, 0x43,
        ];

        self.lower_ctx
            .use_constant(VCodeConstantData::WellKnown(&UINT_MASK))
    }

    fn emit_div_or_rem(
        &mut self,
        kind: &DivOrRemKind,
        ty: Type,
        dst: WritableGpr,
        dividend: Gpr,
        divisor: Gpr,
    ) {
        let is_div = kind.is_div();
        let size = OperandSize::from_ty(ty);

        let dst_quotient = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();
        let dst_remainder = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();

        // Always do explicit checks for `srem`: otherwise, INT_MIN % -1 is not handled properly.
        if self.flags.avoid_div_traps() || *kind == DivOrRemKind::SignedRem {
            // A vcode meta-instruction is used to lower the inline checks, since they embed
            // pc-relative offsets that must not change, thus requiring regalloc to not
            // interfere by introducing spills and reloads.
            let tmp = if *kind == DivOrRemKind::SignedDiv && size == OperandSize::Size64 {
                Some(self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap())
            } else {
                None
            };
            let dividend_hi = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();
            self.lower_ctx.emit(MInst::alu_rmi_r(
                OperandSize::Size32,
                AluRmiROpcode::Xor,
                RegMemImm::reg(dividend_hi.to_reg()),
                dividend_hi,
            ));
            self.lower_ctx.emit(MInst::checked_div_or_rem_seq(
                kind.clone(),
                size,
                divisor.to_reg(),
                Gpr::new(dividend.to_reg()).unwrap(),
                Gpr::new(dividend_hi.to_reg()).unwrap(),
                WritableGpr::from_reg(Gpr::new(dst_quotient.to_reg()).unwrap()),
                WritableGpr::from_reg(Gpr::new(dst_remainder.to_reg()).unwrap()),
                tmp,
            ));
        } else {
            // We don't want more than one trap record for a single instruction,
            // so let's not allow the "mem" case (load-op merging) here; force
            // divisor into a register instead.
            let divisor = RegMem::reg(divisor.to_reg());

            let dividend_hi = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();

            // Fill in the high parts:
            let dividend_lo = if kind.is_signed() && ty == types::I8 {
                let dividend_lo = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();
                // 8-bit div takes its dividend in only the `lo` reg.
                self.lower_ctx.emit(MInst::sign_extend_data(
                    size,
                    Gpr::new(dividend.to_reg()).unwrap(),
                    WritableGpr::from_reg(Gpr::new(dividend_lo.to_reg()).unwrap()),
                ));
                // `dividend_hi` is not used by the Div below, so we
                // don't def it here.

                dividend_lo.to_reg()
            } else if kind.is_signed() {
                // 16-bit and higher div takes its operand in hi:lo
                // with half in each (64:64, 32:32 or 16:16).
                self.lower_ctx.emit(MInst::sign_extend_data(
                    size,
                    Gpr::new(dividend.to_reg()).unwrap(),
                    WritableGpr::from_reg(Gpr::new(dividend_hi.to_reg()).unwrap()),
                ));

                dividend.to_reg()
            } else if ty == types::I8 {
                let dividend_lo = self.lower_ctx.alloc_tmp(types::I64).only_reg().unwrap();
                self.lower_ctx.emit(MInst::movzx_rm_r(
                    ExtMode::BL,
                    RegMem::reg(dividend.to_reg()),
                    dividend_lo,
                ));

                dividend_lo.to_reg()
            } else {
                // zero for unsigned opcodes.
                self.lower_ctx
                    .emit(MInst::imm(OperandSize::Size64, 0, dividend_hi));

                dividend.to_reg()
            };

            // Emit the actual idiv.
            self.lower_ctx.emit(MInst::div(
                size,
                kind.is_signed(),
                divisor,
                Gpr::new(dividend_lo).unwrap(),
                Gpr::new(dividend_hi.to_reg()).unwrap(),
                WritableGpr::from_reg(Gpr::new(dst_quotient.to_reg()).unwrap()),
                WritableGpr::from_reg(Gpr::new(dst_remainder.to_reg()).unwrap()),
            ));
        }

        // Move the result back into the destination reg.
        if is_div {
            // The quotient is in rax.
            self.lower_ctx.emit(MInst::gen_move(
                dst.to_writable_reg(),
                dst_quotient.to_reg(),
                ty,
            ));
        } else {
            if size == OperandSize::Size8 {
                let tmp = self.temp_writable_reg(ty);
                // The remainder is in AH. Right-shift by 8 bits then move from rax.
                self.lower_ctx.emit(MInst::shift_r(
                    OperandSize::Size64,
                    ShiftKind::ShiftRightLogical,
                    Imm8Gpr::new(Imm8Reg::Imm8 { imm: 8 }).unwrap(),
                    dst_quotient.to_reg(),
                    tmp,
                ));
                self.lower_ctx
                    .emit(MInst::gen_move(dst.to_writable_reg(), tmp.to_reg(), ty));
            } else {
                // The remainder is in rdx.
                self.lower_ctx.emit(MInst::gen_move(
                    dst.to_writable_reg(),
                    dst_remainder.to_reg(),
                    ty,
                ));
            }
        }
    }
}

impl IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_prelude_method_helpers!(X64Caller);
}

// Since x64 doesn't have 8x16 shifts and we must use a 16x8 shift instead, we
// need to fix up the bits that migrate from one half of the lane to the
// other. Each 16-byte mask is indexed by the shift amount: e.g. if we shift
// right by 0 (no movement), we want to retain all the bits so we mask with
// `0xff`; if we shift right by 1, we want to retain all bits except the MSB so
// we mask with `0x7f`; etc.

#[rustfmt::skip] // Preserve 16 bytes (i.e. one mask) per row.
const I8X16_ISHL_MASKS: [u8; 128] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe,
    0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfc,
    0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8, 0xf8,
    0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0, 0xf0,
    0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0, 0xe0,
    0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0, 0xc0,
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
];

#[rustfmt::skip] // Preserve 16 bytes (i.e. one mask) per row.
const I8X16_USHR_MASKS: [u8; 128] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f,
    0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f, 0x3f,
    0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f, 0x1f,
    0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f,
    0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
    0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
];

/// Number of bits set in a given nibble (4-bit value). Used in the
/// vector implementation of popcount.
#[rustfmt::skip] // Preserve 4x4 layout.
const POPCOUNT_4BIT_TABLE: [u8; 16] = [
    0x00, 0x01, 0x01, 0x02,
    0x01, 0x02, 0x02, 0x03,
    0x01, 0x02, 0x02, 0x03,
    0x02, 0x03, 0x03, 0x04,
];

const POPCOUNT_LOW_MASK: [u8; 16] = [0x0f; 16];

#[inline]
fn to_simm32(constant: i64) -> Option<GprMemImm> {
    if constant == ((constant << 32) >> 32) {
        Some(
            GprMemImm::new(RegMemImm::Imm {
                simm32: constant as u32,
            })
            .unwrap(),
        )
    } else {
        None
    }
}

const UINT_MASK: [u8; 16] = [
    0x00, 0x00, 0x30, 0x43, 0x00, 0x00, 0x30, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

const UINT_MASK_HIGH: [u8; 16] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x43,
];

const IADD_PAIRWISE_MUL_CONST_16: [u8; 16] = [0x01; 16];

const IADD_PAIRWISE_MUL_CONST_32: [u8; 16] = [
    0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00,
];

const IADD_PAIRWISE_XOR_CONST_32: [u8; 16] = [
    0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80, 0x00, 0x80,
];

const IADD_PAIRWISE_ADDD_CONST_32: [u8; 16] = [
    0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00,
];
