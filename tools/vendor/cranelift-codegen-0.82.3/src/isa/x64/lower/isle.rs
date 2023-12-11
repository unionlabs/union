//! ISLE integration glue code for x64 lowering.

// Pull in the ISLE generated code.
pub(crate) mod generated_code;
use generated_code::MInst;
use regalloc::Writable;

// Types that the generated ISLE code uses via `use super::*`.
use super::{is_int_or_ref_ty, is_mergeable_load, lower_to_amode, Reg};
use crate::{
    ir::{
        condcodes::FloatCC, immediates::*, types::*, Inst, InstructionData, Opcode, TrapCode,
        Value, ValueLabel, ValueList,
    },
    isa::{
        settings::Flags,
        unwind::UnwindInst,
        x64::{
            inst::{args::*, regs, x64_map_regs},
            settings::Flags as IsaFlags,
        },
    },
    machinst::{
        isle::*, AtomicRmwOp, InsnInput, InsnOutput, LowerCtx, VCodeConstant, VCodeConstantData,
    },
};
use std::convert::TryFrom;

pub struct SinkableLoad {
    inst: Inst,
    addr_input: InsnInput,
    offset: i32,
}

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
        x64_map_regs,
    )
}

impl<C> generated_code::Context for IsleContext<'_, C, Flags, IsaFlags, 6>
where
    C: LowerCtx<I = MInst>,
{
    isle_prelude_methods!();

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

            // Generate constants fresh at each use to minimize long-range
            // register pressure.
            let ty = self.value_type(val);
            return RegMemImm::reg(generated_code::constructor_imm(self, ty, c).unwrap());
        }

        if let Some((src_insn, 0)) = inputs.inst {
            if let Some((addr_input, offset)) = is_mergeable_load(self.lower_ctx, src_insn) {
                self.lower_ctx.sink_inst(src_insn);
                let amode = lower_to_amode(self.lower_ctx, addr_input, offset);
                return RegMemImm::mem(amode);
            }
        }

        RegMemImm::reg(self.put_in_reg(val))
    }

    fn put_in_reg_mem(&mut self, val: Value) -> RegMem {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            // Generate constants fresh at each use to minimize long-range
            // register pressure.
            let ty = self.value_type(val);
            return RegMem::reg(generated_code::constructor_imm(self, ty, c).unwrap());
        }

        if let Some((src_insn, 0)) = inputs.inst {
            if let Some((addr_input, offset)) = is_mergeable_load(self.lower_ctx, src_insn) {
                self.lower_ctx.sink_inst(src_insn);
                let amode = lower_to_amode(self.lower_ctx, addr_input, offset);
                return RegMem::mem(amode);
            }
        }

        RegMem::reg(self.put_in_reg(val))
    }

    fn put_masked_in_imm8_gpr(&mut self, val: Value, ty: Type) -> Imm8Gpr {
        let inputs = self.lower_ctx.get_value_as_source_or_const(val);

        if let Some(c) = inputs.constant {
            let mask = 1_u64
                .checked_shl(ty.bits() as u32)
                .map_or(u64::MAX, |x| x - 1);
            return Imm8Gpr::new(Imm8Reg::Imm8 {
                imm: (c & mask) as u8,
            })
            .unwrap();
        }

        Imm8Gpr::new(Imm8Reg::Reg {
            reg: self.put_in_regs(val).regs()[0],
        })
        .unwrap()
    }

    #[inline]
    fn encode_fcmp_imm(&mut self, imm: &FcmpImm) -> u8 {
        imm.encode()
    }

    #[inline]
    fn avx512vl_enabled(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_avx512vl_simd() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn avx512dq_enabled(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_avx512dq_simd() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn avx512f_enabled(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_avx512f_simd() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn avx512bitalg_enabled(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_avx512bitalg_simd() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn use_lzcnt(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_lzcnt() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn use_bmi1(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_bmi1() {
            Some(())
        } else {
            None
        }
    }

    #[inline]
    fn use_popcnt(&mut self, _: Type) -> Option<()> {
        if self.isa_flags.use_popcnt() {
            Some(())
        } else {
            None
        }
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
        let mask = 1_u64
            .checked_shl(ty.bits() as u32)
            .map_or(u64::MAX, |x| x - 1);
        Imm8Gpr::new(Imm8Reg::Imm8 {
            imm: (c & mask) as u8,
        })
        .unwrap()
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
        if let Some((inst, 0)) = input.inst {
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

    fn sink_load(&mut self, load: &SinkableLoad) -> RegMemImm {
        self.lower_ctx.sink_inst(load.inst);
        let addr = lower_to_amode(self.lower_ctx, load.addr_input, load.offset);
        RegMemImm::Mem {
            addr: SyntheticAmode::Real(addr),
        }
    }

    #[inline]
    fn ext_mode(&mut self, from_bits: u16, to_bits: u16) -> ExtMode {
        ExtMode::new(from_bits, to_bits).unwrap()
    }

    fn emit(&mut self, inst: &MInst) -> Unit {
        for inst in inst.clone().mov_mitosis() {
            self.emitted_insts.push((inst, false));
        }
    }

    fn emit_safepoint(&mut self, inst: &MInst) -> Unit {
        use crate::machinst::MachInst;
        for inst in inst.clone().mov_mitosis() {
            let is_safepoint = !inst.is_move().is_some();
            self.emitted_insts.push((inst, is_safepoint));
        }
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
    fn xmm0(&mut self) -> WritableXmm {
        WritableXmm::from_reg(Xmm::new(regs::xmm0()).unwrap())
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
    fn amode_to_synthetic_amode(&mut self, amode: &Amode) -> SyntheticAmode {
        amode.clone().into()
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

    fn is_gpr_type(&mut self, ty: Type) -> Option<Type> {
        if is_int_or_ref_ty(ty) || ty == I128 || ty == B128 {
            Some(ty)
        } else {
            None
        }
    }

    #[inline]
    fn is_xmm_type(&mut self, ty: Type) -> Option<Type> {
        if ty == F32 || ty == F64 || (ty.is_vector() && ty.bits() == 128) {
            Some(ty)
        } else {
            None
        }
    }

    #[inline]
    fn is_single_register_type(&mut self, ty: Type) -> Option<Type> {
        if ty != I128 {
            Some(ty)
        } else {
            None
        }
    }
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
