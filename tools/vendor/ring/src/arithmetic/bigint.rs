// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Multi-precision integers.
//!
//! # Modular Arithmetic.
//!
//! Modular arithmetic is done in finite commutative rings ℤ/mℤ for some
//! modulus *m*. We work in finite commutative rings instead of finite fields
//! because the RSA public modulus *n* is not prime, which means ℤ/nℤ contains
//! nonzero elements that have no multiplicative inverse, so ℤ/nℤ is not a
//! finite field.
//!
//! In some calculations we need to deal with multiple rings at once. For
//! example, RSA private key operations operate in the rings ℤ/nℤ, ℤ/pℤ, and
//! ℤ/qℤ. Types and functions dealing with such rings are all parameterized
//! over a type `M` to ensure that we don't wrongly mix up the math, e.g. by
//! multiplying an element of ℤ/pℤ by an element of ℤ/qℤ modulo q. This follows
//! the "unit" pattern described in [Static checking of units in Servo].
//!
//! `Elem` also uses the static unit checking pattern to statically track the
//! Montgomery factors that need to be canceled out in each value using it's
//! `E` parameter.
//!
//! [Static checking of units in Servo]:
//!     https://blog.mozilla.org/research/2014/06/23/static-checking-of-units-in-servo/

use crate::{
    arithmetic::montgomery::*,
    bits, bssl, c, error,
    limb::{self, Limb, LimbMask, LIMB_BITS, LIMB_BYTES},
};
use alloc::{borrow::ToOwned as _, boxed::Box, vec, vec::Vec};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub unsafe trait Prime {}

struct Width<M> {
    num_limbs: usize,

    /// The modulus *m* that the width originated from.
    m: PhantomData<M>,
}

/// All `BoxedLimbs<M>` are stored in the same number of limbs.
struct BoxedLimbs<M> {
    limbs: Box<[Limb]>,

    /// The modulus *m* that determines the size of `limbx`.
    m: PhantomData<M>,
}

impl<M> Deref for BoxedLimbs<M> {
    type Target = [Limb];
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.limbs
    }
}

impl<M> DerefMut for BoxedLimbs<M> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.limbs
    }
}

// TODO: `derive(Clone)` after https://github.com/rust-lang/rust/issues/26925
// is resolved or restrict `M: Clone`.
impl<M> Clone for BoxedLimbs<M> {
    fn clone(&self) -> Self {
        Self {
            limbs: self.limbs.clone(),
            m: self.m,
        }
    }
}

impl<M> BoxedLimbs<M> {
    fn positive_minimal_width_from_be_bytes(
        input: untrusted::Input,
    ) -> Result<Self, error::KeyRejected> {
        // Reject leading zeros. Also reject the value zero ([0]) because zero
        // isn't positive.
        if untrusted::Reader::new(input).peek(0) {
            return Err(error::KeyRejected::invalid_encoding());
        }
        let num_limbs = (input.len() + LIMB_BYTES - 1) / LIMB_BYTES;
        let mut r = Self::zero(Width {
            num_limbs,
            m: PhantomData,
        });
        limb::parse_big_endian_and_pad_consttime(input, &mut r)
            .map_err(|error::Unspecified| error::KeyRejected::unexpected_error())?;
        Ok(r)
    }

    fn minimal_width_from_unpadded(limbs: &[Limb]) -> Self {
        debug_assert_ne!(limbs.last(), Some(&0));
        Self {
            limbs: limbs.to_owned().into_boxed_slice(),
            m: PhantomData,
        }
    }

    fn from_be_bytes_padded_less_than(
        input: untrusted::Input,
        m: &Modulus<M>,
    ) -> Result<Self, error::Unspecified> {
        let mut r = Self::zero(m.width());
        limb::parse_big_endian_and_pad_consttime(input, &mut r)?;
        if limb::limbs_less_than_limbs_consttime(&r, &m.limbs) != LimbMask::True {
            return Err(error::Unspecified);
        }
        Ok(r)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        limb::limbs_are_zero_constant_time(&self.limbs) == LimbMask::True
    }

    fn zero(width: Width<M>) -> Self {
        Self {
            limbs: vec![0; width.num_limbs].into_boxed_slice(),
            m: PhantomData,
        }
    }

    fn width(&self) -> Width<M> {
        Width {
            num_limbs: self.limbs.len(),
            m: PhantomData,
        }
    }
}

/// A modulus *s* that is smaller than another modulus *l* so every element of
/// ℤ/sℤ is also an element of ℤ/lℤ.
pub unsafe trait SmallerModulus<L> {}

/// A modulus *s* where s < l < 2*s for the given larger modulus *l*. This is
/// the precondition for reduction by conditional subtraction,
/// `elem_reduce_once()`.
pub unsafe trait SlightlySmallerModulus<L>: SmallerModulus<L> {}

/// A modulus *s* where √l <= s < l for the given larger modulus *l*. This is
/// the precondition for the more general Montgomery reduction from ℤ/lℤ to
/// ℤ/sℤ.
pub unsafe trait NotMuchSmallerModulus<L>: SmallerModulus<L> {}

pub unsafe trait PublicModulus {}

/// The x86 implementation of `GFp_bn_mul_mont`, at least, requires at least 4
/// limbs. For a long time we have required 4 limbs for all targets, though
/// this may be unnecessary. TODO: Replace this with
/// `n.len() < 256 / LIMB_BITS` so that 32-bit and 64-bit platforms behave the
/// same.
pub const MODULUS_MIN_LIMBS: usize = 4;

pub const MODULUS_MAX_LIMBS: usize = 8192 / LIMB_BITS;

/// The modulus *m* for a ring ℤ/mℤ, along with the precomputed values needed
/// for efficient Montgomery multiplication modulo *m*. The value must be odd
/// and larger than 2. The larger-than-1 requirement is imposed, at least, by
/// the modular inversion code.
pub struct Modulus<M> {
    limbs: BoxedLimbs<M>, // Also `value >= 3`.

    // n0 * N == -1 (mod r).
    //
    // r == 2**(N0_LIMBS_USED * LIMB_BITS) and LG_LITTLE_R == lg(r). This
    // ensures that we can do integer division by |r| by simply ignoring
    // `N0_LIMBS_USED` limbs. Similarly, we can calculate values modulo `r` by
    // just looking at the lowest `N0_LIMBS_USED` limbs. This is what makes
    // Montgomery multiplication efficient.
    //
    // As shown in Algorithm 1 of "Fast Prime Field Elliptic Curve Cryptography
    // with 256 Bit Primes" by Shay Gueron and Vlad Krasnov, in the loop of a
    // multi-limb Montgomery multiplication of a * b (mod n), given the
    // unreduced product t == a * b, we repeatedly calculate:
    //
    //    t1 := t % r         |t1| is |t|'s lowest limb (see previous paragraph).
    //    t2 := t1*n0*n
    //    t3 := t + t2
    //    t := t3 / r         copy all limbs of |t3| except the lowest to |t|.
    //
    // In the last step, it would only make sense to ignore the lowest limb of
    // |t3| if it were zero. The middle steps ensure that this is the case:
    //
    //                            t3 ==  0 (mod r)
    //                        t + t2 ==  0 (mod r)
    //                   t + t1*n0*n ==  0 (mod r)
    //                       t1*n0*n == -t (mod r)
    //                        t*n0*n == -t (mod r)
    //                          n0*n == -1 (mod r)
    //                            n0 == -1/n (mod r)
    //
    // Thus, in each iteration of the loop, we multiply by the constant factor
    // n0, the negative inverse of n (mod r).
    //
    // TODO(perf): Not all 32-bit platforms actually make use of n0[1]. For the
    // ones that don't, we could use a shorter `R` value and use faster `Limb`
    // calculations instead of double-precision `u64` calculations.
    n0: N0,

    oneRR: One<M, RR>,
}

impl<M: PublicModulus> core::fmt::Debug for Modulus<M> {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        fmt.debug_struct("Modulus")
            // TODO: Print modulus value.
            .finish()
    }
}

impl<M> Modulus<M> {
    pub fn from_be_bytes_with_bit_length(
        input: untrusted::Input,
    ) -> Result<(Self, bits::BitLength), error::KeyRejected> {
        let limbs = BoxedLimbs::positive_minimal_width_from_be_bytes(input)?;
        Self::from_boxed_limbs(limbs)
    }

    pub fn from_nonnegative_with_bit_length(
        n: Nonnegative,
    ) -> Result<(Self, bits::BitLength), error::KeyRejected> {
        let limbs = BoxedLimbs {
            limbs: n.limbs.into_boxed_slice(),
            m: PhantomData,
        };
        Self::from_boxed_limbs(limbs)
    }

    fn from_boxed_limbs(n: BoxedLimbs<M>) -> Result<(Self, bits::BitLength), error::KeyRejected> {
        if n.len() > MODULUS_MAX_LIMBS {
            return Err(error::KeyRejected::too_large());
        }
        if n.len() < MODULUS_MIN_LIMBS {
            return Err(error::KeyRejected::unexpected_error());
        }
        if limb::limbs_are_even_constant_time(&n) != LimbMask::False {
            return Err(error::KeyRejected::invalid_component());
        }
        if limb::limbs_less_than_limb_constant_time(&n, 3) != LimbMask::False {
            return Err(error::KeyRejected::unexpected_error());
        }

        // n_mod_r = n % r. As explained in the documentation for `n0`, this is
        // done by taking the lowest `N0_LIMBS_USED` limbs of `n`.
        #[allow(clippy::useless_conversion)]
        let n0 = {
            extern "C" {
                fn GFp_bn_neg_inv_mod_r_u64(n: u64) -> u64;
            }

            // XXX: u64::from isn't guaranteed to be constant time.
            let mut n_mod_r: u64 = u64::from(n[0]);

            if N0_LIMBS_USED == 2 {
                // XXX: If we use `<< LIMB_BITS` here then 64-bit builds
                // fail to compile because of `deny(exceeding_bitshifts)`.
                debug_assert_eq!(LIMB_BITS, 32);
                n_mod_r |= u64::from(n[1]) << 32;
            }
            N0::from(unsafe { GFp_bn_neg_inv_mod_r_u64(n_mod_r) })
        };

        let bits = limb::limbs_minimal_bits(&n.limbs);
        let oneRR = {
            let partial = PartialModulus {
                limbs: &n.limbs,
                n0: n0.clone(),
                m: PhantomData,
            };

            One::newRR(&partial, bits)
        };

        Ok((
            Self {
                limbs: n,
                n0,
                oneRR,
            },
            bits,
        ))
    }

    #[inline]
    fn width(&self) -> Width<M> {
        self.limbs.width()
    }

    fn zero<E>(&self) -> Elem<M, E> {
        Elem {
            limbs: BoxedLimbs::zero(self.width()),
            encoding: PhantomData,
        }
    }

    // TODO: Get rid of this
    fn one(&self) -> Elem<M, Unencoded> {
        let mut r = self.zero();
        r.limbs[0] = 1;
        r
    }

    pub fn oneRR(&self) -> &One<M, RR> {
        &self.oneRR
    }

    pub fn to_elem<L>(&self, l: &Modulus<L>) -> Elem<L, Unencoded>
    where
        M: SmallerModulus<L>,
    {
        // TODO: Encode this assertion into the `where` above.
        assert_eq!(self.width().num_limbs, l.width().num_limbs);
        let limbs = self.limbs.clone();
        Elem {
            limbs: BoxedLimbs {
                limbs: limbs.limbs,
                m: PhantomData,
            },
            encoding: PhantomData,
        }
    }

    fn as_partial(&self) -> PartialModulus<M> {
        PartialModulus {
            limbs: &self.limbs,
            n0: self.n0.clone(),
            m: PhantomData,
        }
    }
}

struct PartialModulus<'a, M> {
    limbs: &'a [Limb],
    n0: N0,
    m: PhantomData<M>,
}

impl<M> PartialModulus<'_, M> {
    // TODO: XXX Avoid duplication with `Modulus`.
    fn zero(&self) -> Elem<M, R> {
        let width = Width {
            num_limbs: self.limbs.len(),
            m: PhantomData,
        };
        Elem {
            limbs: BoxedLimbs::zero(width),
            encoding: PhantomData,
        }
    }
}

/// Elements of ℤ/mℤ for some modulus *m*.
//
// Defaulting `E` to `Unencoded` is a convenience for callers from outside this
// submodule. However, for maximum clarity, we always explicitly use
// `Unencoded` within the `bigint` submodule.
pub struct Elem<M, E = Unencoded> {
    limbs: BoxedLimbs<M>,

    /// The number of Montgomery factors that need to be canceled out from
    /// `value` to get the actual value.
    encoding: PhantomData<E>,
}

// TODO: `derive(Clone)` after https://github.com/rust-lang/rust/issues/26925
// is resolved or restrict `M: Clone` and `E: Clone`.
impl<M, E> Clone for Elem<M, E> {
    fn clone(&self) -> Self {
        Self {
            limbs: self.limbs.clone(),
            encoding: self.encoding,
        }
    }
}

impl<M, E> Elem<M, E> {
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.limbs.is_zero()
    }
}

impl<M, E: ReductionEncoding> Elem<M, E> {
    fn decode_once(self, m: &Modulus<M>) -> Elem<M, <E as ReductionEncoding>::Output> {
        // A multiplication isn't required since we're multiplying by the
        // unencoded value one (1); only a Montgomery reduction is needed.
        // However the only non-multiplication Montgomery reduction function we
        // have requires the input to be large, so we avoid using it here.
        let mut limbs = self.limbs;
        let num_limbs = m.width().num_limbs;
        let mut one = [0; MODULUS_MAX_LIMBS];
        one[0] = 1;
        let one = &one[..num_limbs]; // assert!(num_limbs <= MODULUS_MAX_LIMBS);
        limbs_mont_mul(&mut limbs, &one, &m.limbs, &m.n0);
        Elem {
            limbs,
            encoding: PhantomData,
        }
    }
}

impl<M> Elem<M, R> {
    #[inline]
    pub fn into_unencoded(self, m: &Modulus<M>) -> Elem<M, Unencoded> {
        self.decode_once(m)
    }
}

impl<M> Elem<M, Unencoded> {
    pub fn from_be_bytes_padded(
        input: untrusted::Input,
        m: &Modulus<M>,
    ) -> Result<Self, error::Unspecified> {
        Ok(Elem {
            limbs: BoxedLimbs::from_be_bytes_padded_less_than(input, m)?,
            encoding: PhantomData,
        })
    }

    #[inline]
    pub fn fill_be_bytes(&self, out: &mut [u8]) {
        // See Falko Strenzke, "Manger's Attack revisited", ICICS 2010.
        limb::big_endian_from_limbs(&self.limbs, out)
    }

    pub fn into_modulus<MM>(self) -> Result<Modulus<MM>, error::KeyRejected> {
        let (m, _bits) =
            Modulus::from_boxed_limbs(BoxedLimbs::minimal_width_from_unpadded(&self.limbs))?;
        Ok(m)
    }

    fn is_one(&self) -> bool {
        limb::limbs_equal_limb_constant_time(&self.limbs, 1) == LimbMask::True
    }
}

pub fn elem_mul<M, AF, BF>(
    a: &Elem<M, AF>,
    b: Elem<M, BF>,
    m: &Modulus<M>,
) -> Elem<M, <(AF, BF) as ProductEncoding>::Output>
where
    (AF, BF): ProductEncoding,
{
    elem_mul_(a, b, &m.as_partial())
}

fn elem_mul_<M, AF, BF>(
    a: &Elem<M, AF>,
    mut b: Elem<M, BF>,
    m: &PartialModulus<M>,
) -> Elem<M, <(AF, BF) as ProductEncoding>::Output>
where
    (AF, BF): ProductEncoding,
{
    limbs_mont_mul(&mut b.limbs, &a.limbs, &m.limbs, &m.n0);
    Elem {
        limbs: b.limbs,
        encoding: PhantomData,
    }
}

fn elem_mul_by_2<M, AF>(a: &mut Elem<M, AF>, m: &PartialModulus<M>) {
    extern "C" {
        fn LIMBS_shl_mod(r: *mut Limb, a: *const Limb, m: *const Limb, num_limbs: c::size_t);
    }
    unsafe {
        LIMBS_shl_mod(
            a.limbs.as_mut_ptr(),
            a.limbs.as_ptr(),
            m.limbs.as_ptr(),
            m.limbs.len(),
        );
    }
}

pub fn elem_reduced_once<Larger, Smaller: SlightlySmallerModulus<Larger>>(
    a: &Elem<Larger, Unencoded>,
    m: &Modulus<Smaller>,
) -> Elem<Smaller, Unencoded> {
    let mut r = a.limbs.clone();
    assert!(r.len() <= m.limbs.len());
    limb::limbs_reduce_once_constant_time(&mut r, &m.limbs);
    Elem {
        limbs: BoxedLimbs {
            limbs: r.limbs,
            m: PhantomData,
        },
        encoding: PhantomData,
    }
}

#[inline]
pub fn elem_reduced<Larger, Smaller: NotMuchSmallerModulus<Larger>>(
    a: &Elem<Larger, Unencoded>,
    m: &Modulus<Smaller>,
) -> Elem<Smaller, RInverse> {
    let mut tmp = [0; MODULUS_MAX_LIMBS];
    let tmp = &mut tmp[..a.limbs.len()];
    tmp.copy_from_slice(&a.limbs);

    let mut r = m.zero();
    limbs_from_mont_in_place(&mut r.limbs, tmp, &m.limbs, &m.n0);
    r
}

fn elem_squared<M, E>(
    mut a: Elem<M, E>,
    m: &PartialModulus<M>,
) -> Elem<M, <(E, E) as ProductEncoding>::Output>
where
    (E, E): ProductEncoding,
{
    limbs_mont_square(&mut a.limbs, &m.limbs, &m.n0);
    Elem {
        limbs: a.limbs,
        encoding: PhantomData,
    }
}

pub fn elem_widen<Larger, Smaller: SmallerModulus<Larger>>(
    a: Elem<Smaller, Unencoded>,
    m: &Modulus<Larger>,
) -> Elem<Larger, Unencoded> {
    let mut r = m.zero();
    r.limbs[..a.limbs.len()].copy_from_slice(&a.limbs);
    r
}

// TODO: Document why this works for all Montgomery factors.
pub fn elem_add<M, E>(mut a: Elem<M, E>, b: Elem<M, E>, m: &Modulus<M>) -> Elem<M, E> {
    extern "C" {
        // `r` and `a` may alias.
        fn LIMBS_add_mod(
            r: *mut Limb,
            a: *const Limb,
            b: *const Limb,
            m: *const Limb,
            num_limbs: c::size_t,
        );
    }
    unsafe {
        LIMBS_add_mod(
            a.limbs.as_mut_ptr(),
            a.limbs.as_ptr(),
            b.limbs.as_ptr(),
            m.limbs.as_ptr(),
            m.limbs.len(),
        )
    }
    a
}

// TODO: Document why this works for all Montgomery factors.
pub fn elem_sub<M, E>(mut a: Elem<M, E>, b: &Elem<M, E>, m: &Modulus<M>) -> Elem<M, E> {
    extern "C" {
        // `r` and `a` may alias.
        fn LIMBS_sub_mod(
            r: *mut Limb,
            a: *const Limb,
            b: *const Limb,
            m: *const Limb,
            num_limbs: c::size_t,
        );
    }
    unsafe {
        LIMBS_sub_mod(
            a.limbs.as_mut_ptr(),
            a.limbs.as_ptr(),
            b.limbs.as_ptr(),
            m.limbs.as_ptr(),
            m.limbs.len(),
        );
    }
    a
}

// The value 1, Montgomery-encoded some number of times.
pub struct One<M, E>(Elem<M, E>);

impl<M> One<M, RR> {
    // Returns RR = = R**2 (mod n) where R = 2**r is the smallest power of
    // 2**LIMB_BITS such that R > m.
    //
    // Even though the assembly on some 32-bit platforms works with 64-bit
    // values, using `LIMB_BITS` here, rather than `N0_LIMBS_USED * LIMB_BITS`,
    // is correct because R**2 will still be a multiple of the latter as
    // `N0_LIMBS_USED` is either one or two.
    fn newRR(m: &PartialModulus<M>, m_bits: bits::BitLength) -> Self {
        let m_bits = m_bits.as_usize_bits();
        let r = (m_bits + (LIMB_BITS - 1)) / LIMB_BITS * LIMB_BITS;

        // base = 2**(lg m - 1).
        let bit = m_bits - 1;
        let mut base = m.zero();
        base.limbs[bit / LIMB_BITS] = 1 << (bit % LIMB_BITS);

        // Double `base` so that base == R == 2**r (mod m). For normal moduli
        // that have the high bit of the highest limb set, this requires one
        // doubling. Unusual moduli require more doublings but we are less
        // concerned about the performance of those.
        //
        // Then double `base` again so that base == 2*R (mod n), i.e. `2` in
        // Montgomery form (`elem_exp_vartime_()` requires the base to be in
        // Montgomery form). Then compute
        // RR = R**2 == base**r == R**r == (2**r)**r (mod n).
        //
        // Take advantage of the fact that `elem_mul_by_2` is faster than
        // `elem_squared` by replacing some of the early squarings with shifts.
        // TODO: Benchmark shift vs. squaring performance to determine the
        // optimal value of `lg_base`.
        let lg_base = 2usize; // Shifts vs. squaring trade-off.
        debug_assert_eq!(lg_base.count_ones(), 1); // Must 2**n for n >= 0.
        let shifts = r - bit + lg_base;
        let exponent = (r / lg_base) as u64;
        for _ in 0..shifts {
            elem_mul_by_2(&mut base, m)
        }
        let RR = elem_exp_vartime_(base, exponent, m);

        Self(Elem {
            limbs: RR.limbs,
            encoding: PhantomData, // PhantomData<RR>
        })
    }
}

impl<M, E> AsRef<Elem<M, E>> for One<M, E> {
    fn as_ref(&self) -> &Elem<M, E> {
        &self.0
    }
}

/// A non-secret odd positive value in the range
/// [3, PUBLIC_EXPONENT_MAX_VALUE].
#[derive(Clone, Copy, Debug)]
pub struct PublicExponent(u64);

impl PublicExponent {
    pub fn from_be_bytes(
        input: untrusted::Input,
        min_value: u64,
    ) -> Result<Self, error::KeyRejected> {
        if input.len() > 5 {
            return Err(error::KeyRejected::too_large());
        }
        let value = input.read_all(error::KeyRejected::invalid_encoding(), |input| {
            // The exponent can't be zero and it can't be prefixed with
            // zero-valued bytes.
            if input.peek(0) {
                return Err(error::KeyRejected::invalid_encoding());
            }
            let mut value = 0u64;
            loop {
                let byte = input
                    .read_byte()
                    .map_err(|untrusted::EndOfInput| error::KeyRejected::invalid_encoding())?;
                value = (value << 8) | u64::from(byte);
                if input.at_end() {
                    return Ok(value);
                }
            }
        })?;

        // Step 2 / Step b. NIST SP800-89 defers to FIPS 186-3, which requires
        // `e >= 65537`. We enforce this when signing, but are more flexible in
        // verification, for compatibility. Only small public exponents are
        // supported.
        if value & 1 != 1 {
            return Err(error::KeyRejected::invalid_component());
        }
        debug_assert!(min_value & 1 == 1);
        debug_assert!(min_value <= PUBLIC_EXPONENT_MAX_VALUE);
        if min_value < 3 {
            return Err(error::KeyRejected::invalid_component());
        }
        if value < min_value {
            return Err(error::KeyRejected::too_small());
        }
        if value > PUBLIC_EXPONENT_MAX_VALUE {
            return Err(error::KeyRejected::too_large());
        }

        Ok(Self(value))
    }
}

// This limit was chosen to bound the performance of the simple
// exponentiation-by-squaring implementation in `elem_exp_vartime`. In
// particular, it helps mitigate theoretical resource exhaustion attacks. 33
// bits was chosen as the limit based on the recommendations in [1] and
// [2]. Windows CryptoAPI (at least older versions) doesn't support values
// larger than 32 bits [3], so it is unlikely that exponents larger than 32
// bits are being used for anything Windows commonly does.
//
// [1] https://www.imperialviolet.org/2012/03/16/rsae.html
// [2] https://www.imperialviolet.org/2012/03/17/rsados.html
// [3] https://msdn.microsoft.com/en-us/library/aa387685(VS.85).aspx
const PUBLIC_EXPONENT_MAX_VALUE: u64 = (1u64 << 33) - 1;

/// Calculates base**exponent (mod m).
// TODO: The test coverage needs to be expanded, e.g. test with the largest
// accepted exponent and with the most common values of 65537 and 3.
pub fn elem_exp_vartime<M>(
    base: Elem<M, Unencoded>,
    PublicExponent(exponent): PublicExponent,
    m: &Modulus<M>,
) -> Elem<M, R> {
    let base = elem_mul(m.oneRR().as_ref(), base, &m);
    elem_exp_vartime_(base, exponent, &m.as_partial())
}

/// Calculates base**exponent (mod m).
fn elem_exp_vartime_<M>(base: Elem<M, R>, exponent: u64, m: &PartialModulus<M>) -> Elem<M, R> {
    // Use what [Knuth] calls the "S-and-X binary method", i.e. variable-time
    // square-and-multiply that scans the exponent from the most significant
    // bit to the least significant bit (left-to-right). Left-to-right requires
    // less storage compared to right-to-left scanning, at the cost of needing
    // to compute `exponent.leading_zeros()`, which we assume to be cheap.
    //
    // During RSA public key operations the exponent is almost always either 65537
    // (0b10000000000000001) or 3 (0b11), both of which have a Hamming weight
    // of 2. During Montgomery setup the exponent is almost always a power of two,
    // with Hamming weight 1. As explained in [Knuth], exponentiation by squaring
    // is the most efficient algorithm when the Hamming weight is 2 or less. It
    // isn't the most efficient for all other, uncommon, exponent values but any
    // suboptimality is bounded by `PUBLIC_EXPONENT_MAX_VALUE`.
    //
    // This implementation is slightly simplified by taking advantage of the
    // fact that we require the exponent to be a positive integer.
    //
    // [Knuth]: The Art of Computer Programming, Volume 2: Seminumerical
    //          Algorithms (3rd Edition), Section 4.6.3.
    assert!(exponent >= 1);
    assert!(exponent <= PUBLIC_EXPONENT_MAX_VALUE);
    let mut acc = base.clone();
    let mut bit = 1 << (64 - 1 - exponent.leading_zeros());
    debug_assert!((exponent & bit) != 0);
    while bit > 1 {
        bit >>= 1;
        acc = elem_squared(acc, m);
        if (exponent & bit) != 0 {
            acc = elem_mul_(&base, acc, m);
        }
    }
    acc
}

// `M` represents the prime modulus for which the exponent is in the interval
// [1, `m` - 1).
pub struct PrivateExponent<M> {
    limbs: BoxedLimbs<M>,
}

impl<M> PrivateExponent<M> {
    pub fn from_be_bytes_padded(
        input: untrusted::Input,
        p: &Modulus<M>,
    ) -> Result<Self, error::Unspecified> {
        let dP = BoxedLimbs::from_be_bytes_padded_less_than(input, p)?;

        // Proof that `dP < p - 1`:
        //
        // If `dP < p` then either `dP == p - 1` or `dP < p - 1`. Since `p` is
        // odd, `p - 1` is even. `d` is odd, and an odd number modulo an even
        // number is odd. Therefore `dP` must be odd. But then it cannot be
        // `p - 1` and so we know `dP < p - 1`.
        //
        // Further we know `dP != 0` because `dP` is not even.
        if limb::limbs_are_even_constant_time(&dP) != LimbMask::False {
            return Err(error::Unspecified);
        }

        Ok(Self { limbs: dP })
    }
}

impl<M: Prime> PrivateExponent<M> {
    // Returns `p - 2`.
    fn for_flt(p: &Modulus<M>) -> Self {
        let two = elem_add(p.one(), p.one(), p);
        let p_minus_2 = elem_sub(p.zero(), &two, p);
        Self {
            limbs: p_minus_2.limbs,
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn elem_exp_consttime<M>(
    base: Elem<M, R>,
    exponent: &PrivateExponent<M>,
    m: &Modulus<M>,
) -> Result<Elem<M, Unencoded>, error::Unspecified> {
    use crate::limb::Window;

    const WINDOW_BITS: usize = 5;
    const TABLE_ENTRIES: usize = 1 << WINDOW_BITS;

    let num_limbs = m.limbs.len();

    let mut table = vec![0; TABLE_ENTRIES * num_limbs];

    fn gather<M>(table: &[Limb], i: Window, r: &mut Elem<M, R>) {
        extern "C" {
            fn LIMBS_select_512_32(
                r: *mut Limb,
                table: *const Limb,
                num_limbs: c::size_t,
                i: Window,
            ) -> bssl::Result;
        }
        Result::from(unsafe {
            LIMBS_select_512_32(r.limbs.as_mut_ptr(), table.as_ptr(), r.limbs.len(), i)
        })
        .unwrap();
    }

    fn power<M>(
        table: &[Limb],
        i: Window,
        mut acc: Elem<M, R>,
        mut tmp: Elem<M, R>,
        m: &Modulus<M>,
    ) -> (Elem<M, R>, Elem<M, R>) {
        for _ in 0..WINDOW_BITS {
            acc = elem_squared(acc, &m.as_partial());
        }
        gather(table, i, &mut tmp);
        let acc = elem_mul(&tmp, acc, m);
        (acc, tmp)
    }

    let tmp = m.one();
    let tmp = elem_mul(m.oneRR().as_ref(), tmp, m);

    fn entry(table: &[Limb], i: usize, num_limbs: usize) -> &[Limb] {
        &table[(i * num_limbs)..][..num_limbs]
    }
    fn entry_mut(table: &mut [Limb], i: usize, num_limbs: usize) -> &mut [Limb] {
        &mut table[(i * num_limbs)..][..num_limbs]
    }
    let num_limbs = m.limbs.len();
    entry_mut(&mut table, 0, num_limbs).copy_from_slice(&tmp.limbs);
    entry_mut(&mut table, 1, num_limbs).copy_from_slice(&base.limbs);
    for i in 2..TABLE_ENTRIES {
        let (src1, src2) = if i % 2 == 0 {
            (i / 2, i / 2)
        } else {
            (i - 1, 1)
        };
        let (previous, rest) = table.split_at_mut(num_limbs * i);
        let src1 = entry(previous, src1, num_limbs);
        let src2 = entry(previous, src2, num_limbs);
        let dst = entry_mut(rest, 0, num_limbs);
        limbs_mont_product(dst, src1, src2, &m.limbs, &m.n0);
    }

    let (r, _) = limb::fold_5_bit_windows(
        &exponent.limbs,
        |initial_window| {
            let mut r = Elem {
                limbs: base.limbs,
                encoding: PhantomData,
            };
            gather(&table, initial_window, &mut r);
            (r, tmp)
        },
        |(acc, tmp), window| power(&table, window, acc, tmp, m),
    );

    let r = r.into_unencoded(m);

    Ok(r)
}

/// Uses Fermat's Little Theorem to calculate modular inverse in constant time.
pub fn elem_inverse_consttime<M: Prime>(
    a: Elem<M, R>,
    m: &Modulus<M>,
) -> Result<Elem<M, Unencoded>, error::Unspecified> {
    elem_exp_consttime(a, &PrivateExponent::for_flt(&m), m)
}

#[cfg(target_arch = "x86_64")]
pub fn elem_exp_consttime<M>(
    base: Elem<M, R>,
    exponent: &PrivateExponent<M>,
    m: &Modulus<M>,
) -> Result<Elem<M, Unencoded>, error::Unspecified> {
    // The x86_64 assembly was written under the assumption that the input data
    // is aligned to `MOD_EXP_CTIME_MIN_CACHE_LINE_WIDTH` bytes, which was/is
    // 64 in OpenSSL. Similarly, OpenSSL uses the x86_64 assembly functions by
    // giving it only inputs `tmp`, `am`, and `np` that immediately follow the
    // table. The code seems to "work" even when the inputs aren't exactly
    // like that but the side channel defenses might not be as effective. All
    // the awkwardness here stems from trying to use the assembly code like
    // OpenSSL does.

    use crate::limb::Window;

    const WINDOW_BITS: usize = 5;
    const TABLE_ENTRIES: usize = 1 << WINDOW_BITS;

    let num_limbs = m.limbs.len();

    const ALIGNMENT: usize = 64;
    assert_eq!(ALIGNMENT % LIMB_BYTES, 0);
    let mut table = vec![0; ((TABLE_ENTRIES + 3) * num_limbs) + ALIGNMENT];
    let (table, state) = {
        let misalignment = (table.as_ptr() as usize) % ALIGNMENT;
        let table = &mut table[((ALIGNMENT - misalignment) / LIMB_BYTES)..];
        assert_eq!((table.as_ptr() as usize) % ALIGNMENT, 0);
        table.split_at_mut(TABLE_ENTRIES * num_limbs)
    };

    fn entry(table: &[Limb], i: usize, num_limbs: usize) -> &[Limb] {
        &table[(i * num_limbs)..][..num_limbs]
    }
    fn entry_mut(table: &mut [Limb], i: usize, num_limbs: usize) -> &mut [Limb] {
        &mut table[(i * num_limbs)..][..num_limbs]
    }

    const ACC: usize = 0; // `tmp` in OpenSSL
    const BASE: usize = ACC + 1; // `am` in OpenSSL
    const M: usize = BASE + 1; // `np` in OpenSSL

    entry_mut(state, BASE, num_limbs).copy_from_slice(&base.limbs);
    entry_mut(state, M, num_limbs).copy_from_slice(&m.limbs);

    fn scatter(table: &mut [Limb], state: &[Limb], i: Window, num_limbs: usize) {
        extern "C" {
            fn GFp_bn_scatter5(a: *const Limb, a_len: c::size_t, table: *mut Limb, i: Window);
        }
        unsafe {
            GFp_bn_scatter5(
                entry(state, ACC, num_limbs).as_ptr(),
                num_limbs,
                table.as_mut_ptr(),
                i,
            )
        }
    }

    fn gather(table: &[Limb], state: &mut [Limb], i: Window, num_limbs: usize) {
        extern "C" {
            fn GFp_bn_gather5(r: *mut Limb, a_len: c::size_t, table: *const Limb, i: Window);
        }
        unsafe {
            GFp_bn_gather5(
                entry_mut(state, ACC, num_limbs).as_mut_ptr(),
                num_limbs,
                table.as_ptr(),
                i,
            )
        }
    }

    fn gather_square(table: &[Limb], state: &mut [Limb], n0: &N0, i: Window, num_limbs: usize) {
        gather(table, state, i, num_limbs);
        assert_eq!(ACC, 0);
        let (acc, rest) = state.split_at_mut(num_limbs);
        let m = entry(rest, M - 1, num_limbs);
        limbs_mont_square(acc, m, n0);
    }

    fn gather_mul_base(table: &[Limb], state: &mut [Limb], n0: &N0, i: Window, num_limbs: usize) {
        extern "C" {
            fn GFp_bn_mul_mont_gather5(
                rp: *mut Limb,
                ap: *const Limb,
                table: *const Limb,
                np: *const Limb,
                n0: &N0,
                num: c::size_t,
                power: Window,
            );
        }
        unsafe {
            GFp_bn_mul_mont_gather5(
                entry_mut(state, ACC, num_limbs).as_mut_ptr(),
                entry(state, BASE, num_limbs).as_ptr(),
                table.as_ptr(),
                entry(state, M, num_limbs).as_ptr(),
                n0,
                num_limbs,
                i,
            );
        }
    }

    fn power(table: &[Limb], state: &mut [Limb], n0: &N0, i: Window, num_limbs: usize) {
        extern "C" {
            fn GFp_bn_power5(
                r: *mut Limb,
                a: *const Limb,
                table: *const Limb,
                n: *const Limb,
                n0: &N0,
                num: c::size_t,
                i: Window,
            );
        }
        unsafe {
            GFp_bn_power5(
                entry_mut(state, ACC, num_limbs).as_mut_ptr(),
                entry_mut(state, ACC, num_limbs).as_mut_ptr(),
                table.as_ptr(),
                entry(state, M, num_limbs).as_ptr(),
                n0,
                num_limbs,
                i,
            );
        }
    }

    // table[0] = base**0.
    {
        let acc = entry_mut(state, ACC, num_limbs);
        acc[0] = 1;
        limbs_mont_mul(acc, &m.oneRR.0.limbs, &m.limbs, &m.n0);
    }
    scatter(table, state, 0, num_limbs);

    // table[1] = base**1.
    entry_mut(state, ACC, num_limbs).copy_from_slice(&base.limbs);
    scatter(table, state, 1, num_limbs);

    for i in 2..(TABLE_ENTRIES as Window) {
        if i % 2 == 0 {
            // TODO: Optimize this to avoid gathering
            gather_square(table, state, &m.n0, i / 2, num_limbs);
        } else {
            gather_mul_base(table, state, &m.n0, i - 1, num_limbs)
        };
        scatter(table, state, i, num_limbs);
    }

    let state = limb::fold_5_bit_windows(
        &exponent.limbs,
        |initial_window| {
            gather(table, state, initial_window, num_limbs);
            state
        },
        |state, window| {
            power(table, state, &m.n0, window, num_limbs);
            state
        },
    );

    extern "C" {
        fn GFp_bn_from_montgomery(
            r: *mut Limb,
            a: *const Limb,
            not_used: *const Limb,
            n: *const Limb,
            n0: &N0,
            num: c::size_t,
        ) -> bssl::Result;
    }
    Result::from(unsafe {
        GFp_bn_from_montgomery(
            entry_mut(state, ACC, num_limbs).as_mut_ptr(),
            entry(state, ACC, num_limbs).as_ptr(),
            core::ptr::null(),
            entry(state, M, num_limbs).as_ptr(),
            &m.n0,
            num_limbs,
        )
    })?;
    let mut r = Elem {
        limbs: base.limbs,
        encoding: PhantomData,
    };
    r.limbs.copy_from_slice(entry(state, ACC, num_limbs));
    Ok(r)
}

/// Verified a == b**-1 (mod m), i.e. a**-1 == b (mod m).
pub fn verify_inverses_consttime<M>(
    a: &Elem<M, R>,
    b: Elem<M, Unencoded>,
    m: &Modulus<M>,
) -> Result<(), error::Unspecified> {
    if elem_mul(a, b, m).is_one() {
        Ok(())
    } else {
        Err(error::Unspecified)
    }
}

#[inline]
pub fn elem_verify_equal_consttime<M, E>(
    a: &Elem<M, E>,
    b: &Elem<M, E>,
) -> Result<(), error::Unspecified> {
    if limb::limbs_equal_limbs_consttime(&a.limbs, &b.limbs) == LimbMask::True {
        Ok(())
    } else {
        Err(error::Unspecified)
    }
}

/// Nonnegative integers.
pub struct Nonnegative {
    limbs: Vec<Limb>,
}

impl Nonnegative {
    pub fn from_be_bytes_with_bit_length(
        input: untrusted::Input,
    ) -> Result<(Self, bits::BitLength), error::Unspecified> {
        let mut limbs = vec![0; (input.len() + LIMB_BYTES - 1) / LIMB_BYTES];
        // Rejects empty inputs.
        limb::parse_big_endian_and_pad_consttime(input, &mut limbs)?;
        while limbs.last() == Some(&0) {
            let _ = limbs.pop();
        }
        let r_bits = limb::limbs_minimal_bits(&limbs);
        Ok((Self { limbs }, r_bits))
    }

    #[inline]
    pub fn is_odd(&self) -> bool {
        limb::limbs_are_even_constant_time(&self.limbs) != LimbMask::True
    }

    pub fn verify_less_than(&self, other: &Self) -> Result<(), error::Unspecified> {
        if !greater_than(other, self) {
            return Err(error::Unspecified);
        }
        Ok(())
    }

    pub fn to_elem<M>(&self, m: &Modulus<M>) -> Result<Elem<M, Unencoded>, error::Unspecified> {
        self.verify_less_than_modulus(&m)?;
        let mut r = m.zero();
        r.limbs[0..self.limbs.len()].copy_from_slice(&self.limbs);
        Ok(r)
    }

    pub fn verify_less_than_modulus<M>(&self, m: &Modulus<M>) -> Result<(), error::Unspecified> {
        if self.limbs.len() > m.limbs.len() {
            return Err(error::Unspecified);
        }
        if self.limbs.len() == m.limbs.len() {
            if limb::limbs_less_than_limbs_consttime(&self.limbs, &m.limbs) != LimbMask::True {
                return Err(error::Unspecified);
            }
        }
        Ok(())
    }
}

// Returns a > b.
fn greater_than(a: &Nonnegative, b: &Nonnegative) -> bool {
    if a.limbs.len() == b.limbs.len() {
        limb::limbs_less_than_limbs_vartime(&b.limbs, &a.limbs)
    } else {
        a.limbs.len() > b.limbs.len()
    }
}

#[derive(Clone)]
#[repr(transparent)]
struct N0([Limb; 2]);

const N0_LIMBS_USED: usize = 64 / LIMB_BITS;

impl From<u64> for N0 {
    #[inline]
    fn from(n0: u64) -> Self {
        #[cfg(target_pointer_width = "64")]
        {
            Self([n0, 0])
        }

        #[cfg(target_pointer_width = "32")]
        {
            Self([n0 as Limb, (n0 >> LIMB_BITS) as Limb])
        }
    }
}

/// r *= a
fn limbs_mont_mul(r: &mut [Limb], a: &[Limb], m: &[Limb], n0: &N0) {
    debug_assert_eq!(r.len(), m.len());
    debug_assert_eq!(a.len(), m.len());

    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    ))]
    unsafe {
        GFp_bn_mul_mont(
            r.as_mut_ptr(),
            r.as_ptr(),
            a.as_ptr(),
            m.as_ptr(),
            n0,
            r.len(),
        )
    }

    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    )))]
    {
        let mut tmp = [0; 2 * MODULUS_MAX_LIMBS];
        let tmp = &mut tmp[..(2 * a.len())];
        limbs_mul(tmp, r, a);
        limbs_from_mont_in_place(r, tmp, m, n0);
    }
}

fn limbs_from_mont_in_place(r: &mut [Limb], tmp: &mut [Limb], m: &[Limb], n0: &N0) {
    extern "C" {
        fn GFp_bn_from_montgomery_in_place(
            r: *mut Limb,
            num_r: c::size_t,
            a: *mut Limb,
            num_a: c::size_t,
            n: *const Limb,
            num_n: c::size_t,
            n0: &N0,
        ) -> bssl::Result;
    }
    Result::from(unsafe {
        GFp_bn_from_montgomery_in_place(
            r.as_mut_ptr(),
            r.len(),
            tmp.as_mut_ptr(),
            tmp.len(),
            m.as_ptr(),
            m.len(),
            &n0,
        )
    })
    .unwrap()
}

#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "x86_64",
    target_arch = "x86"
)))]
fn limbs_mul(r: &mut [Limb], a: &[Limb], b: &[Limb]) {
    debug_assert_eq!(r.len(), 2 * a.len());
    debug_assert_eq!(a.len(), b.len());
    let ab_len = a.len();

    crate::polyfill::slice::fill(&mut r[..ab_len], 0);
    for (i, &b_limb) in b.iter().enumerate() {
        r[ab_len + i] = unsafe {
            GFp_limbs_mul_add_limb(
                (&mut r[i..][..ab_len]).as_mut_ptr(),
                a.as_ptr(),
                b_limb,
                ab_len,
            )
        };
    }
}

/// r = a * b
#[cfg(not(target_arch = "x86_64"))]
fn limbs_mont_product(r: &mut [Limb], a: &[Limb], b: &[Limb], m: &[Limb], n0: &N0) {
    debug_assert_eq!(r.len(), m.len());
    debug_assert_eq!(a.len(), m.len());
    debug_assert_eq!(b.len(), m.len());

    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    ))]
    unsafe {
        GFp_bn_mul_mont(
            r.as_mut_ptr(),
            a.as_ptr(),
            b.as_ptr(),
            m.as_ptr(),
            n0,
            r.len(),
        )
    }

    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    )))]
    {
        let mut tmp = [0; 2 * MODULUS_MAX_LIMBS];
        let tmp = &mut tmp[..(2 * a.len())];
        limbs_mul(tmp, a, b);
        limbs_from_mont_in_place(r, tmp, m, n0)
    }
}

/// r = r**2
fn limbs_mont_square(r: &mut [Limb], m: &[Limb], n0: &N0) {
    debug_assert_eq!(r.len(), m.len());
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    ))]
    unsafe {
        GFp_bn_mul_mont(
            r.as_mut_ptr(),
            r.as_ptr(),
            r.as_ptr(),
            m.as_ptr(),
            n0,
            r.len(),
        )
    }

    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    )))]
    {
        let mut tmp = [0; 2 * MODULUS_MAX_LIMBS];
        let tmp = &mut tmp[..(2 * r.len())];
        limbs_mul(tmp, r, r);
        limbs_from_mont_in_place(r, tmp, m, n0)
    }
}

extern "C" {
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86_64",
        target_arch = "x86"
    ))]
    // `r` and/or 'a' and/or 'b' may alias.
    fn GFp_bn_mul_mont(
        r: *mut Limb,
        a: *const Limb,
        b: *const Limb,
        n: *const Limb,
        n0: &N0,
        num_limbs: c::size_t,
    );

    // `r` must not alias `a`
    #[cfg(any(
        test,
        not(any(
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "x86_64",
            target_arch = "x86"
        ))
    ))]
    #[must_use]
    fn GFp_limbs_mul_add_limb(r: *mut Limb, a: *const Limb, b: Limb, num_limbs: c::size_t) -> Limb;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use alloc::format;

    // Type-level representation of an arbitrary modulus.
    struct M {}

    unsafe impl PublicModulus for M {}

    #[test]
    fn test_elem_exp_consttime() {
        test::run(
            test_file!("bigint_elem_exp_consttime_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                let m = consume_modulus::<M>(test_case, "M");
                let expected_result = consume_elem(test_case, "ModExp", &m);
                let base = consume_elem(test_case, "A", &m);
                let e = {
                    let bytes = test_case.consume_bytes("E");
                    PrivateExponent::from_be_bytes_padded(untrusted::Input::from(&bytes), &m)
                        .expect("valid exponent")
                };
                let base = into_encoded(base, &m);
                let actual_result = elem_exp_consttime(base, &e, &m).unwrap();
                assert_elem_eq(&actual_result, &expected_result);

                Ok(())
            },
        )
    }

    // TODO: fn test_elem_exp_vartime() using
    // "src/rsa/bigint_elem_exp_vartime_tests.txt". See that file for details.
    // In the meantime, the function is tested indirectly via the RSA
    // verification and signing tests.
    #[test]
    fn test_elem_mul() {
        test::run(
            test_file!("bigint_elem_mul_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                let m = consume_modulus::<M>(test_case, "M");
                let expected_result = consume_elem(test_case, "ModMul", &m);
                let a = consume_elem(test_case, "A", &m);
                let b = consume_elem(test_case, "B", &m);

                let b = into_encoded(b, &m);
                let a = into_encoded(a, &m);
                let actual_result = elem_mul(&a, b, &m);
                let actual_result = actual_result.into_unencoded(&m);
                assert_elem_eq(&actual_result, &expected_result);

                Ok(())
            },
        )
    }

    #[test]
    fn test_elem_squared() {
        test::run(
            test_file!("bigint_elem_squared_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                let m = consume_modulus::<M>(test_case, "M");
                let expected_result = consume_elem(test_case, "ModSquare", &m);
                let a = consume_elem(test_case, "A", &m);

                let a = into_encoded(a, &m);
                let actual_result = elem_squared(a, &m.as_partial());
                let actual_result = actual_result.into_unencoded(&m);
                assert_elem_eq(&actual_result, &expected_result);

                Ok(())
            },
        )
    }

    #[test]
    fn test_elem_reduced() {
        test::run(
            test_file!("bigint_elem_reduced_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                struct MM {}
                unsafe impl SmallerModulus<MM> for M {}
                unsafe impl NotMuchSmallerModulus<MM> for M {}

                let m = consume_modulus::<M>(test_case, "M");
                let expected_result = consume_elem(test_case, "R", &m);
                let a =
                    consume_elem_unchecked::<MM>(test_case, "A", expected_result.limbs.len() * 2);

                let actual_result = elem_reduced(&a, &m);
                let oneRR = m.oneRR();
                let actual_result = elem_mul(oneRR.as_ref(), actual_result, &m);
                assert_elem_eq(&actual_result, &expected_result);

                Ok(())
            },
        )
    }

    #[test]
    fn test_elem_reduced_once() {
        test::run(
            test_file!("bigint_elem_reduced_once_tests.txt"),
            |section, test_case| {
                assert_eq!(section, "");

                struct N {}
                struct QQ {}
                unsafe impl SmallerModulus<N> for QQ {}
                unsafe impl SlightlySmallerModulus<N> for QQ {}

                let qq = consume_modulus::<QQ>(test_case, "QQ");
                let expected_result = consume_elem::<QQ>(test_case, "R", &qq);
                let n = consume_modulus::<N>(test_case, "N");
                let a = consume_elem::<N>(test_case, "A", &n);

                let actual_result = elem_reduced_once(&a, &qq);
                assert_elem_eq(&actual_result, &expected_result);

                Ok(())
            },
        )
    }

    #[test]
    fn test_modulus_debug() {
        let (modulus, _) = Modulus::<M>::from_be_bytes_with_bit_length(untrusted::Input::from(
            &[0xff; LIMB_BYTES * MODULUS_MIN_LIMBS],
        ))
        .unwrap();
        assert_eq!("Modulus", format!("{:?}", modulus));
    }

    #[test]
    fn test_public_exponent_debug() {
        let exponent =
            PublicExponent::from_be_bytes(untrusted::Input::from(&[0x1, 0x00, 0x01]), 65537)
                .unwrap();
        assert_eq!("PublicExponent(65537)", format!("{:?}", exponent));
    }

    fn consume_elem<M>(
        test_case: &mut test::TestCase,
        name: &str,
        m: &Modulus<M>,
    ) -> Elem<M, Unencoded> {
        let value = test_case.consume_bytes(name);
        Elem::from_be_bytes_padded(untrusted::Input::from(&value), m).unwrap()
    }

    fn consume_elem_unchecked<M>(
        test_case: &mut test::TestCase,
        name: &str,
        num_limbs: usize,
    ) -> Elem<M, Unencoded> {
        let value = consume_nonnegative(test_case, name);
        let mut limbs = BoxedLimbs::zero(Width {
            num_limbs,
            m: PhantomData,
        });
        limbs[0..value.limbs.len()].copy_from_slice(&value.limbs);
        Elem {
            limbs,
            encoding: PhantomData,
        }
    }

    fn consume_modulus<M>(test_case: &mut test::TestCase, name: &str) -> Modulus<M> {
        let value = test_case.consume_bytes(name);
        let (value, _) =
            Modulus::from_be_bytes_with_bit_length(untrusted::Input::from(&value)).unwrap();
        value
    }

    fn consume_nonnegative(test_case: &mut test::TestCase, name: &str) -> Nonnegative {
        let bytes = test_case.consume_bytes(name);
        let (r, _r_bits) =
            Nonnegative::from_be_bytes_with_bit_length(untrusted::Input::from(&bytes)).unwrap();
        r
    }

    fn assert_elem_eq<M, E>(a: &Elem<M, E>, b: &Elem<M, E>) {
        if elem_verify_equal_consttime(&a, b).is_err() {
            panic!("{:x?} != {:x?}", &*a.limbs, &*b.limbs);
        }
    }

    fn into_encoded<M>(a: Elem<M, Unencoded>, m: &Modulus<M>) -> Elem<M, R> {
        elem_mul(m.oneRR().as_ref(), a, m)
    }

    #[test]
    // TODO: wasm
    fn test_mul_add_words() {
        const ZERO: Limb = 0;
        const MAX: Limb = ZERO.wrapping_sub(1);
        static TEST_CASES: &[(&[Limb], &[Limb], Limb, Limb, &[Limb])] = &[
            (&[0], &[0], 0, 0, &[0]),
            (&[MAX], &[0], MAX, 0, &[MAX]),
            (&[0], &[MAX], MAX, MAX - 1, &[1]),
            (&[MAX], &[MAX], MAX, MAX, &[0]),
            (&[0, 0], &[MAX, MAX], MAX, MAX - 1, &[1, MAX]),
            (&[1, 0], &[MAX, MAX], MAX, MAX - 1, &[2, MAX]),
            (&[MAX, 0], &[MAX, MAX], MAX, MAX, &[0, 0]),
            (&[0, 1], &[MAX, MAX], MAX, MAX, &[1, 0]),
            (&[MAX, MAX], &[MAX, MAX], MAX, MAX, &[0, MAX]),
        ];

        for (i, (r_input, a, w, expected_retval, expected_r)) in TEST_CASES.iter().enumerate() {
            extern crate std;
            let mut r = std::vec::Vec::from(*r_input);
            assert_eq!(r.len(), a.len()); // Sanity check
            let actual_retval =
                unsafe { GFp_limbs_mul_add_limb(r.as_mut_ptr(), a.as_ptr(), *w, a.len()) };
            assert_eq!(&r, expected_r, "{}: {:x?} != {:x?}", i, &r[..], expected_r);
            assert_eq!(
                actual_retval, *expected_retval,
                "{}: {:x?} != {:x?}",
                i, actual_retval, *expected_retval
            );
        }
    }
}
