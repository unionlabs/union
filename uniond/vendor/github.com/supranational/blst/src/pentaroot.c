/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "fields.h"

static inline void mul_fr(vec256 ret, const vec256 a, const vec256 b)
{   mul_mont_sparse_256(ret, a, b, BLS12_381_r, r0);   }

static inline void sqr_fr(vec256 ret, const vec256 a)
{   sqr_mont_sparse_256(ret, a, BLS12_381_r, r0);   }

#ifdef __OPTIMIZE_SIZE__
void blst_fr_pentaroot(vec256 out, const vec256 inp)
{
    static const byte pow[] = {
        TO_BYTES(0x33333332cccccccd), TO_BYTES(0x217f0e679998f199),
        TO_BYTES(0xe14a56699d73f002), TO_BYTES(0x2e5f0fbadd72321c)
    };
    size_t pow_bits = 254;
    vec256 ret;

    vec_copy(ret, inp, sizeof(ret));  /* ret = inp^1 */
    --pow_bits; /* most significant bit is set, skip over */
    while (pow_bits--) {
        sqr_fr(ret, ret);
        if (is_bit_set(pow, pow_bits))
            mul_fr(ret, ret, inp);
    }
    vec_copy(out, ret, sizeof(ret));  /* out = ret */
}
#else
# if 0
/*
 * "255"-bit variant omits full reductions at the ends of squarings,
 * not implemented yet[?].
 */
static inline void sqr_n_mul_fr(vec256 out, const vec256 a, size_t count,
                                const vec256 b)
{   sqr_n_mul_mont_255(out, a, count, BLS12_381_r, r0, b);   }
# else
static void sqr_n_mul_fr(vec256 out, const vec256 a, size_t count,
                         const vec256 b)
{
    do {
        sqr_fr(out, a);
        a = out;
    } while (--count);
    mul_fr(out, out, b);
}
# endif

# define sqr(ret,a)		sqr_fr(ret,a)
# define mul(ret,a,b)		mul_fr(ret,a,b)
# define sqr_n_mul(ret,a,n,b)	sqr_n_mul_fr(ret,a,n,b)

# include "pentaroot-addchain.h"
void blst_fr_pentaroot(vec256 out, const vec256 inp)
{   PENTAROOT_MOD_BLS12_381_r(out, inp, vec256);   }
# undef PENTAROOT_MOD_BLS12_381_r

# undef sqr_n_mul
# undef sqr
# undef mul
#endif

void blst_fr_pentapow(vec256 out, const vec256 inp)
{
    vec256 tmp;

    sqr_fr(tmp, inp);
    sqr_fr(tmp, tmp);
    mul_fr(out, tmp, inp);
}
