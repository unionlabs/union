//! Helper methods for implementing the `ff` traits.

use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

use crate::PrimeField;

/// Constant-time implementation of Tonelli–Shanks' square-root algorithm for
/// `p mod 16 = 1`.
///
/// `tm1d2` should be set to `(t - 1) // 2`, where `t = (modulus - 1) >> F::S`.
///
/// ## Implementing [`Field::sqrt`]
///
/// This function can be used to implement [`Field::sqrt`] for fields that both implement
/// [`PrimeField`] and satisfy `p mod 16 = 1`.
///
/// [`Field::sqrt`]: crate::Field::sqrt
pub fn sqrt_tonelli_shanks<F: PrimeField, S: AsRef<[u64]>>(f: &F, tm1d2: S) -> CtOption<F> {
    // This is a constant-time version of https://eprint.iacr.org/2012/685.pdf (page 12,
    // algorithm 5). Steps 2-5 of the algorithm are omitted because they are only needed
    // to detect non-square input; it is more efficient to do that by checking at the end
    // whether the square of the result is the input.

    // w = self^((t - 1) // 2)
    let w = f.pow_vartime(tm1d2);

    let mut v = F::S;
    let mut x = w * f;
    let mut b = x * w;

    // Initialize z as the 2^S root of unity.
    let mut z = F::ROOT_OF_UNITY;

    for max_v in (1..=F::S).rev() {
        let mut k = 1;
        let mut b2k = b.square();
        let mut j_less_than_v: Choice = 1.into();

        // This loop has three phases based on the value of k for algorithm 5:
        // - for j <= k, we square b2k in order to calculate b^{2^k}.
        // - for k < j <= v, we square z in order to calculate ω.
        // - for j > v, we do nothing.
        for j in 2..max_v {
            let b2k_is_one = b2k.ct_eq(&F::ONE);
            let squared = F::conditional_select(&b2k, &z, b2k_is_one).square();
            b2k = F::conditional_select(&squared, &b2k, b2k_is_one);
            let new_z = F::conditional_select(&z, &squared, b2k_is_one);
            j_less_than_v &= !j.ct_eq(&v);
            k = u32::conditional_select(&j, &k, b2k_is_one);
            z = F::conditional_select(&z, &new_z, j_less_than_v);
        }

        let result = x * z;
        x = F::conditional_select(&result, &x, b.ct_eq(&F::ONE));
        z = z.square();
        b *= z;
        v = k;
    }

    CtOption::new(
        x,
        (x * x).ct_eq(f), // Only return Some if it's the square root.
    )
}

/// Computes:
///
/// - $(\textsf{true}, \sqrt{\textsf{num}/\textsf{div}})$, if $\textsf{num}$ and
///   $\textsf{div}$ are nonzero and $\textsf{num}/\textsf{div}$ is a square in the
///   field;
/// - $(\textsf{true}, 0)$, if $\textsf{num}$ is zero;
/// - $(\textsf{false}, 0)$, if $\textsf{num}$ is nonzero and $\textsf{div}$ is zero;
/// - $(\textsf{false}, \sqrt{G_S \cdot \textsf{num}/\textsf{div}})$, if
///   $\textsf{num}$ and $\textsf{div}$ are nonzero and $\textsf{num}/\textsf{div}$ is
///   a nonsquare in the field;
///
/// where $G_S$ is a non-square.
///
/// For this method, $G_S$ is currently [`PrimeField::ROOT_OF_UNITY`], a generator of the
/// order $2^S$ subgroup. Users of this crate should not rely on this generator being
/// fixed; it may be changed in future crate versions to simplify the implementation of
/// the SSWU hash-to-curve algorithm.
///
/// The choice of root from sqrt is unspecified.
///
/// ## Implementing [`Field::sqrt_ratio`]
///
/// This function can be used to implement [`Field::sqrt_ratio`] for fields that also
/// implement [`PrimeField`]. If doing so, the default implementation of [`Field::sqrt`]
/// *MUST* be overridden, or else both functions will recurse in a cycle until a stack
/// overflow occurs.
///
/// [`Field::sqrt_ratio`]: crate::Field::sqrt_ratio
/// [`Field::sqrt`]: crate::Field::sqrt
pub fn sqrt_ratio_generic<F: PrimeField>(num: &F, div: &F) -> (Choice, F) {
    // General implementation:
    //
    // a = num * inv0(div)
    //   = {    0    if div is zero
    //     { num/div otherwise
    //
    // b = G_S * a
    //   = {      0      if div is zero
    //     { G_S*num/div otherwise
    //
    // Since G_S is non-square, a and b are either both zero (and both square), or
    // only one of them is square. We can therefore choose the square root to return
    // based on whether a is square, but for the boolean output we need to handle the
    // num != 0 && div == 0 case specifically.

    let a = div.invert().unwrap_or(F::ZERO) * num;
    let b = a * F::ROOT_OF_UNITY;
    let sqrt_a = a.sqrt();
    let sqrt_b = b.sqrt();

    let num_is_zero = num.is_zero();
    let div_is_zero = div.is_zero();
    let is_square = sqrt_a.is_some();
    let is_nonsquare = sqrt_b.is_some();
    assert!(bool::from(
        num_is_zero | div_is_zero | (is_square ^ is_nonsquare)
    ));

    (
        is_square & (num_is_zero | !div_is_zero),
        CtOption::conditional_select(&sqrt_b, &sqrt_a, is_square).unwrap(),
    )
}
