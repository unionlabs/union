// Copyright 2017 Brian Smith.
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

use crate::{
    arithmetic::montgomery::{Encoding, ProductEncoding},
    limb::{Limb, LIMB_BITS},
};
use core::marker::PhantomData;

/// Elements of ℤ/mℤ for some modulus *m*. Elements are always fully reduced
/// with respect to *m*; i.e. the 0 <= x < m for every value x.
#[derive(Clone, Copy)]
pub struct Elem<M, E: Encoding> {
    // XXX: pub
    pub limbs: [Limb; MAX_LIMBS],

    /// The modulus *m* for the ring ℤ/mℤ for which this element is a value.
    pub m: PhantomData<M>,

    /// The number of Montgomery factors that need to be canceled out from
    /// `value` to get the actual value.
    pub encoding: PhantomData<E>,
}

impl<M, E: Encoding> Elem<M, E> {
    // There's no need to convert `value` to the Montgomery domain since
    // 0 * R**2 (mod m) == 0, so neither the modulus nor the encoding are needed
    // as inputs for constructing a zero-valued element.
    pub fn zero() -> Self {
        Self {
            limbs: [0; MAX_LIMBS],
            m: PhantomData,
            encoding: PhantomData,
        }
    }
}

#[inline]
pub fn mul_mont<M, EA: Encoding, EB: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb, b: *const Limb),
    a: &Elem<M, EA>,
    b: &Elem<M, EB>,
) -> Elem<M, <(EA, EB) as ProductEncoding>::Output>
where
    (EA, EB): ProductEncoding,
{
    binary_op(f, a, b)
}

// let r = f(a, b); return r;
#[inline]
pub fn binary_op<M, EA: Encoding, EB: Encoding, ER: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb, b: *const Limb),
    a: &Elem<M, EA>,
    b: &Elem<M, EB>,
) -> Elem<M, ER> {
    let mut r = Elem {
        limbs: [0; MAX_LIMBS],
        m: PhantomData,
        encoding: PhantomData,
    };
    unsafe { f(r.limbs.as_mut_ptr(), a.limbs.as_ptr(), b.limbs.as_ptr()) }
    r
}

// a := f(a, b);
#[inline]
pub fn binary_op_assign<M, EA: Encoding, EB: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb, b: *const Limb),
    a: &mut Elem<M, EA>,
    b: &Elem<M, EB>,
) {
    unsafe { f(a.limbs.as_mut_ptr(), a.limbs.as_ptr(), b.limbs.as_ptr()) }
}

// let r = f(a); return r;
#[inline]
pub fn unary_op<M, E: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb),
    a: &Elem<M, E>,
) -> Elem<M, E> {
    let mut r = Elem {
        limbs: [0; MAX_LIMBS],
        m: PhantomData,
        encoding: PhantomData,
    };
    unsafe { f(r.limbs.as_mut_ptr(), a.limbs.as_ptr()) }
    r
}

// a := f(a);
#[inline]
pub fn unary_op_assign<M, E: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb),
    a: &mut Elem<M, E>,
) {
    unsafe { f(a.limbs.as_mut_ptr(), a.limbs.as_ptr()) }
}

// a := f(a, a);
#[inline]
pub fn unary_op_from_binary_op_assign<M, E: Encoding>(
    f: unsafe extern "C" fn(r: *mut Limb, a: *const Limb, b: *const Limb),
    a: &mut Elem<M, E>,
) {
    unsafe { f(a.limbs.as_mut_ptr(), a.limbs.as_ptr(), a.limbs.as_ptr()) }
}

pub const MAX_LIMBS: usize = (384 + (LIMB_BITS - 1)) / LIMB_BITS;
