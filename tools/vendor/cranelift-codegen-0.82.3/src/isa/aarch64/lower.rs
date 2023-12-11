//! Lowering rules for AArch64.
//!
//! TODO: opportunities for better code generation:
//!
//! - Smarter use of addressing modes. Recognize a+SCALE*b patterns. Recognize
//!   pre/post-index opportunities.
//!
//! - Floating-point immediates (FIMM instruction).

use crate::ir::condcodes::{FloatCC, IntCC};
use crate::ir::types::*;
use crate::ir::Inst as IRInst;
use crate::ir::{Opcode, Type, Value};
use crate::machinst::lower::*;
use crate::machinst::*;
use crate::{CodegenError, CodegenResult};

use crate::isa::aarch64::inst::*;
use crate::isa::aarch64::AArch64Backend;

use super::lower_inst;

use crate::data_value::DataValue;
use regalloc::{Reg, Writable};
use smallvec::SmallVec;
use std::cmp;

pub mod isle;

//============================================================================
// Result enum types.
//
// Lowering of a given value results in one of these enums, depending on the
// modes in which we can accept the value.

/// A lowering result: register, register-shift.  An SSA value can always be
/// lowered into one of these options; the register form is the fallback.
#[derive(Clone, Debug)]
enum ResultRS {
    Reg(Reg),
    RegShift(Reg, ShiftOpAndAmt),
}

/// A lowering result: register, register-shift, register-extend.  An SSA value can always be
/// lowered into one of these options; the register form is the fallback.
#[derive(Clone, Debug)]
enum ResultRSE {
    Reg(Reg),
    RegShift(Reg, ShiftOpAndAmt),
    RegExtend(Reg, ExtendOp),
}

impl ResultRSE {
    fn from_rs(rs: ResultRS) -> ResultRSE {
        match rs {
            ResultRS::Reg(r) => ResultRSE::Reg(r),
            ResultRS::RegShift(r, s) => ResultRSE::RegShift(r, s),
        }
    }
}

/// A lowering result: register, register-shift, register-extend, or 12-bit immediate form.
/// An SSA value can always be lowered into one of these options; the register form is the
/// fallback.
#[derive(Clone, Debug)]
pub(crate) enum ResultRSEImm12 {
    Reg(Reg),
    RegShift(Reg, ShiftOpAndAmt),
    RegExtend(Reg, ExtendOp),
    Imm12(Imm12),
}

impl ResultRSEImm12 {
    fn from_rse(rse: ResultRSE) -> ResultRSEImm12 {
        match rse {
            ResultRSE::Reg(r) => ResultRSEImm12::Reg(r),
            ResultRSE::RegShift(r, s) => ResultRSEImm12::RegShift(r, s),
            ResultRSE::RegExtend(r, e) => ResultRSEImm12::RegExtend(r, e),
        }
    }
}

//============================================================================
// Lowering: convert instruction inputs to forms that we can use.

/// Lower an instruction input to a 64-bit constant, if possible.
pub(crate) fn input_to_const<C: LowerCtx<I = Inst>>(ctx: &mut C, input: InsnInput) -> Option<u64> {
    let input = ctx.get_input_as_source_or_const(input.insn, input.input);
    input.constant
}

/// Lower an instruction input to a constant register-shift amount, if possible.
pub(crate) fn input_to_shiftimm<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
) -> Option<ShiftOpShiftImm> {
    input_to_const(ctx, input).and_then(ShiftOpShiftImm::maybe_from_shift)
}

pub(crate) fn const_param_to_u128<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    inst: IRInst,
) -> Option<u128> {
    match ctx.get_immediate(inst) {
        Some(DataValue::V128(bytes)) => Some(u128::from_le_bytes(bytes)),
        _ => None,
    }
}

/// How to handle narrow values loaded into registers; see note on `narrow_mode`
/// parameter to `put_input_in_*` below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NarrowValueMode {
    None,
    /// Zero-extend to 32 bits if original is < 32 bits.
    ZeroExtend32,
    /// Sign-extend to 32 bits if original is < 32 bits.
    SignExtend32,
    /// Zero-extend to 64 bits if original is < 64 bits.
    ZeroExtend64,
    /// Sign-extend to 64 bits if original is < 64 bits.
    SignExtend64,
}

impl NarrowValueMode {
    fn is_32bit(&self) -> bool {
        match self {
            NarrowValueMode::None => false,
            NarrowValueMode::ZeroExtend32 | NarrowValueMode::SignExtend32 => true,
            NarrowValueMode::ZeroExtend64 | NarrowValueMode::SignExtend64 => false,
        }
    }

    fn is_signed(&self) -> bool {
        match self {
            NarrowValueMode::SignExtend32 | NarrowValueMode::SignExtend64 => true,
            NarrowValueMode::ZeroExtend32 | NarrowValueMode::ZeroExtend64 => false,
            NarrowValueMode::None => false,
        }
    }
}

/// Emits instruction(s) to generate the given constant value into newly-allocated
/// temporary registers, returning these registers.
fn generate_constant<C: LowerCtx<I = Inst>>(ctx: &mut C, ty: Type, c: u128) -> ValueRegs<Reg> {
    let from_bits = ty_bits(ty);
    let masked = if from_bits < 128 {
        c & ((1u128 << from_bits) - 1)
    } else {
        c
    };

    let cst_copy = ctx.alloc_tmp(ty);
    for inst in Inst::gen_constant(cst_copy, masked, ty, |ty| {
        ctx.alloc_tmp(ty).only_reg().unwrap()
    })
    .into_iter()
    {
        ctx.emit(inst);
    }
    non_writable_value_regs(cst_copy)
}

/// Extends a register according to `narrow_mode`.
/// If extended, the value is always extended to 64 bits, for simplicity.
fn extend_reg<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    ty: Type,
    in_reg: Reg,
    is_const: bool,
    narrow_mode: NarrowValueMode,
) -> Reg {
    let from_bits = ty_bits(ty) as u8;
    match (narrow_mode, from_bits) {
        (NarrowValueMode::None, _) => in_reg,
        (NarrowValueMode::ZeroExtend32, n) if n < 32 => {
            let tmp = ctx.alloc_tmp(I32).only_reg().unwrap();
            ctx.emit(Inst::Extend {
                rd: tmp,
                rn: in_reg,
                signed: false,
                from_bits,
                to_bits: 32,
            });
            tmp.to_reg()
        }
        (NarrowValueMode::SignExtend32, n) if n < 32 => {
            let tmp = ctx.alloc_tmp(I32).only_reg().unwrap();
            ctx.emit(Inst::Extend {
                rd: tmp,
                rn: in_reg,
                signed: true,
                from_bits,
                to_bits: 32,
            });
            tmp.to_reg()
        }
        (NarrowValueMode::ZeroExtend32, 32) | (NarrowValueMode::SignExtend32, 32) => in_reg,

        (NarrowValueMode::ZeroExtend64, n) if n < 64 => {
            if is_const {
                // Constants are zero-extended to full 64-bit width on load already.
                in_reg
            } else {
                let tmp = ctx.alloc_tmp(I32).only_reg().unwrap();
                ctx.emit(Inst::Extend {
                    rd: tmp,
                    rn: in_reg,
                    signed: false,
                    from_bits,
                    to_bits: 64,
                });
                tmp.to_reg()
            }
        }
        (NarrowValueMode::SignExtend64, n) if n < 64 => {
            let tmp = ctx.alloc_tmp(I32).only_reg().unwrap();
            ctx.emit(Inst::Extend {
                rd: tmp,
                rn: in_reg,
                signed: true,
                from_bits,
                to_bits: 64,
            });
            tmp.to_reg()
        }
        (_, 64) => in_reg,
        (_, 128) => in_reg,

        _ => panic!(
            "Unsupported input width: input ty {} bits {} mode {:?}",
            ty, from_bits, narrow_mode
        ),
    }
}

/// Lowers an instruction input to multiple regs
fn lower_value_to_regs<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    value: Value,
) -> (ValueRegs<Reg>, Type, bool) {
    log::trace!("lower_value_to_regs: value {:?}", value);
    let ty = ctx.value_ty(value);
    let inputs = ctx.get_value_as_source_or_const(value);
    let is_const = inputs.constant.is_some();

    let in_regs = if let Some(c) = inputs.constant {
        // Generate constants fresh at each use to minimize long-range register pressure.
        generate_constant(ctx, ty, c as u128)
    } else {
        ctx.put_value_in_regs(value)
    };

    (in_regs, ty, is_const)
}

/// Lower an instruction input to a register
///
/// The given register will be extended appropriately, according to
/// `narrow_mode` and the input's type. If extended, the value is
/// always extended to 64 bits, for simplicity.
pub(crate) fn put_input_in_reg<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
    narrow_mode: NarrowValueMode,
) -> Reg {
    let value = ctx.input_as_value(input.insn, input.input);
    put_value_in_reg(ctx, value, narrow_mode)
}

/// Like above, only for values
fn put_value_in_reg<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    value: Value,
    narrow_mode: NarrowValueMode,
) -> Reg {
    let (in_regs, ty, is_const) = lower_value_to_regs(ctx, value);
    let reg = in_regs
        .only_reg()
        .expect("Multi-register value not expected");

    extend_reg(ctx, ty, reg, is_const, narrow_mode)
}

/// Lower an instruction input to multiple regs
pub(crate) fn put_input_in_regs<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
) -> ValueRegs<Reg> {
    let value = ctx.input_as_value(input.insn, input.input);
    let (in_regs, _, _) = lower_value_to_regs(ctx, value);
    in_regs
}

/// Lower an instruction input to a reg or reg/shift, or reg/extend operand.
///
/// The `narrow_mode` flag indicates whether the consumer of this value needs
/// the high bits clear. For many operations, such as an add/sub/mul or any
/// bitwise logical operation, the low-bit results depend only on the low-bit
/// inputs, so e.g. we can do an 8 bit add on 32 bit registers where the 8-bit
/// value is stored in the low 8 bits of the register and the high 24 bits are
/// undefined. If the op truly needs the high N bits clear (such as for a
/// divide or a right-shift or a compare-to-zero), `narrow_mode` should be
/// set to `ZeroExtend` or `SignExtend` as appropriate, and the resulting
/// register will be provided the extended value.
fn put_input_in_rs<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
    narrow_mode: NarrowValueMode,
) -> ResultRS {
    let inputs = ctx.get_input_as_source_or_const(input.insn, input.input);
    if let Some((insn, 0)) = inputs.inst {
        let op = ctx.data(insn).opcode();

        if op == Opcode::Ishl {
            let shiftee = InsnInput { insn, input: 0 };
            let shift_amt = InsnInput { insn, input: 1 };

            // Can we get the shift amount as an immediate?
            if let Some(shiftimm) = input_to_shiftimm(ctx, shift_amt) {
                let shiftee_bits = ty_bits(ctx.input_ty(insn, 0));
                if shiftee_bits <= std::u8::MAX as usize {
                    let shiftimm = shiftimm.mask(shiftee_bits as u8);
                    let reg = put_input_in_reg(ctx, shiftee, narrow_mode);
                    return ResultRS::RegShift(reg, ShiftOpAndAmt::new(ShiftOp::LSL, shiftimm));
                }
            }
        }
    }

    ResultRS::Reg(put_input_in_reg(ctx, input, narrow_mode))
}

/// Lower an instruction input to a reg or reg/shift, or reg/extend operand.
/// This does not actually codegen the source instruction; it just uses the
/// vreg into which the source instruction will generate its value.
///
/// See note on `put_input_in_rs` for a description of `narrow_mode`.
fn put_input_in_rse<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
    narrow_mode: NarrowValueMode,
) -> ResultRSE {
    let value = ctx.input_as_value(input.insn, input.input);
    if let Some((val, extendop)) = get_as_extended_value(ctx, value, narrow_mode) {
        let reg = put_value_in_reg(ctx, val, NarrowValueMode::None);
        return ResultRSE::RegExtend(reg, extendop);
    }

    ResultRSE::from_rs(put_input_in_rs(ctx, input, narrow_mode))
}

fn get_as_extended_value<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    val: Value,
    narrow_mode: NarrowValueMode,
) -> Option<(Value, ExtendOp)> {
    let inputs = ctx.get_value_as_source_or_const(val);
    let (insn, n) = inputs.inst?;
    if n != 0 {
        return None;
    }
    let op = ctx.data(insn).opcode();
    let out_ty = ctx.output_ty(insn, 0);
    let out_bits = ty_bits(out_ty);

    // Is this a zero-extend or sign-extend and can we handle that with a register-mode operator?
    if op == Opcode::Uextend || op == Opcode::Sextend {
        let sign_extend = op == Opcode::Sextend;
        let inner_ty = ctx.input_ty(insn, 0);
        let inner_bits = ty_bits(inner_ty);
        assert!(inner_bits < out_bits);
        if match (sign_extend, narrow_mode) {
            // A single zero-extend or sign-extend is equal to itself.
            (_, NarrowValueMode::None) => true,
            // Two zero-extends or sign-extends in a row is equal to a single zero-extend or sign-extend.
            (false, NarrowValueMode::ZeroExtend32) | (false, NarrowValueMode::ZeroExtend64) => true,
            (true, NarrowValueMode::SignExtend32) | (true, NarrowValueMode::SignExtend64) => true,
            // A zero-extend and a sign-extend in a row is not equal to a single zero-extend or sign-extend
            (false, NarrowValueMode::SignExtend32) | (false, NarrowValueMode::SignExtend64) => {
                false
            }
            (true, NarrowValueMode::ZeroExtend32) | (true, NarrowValueMode::ZeroExtend64) => false,
        } {
            let extendop = match (sign_extend, inner_bits) {
                (true, 8) => ExtendOp::SXTB,
                (false, 8) => ExtendOp::UXTB,
                (true, 16) => ExtendOp::SXTH,
                (false, 16) => ExtendOp::UXTH,
                (true, 32) => ExtendOp::SXTW,
                (false, 32) => ExtendOp::UXTW,
                _ => unreachable!(),
            };
            return Some((ctx.input_as_value(insn, 0), extendop));
        }
    }

    // If `out_ty` is smaller than 32 bits and we need to zero- or sign-extend,
    // then get the result into a register and return an Extend-mode operand on
    // that register.
    if narrow_mode != NarrowValueMode::None
        && ((narrow_mode.is_32bit() && out_bits < 32) || (!narrow_mode.is_32bit() && out_bits < 64))
    {
        let extendop = match (narrow_mode, out_bits) {
            (NarrowValueMode::SignExtend32, 1) | (NarrowValueMode::SignExtend64, 1) => {
                ExtendOp::SXTB
            }
            (NarrowValueMode::ZeroExtend32, 1) | (NarrowValueMode::ZeroExtend64, 1) => {
                ExtendOp::UXTB
            }
            (NarrowValueMode::SignExtend32, 8) | (NarrowValueMode::SignExtend64, 8) => {
                ExtendOp::SXTB
            }
            (NarrowValueMode::ZeroExtend32, 8) | (NarrowValueMode::ZeroExtend64, 8) => {
                ExtendOp::UXTB
            }
            (NarrowValueMode::SignExtend32, 16) | (NarrowValueMode::SignExtend64, 16) => {
                ExtendOp::SXTH
            }
            (NarrowValueMode::ZeroExtend32, 16) | (NarrowValueMode::ZeroExtend64, 16) => {
                ExtendOp::UXTH
            }
            (NarrowValueMode::SignExtend64, 32) => ExtendOp::SXTW,
            (NarrowValueMode::ZeroExtend64, 32) => ExtendOp::UXTW,
            _ => unreachable!(),
        };
        return Some((val, extendop));
    }
    None
}

pub(crate) fn put_input_in_rse_imm12<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    input: InsnInput,
    narrow_mode: NarrowValueMode,
) -> ResultRSEImm12 {
    if let Some(imm_value) = input_to_const(ctx, input) {
        if let Some(i) = Imm12::maybe_from_u64(imm_value) {
            let out_ty_bits = ty_bits(ctx.input_ty(input.insn, input.input));
            let is_negative = (i.bits as u64) & (1 << (cmp::max(out_ty_bits, 1) - 1)) != 0;

            // This condition can happen if we matched a value that overflows the output type of
            // its `iconst` when viewed as a signed value (i.e. iconst.i8 200).
            // When that happens we need to lower as a negative value, which we cannot do here.
            if !(narrow_mode.is_signed() && is_negative) {
                return ResultRSEImm12::Imm12(i);
            }
        }
    }

    ResultRSEImm12::from_rse(put_input_in_rse(ctx, input, narrow_mode))
}

//============================================================================
// ALU instruction constructors.

pub(crate) fn alu_inst_imm12(
    op: ALUOp,
    ty: Type,
    rd: Writable<Reg>,
    rn: Reg,
    rm: ResultRSEImm12,
) -> Inst {
    let size = OperandSize::from_ty(ty);
    match rm {
        ResultRSEImm12::Imm12(imm12) => Inst::AluRRImm12 {
            alu_op: op,
            size,
            rd,
            rn,
            imm12,
        },
        ResultRSEImm12::Reg(rm) => Inst::AluRRR {
            alu_op: op,
            size,
            rd,
            rn,
            rm,
        },
        ResultRSEImm12::RegShift(rm, shiftop) => Inst::AluRRRShift {
            alu_op: op,
            size,
            rd,
            rn,
            rm,
            shiftop,
        },
        ResultRSEImm12::RegExtend(rm, extendop) => Inst::AluRRRExtend {
            alu_op: op,
            size,
            rd,
            rn,
            rm,
            extendop,
        },
    }
}

//============================================================================
// Lowering: addressing mode support. Takes instruction directly, rather
// than an `InsnInput`, to do more introspection.

/// 32-bit addends that make up an address: an input, and an extension mode on that
/// input.
type AddressAddend32List = SmallVec<[(Reg, ExtendOp); 4]>;
/// 64-bit addends that make up an address: just an input.
type AddressAddend64List = SmallVec<[Reg; 4]>;

/// Collect all addends that feed into an address computation, with extend-modes
/// on each.  Note that a load/store may have multiple address components (and
/// the CLIF semantics are that these components are added to form the final
/// address), but sometimes the CLIF that we receive still has arguments that
/// refer to `iadd` instructions. We also want to handle uextend/sextend below
/// the add(s).
///
/// We match any 64-bit add (and descend into its inputs), and we match any
/// 32-to-64-bit sign or zero extension. The returned addend-list will use
/// NarrowValueMode values to indicate how to extend each input:
///
/// - NarrowValueMode::None: the associated input is 64 bits wide; no extend.
/// - NarrowValueMode::SignExtend64: the associated input is 32 bits wide;
///                                  do a sign-extension.
/// - NarrowValueMode::ZeroExtend64: the associated input is 32 bits wide;
///                                  do a zero-extension.
///
/// We do not descend further into the inputs of extensions (unless it is a constant),
/// because supporting (e.g.) a 32-bit add that is later extended would require
/// additional masking of high-order bits, which is too complex. So, in essence, we
/// descend any number of adds from the roots, collecting all 64-bit address addends;
/// then possibly support extensions at these leaves.
fn collect_address_addends<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    roots: &[InsnInput],
) -> (AddressAddend64List, AddressAddend32List, i64) {
    let mut result32: AddressAddend32List = SmallVec::new();
    let mut result64: AddressAddend64List = SmallVec::new();
    let mut offset: i64 = 0;

    let mut workqueue: SmallVec<[InsnInput; 4]> = roots.iter().cloned().collect();

    while let Some(input) = workqueue.pop() {
        debug_assert!(ty_bits(ctx.input_ty(input.insn, input.input)) == 64);
        if let Some((op, insn)) = maybe_input_insn_multi(
            ctx,
            input,
            &[
                Opcode::Uextend,
                Opcode::Sextend,
                Opcode::Iadd,
                Opcode::Iconst,
            ],
        ) {
            match op {
                Opcode::Uextend | Opcode::Sextend if ty_bits(ctx.input_ty(insn, 0)) == 32 => {
                    let extendop = if op == Opcode::Uextend {
                        ExtendOp::UXTW
                    } else {
                        ExtendOp::SXTW
                    };
                    let extendee_input = InsnInput { insn, input: 0 };
                    // If the input is a zero-extension of a constant, add the value to the known
                    // offset.
                    // Only do this for zero-extension, as generating a sign-extended
                    // constant may be more instructions than using the 'SXTW' addressing mode.
                    if let (Some(insn), ExtendOp::UXTW) = (
                        maybe_input_insn(ctx, extendee_input, Opcode::Iconst),
                        extendop,
                    ) {
                        let value = (ctx.get_constant(insn).unwrap() & 0xFFFF_FFFF_u64) as i64;
                        offset += value;
                    } else {
                        let reg = put_input_in_reg(ctx, extendee_input, NarrowValueMode::None);
                        result32.push((reg, extendop));
                    }
                }
                Opcode::Uextend | Opcode::Sextend => {
                    let reg = put_input_in_reg(ctx, input, NarrowValueMode::None);
                    result64.push(reg);
                }
                Opcode::Iadd => {
                    for input in 0..ctx.num_inputs(insn) {
                        let addend = InsnInput { insn, input };
                        workqueue.push(addend);
                    }
                }
                Opcode::Iconst => {
                    let value: i64 = ctx.get_constant(insn).unwrap() as i64;
                    offset += value;
                }
                _ => panic!("Unexpected opcode from maybe_input_insn_multi"),
            }
        } else {
            let reg = put_input_in_reg(ctx, input, NarrowValueMode::ZeroExtend64);
            result64.push(reg);
        }
    }

    (result64, result32, offset)
}

/// Lower the address of a pair load or store.
pub(crate) fn lower_pair_address<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    roots: &[InsnInput],
    offset: i32,
) -> PairAMode {
    // Collect addends through an arbitrary tree of 32-to-64-bit sign/zero
    // extends and addition ops. We update these as we consume address
    // components, so they represent the remaining addends not yet handled.
    let (mut addends64, mut addends32, args_offset) = collect_address_addends(ctx, roots);
    let offset = args_offset + (offset as i64);

    log::trace!(
        "lower_pair_address: addends64 {:?}, addends32 {:?}, offset {}",
        addends64,
        addends32,
        offset
    );

    // Pairs basically only have reg + imm formats so we only have to worry about those

    let base_reg = if let Some(reg64) = addends64.pop() {
        reg64
    } else if let Some((reg32, extendop)) = addends32.pop() {
        let tmp = ctx.alloc_tmp(I64).only_reg().unwrap();
        let signed = match extendop {
            ExtendOp::SXTW => true,
            ExtendOp::UXTW => false,
            _ => unreachable!(),
        };
        ctx.emit(Inst::Extend {
            rd: tmp,
            rn: reg32,
            signed,
            from_bits: 32,
            to_bits: 64,
        });
        tmp.to_reg()
    } else {
        zero_reg()
    };

    let addr = ctx.alloc_tmp(I64).only_reg().unwrap();
    ctx.emit(Inst::gen_move(addr, base_reg, I64));

    // We have the base register, if we have any others, we need to add them
    lower_add_addends(ctx, addr, addends64, addends32);

    // Figure out what offset we should emit
    let imm7 = SImm7Scaled::maybe_from_i64(offset, I64).unwrap_or_else(|| {
        lower_add_immediate(ctx, addr, addr.to_reg(), offset);
        SImm7Scaled::maybe_from_i64(0, I64).unwrap()
    });

    PairAMode::SignedOffset(addr.to_reg(), imm7)
}

/// Lower the address of a load or store.
pub(crate) fn lower_address<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    elem_ty: Type,
    roots: &[InsnInput],
    offset: i32,
) -> AMode {
    // TODO: support base_reg + scale * index_reg. For this, we would need to pattern-match shl or
    // mul instructions (Load/StoreComplex don't include scale factors).

    // Collect addends through an arbitrary tree of 32-to-64-bit sign/zero
    // extends and addition ops. We update these as we consume address
    // components, so they represent the remaining addends not yet handled.
    let (mut addends64, mut addends32, args_offset) = collect_address_addends(ctx, roots);
    let mut offset = args_offset + (offset as i64);

    log::trace!(
        "lower_address: addends64 {:?}, addends32 {:?}, offset {}",
        addends64,
        addends32,
        offset
    );

    // First, decide what the `AMode` will be. Take one extendee and one 64-bit
    // reg, or two 64-bit regs, or a 64-bit reg and a 32-bit reg with extension,
    // or some other combination as appropriate.
    let memarg = if addends64.len() > 0 {
        if addends32.len() > 0 {
            let (reg32, extendop) = addends32.pop().unwrap();
            let reg64 = addends64.pop().unwrap();
            AMode::RegExtended(reg64, reg32, extendop)
        } else if offset > 0 && offset < 0x1000 {
            let reg64 = addends64.pop().unwrap();
            let off = offset;
            offset = 0;
            AMode::RegOffset(reg64, off, elem_ty)
        } else if addends64.len() >= 2 {
            let reg1 = addends64.pop().unwrap();
            let reg2 = addends64.pop().unwrap();
            AMode::RegReg(reg1, reg2)
        } else {
            let reg1 = addends64.pop().unwrap();
            AMode::reg(reg1)
        }
    } else
    /* addends64.len() == 0 */
    {
        if addends32.len() > 0 {
            let tmp = ctx.alloc_tmp(I64).only_reg().unwrap();
            let (reg1, extendop) = addends32.pop().unwrap();
            let signed = match extendop {
                ExtendOp::SXTW => true,
                ExtendOp::UXTW => false,
                _ => unreachable!(),
            };
            ctx.emit(Inst::Extend {
                rd: tmp,
                rn: reg1,
                signed,
                from_bits: 32,
                to_bits: 64,
            });
            if let Some((reg2, extendop)) = addends32.pop() {
                AMode::RegExtended(tmp.to_reg(), reg2, extendop)
            } else {
                AMode::reg(tmp.to_reg())
            }
        } else
        /* addends32.len() == 0 */
        {
            let off_reg = ctx.alloc_tmp(I64).only_reg().unwrap();
            lower_constant_u64(ctx, off_reg, offset as u64);
            offset = 0;
            AMode::reg(off_reg.to_reg())
        }
    };

    // At this point, if we have any remaining components, we need to allocate a
    // temp, replace one of the registers in the AMode with the temp, and emit
    // instructions to add together the remaining components. Return immediately
    // if this is *not* the case.
    if offset == 0 && addends32.len() == 0 && addends64.len() == 0 {
        return memarg;
    }

    // Allocate the temp and shoehorn it into the AMode.
    let addr = ctx.alloc_tmp(I64).only_reg().unwrap();
    let (reg, memarg) = match memarg {
        AMode::RegExtended(r1, r2, extendop) => {
            (r1, AMode::RegExtended(addr.to_reg(), r2, extendop))
        }
        AMode::RegOffset(r, off, ty) => (r, AMode::RegOffset(addr.to_reg(), off, ty)),
        AMode::RegReg(r1, r2) => (r2, AMode::RegReg(addr.to_reg(), r1)),
        AMode::UnsignedOffset(r, imm) => (r, AMode::UnsignedOffset(addr.to_reg(), imm)),
        _ => unreachable!(),
    };

    // If there is any offset, load that first into `addr`, and add the `reg`
    // that we kicked out of the `AMode`; otherwise, start with that reg.
    if offset != 0 {
        lower_add_immediate(ctx, addr, reg, offset)
    } else {
        ctx.emit(Inst::gen_move(addr, reg, I64));
    }

    // Now handle reg64 and reg32-extended components.
    lower_add_addends(ctx, addr, addends64, addends32);

    memarg
}

fn lower_add_addends<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    addends64: AddressAddend64List,
    addends32: AddressAddend32List,
) {
    for reg in addends64 {
        // If the register is the stack reg, we must move it to another reg
        // before adding it.
        let reg = if reg == stack_reg() {
            let tmp = ctx.alloc_tmp(I64).only_reg().unwrap();
            ctx.emit(Inst::gen_move(tmp, stack_reg(), I64));
            tmp.to_reg()
        } else {
            reg
        };
        ctx.emit(Inst::AluRRR {
            alu_op: ALUOp::Add,
            size: OperandSize::Size64,
            rd,
            rn: rd.to_reg(),
            rm: reg,
        });
    }
    for (reg, extendop) in addends32 {
        assert!(reg != stack_reg());
        ctx.emit(Inst::AluRRRExtend {
            alu_op: ALUOp::Add,
            size: OperandSize::Size64,
            rd,
            rn: rd.to_reg(),
            rm: reg,
            extendop,
        });
    }
}

/// Adds into `rd` a signed imm pattern matching the best instruction for it.
// TODO: This function is duplicated in ctx.gen_add_imm
fn lower_add_immediate<C: LowerCtx<I = Inst>>(ctx: &mut C, dst: Writable<Reg>, src: Reg, imm: i64) {
    // If we can fit offset or -offset in an imm12, use an add-imm
    // Otherwise, lower the constant first then add.
    if let Some(imm12) = Imm12::maybe_from_u64(imm as u64) {
        ctx.emit(Inst::AluRRImm12 {
            alu_op: ALUOp::Add,
            size: OperandSize::Size64,
            rd: dst,
            rn: src,
            imm12,
        });
    } else if let Some(imm12) = Imm12::maybe_from_u64(imm.wrapping_neg() as u64) {
        ctx.emit(Inst::AluRRImm12 {
            alu_op: ALUOp::Sub,
            size: OperandSize::Size64,
            rd: dst,
            rn: src,
            imm12,
        });
    } else {
        lower_constant_u64(ctx, dst, imm as u64);
        ctx.emit(Inst::AluRRR {
            alu_op: ALUOp::Add,
            size: OperandSize::Size64,
            rd: dst,
            rn: dst.to_reg(),
            rm: src,
        });
    }
}

pub(crate) fn lower_constant_u64<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    value: u64,
) {
    for inst in Inst::load_constant(rd, value) {
        ctx.emit(inst);
    }
}

pub(crate) fn lower_constant_f32<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    value: f32,
) {
    let alloc_tmp = |ty| ctx.alloc_tmp(ty).only_reg().unwrap();

    for inst in Inst::load_fp_constant32(rd, value.to_bits(), alloc_tmp) {
        ctx.emit(inst);
    }
}

pub(crate) fn lower_constant_f64<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    value: f64,
) {
    let alloc_tmp = |ty| ctx.alloc_tmp(ty).only_reg().unwrap();

    for inst in Inst::load_fp_constant64(rd, value.to_bits(), alloc_tmp) {
        ctx.emit(inst);
    }
}

pub(crate) fn lower_constant_f128<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    value: u128,
) {
    if value == 0 {
        // Fast-track a common case.  The general case, viz, calling `Inst::load_fp_constant128`,
        // is potentially expensive.
        ctx.emit(Inst::VecDupImm {
            rd,
            imm: ASIMDMovModImm::zero(ScalarSize::Size8),
            invert: false,
            size: VectorSize::Size8x16,
        });
    } else {
        let alloc_tmp = |ty| ctx.alloc_tmp(ty).only_reg().unwrap();
        for inst in Inst::load_fp_constant128(rd, value, alloc_tmp) {
            ctx.emit(inst);
        }
    }
}

pub(crate) fn lower_splat_const<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    value: u64,
    size: VectorSize,
) {
    let (value, narrow_size) = match size.lane_size() {
        ScalarSize::Size8 => (value as u8 as u64, ScalarSize::Size128),
        ScalarSize::Size16 => (value as u16 as u64, ScalarSize::Size8),
        ScalarSize::Size32 => (value as u32 as u64, ScalarSize::Size16),
        ScalarSize::Size64 => (value, ScalarSize::Size32),
        _ => unreachable!(),
    };
    let (value, size) = match Inst::get_replicated_vector_pattern(value as u128, narrow_size) {
        Some((value, lane_size)) => (
            value,
            VectorSize::from_lane_size(lane_size, size.is_128bits()),
        ),
        None => (value, size),
    };
    let alloc_tmp = |ty| ctx.alloc_tmp(ty).only_reg().unwrap();

    for inst in Inst::load_replicated_vector_pattern(rd, value, size, alloc_tmp) {
        ctx.emit(inst);
    }
}

pub(crate) fn lower_condcode(cc: IntCC) -> Cond {
    match cc {
        IntCC::Equal => Cond::Eq,
        IntCC::NotEqual => Cond::Ne,
        IntCC::SignedGreaterThanOrEqual => Cond::Ge,
        IntCC::SignedGreaterThan => Cond::Gt,
        IntCC::SignedLessThanOrEqual => Cond::Le,
        IntCC::SignedLessThan => Cond::Lt,
        IntCC::UnsignedGreaterThanOrEqual => Cond::Hs,
        IntCC::UnsignedGreaterThan => Cond::Hi,
        IntCC::UnsignedLessThanOrEqual => Cond::Ls,
        IntCC::UnsignedLessThan => Cond::Lo,
        IntCC::Overflow => Cond::Vs,
        IntCC::NotOverflow => Cond::Vc,
    }
}

pub(crate) fn lower_fp_condcode(cc: FloatCC) -> Cond {
    // Refer to `codegen/shared/src/condcodes.rs` and to the `FCMP` AArch64 docs.
    // The FCMP instruction sets:
    //               NZCV
    // - PCSR.NZCV = 0011 on UN (unordered),
    //               0110 on EQ,
    //               1000 on LT,
    //               0010 on GT.
    match cc {
        // EQ | LT | GT. Vc => V clear.
        FloatCC::Ordered => Cond::Vc,
        // UN. Vs => V set.
        FloatCC::Unordered => Cond::Vs,
        // EQ. Eq => Z set.
        FloatCC::Equal => Cond::Eq,
        // UN | LT | GT. Ne => Z clear.
        FloatCC::NotEqual => Cond::Ne,
        // LT | GT.
        FloatCC::OrderedNotEqual => unimplemented!(),
        //  UN | EQ
        FloatCC::UnorderedOrEqual => unimplemented!(),
        // LT. Mi => N set.
        FloatCC::LessThan => Cond::Mi,
        // LT | EQ. Ls => C clear or Z set.
        FloatCC::LessThanOrEqual => Cond::Ls,
        // GT. Gt => Z clear, N = V.
        FloatCC::GreaterThan => Cond::Gt,
        // GT | EQ. Ge => N = V.
        FloatCC::GreaterThanOrEqual => Cond::Ge,
        // UN | LT
        FloatCC::UnorderedOrLessThan => unimplemented!(),
        // UN | LT | EQ
        FloatCC::UnorderedOrLessThanOrEqual => unimplemented!(),
        // UN | GT
        FloatCC::UnorderedOrGreaterThan => unimplemented!(),
        // UN | GT | EQ
        FloatCC::UnorderedOrGreaterThanOrEqual => unimplemented!(),
    }
}

pub(crate) fn lower_vector_compare<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rd: Writable<Reg>,
    mut rn: Reg,
    mut rm: Reg,
    ty: Type,
    cond: Cond,
) -> CodegenResult<()> {
    let is_float = ty.lane_type().is_float();
    let size = VectorSize::from_ty(ty);

    if is_float && (cond == Cond::Vc || cond == Cond::Vs) {
        let tmp = ctx.alloc_tmp(ty).only_reg().unwrap();

        ctx.emit(Inst::VecRRR {
            alu_op: VecALUOp::Fcmeq,
            rd,
            rn,
            rm: rn,
            size,
        });
        ctx.emit(Inst::VecRRR {
            alu_op: VecALUOp::Fcmeq,
            rd: tmp,
            rn: rm,
            rm,
            size,
        });
        ctx.emit(Inst::VecRRR {
            alu_op: VecALUOp::And,
            rd,
            rn: rd.to_reg(),
            rm: tmp.to_reg(),
            size,
        });

        if cond == Cond::Vs {
            ctx.emit(Inst::VecMisc {
                op: VecMisc2::Not,
                rd,
                rn: rd.to_reg(),
                size,
            });
        }
    } else {
        // 'Less than' operations are implemented by swapping
        // the order of operands and using the 'greater than'
        // instructions.
        // 'Not equal' is implemented with 'equal' and inverting
        // the result.
        let (alu_op, swap) = match (is_float, cond) {
            (false, Cond::Eq) => (VecALUOp::Cmeq, false),
            (false, Cond::Ne) => (VecALUOp::Cmeq, false),
            (false, Cond::Ge) => (VecALUOp::Cmge, false),
            (false, Cond::Gt) => (VecALUOp::Cmgt, false),
            (false, Cond::Le) => (VecALUOp::Cmge, true),
            (false, Cond::Lt) => (VecALUOp::Cmgt, true),
            (false, Cond::Hs) => (VecALUOp::Cmhs, false),
            (false, Cond::Hi) => (VecALUOp::Cmhi, false),
            (false, Cond::Ls) => (VecALUOp::Cmhs, true),
            (false, Cond::Lo) => (VecALUOp::Cmhi, true),
            (true, Cond::Eq) => (VecALUOp::Fcmeq, false),
            (true, Cond::Ne) => (VecALUOp::Fcmeq, false),
            (true, Cond::Mi) => (VecALUOp::Fcmgt, true),
            (true, Cond::Ls) => (VecALUOp::Fcmge, true),
            (true, Cond::Ge) => (VecALUOp::Fcmge, false),
            (true, Cond::Gt) => (VecALUOp::Fcmgt, false),
            _ => {
                return Err(CodegenError::Unsupported(format!(
                    "Unsupported {} SIMD vector comparison: {:?}",
                    if is_float {
                        "floating-point"
                    } else {
                        "integer"
                    },
                    cond
                )))
            }
        };

        if swap {
            std::mem::swap(&mut rn, &mut rm);
        }

        ctx.emit(Inst::VecRRR {
            alu_op,
            rd,
            rn,
            rm,
            size,
        });

        if cond == Cond::Ne {
            ctx.emit(Inst::VecMisc {
                op: VecMisc2::Not,
                rd,
                rn: rd.to_reg(),
                size,
            });
        }
    }

    Ok(())
}

/// Determines whether this condcode interprets inputs as signed or unsigned.  See the
/// documentation for the `icmp` instruction in cranelift-codegen/meta/src/shared/instructions.rs
/// for further insights into this.
pub(crate) fn condcode_is_signed(cc: IntCC) -> bool {
    match cc {
        IntCC::Equal
        | IntCC::UnsignedGreaterThanOrEqual
        | IntCC::UnsignedGreaterThan
        | IntCC::UnsignedLessThanOrEqual
        | IntCC::UnsignedLessThan
        | IntCC::NotEqual => false,
        IntCC::SignedGreaterThanOrEqual
        | IntCC::SignedGreaterThan
        | IntCC::SignedLessThanOrEqual
        | IntCC::SignedLessThan
        | IntCC::Overflow
        | IntCC::NotOverflow => true,
    }
}

//=============================================================================
// Helpers for instruction lowering.

pub(crate) fn choose_32_64<T: Copy>(ty: Type, op32: T, op64: T) -> T {
    let bits = ty_bits(ty);
    if bits <= 32 {
        op32
    } else if bits == 64 {
        op64
    } else {
        panic!("choose_32_64 on > 64 bits!")
    }
}

/// Checks for an instance of `op` feeding the given input.
pub(crate) fn maybe_input_insn<C: LowerCtx<I = Inst>>(
    c: &mut C,
    input: InsnInput,
    op: Opcode,
) -> Option<IRInst> {
    let inputs = c.get_input_as_source_or_const(input.insn, input.input);
    log::trace!(
        "maybe_input_insn: input {:?} has options {:?}; looking for op {:?}",
        input,
        inputs,
        op
    );
    if let Some((src_inst, _)) = inputs.inst {
        let data = c.data(src_inst);
        log::trace!(" -> input inst {:?}", data);
        if data.opcode() == op {
            return Some(src_inst);
        }
    }
    None
}

/// Checks for an instance of any one of `ops` feeding the given input.
pub(crate) fn maybe_input_insn_multi<C: LowerCtx<I = Inst>>(
    c: &mut C,
    input: InsnInput,
    ops: &[Opcode],
) -> Option<(Opcode, IRInst)> {
    for &op in ops {
        if let Some(inst) = maybe_input_insn(c, input, op) {
            return Some((op, inst));
        }
    }
    None
}

/// Checks for an instance of `op` feeding the given input, possibly via a conversion `conv` (e.g.,
/// Bint or a bitcast).
///
/// FIXME cfallin 2020-03-30: this is really ugly. Factor out tree-matching stuff and make it
/// a bit more generic.
pub(crate) fn maybe_input_insn_via_conv<C: LowerCtx<I = Inst>>(
    c: &mut C,
    input: InsnInput,
    op: Opcode,
    conv: Opcode,
) -> Option<IRInst> {
    let inputs = c.get_input_as_source_or_const(input.insn, input.input);
    if let Some((src_inst, _)) = inputs.inst {
        let data = c.data(src_inst);
        if data.opcode() == op {
            return Some(src_inst);
        }
        if data.opcode() == conv {
            let inputs = c.get_input_as_source_or_const(src_inst, 0);
            if let Some((src_inst, _)) = inputs.inst {
                let data = c.data(src_inst);
                if data.opcode() == op {
                    return Some(src_inst);
                }
            }
        }
    }
    None
}

/// Specifies what [lower_icmp] should do when lowering
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IcmpOutput {
    /// Lowers the comparison into a cond code, discarding the results. The cond code emitted can
    /// be checked in the resulting [IcmpResult].
    CondCode,
    /// Materializes the results into a register. This may overwrite any flags previously set.
    Register(Writable<Reg>),
}

impl IcmpOutput {
    pub fn reg(&self) -> Option<Writable<Reg>> {
        match self {
            IcmpOutput::CondCode => None,
            IcmpOutput::Register(reg) => Some(*reg),
        }
    }
}

/// The output of an Icmp lowering.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IcmpResult {
    /// The result was output into the given [Cond]. Callers may perform operations using this [Cond]
    /// and its inverse, other [Cond]'s are not guaranteed to be correct.
    CondCode(Cond),
    /// The result was materialized into the output register.
    Register,
}

impl IcmpResult {
    pub fn unwrap_cond(&self) -> Cond {
        match self {
            IcmpResult::CondCode(c) => *c,
            _ => panic!("Unwrapped cond, but IcmpResult was {:?}", self),
        }
    }
}

/// Lower an icmp comparision
///
/// We can lower into the status flags, or materialize the result into a register
/// This is controlled by the `output` parameter.
pub(crate) fn lower_icmp<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    insn: IRInst,
    condcode: IntCC,
    output: IcmpOutput,
) -> CodegenResult<IcmpResult> {
    log::trace!(
        "lower_icmp: insn {}, condcode: {}, output: {:?}",
        insn,
        condcode,
        output
    );

    let rd = output.reg().unwrap_or(writable_zero_reg());
    let inputs = insn_inputs(ctx, insn);
    let cond = lower_condcode(condcode);
    let is_signed = condcode_is_signed(condcode);
    let ty = ctx.input_ty(insn, 0);
    let bits = ty_bits(ty);
    let narrow_mode = match (bits <= 32, is_signed) {
        (true, true) => NarrowValueMode::SignExtend32,
        (true, false) => NarrowValueMode::ZeroExtend32,
        (false, true) => NarrowValueMode::SignExtend64,
        (false, false) => NarrowValueMode::ZeroExtend64,
    };
    let mut should_materialize = output.reg().is_some();

    let out_condcode = if ty == I128 {
        let lhs = put_input_in_regs(ctx, inputs[0]);
        let rhs = put_input_in_regs(ctx, inputs[1]);

        let tmp1 = ctx.alloc_tmp(I64).only_reg().unwrap();
        let tmp2 = ctx.alloc_tmp(I64).only_reg().unwrap();

        match condcode {
            IntCC::Equal | IntCC::NotEqual => {
                // eor     tmp1, lhs_lo, rhs_lo
                // eor     tmp2, lhs_hi, rhs_hi
                // adds    xzr, tmp1, tmp2
                // cset    dst, {eq, ne}

                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::Eor,
                    size: OperandSize::Size64,
                    rd: tmp1,
                    rn: lhs.regs()[0],
                    rm: rhs.regs()[0],
                });
                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::Eor,
                    size: OperandSize::Size64,
                    rd: tmp2,
                    rn: lhs.regs()[1],
                    rm: rhs.regs()[1],
                });
                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::AddS,
                    size: OperandSize::Size64,
                    rd: writable_zero_reg(),
                    rn: tmp1.to_reg(),
                    rm: tmp2.to_reg(),
                });
            }
            IntCC::Overflow | IntCC::NotOverflow => {
                // We can do an 128bit add while throwing away the results
                // and check the overflow flags at the end.
                //
                // adds    xzr, lhs_lo, rhs_lo
                // adcs    xzr, lhs_hi, rhs_hi
                // cset    dst, {vs, vc}

                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::AddS,
                    size: OperandSize::Size64,
                    rd: writable_zero_reg(),
                    rn: lhs.regs()[0],
                    rm: rhs.regs()[0],
                });
                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::AdcS,
                    size: OperandSize::Size64,
                    rd: writable_zero_reg(),
                    rn: lhs.regs()[1],
                    rm: rhs.regs()[1],
                });
            }
            _ => {
                // cmp     lhs_lo, rhs_lo
                // cset    tmp1, unsigned_cond
                // cmp     lhs_hi, rhs_hi
                // cset    tmp2, cond
                // csel    dst, tmp1, tmp2, eq

                let rd = output.reg().unwrap_or(tmp1);
                let unsigned_cond = lower_condcode(condcode.unsigned());

                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::SubS,
                    size: OperandSize::Size64,
                    rd: writable_zero_reg(),
                    rn: lhs.regs()[0],
                    rm: rhs.regs()[0],
                });
                materialize_bool_result(ctx, insn, tmp1, unsigned_cond);
                ctx.emit(Inst::AluRRR {
                    alu_op: ALUOp::SubS,
                    size: OperandSize::Size64,
                    rd: writable_zero_reg(),
                    rn: lhs.regs()[1],
                    rm: rhs.regs()[1],
                });
                materialize_bool_result(ctx, insn, tmp2, cond);
                ctx.emit(Inst::CSel {
                    cond: Cond::Eq,
                    rd,
                    rn: tmp1.to_reg(),
                    rm: tmp2.to_reg(),
                });

                if output == IcmpOutput::CondCode {
                    // We only need to guarantee that the flags for `cond` are correct, so we can
                    // compare rd with 0 or 1

                    // If we are doing compare or equal, we want to compare with 1 instead of zero
                    if condcode.without_equal() != condcode {
                        lower_constant_u64(ctx, tmp2, 1);
                    }

                    let xzr = zero_reg();
                    let rd = rd.to_reg();
                    let tmp2 = tmp2.to_reg();
                    let (rn, rm) = match condcode {
                        IntCC::SignedGreaterThanOrEqual => (rd, tmp2),
                        IntCC::UnsignedGreaterThanOrEqual => (rd, tmp2),
                        IntCC::SignedLessThanOrEqual => (tmp2, rd),
                        IntCC::UnsignedLessThanOrEqual => (tmp2, rd),
                        IntCC::SignedGreaterThan => (rd, xzr),
                        IntCC::UnsignedGreaterThan => (rd, xzr),
                        IntCC::SignedLessThan => (xzr, rd),
                        IntCC::UnsignedLessThan => (xzr, rd),
                        _ => unreachable!(),
                    };

                    ctx.emit(Inst::AluRRR {
                        alu_op: ALUOp::SubS,
                        size: OperandSize::Size64,
                        rd: writable_zero_reg(),
                        rn,
                        rm,
                    });
                }

                // Prevent a second materialize_bool_result to be emitted at the end of the function
                should_materialize = false;
            }
        }
        cond
    } else if ty.is_vector() {
        assert_ne!(output, IcmpOutput::CondCode);
        should_materialize = false;

        let rn = put_input_in_reg(ctx, inputs[0], narrow_mode);
        let rm = put_input_in_reg(ctx, inputs[1], narrow_mode);
        lower_vector_compare(ctx, rd, rn, rm, ty, cond)?;
        cond
    } else {
        let rn = put_input_in_reg(ctx, inputs[0], narrow_mode);
        let rm = put_input_in_rse_imm12(ctx, inputs[1], narrow_mode);

        let is_overflow = condcode == IntCC::Overflow || condcode == IntCC::NotOverflow;
        let is_small_type = ty == I8 || ty == I16;
        let (cond, rn, rm) = if is_overflow && is_small_type {
            // Overflow checks for non native types require additional instructions, other than
            // just the extend op.
            //
            // TODO: Codegen improvements: Merge the second sxt{h,b} into the following sub instruction.
            //
            // sxt{h,b}  w0, w0
            // sxt{h,b}  w1, w1
            // sub       w0, w0, w1
            // cmp       w0, w0, sxt{h,b}
            //
            // The result of this comparison is either the EQ or NE condition code, so we need to
            // signal that to the caller

            let extend_op = if ty == I8 {
                ExtendOp::SXTB
            } else {
                ExtendOp::SXTH
            };
            let tmp1 = ctx.alloc_tmp(I32).only_reg().unwrap();
            ctx.emit(alu_inst_imm12(ALUOp::Sub, I32, tmp1, rn, rm));

            let out_cond = match condcode {
                IntCC::Overflow => Cond::Ne,
                IntCC::NotOverflow => Cond::Eq,
                _ => unreachable!(),
            };
            (
                out_cond,
                tmp1.to_reg(),
                ResultRSEImm12::RegExtend(tmp1.to_reg(), extend_op),
            )
        } else {
            (cond, rn, rm)
        };

        ctx.emit(alu_inst_imm12(ALUOp::SubS, ty, writable_zero_reg(), rn, rm));
        cond
    };

    // Most of the comparisons above produce flags by default, if the caller requested the result
    // in a register we materialize those flags into a register. Some branches do end up producing
    // the result as a register by default, so we ignore those.
    if should_materialize {
        materialize_bool_result(ctx, insn, rd, cond);
    }

    Ok(match output {
        // We currently never emit a different register than what was asked for
        IcmpOutput::Register(_) => IcmpResult::Register,
        IcmpOutput::CondCode => IcmpResult::CondCode(out_condcode),
    })
}

pub(crate) fn lower_fcmp_or_ffcmp_to_flags<C: LowerCtx<I = Inst>>(ctx: &mut C, insn: IRInst) {
    let ty = ctx.input_ty(insn, 0);
    let bits = ty_bits(ty);
    let inputs = [InsnInput { insn, input: 0 }, InsnInput { insn, input: 1 }];
    let rn = put_input_in_reg(ctx, inputs[0], NarrowValueMode::None);
    let rm = put_input_in_reg(ctx, inputs[1], NarrowValueMode::None);
    match bits {
        32 => {
            ctx.emit(Inst::FpuCmp32 { rn, rm });
        }
        64 => {
            ctx.emit(Inst::FpuCmp64 { rn, rm });
        }
        _ => panic!("Unknown float size"),
    }
}

/// Materialize a boolean value into a register from the flags
/// (e.g set by a comparison).
/// A 0 / -1 (all-ones) result as expected for bool operations.
pub(crate) fn materialize_bool_result<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    insn: IRInst,
    rd: Writable<Reg>,
    cond: Cond,
) {
    // A boolean is 0 / -1; if output width is > 1 use `csetm`,
    // otherwise use `cset`.
    if ty_bits(ctx.output_ty(insn, 0)) > 1 {
        ctx.emit(Inst::CSetm { rd, cond });
    } else {
        ctx.emit(Inst::CSet { rd, cond });
    }
}

/// This is target-word-size dependent.  And it excludes booleans and reftypes.
pub(crate) fn is_valid_atomic_transaction_ty(ty: Type) -> bool {
    match ty {
        I8 | I16 | I32 | I64 => true,
        _ => false,
    }
}

pub(crate) fn emit_atomic_load<C: LowerCtx<I = Inst>>(
    ctx: &mut C,
    rt: Writable<Reg>,
    insn: IRInst,
) -> Inst {
    assert!(ctx.data(insn).opcode() == Opcode::AtomicLoad);
    let inputs = insn_inputs(ctx, insn);
    let rn = put_input_in_reg(ctx, inputs[0], NarrowValueMode::None);
    let access_ty = ctx.output_ty(insn, 0);
    assert!(is_valid_atomic_transaction_ty(access_ty));
    // We're ignoring the result type of the load because the LoadAcquire will
    // explicitly zero extend to the nearest word, and also zero the high half
    // of an X register.
    Inst::LoadAcquire { access_ty, rt, rn }
}

fn load_op_to_ty(op: Opcode) -> Option<Type> {
    match op {
        Opcode::Sload8 | Opcode::Uload8 | Opcode::Sload8Complex | Opcode::Uload8Complex => Some(I8),
        Opcode::Sload16 | Opcode::Uload16 | Opcode::Sload16Complex | Opcode::Uload16Complex => {
            Some(I16)
        }
        Opcode::Sload32 | Opcode::Uload32 | Opcode::Sload32Complex | Opcode::Uload32Complex => {
            Some(I32)
        }
        Opcode::Load | Opcode::LoadComplex => None,
        Opcode::Sload8x8 | Opcode::Uload8x8 | Opcode::Sload8x8Complex | Opcode::Uload8x8Complex => {
            Some(I8X8)
        }
        Opcode::Sload16x4
        | Opcode::Uload16x4
        | Opcode::Sload16x4Complex
        | Opcode::Uload16x4Complex => Some(I16X4),
        Opcode::Sload32x2
        | Opcode::Uload32x2
        | Opcode::Sload32x2Complex
        | Opcode::Uload32x2Complex => Some(I32X2),
        _ => None,
    }
}

/// Helper to lower a load instruction; this is used in several places, because
/// a load can sometimes be merged into another operation.
pub(crate) fn lower_load<
    C: LowerCtx<I = Inst>,
    F: FnMut(&mut C, ValueRegs<Writable<Reg>>, Type, AMode) -> CodegenResult<()>,
>(
    ctx: &mut C,
    ir_inst: IRInst,
    inputs: &[InsnInput],
    output: InsnOutput,
    mut f: F,
) -> CodegenResult<()> {
    let op = ctx.data(ir_inst).opcode();

    let elem_ty = load_op_to_ty(op).unwrap_or_else(|| ctx.output_ty(ir_inst, 0));

    let off = ctx.data(ir_inst).load_store_offset().unwrap();
    let mem = lower_address(ctx, elem_ty, &inputs[..], off);
    let rd = get_output_reg(ctx, output);

    f(ctx, rd, elem_ty, mem)
}

//=============================================================================
// Lowering-backend trait implementation.

impl LowerBackend for AArch64Backend {
    type MInst = Inst;

    fn lower<C: LowerCtx<I = Inst>>(&self, ctx: &mut C, ir_inst: IRInst) -> CodegenResult<()> {
        lower_inst::lower_insn_to_regs(ctx, ir_inst, &self.flags, &self.isa_flags)
    }

    fn lower_branch_group<C: LowerCtx<I = Inst>>(
        &self,
        ctx: &mut C,
        branches: &[IRInst],
        targets: &[MachLabel],
    ) -> CodegenResult<()> {
        lower_inst::lower_branch(ctx, branches, targets)
    }

    fn maybe_pinned_reg(&self) -> Option<Reg> {
        Some(xreg(PINNED_REG))
    }
}
