/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "fields.h"

#ifdef __OPTIMIZE_SIZE__
static void recip_sqrt_fp_3mod4(vec384 out, const vec384 inp)
{
    static const byte BLS_12_381_P_minus_3_div_4[] = {
        TO_BYTES(0xee7fbfffffffeaaa), TO_BYTES(0x07aaffffac54ffff),
        TO_BYTES(0xd9cc34a83dac3d89), TO_BYTES(0xd91dd2e13ce144af),
        TO_BYTES(0x92c6e9ed90d2eb35), TO_BYTES(0x0680447a8e5ff9a6)
    };

    exp_mont_384(out, inp, BLS_12_381_P_minus_3_div_4, 379, BLS12_381_P, p0);
}
#else
# if 1
/*
 * "383"-bit variant omits full reductions at the ends of squarings,
 * which results in up to ~15% improvement. [One can improve further
 * by omitting full reductions even after multiplications and
 * performing final reduction at the very end of the chain.]
 */
static inline void sqr_n_mul_fp(vec384 out, const vec384 a, size_t count,
                                const vec384 b)
{   sqr_n_mul_mont_383(out, a, count, BLS12_381_P, p0, b);   }
# else
static void sqr_n_mul_fp(vec384 out, const vec384 a, size_t count,
                         const vec384 b)
{
    while(count--) {
        sqr_fp(out, a);
        a = out;
    }
    mul_fp(out, out, b);
}
# endif

# define sqr(ret,a)		sqr_fp(ret,a)
# define mul(ret,a,b)		mul_fp(ret,a,b)
# define sqr_n_mul(ret,a,n,b)	sqr_n_mul_fp(ret,a,n,b)

# include "sqrt-addchain.h"
static void recip_sqrt_fp_3mod4(vec384 out, const vec384 inp)
{
    RECIP_SQRT_MOD_BLS12_381_P(out, inp, vec384);
}
# undef RECIP_SQRT_MOD_BLS12_381_P

# undef sqr_n_mul
# undef sqr
# undef mul
#endif

static bool_t recip_sqrt_fp(vec384 out, const vec384 inp)
{
    vec384 t0, t1;
    bool_t ret;

    recip_sqrt_fp_3mod4(t0, inp);

    mul_fp(t1, t0, inp);
    sqr_fp(t1, t1);
    ret = vec_is_equal(t1, inp, sizeof(t1));
    vec_copy(out, t0, sizeof(t0));

    return ret;
}

static bool_t sqrt_fp(vec384 out, const vec384 inp)
{
    vec384 t0, t1;
    bool_t ret;

    recip_sqrt_fp_3mod4(t0, inp);

    mul_fp(t0, t0, inp);
    sqr_fp(t1, t0);
    ret = vec_is_equal(t1, inp, sizeof(t1));
    vec_copy(out, t0, sizeof(t0));

    return ret;
}

int blst_fp_sqrt(vec384 out, const vec384 inp)
{   return (int)sqrt_fp(out, inp);   }

int blst_fp_is_square(const vec384 inp)
{
    return (int)ct_is_square_mod_384(inp, BLS12_381_P);
}

static bool_t sqrt_align_fp2(vec384x out, const vec384x ret,
                             const vec384x sqrt, const vec384x inp)
{
    static const vec384x sqrt_minus_1 = { { 0 }, { ONE_MONT_P } };
    static const vec384x sqrt_sqrt_minus_1 = {
      /*
       * "magic" number is ±2^((p-3)/4)%p, which is "1/sqrt(2)",
       * in quotes because 2*"1/sqrt(2)"^2 == -1 mod p, not 1,
       * but it pivots into "complex" plane nevertheless...
       */
      { TO_LIMB_T(0x3e2f585da55c9ad1), TO_LIMB_T(0x4294213d86c18183),
        TO_LIMB_T(0x382844c88b623732), TO_LIMB_T(0x92ad2afd19103e18),
        TO_LIMB_T(0x1d794e4fac7cf0b9), TO_LIMB_T(0x0bd592fc7d825ec8) },
      { TO_LIMB_T(0x7bcfa7a25aa30fda), TO_LIMB_T(0xdc17dec12a927e7c),
        TO_LIMB_T(0x2f088dd86b4ebef1), TO_LIMB_T(0xd1ca2087da74d4a7),
        TO_LIMB_T(0x2da2596696cebc1d), TO_LIMB_T(0x0e2b7eedbbfd87d2) }
    };
    static const vec384x sqrt_minus_sqrt_minus_1 = {
      { TO_LIMB_T(0x7bcfa7a25aa30fda), TO_LIMB_T(0xdc17dec12a927e7c),
        TO_LIMB_T(0x2f088dd86b4ebef1), TO_LIMB_T(0xd1ca2087da74d4a7),
        TO_LIMB_T(0x2da2596696cebc1d), TO_LIMB_T(0x0e2b7eedbbfd87d2) },
      { TO_LIMB_T(0x7bcfa7a25aa30fda), TO_LIMB_T(0xdc17dec12a927e7c),
        TO_LIMB_T(0x2f088dd86b4ebef1), TO_LIMB_T(0xd1ca2087da74d4a7),
        TO_LIMB_T(0x2da2596696cebc1d), TO_LIMB_T(0x0e2b7eedbbfd87d2) }
    };
    vec384x coeff, t0, t1;
    bool_t is_sqrt, flag;

    /*
     * Instead of multiple trial squarings we can perform just one
     * and see if the result is "rotated by multiple of 90°" in
     * relation to |inp|, and "rotate" |ret| accordingly.
     */
    sqr_fp2(t0, sqrt);
    /* "sqrt(|inp|)"^2 = (a + b*i)^2 = (a^2-b^2) + 2ab*i */

    /* (a^2-b^2) + 2ab*i == |inp| ? |ret| is spot on */
    sub_fp2(t1, t0, inp);
    is_sqrt = vec_is_zero(t1, sizeof(t1));
    vec_copy(coeff, BLS12_381_Rx.p2, sizeof(coeff));

    /* -(a^2-b^2) - 2ab*i == |inp| ? "rotate |ret| by 90°" */
    add_fp2(t1, t0, inp);
    vec_select(coeff, sqrt_minus_1, coeff, sizeof(coeff),
               flag = vec_is_zero(t1, sizeof(t1)));
    is_sqrt |= flag;

    /* 2ab - (a^2-b^2)*i == |inp| ? "rotate |ret| by 135°" */
    sub_fp(t1[0], t0[0], inp[1]);
    add_fp(t1[1], t0[1], inp[0]);
    vec_select(coeff, sqrt_sqrt_minus_1, coeff, sizeof(coeff),
               flag = vec_is_zero(t1, sizeof(t1)));
    is_sqrt |= flag;

    /* -2ab + (a^2-b^2)*i == |inp| ? "rotate |ret| by 45°" */
    add_fp(t1[0], t0[0], inp[1]);
    sub_fp(t1[1], t0[1], inp[0]);
    vec_select(coeff, sqrt_minus_sqrt_minus_1, coeff, sizeof(coeff),
               flag = vec_is_zero(t1, sizeof(t1)));
    is_sqrt |= flag;

    /* actual "rotation" */
    mul_fp2(out, ret, coeff);

    return is_sqrt;
}

/*
 * |inp| = a + b*i
 */
static bool_t recip_sqrt_fp2(vec384x out, const vec384x inp,
                                          const vec384x recip_ZZZ,
                                          const vec384x magic_ZZZ)
{
    vec384 aa, bb, cc;
    vec384x inp_;
    bool_t is_sqrt;

    sqr_fp(aa, inp[0]);
    sqr_fp(bb, inp[1]);
    add_fp(aa, aa, bb);

    is_sqrt = recip_sqrt_fp(cc, aa);  /* 1/sqrt(a²+b²)                    */

    /* if |inp| doesn't have quadratic residue, multiply by "1/Z³" ...    */
    mul_fp2(inp_, inp, recip_ZZZ);
    /* ... and adjust |aa| and |cc| accordingly                           */
    {
        vec384 za, zc;

        mul_fp(za, aa, magic_ZZZ[0]); /* aa*(za² + zb²)                   */
        mul_fp(zc, cc, magic_ZZZ[1]); /* cc*(za² + zb²)^((p-3)/4)         */
        vec_select(aa, aa, za, sizeof(aa), is_sqrt);
        vec_select(cc, cc, zc, sizeof(cc), is_sqrt);
    }
    vec_select(inp_, inp, inp_, sizeof(inp_), is_sqrt);

    mul_fp(aa, aa, cc);               /* sqrt(a²+b²)                      */

    sub_fp(bb, inp_[0], aa);
    add_fp(aa, inp_[0], aa);
    vec_select(aa, bb, aa, sizeof(aa), vec_is_zero(aa, sizeof(aa)));
    div_by_2_fp(aa, aa);              /* (a ± sqrt(a²+b²))/2              */

    /* if it says "no sqrt," final "align" will find right one...         */
    (void)recip_sqrt_fp(out[0], aa);  /* 1/sqrt((a ± sqrt(a²+b²))/2)      */

    div_by_2_fp(out[1], inp_[1]);
    mul_fp(out[1], out[1], out[0]);   /* b/(2*sqrt((a ± sqrt(a²+b²))/2))  */
    mul_fp(out[0], out[0], aa);       /* sqrt((a ± sqrt(a²+b²))/2)        */

    /* bound to succeed                                                   */
    (void)sqrt_align_fp2(out, out, out, inp_);

    mul_fp(out[0], out[0], cc);       /* inverse the result               */
    mul_fp(out[1], out[1], cc);
    neg_fp(out[1], out[1]);

    return is_sqrt;
}

static bool_t sqrt_fp2(vec384x out, const vec384x inp)
{
    vec384x ret;
    vec384 aa, bb;

    sqr_fp(aa, inp[0]);
    sqr_fp(bb, inp[1]);
    add_fp(aa, aa, bb);

    /* don't pay attention to return value, final "align" will tell...    */
    (void)sqrt_fp(aa, aa);            /* sqrt(a²+b²)                      */

    sub_fp(bb, inp[0], aa);
    add_fp(aa, inp[0], aa);
    vec_select(aa, bb, aa, sizeof(aa), vec_is_zero(aa, sizeof(aa)));
    div_by_2_fp(aa, aa);              /* (a ± sqrt(a²+b²))/2              */

    /* if it says "no sqrt," final "align" will find right one...         */
    (void)recip_sqrt_fp(ret[0], aa);  /* 1/sqrt((a ± sqrt(a²+b²))/2)      */

    div_by_2_fp(ret[1], inp[1]);
    mul_fp(ret[1], ret[1], ret[0]);   /* b/(2*sqrt((a ± sqrt(a²+b²))/2))  */
    mul_fp(ret[0], ret[0], aa);       /* sqrt((a ± sqrt(a²+b²))/2)        */

    /*
     * Now see if |ret| is or can be made sqrt(|inp|)...
     */

    return sqrt_align_fp2(out, ret, ret, inp);
}

int blst_fp2_sqrt(vec384x out, const vec384x inp)
{   return (int)sqrt_fp2(out, inp);   }

int blst_fp2_is_square(const vec384x inp)
{
    vec384 aa, bb;

    sqr_fp(aa, inp[0]);
    sqr_fp(bb, inp[1]);
    add_fp(aa, aa, bb);

    return (int)ct_is_square_mod_384(aa, BLS12_381_P);
}
