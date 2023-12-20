//! A place to park MachInst::Inst fragments which are common across multiple architectures.

use super::{Lower, VCodeInst};
use crate::ir::{self, Inst as IRInst};
use smallvec::SmallVec;

//============================================================================
// Instruction input "slots".
//
// We use these types to refer to operand numbers, and result numbers, together
// with the associated instruction, in a type-safe way.

/// Identifier for a particular input of an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InsnInput {
    pub(crate) insn: IRInst,
    pub(crate) input: usize,
}

/// Identifier for a particular output of an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InsnOutput {
    pub(crate) insn: IRInst,
    pub(crate) output: usize,
}

pub(crate) fn insn_outputs<I: VCodeInst>(
    ctx: &Lower<I>,
    insn: IRInst,
) -> SmallVec<[InsnOutput; 4]> {
    (0..ctx.num_outputs(insn))
        .map(|i| InsnOutput { insn, output: i })
        .collect()
}

//============================================================================
// Atomic instructions.

/// Atomic memory update operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MachAtomicRmwOp {
    /// Add
    Add,
    /// Sub
    Sub,
    /// And
    And,
    /// Nand
    Nand,
    /// Or
    Or,
    /// Exclusive Or
    Xor,
    /// Exchange (swap operands)
    Xchg,
    /// Unsigned min
    Umin,
    /// Unsigned max
    Umax,
    /// Signed min
    Smin,
    /// Signed max
    Smax,
}

impl MachAtomicRmwOp {
    /// Converts an `ir::AtomicRmwOp` to the corresponding
    /// `inst_common::AtomicRmwOp`.
    pub fn from(ir_op: ir::AtomicRmwOp) -> Self {
        match ir_op {
            ir::AtomicRmwOp::Add => MachAtomicRmwOp::Add,
            ir::AtomicRmwOp::Sub => MachAtomicRmwOp::Sub,
            ir::AtomicRmwOp::And => MachAtomicRmwOp::And,
            ir::AtomicRmwOp::Nand => MachAtomicRmwOp::Nand,
            ir::AtomicRmwOp::Or => MachAtomicRmwOp::Or,
            ir::AtomicRmwOp::Xor => MachAtomicRmwOp::Xor,
            ir::AtomicRmwOp::Xchg => MachAtomicRmwOp::Xchg,
            ir::AtomicRmwOp::Umin => MachAtomicRmwOp::Umin,
            ir::AtomicRmwOp::Umax => MachAtomicRmwOp::Umax,
            ir::AtomicRmwOp::Smin => MachAtomicRmwOp::Smin,
            ir::AtomicRmwOp::Smax => MachAtomicRmwOp::Smax,
        }
    }
}
