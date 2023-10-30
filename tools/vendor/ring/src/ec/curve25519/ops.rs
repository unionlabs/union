// Copyright 2015-2017 Brian Smith.
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

//! Elliptic curve operations on the birationally equivalent curves Curve25519
//! and Edwards25519.

pub use super::scalar::{MaskedScalar, Scalar, SCALAR_LEN};
use crate::{
    bssl, error,
    limb::{Limb, LIMB_BITS},
};
use core::marker::PhantomData;

// Elem<T>` is `fe` in curve25519/internal.h.
// Elem<L> is `fe_loose` in curve25519/internal.h.
// Keep this in sync with curve25519/internal.h.
#[repr(C)]
pub struct Elem<E: Encoding> {
    limbs: [Limb; ELEM_LIMBS], // This is called `v` in the C code.
    encoding: PhantomData<E>,
}

pub trait Encoding {}
pub struct T;
impl Encoding for T {}

const ELEM_LIMBS: usize = 5 * 64 / LIMB_BITS;

impl<E: Encoding> Elem<E> {
    fn zero() -> Self {
        Self {
            limbs: Default::default(),
            encoding: PhantomData,
        }
    }
}

impl Elem<T> {
    fn negate(&mut self) {
        unsafe {
            GFp_x25519_fe_neg(self);
        }
    }
}

// An encoding of a curve point. If on Curve25519, it should be encoded as
// described in Section 5 of [RFC 7748]. If on Edwards25519, it should be
// encoded as described in section 5.1.2 of [RFC 8032].
//
// [RFC 7748] https://tools.ietf.org/html/rfc7748#section-5
// [RFC 8032] https://tools.ietf.org/html/rfc8032#section-5.1.2
pub type EncodedPoint = [u8; ELEM_LEN];
pub const ELEM_LEN: usize = 32;

// Keep this in sync with `ge_p3` in curve25519/internal.h.
#[repr(C)]
pub struct ExtPoint {
    x: Elem<T>,
    y: Elem<T>,
    z: Elem<T>,
    t: Elem<T>,
}

impl ExtPoint {
    pub fn new_at_infinity() -> Self {
        Self {
            x: Elem::zero(),
            y: Elem::zero(),
            z: Elem::zero(),
            t: Elem::zero(),
        }
    }

    pub fn from_encoded_point_vartime(encoded: &EncodedPoint) -> Result<Self, error::Unspecified> {
        let mut point = Self::new_at_infinity();

        Result::from(unsafe { GFp_x25519_ge_frombytes_vartime(&mut point, encoded) })
            .map(|()| point)
    }

    pub fn into_encoded_point(self) -> EncodedPoint {
        encode_point(self.x, self.y, self.z)
    }

    pub fn invert_vartime(&mut self) {
        self.x.negate();
        self.t.negate();
    }
}

// Keep this in sync with `ge_p2` in curve25519/internal.h.
#[repr(C)]
pub struct Point {
    x: Elem<T>,
    y: Elem<T>,
    z: Elem<T>,
}

impl Point {
    pub fn new_at_infinity() -> Self {
        Self {
            x: Elem::zero(),
            y: Elem::zero(),
            z: Elem::zero(),
        }
    }

    pub fn into_encoded_point(self) -> EncodedPoint {
        encode_point(self.x, self.y, self.z)
    }
}

fn encode_point(x: Elem<T>, y: Elem<T>, z: Elem<T>) -> EncodedPoint {
    let mut bytes = [0; ELEM_LEN];

    let sign_bit: u8 = unsafe {
        let mut recip = Elem::zero();
        GFp_x25519_fe_invert(&mut recip, &z);

        let mut x_over_z = Elem::zero();
        GFp_x25519_fe_mul_ttt(&mut x_over_z, &x, &recip);

        let mut y_over_z = Elem::zero();
        GFp_x25519_fe_mul_ttt(&mut y_over_z, &y, &recip);
        GFp_x25519_fe_tobytes(&mut bytes, &y_over_z);

        GFp_x25519_fe_isnegative(&x_over_z)
    };

    // The preceding computations must execute in constant time, but this
    // doesn't need to.
    bytes[ELEM_LEN - 1] ^= sign_bit << 7;

    bytes
}

extern "C" {
    fn GFp_x25519_fe_invert(out: &mut Elem<T>, z: &Elem<T>);
    fn GFp_x25519_fe_isnegative(elem: &Elem<T>) -> u8;
    fn GFp_x25519_fe_mul_ttt(h: &mut Elem<T>, f: &Elem<T>, g: &Elem<T>);
    fn GFp_x25519_fe_neg(f: &mut Elem<T>);
    fn GFp_x25519_fe_tobytes(bytes: &mut EncodedPoint, elem: &Elem<T>);
    fn GFp_x25519_ge_frombytes_vartime(h: &mut ExtPoint, s: &EncodedPoint) -> bssl::Result;
}
