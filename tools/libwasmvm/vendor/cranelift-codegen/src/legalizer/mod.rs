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
use crate::ir::types::I32;
use crate::ir::{self, InstBuilder, InstructionData, MemFlags};
use crate::isa::TargetIsa;

mod globalvalue;
mod heap;
mod table;

use self::globalvalue::expand_global_value;
use self::heap::expand_heap_addr;
use self::table::expand_table_addr;

/// Perform a simple legalization by expansion of the function, without
/// platform-specific transforms.
pub fn simple_legalize(func: &mut ir::Function, cfg: &mut ControlFlowGraph, isa: &dyn TargetIsa) {
    let mut pos = FuncCursor::new(func);
    let func_begin = pos.position();
    pos.set_position(func_begin);
    while let Some(_block) = pos.next_block() {
        let mut prev_pos = pos.position();
        while let Some(inst) = pos.next_inst() {
            match pos.func.dfg[inst] {
                // control flow
                InstructionData::BranchIcmp {
                    opcode: ir::Opcode::BrIcmp,
                    cond,
                    destination,
                    ref args,
                } => {
                    let a = args.get(0, &pos.func.dfg.value_lists).unwrap();
                    let b = args.get(1, &pos.func.dfg.value_lists).unwrap();
                    let block_args = args.as_slice(&pos.func.dfg.value_lists)[2..].to_vec();

                    let old_block = pos.func.layout.pp_block(inst);
                    pos.func.dfg.clear_results(inst);

                    let icmp_res = pos.func.dfg.replace(inst).icmp(cond, a, b);
                    let mut pos = FuncCursor::new(pos.func).after_inst(inst);
                    pos.use_srcloc(inst);
                    pos.ins().brnz(icmp_res, destination, &block_args);

                    cfg.recompute_block(pos.func, destination);
                    cfg.recompute_block(pos.func, old_block);
                }
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
                    imm,
                } => expand_heap_addr(inst, &mut pos.func, cfg, isa, heap, arg, imm),
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
                InstructionData::TableAddr {
                    opcode: ir::Opcode::TableAddr,
                    table,
                    arg,
                    offset,
                } => expand_table_addr(inst, &mut pos.func, table, arg, offset),

                // bitops
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::BandImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).band(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::BorImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).bor(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::BxorImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).bxor(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::IaddImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).iadd(arg, imm);
                }

                // bitshifting
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::IshlImm,
                    arg,
                    imm,
                } => {
                    let imm = pos.ins().iconst(I32, imm);
                    pos.func.dfg.replace(inst).ishl(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::RotlImm,
                    arg,
                    imm,
                } => {
                    let imm = pos.ins().iconst(I32, imm);
                    pos.func.dfg.replace(inst).rotl(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::RotrImm,
                    arg,
                    imm,
                } => {
                    let imm = pos.ins().iconst(I32, imm);
                    pos.func.dfg.replace(inst).rotr(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::SshrImm,
                    arg,
                    imm,
                } => {
                    let imm = pos.ins().iconst(I32, imm);
                    pos.func.dfg.replace(inst).sshr(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::UshrImm,
                    arg,
                    imm,
                } => {
                    let imm = pos.ins().iconst(I32, imm);
                    pos.func.dfg.replace(inst).ushr(arg, imm);
                }

                // math
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::IrsubImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).isub(imm, arg); // note: arg order reversed
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::ImulImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).imul(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::SdivImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).sdiv(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::SremImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).srem(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::UdivImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).udiv(arg, imm);
                }
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::UremImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).urem(arg, imm);
                }

                // comparisons
                InstructionData::BinaryImm64 {
                    opcode: ir::Opcode::IfcmpImm,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
                    pos.func.dfg.replace(inst).ifcmp(arg, imm);
                }
                InstructionData::IntCompareImm {
                    opcode: ir::Opcode::IcmpImm,
                    cond,
                    arg,
                    imm,
                } => {
                    let ty = pos.func.dfg.value_type(arg);
                    let imm = pos.ins().iconst(ty, imm);
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
