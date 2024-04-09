/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "vect.h"

/*
 * |out| = |inp|^|pow|, small footprint, public exponent
 */
static void exp_mont_384(vec384 out, const vec384 inp, const byte *pow,
                         size_t pow_bits, const vec384 p, limb_t n0)
{
#if 1
    vec384 ret;

    vec_copy(ret, inp, sizeof(ret));  /* ret = inp^1 */
    --pow_bits; /* most significant bit is set, skip over */
    while (pow_bits--) {
        sqr_mont_384(ret, ret, p, n0);
        if (is_bit_set(pow, pow_bits))
            mul_mont_384(ret, ret, inp, p, n0);
    }
    vec_copy(out, ret, sizeof(ret));  /* out = ret */
#else
    unsigned int i;
    vec384 sqr;

    vec_copy(sqr, inp, sizeof(sqr));
    for (i = 0; !is_bit_set(pow, i++);)
        sqr_mont_384(sqr, sqr, sqr, p, n0);
    vec_copy(out, sqr, sizeof(sqr));
    for (; i < pow_bits; i++) {
        sqr_mont_384(sqr, sqr, sqr, p, n0);
        if (is_bit_set(pow, i))
            mul_mont_384(out, out, sqr, p, n0);
    }
#endif
}

static void exp_mont_384x(vec384x out, const vec384x inp, const byte *pow,
                          size_t pow_bits, const vec384 p, limb_t n0)
{
    vec384x ret;

    vec_copy(ret, inp, sizeof(ret));  /* |ret| = |inp|^1 */
    --pow_bits; /* most significant bit is accounted for, skip over */
    while (pow_bits--) {
        sqr_mont_384x(ret, ret, p, n0);
        if (is_bit_set(pow, pow_bits))
            mul_mont_384x(ret, ret, inp, p, n0);
    }
    vec_copy(out, ret, sizeof(ret));  /* |out| = |ret| */
}
