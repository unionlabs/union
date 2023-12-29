use crate::{modular::pow::pow_montgomery_form, PowBoundedExp, Uint};

use super::DynResidue;

impl<const LIMBS: usize> DynResidue<LIMBS> {
    /// Raises to the `exponent` power.
    pub const fn pow(&self, exponent: &Uint<LIMBS>) -> DynResidue<LIMBS> {
        self.pow_bounded_exp(exponent, Uint::<LIMBS>::BITS)
    }

    /// Raises to the `exponent` power,
    /// with `exponent_bits` representing the number of (least significant) bits
    /// to take into account for the exponent.
    ///
    /// NOTE: `exponent_bits` may be leaked in the time pattern.
    pub const fn pow_bounded_exp(&self, exponent: &Uint<LIMBS>, exponent_bits: usize) -> Self {
        Self {
            montgomery_form: pow_montgomery_form(
                &self.montgomery_form,
                exponent,
                exponent_bits,
                &self.residue_params.modulus,
                &self.residue_params.r,
                self.residue_params.mod_neg_inv,
            ),
            residue_params: self.residue_params,
        }
    }
}

impl<const LIMBS: usize> PowBoundedExp<Uint<LIMBS>> for DynResidue<LIMBS> {
    fn pow_bounded_exp(&self, exponent: &Uint<LIMBS>, exponent_bits: usize) -> Self {
        self.pow_bounded_exp(exponent, exponent_bits)
    }
}
