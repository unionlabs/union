//! Legalization of heaps.
//!
//! This module exports the `expand_heap_addr` function which transforms a `heap_addr`
//! instruction into code that depends on the kind of heap referenced.

use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::condcodes::IntCC;
use crate::ir::immediates::Uimm32;
use crate::ir::{self, InstBuilder};
use crate::isa::TargetIsa;

/// Expand a `heap_addr` instruction according to the definition of the heap.
pub fn expand_heap_addr(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
    heap: ir::Heap,
    offset: ir::Value,
    access_size: Uimm32,
) {
    match func.heaps[heap].style {
        ir::HeapStyle::Dynamic { bound_gv } => dynamic_addr(
            isa,
            inst,
            heap,
            offset,
            u64::from(access_size),
            bound_gv,
            func,
        ),
        ir::HeapStyle::Static { bound } => static_addr(
            isa,
            inst,
            heap,
            offset,
            u64::from(access_size),
            bound.into(),
            func,
            cfg,
        ),
    }
}

/// Expand a `heap_addr` for a dynamic heap.
fn dynamic_addr(
    isa: &dyn TargetIsa,
    inst: ir::Inst,
    heap: ir::Heap,
    offset: ir::Value,
    access_size: u64,
    bound_gv: ir::GlobalValue,
    func: &mut ir::Function,
) {
    let offset_ty = func.dfg.value_type(offset);
    let addr_ty = func.dfg.value_type(func.dfg.first_result(inst));
    let min_size = func.heaps[heap].min_size.into();
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let offset = cast_offset_to_pointer_ty(offset, offset_ty, addr_ty, &mut pos);

    // Start with the bounds check. Trap if `offset + access_size > bound`.
    let bound = pos.ins().global_value(addr_ty, bound_gv);
    let (cc, lhs, bound) = if access_size == 1 {
        // `offset > bound - 1` is the same as `offset >= bound`.
        (IntCC::UnsignedGreaterThanOrEqual, offset, bound)
    } else if access_size <= min_size {
        // We know that bound >= min_size, so here we can compare `offset > bound - access_size`
        // without wrapping.
        let adj_bound = pos.ins().iadd_imm(bound, -(access_size as i64));
        (IntCC::UnsignedGreaterThan, offset, adj_bound)
    } else {
        // We need an overflow check for the adjusted offset.
        let access_size_val = pos.ins().iconst(addr_ty, access_size as i64);
        let (adj_offset, overflow) = pos.ins().iadd_ifcout(offset, access_size_val);
        pos.ins().trapif(
            isa.unsigned_add_overflow_condition(),
            overflow,
            ir::TrapCode::HeapOutOfBounds,
        );
        (IntCC::UnsignedGreaterThan, adj_offset, bound)
    };
    let oob = pos.ins().icmp(cc, lhs, bound);
    pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);

    let spectre_oob_comparison = if isa.flags().enable_heap_access_spectre_mitigation() {
        Some((cc, lhs, bound))
    } else {
        None
    };

    compute_addr(
        isa,
        inst,
        heap,
        addr_ty,
        offset,
        pos.func,
        spectre_oob_comparison,
    );
}

/// Expand a `heap_addr` for a static heap.
fn static_addr(
    isa: &dyn TargetIsa,
    inst: ir::Inst,
    heap: ir::Heap,
    mut offset: ir::Value,
    access_size: u64,
    bound: u64,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
) {
    let offset_ty = func.dfg.value_type(offset);
    let addr_ty = func.dfg.value_type(func.dfg.first_result(inst));
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // The goal here is to trap if `offset + access_size > bound`.
    //
    // This first case is a trivial case where we can easily trap.
    if access_size > bound {
        // This will simply always trap since `offset >= 0`.
        pos.ins().trap(ir::TrapCode::HeapOutOfBounds);
        pos.func.dfg.replace(inst).iconst(addr_ty, 0);

        // Split Block, as the trap is a terminator instruction.
        let curr_block = pos.current_block().expect("Cursor is not in a block");
        let new_block = pos.func.dfg.make_block();
        pos.insert_block(new_block);
        cfg.recompute_block(pos.func, curr_block);
        cfg.recompute_block(pos.func, new_block);
        return;
    }

    // After the trivial case is done we're now mostly interested in trapping
    // if `offset > bound - access_size`. We know `bound - access_size` here is
    // non-negative from the above comparison.
    //
    // If we can know `bound - access_size >= 4GB` then with a 32-bit offset
    // we're guaranteed:
    //
    //      bound - access_size >= 4GB > offset
    //
    // or, in other words, `offset < bound - access_size`, meaning we can't trap
    // for any value of `offset`.
    //
    // With that we have an optimization here where with 32-bit offsets and
    // `bound - access_size >= 4GB` we can omit a bounds check.
    let limit = bound - access_size;
    let mut spectre_oob_comparison = None;
    offset = cast_offset_to_pointer_ty(offset, offset_ty, addr_ty, &mut pos);
    if offset_ty != ir::types::I32 || limit < 0xffff_ffff {
        // Here we want to test the condition `offset > limit` and if that's
        // true then this is an out-of-bounds access and needs to trap. For ARM
        // and other RISC architectures it's easier to test against an immediate
        // that's even instead of odd, so if `limit` is odd then we instead test
        // for `offset >= limit + 1`.
        //
        // The thinking behind this is that:
        //
        //      A >= B + 1  =>  A - 1 >= B  =>  A > B
        //
        // where the last step here is true because A/B are integers, which
        // should mean that `A >= B + 1` is an equivalent check for `A > B`
        let (cc, lhs, limit_imm) = if limit & 1 == 1 {
            let limit = limit as i64 + 1;
            (IntCC::UnsignedGreaterThanOrEqual, offset, limit)
        } else {
            let limit = limit as i64;
            (IntCC::UnsignedGreaterThan, offset, limit)
        };
        let oob = pos.ins().icmp_imm(cc, lhs, limit_imm);
        pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);
        if isa.flags().enable_heap_access_spectre_mitigation() {
            let limit = pos.ins().iconst(addr_ty, limit_imm);
            spectre_oob_comparison = Some((cc, lhs, limit));
        }
    }

    compute_addr(
        isa,
        inst,
        heap,
        addr_ty,
        offset,
        pos.func,
        spectre_oob_comparison,
    );
}

fn cast_offset_to_pointer_ty(
    offset: ir::Value,
    offset_ty: ir::Type,
    addr_ty: ir::Type,
    pos: &mut FuncCursor,
) -> ir::Value {
    if offset_ty == addr_ty {
        return offset;
    }
    // Note that using 64-bit heaps on a 32-bit host is not currently supported,
    // would require at least a bounds check here to ensure that the truncation
    // from 64-to-32 bits doesn't lose any upper bits. For now though we're
    // mostly interested in the 32-bit-heaps-on-64-bit-hosts cast.
    assert!(offset_ty.bits() < addr_ty.bits());

    // Convert `offset` to `addr_ty`.
    let extended_offset = pos.ins().uextend(addr_ty, offset);

    // Add debug value-label alias so that debuginfo can name the extended
    // value as the address
    let loc = pos.srcloc();
    pos.func
        .dfg
        .add_value_label_alias(extended_offset, loc, offset);

    extended_offset
}

/// Emit code for the base address computation of a `heap_addr` instruction.
fn compute_addr(
    isa: &dyn TargetIsa,
    inst: ir::Inst,
    heap: ir::Heap,
    addr_ty: ir::Type,
    offset: ir::Value,
    func: &mut ir::Function,
    // If we are performing Spectre mitigation with conditional selects, the
    // values to compare and the condition code that indicates an out-of bounds
    // condition; on this condition, the conditional move will choose a
    // speculatively safe address (a zero / null pointer) instead.
    spectre_oob_comparison: Option<(IntCC, ir::Value, ir::Value)>,
) {
    debug_assert_eq!(func.dfg.value_type(offset), addr_ty);
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    // Add the heap base address base
    let base = if isa.flags().enable_pinned_reg() && isa.flags().use_pinned_reg_as_heap_base() {
        pos.ins().get_pinned_reg(isa.pointer_type())
    } else {
        let base_gv = pos.func.heaps[heap].base;
        pos.ins().global_value(addr_ty, base_gv)
    };

    if let Some((cc, a, b)) = spectre_oob_comparison {
        let final_addr = pos.ins().iadd(base, offset);
        let zero = pos.ins().iconst(addr_ty, 0);
        let flags = pos.ins().ifcmp(a, b);
        pos.func
            .dfg
            .replace(inst)
            .selectif_spectre_guard(addr_ty, cc, flags, zero, final_addr);
    } else {
        pos.func.dfg.replace(inst).iadd(base, offset);
    }
}
