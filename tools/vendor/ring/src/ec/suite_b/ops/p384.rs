// Copyright 2016 Brian Smith.
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

use super::{
    elem::{binary_op, binary_op_assign},
    elem_sqr_mul, elem_sqr_mul_acc, Modulus, *,
};
use core::marker::PhantomData;

macro_rules! p384_limbs {
    [$($limb:expr),+] => {
        limbs![$($limb),+]
    };
}

pub static COMMON_OPS: CommonOps = CommonOps {
    num_limbs: 384 / LIMB_BITS,

    q: Modulus {
        p: p384_limbs![
            0xffffffff, 0x00000000, 0x00000000, 0xffffffff, 0xfffffffe, 0xffffffff, 0xffffffff,
            0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff
        ],
        rr: p384_limbs![1, 0xfffffffe, 0, 2, 0, 0xfffffffe, 0, 2, 1, 0, 0, 0],
    },
    n: Elem {
        limbs: p384_limbs![
            0xccc52973, 0xecec196a, 0x48b0a77a, 0x581a0db2, 0xf4372ddf, 0xc7634d81, 0xffffffff,
            0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff
        ],
        m: PhantomData,
        encoding: PhantomData, // Unencoded
    },

    a: Elem {
        limbs: p384_limbs![
            0xfffffffc, 0x00000003, 0x00000000, 0xfffffffc, 0xfffffffb, 0xffffffff, 0xffffffff,
            0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff
        ],
        m: PhantomData,
        encoding: PhantomData, // Unreduced
    },
    b: Elem {
        limbs: p384_limbs![
            0x9d412dcc, 0x08118871, 0x7a4c32ec, 0xf729add8, 0x1920022e, 0x77f2209b, 0x94938ae2,
            0xe3374bee, 0x1f022094, 0xb62b21f4, 0x604fbff9, 0xcd08114b
        ],
        m: PhantomData,
        encoding: PhantomData, // Unreduced
    },

    elem_add_impl: GFp_p384_elem_add,
    elem_mul_mont: GFp_p384_elem_mul_mont,
    elem_sqr_mont: GFp_p384_elem_sqr_mont,

    point_add_jacobian_impl: GFp_nistz384_point_add,
};

pub static PRIVATE_KEY_OPS: PrivateKeyOps = PrivateKeyOps {
    common: &COMMON_OPS,
    elem_inv_squared: p384_elem_inv_squared,
    point_mul_base_impl: p384_point_mul_base_impl,
    point_mul_impl: GFp_nistz384_point_mul,
};

fn p384_elem_inv_squared(a: &Elem<R>) -> Elem<R> {
    // Calculate a**-2 (mod q) == a**(q - 3) (mod q)
    //
    // The exponent (q - 3) is:
    //
    //    0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe\
    //      ffffffff0000000000000000fffffffc

    #[inline]
    fn sqr_mul(a: &Elem<R>, squarings: usize, b: &Elem<R>) -> Elem<R> {
        elem_sqr_mul(&COMMON_OPS, a, squarings, b)
    }

    #[inline]
    fn sqr_mul_acc(a: &mut Elem<R>, squarings: usize, b: &Elem<R>) {
        elem_sqr_mul_acc(&COMMON_OPS, a, squarings, b)
    }

    let b_1 = &a;
    let b_11 = sqr_mul(b_1, 1, b_1);
    let b_111 = sqr_mul(&b_11, 1, b_1);
    let f_11 = sqr_mul(&b_111, 3, &b_111);
    let fff = sqr_mul(&f_11, 6, &f_11);
    let fff_111 = sqr_mul(&fff, 3, &b_111);
    let fffffff_11 = sqr_mul(&fff_111, 15, &fff_111);

    let fffffffffffffff = sqr_mul(&fffffff_11, 30, &fffffff_11);

    let ffffffffffffffffffffffffffffff = sqr_mul(&fffffffffffffff, 60, &fffffffffffffff);

    // ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
    let mut acc = sqr_mul(
        &ffffffffffffffffffffffffffffff,
        120,
        &ffffffffffffffffffffffffffffff,
    );

    // fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_111
    sqr_mul_acc(&mut acc, 15, &fff_111);

    // fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff
    sqr_mul_acc(&mut acc, 1 + 30, &fffffff_11);
    sqr_mul_acc(&mut acc, 2, &b_11);

    // fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff
    // 0000000000000000fffffff_11
    sqr_mul_acc(&mut acc, 64 + 30, &fffffff_11);

    // fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff
    // 0000000000000000fffffffc
    COMMON_OPS.elem_square(&mut acc);
    COMMON_OPS.elem_square(&mut acc);

    acc
}

fn p384_point_mul_base_impl(a: &Scalar) -> Point {
    // XXX: Not efficient. TODO: Precompute multiples of the generator.
    static GENERATOR: (Elem<R>, Elem<R>) = (
        Elem {
            limbs: p384_limbs![
                0x49c0b528, 0x3dd07566, 0xa0d6ce38, 0x20e378e2, 0x541b4d6e, 0x879c3afc, 0x59a30eff,
                0x64548684, 0x614ede2b, 0x812ff723, 0x299e1513, 0x4d3aadc2
            ],
            m: PhantomData,
            encoding: PhantomData,
        },
        Elem {
            limbs: p384_limbs![
                0x4b03a4fe, 0x23043dad, 0x7bb4a9ac, 0xa1bfa8bf, 0x2e83b050, 0x8bade756, 0x68f4ffd9,
                0xc6c35219, 0x3969a840, 0xdd800226, 0x5a15c5e9, 0x2b78abc2
            ],
            m: PhantomData,
            encoding: PhantomData,
        },
    );

    PRIVATE_KEY_OPS.point_mul(a, &GENERATOR)
}

pub static PUBLIC_KEY_OPS: PublicKeyOps = PublicKeyOps {
    common: &COMMON_OPS,
};

pub static SCALAR_OPS: ScalarOps = ScalarOps {
    common: &COMMON_OPS,
    scalar_inv_to_mont_impl: p384_scalar_inv_to_mont,
    scalar_mul_mont: GFp_p384_scalar_mul_mont,
};

pub static PUBLIC_SCALAR_OPS: PublicScalarOps = PublicScalarOps {
    scalar_ops: &SCALAR_OPS,
    public_key_ops: &PUBLIC_KEY_OPS,
    private_key_ops: &PRIVATE_KEY_OPS,

    q_minus_n: Elem {
        limbs: p384_limbs![
            0x333ad68c, 0x1313e696, 0xb74f5885, 0xa7e5f24c, 0x0bc8d21f, 0x389cb27e, 0, 0, 0, 0, 0,
            0
        ],

        m: PhantomData,
        encoding: PhantomData, // Unencoded
    },
};

pub static PRIVATE_SCALAR_OPS: PrivateScalarOps = PrivateScalarOps {
    scalar_ops: &SCALAR_OPS,

    oneRR_mod_n: Scalar {
        limbs: N_RR_LIMBS,
        m: PhantomData,
        encoding: PhantomData, // R
    },
};

fn p384_scalar_inv_to_mont(a: &Scalar<Unencoded>) -> Scalar<R> {
    // Calculate the modular inverse of scalar |a| using Fermat's Little
    // Theorem:
    //
    //   a**-1 (mod n) == a**(n - 2) (mod n)
    //
    // The exponent (n - 2) is:
    //
    //     0xffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf\
    //       581a0db248b0a77aecec196accc52971.

    fn mul(a: &Scalar<R>, b: &Scalar<R>) -> Scalar<R> {
        binary_op(GFp_p384_scalar_mul_mont, a, b)
    }

    fn sqr(a: &Scalar<R>) -> Scalar<R> {
        binary_op(GFp_p384_scalar_mul_mont, a, a)
    }

    fn sqr_mut(a: &mut Scalar<R>) {
        unary_op_from_binary_op_assign(GFp_p384_scalar_mul_mont, a);
    }

    // Returns (`a` squared `squarings` times) * `b`.
    fn sqr_mul(a: &Scalar<R>, squarings: usize, b: &Scalar<R>) -> Scalar<R> {
        debug_assert!(squarings >= 1);
        let mut tmp = sqr(a);
        for _ in 1..squarings {
            sqr_mut(&mut tmp);
        }
        mul(&tmp, b)
    }

    // Sets `acc` = (`acc` squared `squarings` times) * `b`.
    fn sqr_mul_acc(acc: &mut Scalar<R>, squarings: usize, b: &Scalar<R>) {
        debug_assert!(squarings >= 1);
        for _ in 0..squarings {
            sqr_mut(acc);
        }
        binary_op_assign(GFp_p384_scalar_mul_mont, acc, b)
    }

    fn to_mont(a: &Scalar<Unencoded>) -> Scalar<R> {
        static N_RR: Scalar<Unencoded> = Scalar {
            limbs: N_RR_LIMBS,
            m: PhantomData,
            encoding: PhantomData,
        };
        binary_op(GFp_p384_scalar_mul_mont, a, &N_RR)
    }

    // Indexes into `d`.
    const B_1: usize = 0;
    const B_11: usize = 1;
    const B_101: usize = 2;
    const B_111: usize = 3;
    const B_1001: usize = 4;
    const B_1011: usize = 5;
    const B_1101: usize = 6;
    const B_1111: usize = 7;
    const DIGIT_COUNT: usize = 8;

    let mut d = [Scalar::zero(); DIGIT_COUNT];
    d[B_1] = to_mont(a);
    let b_10 = sqr(&d[B_1]);
    for i in B_11..DIGIT_COUNT {
        d[i] = mul(&d[i - 1], &b_10);
    }

    let ff = sqr_mul(&d[B_1111], 0 + 4, &d[B_1111]);
    let ffff = sqr_mul(&ff, 0 + 8, &ff);
    let ffffffff = sqr_mul(&ffff, 0 + 16, &ffff);

    let ffffffffffffffff = sqr_mul(&ffffffff, 0 + 32, &ffffffff);

    let ffffffffffffffffffffffff = sqr_mul(&ffffffffffffffff, 0 + 32, &ffffffff);

    // ffffffffffffffffffffffffffffffffffffffffffffffff
    let mut acc = sqr_mul(&ffffffffffffffffffffffff, 0 + 96, &ffffffffffffffffffffffff);

    // The rest of the exponent, in binary, is:
    //
    //    1100011101100011010011011000000111110100001101110010110111011111
    //    0101100000011010000011011011001001001000101100001010011101111010
    //    1110110011101100000110010110101011001100110001010010100101110001

    static REMAINING_WINDOWS: [(u8, u8); 39] = [
        (2, B_11 as u8),
        (3 + 3, B_111 as u8),
        (1 + 2, B_11 as u8),
        (3 + 2, B_11 as u8),
        (1 + 4, B_1001 as u8),
        (4, B_1011 as u8),
        (6 + 4, B_1111 as u8),
        (3, B_101 as u8),
        (4 + 1, B_1 as u8),
        (4, B_1011 as u8),
        (4, B_1001 as u8),
        (1 + 4, B_1101 as u8),
        (4, B_1101 as u8),
        (4, B_1111 as u8),
        (1 + 4, B_1011 as u8),
        (6 + 4, B_1101 as u8),
        (5 + 4, B_1101 as u8),
        (4, B_1011 as u8),
        (2 + 4, B_1001 as u8),
        (2 + 1, B_1 as u8),
        (3 + 4, B_1011 as u8),
        (4 + 3, B_101 as u8),
        (2 + 3, B_111 as u8),
        (1 + 4, B_1111 as u8),
        (1 + 4, B_1011 as u8),
        (4, B_1011 as u8),
        (2 + 3, B_111 as u8),
        (1 + 2, B_11 as u8),
        (5 + 2, B_11 as u8),
        (2 + 4, B_1011 as u8),
        (1 + 3, B_101 as u8),
        (1 + 2, B_11 as u8),
        (2 + 2, B_11 as u8),
        (2 + 2, B_11 as u8),
        (3 + 3, B_101 as u8),
        (2 + 3, B_101 as u8),
        (2 + 3, B_101 as u8),
        (2, B_11 as u8),
        (3 + 1, B_1 as u8),
    ];

    for &(squarings, digit) in &REMAINING_WINDOWS[..] {
        sqr_mul_acc(&mut acc, usize::from(squarings), &d[usize::from(digit)]);
    }

    acc
}

unsafe extern "C" fn GFp_p384_elem_sqr_mont(
    r: *mut Limb,   // [COMMON_OPS.num_limbs]
    a: *const Limb, // [COMMON_OPS.num_limbs]
) {
    // XXX: Inefficient. TODO: Make a dedicated squaring routine.
    GFp_p384_elem_mul_mont(r, a, a);
}

const N_RR_LIMBS: [Limb; MAX_LIMBS] = p384_limbs![
    0x19b409a9, 0x2d319b24, 0xdf1aa419, 0xff3d81e5, 0xfcb82947, 0xbc3e483a, 0x4aab1cc5, 0xd40d4917,
    0x28266895, 0x3fb05b7a, 0x2b39bf21, 0x0c84ee01
];

extern "C" {
    fn GFp_p384_elem_add(
        r: *mut Limb,   // [COMMON_OPS.num_limbs]
        a: *const Limb, // [COMMON_OPS.num_limbs]
        b: *const Limb, // [COMMON_OPS.num_limbs]
    );
    fn GFp_p384_elem_mul_mont(
        r: *mut Limb,   // [COMMON_OPS.num_limbs]
        a: *const Limb, // [COMMON_OPS.num_limbs]
        b: *const Limb, // [COMMON_OPS.num_limbs]
    );

    fn GFp_nistz384_point_add(
        r: *mut Limb,   // [3][COMMON_OPS.num_limbs]
        a: *const Limb, // [3][COMMON_OPS.num_limbs]
        b: *const Limb, // [3][COMMON_OPS.num_limbs]
    );
    fn GFp_nistz384_point_mul(
        r: *mut Limb,          // [3][COMMON_OPS.num_limbs]
        p_scalar: *const Limb, // [COMMON_OPS.num_limbs]
        p_x: *const Limb,      // [COMMON_OPS.num_limbs]
        p_y: *const Limb,      // [COMMON_OPS.num_limbs]
    );

    fn GFp_p384_scalar_mul_mont(
        r: *mut Limb,   // [COMMON_OPS.num_limbs]
        a: *const Limb, // [COMMON_OPS.num_limbs]
        b: *const Limb, // [COMMON_OPS.num_limbs]
    );
}
