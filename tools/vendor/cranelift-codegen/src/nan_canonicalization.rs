//! A NaN-canonicalizing rewriting pass. Patch floating point arithmetic
//! instructions that may return a NaN result with a sequence of operations
//! that will replace nondeterministic NaN's with a single canonical NaN value.

use crate::cursor::{Cursor, FuncCursor};
use crate::ir::condcodes::FloatCC;
use crate::ir::immediates::{Ieee32, Ieee64};
use crate::ir::types;
use crate::ir::{Function, Inst, InstBuilder, InstructionData, Opcode, Value};
use crate::timing;

// Canonical 32-bit and 64-bit NaN values.
static CANON_32BIT_NAN: u32 = 0b01111111110000000000000000000000;
static CANON_64BIT_NAN: u64 = 0b0111111111111000000000000000000000000000000000000000000000000000;

/// Perform the NaN canonicalization pass.
pub fn do_nan_canonicalization(func: &mut Function) {
    let _tt = timing::canonicalize_nans();
    let mut pos = FuncCursor::new(func);
    while let Some(_block) = pos.next_block() {
        while let Some(inst) = pos.next_inst() {
            if is_fp_arith(&mut pos, inst) {
                add_nan_canon_seq(&mut pos, inst);
            }
        }
    }
}

/// Returns true/false based on whether the instruction is a floating-point
/// arithmetic operation. This ignores operations like `fneg`, `fabs`, or
/// `fcopysign` that only operate on the sign bit of a floating point value.
fn is_fp_arith(pos: &mut FuncCursor, inst: Inst) -> bool {
    match pos.func.dfg[inst] {
        InstructionData::Unary { opcode, .. } => {
            opcode == Opcode::Ceil
                || opcode == Opcode::Floor
                || opcode == Opcode::Nearest
                || opcode == Opcode::Sqrt
                || opcode == Opcode::Trunc
        }
        InstructionData::Binary { opcode, .. } => {
            opcode == Opcode::Fadd
                || opcode == Opcode::Fdiv
                || opcode == Opcode::Fmax
                || opcode == Opcode::Fmin
                || opcode == Opcode::Fmul
                || opcode == Opcode::Fsub
        }
        InstructionData::Ternary { opcode, .. } => opcode == Opcode::Fma,
        _ => false,
    }
}

/// Append a sequence of canonicalizing instructions after the given instruction.
fn add_nan_canon_seq(pos: &mut FuncCursor, inst: Inst) {
    // Select the instruction result, result type. Replace the instruction
    // result and step forward before inserting the canonicalization sequence.
    let val = pos.func.dfg.first_result(inst);
    let val_type = pos.func.dfg.value_type(val);
    let new_res = pos.func.dfg.replace_result(val, val_type);
    let _next_inst = pos.next_inst().expect("block missing terminator!");

    // Insert a comparison instruction, to check if `inst_res` is NaN. Select
    // the canonical NaN value if `val` is NaN, assign the result to `inst`.
    let is_nan = pos.ins().fcmp(FloatCC::NotEqual, new_res, new_res);

    let scalar_select = |pos: &mut FuncCursor, canon_nan: Value| {
        pos.ins()
            .with_result(val)
            .select(is_nan, canon_nan, new_res);
    };
    let vector_select = |pos: &mut FuncCursor, canon_nan: Value| {
        let cond = pos.ins().raw_bitcast(types::I8X16, is_nan);
        let canon_nan = pos.ins().raw_bitcast(types::I8X16, canon_nan);
        let result = pos.ins().raw_bitcast(types::I8X16, new_res);
        let bitmask = pos.ins().bitselect(cond, canon_nan, result);
        pos.ins().with_result(val).raw_bitcast(val_type, bitmask);
    };

    match val_type {
        types::F32 => {
            let canon_nan = pos.ins().f32const(Ieee32::with_bits(CANON_32BIT_NAN));
            scalar_select(pos, canon_nan);
        }
        types::F64 => {
            let canon_nan = pos.ins().f64const(Ieee64::with_bits(CANON_64BIT_NAN));
            scalar_select(pos, canon_nan);
        }
        types::F32X4 => {
            let canon_nan = pos.ins().iconst(types::I32, i64::from(CANON_32BIT_NAN));
            let canon_nan = pos.ins().splat(types::I32X4, canon_nan);
            vector_select(pos, canon_nan);
        }
        types::F64X2 => {
            let canon_nan = pos.ins().iconst(types::I64, CANON_64BIT_NAN as i64);
            let canon_nan = pos.ins().splat(types::I64X2, canon_nan);
            vector_select(pos, canon_nan);
        }
        _ => {
            // Panic if the type given was not an IEEE floating point type.
            panic!("Could not canonicalize NaN: Unexpected result type found.");
        }
    }

    pos.prev_inst(); // Step backwards so the pass does not skip instructions.
}
