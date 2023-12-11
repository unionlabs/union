//! Legalize instructions.
//!
//! A legal instruction is one that can be mapped directly to a machine code instruction for the
//! target ISA. The `legalize_function()` function takes as input any function and transforms it
//! into an equivalent function using only legal instructions.
//!
//! The characteristics of legal instructions depend on the target ISA, so any given instruction
//! can be legal for one ISA and illegal for another.
//!
//! Besides transforming instructions, the legalizer also fills out the `function.encodings` map
//! which provides a legal encoding recipe for every instruction.
//!
//! The legalizer does not deal with register allocation constraints. These constraints are derived
//! from the encoding recipes, and solved later by the register allocator.

use crate::cursor::{Cursor, FuncCursor};
use crate::flowgraph::ControlFlowGraph;
use crate::ir::immediates::Imm64;
use crate::ir::types::{I128, I64};
use crate::ir::{self, InstBuilder, InstructionData, MemFlags, Value};
use crate::isa::TargetIsa;
use crate::trace;

mod globalvalue;
mod heap;
mod table;

use self::globalvalue::expand_global_value;
use self::heap::{expand_heap_addr, expand_heap_load, expand_heap_store};
use self::table::expand_table_addr;

fn imm_const(pos: &mut FuncCursor, arg: Value, imm: Imm64, is_signed: bool) -> Value {
    let ty = pos.func.dfg.value_type(arg);
    match (ty, is_signed) {
        (I128, true) => {
            let imm = pos.ins().iconst(I64, imm);
            pos.ins().sextend(I128, imm)
        }
        (I128, false) => {
            let imm = pos.ins().iconst(I64, imm);
            pos.ins().uextend(I128, imm)
        }
        _ => pos.ins().iconst(ty.lane_type(), imm),
    }
}

/// Perform a simple legalization by expansion of the function, without
/// platform-specific transforms.
pub fn simple_legalize(func: &mut ir::Function, cfg: &mut ControlFlowGraph, isa: &dyn TargetIsa) {
    trace!("Pre-legalization function:\n{}", func.display());

    let mut pos = FuncCursor::new(func);
    let func_begin = pos.position();
    pos.set_position(func_begin);
    while let Some(_block) = pos.next_block() {
        let mut prev_pos = pos.position();
        while let Some(inst) = pos.next_inst() {
            match pos.func.dfg[inst] {
                // control flow
                InstructionData::CondTrap {
                    opcode:
                        opcode @ (ir::Opcode::Trapnz | ir::Opcode::Trapz | ir::Opcode::ResumableTrapnz),
                    arg,
                    code,
                } => {
                    expand_cond_trap(inst, &mut pos.func, cfg, opcode, arg, code);
                }

                // memory and constants
                InstructionData::UnaryGlobalValue {
                    opcode: ir::Opcode::GlobalValue,
                    global_value,
                } => expand_global_value(inst, &mut pos.func, isa, global_value),
                InstructionData::HeapAddr {
                    opcode: ir::Opcode::HeapAddr,
                    heap,
                    arg,
                    offset,
                    size,
                } => expand_heap_addr(inst, &mut pos.func, cfg, isa, heap, arg, offset, size),
                InstructionData::HeapLoad {
                    opcode: ir::Opcode::HeapLoad,
                    heap_imm,
                    arg,
                } => expand_heap_load(inst, &mut pos.func, cfg, isa, heap_imm, arg),
                InstructionData::HeapStore {
                    opcode: ir::Opcode::HeapStore,
                    heap_imm,
                    args,
                } => expand_heap_store(inst, &mut pos.func, cfg, isa, heap_imm, args[0], args[1]),
                InstructionData::StackLoad {
                    opcode: ir::Opcode::StackLoad,
                    stack_slot,
                    offset,
                } => {
                    let ty = pos.func.dfg.value_type(pos.func.dfg.first_result(inst));
                    let addr_ty = isa.pointer_type();

                    let mut pos = FuncCursor::new(pos.func).at_inst(inst);
                    pos.use_srcloc(inst);

                    let addr = pos.ins().stack_addr(addr_ty, stack_slot, offset);

                    // Stack slots are required to be accessible and aligned.
                    let mflags = MemFlags::trusted();
                    pos.func.dfg.replace(inst).load(ty, mflags, addr, 0);
                }
                InstructionData::StackStore {
                    opcode: ir::Opcode::StackStore,
                    arg,
                    stack_slot,
                    offset,
                } => {
                    let addr_ty = isa.pointer_type();

                    let mut pos = FuncCursor::new(pos.func).at_inst(inst);
                    pos.use_srcloc(inst);

                    let addr = pos.ins().stack_addr(addr_ty, stack_slot, offset);

                    let mut mflags = MemFlags::new();
                    // Stack slots are required to be accessible and aligned.
                    mflags.set_notrap();
                    mflags.set_aligned();
                    pos.func.dfg.replace(inst).store(mflags, arg, addr, 0);
                }
                InstructionData::DynamicStackLoad {
                    opcode: ir::Opcode::DynamicStackLoad,
                    dynamic_stack_slot,
                } => {
                    let ty = pos.func.dfg.value_type(pos.func.dfg.first_result(inst));
                    assert!(ty.is_dynamic_vector());
                    let addr_ty = isa.pointer_type();

                    let mut pos = FuncCursor::new(pos.func).at_inst(inst);
                    pos.use_srcloc(inst);

                    let addr = pos.ins().dynamic_stack_addr(addr_ty, dynamic_stack_slot);

                    // Stack slots are required to be accessible and aligned.
                    let mflags = MemFlags::trusted();
                    pos.func.dfg.replace(inst).load(ty, mflags, addr, 0);
                }
                InstructionData::DynamicStackStore {
                    opcode: ir::Opcode::DynamicStackStore,
                    arg,
                    dynamic_stack_slot,
                } => {
                    pos.use_srcloc(inst);
                    let addr_ty = isa.pointer_type();
                    let vector_ty = pos.func.dfg.value_type(arg);
                    assert!(vector_ty.is_dynamic_vector());

                    let addr = pos.ins().dynamic_stack_addr(addr_ty, dynamic_stack_slot);

                    let mut mflags = MemFlags::new();
                    // Stack slots are required to be accessible and aligned.
                    mflags.set_notrap();
                    mflags.set_aligned();
                    pos.func.dfg.replace(inst).store(mflags, arg, addr, 0);
                }
                InstructionData::TableAddr {
                    opcode: ir::Opcode::TableAddr,
                    table,
                    arg,
                    offset,
                } => expand_table_addr(isa, inst, &mut pos.func, table, arg, offset),

                InstructionData::BinaryImm64 { opcode, arg, imm } => {
                    let is_signed = match opcode {
                        ir::Opcode::IaddImm
                        | ir::Opcode::IrsubImm
                        | ir::Opcode::ImulImm
                        | ir::Opcode::SdivImm
                        | ir::Opcode::SremImm
                        | ir::Opcode::IfcmpImm => true,
                        _ => false,
                    };

                    let imm = imm_const(&mut pos, arg, imm, is_signed);
                    let replace = pos.func.dfg.replace(inst);
                    match opcode {
                        // bitops
                        ir::Opcode::BandImm => {
                            replace.band(arg, imm);
                        }
                        ir::Opcode::BorImm => {
                            replace.bor(arg, imm);
                        }
                        ir::Opcode::BxorImm => {
                            replace.bxor(arg, imm);
                        }
                        // bitshifting
                        ir::Opcode::IshlImm => {
                            replace.ishl(arg, imm);
                        }
                        ir::Opcode::RotlImm => {
                            replace.rotl(arg, imm);
                        }
                        ir::Opcode::RotrImm => {
                            replace.rotr(arg, imm);
                        }
                        ir::Opcode::SshrImm => {
                            replace.sshr(arg, imm);
                        }
                        ir::Opcode::UshrImm => {
                            replace.ushr(arg, imm);
                        }
                        // math
                        ir::Opcode::IaddImm => {
                            replace.iadd(arg, imm);
                        }
                        ir::Opcode::IrsubImm => {
                            // note: arg order reversed
                            replace.isub(imm, arg);
                        }
                        ir::Opcode::ImulImm => {
                            replace.imul(arg, imm);
                        }
                        ir::Opcode::SdivImm => {
                            replace.sdiv(arg, imm);
                        }
                        ir::Opcode::SremImm => {
                            replace.srem(arg, imm);
                        }
                        ir::Opcode::UdivImm => {
                            replace.udiv(arg, imm);
                        }
                        ir::Opcode::UremImm => {
                            replace.urem(arg, imm);
                        }
                        // comparisons
                        ir::Opcode::IfcmpImm => {
                            replace.ifcmp(arg, imm);
                        }
                        _ => prev_pos = pos.position(),
                    };
                }

                // comparisons
                InstructionData::IntCompareImm {
                    opcode: ir::Opcode::IcmpImm,
                    cond,
                    arg,
                    imm,
                } => {
                    let imm = imm_const(&mut pos, arg, imm, true);
                    pos.func.dfg.replace(inst).icmp(cond, arg, imm);
                }

                _ => {
                    prev_pos = pos.position();
                    continue;
                }
            }

            // Legalization implementations require fixpoint loop here.
            // TODO: fix this.
            pos.set_position(prev_pos);
        }
    }

    trace!("Post-legalization function:\n{}", func.display());
}

/// Custom expansion for conditional trap instructions.
fn expand_cond_trap(
    inst: ir::Inst,
    func: &mut ir::Function,
    cfg: &mut ControlFlowGraph,
    opcode: ir::Opcode,
    arg: ir::Value,
    code: ir::TrapCode,
) {
    trace!(
        "expanding conditional trap: {:?}: {}",
        inst,
        func.dfg.display_inst(inst)
    );

    // Parse the instruction.
    let trapz = match opcode {
        ir::Opcode::Trapz => true,
        ir::Opcode::Trapnz | ir::Opcode::ResumableTrapnz => false,
        _ => panic!("Expected cond trap: {}", func.dfg.display_inst(inst)),
    };

    // Split the block after `inst`:
    //
    //     trapnz arg
    //     ..
    //
    // Becomes:
    //
    //     brz arg, new_block_resume
    //     jump new_block_trap
    //
    //   new_block_trap:
    //     trap
    //
    //   new_block_resume:
    //     ..
    let old_block = func.layout.pp_block(inst);
    let new_block_trap = func.dfg.make_block();
    let new_block_resume = func.dfg.make_block();

    // Trapping is a rare event, mark the trapping block as cold.
    func.layout.set_cold(new_block_trap);

    // Replace trap instruction by the inverted condition.
    if trapz {
        func.dfg.replace(inst).brnz(arg, new_block_resume, &[]);
    } else {
        func.dfg.replace(inst).brz(arg, new_block_resume, &[]);
    }

    // Add jump instruction after the inverted branch.
    let mut pos = FuncCursor::new(func).after_inst(inst);
    pos.use_srcloc(inst);
    pos.ins().jump(new_block_trap, &[]);

    // Insert the new label and the unconditional trap terminator.
    pos.insert_block(new_block_trap);

    match opcode {
        ir::Opcode::Trapz | ir::Opcode::Trapnz => {
            pos.ins().trap(code);
        }
        ir::Opcode::ResumableTrapnz => {
            pos.ins().resumable_trap(code);
            pos.ins().jump(new_block_resume, &[]);
        }
        _ => unreachable!(),
    }

    // Insert the new label and resume the execution when the trap fails.
    pos.insert_block(new_block_resume);

    // Finally update the CFG.
    cfg.recompute_block(pos.func, old_block);
    cfg.recompute_block(pos.func, new_block_resume);
    cfg.recompute_block(pos.func, new_block_trap);
}
