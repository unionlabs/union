//! Miscellaneous helpers for machine backends.

use super::{InsnOutput, LowerCtx, VCodeInst, ValueRegs};
use crate::ir::Type;
use regalloc::{Reg, Writable};
use std::ops::{Add, BitAnd, Not, Sub};

/// Returns the size (in bits) of a given type.
pub fn ty_bits(ty: Type) -> usize {
    usize::from(ty.bits())
}

/// Is the type represented by an integer (not float) at the machine level?
pub(crate) fn ty_has_int_representation(ty: Type) -> bool {
    ty.is_int() || ty.is_bool() || ty.is_ref()
}

/// Is the type represented by a float or vector value at the machine level?
pub(crate) fn ty_has_float_or_vec_representation(ty: Type) -> bool {
    ty.is_vector() || ty.is_float()
}

/// Allocate a register for an instruction output and return it.
pub(crate) fn get_output_reg<I: VCodeInst, C: LowerCtx<I = I>>(
    ctx: &mut C,
    spec: InsnOutput,
) -> ValueRegs<Writable<Reg>> {
    ctx.get_output(spec.insn, spec.output)
}

/// Align a size up to a power-of-two alignment.
pub(crate) fn align_to<N>(x: N, alignment: N) -> N
where
    N: Not<Output = N>
        + BitAnd<N, Output = N>
        + Add<N, Output = N>
        + Sub<N, Output = N>
        + From<u8>
        + Copy,
{
    let alignment_mask = alignment - 1.into();
    (x + alignment_mask) & !alignment_mask
}
