//! A pre-legalization rewriting pass.
//!
//! This module provides early-stage optimizations. The optimizations found
//! should be useful for already well-optimized code. More general purpose
//! early-stage optimizations can be found in the preopt crate.

use crate::cursor::{Cursor, FuncCursor};
use crate::divconst_magic_numbers::{magic_s32, magic_s64, magic_u32, magic_u64};
use crate::divconst_magic_numbers::{MS32, MS64, MU32, MU64};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::{
    condcodes::{CondCode, IntCC},
    instructions::Opcode,
    types::{I32, I64},
    Block, DataFlowGraph, Function, Inst, InstBuilder, InstructionData, Type, Value,
};
use crate::isa::TargetIsa;
use crate::timing;

#[inline]
/// Replaces the unique result of the instruction inst to an alias of the given value, and
/// replaces the instruction with a nop. Can be used only on instructions producing one unique
/// result, otherwise will assert.
fn replace_single_result_with_alias(dfg: &mut DataFlowGraph, inst: Inst, value: Value) {
    // Replace the result value by an alias.
    let results = dfg.detach_results(inst);
    debug_assert!(results.len(&dfg.value_lists) == 1);
    let result = results.get(0, &dfg.value_lists).unwrap();
    dfg.change_to_alias(result, value);

    // Replace instruction by a nop.
    dfg.replace(inst).nop();
}

//----------------------------------------------------------------------
//
// Pattern-match helpers and transformation for div and rem by constants.

// Simple math helpers

/// if `x` is a power of two, or the negation thereof, return the power along
/// with a boolean that indicates whether `x` is negative. Else return None.
#[inline]
fn i32_is_power_of_two(x: i32) -> Option<(bool, u32)> {
    // We have to special-case this because abs(x) isn't representable.
    if x == -0x8000_0000 {
        return Some((true, 31));
    }
    let abs_x = i32::wrapping_abs(x) as u32;
    if abs_x.is_power_of_two() {
        return Some((x < 0, abs_x.trailing_zeros()));
    }
    None
}

/// Same comments as for i32_is_power_of_two apply.
#[inline]
fn i64_is_power_of_two(x: i64) -> Option<(bool, u32)> {
    // We have to special-case this because abs(x) isn't representable.
    if x == -0x8000_0000_0000_0000 {
        return Some((true, 63));
    }
    let abs_x = i64::wrapping_abs(x) as u64;
    if abs_x.is_power_of_two() {
        return Some((x < 0, abs_x.trailing_zeros()));
    }
    None
}

/// Representation of an instruction that can be replaced by a single division/remainder operation
/// between a left Value operand and a right immediate operand.
#[derive(Debug)]
enum DivRemByConstInfo {
    DivU32(Value, u32),
    DivU64(Value, u64),
    DivS32(Value, i32),
    DivS64(Value, i64),
    RemU32(Value, u32),
    RemU64(Value, u64),
    RemS32(Value, i32),
    RemS64(Value, i64),
}

/// Possibly create a DivRemByConstInfo from the given components, by figuring out which, if any,
/// of the 8 cases apply, and also taking care to sanity-check the immediate.
fn package_up_divrem_info(
    value: Value,
    value_type: Type,
    imm_i64: i64,
    is_signed: bool,
    is_rem: bool,
) -> Option<DivRemByConstInfo> {
    let imm_u64 = imm_i64 as u64;

    match (is_signed, value_type) {
        (false, I32) => {
            if imm_u64 < 0x1_0000_0000 {
                if is_rem {
                    Some(DivRemByConstInfo::RemU32(value, imm_u64 as u32))
                } else {
                    Some(DivRemByConstInfo::DivU32(value, imm_u64 as u32))
                }
            } else {
                None
            }
        }

        (false, I64) => {
            // unsigned 64, no range constraint.
            if is_rem {
                Some(DivRemByConstInfo::RemU64(value, imm_u64))
            } else {
                Some(DivRemByConstInfo::DivU64(value, imm_u64))
            }
        }

        (true, I32) => {
            if imm_u64 <= 0x7fff_ffff || imm_u64 >= 0xffff_ffff_8000_0000 {
                if is_rem {
                    Some(DivRemByConstInfo::RemS32(value, imm_u64 as i32))
                } else {
                    Some(DivRemByConstInfo::DivS32(value, imm_u64 as i32))
                }
            } else {
                None
            }
        }

        (true, I64) => {
            // signed 64, no range constraint.
            if is_rem {
                Some(DivRemByConstInfo::RemS64(value, imm_u64 as i64))
            } else {
                Some(DivRemByConstInfo::DivS64(value, imm_u64 as i64))
            }
        }

        _ => None,
    }
}

/// Examine `inst` to see if it is a div or rem by a constant, and if so return the operands,
/// signedness, operation size and div-vs-rem-ness in a handy bundle.
fn get_div_info(inst: Inst, dfg: &DataFlowGraph) -> Option<DivRemByConstInfo> {
    if let InstructionData::BinaryImm64 { opcode, arg, imm } = dfg[inst] {
        let (is_signed, is_rem) = match opcode {
            Opcode::UdivImm => (false, false),
            Opcode::UremImm => (false, true),
            Opcode::SdivImm => (true, false),
            Opcode::SremImm => (true, true),
            _ => return None,
        };
        return package_up_divrem_info(arg, dfg.value_type(arg), imm.into(), is_signed, is_rem);
    }

    None
}

/// Actually do the transformation given a bundle containing the relevant information.
/// `divrem_info` describes a div or rem by a constant, that `pos` currently points at, and `inst`
/// is the associated instruction.  `inst` is replaced by a sequence of other operations that
/// calculate the same result. Note that there are various `divrem_info` cases where we cannot do
/// any transformation, in which case `inst` is left unchanged.
fn do_divrem_transformation(divrem_info: &DivRemByConstInfo, pos: &mut FuncCursor, inst: Inst) {
    let is_rem = match *divrem_info {
        DivRemByConstInfo::DivU32(_, _)
        | DivRemByConstInfo::DivU64(_, _)
        | DivRemByConstInfo::DivS32(_, _)
        | DivRemByConstInfo::DivS64(_, _) => false,
        DivRemByConstInfo::RemU32(_, _)
        | DivRemByConstInfo::RemU64(_, _)
        | DivRemByConstInfo::RemS32(_, _)
        | DivRemByConstInfo::RemS64(_, _) => true,
    };

    match *divrem_info {
        // -------------------- U32 --------------------

        // U32 div, rem by zero: ignore
        DivRemByConstInfo::DivU32(_n1, 0) | DivRemByConstInfo::RemU32(_n1, 0) => {}

        // U32 div by 1: identity
        // U32 rem by 1: zero
        DivRemByConstInfo::DivU32(n1, 1) | DivRemByConstInfo::RemU32(n1, 1) => {
            if is_rem {
                pos.func.dfg.replace(inst).iconst(I32, 0);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, n1);
            }
        }

        // U32 div, rem by a power-of-2
        DivRemByConstInfo::DivU32(n1, d) | DivRemByConstInfo::RemU32(n1, d)
            if d.is_power_of_two() =>
        {
            debug_assert!(d >= 2);
            // compute k where d == 2^k
            let k = d.trailing_zeros();
            debug_assert!(k >= 1 && k <= 31);
            if is_rem {
                let mask = (1u64 << k) - 1;
                pos.func.dfg.replace(inst).band_imm(n1, mask as i64);
            } else {
                pos.func.dfg.replace(inst).ushr_imm(n1, k as i64);
            }
        }

        // U32 div, rem by non-power-of-2
        DivRemByConstInfo::DivU32(n1, d) | DivRemByConstInfo::RemU32(n1, d) => {
            debug_assert!(d >= 3);
            let MU32 {
                mul_by,
                do_add,
                shift_by,
            } = magic_u32(d);
            let qf; // final quotient
            let q0 = pos.ins().iconst(I32, mul_by as i64);
            let q1 = pos.ins().umulhi(n1, q0);
            if do_add {
                debug_assert!(shift_by >= 1 && shift_by <= 32);
                let t1 = pos.ins().isub(n1, q1);
                let t2 = pos.ins().ushr_imm(t1, 1);
                let t3 = pos.ins().iadd(t2, q1);
                // I never found any case where shift_by == 1 here.
                // So there's no attempt to fold out a zero shift.
                debug_assert_ne!(shift_by, 1);
                qf = pos.ins().ushr_imm(t3, (shift_by - 1) as i64);
            } else {
                debug_assert!(shift_by >= 0 && shift_by <= 31);
                // Whereas there are known cases here for shift_by == 0.
                if shift_by > 0 {
                    qf = pos.ins().ushr_imm(q1, shift_by as i64);
                } else {
                    qf = q1;
                }
            }
            // Now qf holds the final quotient. If necessary calculate the
            // remainder instead.
            if is_rem {
                let tt = pos.ins().imul_imm(qf, d as i64);
                pos.func.dfg.replace(inst).isub(n1, tt);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, qf);
            }
        }

        // -------------------- U64 --------------------

        // U64 div, rem by zero: ignore
        DivRemByConstInfo::DivU64(_n1, 0) | DivRemByConstInfo::RemU64(_n1, 0) => {}

        // U64 div by 1: identity
        // U64 rem by 1: zero
        DivRemByConstInfo::DivU64(n1, 1) | DivRemByConstInfo::RemU64(n1, 1) => {
            if is_rem {
                pos.func.dfg.replace(inst).iconst(I64, 0);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, n1);
            }
        }

        // U64 div, rem by a power-of-2
        DivRemByConstInfo::DivU64(n1, d) | DivRemByConstInfo::RemU64(n1, d)
            if d.is_power_of_two() =>
        {
            debug_assert!(d >= 2);
            // compute k where d == 2^k
            let k = d.trailing_zeros();
            debug_assert!(k >= 1 && k <= 63);
            if is_rem {
                let mask = (1u64 << k) - 1;
                pos.func.dfg.replace(inst).band_imm(n1, mask as i64);
            } else {
                pos.func.dfg.replace(inst).ushr_imm(n1, k as i64);
            }
        }

        // U64 div, rem by non-power-of-2
        DivRemByConstInfo::DivU64(n1, d) | DivRemByConstInfo::RemU64(n1, d) => {
            debug_assert!(d >= 3);
            let MU64 {
                mul_by,
                do_add,
                shift_by,
            } = magic_u64(d);
            let qf; // final quotient
            let q0 = pos.ins().iconst(I64, mul_by as i64);
            let q1 = pos.ins().umulhi(n1, q0);
            if do_add {
                debug_assert!(shift_by >= 1 && shift_by <= 64);
                let t1 = pos.ins().isub(n1, q1);
                let t2 = pos.ins().ushr_imm(t1, 1);
                let t3 = pos.ins().iadd(t2, q1);
                // I never found any case where shift_by == 1 here.
                // So there's no attempt to fold out a zero shift.
                debug_assert_ne!(shift_by, 1);
                qf = pos.ins().ushr_imm(t3, (shift_by - 1) as i64);
            } else {
                debug_assert!(shift_by >= 0 && shift_by <= 63);
                // Whereas there are known cases here for shift_by == 0.
                if shift_by > 0 {
                    qf = pos.ins().ushr_imm(q1, shift_by as i64);
                } else {
                    qf = q1;
                }
            }
            // Now qf holds the final quotient. If necessary calculate the
            // remainder instead.
            if is_rem {
                let tt = pos.ins().imul_imm(qf, d as i64);
                pos.func.dfg.replace(inst).isub(n1, tt);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, qf);
            }
        }

        // -------------------- S32 --------------------

        // S32 div, rem by zero or -1: ignore
        DivRemByConstInfo::DivS32(_n1, -1)
        | DivRemByConstInfo::RemS32(_n1, -1)
        | DivRemByConstInfo::DivS32(_n1, 0)
        | DivRemByConstInfo::RemS32(_n1, 0) => {}

        // S32 div by 1: identity
        // S32 rem by 1: zero
        DivRemByConstInfo::DivS32(n1, 1) | DivRemByConstInfo::RemS32(n1, 1) => {
            if is_rem {
                pos.func.dfg.replace(inst).iconst(I32, 0);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, n1);
            }
        }

        DivRemByConstInfo::DivS32(n1, d) | DivRemByConstInfo::RemS32(n1, d) => {
            if let Some((is_negative, k)) = i32_is_power_of_two(d) {
                // k can be 31 only in the case that d is -2^31.
                debug_assert!(k >= 1 && k <= 31);
                let t1 = if k - 1 == 0 {
                    n1
                } else {
                    pos.ins().sshr_imm(n1, (k - 1) as i64)
                };
                let t2 = pos.ins().ushr_imm(t1, (32 - k) as i64);
                let t3 = pos.ins().iadd(n1, t2);
                if is_rem {
                    // S32 rem by a power-of-2
                    let t4 = pos.ins().band_imm(t3, i32::wrapping_neg(1 << k) as i64);
                    // Curiously, we don't care here what the sign of d is.
                    pos.func.dfg.replace(inst).isub(n1, t4);
                } else {
                    // S32 div by a power-of-2
                    let t4 = pos.ins().sshr_imm(t3, k as i64);
                    if is_negative {
                        pos.func.dfg.replace(inst).irsub_imm(t4, 0);
                    } else {
                        replace_single_result_with_alias(&mut pos.func.dfg, inst, t4);
                    }
                }
            } else {
                // S32 div, rem by a non-power-of-2
                debug_assert!(d < -2 || d > 2);
                let MS32 { mul_by, shift_by } = magic_s32(d);
                let q0 = pos.ins().iconst(I32, mul_by as i64);
                let q1 = pos.ins().smulhi(n1, q0);
                let q2 = if d > 0 && mul_by < 0 {
                    pos.ins().iadd(q1, n1)
                } else if d < 0 && mul_by > 0 {
                    pos.ins().isub(q1, n1)
                } else {
                    q1
                };
                debug_assert!(shift_by >= 0 && shift_by <= 31);
                let q3 = if shift_by == 0 {
                    q2
                } else {
                    pos.ins().sshr_imm(q2, shift_by as i64)
                };
                let t1 = pos.ins().ushr_imm(q3, 31);
                let qf = pos.ins().iadd(q3, t1);
                // Now qf holds the final quotient. If necessary calculate
                // the remainder instead.
                if is_rem {
                    let tt = pos.ins().imul_imm(qf, d as i64);
                    pos.func.dfg.replace(inst).isub(n1, tt);
                } else {
                    replace_single_result_with_alias(&mut pos.func.dfg, inst, qf);
                }
            }
        }

        // -------------------- S64 --------------------

        // S64 div, rem by zero or -1: ignore
        DivRemByConstInfo::DivS64(_n1, -1)
        | DivRemByConstInfo::RemS64(_n1, -1)
        | DivRemByConstInfo::DivS64(_n1, 0)
        | DivRemByConstInfo::RemS64(_n1, 0) => {}

        // S64 div by 1: identity
        // S64 rem by 1: zero
        DivRemByConstInfo::DivS64(n1, 1) | DivRemByConstInfo::RemS64(n1, 1) => {
            if is_rem {
                pos.func.dfg.replace(inst).iconst(I64, 0);
            } else {
                replace_single_result_with_alias(&mut pos.func.dfg, inst, n1);
            }
        }

        DivRemByConstInfo::DivS64(n1, d) | DivRemByConstInfo::RemS64(n1, d) => {
            if let Some((is_negative, k)) = i64_is_power_of_two(d) {
                // k can be 63 only in the case that d is -2^63.
                debug_assert!(k >= 1 && k <= 63);
                let t1 = if k - 1 == 0 {
                    n1
                } else {
                    pos.ins().sshr_imm(n1, (k - 1) as i64)
                };
                let t2 = pos.ins().ushr_imm(t1, (64 - k) as i64);
                let t3 = pos.ins().iadd(n1, t2);
                if is_rem {
                    // S64 rem by a power-of-2
                    let t4 = pos.ins().band_imm(t3, i64::wrapping_neg(1 << k));
                    // Curiously, we don't care here what the sign of d is.
                    pos.func.dfg.replace(inst).isub(n1, t4);
                } else {
                    // S64 div by a power-of-2
                    let t4 = pos.ins().sshr_imm(t3, k as i64);
                    if is_negative {
                        pos.func.dfg.replace(inst).irsub_imm(t4, 0);
                    } else {
                        replace_single_result_with_alias(&mut pos.func.dfg, inst, t4);
                    }
                }
            } else {
                // S64 div, rem by a non-power-of-2
                debug_assert!(d < -2 || d > 2);
                let MS64 { mul_by, shift_by } = magic_s64(d);
                let q0 = pos.ins().iconst(I64, mul_by);
                let q1 = pos.ins().smulhi(n1, q0);
                let q2 = if d > 0 && mul_by < 0 {
                    pos.ins().iadd(q1, n1)
                } else if d < 0 && mul_by > 0 {
                    pos.ins().isub(q1, n1)
                } else {
                    q1
                };
                debug_assert!(shift_by >= 0 && shift_by <= 63);
                let q3 = if shift_by == 0 {
                    q2
                } else {
                    pos.ins().sshr_imm(q2, shift_by as i64)
                };
                let t1 = pos.ins().ushr_imm(q3, 63);
                let qf = pos.ins().iadd(q3, t1);
                // Now qf holds the final quotient. If necessary calculate
                // the remainder instead.
                if is_rem {
                    let tt = pos.ins().imul_imm(qf, d);
                    pos.func.dfg.replace(inst).isub(n1, tt);
                } else {
                    replace_single_result_with_alias(&mut pos.func.dfg, inst, qf);
                }
            }
        }
    }
}

enum BranchOrderKind {
    BrzToBrnz(Value),
    BrnzToBrz(Value),
    InvertIcmpCond(IntCC, Value, Value),
}

/// Reorder branches to encourage fallthroughs.
///
/// When a block ends with a conditional branch followed by an unconditional
/// branch, this will reorder them if one of them is branching to the next Block
/// layout-wise. The unconditional jump can then become a fallthrough.
fn branch_order(pos: &mut FuncCursor, cfg: &mut ControlFlowGraph, block: Block, inst: Inst) {
    let (term_inst, term_inst_args, term_dest, cond_inst, cond_inst_args, cond_dest, kind) =
        match pos.func.dfg[inst] {
            InstructionData::Jump {
                opcode: Opcode::Jump,
                destination,
                ref args,
            } => {
                let next_block = if let Some(next_block) = pos.func.layout.next_block(block) {
                    next_block
                } else {
                    return;
                };

                if destination == next_block {
                    return;
                }

                let prev_inst = if let Some(prev_inst) = pos.func.layout.prev_inst(inst) {
                    prev_inst
                } else {
                    return;
                };

                let prev_inst_data = &pos.func.dfg[prev_inst];

                if let Some(prev_dest) = prev_inst_data.branch_destination() {
                    if prev_dest != next_block {
                        return;
                    }
                } else {
                    return;
                }

                match prev_inst_data {
                    InstructionData::Branch {
                        opcode,
                        args: ref prev_args,
                        destination: cond_dest,
                    } => {
                        let cond_arg = {
                            let args = pos.func.dfg.inst_args(prev_inst);
                            args[0]
                        };

                        let kind = match opcode {
                            Opcode::Brz => BranchOrderKind::BrzToBrnz(cond_arg),
                            Opcode::Brnz => BranchOrderKind::BrnzToBrz(cond_arg),
                            _ => panic!("unexpected opcode"),
                        };

                        (
                            inst,
                            args.clone(),
                            destination,
                            prev_inst,
                            prev_args.clone(),
                            *cond_dest,
                            kind,
                        )
                    }
                    InstructionData::BranchIcmp {
                        opcode: Opcode::BrIcmp,
                        cond,
                        destination: cond_dest,
                        args: ref prev_args,
                    } => {
                        let (x_arg, y_arg) = {
                            let args = pos.func.dfg.inst_args(prev_inst);
                            (args[0], args[1])
                        };

                        (
                            inst,
                            args.clone(),
                            destination,
                            prev_inst,
                            prev_args.clone(),
                            *cond_dest,
                            BranchOrderKind::InvertIcmpCond(*cond, x_arg, y_arg),
                        )
                    }
                    _ => return,
                }
            }

            _ => return,
        };

    let cond_args = cond_inst_args.as_slice(&pos.func.dfg.value_lists).to_vec();
    let term_args = term_inst_args.as_slice(&pos.func.dfg.value_lists).to_vec();

    match kind {
        BranchOrderKind::BrnzToBrz(cond_arg) => {
            pos.func
                .dfg
                .replace(term_inst)
                .jump(cond_dest, &cond_args[1..]);
            pos.func
                .dfg
                .replace(cond_inst)
                .brz(cond_arg, term_dest, &term_args);
        }
        BranchOrderKind::BrzToBrnz(cond_arg) => {
            pos.func
                .dfg
                .replace(term_inst)
                .jump(cond_dest, &cond_args[1..]);
            pos.func
                .dfg
                .replace(cond_inst)
                .brnz(cond_arg, term_dest, &term_args);
        }
        BranchOrderKind::InvertIcmpCond(cond, x_arg, y_arg) => {
            pos.func
                .dfg
                .replace(term_inst)
                .jump(cond_dest, &cond_args[2..]);
            pos.func.dfg.replace(cond_inst).br_icmp(
                cond.inverse(),
                x_arg,
                y_arg,
                term_dest,
                &term_args,
            );
        }
    }

    cfg.recompute_block(pos.func, block);
}

mod simplify {
    use super::*;
    use crate::ir::{
        dfg::ValueDef,
        immediates,
        instructions::{Opcode, ValueList},
        types::{B8, I16, I32, I8},
    };
    use std::marker::PhantomData;

    pub struct PeepholeOptimizer<'a, 'b> {
        phantom: PhantomData<(&'a (), &'b ())>,
    }

    pub fn peephole_optimizer<'a, 'b>(_: &dyn TargetIsa) -> PeepholeOptimizer<'a, 'b> {
        PeepholeOptimizer {
            phantom: PhantomData,
        }
    }

    pub fn apply_all<'a, 'b>(
        _optimizer: &mut PeepholeOptimizer<'a, 'b>,
        pos: &mut FuncCursor<'a>,
        inst: Inst,
        native_word_width: u32,
    ) {
        simplify(pos, inst, native_word_width);
        branch_opt(pos, inst);
    }

    #[inline]
    fn resolve_imm64_value(dfg: &DataFlowGraph, value: Value) -> Option<immediates::Imm64> {
        if let ValueDef::Result(candidate_inst, _) = dfg.value_def(value) {
            if let InstructionData::UnaryImm {
                opcode: Opcode::Iconst,
                imm,
            } = dfg[candidate_inst]
            {
                return Some(imm);
            }
        }
        None
    }

    /// Try to transform [(x << N) >> N] into a (un)signed-extending move.
    /// Returns true if the final instruction has been converted to such a move.
    fn try_fold_extended_move(
        pos: &mut FuncCursor,
        inst: Inst,
        opcode: Opcode,
        arg: Value,
        imm: immediates::Imm64,
    ) -> bool {
        if let ValueDef::Result(arg_inst, _) = pos.func.dfg.value_def(arg) {
            if let InstructionData::BinaryImm64 {
                opcode: Opcode::IshlImm,
                arg: prev_arg,
                imm: prev_imm,
            } = &pos.func.dfg[arg_inst]
            {
                if imm != *prev_imm {
                    return false;
                }

                let dest_ty = pos.func.dfg.ctrl_typevar(inst);
                if dest_ty != pos.func.dfg.ctrl_typevar(arg_inst) || !dest_ty.is_int() {
                    return false;
                }

                let imm_bits: i64 = imm.into();
                let ireduce_ty = match (dest_ty.lane_bits() as i64).wrapping_sub(imm_bits) {
                    8 => I8,
                    16 => I16,
                    32 => I32,
                    _ => return false,
                };
                let ireduce_ty = ireduce_ty.by(dest_ty.lane_count()).unwrap();

                // This becomes a no-op, since ireduce_ty has a smaller lane width than
                // the argument type (also the destination type).
                let arg = *prev_arg;
                let narrower_arg = pos.ins().ireduce(ireduce_ty, arg);

                if opcode == Opcode::UshrImm {
                    pos.func.dfg.replace(inst).uextend(dest_ty, narrower_arg);
                } else {
                    pos.func.dfg.replace(inst).sextend(dest_ty, narrower_arg);
                }
                return true;
            }
        }
        false
    }

    /// Apply basic simplifications.
    ///
    /// This folds constants with arithmetic to form `_imm` instructions, and other minor
    /// simplifications.
    ///
    /// Doesn't apply some simplifications if the native word width (in bytes) is smaller than the
    /// controlling type's width of the instruction. This would result in an illegal instruction that
    /// would likely be expanded back into an instruction on smaller types with the same initial
    /// opcode, creating unnecessary churn.
    fn simplify(pos: &mut FuncCursor, inst: Inst, native_word_width: u32) {
        match pos.func.dfg[inst] {
            InstructionData::Binary { opcode, args } => {
                if let Some(mut imm) = resolve_imm64_value(&pos.func.dfg, args[1]) {
                    let new_opcode = match opcode {
                        Opcode::Iadd => Opcode::IaddImm,
                        Opcode::Imul => Opcode::ImulImm,
                        Opcode::Sdiv => Opcode::SdivImm,
                        Opcode::Udiv => Opcode::UdivImm,
                        Opcode::Srem => Opcode::SremImm,
                        Opcode::Urem => Opcode::UremImm,
                        Opcode::Band => Opcode::BandImm,
                        Opcode::Bor => Opcode::BorImm,
                        Opcode::Bxor => Opcode::BxorImm,
                        Opcode::Rotl => Opcode::RotlImm,
                        Opcode::Rotr => Opcode::RotrImm,
                        Opcode::Ishl => Opcode::IshlImm,
                        Opcode::Ushr => Opcode::UshrImm,
                        Opcode::Sshr => Opcode::SshrImm,
                        Opcode::Isub => {
                            imm = imm.wrapping_neg();
                            Opcode::IaddImm
                        }
                        Opcode::Ifcmp => Opcode::IfcmpImm,
                        _ => return,
                    };
                    let ty = pos.func.dfg.ctrl_typevar(inst);
                    if ty.bytes() <= native_word_width {
                        pos.func
                            .dfg
                            .replace(inst)
                            .BinaryImm64(new_opcode, ty, imm, args[0]);

                        // Repeat for BinaryImm simplification.
                        simplify(pos, inst, native_word_width);
                    }
                } else if let Some(imm) = resolve_imm64_value(&pos.func.dfg, args[0]) {
                    let new_opcode = match opcode {
                        Opcode::Iadd => Opcode::IaddImm,
                        Opcode::Imul => Opcode::ImulImm,
                        Opcode::Band => Opcode::BandImm,
                        Opcode::Bor => Opcode::BorImm,
                        Opcode::Bxor => Opcode::BxorImm,
                        Opcode::Isub => Opcode::IrsubImm,
                        _ => return,
                    };
                    let ty = pos.func.dfg.ctrl_typevar(inst);
                    if ty.bytes() <= native_word_width {
                        pos.func
                            .dfg
                            .replace(inst)
                            .BinaryImm64(new_opcode, ty, imm, args[1]);
                    }
                }
            }

            InstructionData::BinaryImm64 { opcode, arg, imm } => {
                let ty = pos.func.dfg.ctrl_typevar(inst);

                let mut arg = arg;
                let mut imm = imm;
                match opcode {
                    Opcode::IaddImm
                    | Opcode::ImulImm
                    | Opcode::BorImm
                    | Opcode::BandImm
                    | Opcode::BxorImm => {
                        // Fold binary_op(C2, binary_op(C1, x)) into binary_op(binary_op(C1, C2), x)
                        if let ValueDef::Result(arg_inst, _) = pos.func.dfg.value_def(arg) {
                            if let InstructionData::BinaryImm64 {
                                opcode: prev_opcode,
                                arg: prev_arg,
                                imm: prev_imm,
                            } = &pos.func.dfg[arg_inst]
                            {
                                if opcode == *prev_opcode
                                    && ty == pos.func.dfg.ctrl_typevar(arg_inst)
                                {
                                    let lhs: i64 = imm.into();
                                    let rhs: i64 = (*prev_imm).into();
                                    let new_imm = match opcode {
                                        Opcode::BorImm => lhs | rhs,
                                        Opcode::BandImm => lhs & rhs,
                                        Opcode::BxorImm => lhs ^ rhs,
                                        Opcode::IaddImm => lhs.wrapping_add(rhs),
                                        Opcode::ImulImm => lhs.wrapping_mul(rhs),
                                        _ => panic!("can't happen"),
                                    };
                                    let new_imm = immediates::Imm64::from(new_imm);
                                    let new_arg = *prev_arg;
                                    pos.func
                                        .dfg
                                        .replace(inst)
                                        .BinaryImm64(opcode, ty, new_imm, new_arg);
                                    imm = new_imm;
                                    arg = new_arg;
                                }
                            }
                        }
                    }

                    Opcode::UshrImm | Opcode::SshrImm => {
                        if pos.func.dfg.ctrl_typevar(inst).bytes() <= native_word_width
                            && try_fold_extended_move(pos, inst, opcode, arg, imm)
                        {
                            return;
                        }
                    }

                    _ => {}
                };

                // Replace operations that are no-ops.
                match (opcode, imm.into()) {
                    (Opcode::IaddImm, 0)
                    | (Opcode::ImulImm, 1)
                    | (Opcode::SdivImm, 1)
                    | (Opcode::UdivImm, 1)
                    | (Opcode::BorImm, 0)
                    | (Opcode::BandImm, -1)
                    | (Opcode::BxorImm, 0)
                    | (Opcode::RotlImm, 0)
                    | (Opcode::RotrImm, 0)
                    | (Opcode::IshlImm, 0)
                    | (Opcode::UshrImm, 0)
                    | (Opcode::SshrImm, 0) => {
                        // Alias the result value with the original argument.
                        replace_single_result_with_alias(&mut pos.func.dfg, inst, arg);
                    }
                    (Opcode::ImulImm, 0) | (Opcode::BandImm, 0) => {
                        // Replace by zero.
                        pos.func.dfg.replace(inst).iconst(ty, 0);
                    }
                    (Opcode::BorImm, -1) => {
                        // Replace by minus one.
                        pos.func.dfg.replace(inst).iconst(ty, -1);
                    }
                    _ => {}
                }
            }

            InstructionData::IntCompare { opcode, cond, args } => {
                debug_assert_eq!(opcode, Opcode::Icmp);
                if let Some(imm) = resolve_imm64_value(&pos.func.dfg, args[1]) {
                    if pos.func.dfg.ctrl_typevar(inst).bytes() <= native_word_width {
                        pos.func.dfg.replace(inst).icmp_imm(cond, args[0], imm);
                    }
                }
            }

            InstructionData::CondTrap { .. }
            | InstructionData::Branch { .. }
            | InstructionData::Ternary {
                opcode: Opcode::Select,
                ..
            } => {
                // Fold away a redundant `bint`.
                let condition_def = {
                    let args = pos.func.dfg.inst_args(inst);
                    pos.func.dfg.value_def(args[0])
                };
                if let ValueDef::Result(def_inst, _) = condition_def {
                    if let InstructionData::Unary {
                        opcode: Opcode::Bint,
                        arg: bool_val,
                    } = pos.func.dfg[def_inst]
                    {
                        let args = pos.func.dfg.inst_args_mut(inst);
                        args[0] = bool_val;
                    }
                }
            }

            InstructionData::Ternary {
                opcode: Opcode::Bitselect,
                args,
            } => {
                let old_cond_type = pos.func.dfg.value_type(args[0]);
                if !old_cond_type.is_vector() {
                    return;
                }

                // Replace bitselect with vselect if each lane of controlling mask is either
                // all ones or all zeroes; on x86 bitselect is encoded using 3 instructions,
                // while vselect can be encoded using single BLEND instruction.
                if let ValueDef::Result(def_inst, _) = pos.func.dfg.value_def(args[0]) {
                    let (cond_val, cond_type) = match pos.func.dfg[def_inst] {
                        InstructionData::Unary {
                            opcode: Opcode::RawBitcast,
                            arg,
                        } => {
                            // If controlling mask is raw-bitcasted boolean vector then
                            // we know each lane is either all zeroes or ones,
                            // so we can use vselect instruction instead.
                            let arg_type = pos.func.dfg.value_type(arg);
                            if !arg_type.is_vector() || !arg_type.lane_type().is_bool() {
                                return;
                            }
                            (arg, arg_type)
                        }
                        InstructionData::UnaryConst {
                            opcode: Opcode::Vconst,
                            constant_handle,
                        } => {
                            // If each byte of controlling mask is 0x00 or 0xFF then
                            // we will always bitcast our way to vselect(B8x16, I8x16, I8x16).
                            // Bitselect operates at bit level, so the lane types don't matter.
                            let const_data = pos.func.dfg.constants.get(constant_handle);
                            if !const_data.iter().all(|&b| b == 0 || b == 0xFF) {
                                return;
                            }
                            let new_type = B8.by(old_cond_type.bytes() as u16).unwrap();
                            (pos.ins().raw_bitcast(new_type, args[0]), new_type)
                        }
                        _ => return,
                    };

                    let lane_type = Type::int(cond_type.lane_bits() as u16).unwrap();
                    let arg_type = lane_type.by(cond_type.lane_count()).unwrap();
                    let old_arg_type = pos.func.dfg.value_type(args[1]);

                    if arg_type != old_arg_type {
                        // Operands types must match, we need to add bitcasts.
                        let arg1 = pos.ins().raw_bitcast(arg_type, args[1]);
                        let arg2 = pos.ins().raw_bitcast(arg_type, args[2]);
                        let ret = pos.ins().vselect(cond_val, arg1, arg2);
                        pos.func.dfg.replace(inst).raw_bitcast(old_arg_type, ret);
                    } else {
                        pos.func
                            .dfg
                            .replace(inst)
                            .vselect(cond_val, args[1], args[2]);
                    }
                }
            }

            _ => {}
        }
    }

    struct BranchOptInfo {
        br_inst: Inst,
        cmp_arg: Value,
        args: ValueList,
        new_opcode: Opcode,
    }

    /// Fold comparisons into branch operations when possible.
    ///
    /// This matches against operations which compare against zero, then use the
    /// result in a `brz` or `brnz` branch. It folds those two operations into a
    /// single `brz` or `brnz`.
    fn branch_opt(pos: &mut FuncCursor, inst: Inst) {
        let mut info = if let InstructionData::Branch {
            opcode: br_opcode,
            args: ref br_args,
            ..
        } = pos.func.dfg[inst]
        {
            let first_arg = {
                let args = pos.func.dfg.inst_args(inst);
                args[0]
            };

            let icmp_inst =
                if let ValueDef::Result(icmp_inst, _) = pos.func.dfg.value_def(first_arg) {
                    icmp_inst
                } else {
                    return;
                };

            if let InstructionData::IntCompareImm {
                opcode: Opcode::IcmpImm,
                arg: cmp_arg,
                cond: cmp_cond,
                imm: cmp_imm,
            } = pos.func.dfg[icmp_inst]
            {
                let cmp_imm: i64 = cmp_imm.into();
                if cmp_imm != 0 {
                    return;
                }

                // icmp_imm returns non-zero when the comparison is true. So, if
                // we're branching on zero, we need to invert the condition.
                let cond = match br_opcode {
                    Opcode::Brz => cmp_cond.inverse(),
                    Opcode::Brnz => cmp_cond,
                    _ => return,
                };

                let new_opcode = match cond {
                    IntCC::Equal => Opcode::Brz,
                    IntCC::NotEqual => Opcode::Brnz,
                    _ => return,
                };

                BranchOptInfo {
                    br_inst: inst,
                    cmp_arg,
                    args: br_args.clone(),
                    new_opcode,
                }
            } else {
                return;
            }
        } else {
            return;
        };

        info.args.as_mut_slice(&mut pos.func.dfg.value_lists)[0] = info.cmp_arg;
        if let InstructionData::Branch { ref mut opcode, .. } = pos.func.dfg[info.br_inst] {
            *opcode = info.new_opcode;
        } else {
            panic!();
        }
    }
}

/// The main pre-opt pass.
pub fn do_preopt(func: &mut Function, cfg: &mut ControlFlowGraph, isa: &dyn TargetIsa) {
    let _tt = timing::preopt();

    let mut pos = FuncCursor::new(func);
    let native_word_width = isa.pointer_bytes() as u32;
    let mut optimizer = simplify::peephole_optimizer(isa);

    while let Some(block) = pos.next_block() {
        while let Some(inst) = pos.next_inst() {
            simplify::apply_all(&mut optimizer, &mut pos, inst, native_word_width);

            // Try to transform divide-by-constant into simpler operations.
            if let Some(divrem_info) = get_div_info(inst, &pos.func.dfg) {
                do_divrem_transformation(&divrem_info, &mut pos, inst);
                continue;
            }

            branch_order(&mut pos, cfg, block, inst);
        }
    }
}
