/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
/*
 * Why this file? Overall goal is to ensure that all internal calls
 * remain internal after linking application. This is to both
 *
 * a) minimize possibility of external name conflicts (since all
 *    non-blst-prefixed and [assembly subroutines] remain static);
 * b) preclude possibility of unintentional internal reference
 *    overload in shared library context (one can achieve same
 *    effect with -Bsymbolic, but we don't want to rely on end-user
 *    to remember to use it);
 */

#include "fields.h"

/*
 * BLS12-381-specifc Fr shortcuts to assembly.
 */
void blst_fr_add(vec256 ret, const vec256 a, const vec256 b)
{   add_mod_256(ret, a, b, BLS12_381_r);   }

void blst_fr_sub(vec256 ret, const vec256 a, const vec256 b)
{   sub_mod_256(ret, a, b, BLS12_381_r);   }

void blst_fr_mul_by_3(vec256 ret, const vec256 a)
{   mul_by_3_mod_256(ret, a, BLS12_381_r);   }

void blst_fr_lshift(vec256 ret, const vec256 a, size_t count)
{   lshift_mod_256(ret, a, count, BLS12_381_r);   }

void blst_fr_rshift(vec256 ret, const vec256 a, size_t count)
{   rshift_mod_256(ret, a, count, BLS12_381_r);   }

void blst_fr_mul(vec256 ret, const vec256 a, const vec256 b)
{   mul_mont_sparse_256(ret, a, b, BLS12_381_r, r0);   }

void blst_fr_sqr(vec256 ret, const vec256 a)
{   sqr_mont_sparse_256(ret, a, BLS12_381_r, r0);   }

void blst_fr_cneg(vec256 ret, const vec256 a, int flag)
{   cneg_mod_256(ret, a, is_zero(flag) ^ 1, BLS12_381_r);   }

void blst_fr_to(vec256 ret, const vec256 a)
{   mul_mont_sparse_256(ret, a, BLS12_381_rRR, BLS12_381_r, r0);   }

void blst_fr_from(vec256 ret, const vec256 a)
{   from_mont_256(ret, a, BLS12_381_r, r0);   }

void blst_fr_from_scalar(vec256 ret, const pow256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if ((uptr_t)ret == (uptr_t)a && is_endian.little) {
        mul_mont_sparse_256(ret, (const limb_t *)a, BLS12_381_rRR,
                                                    BLS12_381_r, r0);
    } else {
        vec256 out;
        limbs_from_le_bytes(out, a, 32);
        mul_mont_sparse_256(ret, out, BLS12_381_rRR, BLS12_381_r, r0);
        vec_zero(out, sizeof(out));
    }
}

void blst_scalar_from_fr(pow256 ret, const vec256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if ((uptr_t)ret == (uptr_t)a && is_endian.little) {
        from_mont_256((limb_t *)ret, a, BLS12_381_r, r0);
    } else {
        vec256 out;
        from_mont_256(out, a, BLS12_381_r, r0);
        le_bytes_from_limbs(ret, out, 32);
        vec_zero(out, sizeof(out));
    }
}

int blst_scalar_fr_check(const pow256 a)
{   return (int)(check_mod_256(a, BLS12_381_r) |
                 bytes_are_zero(a, sizeof(pow256)));
}

int blst_sk_check(const pow256 a)
{   return (int)check_mod_256(a, BLS12_381_r);   }

int blst_sk_add_n_check(pow256 ret, const pow256 a, const pow256 b)
{   return (int)add_n_check_mod_256(ret, a, b, BLS12_381_r);   }

int blst_sk_sub_n_check(pow256 ret, const pow256 a, const pow256 b)
{   return (int)sub_n_check_mod_256(ret, a, b, BLS12_381_r);   }

int blst_sk_mul_n_check(pow256 ret, const pow256 a, const pow256 b)
{
    vec256 a_fr, b_fr;
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (((size_t)a|(size_t)b)%sizeof(limb_t) != 0 || !is_endian.little) {
        limbs_from_le_bytes(a_fr, a, sizeof(a_fr));
        limbs_from_le_bytes(b_fr, b, sizeof(a_fr));
        a = (const byte *)a_fr;
        b = (const byte *)b_fr;
    }
    mul_mont_sparse_256(a_fr, (const limb_t *)a, BLS12_381_rRR,
                                                 BLS12_381_r, r0);
    mul_mont_sparse_256(b_fr, (const limb_t *)b, BLS12_381_rRR,
                                                 BLS12_381_r, r0);
    mul_mont_sparse_256(a_fr, a_fr, b_fr, BLS12_381_r, r0);
    from_mont_256(a_fr, a_fr, BLS12_381_r, r0);
    le_bytes_from_limbs(ret, a_fr, sizeof(a_fr));

    return (int)(vec_is_zero(a_fr, sizeof(a_fr)) ^ 1);
}

void blst_sk_inverse(pow256 ret, const pow256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (((size_t)a|(size_t)ret)%sizeof(limb_t) == 0 && is_endian.little) {
        limb_t *out = (limb_t *)ret;
        mul_mont_sparse_256(out, (const limb_t *)a, BLS12_381_rRR,
                                                    BLS12_381_r, r0);
        reciprocal_fr(out, out);
        from_mont_256(out, out, BLS12_381_r, r0);
    } else {
        vec256 out;
        limbs_from_le_bytes(out, a, 32);
        mul_mont_sparse_256(out, out, BLS12_381_rRR, BLS12_381_r, r0);
        reciprocal_fr(out, out);
        from_mont_256(out, out, BLS12_381_r, r0);
        le_bytes_from_limbs(ret, out, 32);
        vec_zero(out, sizeof(out));
    }
}

/*
 * BLS12-381-specifc Fp shortcuts to assembly.
 */
void blst_fp_add(vec384 ret, const vec384 a, const vec384 b)
{   add_fp(ret, a, b);   }

void blst_fp_sub(vec384 ret, const vec384 a, const vec384 b)
{   sub_fp(ret, a, b);   }

void blst_fp_mul_by_3(vec384 ret, const vec384 a)
{   mul_by_3_fp(ret, a);   }

void blst_fp_mul_by_8(vec384 ret, const vec384 a)
{   mul_by_8_fp(ret, a);   }

void blst_fp_lshift(vec384 ret, const vec384 a, size_t count)
{   lshift_fp(ret, a, count);   }

void blst_fp_mul(vec384 ret, const vec384 a, const vec384 b)
{   mul_fp(ret, a, b);   }

void blst_fp_sqr(vec384 ret, const vec384 a)
{   sqr_fp(ret, a);   }

void blst_fp_cneg(vec384 ret, const vec384 a, int flag)
{   cneg_fp(ret, a, is_zero(flag) ^ 1);   }

void blst_fp_to(vec384 ret, const vec384 a)
{   mul_fp(ret, a, BLS12_381_RR);   }

void blst_fp_from(vec384 ret, const vec384 a)
{   from_fp(ret, a);   }

/*
 * Fp serialization/deserialization.
 */
void blst_fp_from_uint32(vec384 ret, const unsigned int a[12])
{
    if (sizeof(limb_t) == 8) {
        int i;
        for (i = 0; i < 6; i++)
            ret[i] = a[2*i] | ((limb_t)a[2*i+1] << (32 & (8*sizeof(limb_t)-1)));
        a = (const unsigned int *)ret;
    }
    mul_fp(ret, (const limb_t *)a, BLS12_381_RR);
}

void blst_uint32_from_fp(unsigned int ret[12], const vec384 a)
{
    if (sizeof(limb_t) == 4) {
        from_fp((limb_t *)ret, a);
    } else {
        vec384 out;
        int i;

        from_fp(out, a);
        for (i = 0; i < 6; i++) {
            limb_t limb = out[i];
            ret[2*i]   = (unsigned int)limb;
            ret[2*i+1] = (unsigned int)(limb >> (32 & (8*sizeof(limb_t)-1)));
        }
    }
}

void blst_fp_from_uint64(vec384 ret, const unsigned long long a[6])
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (sizeof(limb_t) == 4 && !is_endian.little) {
        int i;
        for (i = 0; i < 6; i++) {
            unsigned long long limb = a[i];
            ret[2*i]   = (limb_t)limb;
            ret[2*i+1] = (limb_t)(limb >> 32);
        }
        a = (const unsigned long long *)ret;
    }
    mul_fp(ret, (const limb_t *)a, BLS12_381_RR);
}

void blst_uint64_from_fp(unsigned long long ret[6], const vec384 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (sizeof(limb_t) == 8 || is_endian.little) {
        from_fp((limb_t *)ret, a);
    } else {
        vec384 out;
        int i;

        from_fp(out, a);
        for (i = 0; i < 6; i++)
            ret[i] = out[2*i] | ((unsigned long long)out[2*i+1] << 32);
    }
}

void blst_fp_from_bendian(vec384 ret, const unsigned char a[48])
{
    vec384 out;

    limbs_from_be_bytes(out, a, sizeof(vec384));
    mul_fp(ret, out, BLS12_381_RR);
}

void blst_bendian_from_fp(unsigned char ret[48], const vec384 a)
{
    vec384 out;

    from_fp(out, a);
    be_bytes_from_limbs(ret, out, sizeof(vec384));
}

void blst_fp_from_lendian(vec384 ret, const unsigned char a[48])
{
    vec384 out;

    limbs_from_le_bytes(out, a, sizeof(vec384));
    mul_fp(ret, out, BLS12_381_RR);
}

void blst_lendian_from_fp(unsigned char ret[48], const vec384 a)
{
    vec384 out;

    from_fp(out, a);
    le_bytes_from_limbs(ret, out, sizeof(vec384));
}

/*
 * BLS12-381-specifc Fp2 shortcuts to assembly.
 */
void blst_fp2_add(vec384x ret, const vec384x a, const vec384x b)
{   add_fp2(ret, a, b);   }

void blst_fp2_sub(vec384x ret, const vec384x a, const vec384x b)
{   sub_fp2(ret, a, b);   }

void blst_fp2_mul_by_3(vec384x ret, const vec384x a)
{   mul_by_3_fp2(ret, a);   }

void blst_fp2_mul_by_8(vec384x ret, const vec384x a)
{   mul_by_8_fp2(ret, a);   }

void blst_fp2_lshift(vec384x ret, const vec384x a, size_t count)
{   lshift_fp2(ret, a, count);    }

void blst_fp2_mul(vec384x ret, const vec384x a, const vec384x b)
{   mul_fp2(ret, a, b);   }

void blst_fp2_sqr(vec384x ret, const vec384x a)
{   sqr_fp2(ret, a);   }

void blst_fp2_cneg(vec384x ret, const vec384x a, int flag)
{   cneg_fp2(ret, a, is_zero(flag) ^ 1);   }

/*
 * Scalar serialization/deseriazation
 */
void blst_scalar_from_uint32(pow256 ret, const unsigned int a[8])
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };
    size_t i;

    if ((uptr_t)ret==(uptr_t)a && is_endian.little)
        return;

    for(i = 0; i < 8; i++) {
        unsigned int w = a[i];
        *ret++ = (byte)w;
        *ret++ = (byte)(w >> 8);
        *ret++ = (byte)(w >> 16);
        *ret++ = (byte)(w >> 24);
    }
}

void blst_uint32_from_scalar(unsigned int ret[8], const pow256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };
    size_t i;

    if ((uptr_t)ret==(uptr_t)a && is_endian.little)
        return;

    for(i = 0; i < 8; i++) {
        unsigned int w = (unsigned int)(*a++);
        w |= (unsigned int)(*a++) << 8;
        w |= (unsigned int)(*a++) << 16;
        w |= (unsigned int)(*a++) << 24;
        ret[i] = w;
    }
}

void blst_scalar_from_uint64(pow256 ret, const unsigned long long a[4])
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };
    size_t i;

    if ((uptr_t)ret==(uptr_t)a && is_endian.little)
        return;

    for(i = 0; i < 4; i++) {
        unsigned long long w = a[i];
        *ret++ = (byte)w;
        *ret++ = (byte)(w >> 8);
        *ret++ = (byte)(w >> 16);
        *ret++ = (byte)(w >> 24);
        *ret++ = (byte)(w >> 32);
        *ret++ = (byte)(w >> 40);
        *ret++ = (byte)(w >> 48);
        *ret++ = (byte)(w >> 56);
    }
}

void blst_uint64_from_scalar(unsigned long long ret[4], const pow256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };
    size_t i;

    if ((uptr_t)ret==(uptr_t)a && is_endian.little)
        return;

    for(i = 0; i < 4; i++) {
        unsigned long long w = (unsigned long long)(*a++);
        w |= (unsigned long long)(*a++) << 8;
        w |= (unsigned long long)(*a++) << 16;
        w |= (unsigned long long)(*a++) << 24;
        w |= (unsigned long long)(*a++) << 32;
        w |= (unsigned long long)(*a++) << 40;
        w |= (unsigned long long)(*a++) << 48;
        w |= (unsigned long long)(*a++) << 56;
        ret[i] = w;
    }
}

void blst_scalar_from_bendian(pow256 ret, const unsigned char a[32])
{
    vec256 out;
    limbs_from_be_bytes(out, a, sizeof(out));
    le_bytes_from_limbs(ret, out, sizeof(out));
    vec_zero(out, sizeof(out));
}

void blst_bendian_from_scalar(unsigned char ret[32], const pow256 a)
{
    vec256 out;
    limbs_from_le_bytes(out, a, sizeof(out));
    be_bytes_from_limbs(ret, out, sizeof(out));
    vec_zero(out, sizeof(out));
}

void blst_scalar_from_lendian(pow256 ret, const unsigned char a[32])
{
    size_t i;

    if ((uptr_t)ret==(uptr_t)a)
        return;

    for (i = 0; i < 32; i++)
        ret[i] = a[i];
}

void blst_lendian_from_scalar(unsigned char ret[32], const pow256 a)
{
    size_t i;

    if ((uptr_t)ret==(uptr_t)a)
        return;

    for (i = 0; i < 32; i++)
        ret[i] = a[i];
}

void blst_fr_from_uint64(vec256 ret, const unsigned long long a[4])
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (sizeof(limb_t) == 4 && !is_endian.little) {
        int i;
        for (i = 0; i < 4; i++) {
            unsigned long long limb = a[i];
            ret[2*i]   = (limb_t)limb;
            ret[2*i+1] = (limb_t)(limb >> 32);
        }
        a = (const unsigned long long *)ret;
    }
    mul_mont_sparse_256(ret, (const limb_t *)a, BLS12_381_rRR, BLS12_381_r, r0);
}

void blst_uint64_from_fr(unsigned long long ret[4], const vec256 a)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };

    if (sizeof(limb_t) == 8 || is_endian.little) {
        from_mont_256((limb_t *)ret, a, BLS12_381_r, r0);
    } else {
        vec256 out;
        int i;

        from_mont_256(out, a, BLS12_381_r, r0);
        for (i = 0; i < 4; i++)
            ret[i] = out[2*i] | ((unsigned long long)out[2*i+1] << 32);
        vec_zero(out, sizeof(out));
    }
}

int blst_scalar_from_le_bytes(pow256 out, const unsigned char *bytes, size_t n)
{
    struct { vec256 out, digit, radix; } t;
    limb_t ret;

    vec_zero(t.out, sizeof(t.out));
    vec_copy(t.radix, BLS12_381_rRR, sizeof(t.radix));

    while (n > 32) {
        limbs_from_le_bytes(t.digit, bytes, 32);
        from_mont_256(t.digit, t.digit, BLS12_381_r, r0);
        mul_mont_sparse_256(t.digit, t.digit, t.radix, BLS12_381_r, r0);
        add_mod_256(t.out, t.out, t.digit, BLS12_381_r);
        mul_mont_sparse_256(t.radix, t.radix, BLS12_381_rRR, BLS12_381_r, r0);
        bytes += 32;
        n -= 32;
    }

    vec_zero(t.digit, sizeof(t.digit));
    limbs_from_le_bytes(t.digit, bytes, n);
    from_mont_256(t.digit, t.digit, BLS12_381_r, r0);
    mul_mont_sparse_256(t.digit, t.digit, t.radix, BLS12_381_r, r0);
    add_mod_256(t.out, t.out, t.digit, BLS12_381_r);

    ret = vec_is_zero(t.out, sizeof(t.out));
    le_bytes_from_limbs(out, t.out, 32);
    vec_zero(t.out, 2*sizeof(t.out));

    return (int)(ret^1);
}

int blst_scalar_from_be_bytes(pow256 out, const unsigned char *bytes, size_t n)
{
    struct { vec256 out, digit, radix; } t;
    limb_t ret;

    vec_zero(t.out, sizeof(t.out));
    vec_copy(t.radix, BLS12_381_rRR, sizeof(t.radix));

    bytes += n;
    while (n > 32) {
        limbs_from_be_bytes(t.digit, bytes -= 32, 32);
        from_mont_256(t.digit, t.digit, BLS12_381_r, r0);
        mul_mont_sparse_256(t.digit, t.digit, t.radix, BLS12_381_r, r0);
        add_mod_256(t.out, t.out, t.digit, BLS12_381_r);
        mul_mont_sparse_256(t.radix, t.radix, BLS12_381_rRR, BLS12_381_r, r0);
        n -= 32;
    }

    vec_zero(t.digit, sizeof(t.digit));
    limbs_from_be_bytes(t.digit, bytes -= n, n);
    from_mont_256(t.digit, t.digit, BLS12_381_r, r0);
    mul_mont_sparse_256(t.digit, t.digit, t.radix, BLS12_381_r, r0);
    add_mod_256(t.out, t.out, t.digit, BLS12_381_r);

    ret = vec_is_zero(t.out, sizeof(t.out));
    le_bytes_from_limbs(out, t.out, 32);
    vec_zero(t.out, 2*sizeof(t.out));

    return (int)(ret^1);
}

/*
 * Test facilitator
 */
static unsigned char nibble(char c)
{
    if (c >= '0' && c <= '9')
        return c - '0';
    else if (c >= 'a' && c <= 'f')
        return 10 + c - 'a';
    else if (c >= 'A' && c <= 'F')
        return 10 + c - 'A';
    else
        return 16;
}

static void limbs_from_hexascii(limb_t *ret, size_t sz, const char *hex)
{
    size_t len;
    limb_t limb = 0;

    if (hex[0]=='0' && (hex[1]=='x' || hex[1]=='X'))
        hex += 2;

    for (len = 0; len<2*sz && nibble(hex[len])<16; len++) ;

    vec_zero(ret, sz);

    while(len--) {
        limb <<= 4;
        limb |= nibble(*hex++);
        if (len % (2*sizeof(limb_t)) == 0)
            ret[len / (2*sizeof(limb_t))] = limb;
    }
}

void blst_scalar_from_hexascii(vec256 ret, const char *hex)
{   limbs_from_hexascii(ret, sizeof(vec256), hex);   }

void blst_fp_from_hexascii(vec384 ret, const char *hex)
{
    limbs_from_hexascii(ret, sizeof(vec384), hex);
    mul_fp(ret, ret, BLS12_381_RR);
}
