use crate::ir::{types, Inst, Value, ValueList};
use crate::machinst::{get_output_reg, InsnOutput, LowerCtx, MachInst, RegRenamer};
use alloc::boxed::Box;
use alloc::vec::Vec;
use regalloc::{Reg, Writable};
use smallvec::SmallVec;
use std::cell::Cell;

pub use super::MachLabel;
pub use crate::ir::{ExternalName, FuncRef, GlobalValue, SigRef};
pub use crate::isa::unwind::UnwindInst;
pub use crate::machinst::RelocDistance;

pub type Unit = ();
pub type ValueSlice = (ValueList, usize);
pub type ValueArray2 = [Value; 2];
pub type ValueArray3 = [Value; 3];
pub type WritableReg = Writable<Reg>;
pub type VecReg = Vec<Reg>;
pub type VecWritableReg = Vec<WritableReg>;
pub type ValueRegs = crate::machinst::ValueRegs<Reg>;
pub type InstOutput = SmallVec<[ValueRegs; 2]>;
pub type InstOutputBuilder = Cell<InstOutput>;
pub type VecMachLabel = Vec<MachLabel>;
pub type BoxExternalName = Box<ExternalName>;

/// Helper macro to define methods in `prelude.isle` within `impl Context for
/// ...` for each backend. These methods are shared amongst all backends.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_prelude_methods {
    () => {
        #[inline]
        fn unpack_value_array_2(&mut self, arr: &ValueArray2) -> (Value, Value) {
            let [a, b] = *arr;
            (a, b)
        }

        #[inline]
        fn pack_value_array_2(&mut self, a: Value, b: Value) -> ValueArray2 {
            [a, b]
        }

        #[inline]
        fn unpack_value_array_3(&mut self, arr: &ValueArray3) -> (Value, Value, Value) {
            let [a, b, c] = *arr;
            (a, b, c)
        }

        #[inline]
        fn pack_value_array_3(&mut self, a: Value, b: Value, c: Value) -> ValueArray3 {
            [a, b, c]
        }

        #[inline]
        fn value_reg(&mut self, reg: Reg) -> ValueRegs {
            ValueRegs::one(reg)
        }

        #[inline]
        fn value_regs(&mut self, r1: Reg, r2: Reg) -> ValueRegs {
            ValueRegs::two(r1, r2)
        }

        #[inline]
        fn value_regs_invalid(&mut self) -> ValueRegs {
            ValueRegs::invalid()
        }

        #[inline]
        fn output_none(&mut self) -> InstOutput {
            smallvec::smallvec![]
        }

        #[inline]
        fn output(&mut self, regs: ValueRegs) -> InstOutput {
            smallvec::smallvec![regs]
        }

        #[inline]
        fn output_pair(&mut self, r1: ValueRegs, r2: ValueRegs) -> InstOutput {
            smallvec::smallvec![r1, r2]
        }

        #[inline]
        fn output_builder_new(&mut self) -> InstOutputBuilder {
            std::cell::Cell::new(InstOutput::new())
        }

        #[inline]
        fn output_builder_push(&mut self, builder: &InstOutputBuilder, regs: ValueRegs) -> Unit {
            let mut vec = builder.take();
            vec.push(regs);
            builder.set(vec);
        }

        #[inline]
        fn output_builder_finish(&mut self, builder: &InstOutputBuilder) -> InstOutput {
            builder.take()
        }

        #[inline]
        fn temp_writable_reg(&mut self, ty: Type) -> WritableReg {
            let value_regs = self.lower_ctx.alloc_tmp(ty);
            value_regs.only_reg().unwrap()
        }

        #[inline]
        fn invalid_reg(&mut self) -> Reg {
            Reg::invalid()
        }

        #[inline]
        fn put_in_reg(&mut self, val: Value) -> Reg {
            self.lower_ctx.put_value_in_regs(val).only_reg().unwrap()
        }

        #[inline]
        fn put_in_regs(&mut self, val: Value) -> ValueRegs {
            self.lower_ctx.put_value_in_regs(val)
        }

        #[inline]
        fn value_regs_get(&mut self, regs: ValueRegs, i: usize) -> Reg {
            regs.regs()[i]
        }

        #[inline]
        fn u8_as_u64(&mut self, x: u8) -> u64 {
            x.into()
        }

        #[inline]
        fn u16_as_u64(&mut self, x: u16) -> u64 {
            x.into()
        }

        #[inline]
        fn u32_as_u64(&mut self, x: u32) -> u64 {
            x.into()
        }

        #[inline]
        fn i64_as_u64(&mut self, x: i64) -> u64 {
            x as u64
        }

        #[inline]
        fn u64_add(&mut self, x: u64, y: u64) -> u64 {
            x.wrapping_add(y)
        }

        #[inline]
        fn u64_sub(&mut self, x: u64, y: u64) -> u64 {
            x.wrapping_sub(y)
        }

        #[inline]
        fn u64_and(&mut self, x: u64, y: u64) -> u64 {
            x & y
        }

        #[inline]
        fn ty_bits(&mut self, ty: Type) -> u8 {
            use std::convert::TryInto;
            ty.bits().try_into().unwrap()
        }

        #[inline]
        fn ty_bits_u16(&mut self, ty: Type) -> u16 {
            ty.bits()
        }

        #[inline]
        fn ty_bits_u64(&mut self, ty: Type) -> u64 {
            ty.bits() as u64
        }

        #[inline]
        fn ty_bytes(&mut self, ty: Type) -> u16 {
            u16::try_from(ty.bytes()).unwrap()
        }

        #[inline]
        fn ty_mask(&mut self, ty: Type) -> u64 {
            match ty.bits() {
                1 => 1,
                8 => 0xff,
                16 => 0xffff,
                32 => 0xffff_ffff,
                64 => 0xffff_ffff_ffff_ffff,
                _ => unimplemented!(),
            }
        }

        fn fits_in_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 16 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_32(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 32 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn fits_in_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() <= 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_32_or_64(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 32 || ty.bits() == 64 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_8_or_16(&mut self, ty: Type) -> Option<Type> {
            if ty.bits() == 8 || ty.bits() == 16 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn ty_int_bool_64(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I64 | B64 => Some(ty),
                _ => None,
            }
        }

        #[inline]
        fn ty_int_bool_128(&mut self, ty: Type) -> Option<Type> {
            match ty {
                I128 | B128 => Some(ty),
                _ => None,
            }
        }

        fn vec128(&mut self, ty: Type) -> Option<Type> {
            if ty.is_vector() && ty.bits() == 128 {
                Some(ty)
            } else {
                None
            }
        }

        #[inline]
        fn value_list_slice(&mut self, list: ValueList) -> ValueSlice {
            (list, 0)
        }

        #[inline]
        fn value_slice_empty(&mut self, slice: ValueSlice) -> Option<()> {
            let (list, off) = slice;
            if off >= list.len(&self.lower_ctx.dfg().value_lists) {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn value_slice_unwrap(&mut self, slice: ValueSlice) -> Option<(Value, ValueSlice)> {
            let (list, off) = slice;
            if let Some(val) = list.get(off, &self.lower_ctx.dfg().value_lists) {
                Some((val, (list, off + 1)))
            } else {
                None
            }
        }

        #[inline]
        fn value_slice_len(&mut self, slice: ValueSlice) -> usize {
            let (list, off) = slice;
            list.len(&self.lower_ctx.dfg().value_lists) - off
        }

        #[inline]
        fn value_slice_get(&mut self, slice: ValueSlice, idx: usize) -> Value {
            let (list, off) = slice;
            list.get(off + idx, &self.lower_ctx.dfg().value_lists)
                .unwrap()
        }

        #[inline]
        fn writable_reg_to_reg(&mut self, r: WritableReg) -> Reg {
            r.to_reg()
        }

        #[inline]
        fn u64_from_imm64(&mut self, imm: Imm64) -> u64 {
            imm.bits() as u64
        }

        #[inline]
        fn inst_results(&mut self, inst: Inst) -> ValueSlice {
            (self.lower_ctx.dfg().inst_results_list(inst), 0)
        }

        #[inline]
        fn first_result(&mut self, inst: Inst) -> Option<Value> {
            self.lower_ctx.dfg().inst_results(inst).first().copied()
        }

        #[inline]
        fn inst_data(&mut self, inst: Inst) -> InstructionData {
            self.lower_ctx.dfg()[inst].clone()
        }

        #[inline]
        fn value_type(&mut self, val: Value) -> Type {
            self.lower_ctx.dfg().value_type(val)
        }

        #[inline]
        fn multi_lane(&mut self, ty: Type) -> Option<(u8, u16)> {
            if ty.lane_count() > 1 {
                Some((ty.lane_bits(), ty.lane_count()))
            } else {
                None
            }
        }

        #[inline]
        fn def_inst(&mut self, val: Value) -> Option<Inst> {
            self.lower_ctx.dfg().value_def(val).inst()
        }

        fn u64_from_ieee32(&mut self, val: Ieee32) -> u64 {
            val.bits().into()
        }

        fn u64_from_ieee64(&mut self, val: Ieee64) -> u64 {
            val.bits()
        }

        fn u8_from_uimm8(&mut self, val: Uimm8) -> u8 {
            val
        }

        fn not_i64x2(&mut self, ty: Type) -> Option<()> {
            if ty == I64X2 {
                None
            } else {
                Some(())
            }
        }

        fn trap_code_division_by_zero(&mut self) -> TrapCode {
            TrapCode::IntegerDivisionByZero
        }

        fn trap_code_integer_overflow(&mut self) -> TrapCode {
            TrapCode::IntegerOverflow
        }

        fn trap_code_bad_conversion_to_integer(&mut self) -> TrapCode {
            TrapCode::BadConversionToInteger
        }

        fn avoid_div_traps(&mut self, _: Type) -> Option<()> {
            if self.flags.avoid_div_traps() {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn func_ref_data(&mut self, func_ref: FuncRef) -> (SigRef, ExternalName, RelocDistance) {
            let funcdata = &self.lower_ctx.dfg().ext_funcs[func_ref];
            (
                funcdata.signature,
                funcdata.name.clone(),
                funcdata.reloc_distance(),
            )
        }

        #[inline]
        fn symbol_value_data(
            &mut self,
            global_value: GlobalValue,
        ) -> Option<(ExternalName, RelocDistance, i64)> {
            let (name, reloc, offset) = self.lower_ctx.symbol_value_data(global_value)?;
            Some((name.clone(), reloc, offset))
        }

        #[inline]
        fn reloc_distance_near(&mut self, dist: RelocDistance) -> Option<()> {
            if dist == RelocDistance::Near {
                Some(())
            } else {
                None
            }
        }

        fn nonzero_u64_from_imm64(&mut self, val: Imm64) -> Option<u64> {
            match val.bits() {
                0 => None,
                n => Some(n as u64),
            }
        }

        #[inline]
        fn u32_add(&mut self, a: u32, b: u32) -> u32 {
            a.wrapping_add(b)
        }

        #[inline]
        fn u8_and(&mut self, a: u8, b: u8) -> u8 {
            a & b
        }

        #[inline]
        fn lane_type(&mut self, ty: Type) -> Type {
            ty.lane_type()
        }
    };
}

/// This structure is used to implement the ISLE-generated `Context` trait and
/// internally has a temporary reference to a machinst `LowerCtx`.
pub(crate) struct IsleContext<'a, C: LowerCtx, F, I, const N: usize>
where
    [(C::I, bool); N]: smallvec::Array,
{
    pub lower_ctx: &'a mut C,
    pub flags: &'a F,
    pub isa_flags: &'a I,
    pub emitted_insts: SmallVec<[(C::I, bool); N]>,
}

/// Shared lowering code amongst all backends for doing ISLE-based lowering.
///
/// The `isle_lower` argument here is an ISLE-generated function for `lower` and
/// then this function otherwise handles register mapping and such around the
/// lowering.
pub(crate) fn lower_common<C, F, I, IF, const N: usize>(
    lower_ctx: &mut C,
    flags: &F,
    isa_flags: &I,
    outputs: &[InsnOutput],
    inst: Inst,
    isle_lower: IF,
    map_regs: fn(&mut C::I, &RegRenamer),
) -> Result<(), ()>
where
    C: LowerCtx,
    [(C::I, bool); N]: smallvec::Array<Item = (C::I, bool)>,
    IF: Fn(&mut IsleContext<'_, C, F, I, N>, Inst) -> Option<InstOutput>,
{
    // TODO: reuse the ISLE context across lowerings so we can reuse its
    // internal heap allocations.
    let mut isle_ctx = IsleContext {
        lower_ctx,
        flags,
        isa_flags,
        emitted_insts: SmallVec::new(),
    };

    let temp_regs = isle_lower(&mut isle_ctx, inst).ok_or(())?;

    #[cfg(debug_assertions)]
    {
        debug_assert_eq!(
            temp_regs.len(),
            outputs.len(),
            "the number of temporary values and destination values do \
         not match ({} != {}); ensure the correct registers are being \
         returned.",
            temp_regs.len(),
            outputs.len(),
        );
    }

    // The ISLE generated code emits its own registers to define the
    // instruction's lowered values in. We rename those registers to the
    // registers they were assigned when their value was used as an operand in
    // earlier lowerings.
    let mut renamer = RegRenamer::default();
    for i in 0..outputs.len() {
        let regs = temp_regs[i];
        let dsts = get_output_reg(isle_ctx.lower_ctx, outputs[i]);
        let ty = isle_ctx
            .lower_ctx
            .output_ty(outputs[i].insn, outputs[i].output);
        if ty == types::IFLAGS || ty == types::FFLAGS {
            // Flags values do not occupy any registers.
            assert!(regs.len() == 0);
        } else {
            let (_, tys) = <C::I>::rc_for_type(ty).unwrap();
            assert!(regs.len() == tys.len());
            assert!(regs.len() == dsts.len());
            for ((dst, temp), ty) in dsts.regs().iter().zip(regs.regs().iter()).zip(tys) {
                renamer.add_rename(*temp, dst.to_reg(), *ty);
            }
        }
    }
    for (inst, _) in isle_ctx.emitted_insts.iter_mut() {
        map_regs(inst, &renamer);
    }

    // If any renamed register wasn't actually defined in the ISLE-generated
    // instructions then what we're actually doing is "renaming" an input to a
    // new name which requires manually inserting a `mov` instruction. Note that
    // this typically doesn't happen and is only here for cases where the input
    // is sometimes passed through unmodified to the output, such as
    // zero-extending a 64-bit input to a 128-bit output which doesn't actually
    // change the input and simply produces another zero'd register.
    for (old, new, ty) in renamer.unmapped_defs() {
        isle_ctx
            .lower_ctx
            .emit(<C::I>::gen_move(Writable::from_reg(new), old, ty));
    }

    // Once everything is remapped we forward all emitted instructions to the
    // `lower_ctx`. Note that this happens after the synthetic mov's above in
    // case any of these instruction use those movs.
    for (inst, is_safepoint) in isle_ctx.emitted_insts {
        if is_safepoint {
            lower_ctx.emit_safepoint(inst);
        } else {
            lower_ctx.emit(inst);
        }
    }

    Ok(())
}
