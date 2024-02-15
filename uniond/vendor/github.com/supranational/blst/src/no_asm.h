/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#if LIMB_T_BITS==32
typedef unsigned long long llimb_t;
#endif

#if !defined(__STDC_VERSION__) || __STDC_VERSION__<199901 || defined(__STDC_NO_VLA__)
# error "unsupported compiler"
#endif

#if defined(__clang__)
# pragma GCC diagnostic ignored "-Wstatic-in-inline"
#endif

#if !defined(__clang__) && !defined(__builtin_assume)
# if defined(__GNUC__) && __GNUC__>=5
#  define __builtin_assume(condition) if (!(condition)) __builtin_unreachable()
# elif defined(_MSC_VER)
#  define __builtin_assume(condition) __assume(condition)
# else
#  define __builtin_assume(condition) (void)(condition)
# endif
#endif

static void mul_mont_n(limb_t ret[], const limb_t a[], const limb_t b[],
                       const limb_t p[], limb_t n0, size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t mask, borrow, mx, hi, tmp[n+1], carry;
    size_t i, j;

    for (mx=b[0], hi=0, i=0; i<n; i++) {
        limbx = (mx * (llimb_t)a[i]) + hi;
        tmp[i] = (limb_t)limbx;
        hi = (limb_t)(limbx >> LIMB_T_BITS);
    }
    mx = n0*tmp[0];
    tmp[i] = hi;

    for (carry=0, j=0; ; ) {
        limbx = (mx * (llimb_t)p[0]) + tmp[0];
        hi = (limb_t)(limbx >> LIMB_T_BITS);
        for (i=1; i<n; i++) {
            limbx = (mx * (llimb_t)p[i] + hi) + tmp[i];
            tmp[i-1] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        limbx = tmp[i] + (hi + (llimb_t)carry);
        tmp[i-1] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);

        if (++j==n)
            break;

        for (mx=b[j], hi=0, i=0; i<n; i++) {
            limbx = (mx * (llimb_t)a[i] + hi) + tmp[i];
            tmp[i] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        mx = n0*tmp[0];
        limbx = hi + (llimb_t)carry;
        tmp[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = carry - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);
}

#define MUL_MONT_IMPL(bits) \
inline void mul_mont_##bits(vec##bits ret, const vec##bits a, \
                            const vec##bits b, const vec##bits p, limb_t n0) \
{   mul_mont_n(ret, a, b, p, n0, NLIMBS(bits));   } \
\
inline void sqr_mont_##bits(vec##bits ret, const vec##bits a, \
                            const vec##bits p, limb_t n0) \
{   mul_mont_n(ret, a, a, p, n0, NLIMBS(bits));   }

/*
 * 256-bit subroutines can handle arbitrary modulus, even non-"sparse",
 * but we have to harmonize the naming with assembly.
 */
#define mul_mont_256 mul_mont_sparse_256
#define sqr_mont_256 sqr_mont_sparse_256
MUL_MONT_IMPL(256)
#undef mul_mont_256
#undef sqr_mont_256
MUL_MONT_IMPL(384)

static void add_mod_n(limb_t ret[], const limb_t a[], const limb_t b[],
                      const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t mask, carry, borrow, tmp[n];
    size_t i;

    for (carry=0, i=0; i<n; i++) {
        limbx = a[i] + (b[i] + (llimb_t)carry);
        tmp[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = carry - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);
}

#define ADD_MOD_IMPL(bits) \
inline void add_mod_##bits(vec##bits ret, const vec##bits a, \
                           const vec##bits b, const vec##bits p) \
{   add_mod_n(ret, a, b, p, NLIMBS(bits));   }

ADD_MOD_IMPL(256)
ADD_MOD_IMPL(384)

static void sub_mod_n(limb_t ret[], const limb_t a[], const limb_t b[],
                      const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t mask, carry, borrow;
    size_t i;

    for (borrow=0, i=0; i<n; i++) {
        limbx = a[i] - (b[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = 0 - borrow;
    launder(mask);

    for (carry=0, i=0; i<n; i++) {
        limbx = ret[i] + ((p[i] & mask) + (llimb_t)carry);
        ret[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }
}

#define SUB_MOD_IMPL(bits) \
inline void sub_mod_##bits(vec##bits ret, const vec##bits a, \
                           const vec##bits b, const vec##bits p) \
{   sub_mod_n(ret, a, b, p, NLIMBS(bits));   }

SUB_MOD_IMPL(256)
SUB_MOD_IMPL(384)

static void mul_by_3_mod_n(limb_t ret[], const limb_t a[], const limb_t p[],
                           size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t mask, carry, borrow, tmp[n], two_a[n];
    size_t i;

    for (carry=0, i=0; i<n; i++) {
        limb_t a_i = a[i];
        tmp[i] = a_i<<1 | carry;
        carry = a_i>>(LIMB_T_BITS-1);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        two_a[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = carry - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        two_a[i] = (two_a[i] & ~mask) | (tmp[i] & mask);

    for (carry=0, i=0; i<n; i++) {
        limbx = a[i] + (two_a[i] + (llimb_t)carry);
        tmp[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = carry - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);
}

#define MUL_BY_3_MOD_IMPL(bits) \
inline void mul_by_3_mod_##bits(vec##bits ret, const vec##bits a, \
                                const vec##bits p) \
{   mul_by_3_mod_n(ret, a, p, NLIMBS(bits));   }

MUL_BY_3_MOD_IMPL(256)
MUL_BY_3_MOD_IMPL(384)

static void lshift_mod_n(limb_t ret[], const limb_t a[], size_t count,
                         const limb_t p[], size_t n)
{
    __builtin_assume(count != 0);
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t mask, carry, borrow, tmp[n];
    size_t i;

    while (count--) {
        for (carry=0, i=0; i<n; i++) {
            limb_t a_i = a[i];
            tmp[i] = a_i<<1 | carry;
            carry = a_i>>(LIMB_T_BITS-1);
        }

        for (borrow=0, i=0; i<n; i++) {
            limbx = tmp[i] - (p[i] + (llimb_t)borrow);
            ret[i] = (limb_t)limbx;
            borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
        }

        mask = carry - borrow;
        launder(mask);

        for(i=0; i<n; i++)
            ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);

        a = ret;
    }
}

#define LSHIFT_MOD_IMPL(bits) \
inline void lshift_mod_##bits(vec##bits ret, const vec##bits a, size_t count, \
                              const vec##bits p) \
{   lshift_mod_n(ret, a, count, p, NLIMBS(bits));   }

LSHIFT_MOD_IMPL(256)
LSHIFT_MOD_IMPL(384)

static void cneg_mod_n(limb_t ret[], const limb_t a[], bool_t flag,
                       const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t borrow, mask, tmp[n];
    size_t i;

    for (borrow=0, i=0; i<n; i++) {
        limbx = p[i] - (a[i] + (llimb_t)borrow);
        tmp[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    flag &= vec_is_zero(a, sizeof(tmp)) ^ 1;
    mask = (limb_t)0 - flag;

    for(i=0; i<n; i++)
        ret[i] = (a[i] & ~mask) | (tmp[i] & mask);
}

#define CNEG_MOD_IMPL(bits) \
inline void cneg_mod_##bits(vec##bits ret, const vec##bits a, bool_t flag, \
                            const vec##bits p) \
{   cneg_mod_n(ret, a, flag, p, NLIMBS(bits));   }

CNEG_MOD_IMPL(256)
CNEG_MOD_IMPL(384)

static limb_t check_mod_n(const byte a[], const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t borrow, ai, acc;
    size_t i, j;

    for (acc=borrow=0, i=0; i<n; i++) {
        for (ai=0, j=0; j<8*sizeof(limb_t); j+=8)
            ai |= (limb_t)(*a++) << j;
        acc |= ai;
        limbx = ai - (p[i] + (llimb_t)borrow);
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    return borrow & (is_zero(acc) ^ 1);
}

#define CHECK_MOD_IMPL(bits) \
inline limb_t check_mod_##bits(const pow##bits a, const vec##bits p) \
{   return check_mod_n(a, p, NLIMBS(bits));   }

CHECK_MOD_IMPL(256)

static limb_t add_n_check_mod_n(byte ret[], const byte a[], const byte b[],
                                            const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    limb_t ret_[n], a_[n], b_[n], zero;

    limbs_from_le_bytes(a_, a, sizeof(a_));
    limbs_from_le_bytes(b_, b, sizeof(b_));

    add_mod_n(ret_, a_, b_, p, n);
    zero = vec_is_zero(ret_, sizeof(ret_));

    le_bytes_from_limbs(ret, ret_, sizeof(ret_));

    return zero^1;
}

#define ADD_N_CHECK_MOD_IMPL(bits) \
inline limb_t add_n_check_mod_##bits(pow##bits ret, const pow##bits a, \
                                     const pow##bits b, const vec##bits p) \
{   return add_n_check_mod_n(ret, a, b, p, NLIMBS(bits));   }

ADD_N_CHECK_MOD_IMPL(256)

static limb_t sub_n_check_mod_n(byte ret[], const byte a[], const byte b[],
                                            const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    limb_t ret_[n], a_[n], b_[n], zero;

    limbs_from_le_bytes(a_, a, sizeof(a_));
    limbs_from_le_bytes(b_, b, sizeof(b_));

    sub_mod_n(ret_, a_, b_, p, n);
    zero = vec_is_zero(ret_, sizeof(ret_));

    le_bytes_from_limbs(ret, ret_, sizeof(ret_));

    return zero^1;
}

#define SUB_N_CHECK_MOD_IMPL(bits) \
inline limb_t sub_n_check_mod_##bits(pow##bits ret, const pow##bits a, \
                                     const pow##bits b, const vec##bits p) \
{   return sub_n_check_mod_n(ret, a, b, p, NLIMBS(bits));   }

SUB_N_CHECK_MOD_IMPL(256)

static void from_mont_n(limb_t ret[], const limb_t a[],
                        const limb_t p[], limb_t n0, size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t mask, borrow, mx, hi, tmp[n];
    size_t i, j;

    for (j=0; j<n; j++) {
        mx = n0*a[0];
        limbx = (mx * (llimb_t)p[0]) + a[0];
        hi = (limb_t)(limbx >> LIMB_T_BITS);
        for (i=1; i<n; i++) {
            limbx = (mx * (llimb_t)p[i] + hi) + a[i];
            tmp[i-1] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        tmp[i-1] = hi;
        a = tmp;
    }

    /* this is needed only if input can be non-fully-reduced */
    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = 0 - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);
}

#define FROM_MONT_IMPL(bits) \
inline void from_mont_##bits(vec##bits ret, const vec##bits a, \
                             const vec##bits p, limb_t n0) \
{   from_mont_n(ret, a, p, n0, NLIMBS(bits));   }

FROM_MONT_IMPL(256)
FROM_MONT_IMPL(384)

static void redc_mont_n(limb_t ret[], const limb_t a[],
                        const limb_t p[], limb_t n0, size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t mask, carry, borrow, mx, hi, tmp[n];
    const limb_t *b = a;
    size_t i, j;

    for (j=0; j<n; j++) {
        mx = n0*b[0];
        limbx = (mx * (llimb_t)p[0]) + b[0];
        hi = (limb_t)(limbx >> LIMB_T_BITS);
        for (i=1; i<n; i++) {
            limbx = (mx * (llimb_t)p[i] + hi) + b[i];
            tmp[i-1] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        tmp[i-1] = hi;
        b = tmp;
    }

    for (carry=0, i=0; i<n; i++) {
        limbx = a[n+i] + (tmp[i] + (llimb_t)carry);
        tmp[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        ret[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = carry - borrow;
    launder(mask);

    for(i=0; i<n; i++)
        ret[i] = (ret[i] & ~mask) | (tmp[i] & mask);
}

#define REDC_MONT_IMPL(bits, bits2) \
inline void redc_mont_##bits(vec##bits ret, const vec##bits2 a, \
                             const vec##bits p, limb_t n0) \
{   redc_mont_n(ret, a, p, n0, NLIMBS(bits));   }

REDC_MONT_IMPL(256, 512)
REDC_MONT_IMPL(384, 768)

static void rshift_mod_n(limb_t ret[], const limb_t a[], size_t count,
                         const limb_t p[], size_t n)
{
    __builtin_assume(count != 0);
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t mask, carry, limb, next;
    size_t i;

    while (count--) {
        mask = 0 - (a[0] & 1);
        launder(mask);
        for (carry=0, i=0; i<n; i++) {
            limbx = a[i] + ((p[i]&mask) + (llimb_t)carry);
            ret[i] = (limb_t)limbx;
            carry = (limb_t)(limbx >> LIMB_T_BITS);
        }

        for (next=ret[0], i=0; i<n-1; i++) {
            limb = next >> 1;
            next = ret[i+1];
            ret[i] = limb | next << (LIMB_T_BITS-1);
        }
        ret[i] = next >> 1 | carry << (LIMB_T_BITS-1);

        a = ret;
    }
}

#define RSHIFT_MOD_IMPL(bits) \
inline void rshift_mod_##bits(vec##bits ret, const vec##bits a, size_t count, \
                              const vec##bits p) \
{   rshift_mod_n(ret, a, count, p, NLIMBS(bits));   }

RSHIFT_MOD_IMPL(256)
RSHIFT_MOD_IMPL(384)

#define DIV_BY_2_MOD_IMPL(bits) \
inline void div_by_2_mod_##bits(vec##bits ret, const vec##bits a, \
                                const vec##bits p) \
{   rshift_mod_n(ret, a, 1, p, NLIMBS(bits));   }

DIV_BY_2_MOD_IMPL(384)

static limb_t sgn0_pty_mod_n(const limb_t a[], const limb_t p[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t carry, borrow, ret, tmp[n];
    size_t i;

    ret = a[0] & 1; /* parity */

    for (carry=0, i=0; i<n; i++) {
        limb_t a_i = a[i];
        tmp[i] = a_i<<1 | carry;
        carry = a_i>>(LIMB_T_BITS-1);
    }

    for (borrow=0, i=0; i<n; i++) {
        limbx = tmp[i] - (p[i] + (llimb_t)borrow);
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    ret |= ((carry - borrow) & 2) ^ 2;

    return ret;
}

inline limb_t sgn0_pty_mod_384(const vec384 a, const vec384 p)
{   return sgn0_pty_mod_n(a, p, NLIMBS(384));   }

inline limb_t sgn0_pty_mont_384(const vec384 a, const vec384 p, limb_t n0)
{
    vec384 tmp;

    from_mont_n(tmp, a, p, n0, NLIMBS(384));

    return sgn0_pty_mod_n(tmp, p, NLIMBS(384));
}

inline limb_t sgn0_pty_mod_384x(const vec384x a, const vec384 p)
{
    limb_t re, im, sign, prty;

    re = sgn0_pty_mod_n(a[0], p, NLIMBS(384));
    im = sgn0_pty_mod_n(a[1], p, NLIMBS(384));

    /* a->im!=0 ? sgn0(a->im) : sgn0(a->re) */
    sign = (limb_t)0 - vec_is_zero(a[1], sizeof(vec384));
    sign = (re & sign) | (im & ~sign);

    /* a->re==0 ? prty(a->im) : prty(a->re) */
    prty = (limb_t)0 - vec_is_zero(a[0], sizeof(vec384));
    prty = (im & prty) | (re & ~prty);

    return (sign & 2) | (prty & 1);
}

inline limb_t sgn0_pty_mont_384x(const vec384x a, const vec384 p, limb_t n0)
{
    vec384x tmp;

    from_mont_n(tmp[0], a[0], p, n0, NLIMBS(384));
    from_mont_n(tmp[1], a[1], p, n0, NLIMBS(384));

    return sgn0_pty_mod_384x(tmp, p);
}

void mul_mont_384x(vec384x ret, const vec384x a, const vec384x b,
                          const vec384 p, limb_t n0)
{
    vec384 aa, bb, cc;

    add_mod_n(aa, a[0], a[1], p, NLIMBS(384));
    add_mod_n(bb, b[0], b[1], p, NLIMBS(384));
    mul_mont_n(bb, bb, aa, p, n0, NLIMBS(384));
    mul_mont_n(aa, a[0], b[0], p, n0, NLIMBS(384));
    mul_mont_n(cc, a[1], b[1], p, n0, NLIMBS(384));
    sub_mod_n(ret[0], aa, cc, p, NLIMBS(384));
    sub_mod_n(ret[1], bb, aa, p, NLIMBS(384));
    sub_mod_n(ret[1], ret[1], cc, p, NLIMBS(384));
}

/*
 * mul_mont_n without final conditional subtraction, which implies
 * that modulus is one bit short, which in turn means that there are
 * no carries to handle between iterations...
 */
static void mul_mont_nonred_n(limb_t ret[], const limb_t a[], const limb_t b[],
                              const limb_t p[], limb_t n0, size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t mx, hi, tmp[n+1];
    size_t i, j;

    for (mx=b[0], hi=0, i=0; i<n; i++) {
        limbx = (mx * (llimb_t)a[i]) + hi;
        tmp[i] = (limb_t)limbx;
        hi = (limb_t)(limbx >> LIMB_T_BITS);
    }
    mx = n0*tmp[0];
    tmp[i] = hi;

    for (j=0; ; ) {
        limbx = (mx * (llimb_t)p[0]) + tmp[0];
        hi = (limb_t)(limbx >> LIMB_T_BITS);
        for (i=1; i<n; i++) {
            limbx = (mx * (llimb_t)p[i] + hi) + tmp[i];
            tmp[i-1] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        tmp[i-1] = tmp[i] + hi;

        if (++j==n)
            break;

        for (mx=b[j], hi=0, i=0; i<n; i++) {
            limbx = (mx * (llimb_t)a[i] + hi) + tmp[i];
            tmp[i] = (limb_t)limbx;
            hi = (limb_t)(limbx >> LIMB_T_BITS);
        }
        mx = n0*tmp[0];
        tmp[i] = hi;
    }

    vec_copy(ret, tmp, sizeof(tmp)-sizeof(limb_t));
}

void sqr_n_mul_mont_383(vec384 ret, const vec384 a, size_t count,
                        const vec384 p, limb_t n0, const vec384 b)
{
    __builtin_assume(count != 0);
    while(count--) {
        mul_mont_nonred_n(ret, a, a, p, n0, NLIMBS(384));
        a = ret;
    }
    mul_mont_n(ret, ret, b, p, n0, NLIMBS(384));
}

void sqr_mont_382x(vec384x ret, const vec384x a,
                          const vec384 p, limb_t n0)
{
    llimb_t limbx;
    limb_t mask, carry, borrow;
    size_t i;
    vec384 t0, t1;

    /* "add_mod_n(t0, a[0], a[1], p, NLIMBS(384));" */
    for (carry=0, i=0; i<NLIMBS(384); i++) {
        limbx = a[0][i] + (a[1][i] + (llimb_t)carry);
        t0[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    /* "sub_mod_n(t1, a[0], a[1], p, NLIMBS(384));" */
    for (borrow=0, i=0; i<NLIMBS(384); i++) {
        limbx = a[0][i] - (a[1][i] + (llimb_t)borrow);
        t1[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }
    mask = 0 - borrow;
    launder(mask);

    /* "mul_mont_n(ret[1], a[0], a[1], p, n0, NLIMBS(384));" */
    mul_mont_nonred_n(ret[1], a[0], a[1], p, n0, NLIMBS(384));

    /* "add_mod_n(ret[1], ret[1], ret[1], p, NLIMBS(384));" */
    for (carry=0, i=0; i<NLIMBS(384); i++) {
        limb_t a_i = ret[1][i];
        ret[1][i] = a_i<<1 | carry;
        carry = a_i>>(LIMB_T_BITS-1);
    }

    /* "mul_mont_n(ret[0], t0, t1, p, n0, NLIMBS(384));" */
    mul_mont_nonred_n(ret[0], t0, t1, p, n0, NLIMBS(384));

    /* account for t1's sign... */
    for (borrow=0, i=0; i<NLIMBS(384); i++) {
        limbx = ret[0][i] - ((t0[i] & mask) + (llimb_t)borrow);
        ret[0][i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }
    mask = 0 - borrow;
    launder(mask);
    for (carry=0, i=0; i<NLIMBS(384); i++) {
        limbx = ret[0][i] + ((p[i] & mask) + (llimb_t)carry);
        ret[0][i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }
}

#if defined(__GNUC__) || defined(__clang__)
# define MSB(x) ({ limb_t ret = (x) >> (LIMB_T_BITS-1); launder(ret); ret; })
#else
# define MSB(x) ((x) >> (LIMB_T_BITS-1))
#endif

static size_t num_bits(limb_t l)
{
    limb_t x, mask;
    size_t bits = is_zero(l) ^ 1;

    if (sizeof(limb_t) == 8) {
        x = l >> (32 & (8*sizeof(limb_t)-1));
        mask = 0 - MSB(0 - x);
        bits += 32 & mask;
        l ^= (x ^ l) & mask;
    }

    x = l >> 16;
    mask = 0 - MSB(0 - x);
    bits += 16 & mask;
    l ^= (x ^ l) & mask;

    x = l >> 8;
    mask = 0 - MSB(0 - x);
    bits += 8 & mask;
    l ^= (x ^ l) & mask;

    x = l >> 4;
    mask = 0 - MSB(0 - x);
    bits += 4 & mask;
    l ^= (x ^ l) & mask;

    x = l >> 2;
    mask = 0 - MSB(0 - x);
    bits += 2 & mask;
    l ^= (x ^ l) & mask;

    bits += l >> 1;

    return bits;
}

#if defined(__clang_major__) && __clang_major__>7
__attribute__((optnone))
#endif
static limb_t lshift_2(limb_t hi, limb_t lo, size_t l)
{
    size_t r = LIMB_T_BITS - l;
    limb_t mask = 0 - (is_zero(l)^1);
    return (hi << (l&(LIMB_T_BITS-1))) | ((lo & mask) >> (r&(LIMB_T_BITS-1)));
}

/*
 * https://eprint.iacr.org/2020/972 with 'k' being LIMB_T_BITS-1.
 */
static void ab_approximation_n(limb_t a_[2], const limb_t a[],
                               limb_t b_[2], const limb_t b[], size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    limb_t a_hi, a_lo, b_hi, b_lo, mask;
    size_t i;

    i = n-1;
    a_hi = a[i],    a_lo = a[i-1];
    b_hi = b[i],    b_lo = b[i-1];
    for (i--; --i;) {
        mask = 0 - is_zero(a_hi | b_hi);
        a_hi = ((a_lo ^ a_hi) & mask) ^ a_hi;
        b_hi = ((b_lo ^ b_hi) & mask) ^ b_hi;
        a_lo = ((a[i] ^ a_lo) & mask) ^ a_lo;
        b_lo = ((b[i] ^ b_lo) & mask) ^ b_lo;
    }
    i = LIMB_T_BITS - num_bits(a_hi | b_hi);
    /* |i| can be LIMB_T_BITS if all a[2..]|b[2..] were zeros */

    a_[0] = a[0], a_[1] = lshift_2(a_hi, a_lo, i);
    b_[0] = b[0], b_[1] = lshift_2(b_hi, b_lo, i);
}

typedef struct { limb_t f0, g0, f1, g1; } factors;

static void inner_loop_n(factors *fg, const limb_t a_[2], const limb_t b_[2],
                         size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t f0 = 1, g0 = 0, f1 = 0, g1 = 1;
    limb_t a_lo, a_hi, b_lo, b_hi, t_lo, t_hi, odd, borrow, xorm;

    a_lo = a_[0], a_hi = a_[1];
    b_lo = b_[0], b_hi = b_[1];

    while(n--) {
        odd = 0 - (a_lo&1);

        /* a_ -= b_ if a_ is odd */
        t_lo = a_lo, t_hi = a_hi;
        limbx = a_lo - (llimb_t)(b_lo & odd);
        a_lo = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
        limbx = a_hi - ((llimb_t)(b_hi & odd) + borrow);
        a_hi = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS);

        /* negate a_-b_ if it borrowed */
        a_lo ^= borrow;
        a_hi ^= borrow;
        limbx = a_lo + (llimb_t)(borrow & 1);
        a_lo = (limb_t)limbx;
        a_hi += (limb_t)(limbx >> LIMB_T_BITS) & 1;

        /* b_=a_ if a_-b_ borrowed */
        b_lo = ((t_lo ^ b_lo) & borrow) ^ b_lo;
        b_hi = ((t_hi ^ b_hi) & borrow) ^ b_hi;

        /* exchange f0 and f1 if a_-b_ borrowed */
        xorm = (f0 ^ f1) & borrow;
        f0 ^= xorm;
        f1 ^= xorm;

        /* exchange g0 and g1 if a_-b_ borrowed */
        xorm = (g0 ^ g1) & borrow;
        g0 ^= xorm;
        g1 ^= xorm;

        /* subtract if a_ was odd */
        f0 -= f1 & odd;
        g0 -= g1 & odd;

        f1 <<= 1;
        g1 <<= 1;
        a_lo >>= 1; a_lo |= a_hi << (LIMB_T_BITS-1);
        a_hi >>= 1;
    }

    fg->f0 = f0, fg->g0 = g0, fg->f1 = f1, fg->g1= g1;
}

static limb_t cneg_n(limb_t ret[], const limb_t a[], limb_t neg, size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx = 0;
    limb_t carry;
    size_t i;

    for (carry=neg&1, i=0; i<n; i++) {
        limbx = (llimb_t)(a[i] ^ neg) + carry;
        ret[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    return 0 - MSB((limb_t)limbx);
}

static limb_t add_n(limb_t ret[], const limb_t a[], limb_t b[], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t carry;
    size_t i;

    for (carry=0, i=0; i<n; i++) {
        limbx = a[i] + (b[i] + (llimb_t)carry);
        ret[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }

    return carry;
}

static limb_t umul_n(limb_t ret[], const limb_t a[], limb_t b, size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t hi;
    size_t i;

    for (hi=0, i=0; i<n; i++) {
        limbx = (b * (llimb_t)a[i]) + hi;
        ret[i] = (limb_t)limbx;
        hi = (limb_t)(limbx >> LIMB_T_BITS);
    }

    return hi;
}

static limb_t smul_n_shift_n(limb_t ret[], const limb_t a[], limb_t *f_,
                                           const limb_t b[], limb_t *g_,
                                           size_t n)
{
    __builtin_assume(n != 0);
    limb_t a_[n+1], b_[n+1], f, g, neg, carry, hi;
    size_t i;

    /* |a|*|f_| */
    f = *f_;
    neg = 0 - MSB(f);
    f = (f ^ neg) - neg;            /* ensure |f| is positive */
    (void)cneg_n(a_, a, neg, n);
    hi = umul_n(a_, a_, f, n);
    a_[n] = hi - (f & neg);

    /* |b|*|g_| */
    g = *g_;
    neg = 0 - MSB(g);
    g = (g ^ neg) - neg;            /* ensure |g| is positive */
    (void)cneg_n(b_, b, neg, n);
    hi = umul_n(b_, b_, g, n);
    b_[n] = hi - (g & neg);

    /* |a|*|f_| + |b|*|g_| */
    (void)add_n(a_, a_, b_, n+1);

    /* (|a|*|f_| + |b|*|g_|) >> k */
    for (carry=a_[0], i=0; i<n; i++) {
        hi = carry >> (LIMB_T_BITS-2);
        carry = a_[i+1];
        ret[i] = hi | (carry << 2);
    }

    /* ensure result is non-negative, fix up |f_| and |g_| accordingly */
    neg = 0 - MSB(carry);
    *f_ = (*f_ ^ neg) - neg;
    *g_ = (*g_ ^ neg) - neg;
    (void)cneg_n(ret, ret, neg, n);

    return neg;
}

static limb_t smul_2n(limb_t ret[], const limb_t u[], limb_t f,
                                    const limb_t v[], limb_t g, size_t n)
{
    __builtin_assume(n != 0);
    limb_t u_[n], v_[n], neg, hi;

    /* |u|*|f_| */
    neg = 0 - MSB(f);
    f = (f ^ neg) - neg;            /* ensure |f| is positive */
    neg = cneg_n(u_, u, neg, n);
    hi = umul_n(u_, u_, f, n) - (f&neg);

    /* |v|*|g_| */
    neg = 0 - MSB(g);
    g = (g ^ neg) - neg;            /* ensure |g| is positive */
    neg = cneg_n(v_, v, neg, n);
    hi += umul_n(v_, v_, g, n) - (g&neg);

    /* |u|*|f_| + |v|*|g_| */
    hi += add_n(ret, u_, v_, n);

    return hi;
}

static void ct_inverse_mod_n(limb_t ret[], const limb_t inp[],
                             const limb_t mod[], const limb_t modx[], size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t a[n], b[n], u[2*n], v[2*n], t[2*n];
    limb_t a_[2], b_[2], sign, carry, top;
    factors fg;
    size_t i;

    vec_copy(a, inp, sizeof(a));
    vec_copy(b, mod, sizeof(b));
    vec_zero(u, sizeof(u)); u[0] = 1;
    vec_zero(v, sizeof(v));

    for (i=0; i<(2*n*LIMB_T_BITS)/(LIMB_T_BITS-2); i++) {
        ab_approximation_n(a_, a, b_, b, n);
        inner_loop_n(&fg, a_, b_, LIMB_T_BITS-2);
        (void)smul_n_shift_n(t, a, &fg.f0, b, &fg.g0, n);
        (void)smul_n_shift_n(b, a, &fg.f1, b, &fg.g1, n);
        vec_copy(a, t, sizeof(a));
        smul_2n(t, u, fg.f0, v, fg.g0, 2*n);
        smul_2n(v, u, fg.f1, v, fg.g1, 2*n);
        vec_copy(u, t, sizeof(u));
    }

    inner_loop_n(&fg, a, b, (2*n*LIMB_T_BITS)%(LIMB_T_BITS-2));
    top = smul_2n(ret, u, fg.f1, v, fg.g1, 2*n);

    sign = 0 - MSB(top);    /* top is 1, 0 or -1 */
    for (carry=0, i=0; i<n; i++) {
        limbx = ret[n+i] + ((modx[i] & sign) + (llimb_t)carry);
        ret[n+i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }
    top += carry;
    sign = 0 - top;         /* top is 1, 0 or -1 */
    top |= sign;
    for (i=0; i<n; i++)
        a[i] = modx[i] & top;
    (void)cneg_n(a, a, 0 - MSB(sign), n);
    add_n(ret+n, ret+n, a, n);
}

#define CT_INVERSE_MOD_IMPL(bits, bits2) \
inline void ct_inverse_mod_##bits(vec##bits2 ret, const vec##bits inp, \
                                  const vec##bits mod, const vec##bits modx) \
{   ct_inverse_mod_n(ret, inp, mod, modx, NLIMBS(bits));   }

CT_INVERSE_MOD_IMPL(256, 512)
CT_INVERSE_MOD_IMPL(384, 768)

/*
 * Copy of inner_loop_n above, but with |L| updates.
 */
static limb_t legendre_loop_n(limb_t L, factors *fg, const limb_t a_[2],
                              const limb_t b_[2], size_t n)
{
    __builtin_assume(n != 0);
    llimb_t limbx;
    limb_t f0 = 1, g0 = 0, f1 = 0, g1 = 1;
    limb_t a_lo, a_hi, b_lo, b_hi, t_lo, t_hi, odd, borrow, xorm;

    a_lo = a_[0], a_hi = a_[1];
    b_lo = b_[0], b_hi = b_[1];

    while(n--) {
        odd = 0 - (a_lo&1);

        /* a_ -= b_ if a_ is odd */
        t_lo = a_lo, t_hi = a_hi;
        limbx = a_lo - (llimb_t)(b_lo & odd);
        a_lo = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
        limbx = a_hi - ((llimb_t)(b_hi & odd) + borrow);
        a_hi = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS);

        L += ((t_lo & b_lo) >> 1) & borrow;

        /* negate a_-b_ if it borrowed */
        a_lo ^= borrow;
        a_hi ^= borrow;
        limbx = a_lo + (llimb_t)(borrow & 1);
        a_lo = (limb_t)limbx;
        a_hi += (limb_t)(limbx >> LIMB_T_BITS) & 1;

        /* b_=a_ if a_-b_ borrowed */
        b_lo = ((t_lo ^ b_lo) & borrow) ^ b_lo;
        b_hi = ((t_hi ^ b_hi) & borrow) ^ b_hi;

        /* exchange f0 and f1 if a_-b_ borrowed */
        xorm = (f0 ^ f1) & borrow;
        f0 ^= xorm;
        f1 ^= xorm;

        /* exchange g0 and g1 if a_-b_ borrowed */
        xorm = (g0 ^ g1) & borrow;
        g0 ^= xorm;
        g1 ^= xorm;

        /* subtract if a_ was odd */
        f0 -= f1 & odd;
        g0 -= g1 & odd;

        f1 <<= 1;
        g1 <<= 1;
        a_lo >>= 1; a_lo |= a_hi << (LIMB_T_BITS-1);
        a_hi >>= 1;

        L += (b_lo + 2) >> 2;
    }

    fg->f0 = f0, fg->g0 = g0, fg->f1 = f1, fg->g1 = g1;

    return L;
}

static bool_t ct_is_sqr_mod_n(const limb_t inp[], const limb_t mod[], size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    limb_t a[n], b[n], t[n];
    limb_t a_[2], b_[2], neg, L = 0;
    factors fg;
    size_t i;

    vec_copy(a, inp, sizeof(a));
    vec_copy(b, mod, sizeof(b));

    for (i=0; i<(2*n*LIMB_T_BITS)/(LIMB_T_BITS-2); i++) {
        ab_approximation_n(a_, a, b_, b, n);
        L = legendre_loop_n(L, &fg, a_, b_, LIMB_T_BITS-2);
        neg = smul_n_shift_n(t, a, &fg.f0, b, &fg.g0, n);
        (void)smul_n_shift_n(b, a, &fg.f1, b, &fg.g1, n);
        vec_copy(a, t, sizeof(a));
        L += (b[0] >> 1) & neg;
    }

    L = legendre_loop_n(L, &fg, a, b, (2*n*LIMB_T_BITS)%(LIMB_T_BITS-2));

    return (L & 1) ^ 1;
}

#define CT_IS_SQR_MOD_IMPL(bits) \
inline bool_t ct_is_square_mod_##bits(const vec##bits inp, \
                                      const vec##bits mod) \
{   return ct_is_sqr_mod_n(inp, mod, NLIMBS(bits));   }

CT_IS_SQR_MOD_IMPL(384)

/*
 * |div_top| points at two most significant limbs of the dividend, |d_hi|
 * and |d_lo| are two most significant limbs of the divisor. If divisor
 * is only one limb, it is to be passed in |d_hi| with zero in |d_lo|.
 * The divisor is required to be "bitwise left-aligned," and dividend's
 * top limbs to be not larger than the divisor's. The latter limitation
 * can be problematic in the first iteration of multi-precision division,
 * where in most general case the condition would have to be "smaller."
 * The subroutine considers four limbs, two of which are "overlapping,"
 * hence the name... Another way to look at it is to think of the pair
 * of the dividend's limbs being suffixed with a zero:
 *   +-------+-------+-------+
 * R |       |       |   0   |
 *   +-------+-------+-------+
 *           +-------+-------+
 * D         |       |       |
 *           +-------+-------+
 */
limb_t div_3_limbs(const limb_t div_top[2], limb_t d_lo, limb_t d_hi)
{
    llimb_t Rx;
    limb_t r_lo = div_top[0], r_hi = div_top[1];
    limb_t Q = 0, mask, borrow, rx;
    size_t i;

    for (i = 0; i < LIMB_T_BITS; i++) {
        /* "borrow, Rx = R - D" */
        Rx = (llimb_t)r_lo - d_lo;
        rx = (limb_t)Rx;
        borrow = (limb_t)(Rx >> LIMB_T_BITS) & 1;
        Rx = r_hi - (d_hi + (llimb_t)borrow);
        borrow = (limb_t)(Rx >> LIMB_T_BITS);

        /* "if (R >= D) R -= D" */
        r_lo = ((r_lo ^ rx) & borrow) ^ rx;
        rx = (limb_t)Rx;
        r_hi = ((r_hi ^ rx) & borrow) ^ rx;

        Q <<= 1;
        Q |= ~borrow & 1;

        /* "D >>= 1" */
        d_lo >>= 1; d_lo |= d_hi << (LIMB_T_BITS - 1);
        d_hi >>= 1;
    }

    mask = 0 - MSB(Q);  /* does it overflow? */

    /* "borrow, Rx = R - D" */
    Rx = (llimb_t)r_lo - d_lo;
    rx = (limb_t)Rx;
    borrow = (limb_t)(Rx >> LIMB_T_BITS) & 1;
    Rx = r_hi - (d_hi + (llimb_t)borrow);
    borrow = (limb_t)(Rx >> LIMB_T_BITS) & 1;

    Q <<= 1;
    Q |= borrow ^ 1;

    return (Q | mask);
}

static limb_t quot_rem_n(limb_t *div_rem, const limb_t *divisor,
                                          limb_t quotient, size_t n)
{
    __builtin_assume(n != 0 && n%2 == 0);
    llimb_t limbx;
    limb_t tmp[n+1], carry, mask, borrow;
    size_t i;

    /* divisor*quotient */
    for (carry=0, i=0; i<n; i++) {
        limbx = (quotient * (llimb_t)divisor[i]) + carry;
        tmp[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS);
    }
    tmp[i] = carry;

    /* remainder = dividend - divisor*quotient */
    for (borrow=0, i=0; i<=n; i++) {
        limbx = div_rem[i] - (tmp[i] + (llimb_t)borrow);
        tmp[i] = (limb_t)limbx;
        borrow = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    mask = 0 - borrow;
    launder(mask);

    /* if quotient was off by one, add divisor to the remainder */
    for (carry=0, i=0; i<n; i++) {
        limbx = tmp[i] + ((divisor[i] & mask) + (llimb_t)carry);
        div_rem[i] = (limb_t)limbx;
        carry = (limb_t)(limbx >> LIMB_T_BITS) & 1;
    }

    return (div_rem[i] = quotient + mask);
}

inline limb_t quot_rem_128(limb_t *div_rem, const limb_t *divisor,
                                            limb_t quotient)
{   return quot_rem_n(div_rem, divisor, quotient, NLIMBS(128));   }

inline limb_t quot_rem_64(limb_t *div_rem, const limb_t *divisor,
                                           limb_t quotient)
{   return quot_rem_n(div_rem, divisor, quotient, NLIMBS(64));   }

/*
 * Unlock reference implementations in vect.c
 */
#define mul_by_8_mod_384 mul_by_8_mod_384
#define mul_by_8_mod_384x mul_by_8_mod_384x
#define mul_by_3_mod_384x mul_by_3_mod_384x
#define mul_by_1_plus_i_mod_384x mul_by_1_plus_i_mod_384x
#define add_mod_384x add_mod_384x
#define sub_mod_384x sub_mod_384x
#define lshift_mod_384x lshift_mod_384x
#define sqr_mont_384x sqr_mont_384x

inline void vec_prefetch(const void *ptr, size_t len)
{   (void)ptr; (void)len;   }

/*
 * SHA-256
 */
#define ROTR(x,n)	((x)>>n | (x)<<(32-n))
#define Sigma0(x)	(ROTR((x),2) ^ ROTR((x),13) ^ ROTR((x),22))
#define Sigma1(x)	(ROTR((x),6) ^ ROTR((x),11) ^ ROTR((x),25))
#define sigma0(x)	(ROTR((x),7) ^ ROTR((x),18) ^ ((x)>>3))
#define sigma1(x)	(ROTR((x),17) ^ ROTR((x),19) ^ ((x)>>10))
#define Ch(x,y,z)	(((x) & (y)) ^ ((~(x)) & (z)))
#define Maj(x,y,z)	(((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))

void blst_sha256_block_data_order(unsigned int *v, const void *inp,
                                                   size_t blocks)
{
    static const unsigned int K256[64] = {
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
    };
    unsigned int X[16], l, a, b, c, d, e, f, g, h, s0, s1, T1, T2;
    const unsigned char *data = inp;
    size_t round;

    a = v[0];
    b = v[1];
    c = v[2];
    d = v[3];
    e = v[4];
    f = v[5];
    g = v[6];
    h = v[7];

    while (blocks--) {
        for (round = 0; round < 16; round++) {
            l  = (unsigned int)data[0] << 24;
            l |= (unsigned int)data[1] << 16;
            l |= (unsigned int)data[2] << 8;
            l |= (unsigned int)data[3];
            data += 4;
            T1 = X[round] = l;
            T1 += h + Sigma1(e) + Ch(e, f, g) + K256[round];
            T2 = Sigma0(a) + Maj(a, b, c);
            h = g;
            g = f;
            f = e;
            e = d + T1;
            d = c;
            c = b;
            b = a;
            a = T1 + T2;
        }

        for (; round < 64; round++) {
            s0 = X[(round + 1) & 0x0f];
            s0 = sigma0(s0);
            s1 = X[(round + 14) & 0x0f];
            s1 = sigma1(s1);

            T1 = X[round & 0xf] += s0 + s1 + X[(round + 9) & 0xf];
            T1 += h + Sigma1(e) + Ch(e, f, g) + K256[round];
            T2 = Sigma0(a) + Maj(a, b, c);
            h = g;
            g = f;
            f = e;
            e = d + T1;
            d = c;
            c = b;
            b = a;
            a = T1 + T2;
        }

        a += v[0]; v[0] = a;
        b += v[1]; v[1] = b;
        c += v[2]; v[2] = c;
        d += v[3]; v[3] = d;
        e += v[4]; v[4] = e;
        f += v[5]; v[5] = f;
        g += v[6]; v[6] = g;
        h += v[7]; v[7] = h;
    }
}
#undef ROTR
#undef Sigma0
#undef Sigma1
#undef sigma0
#undef sigma1
#undef Ch
#undef Maj

void blst_sha256_hcopy(unsigned int dst[8], const unsigned int src[8])
{
    size_t i;

    for (i=0; i<8; i++)
        dst[i] = src[i];
}

void blst_sha256_emit(unsigned char md[32], const unsigned int h[8])
{
    size_t i;

    for (i=0; i<8; i++, md+=4) {
        unsigned int h_i = h[i];
        md[0] = (unsigned char)(h_i >> 24);
        md[1] = (unsigned char)(h_i >> 16);
        md[2] = (unsigned char)(h_i >> 8);
        md[3] = (unsigned char)h_i;
    }
}

void blst_sha256_bcopy(void *dst_, const void *src_, size_t len)
{
    unsigned char *dst = dst_;
    const unsigned char *src = src_;
    size_t i;

    for (i=0; i<len; i++)
        dst[i] = src[i];
}
