//! Instruction predicates/properties, shared by various analyses.
use crate::ir::immediates::Offset32;
use crate::ir::instructions::BranchInfo;
use crate::ir::{Block, DataFlowGraph, Function, Inst, InstructionData, Opcode, Type, Value};
use cranelift_entity::EntityRef;

/// Preserve instructions with used result values.
pub fn any_inst_results_used(inst: Inst, live: &[bool], dfg: &DataFlowGraph) -> bool {
    dfg.inst_results(inst).iter().any(|v| live[v.index()])
}

/// Test whether the given opcode is unsafe to even consider as side-effect-free.
#[inline(always)]
fn trivially_has_side_effects(opcode: Opcode) -> bool {
    opcode.is_call()
        || opcode.is_branch()
        || opcode.is_terminator()
        || opcode.is_return()
        || opcode.can_trap()
        || opcode.other_side_effects()
        || opcode.can_store()
}

/// Load instructions without the `notrap` flag are defined to trap when
/// operating on inaccessible memory, so we can't treat them as side-effect-free even if the loaded
/// value is unused.
#[inline(always)]
fn is_load_with_defined_trapping(opcode: Opcode, data: &InstructionData) -> bool {
    if !opcode.can_load() {
        return false;
    }
    match *data {
        InstructionData::StackLoad { .. } => false,
        InstructionData::Load { flags, .. } => !flags.notrap(),
        _ => true,
    }
}

/// Does the given instruction have any side-effect that would preclude it from being removed when
/// its value is unused?
#[inline(always)]
pub fn has_side_effect(func: &Function, inst: Inst) -> bool {
    let data = &func.dfg[inst];
    let opcode = data.opcode();
    trivially_has_side_effects(opcode) || is_load_with_defined_trapping(opcode, data)
}

/// Does the given instruction have any side-effect as per [has_side_effect], or else is a load,
/// but not the get_pinned_reg opcode?
pub fn has_lowering_side_effect(func: &Function, inst: Inst) -> bool {
    let op = func.dfg[inst].opcode();
    op != Opcode::GetPinnedReg && (has_side_effect(func, inst) || op.can_load())
}

/// Is the given instruction a constant value (`iconst`, `fconst`) that can be
/// represented in 64 bits?
pub fn is_constant_64bit(func: &Function, inst: Inst) -> Option<u64> {
    let data = &func.dfg[inst];
    if data.opcode() == Opcode::Null {
        return Some(0);
    }
    match data {
        &InstructionData::UnaryImm { imm, .. } => Some(imm.bits() as u64),
        &InstructionData::UnaryIeee32 { imm, .. } => Some(imm.bits() as u64),
        &InstructionData::UnaryIeee64 { imm, .. } => Some(imm.bits()),
        _ => None,
    }
}

/// Get the address, offset, and access type from the given instruction, if any.
pub fn inst_addr_offset_type(func: &Function, inst: Inst) -> Option<(Value, Offset32, Type)> {
    let data = &func.dfg[inst];
    match data {
        InstructionData::Load { arg, offset, .. } => {
            let ty = func.dfg.value_type(func.dfg.inst_results(inst)[0]);
            Some((*arg, *offset, ty))
        }
        InstructionData::LoadNoOffset { arg, .. } => {
            let ty = func.dfg.value_type(func.dfg.inst_results(inst)[0]);
            Some((*arg, 0.into(), ty))
        }
        InstructionData::Store { args, offset, .. } => {
            let ty = func.dfg.value_type(args[0]);
            Some((args[1], *offset, ty))
        }
        InstructionData::StoreNoOffset { args, .. } => {
            let ty = func.dfg.value_type(args[0]);
            Some((args[1], 0.into(), ty))
        }
        _ => None,
    }
}

/// Get the store data, if any, from an instruction.
pub fn inst_store_data(func: &Function, inst: Inst) -> Option<Value> {
    let data = &func.dfg[inst];
    match data {
        InstructionData::Store { args, .. } | InstructionData::StoreNoOffset { args, .. } => {
            Some(args[0])
        }
        _ => None,
    }
}

/// Determine whether this opcode behaves as a memory fence, i.e.,
/// prohibits any moving of memory accesses across it.
pub fn has_memory_fence_semantics(op: Opcode) -> bool {
    match op {
        Opcode::AtomicRmw
        | Opcode::AtomicCas
        | Opcode::AtomicLoad
        | Opcode::AtomicStore
        | Opcode::Fence
        | Opcode::Debugtrap => true,
        Opcode::Call | Opcode::CallIndirect => true,
        op if op.can_trap() => true,
        _ => false,
    }
}

/// Visit all successors of a block with a given visitor closure. The closure
/// arguments are the branch instruction that is used to reach the successor,
/// the successor block itself, and a flag indicating whether the block is
/// branched to via a table entry.
pub(crate) fn visit_block_succs<F: FnMut(Inst, Block, bool)>(
    f: &Function,
    block: Block,
    mut visit: F,
) {
    for inst in f.layout.block_likely_branches(block) {
        if f.dfg[inst].opcode().is_branch() {
            visit_branch_targets(f, inst, &mut visit);
        }
    }
}

fn visit_branch_targets<F: FnMut(Inst, Block, bool)>(f: &Function, inst: Inst, visit: &mut F) {
    match f.dfg[inst].analyze_branch(&f.dfg.value_lists) {
        BranchInfo::NotABranch => {}
        BranchInfo::SingleDest(dest, _) => {
            visit(inst, dest, false);
        }
        BranchInfo::Table(table, maybe_dest) => {
            if let Some(dest) = maybe_dest {
                // The default block is reached via a direct conditional branch,
                // so it is not part of the table.
                visit(inst, dest, false);
            }
            for &dest in f.jump_tables[table].as_slice() {
                visit(inst, dest, true);
            }
        }
    }
}
