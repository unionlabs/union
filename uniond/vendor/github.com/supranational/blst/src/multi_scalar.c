/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "fields.h"
#include "point.h"

/*
 * Infinite point among inputs would be devastating. Shall we change it?
 */
#define POINTS_TO_AFFINE_IMPL(prefix, ptype, bits, field) \
static void ptype##s_to_affine(ptype##_affine dst[], \
                               const ptype *const points[], size_t npoints) \
{ \
    size_t i; \
    vec##bits *acc, ZZ, ZZZ; \
    const ptype *point = NULL; \
    const size_t stride = sizeof(ptype)==sizeof(POINTonE1) ? 1536 : 768; \
\
    while (npoints) { \
        const ptype *p, *const *walkback; \
        size_t delta = stride<npoints ? stride : npoints; \
\
        point = *points ? *points++ : point+1; \
        acc = (vec##bits *)dst; \
        vec_copy(acc++, point->Z, sizeof(vec##bits)); \
        for (i = 1; i < delta; i++, acc++) \
            point = *points ? *points++ : point+1, \
            mul_##field(acc[0], acc[-1], point->Z); \
\
        --acc; reciprocal_##field(acc[0], acc[0]); \
\
        walkback = points-1, p = point, --delta, dst += delta; \
        for (i = 0; i < delta; i++, acc--, dst--) { \
            mul_##field(acc[-1], acc[-1], acc[0]);  /* 1/Z        */\
            sqr_##field(ZZ, acc[-1]);               /* 1/Z^2      */\
            mul_##field(ZZZ, ZZ, acc[-1]);          /* 1/Z^3      */\
            mul_##field(acc[-1], p->Z, acc[0]);     \
            mul_##field(dst->X,  p->X, ZZ);         /* X = X'/Z^2 */\
            mul_##field(dst->Y,  p->Y, ZZZ);        /* Y = Y'/Z^3 */\
            p = (p == *walkback) ? *--walkback : p-1; \
        } \
        sqr_##field(ZZ, acc[0]);                    /* 1/Z^2      */\
        mul_##field(ZZZ, ZZ, acc[0]);               /* 1/Z^3      */\
        mul_##field(dst->X, p->X, ZZ);              /* X = X'/Z^2 */\
        mul_##field(dst->Y, p->Y, ZZZ);             /* Y = Y'/Z^3 */\
        ++delta, dst += delta, npoints -= delta; \
    } \
} \
\
void prefix##s_to_affine(ptype##_affine dst[], const ptype *const points[], \
                         size_t npoints) \
{   ptype##s_to_affine(dst, points, npoints);   }

POINTS_TO_AFFINE_IMPL(blst_p1, POINTonE1, 384, fp)
POINTS_TO_AFFINE_IMPL(blst_p2, POINTonE2, 384x, fp2)

/*
 * This is two-step multi-scalar multiplication procedure. First, given
 * a set of points you pre-compute a table for chosen windowing factor
 * [expressed in bits with value between 2 and 14], and then you pass
 * this table to the actual multiplication procedure along with scalars.
 * Idea is that the pre-computed table will be reused multiple times. In
 * which case multiplication runs faster than below Pippenger algorithm
 * implementation for up to ~16K points for wbits=8, naturally at the
 * expense of multi-megabyte table. One can trade even more memory for
 * performance, but each wbits increment doubles the memory requirement,
 * so at some point it gets prohibively large... For reference, without
 * reusing the table it's faster than Pippenger algorithm for up ~32
 * points [with wbits=5]...
 */

#define SCRATCH_SZ(ptype) (sizeof(ptype)==sizeof(POINTonE1) ? 8192 : 4096)

#define PRECOMPUTE_WBITS_IMPL(prefix, ptype, bits, field, one) \
static void ptype##_precompute_row_wbits(ptype row[], size_t wbits, \
                                         const ptype##_affine *point) \
{ \
    size_t i, j, n = (size_t)1 << (wbits-1); \
                                          /* row[-1] is implicit infinity */\
    vec_copy(&row[0], point, sizeof(*point));           /* row[0]=p*1     */\
    vec_copy(&row[0].Z, one, sizeof(row[0].Z));                             \
    ptype##_double(&row[1],  &row[0]);                  /* row[1]=p*(1+1) */\
    for (i = 2, j = 1; i < n; i += 2, j++) \
        ptype##_add_affine(&row[i], &row[i-1], point),  /* row[2]=p*(2+1) */\
        ptype##_double(&row[i+1], &row[j]);             /* row[3]=p*(2+2) */\
}                                                       /* row[4] ...     */\
\
static void ptype##s_to_affine_row_wbits(ptype##_affine dst[], ptype src[], \
                                         size_t wbits, size_t npoints) \
{ \
    size_t total = npoints << (wbits-1); \
    size_t nwin = (size_t)1 << (wbits-1); \
    size_t i, j; \
    vec##bits *acc, ZZ, ZZZ; \
\
    src += total; \
    acc = (vec##bits *)src; \
    vec_copy(acc++, one, sizeof(vec##bits)); \
    for (i = 0; i < npoints; i++) \
        for (j = nwin; --src, --j; acc++)    \
            mul_##field(acc[0], acc[-1], src->Z); \
\
    --acc; reciprocal_##field(acc[0], acc[0]); \
\
    for (i = 0; i < npoints; i++) { \
        vec_copy(dst++, src++, sizeof(ptype##_affine)); \
        for (j = 1; j < nwin; j++, acc--, src++, dst++) { \
            mul_##field(acc[-1], acc[-1], acc[0]);  /* 1/Z        */\
            sqr_##field(ZZ, acc[-1]);               /* 1/Z^2      */\
            mul_##field(ZZZ, ZZ, acc[-1]);          /* 1/Z^3      */\
            mul_##field(acc[-1], src->Z, acc[0]);                   \
            mul_##field(dst->X, src->X, ZZ);        /* X = X'/Z^2 */\
            mul_##field(dst->Y, src->Y, ZZZ);       /* Y = Y'/Z^3 */\
        } \
    } \
} \
\
/* flat |points[n]| can be placed at the end of |table[n<<(wbits-1)]| */\
static void ptype##s_precompute_wbits(ptype##_affine table[], size_t wbits, \
                                      const ptype##_affine *const points[], \
                                      size_t npoints) \
{ \
    size_t total = npoints << (wbits-1); \
    size_t nwin = (size_t)1 << (wbits-1); \
    size_t nmin = wbits>9 ? (size_t)1: (size_t)1 << (9-wbits); \
    size_t i, top = 0; \
    ptype *rows, *row; \
    const ptype##_affine *point = NULL; \
    size_t stride = ((512*1024)/sizeof(ptype##_affine)) >> wbits; \
    if (stride == 0) stride = 1; \
\
    while (npoints >= nmin) { \
        size_t limit = total - npoints; \
\
        if (top + (stride << wbits) > limit) { \
            stride = (limit - top) >> wbits;   \
            if (stride == 0) break;            \
        } \
        rows = row = (ptype *)(&table[top]); \
        for (i = 0; i < stride; i++, row += nwin) \
            point = *points ? *points++ : point+1, \
            ptype##_precompute_row_wbits(row, wbits, point); \
        ptype##s_to_affine_row_wbits(&table[top], rows, wbits, stride); \
        top += stride << (wbits-1); \
        npoints -= stride; \
    } \
    rows = row = alloca(2*sizeof(ptype##_affine) * npoints * nwin); \
    for (i = 0; i < npoints; i++, row += nwin) \
        point = *points ? *points++ : point+1, \
        ptype##_precompute_row_wbits(row, wbits, point); \
    ptype##s_to_affine_row_wbits(&table[top], rows, wbits, npoints); \
} \
\
size_t prefix##s_mult_wbits_precompute_sizeof(size_t wbits, size_t npoints) \
{ return (sizeof(ptype##_affine)*npoints) << (wbits-1); } \
void prefix##s_mult_wbits_precompute(ptype##_affine table[], size_t wbits, \
                                     const ptype##_affine *const points[], \
                                     size_t npoints) \
{ ptype##s_precompute_wbits(table, wbits, points, npoints); }

#define POINTS_MULT_WBITS_IMPL(prefix, ptype, bits, field, one) \
static void ptype##_gather_booth_wbits(ptype *p, const ptype##_affine row[], \
                                       size_t wbits, limb_t booth_idx) \
{ \
    bool_t booth_sign = (booth_idx >> wbits) & 1; \
    bool_t idx_is_zero; \
    static const ptype##_affine infinity = { 0 }; \
\
    booth_idx &= ((limb_t)1 << wbits) - 1; \
    idx_is_zero = is_zero(booth_idx); \
    booth_idx -= 1 ^ idx_is_zero; \
    vec_select(p, &infinity, &row[booth_idx], sizeof(row[0]), idx_is_zero); \
    ptype##_cneg(p, booth_sign); \
} \
\
static void ptype##s_mult_wbits(ptype *ret, const ptype##_affine table[], \
                                size_t wbits, size_t npoints, \
                                const byte *const scalars[], size_t nbits, \
                                ptype scratch[]) \
{ \
    limb_t wmask, wval; \
    size_t i, j, z, nbytes, window, nwin = (size_t)1 << (wbits-1); \
    const byte *scalar, *const *scalar_s = scalars; \
    const ptype##_affine *row = table; \
\
    size_t scratch_sz = SCRATCH_SZ(ptype); \
    if (scratch == NULL) { \
        scratch_sz /= 4; /* limit to 288K */ \
        scratch_sz = scratch_sz < npoints ? scratch_sz : npoints; \
        scratch = alloca(sizeof(ptype) * scratch_sz); \
    } \
\
    nbytes = (nbits + 7)/8; /* convert |nbits| to bytes */ \
    scalar = *scalar_s++; \
\
    /* top excess bits modulo target window size */ \
    window = nbits % wbits; /* yes, it may be zero */ \
    wmask = ((limb_t)1 << (window + 1)) - 1; \
\
    nbits -= window; \
    z = is_zero(nbits); \
    wval = (get_wval_limb(scalar, nbits - (z^1), wbits + (z^1)) << z) & wmask; \
    wval = booth_encode(wval, wbits); \
    ptype##_gather_booth_wbits(&scratch[0], row, wbits, wval); \
    row += nwin; \
\
    i = 1; vec_zero(ret, sizeof(*ret)); \
    while (nbits > 0) { \
        for (j = i; i < npoints; i++, j++, row += nwin) { \
            if (j == scratch_sz) \
                ptype##s_accumulate(ret, scratch, j), j = 0; \
            scalar = *scalar_s ? *scalar_s++ : scalar+nbytes; \
            wval = get_wval_limb(scalar, nbits - 1, window + 1) & wmask; \
            wval = booth_encode(wval, wbits); \
            ptype##_gather_booth_wbits(&scratch[j], row, wbits, wval); \
        } \
        ptype##s_accumulate(ret, scratch, j); \
\
        for (j = 0; j < wbits; j++) \
            ptype##_double(ret, ret); \
\
        window = wbits; \
        wmask = ((limb_t)1 << (window + 1)) - 1; \
        nbits -= window; \
        i = 0; row = table; scalar_s = scalars; \
    } \
\
    for (j = i; i < npoints; i++, j++, row += nwin) { \
        if (j == scratch_sz) \
            ptype##s_accumulate(ret, scratch, j), j = 0; \
        scalar = *scalar_s ? *scalar_s++ : scalar+nbytes; \
        wval = (get_wval_limb(scalar, 0, wbits) << 1) & wmask; \
        wval = booth_encode(wval, wbits); \
        ptype##_gather_booth_wbits(&scratch[j], row, wbits, wval); \
    } \
    ptype##s_accumulate(ret, scratch, j); \
} \
\
size_t prefix##s_mult_wbits_scratch_sizeof(size_t npoints) \
{ \
    const size_t scratch_sz = SCRATCH_SZ(ptype); \
    return sizeof(ptype) * (npoints < scratch_sz ? npoints : scratch_sz); \
} \
void prefix##s_mult_wbits(ptype *ret, const ptype##_affine table[], \
                          size_t wbits, size_t npoints, \
                          const byte *const scalars[], size_t nbits, \
                          ptype scratch[]) \
{ ptype##s_mult_wbits(ret, table, wbits, npoints, scalars, nbits, scratch); }

PRECOMPUTE_WBITS_IMPL(blst_p1, POINTonE1, 384, fp, BLS12_381_Rx.p)
POINTS_MULT_WBITS_IMPL(blst_p1, POINTonE1, 384, fp, BLS12_381_Rx.p)

PRECOMPUTE_WBITS_IMPL(blst_p2, POINTonE2, 384x, fp2, BLS12_381_Rx.p2)
POINTS_MULT_WBITS_IMPL(blst_p2, POINTonE2, 384x, fp2, BLS12_381_Rx.p2)

/*
 * Pippenger algorithm implementation, fastest option for larger amount
 * of points...
 */

static size_t pippenger_window_size(size_t npoints)
{
    size_t wbits;

    for (wbits=0; npoints>>=1; wbits++) ;

    return wbits>12 ? wbits-3 : (wbits>4 ? wbits-2 : (wbits ? 2 : 1));
}

#define DECLARE_PRIVATE_POINTXYZZ(ptype, bits) \
typedef struct { vec##bits X,Y,ZZZ,ZZ; } ptype##xyzz;

#define POINTS_MULT_PIPPENGER_IMPL(prefix, ptype) \
static void ptype##_integrate_buckets(ptype *out, ptype##xyzz buckets[], \
                                                  size_t wbits) \
{ \
    ptype##xyzz ret[1], acc[1]; \
    size_t n = (size_t)1 << wbits; \
\
    /* Calculate sum of x[i-1]*i for i=1 through 1<<|wbits|. */\
    vec_copy(acc, &buckets[--n], sizeof(acc)); \
    vec_copy(ret, &buckets[n], sizeof(ret)); \
    vec_zero(&buckets[n], sizeof(buckets[n])); \
    while (n--) { \
        ptype##xyzz_dadd(acc, acc, &buckets[n]); \
        ptype##xyzz_dadd(ret, ret, acc); \
        vec_zero(&buckets[n], sizeof(buckets[n])); \
    } \
    ptype##xyzz_to_Jacobian(out, ret); \
} \
\
static void ptype##_bucket(ptype##xyzz buckets[], limb_t booth_idx, \
                           size_t wbits, const ptype##_affine *p) \
{ \
    bool_t booth_sign = (booth_idx >> wbits) & 1; \
\
    booth_idx &= (1<<wbits) - 1; \
    if (booth_idx--) \
        ptype##xyzz_dadd_affine(&buckets[booth_idx], &buckets[booth_idx], \
                                                     p, booth_sign); \
} \
\
static void ptype##_prefetch(const ptype##xyzz buckets[], limb_t booth_idx, \
                             size_t wbits) \
{ \
    booth_idx &= (1<<wbits) - 1; \
    if (booth_idx--) \
        vec_prefetch(&buckets[booth_idx], sizeof(buckets[booth_idx])); \
} \
\
static void ptype##s_tile_pippenger(ptype *ret, \
                                    const ptype##_affine *const points[], \
                                    size_t npoints, \
                                    const byte *const scalars[], size_t nbits, \
                                    ptype##xyzz buckets[], \
                                    size_t bit0, size_t wbits, size_t cbits) \
{ \
    limb_t wmask, wval, wnxt; \
    size_t i, z, nbytes; \
    const byte *scalar = *scalars++; \
    const ptype##_affine *point = *points++; \
\
    nbytes = (nbits + 7)/8; /* convert |nbits| to bytes */ \
    wmask = ((limb_t)1 << (wbits+1)) - 1; \
    z = is_zero(bit0); \
    bit0 -= z^1; wbits += z^1; \
    wval = (get_wval_limb(scalar, bit0, wbits) << z) & wmask; \
    wval = booth_encode(wval, cbits); \
    scalar = *scalars ? *scalars++ : scalar+nbytes; \
    wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask; \
    wnxt = booth_encode(wnxt, cbits); \
    npoints--;  /* account for prefetch */ \
\
    ptype##_bucket(buckets, wval, cbits, point); \
    for (i = 1; i < npoints; i++) { \
        wval = wnxt; \
        scalar = *scalars ? *scalars++ : scalar+nbytes; \
        wnxt = (get_wval_limb(scalar, bit0, wbits) << z) & wmask; \
        wnxt = booth_encode(wnxt, cbits); \
        ptype##_prefetch(buckets, wnxt, cbits); \
        point = *points ? *points++ : point+1; \
        ptype##_bucket(buckets, wval, cbits, point); \
    } \
    point = *points ? *points++ : point+1; \
    ptype##_bucket(buckets, wnxt, cbits, point); \
    ptype##_integrate_buckets(ret, buckets, cbits - 1); \
} \
\
static void ptype##s_mult_pippenger(ptype *ret, \
                                    const ptype##_affine *const points[], \
                                    size_t npoints, \
                                    const byte *const scalars[], size_t nbits, \
                                    ptype##xyzz buckets[], size_t window) \
{ \
    size_t i, wbits, cbits, bit0 = nbits; \
    ptype tile[1]; \
\
    window = window ? window : pippenger_window_size(npoints); \
    vec_zero(buckets, sizeof(buckets[0]) << (window-1)); \
    vec_zero(ret, sizeof(*ret)); \
\
    /* top excess bits modulo target window size */ \
    wbits = nbits % window; /* yes, it may be zero */ \
    cbits = wbits + 1; \
    while (bit0 -= wbits) { \
        ptype##s_tile_pippenger(tile, points, npoints, scalars, nbits, \
                                      buckets, bit0, wbits, cbits); \
        ptype##_dadd(ret, ret, tile, NULL); \
        for (i = 0; i < window; i++) \
            ptype##_double(ret, ret); \
        cbits = wbits = window; \
    } \
    ptype##s_tile_pippenger(tile, points, npoints, scalars, nbits, \
                                  buckets, 0, wbits, cbits); \
    ptype##_dadd(ret, ret, tile, NULL); \
} \
\
size_t prefix##s_mult_pippenger_scratch_sizeof(size_t npoints) \
{   return sizeof(ptype##xyzz) << (pippenger_window_size(npoints)-1);   } \
void prefix##s_tile_pippenger(ptype *ret, \
                              const ptype##_affine *const points[], \
                              size_t npoints, \
                              const byte *const scalars[], size_t nbits, \
                              ptype##xyzz scratch[], \
                              size_t bit0, size_t window) \
{ \
    size_t wbits, cbits; \
\
    if (bit0 + window > nbits)  wbits = nbits - bit0, cbits = wbits + 1; \
    else                        wbits = cbits = window; \
    ptype##s_tile_pippenger(ret, points, npoints, scalars, nbits, scratch, \
                                 bit0, wbits, cbits); \
} \
void prefix##s_mult_pippenger(ptype *ret, \
                              const ptype##_affine *const points[], \
                              size_t npoints, \
                              const byte *const scalars[], size_t nbits, \
                              ptype##xyzz scratch[]) \
{ \
    if (npoints == 1) { \
        prefix##_from_affine(ret, points[0]); \
        prefix##_mult(ret, ret, scalars[0], nbits); \
        return; \
    } \
    if ((npoints * sizeof(ptype##_affine) * 8 * 3) <= SCRATCH_LIMIT) { \
        ptype##_affine *table = alloca(npoints * sizeof(ptype##_affine) * 8); \
        ptype##s_precompute_wbits(table, 4, points, npoints); \
        ptype##s_mult_wbits(ret, table, 4, npoints, scalars, nbits, NULL); \
        return; \
    } \
    ptype##s_mult_pippenger(ret, points, npoints, scalars, nbits, scratch, 0); \
}

DECLARE_PRIVATE_POINTXYZZ(POINTonE1, 384)
POINTXYZZ_TO_JACOBIAN_IMPL(POINTonE1, 384, fp)
POINTXYZZ_DADD_IMPL(POINTonE1, 384, fp)
POINTXYZZ_DADD_AFFINE_IMPL(POINTonE1, 384, fp, BLS12_381_Rx.p)
POINTS_MULT_PIPPENGER_IMPL(blst_p1, POINTonE1)

DECLARE_PRIVATE_POINTXYZZ(POINTonE2, 384x)
POINTXYZZ_TO_JACOBIAN_IMPL(POINTonE2, 384x, fp2)
POINTXYZZ_DADD_IMPL(POINTonE2, 384x, fp2)
POINTXYZZ_DADD_AFFINE_IMPL(POINTonE2, 384x, fp2, BLS12_381_Rx.p2)
POINTS_MULT_PIPPENGER_IMPL(blst_p2, POINTonE2)
