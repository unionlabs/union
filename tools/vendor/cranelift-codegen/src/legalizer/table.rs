//! Legalization of tables.
//!
//! This module exports the `expand_table_addr` function which transforms a `table_addr`
//! instruction into code that depends on the kind of table referenced.

use crate::cursor::{Cursor, FuncCursor};
use crate::ir::condcodes::IntCC;
use crate::ir::immediates::Offset32;
use crate::ir::{self, InstBuilder};

/// Expand a `table_addr` instruction according to the definition of the table.
pub fn expand_table_addr(
    inst: ir::Inst,
    func: &mut ir::Function,
    table: ir::Table,
    index: ir::Value,
    element_offset: Offset32,
) {
    let bound_gv = func.tables[table].bound_gv;
    let index_ty = func.dfg.value_type(index);
    let addr_ty = func.dfg.value_type(func.dfg.first_result(inst));
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Start with the bounds check. Trap if `index + 1 > bound`.
    let bound = pos.ins().global_value(index_ty, bound_gv);

    // `index > bound - 1` is the same as `index >= bound`.
    let oob = pos
        .ins()
        .icmp(IntCC::UnsignedGreaterThanOrEqual, index, bound);
    pos.ins().trapnz(oob, ir::TrapCode::TableOutOfBounds);

    compute_addr(
        inst,
        table,
        addr_ty,
        index,
        index_ty,
        element_offset,
        pos.func,
    );
}

/// Emit code for the base address computation of a `table_addr` instruction.
fn compute_addr(
    inst: ir::Inst,
    table: ir::Table,
    addr_ty: ir::Type,
    mut index: ir::Value,
    index_ty: ir::Type,
    element_offset: Offset32,
    func: &mut ir::Function,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Convert `index` to `addr_ty`.
    if index_ty != addr_ty {
        index = pos.ins().uextend(addr_ty, index);
    }

    // Add the table base address base
    let base_gv = pos.func.tables[table].base_gv;
    let base = pos.ins().global_value(addr_ty, base_gv);

    let element_size = pos.func.tables[table].element_size;
    let mut offset;
    let element_size: u64 = element_size.into();
    if element_size == 1 {
        offset = index;
    } else if element_size.is_power_of_two() {
        offset = pos
            .ins()
            .ishl_imm(index, i64::from(element_size.trailing_zeros()));
    } else {
        offset = pos.ins().imul_imm(index, element_size as i64);
    }

    if element_offset == Offset32::new(0) {
        pos.func.dfg.replace(inst).iadd(base, offset);
    } else {
        let imm: i64 = element_offset.into();
        offset = pos.ins().iadd(base, offset);
        pos.func.dfg.replace(inst).iadd_imm(offset, imm);
    }
}
