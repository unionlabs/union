/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "fields.h"

#ifdef __OPTIMIZE_SIZE__
/*
 * 608 multiplications for scalar inversion modulo BLS12-381 prime, 32%
 * more than corresponding optimal addition-chain, plus mispredicted
 * branch penalties on top of that... The addition chain below was
 * measured to be >50% faster.
 */
static void flt_reciprocal_fp(vec384 out, const vec384 inp)
{
    static const byte BLS12_381_P_minus_2[] = {
        TO_BYTES(0xb9feffffffffaaa9), TO_BYTES(0x1eabfffeb153ffff),
        TO_BYTES(0x6730d2a0f6b0f624), TO_BYTES(0x64774b84f38512bf),
        TO_BYTES(0x4b1ba7b6434bacd7), TO_BYTES(0x1a0111ea397fe69a)
    };

    exp_mont_384(out, inp, BLS12_381_P_minus_2, 381, BLS12_381_P, p0);
}
#else
# define sqr(ret,a)		sqr_fp(ret,a)
# define mul(ret,a,b)		mul_fp(ret,a,b)
# define sqr_n_mul(ret,a,n,b)	sqr_n_mul_fp(ret,a,n,b)

# include "recip-addchain.h"
static void flt_reciprocal_fp(vec384 out, const vec384 inp)
{
    RECIPROCAL_MOD_BLS12_381_P(out, inp, vec384);
}
# undef RECIPROCAL_MOD_BLS12_381_P
# undef sqr_n_mul
# undef mul
# undef sqr
#endif

static void flt_reciprocal_fp2(vec384x out, const vec384x inp)
{
    vec384 t0, t1;

    /*
     * |out| = 1/(a + b*i) = a/(a^2+b^2) - b/(a^2+b^2)*i
     */
    sqr_fp(t0, inp[0]);
    sqr_fp(t1, inp[1]);
    add_fp(t0, t0, t1);
    flt_reciprocal_fp(t1, t0);
    mul_fp(out[0], inp[0], t1);
    mul_fp(out[1], inp[1], t1);
    neg_fp(out[1], out[1]);
}

static void reciprocal_fp(vec384 out, const vec384 inp)
{
    static const vec384 Px8 = {    /* left-aligned value of the modulus */
        TO_LIMB_T(0xcff7fffffffd5558), TO_LIMB_T(0xf55ffff58a9ffffd),
        TO_LIMB_T(0x39869507b587b120), TO_LIMB_T(0x23ba5c279c2895fb),
        TO_LIMB_T(0x58dd3db21a5d66bb), TO_LIMB_T(0xd0088f51cbff34d2)
    };
#ifdef __BLST_NO_ASM__
# define RRx4 BLS12_381_RR
#else
    static const vec384 RRx4 = {   /* (4<<768)%P */
        TO_LIMB_T(0x5f7e7cd070d107c2), TO_LIMB_T(0xec839a9ac49c13c8),
        TO_LIMB_T(0x6933786f44f4ef0b), TO_LIMB_T(0xd6bf8b9c676be983),
        TO_LIMB_T(0xd3adaaaa4dcefb06), TO_LIMB_T(0x12601bc1d82bc175)
    };
#endif
    union { vec768 x; vec384 r[2]; } temp;

    ct_inverse_mod_383(temp.x, inp, BLS12_381_P, Px8);
    redc_mont_384(temp.r[0], temp.x, BLS12_381_P, p0);
    mul_mont_384(temp.r[0], temp.r[0], RRx4, BLS12_381_P, p0);

#ifndef FUZZING_BUILD_MODE_UNSAFE_FOR_PRODUCTION
    /* sign goes straight to flt_reciprocal */
    mul_mont_384(temp.r[1], temp.r[0], inp, BLS12_381_P, p0);
    if (vec_is_equal(temp.r[1],  BLS12_381_Rx.p, sizeof(vec384)) |
        vec_is_zero(temp.r[1], sizeof(vec384)))
        vec_copy(out, temp.r[0], sizeof(vec384));
    else
        flt_reciprocal_fp(out, inp);
#else
    vec_copy(out, temp.r[0], sizeof(vec384));
#endif
#undef RRx4
}

void blst_fp_inverse(vec384 out, const vec384 inp)
{   reciprocal_fp(out, inp);   }

void blst_fp_eucl_inverse(vec384 ret, const vec384 a)
{   reciprocal_fp(ret, a);   }

static void reciprocal_fp2(vec384x out, const vec384x inp)
{
    vec384 t0, t1;

    /*
     * |out| = 1/(a + b*i) = a/(a^2+b^2) - b/(a^2+b^2)*i
     */
    sqr_fp(t0, inp[0]);
    sqr_fp(t1, inp[1]);
    add_fp(t0, t0, t1);
    reciprocal_fp(t1, t0);
    mul_fp(out[0], inp[0], t1);
    mul_fp(out[1], inp[1], t1);
    neg_fp(out[1], out[1]);
}

void blst_fp2_inverse(vec384x out, const vec384x inp)
{   reciprocal_fp2(out, inp);   }

void blst_fp2_eucl_inverse(vec384x out, const vec384x inp)
{   reciprocal_fp2(out, inp);   }

static void reciprocal_fr(vec256 out, const vec256 inp)
{
    static const vec256 rx2 = { /* left-aligned value of the modulus */
        TO_LIMB_T(0xfffffffe00000002), TO_LIMB_T(0xa77b4805fffcb7fd),
        TO_LIMB_T(0x6673b0101343b00a), TO_LIMB_T(0xe7db4ea6533afa90),
    };
    vec512 temp;

    ct_inverse_mod_256(temp, inp, BLS12_381_r, rx2);
    redc_mont_256(out, temp, BLS12_381_r, r0);
    mul_mont_sparse_256(out, out, BLS12_381_rRR, BLS12_381_r, r0);
}

void blst_fr_inverse(vec256 out, const vec256 inp)
{   reciprocal_fr(out, inp);   }

void blst_fr_eucl_inverse(vec256 out, const vec256 inp)
{   reciprocal_fr(out, inp);   }
