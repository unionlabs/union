//! A place to park MachInst::Inst fragments which are common across multiple architectures.

use super::{LowerCtx, VCodeInst};
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

pub(crate) fn insn_inputs<I: VCodeInst, C: LowerCtx<I = I>>(
    ctx: &C,
    insn: IRInst,
) -> SmallVec<[InsnInput; 4]> {
    (0..ctx.num_inputs(insn))
        .map(|i| InsnInput { insn, input: i })
        .collect()
}

pub(crate) fn insn_outputs<I: VCodeInst, C: LowerCtx<I = I>>(
    ctx: &C,
    insn: IRInst,
) -> SmallVec<[InsnOutput; 4]> {
    (0..ctx.num_outputs(insn))
        .map(|i| InsnOutput { insn, output: i })
        .collect()
}

//============================================================================
// Atomic instructions.

/// Atomic memory update operations.  As of 21 Aug 2020 these are used for the aarch64 and x64
/// targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AtomicRmwOp {
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

impl AtomicRmwOp {
    /// Converts an `ir::AtomicRmwOp` to the corresponding `inst_common::AtomicRmwOp`.
    pub fn from(ir_op: ir::AtomicRmwOp) -> Self {
        match ir_op {
            ir::AtomicRmwOp::Add => AtomicRmwOp::Add,
            ir::AtomicRmwOp::Sub => AtomicRmwOp::Sub,
            ir::AtomicRmwOp::And => AtomicRmwOp::And,
            ir::AtomicRmwOp::Nand => AtomicRmwOp::Nand,
            ir::AtomicRmwOp::Or => AtomicRmwOp::Or,
            ir::AtomicRmwOp::Xor => AtomicRmwOp::Xor,
            ir::AtomicRmwOp::Xchg => AtomicRmwOp::Xchg,
            ir::AtomicRmwOp::Umin => AtomicRmwOp::Umin,
            ir::AtomicRmwOp::Umax => AtomicRmwOp::Umax,
            ir::AtomicRmwOp::Smin => AtomicRmwOp::Smin,
            ir::AtomicRmwOp::Smax => AtomicRmwOp::Smax,
        }
    }
}
