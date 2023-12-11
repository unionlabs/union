use crate::ir::{types, Inst, Value, ValueList};
use crate::machinst::{get_output_reg, InsnOutput};
use alloc::boxed::Box;
use alloc::vec::Vec;
use smallvec::SmallVec;
use std::cell::Cell;
use target_lexicon::Triple;

pub use super::MachLabel;
use super::RetPair;
pub use crate::ir::{
    condcodes, condcodes::CondCode, dynamic_to_fixed, ArgumentExtension, ArgumentPurpose, Constant,
    DynamicStackSlot, ExternalName, FuncRef, GlobalValue, Immediate, SigRef, StackSlot,
};
pub use crate::isa::unwind::UnwindInst;
pub use crate::machinst::{
    ABIArg, ABIArgSlot, InputSourceInst, Lower, RealReg, Reg, RelocDistance, Sig, VCodeInst,
    Writable,
};
pub use crate::settings::TlsModel;

pub type Unit = ();
pub type ValueSlice = (ValueList, usize);
pub type ValueArray2 = [Value; 2];
pub type ValueArray3 = [Value; 3];
pub type WritableReg = Writable<Reg>;
pub type VecRetPair = Vec<RetPair>;
pub type VecMask = Vec<u8>;
pub type ValueRegs = crate::machinst::ValueRegs<Reg>;
pub type WritableValueRegs = crate::machinst::ValueRegs<WritableReg>;
pub type InstOutput = SmallVec<[ValueRegs; 2]>;
pub type InstOutputBuilder = Cell<InstOutput>;
pub type BoxExternalName = Box<ExternalName>;
pub type Range = (usize, usize);

pub enum RangeView {
    Empty,
    NonEmpty { index: usize, rest: Range },
}

/// Helper macro to define methods in `prelude.isle` within `impl Context for
/// ...` for each backend. These methods are shared amongst all backends.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_lower_prelude_methods {
    () => {
        isle_common_prelude_methods!();

        #[inline]
        fn same_value(&mut self, a: Value, b: Value) -> Option<Value> {
            if a == b {
                Some(a)
            } else {
                None
            }
        }

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
        fn is_valid_reg(&mut self, reg: Reg) -> bool {
            use crate::machinst::valueregs::InvalidSentinel;
            !reg.is_invalid_sentinel()
        }

        #[inline]
        fn invalid_reg(&mut self) -> Reg {
            use crate::machinst::valueregs::InvalidSentinel;
            Reg::invalid_sentinel()
        }

        #[inline]
        fn mark_value_used(&mut self, val: Value) {
            self.lower_ctx.increment_lowered_uses(val);
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
        fn ensure_in_vreg(&mut self, reg: Reg, ty: Type) -> Reg {
            self.lower_ctx.ensure_in_vreg(reg, ty)
        }

        #[inline]
        fn value_regs_get(&mut self, regs: ValueRegs, i: usize) -> Reg {
            regs.regs()[i]
        }

        #[inline]
        fn value_regs_len(&mut self, regs: ValueRegs) -> usize {
            regs.regs().len()
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
        fn inst_results(&mut self, inst: Inst) -> ValueSlice {
            (self.lower_ctx.dfg().inst_results_list(inst), 0)
        }

        #[inline]
        fn first_result(&mut self, inst: Inst) -> Option<Value> {
            self.lower_ctx.dfg().inst_results(inst).first().copied()
        }

        #[inline]
        fn inst_data(&mut self, inst: Inst) -> InstructionData {
            self.lower_ctx.dfg()[inst]
        }

        #[inline]
        fn value_type(&mut self, val: Value) -> Type {
            self.lower_ctx.dfg().value_type(val)
        }

        #[inline]
        fn def_inst(&mut self, val: Value) -> Option<Inst> {
            self.lower_ctx.dfg().value_def(val).inst()
        }

        fn zero_value(&mut self, value: Value) -> Option<Value> {
            let insn = self.def_inst(value);
            if insn.is_some() {
                let insn = insn.unwrap();
                let inst_data = self.lower_ctx.data(insn);
                match inst_data {
                    InstructionData::Unary {
                        opcode: Opcode::Splat,
                        arg,
                    } => {
                        let arg = arg.clone();
                        return self.zero_value(arg);
                    }
                    InstructionData::UnaryConst {
                        opcode: Opcode::Vconst,
                        constant_handle,
                    } => {
                        let constant_data =
                            self.lower_ctx.get_constant_data(*constant_handle).clone();
                        if constant_data.into_vec().iter().any(|&x| x != 0) {
                            return None;
                        } else {
                            return Some(value);
                        }
                    }
                    InstructionData::UnaryImm { imm, .. } => {
                        if imm.bits() == 0 {
                            return Some(value);
                        } else {
                            return None;
                        }
                    }
                    InstructionData::UnaryIeee32 { imm, .. } => {
                        if imm.bits() == 0 {
                            return Some(value);
                        } else {
                            return None;
                        }
                    }
                    InstructionData::UnaryIeee64 { imm, .. } => {
                        if imm.bits() == 0 {
                            return Some(value);
                        } else {
                            return None;
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }

        fn avoid_div_traps(&mut self, _: Type) -> Option<()> {
            if self.flags.avoid_div_traps() {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn tls_model(&mut self, _: Type) -> TlsModel {
            self.flags.tls_model()
        }

        #[inline]
        fn tls_model_is_elf_gd(&mut self) -> Option<()> {
            if self.flags.tls_model() == TlsModel::ElfGd {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn tls_model_is_macho(&mut self) -> Option<()> {
            if self.flags.tls_model() == TlsModel::Macho {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn tls_model_is_coff(&mut self) -> Option<()> {
            if self.flags.tls_model() == TlsModel::Coff {
                Some(())
            } else {
                None
            }
        }

        #[inline]
        fn preserve_frame_pointers(&mut self) -> Option<()> {
            if self.flags.preserve_frame_pointers() {
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
        fn box_external_name(&mut self, extname: ExternalName) -> BoxExternalName {
            Box::new(extname)
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

        #[inline]
        fn u128_from_immediate(&mut self, imm: Immediate) -> Option<u128> {
            let bytes = self.lower_ctx.get_immediate_data(imm).as_slice();
            Some(u128::from_le_bytes(bytes.try_into().ok()?))
        }

        #[inline]
        fn vec_mask_from_immediate(&mut self, imm: Immediate) -> Option<VecMask> {
            let data = self.lower_ctx.get_immediate_data(imm);
            if data.len() == 16 {
                Some(Vec::from(data.as_slice()))
            } else {
                None
            }
        }

        #[inline]
        fn u64_from_constant(&mut self, constant: Constant) -> Option<u64> {
            let bytes = self.lower_ctx.get_constant_data(constant).as_slice();
            Some(u64::from_le_bytes(bytes.try_into().ok()?))
        }

        #[inline]
        fn u128_from_constant(&mut self, constant: Constant) -> Option<u128> {
            let bytes = self.lower_ctx.get_constant_data(constant).as_slice();
            Some(u128::from_le_bytes(bytes.try_into().ok()?))
        }

        #[inline]
        fn emit_u64_le_const(&mut self, value: u64) -> VCodeConstant {
            let data = VCodeConstantData::U64(value.to_le_bytes());
            self.lower_ctx.use_constant(data)
        }

        #[inline]
        fn emit_u128_le_const(&mut self, value: u128) -> VCodeConstant {
            let data = VCodeConstantData::Generated(value.to_le_bytes().as_slice().into());
            self.lower_ctx.use_constant(data)
        }

        #[inline]
        fn const_to_vconst(&mut self, constant: Constant) -> VCodeConstant {
            self.lower_ctx.use_constant(VCodeConstantData::Pool(
                constant,
                self.lower_ctx.get_constant_data(constant).clone(),
            ))
        }

        fn only_writable_reg(&mut self, regs: WritableValueRegs) -> Option<WritableReg> {
            regs.only_reg()
        }

        fn writable_regs_get(&mut self, regs: WritableValueRegs, idx: usize) -> WritableReg {
            regs.regs()[idx]
        }

        fn abi_num_args(&mut self, abi: &Sig) -> usize {
            self.lower_ctx.sigs().num_args(*abi)
        }

        fn abi_get_arg(&mut self, abi: &Sig, idx: usize) -> ABIArg {
            self.lower_ctx.sigs().get_arg(*abi, idx)
        }

        fn abi_num_rets(&mut self, abi: &Sig) -> usize {
            self.lower_ctx.sigs().num_rets(*abi)
        }

        fn abi_get_ret(&mut self, abi: &Sig, idx: usize) -> ABIArg {
            self.lower_ctx.sigs().get_ret(*abi, idx)
        }

        fn abi_ret_arg(&mut self, abi: &Sig) -> Option<ABIArg> {
            self.lower_ctx.sigs().get_ret_arg(*abi)
        }

        fn abi_no_ret_arg(&mut self, abi: &Sig) -> Option<()> {
            if let Some(_) = self.lower_ctx.sigs().get_ret_arg(*abi) {
                None
            } else {
                Some(())
            }
        }

        fn abi_sized_stack_arg_space(&mut self, abi: &Sig) -> i64 {
            self.lower_ctx.sigs()[*abi].sized_stack_arg_space()
        }

        fn abi_sized_stack_ret_space(&mut self, abi: &Sig) -> i64 {
            self.lower_ctx.sigs()[*abi].sized_stack_ret_space()
        }

        fn abi_arg_only_slot(&mut self, arg: &ABIArg) -> Option<ABIArgSlot> {
            match arg {
                &ABIArg::Slots { ref slots, .. } => {
                    if slots.len() == 1 {
                        Some(slots[0])
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        fn abi_arg_struct_pointer(&mut self, arg: &ABIArg) -> Option<(ABIArgSlot, i64, u64)> {
            match arg {
                &ABIArg::StructArg {
                    pointer,
                    offset,
                    size,
                    ..
                } => {
                    if let Some(pointer) = pointer {
                        Some((pointer, offset, size))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        fn abi_arg_implicit_pointer(&mut self, arg: &ABIArg) -> Option<(ABIArgSlot, i64, Type)> {
            match arg {
                &ABIArg::ImplicitPtrArg {
                    pointer,
                    offset,
                    ty,
                    ..
                } => Some((pointer, offset, ty)),
                _ => None,
            }
        }

        fn abi_stackslot_addr(
            &mut self,
            dst: WritableReg,
            stack_slot: StackSlot,
            offset: Offset32,
        ) -> MInst {
            let offset = u32::try_from(i32::from(offset)).unwrap();
            self.lower_ctx
                .abi()
                .sized_stackslot_addr(stack_slot, offset, dst)
        }

        fn abi_dynamic_stackslot_addr(
            &mut self,
            dst: WritableReg,
            stack_slot: DynamicStackSlot,
        ) -> MInst {
            assert!(self
                .lower_ctx
                .abi()
                .dynamic_stackslot_offsets()
                .is_valid(stack_slot));
            self.lower_ctx.abi().dynamic_stackslot_addr(stack_slot, dst)
        }

        fn real_reg_to_reg(&mut self, reg: RealReg) -> Reg {
            Reg::from(reg)
        }

        fn real_reg_to_writable_reg(&mut self, reg: RealReg) -> WritableReg {
            Writable::from_reg(Reg::from(reg))
        }

        fn is_sinkable_inst(&mut self, val: Value) -> Option<Inst> {
            let input = self.lower_ctx.get_value_as_source_or_const(val);

            if let InputSourceInst::UniqueUse(inst, _) = input.inst {
                Some(inst)
            } else {
                None
            }
        }

        #[inline]
        fn sink_inst(&mut self, inst: Inst) {
            self.lower_ctx.sink_inst(inst);
        }

        #[inline]
        fn preg_to_reg(&mut self, preg: PReg) -> Reg {
            preg.into()
        }

        #[inline]
        fn gen_move(&mut self, ty: Type, dst: WritableReg, src: Reg) -> MInst {
            MInst::gen_move(dst, src, ty)
        }

        #[inline]
        fn intcc_reverse(&mut self, cc: &IntCC) -> IntCC {
            cc.reverse()
        }

        #[inline]
        fn intcc_inverse(&mut self, cc: &IntCC) -> IntCC {
            cc.inverse()
        }

        #[inline]
        fn floatcc_reverse(&mut self, cc: &FloatCC) -> FloatCC {
            cc.reverse()
        }

        #[inline]
        fn floatcc_inverse(&mut self, cc: &FloatCC) -> FloatCC {
            cc.inverse()
        }

        /// Generate the return instruction.
        fn gen_return(&mut self, (list, off): ValueSlice) {
            let rets = (off..list.len(&self.lower_ctx.dfg().value_lists))
                .map(|ix| {
                    let val = list.get(ix, &self.lower_ctx.dfg().value_lists).unwrap();
                    self.put_in_regs(val)
                })
                .collect();
            self.lower_ctx.gen_return(rets);
        }
    };
}

/// Helpers specifically for machines that use ABICaller.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_prelude_caller_methods {
    ($abispec:ty, $abicaller:ty) => {
        fn gen_call(
            &mut self,
            sig_ref: SigRef,
            extname: ExternalName,
            dist: RelocDistance,
            args @ (inputs, off): ValueSlice,
        ) -> InstOutput {
            let caller_conv = self.lower_ctx.abi().call_conv(self.lower_ctx.sigs());
            let sig = &self.lower_ctx.dfg().signatures[sig_ref];
            let num_rets = sig.returns.len();
            let abi = self.lower_ctx.sigs().abi_sig_for_sig_ref(sig_ref);
            let caller = <$abicaller>::from_func(
                self.lower_ctx.sigs(),
                sig_ref,
                &extname,
                dist,
                caller_conv,
                self.flags.clone(),
            )
            .unwrap();

            assert_eq!(
                inputs.len(&self.lower_ctx.dfg().value_lists) - off,
                sig.params.len()
            );

            self.gen_call_common(abi, num_rets, caller, args)
        }

        fn gen_call_indirect(
            &mut self,
            sig_ref: SigRef,
            val: Value,
            args @ (inputs, off): ValueSlice,
        ) -> InstOutput {
            let caller_conv = self.lower_ctx.abi().call_conv(self.lower_ctx.sigs());
            let ptr = self.put_in_reg(val);
            let sig = &self.lower_ctx.dfg().signatures[sig_ref];
            let num_rets = sig.returns.len();
            let abi = self.lower_ctx.sigs().abi_sig_for_sig_ref(sig_ref);
            let caller = <$abicaller>::from_ptr(
                self.lower_ctx.sigs(),
                sig_ref,
                ptr,
                Opcode::CallIndirect,
                caller_conv,
                self.flags.clone(),
            )
            .unwrap();

            assert_eq!(
                inputs.len(&self.lower_ctx.dfg().value_lists) - off,
                sig.params.len()
            );

            self.gen_call_common(abi, num_rets, caller, args)
        }
    };
}

/// Helpers for the above ISLE prelude implementations. Meant to go
/// inside the `impl` for the context type, not the trait impl.
#[macro_export]
#[doc(hidden)]
macro_rules! isle_prelude_method_helpers {
    ($abicaller:ty) => {
        fn gen_call_common(
            &mut self,
            abi: Sig,
            num_rets: usize,
            mut caller: $abicaller,
            (inputs, off): ValueSlice,
        ) -> InstOutput {
            caller.emit_stack_pre_adjust(self.lower_ctx);

            let num_args = self.lower_ctx.sigs().num_args(abi);

            assert_eq!(
                inputs.len(&self.lower_ctx.dfg().value_lists) - off,
                num_args
            );
            let mut arg_regs = vec![];
            for i in 0..num_args {
                let input = inputs
                    .get(off + i, &self.lower_ctx.dfg().value_lists)
                    .unwrap();
                arg_regs.push(self.lower_ctx.put_value_in_regs(input));
            }
            for (i, arg_regs) in arg_regs.iter().enumerate() {
                caller.emit_copy_regs_to_buffer(self.lower_ctx, i, *arg_regs);
            }
            for (i, arg_regs) in arg_regs.iter().enumerate() {
                for inst in caller.gen_arg(self.lower_ctx, i, *arg_regs) {
                    self.lower_ctx.emit(inst);
                }
            }

            // Handle retvals prior to emitting call, so the
            // constraints are on the call instruction; but buffer the
            // instructions till after the call.
            let mut outputs = InstOutput::new();
            let mut retval_insts: crate::machinst::abi::SmallInstVec<_> = smallvec::smallvec![];
            // We take the *last* `num_rets` returns of the sig:
            // this skips a StructReturn, if any, that is present.
            let sigdata_num_rets = self.lower_ctx.sigs().num_rets(abi);
            debug_assert!(num_rets <= sigdata_num_rets);
            for i in (sigdata_num_rets - num_rets)..sigdata_num_rets {
                // Borrow `sigdata` again so we don't hold a `self`
                // borrow across the `&mut self` arg to
                // `abi_arg_slot_regs()` below.
                let ret = self.lower_ctx.sigs().get_ret(abi, i);
                let retval_regs = self.abi_arg_slot_regs(&ret).unwrap();
                retval_insts.extend(
                    caller
                        .gen_retval(self.lower_ctx, i, retval_regs.clone())
                        .into_iter(),
                );
                outputs.push(valueregs::non_writable_value_regs(retval_regs));
            }

            caller.emit_call(self.lower_ctx);

            for inst in retval_insts {
                self.lower_ctx.emit(inst);
            }

            caller.emit_stack_post_adjust(self.lower_ctx);

            outputs
        }

        fn abi_arg_slot_regs(&mut self, arg: &ABIArg) -> Option<WritableValueRegs> {
            match arg {
                &ABIArg::Slots { ref slots, .. } => match slots.len() {
                    1 => {
                        let a = self.temp_writable_reg(slots[0].get_type());
                        Some(WritableValueRegs::one(a))
                    }
                    2 => {
                        let a = self.temp_writable_reg(slots[0].get_type());
                        let b = self.temp_writable_reg(slots[1].get_type());
                        Some(WritableValueRegs::two(a, b))
                    }
                    _ => panic!("Expected to see one or two slots only from {:?}", arg),
                },
                _ => None,
            }
        }
    };
}

/// This structure is used to implement the ISLE-generated `Context` trait and
/// internally has a temporary reference to a machinst `LowerCtx`.
pub(crate) struct IsleContext<'a, 'b, I, Flags, IsaFlags, const N: usize>
where
    I: VCodeInst,
    [(I, bool); N]: smallvec::Array,
{
    pub lower_ctx: &'a mut Lower<'b, I>,
    pub triple: &'a Triple,
    pub flags: &'a Flags,
    pub isa_flags: &'a IsaFlags,
}

/// Shared lowering code amongst all backends for doing ISLE-based lowering.
///
/// The `isle_lower` argument here is an ISLE-generated function for `lower` and
/// then this function otherwise handles register mapping and such around the
/// lowering.
pub(crate) fn lower_common<I, Flags, IsaFlags, IsleFunction, const N: usize>(
    lower_ctx: &mut Lower<I>,
    triple: &Triple,
    flags: &Flags,
    isa_flags: &IsaFlags,
    outputs: &[InsnOutput],
    inst: Inst,
    isle_lower: IsleFunction,
) -> Result<(), ()>
where
    I: VCodeInst,
    [(I, bool); N]: smallvec::Array<Item = (I, bool)>,
    IsleFunction: Fn(&mut IsleContext<'_, '_, I, Flags, IsaFlags, N>, Inst) -> Option<InstOutput>,
{
    // TODO: reuse the ISLE context across lowerings so we can reuse its
    // internal heap allocations.
    let mut isle_ctx = IsleContext {
        lower_ctx,
        triple,
        flags,
        isa_flags,
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
    // instruction's lowered values in. However, other instructions
    // that use this SSA value will be lowered assuming that the value
    // is generated into a pre-assigned, different, register.
    //
    // To connect the two, we set up "aliases" in the VCodeBuilder
    // that apply when it is building the Operand table for the
    // regalloc to use. These aliases effectively rewrite any use of
    // the pre-assigned register to the register that was returned by
    // the ISLE lowering logic.
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
            for (dst, temp) in dsts.regs().iter().zip(regs.regs().iter()) {
                isle_ctx.lower_ctx.set_vreg_alias(dst.to_reg(), *temp);
            }
        }
    }

    Ok(())
}
