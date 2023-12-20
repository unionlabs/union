//! Legalization of heaps.
//!
//! This module exports the `expand_heap_addr` function which transforms a `heap_addr`
//! instruction into code that depends on the kind of heap referenced.

use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::condcodes::IntCC;
use crate::ir::immediates::{HeapImmData, Offset32, Uimm32, Uimm8};
use crate::ir::{self, InstBuilder, RelSourceLoc};
use crate::isa::TargetIsa;
use crate::trace;

/// Expand a `heap_load` instruction according to the definition of the heap.
pub fn expand_heap_load(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
    heap_imm: ir::HeapImm,
    index: ir::Value,
) {
    let HeapImmData {
        flags,
        heap,
        offset,
    } = func.dfg.heap_imms[heap_imm];

    let result_ty = func.dfg.ctrl_typevar(inst);
    let access_size = result_ty.bytes();
    let access_size = u8::try_from(access_size).unwrap();

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let addr =
        bounds_check_and_compute_addr(&mut pos, cfg, isa, heap, index, offset.into(), access_size);

    pos.func
        .dfg
        .replace(inst)
        .load(result_ty, flags, addr, Offset32::new(0));
}

/// Expand a `heap_store` instruction according to the definition of the heap.
pub fn expand_heap_store(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
    heap_imm: ir::HeapImm,
    index: ir::Value,
    value: ir::Value,
) {
    let HeapImmData {
        flags,
        heap,
        offset,
    } = func.dfg.heap_imms[heap_imm];

    let store_ty = func.dfg.value_type(value);
    let access_size = u8::try_from(store_ty.bytes()).unwrap();

    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let addr =
        bounds_check_and_compute_addr(&mut pos, cfg, isa, heap, index, offset.into(), access_size);

    pos.func
        .dfg
        .replace(inst)
        .store(flags, value, addr, Offset32::new(0));
}

/// Expand a `heap_addr` instruction according to the definition of the heap.
pub fn expand_heap_addr(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
    heap: ir::Heap,
    index: ir::Value,
    offset: Uimm32,
    access_size: Uimm8,
) {
    let mut pos = FuncCursor::new(func).at_inst(inst);
    pos.use_srcloc(inst);

    let addr =
        bounds_check_and_compute_addr(&mut pos, cfg, isa, heap, index, offset.into(), access_size);

    // Replace the `heap_addr` and its result value with the legalized native
    // address.
    let addr_inst = pos.func.dfg.value_def(addr).unwrap_inst();
    pos.func.dfg.replace_with_aliases(inst, addr_inst);
    pos.func.layout.remove_inst(inst);
}

/// Helper used to emit bounds checks (as necessary) and compute the native
/// address of a heap access.
///
/// Returns the `ir::Value` holding the native address of the heap access.
fn bounds_check_and_compute_addr(
    pos: &mut FuncCursor,
    cfg: &mut ControlFlowGraph,
    isa: &dyn TargetIsa,
    heap: ir::Heap,
    // Dynamic operand indexing into the heap.
    index: ir::Value,
    // Static immediate added to the index.
    offset: u32,
    // Static size of the heap access.
    access_size: u8,
) -> ir::Value {
    let pointer_type = isa.pointer_type();
    let spectre = isa.flags().enable_heap_access_spectre_mitigation();
    let offset_and_size = offset_plus_size(offset, access_size);

    let ir::HeapData {
        base: _,
        min_size,
        offset_guard_size: guard_size,
        style,
        index_type,
    } = pos.func.heaps[heap].clone();

    let index = cast_index_to_pointer_ty(index, index_type, pointer_type, pos);

    // We need to emit code that will trap (or compute an address that will trap
    // when accessed) if
    //
    //     index + offset + access_size > bound
    //
    // or if the `index + offset + access_size` addition overflows.
    //
    // Note that we ultimately want a 64-bit integer (we only target 64-bit
    // architectures at the moment) and that `offset` is a `u32` and
    // `access_size` is a `u8`. This means that we can add the latter together
    // as `u64`s without fear of overflow, and we only have to be concerned with
    // whether adding in `index` will overflow.
    //
    // Finally, the following right-hand sides of the matches do have a little
    // bit of duplicated code across them, but I think writing it this way is
    // worth it for readability and seeing very clearly each of our cases for
    // different bounds checks and optimizations of those bounds checks. It is
    // intentionally written in a straightforward case-matching style that will
    // hopefully make it easy to port to ISLE one day.
    match style {
        // ====== Dynamic Memories ======
        //
        // 1. First special case for when `offset + access_size == 1`:
        //
        //            index + 1 > bound
        //        ==> index >= bound
        //
        //    1.a. When Spectre mitigations are enabled, avoid duplicating
        //         bounds checks between the mitigations and the regular bounds
        //         checks.
        ir::HeapStyle::Dynamic { bound_gv } if offset_and_size == 1 && spectre => {
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            compute_addr(
                isa,
                pos,
                heap,
                pointer_type,
                index,
                offset,
                Some(SpectreOobComparison {
                    cc: IntCC::UnsignedGreaterThanOrEqual,
                    lhs: index,
                    rhs: bound,
                }),
            )
        }
        //    1.b. Emit explicit `index >= bound` bounds checks.
        ir::HeapStyle::Dynamic { bound_gv } if offset_and_size == 1 => {
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            let oob = pos
                .ins()
                .icmp(IntCC::UnsignedGreaterThanOrEqual, index, bound);
            pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);
            compute_addr(isa, pos, heap, pointer_type, index, offset, None)
        }

        // 2. Second special case for when `offset + access_size <= min_size`.
        //
        //    We know that `bound >= min_size`, so we can do the following
        //    comparison, without fear of the right-hand side wrapping around:
        //
        //            index + offset + access_size > bound
        //        ==> index > bound - (offset + access_size)
        //
        //    2.a. Dedupe bounds checks with Spectre mitigations.
        ir::HeapStyle::Dynamic { bound_gv } if offset_and_size <= min_size.into() && spectre => {
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            let adjusted_bound = pos.ins().iadd_imm(bound, -(offset_and_size as i64));
            compute_addr(
                isa,
                pos,
                heap,
                pointer_type,
                index,
                offset,
                Some(SpectreOobComparison {
                    cc: IntCC::UnsignedGreaterThan,
                    lhs: index,
                    rhs: adjusted_bound,
                }),
            )
        }
        //    2.b. Emit explicit `index > bound - (offset + access_size)` bounds
        //         checks.
        ir::HeapStyle::Dynamic { bound_gv } if offset_and_size <= min_size.into() => {
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            let adjusted_bound = pos.ins().iadd_imm(bound, -(offset_and_size as i64));
            let oob = pos
                .ins()
                .icmp(IntCC::UnsignedGreaterThan, index, adjusted_bound);
            pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);
            compute_addr(isa, pos, heap, pointer_type, index, offset, None)
        }

        // 3. General case for dynamic memories:
        //
        //        index + offset + access_size > bound
        //
        //    And we have to handle the overflow case in the left-hand side.
        //
        //    3.a. Dedupe bounds checks with Spectre mitigations.
        ir::HeapStyle::Dynamic { bound_gv } if spectre => {
            let access_size_val = pos.ins().iconst(pointer_type, offset_and_size as i64);
            let adjusted_index =
                pos.ins()
                    .uadd_overflow_trap(index, access_size_val, ir::TrapCode::HeapOutOfBounds);
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            compute_addr(
                isa,
                pos,
                heap,
                pointer_type,
                index,
                offset,
                Some(SpectreOobComparison {
                    cc: IntCC::UnsignedGreaterThan,
                    lhs: adjusted_index,
                    rhs: bound,
                }),
            )
        }
        //    3.b. Emit an explicit `index + offset + access_size > bound`
        //         check.
        ir::HeapStyle::Dynamic { bound_gv } => {
            let access_size_val = pos.ins().iconst(pointer_type, offset_and_size as i64);
            let adjusted_index =
                pos.ins()
                    .uadd_overflow_trap(index, access_size_val, ir::TrapCode::HeapOutOfBounds);
            let bound = pos.ins().global_value(pointer_type, bound_gv);
            let oob = pos
                .ins()
                .icmp(IntCC::UnsignedGreaterThan, adjusted_index, bound);
            pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);
            compute_addr(isa, pos, heap, pointer_type, index, offset, None)
        }

        // ====== Static Memories ======
        //
        // With static memories we know the size of the heap bound at compile
        // time.
        //
        // 1. First special case: trap immediately if `offset + access_size >
        //    bound`, since we will end up being out-of-bounds regardless of the
        //    given `index`.
        ir::HeapStyle::Static { bound } if offset_and_size > bound.into() => {
            pos.ins().trap(ir::TrapCode::HeapOutOfBounds);

            // Split the block, as the trap is a terminator instruction.
            let curr_block = pos.current_block().expect("Cursor is not in a block");
            let new_block = pos.func.dfg.make_block();
            pos.insert_block(new_block);
            cfg.recompute_block(pos.func, curr_block);
            cfg.recompute_block(pos.func, new_block);

            let null = pos.ins().iconst(pointer_type, 0);
            return null;
        }

        // 2. Second special case for when we can completely omit explicit
        //    bounds checks for 32-bit static memories.
        //
        //    First, let's rewrite our comparison to move all of the constants
        //    to one side:
        //
        //            index + offset + access_size > bound
        //        ==> index > bound - (offset + access_size)
        //
        //    We know the subtraction on the right-hand side won't wrap because
        //    we didn't hit the first special case.
        //
        //    Additionally, we add our guard pages (if any) to the right-hand
        //    side, since we can rely on the virtual memory subsystem at runtime
        //    to catch out-of-bound accesses within the range `bound .. bound +
        //    guard_size`. So now we are dealing with
        //
        //        index > bound + guard_size - (offset + access_size)
        //
        //    Note that `bound + guard_size` cannot overflow for
        //    correctly-configured heaps, as otherwise the heap wouldn't fit in
        //    a 64-bit memory space.
        //
        //    The complement of our should-this-trap comparison expression is
        //    the should-this-not-trap comparison expression:
        //
        //        index <= bound + guard_size - (offset + access_size)
        //
        //    If we know the right-hand side is greater than or equal to
        //    `u32::MAX`, then
        //
        //        index <= u32::MAX <= bound + guard_size - (offset + access_size)
        //
        //    This expression is always true when the heap is indexed with
        //    32-bit integers because `index` cannot be larger than
        //    `u32::MAX`. This means that `index` is always either in bounds or
        //    within the guard page region, neither of which require emitting an
        //    explicit bounds check.
        ir::HeapStyle::Static { bound }
            if index_type == ir::types::I32
                && u64::from(u32::MAX)
                    <= u64::from(bound) + u64::from(guard_size) - offset_and_size =>
        {
            compute_addr(isa, pos, heap, pointer_type, index, offset, None)
        }

        // 3. General case for static memories.
        //
        //    We have to explicitly test whether
        //
        //        index > bound - (offset + access_size)
        //
        //    and trap if so.
        //
        //    Since we have to emit explicit bounds checks, we might as well be
        //    precise, not rely on the virtual memory subsystem at all, and not
        //    factor in the guard pages here.
        //
        //    3.a. Dedupe the Spectre mitigation and the explicit bounds check.
        ir::HeapStyle::Static { bound } if spectre => {
            // NB: this subtraction cannot wrap because we didn't hit the first
            // special case.
            let adjusted_bound = u64::from(bound) - offset_and_size;
            let adjusted_bound = pos.ins().iconst(pointer_type, adjusted_bound as i64);
            compute_addr(
                isa,
                pos,
                heap,
                pointer_type,
                index,
                offset,
                Some(SpectreOobComparison {
                    cc: IntCC::UnsignedGreaterThan,
                    lhs: index,
                    rhs: adjusted_bound,
                }),
            )
        }
        //    3.b. Emit the explicit `index > bound - (offset + access_size)`
        //         check.
        ir::HeapStyle::Static { bound } => {
            // See comment in 3.a. above.
            let adjusted_bound = u64::from(bound) - offset_and_size;
            let oob = pos
                .ins()
                .icmp_imm(IntCC::UnsignedGreaterThan, index, adjusted_bound as i64);
            pos.ins().trapnz(oob, ir::TrapCode::HeapOutOfBounds);
            compute_addr(isa, pos, heap, pointer_type, index, offset, None)
        }
    }
}

fn cast_index_to_pointer_ty(
    index: ir::Value,
    index_ty: ir::Type,
    pointer_ty: ir::Type,
    pos: &mut FuncCursor,
) -> ir::Value {
    if index_ty == pointer_ty {
        return index;
    }
    // Note that using 64-bit heaps on a 32-bit host is not currently supported,
    // would require at least a bounds check here to ensure that the truncation
    // from 64-to-32 bits doesn't lose any upper bits. For now though we're
    // mostly interested in the 32-bit-heaps-on-64-bit-hosts cast.
    assert!(index_ty.bits() < pointer_ty.bits());

    // Convert `index` to `addr_ty`.
    let extended_index = pos.ins().uextend(pointer_ty, index);

    // Add debug value-label alias so that debuginfo can name the extended
    // value as the address
    let loc = pos.srcloc();
    let loc = RelSourceLoc::from_base_offset(pos.func.params.base_srcloc(), loc);
    pos.func
        .stencil
        .dfg
        .add_value_label_alias(extended_index, loc, index);

    extended_index
}

struct SpectreOobComparison {
    cc: IntCC,
    lhs: ir::Value,
    rhs: ir::Value,
}

/// Emit code for the base address computation of a `heap_addr` instruction,
/// without any bounds checks (other than optional Spectre mitigations).
fn compute_addr(
    isa: &dyn TargetIsa,
    pos: &mut FuncCursor,
    heap: ir::Heap,
    addr_ty: ir::Type,
    index: ir::Value,
    offset: u32,
    // If we are performing Spectre mitigation with conditional selects, the
    // values to compare and the condition code that indicates an out-of bounds
    // condition; on this condition, the conditional move will choose a
    // speculatively safe address (a zero / null pointer) instead.
    spectre_oob_comparison: Option<SpectreOobComparison>,
) -> ir::Value {
    debug_assert_eq!(pos.func.dfg.value_type(index), addr_ty);

    // Add the heap base address base
    let base = if isa.flags().enable_pinned_reg() && isa.flags().use_pinned_reg_as_heap_base() {
        let base = pos.ins().get_pinned_reg(isa.pointer_type());
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(base));
        base
    } else {
        let base_gv = pos.func.heaps[heap].base;
        let base = pos.ins().global_value(addr_ty, base_gv);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(base));
        base
    };

    if let Some(SpectreOobComparison { cc, lhs, rhs }) = spectre_oob_comparison {
        let final_base = pos.ins().iadd(base, index);
        // NB: The addition of the offset immediate must happen *before* the
        // `select_spectre_guard`. If it happens after, then we potentially are
        // letting speculative execution read the whole first 4GiB of memory.
        let final_addr = if offset == 0 {
            final_base
        } else {
            let final_addr = pos.ins().iadd_imm(final_base, offset as i64);
            trace!(
                "  inserting: {}",
                pos.func.dfg.display_value_inst(final_addr)
            );
            final_addr
        };
        let zero = pos.ins().iconst(addr_ty, 0);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(zero));

        let cmp = pos.ins().icmp(cc, lhs, rhs);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(cmp));

        let value = pos.ins().select_spectre_guard(cmp, zero, final_addr);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(value));
        value
    } else if offset == 0 {
        let addr = pos.ins().iadd(base, index);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(addr));
        addr
    } else {
        let final_base = pos.ins().iadd(base, index);
        trace!(
            "  inserting: {}",
            pos.func.dfg.display_value_inst(final_base)
        );
        let addr = pos.ins().iadd_imm(final_base, offset as i64);
        trace!("  inserting: {}", pos.func.dfg.display_value_inst(addr));
        addr
    }
}

#[inline]
fn offset_plus_size(offset: u32, size: u8) -> u64 {
    // Cannot overflow because we are widening to `u64`.
    offset as u64 + size as u64
}
