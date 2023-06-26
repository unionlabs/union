/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "vect.h"

#ifdef __BLST_NO_ASM__
# include "no_asm.h"
#endif

/*
 * Following are some reference C implementations to assist new
 * assembly modules development, as starting-point stand-ins and for
 * cross-checking. In order to "polyfil" specific subroutine redefine
 * it on compiler command line, e.g. -Dmul_mont_384x=_mul_mont_384x.
 */

#ifdef lshift_mod_384
inline void lshift_mod_384(vec384 ret, const vec384 a, size_t n,
                           const vec384 mod)
{
    while(n--)
        add_mod_384(ret, a, a, mod), a = ret;
}
#endif

#ifdef mul_by_8_mod_384
inline void mul_by_8_mod_384(vec384 ret, const vec384 a, const vec384 mod)
{   lshift_mod_384(ret, a, 3, mod);   }
#endif

#ifdef mul_by_3_mod_384
inline void mul_by_3_mod_384(vec384 ret, const vec384 a, const vec384 mod)
{
    vec384 t;

    add_mod_384(t, a, a, mod);
    add_mod_384(ret, t, a, mod);
}
#endif

#ifdef mul_by_3_mod_384x
inline void mul_by_3_mod_384x(vec384x ret, const vec384x a, const vec384 mod)
{
    mul_by_3_mod_384(ret[0], a[0], mod);
    mul_by_3_mod_384(ret[1], a[1], mod);
}
#endif

#ifdef mul_by_8_mod_384x
inline void mul_by_8_mod_384x(vec384x ret, const vec384x a, const vec384 mod)
{
    mul_by_8_mod_384(ret[0], a[0], mod);
    mul_by_8_mod_384(ret[1], a[1], mod);
}
#endif

#ifdef mul_by_1_plus_i_mod_384x
inline void mul_by_1_plus_i_mod_384x(vec384x ret, const vec384x a,
                                     const vec384 mod)
{
    vec384 t;

    add_mod_384(t, a[0], a[1], mod);
    sub_mod_384(ret[0], a[0], a[1], mod);
    vec_copy(ret[1], t, sizeof(t));
}
#endif

#ifdef add_mod_384x
inline void add_mod_384x(vec384x ret, const vec384x a, const vec384x b,
                         const vec384 mod)
{
    add_mod_384(ret[0], a[0], b[0], mod);
    add_mod_384(ret[1], a[1], b[1], mod);
}
#endif

#ifdef sub_mod_384x
inline void sub_mod_384x(vec384x ret, const vec384x a, const vec384x b,
                         const vec384 mod)
{
    sub_mod_384(ret[0], a[0], b[0], mod);
    sub_mod_384(ret[1], a[1], b[1], mod);
}
#endif

#ifdef lshift_mod_384x
inline void lshift_mod_384x(vec384x ret, const vec384x a, size_t n,
                            const vec384 mod)
{
    lshift_mod_384(ret[0], a[0], n, mod);
    lshift_mod_384(ret[1], a[1], n, mod);
}
#endif

#if defined(mul_mont_384x) && !(defined(__ADX__) && !defined(__BLST_PORTABLE__))
void mul_mont_384x(vec384x ret, const vec384x a, const vec384x b,
                   const vec384 mod, limb_t n0)
{
    vec768 t0, t1, t2;
    vec384 aa, bb;

    mul_384(t0, a[0], b[0]);
    mul_384(t1, a[1], b[1]);

    add_mod_384(aa, a[0], a[1], mod);
    add_mod_384(bb, b[0], b[1], mod);
    mul_384(t2, aa, bb);
    sub_mod_384x384(t2, t2, t0, mod);
    sub_mod_384x384(t2, t2, t1, mod);

    sub_mod_384x384(t0, t0, t1, mod);

    redc_mont_384(ret[0], t0, mod, n0);
    redc_mont_384(ret[1], t2, mod, n0);
}
#endif

#if defined(sqr_mont_384x) && !(defined(__ADX__) && !defined(__BLST_PORTABLE__))
void sqr_mont_384x(vec384x ret, const vec384x a, const vec384 mod, limb_t n0)
{
    vec384 t0, t1;

    add_mod_384(t0, a[0], a[1], mod);
    sub_mod_384(t1, a[0], a[1], mod);

    mul_mont_384(ret[1], a[0], a[1], mod, n0);
    add_mod_384(ret[1], ret[1], ret[1], mod);

    mul_mont_384(ret[0], t0, t1, mod, n0);
}
#endif

limb_t div_3_limbs(const limb_t dividend_top[2], limb_t d_lo, limb_t d_hi);
limb_t quot_rem_128(limb_t *quot_rem, const limb_t *divisor, limb_t quotient);
limb_t quot_rem_64(limb_t *quot_rem, const limb_t *divisor, limb_t quotient);

/*
 * Divide 255-bit |val| by z^2 yielding 128-bit quotient and remainder in place.
 */
static void div_by_zz(limb_t val[])
{
    static const limb_t zz[] = { TO_LIMB_T(0x0000000100000000),
                                 TO_LIMB_T(0xac45a4010001a402) };
    size_t loop, zz_len = sizeof(zz)/sizeof(zz[0]);
    limb_t d_lo, d_hi;

    d_lo = zz[zz_len - 2];
    d_hi = zz[zz_len - 1];
    for (loop = zz_len, zz_len--; loop--;) {
        limb_t q = div_3_limbs(val + loop + zz_len, d_lo, d_hi);
        (void)quot_rem_128(val + loop, zz, q);
    }
    /* remainder is in low half of val[], quotient is in high */
}

/*
 * Divide 128-bit |val| by z yielding 64-bit quotient and remainder in place.
 */
static void div_by_z(limb_t val[])
{
    static const limb_t z[] = { TO_LIMB_T(0xd201000000010000) };
    size_t loop, z_len = sizeof(z)/sizeof(z[0]);
    limb_t d_lo, d_hi;

    d_lo = (sizeof(z) == sizeof(limb_t)) ? 0 : z[z_len - 2];
    d_hi = z[z_len - 1];
    for (loop = z_len, z_len--; loop--;) {
        limb_t q = div_3_limbs(val + loop + z_len, d_lo, d_hi);
        (void)quot_rem_64(val + loop, z, q);
    }
    /* remainder is in low half of val[], quotient is in high */
}
