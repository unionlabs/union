//! ISLE integration glue code for riscv64 lowering.

// Pull in the ISLE generated code.
#[allow(unused)]
pub mod generated_code;
use generated_code::{Context, MInst};

// Types that the generated ISLE code uses via `use super::*`.
use super::{writable_zero_reg, zero_reg};
use crate::isa::riscv64::abi::Riscv64ABICaller;
use crate::isa::riscv64::settings::Flags as IsaFlags;
use crate::machinst::Reg;
use crate::machinst::{isle::*, MachInst, SmallInstVec};
use crate::machinst::{VCodeConstant, VCodeConstantData};
use crate::settings::Flags;
use crate::{
    ir::{
        immediates::*, types::*, AtomicRmwOp, ExternalName, Inst, InstructionData, MemFlags,
        StackSlot, TrapCode, Value, ValueList,
    },
    isa::riscv64::inst::*,
    machinst::{ArgPair, InsnOutput, Lower},
};
use crate::{isle_common_prelude_methods, isle_lower_prelude_methods};
use regalloc2::PReg;
use std::boxed::Box;
use std::convert::TryFrom;
use std::vec::Vec;
use target_lexicon::Triple;

type BoxCallInfo = Box<CallInfo>;
type BoxCallIndInfo = Box<CallIndInfo>;
type BoxExternalName = Box<ExternalName>;
type VecMachLabel = Vec<MachLabel>;
type VecArgPair = Vec<ArgPair>;
use crate::machinst::valueregs;

/// The main entry point for lowering with ISLE.
pub(crate) fn lower(
    lower_ctx: &mut Lower<MInst>,
    flags: &Flags,
    triple: &Triple,
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

impl IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_prelude_method_helpers!(Riscv64ABICaller);
}

impl generated_code::Context for IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    isle_lower_prelude_methods!();
    isle_prelude_caller_methods!(Riscv64MachineDeps, Riscv64ABICaller);

    fn vec_writable_to_regs(&mut self, val: &VecWritableReg) -> ValueRegs {
        match val.len() {
            1 => ValueRegs::one(val[0].to_reg()),
            2 => ValueRegs::two(val[0].to_reg(), val[1].to_reg()),
            _ => unreachable!(),
        }
    }

    fn lower_br_fcmp(
        &mut self,
        cc: &FloatCC,
        a: Reg,
        b: Reg,
        targets: &VecMachLabel,
        ty: Type,
    ) -> InstOutput {
        let tmp = self.temp_writable_reg(I64);
        MInst::lower_br_fcmp(
            *cc,
            a,
            b,
            BranchTarget::Label(targets[0]),
            BranchTarget::Label(targets[1]),
            ty,
            tmp,
        )
        .iter()
        .for_each(|i| self.emit(i));
        InstOutput::default()
    }

    fn lower_brz_or_nz(
        &mut self,
        cc: &IntCC,
        a: ValueRegs,
        targets: &VecMachLabel,
        ty: Type,
    ) -> InstOutput {
        MInst::lower_br_icmp(
            *cc,
            a,
            self.int_zero_reg(ty),
            BranchTarget::Label(targets[0]),
            BranchTarget::Label(targets[1]),
            ty,
        )
        .iter()
        .for_each(|i| self.emit(i));
        InstOutput::default()
    }
    fn lower_br_icmp(
        &mut self,
        cc: &IntCC,
        a: ValueRegs,
        b: ValueRegs,
        targets: &VecMachLabel,
        ty: Type,
    ) -> InstOutput {
        let test = generated_code::constructor_lower_icmp(self, cc, a, b, ty).unwrap();
        self.emit(&MInst::CondBr {
            taken: BranchTarget::Label(targets[0]),
            not_taken: BranchTarget::Label(targets[1]),
            kind: IntegerCompare {
                kind: IntCC::NotEqual,
                rs1: test,
                rs2: zero_reg(),
            },
        });
        InstOutput::default()
    }
    fn load_ra(&mut self) -> Reg {
        if self.flags.preserve_frame_pointers() {
            let tmp = self.temp_writable_reg(I64);
            self.emit(&MInst::Load {
                rd: tmp,
                op: LoadOP::Ld,
                flags: MemFlags::trusted(),
                from: AMode::FPOffset(8, I64),
            });
            tmp.to_reg()
        } else {
            self.gen_move2(link_reg(), I64, I64)
        }
    }
    fn int_zero_reg(&mut self, ty: Type) -> ValueRegs {
        assert!(ty.is_int(), "{:?}", ty);
        if ty.bits() == 128 {
            ValueRegs::two(self.zero_reg(), self.zero_reg())
        } else {
            ValueRegs::one(self.zero_reg())
        }
    }

    fn vec_label_get(&mut self, val: &VecMachLabel, x: u8) -> MachLabel {
        val[x as usize]
    }

    fn label_to_br_target(&mut self, label: MachLabel) -> BranchTarget {
        BranchTarget::Label(label)
    }

    fn vec_writable_clone(&mut self, v: &VecWritableReg) -> VecWritableReg {
        v.clone()
    }

    fn gen_moves(&mut self, rs: ValueRegs, in_ty: Type, out_ty: Type) -> ValueRegs {
        let tmp = construct_dest(|ty| self.temp_writable_reg(ty), out_ty);
        if in_ty.bits() < 64 {
            self.emit(&gen_move(tmp.regs()[0], out_ty, rs.regs()[0], in_ty));
        } else {
            gen_moves(tmp.regs(), rs.regs())
                .iter()
                .for_each(|i| self.emit(i));
        }
        tmp.map(|r| r.to_reg())
    }
    fn imm12_and(&mut self, imm: Imm12, x: i32) -> Imm12 {
        Imm12::from_bits(imm.as_i16() & (x as i16))
    }
    fn alloc_vec_writable(&mut self, ty: Type) -> VecWritableReg {
        if ty.is_int() || ty == R32 || ty == R64 {
            if ty.bits() <= 64 {
                vec![self.temp_writable_reg(I64)]
            } else {
                vec![self.temp_writable_reg(I64), self.temp_writable_reg(I64)]
            }
        } else if ty.is_float() {
            vec![self.temp_writable_reg(ty)]
        } else {
            unimplemented!("ty:{:?}", ty)
        }
    }

    fn imm(&mut self, ty: Type, val: u64) -> Reg {
        let tmp = self.temp_writable_reg(ty);
        self.emit_list(&MInst::load_constant_u64(tmp, val));
        tmp.to_reg()
    }
    #[inline]
    fn emit(&mut self, arg0: &MInst) -> Unit {
        self.lower_ctx.emit(arg0.clone());
    }
    #[inline]
    fn imm12_from_u64(&mut self, arg0: u64) -> Option<Imm12> {
        Imm12::maybe_from_u64(arg0)
    }
    #[inline]
    fn writable_zero_reg(&mut self) -> WritableReg {
        writable_zero_reg()
    }
    #[inline]
    fn neg_imm12(&mut self, arg0: Imm12) -> Imm12 {
        -arg0
    }
    #[inline]
    fn zero_reg(&mut self) -> Reg {
        zero_reg()
    }
    #[inline]
    fn imm_from_bits(&mut self, val: u64) -> Imm12 {
        Imm12::maybe_from_u64(val).unwrap()
    }
    #[inline]
    fn imm_from_neg_bits(&mut self, val: i64) -> Imm12 {
        Imm12::maybe_from_u64(val as u64).unwrap()
    }

    fn gen_default_frm(&mut self) -> OptionFloatRoundingMode {
        None
    }
    fn gen_select_reg(&mut self, cc: &IntCC, a: Reg, b: Reg, rs1: Reg, rs2: Reg) -> Reg {
        let rd = self.temp_writable_reg(MInst::canonical_type_for_rc(rs1.class()));
        self.emit(&MInst::SelectReg {
            rd,
            rs1,
            rs2,
            condition: IntegerCompare {
                kind: *cc,
                rs1: a,
                rs2: b,
            },
        });
        rd.to_reg()
    }
    fn load_u64_constant(&mut self, val: u64) -> Reg {
        let rd = self.temp_writable_reg(I64);
        MInst::load_constant_u64(rd, val)
            .iter()
            .for_each(|i| self.emit(i));
        rd.to_reg()
    }
    fn u8_as_i32(&mut self, x: u8) -> i32 {
        x as i32
    }

    fn ext_sign_bit(&mut self, ty: Type, r: Reg) -> Reg {
        assert!(ty.is_int());
        let rd = self.temp_writable_reg(I64);
        self.emit(&MInst::AluRRImm12 {
            alu_op: AluOPRRI::Bexti,
            rd,
            rs: r,
            imm12: Imm12::from_bits((ty.bits() - 1) as i16),
        });
        rd.to_reg()
    }
    fn imm12_const(&mut self, val: i32) -> Imm12 {
        if let Some(res) = Imm12::maybe_from_u64(val as u64) {
            res
        } else {
            panic!("Unable to make an Imm12 value from {}", val)
        }
    }
    fn imm12_const_add(&mut self, val: i32, add: i32) -> Imm12 {
        Imm12::maybe_from_u64((val + add) as u64).unwrap()
    }

    //
    fn gen_shamt(&mut self, ty: Type, shamt: Reg) -> ValueRegs {
        let shamt = {
            let tmp = self.temp_writable_reg(I64);
            self.emit(&MInst::AluRRImm12 {
                alu_op: AluOPRRI::Andi,
                rd: tmp,
                rs: shamt,
                imm12: Imm12::from_bits((ty.bits() - 1) as i16),
            });
            tmp.to_reg()
        };
        let len_sub_shamt = {
            let len_sub_shamt = self.temp_writable_reg(I64);
            self.emit(&MInst::load_imm12(
                len_sub_shamt,
                Imm12::from_bits(ty.bits() as i16),
            ));
            self.emit(&MInst::AluRRR {
                alu_op: AluOPRRR::Sub,
                rd: len_sub_shamt,
                rs1: len_sub_shamt.to_reg(),
                rs2: shamt,
            });
            len_sub_shamt.to_reg()
        };
        ValueRegs::two(shamt, len_sub_shamt)
    }

    fn has_b(&mut self) -> Option<bool> {
        Some(self.isa_flags.has_b())
    }
    fn has_zbkb(&mut self) -> Option<bool> {
        Some(self.isa_flags.has_zbkb())
    }

    fn valueregs_2_reg(&mut self, val: Value) -> Reg {
        self.put_in_regs(val).regs()[0]
    }

    fn inst_output_get(&mut self, x: InstOutput, index: u8) -> ValueRegs {
        x[index as usize]
    }

    fn move_f_to_x(&mut self, r: Reg, ty: Type) -> Reg {
        let result = self.temp_writable_reg(I64);
        self.emit(&gen_move(result, I64, r, ty));
        result.to_reg()
    }
    fn offset32_imm(&mut self, offset: i32) -> Offset32 {
        Offset32::new(offset)
    }
    fn default_memflags(&mut self) -> MemFlags {
        MemFlags::new()
    }
    fn move_x_to_f(&mut self, r: Reg, ty: Type) -> Reg {
        let result = self.temp_writable_reg(ty);
        self.emit(&gen_move(result, ty, r, I64));
        result.to_reg()
    }

    fn pack_float_rounding_mode(&mut self, f: &FRM) -> OptionFloatRoundingMode {
        Some(*f)
    }

    fn int_convert_2_float_op(&mut self, from: Type, is_signed: bool, to: Type) -> FpuOPRR {
        FpuOPRR::int_convert_2_float_op(from, is_signed, to)
    }
    fn gen_amode(&mut self, base: Reg, offset: Offset32, ty: Type) -> AMode {
        AMode::RegOffset(base, i64::from(offset), ty)
    }
    fn valid_atomic_transaction(&mut self, ty: Type) -> Option<Type> {
        if ty.is_int() && ty.bits() <= 64 {
            Some(ty)
        } else {
            None
        }
    }
    fn is_atomic_rmw_max_etc(&mut self, op: &AtomicRmwOp) -> Option<(AtomicRmwOp, bool)> {
        let op = *op;
        match op {
            crate::ir::AtomicRmwOp::Umin => Some((op, false)),
            crate::ir::AtomicRmwOp::Umax => Some((op, false)),
            crate::ir::AtomicRmwOp::Smin => Some((op, true)),
            crate::ir::AtomicRmwOp::Smax => Some((op, true)),
            _ => None,
        }
    }
    fn load_op(&mut self, ty: Type) -> LoadOP {
        LoadOP::from_type(ty)
    }
    fn store_op(&mut self, ty: Type) -> StoreOP {
        StoreOP::from_type(ty)
    }
    fn load_ext_name(&mut self, name: ExternalName, offset: i64) -> Reg {
        let tmp = self.temp_writable_reg(I64);
        self.emit(&MInst::LoadExtName {
            rd: tmp,
            name: Box::new(name),
            offset,
        });
        tmp.to_reg()
    }

    fn offset32_add(&mut self, a: Offset32, adden: i64) -> Offset32 {
        a.try_add_i64(adden).expect("offset exceed range.")
    }

    fn gen_stack_addr(&mut self, slot: StackSlot, offset: Offset32) -> Reg {
        let result = self.temp_writable_reg(I64);
        let i = self
            .lower_ctx
            .abi()
            .sized_stackslot_addr(slot, i64::from(offset) as u32, result);
        self.emit(&i);
        result.to_reg()
    }
    fn atomic_amo(&mut self) -> AMO {
        AMO::SeqCst
    }

    fn gen_move2(&mut self, r: Reg, ity: Type, oty: Type) -> Reg {
        let tmp = self.temp_writable_reg(oty);
        self.emit(&gen_move(tmp, oty, r, ity));
        tmp.to_reg()
    }

    fn intcc_is_gt_etc(&mut self, cc: &IntCC) -> Option<(IntCC, bool)> {
        let cc = *cc;
        match cc {
            IntCC::SignedLessThan => Some((cc, true)),
            IntCC::SignedGreaterThanOrEqual => Some((cc, true)),
            IntCC::SignedGreaterThan => Some((cc, true)),
            IntCC::SignedLessThanOrEqual => Some((cc, true)),
            //
            IntCC::UnsignedLessThan => Some((cc, false)),
            IntCC::UnsignedGreaterThanOrEqual => Some((cc, false)),
            IntCC::UnsignedGreaterThan => Some((cc, false)),
            IntCC::UnsignedLessThanOrEqual => Some((cc, false)),
            _ => None,
        }
    }
    fn intcc_is_eq_or_ne(&mut self, cc: &IntCC) -> Option<IntCC> {
        let cc = *cc;
        if cc == IntCC::Equal || cc == IntCC::NotEqual {
            Some(cc)
        } else {
            None
        }
    }
    fn lower_br_table(&mut self, index: Reg, targets: &VecMachLabel) -> InstOutput {
        let tmp = self.temp_writable_reg(I64);
        let default_ = BranchTarget::Label(targets[0]);
        let targets: Vec<BranchTarget> = targets
            .iter()
            .skip(1)
            .map(|bix| BranchTarget::Label(*bix))
            .collect();
        self.emit(&MInst::BrTableCheck {
            index,
            targets_len: targets.len() as i32,
            default_: default_,
        });
        self.emit(&MInst::BrTable {
            index,
            tmp1: tmp,
            targets,
        });
        InstOutput::default()
    }
    fn x_reg(&mut self, x: u8) -> Reg {
        x_reg(x as usize)
    }
    fn shift_int_to_most_significant(&mut self, v: Reg, ty: Type) -> Reg {
        assert!(ty.is_int() && ty.bits() <= 64);
        if ty == I64 {
            return v;
        }
        let tmp = self.temp_writable_reg(I64);
        self.emit(&MInst::AluRRImm12 {
            alu_op: AluOPRRI::Slli,
            rd: tmp,
            rs: v,
            imm12: Imm12::from_bits((64 - ty.bits()) as i16),
        });

        tmp.to_reg()
    }
}

impl IsleContext<'_, '_, MInst, Flags, IsaFlags, 6> {
    #[inline]
    fn emit_list(&mut self, list: &SmallInstVec<MInst>) {
        for i in list {
            self.lower_ctx.emit(i.clone());
        }
    }
}

/// The main entry point for branch lowering with ISLE.
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

/// construct destination according to ty.
fn construct_dest<F: std::ops::FnMut(Type) -> WritableReg>(
    mut alloc: F,
    ty: Type,
) -> WritableValueRegs {
    if ty.is_int() {
        if ty.bits() == 128 {
            WritableValueRegs::two(alloc(I64), alloc(I64))
        } else {
            WritableValueRegs::one(alloc(I64))
        }
    } else if ty.is_float() {
        WritableValueRegs::one(alloc(F64))
    } else {
        unimplemented!("vector type not implemented.");
    }
}
