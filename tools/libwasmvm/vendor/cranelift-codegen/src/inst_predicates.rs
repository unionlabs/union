//! Instruction predicates/properties, shared by various analyses.

use crate::ir::{DataFlowGraph, Function, Inst, InstructionData, Opcode};
use crate::machinst::ty_bits;
use cranelift_entity::EntityRef;

/// Preserve instructions with used result values.
pub fn any_inst_results_used(inst: Inst, live: &[bool], dfg: &DataFlowGraph) -> bool {
    dfg.inst_results(inst).iter().any(|v| live[v.index()])
}

/// Test whether the given opcode is unsafe to even consider as side-effect-free.
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

/// Is the given instruction a constant value (`iconst`, `fconst`, `bconst`) that can be
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
        &InstructionData::UnaryBool { imm, .. } => {
            let imm = if imm {
                let bits = ty_bits(func.dfg.value_type(func.dfg.inst_results(inst)[0]));

                if bits < 64 {
                    (1u64 << bits) - 1
                } else {
                    u64::MAX
                }
            } else {
                0
            };

            Some(imm)
        }
        _ => None,
    }
}
