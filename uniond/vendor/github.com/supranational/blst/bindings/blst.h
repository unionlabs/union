/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLST_H__
#define __BLST_H__

#ifdef __SIZE_TYPE__
typedef __SIZE_TYPE__ size_t;
#else
#include <stddef.h>
#endif

#if defined(__UINT8_TYPE__) && defined(__UINT32_TYPE__) \
                            && defined(__UINT64_TYPE__)
typedef __UINT8_TYPE__  uint8_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
#else
#include <stdint.h>
#endif

#ifdef __cplusplus
extern "C" {
#elif defined(__BLST_CGO__)
typedef _Bool bool; /* it's assumed that cgo calls modern enough compiler */
#elif !defined(bool)
# if defined(__STDC_VERSION__) && __STDC_VERSION__>=199901
#  define bool _Bool
# else
#  define bool int
# endif
# define __blst_h_bool__
#endif

#ifdef SWIG
# define DEFNULL =NULL
#elif defined __cplusplus
# define DEFNULL =0
#else
# define DEFNULL
#endif

typedef enum {
    BLST_SUCCESS = 0,
    BLST_BAD_ENCODING,
    BLST_POINT_NOT_ON_CURVE,
    BLST_POINT_NOT_IN_GROUP,
    BLST_AGGR_TYPE_MISMATCH,
    BLST_VERIFY_FAIL,
    BLST_PK_IS_INFINITY,
    BLST_BAD_SCALAR,
} BLST_ERROR;

typedef uint8_t byte;
typedef uint64_t limb_t;

typedef struct { byte b[256/8]; } blst_scalar;
typedef struct { limb_t l[256/8/sizeof(limb_t)]; } blst_fr;
typedef struct { limb_t l[384/8/sizeof(limb_t)]; } blst_fp;
/* 0 is "real" part, 1 is "imaginary" */
typedef struct { blst_fp fp[2]; } blst_fp2;
typedef struct { blst_fp2 fp2[3]; } blst_fp6;
typedef struct { blst_fp6 fp6[2]; } blst_fp12;

void blst_scalar_from_uint32(blst_scalar *out, const uint32_t a[8]);
void blst_uint32_from_scalar(uint32_t out[8], const blst_scalar *a);
void blst_scalar_from_uint64(blst_scalar *out, const uint64_t a[4]);
void blst_uint64_from_scalar(uint64_t out[4], const blst_scalar *a);
void blst_scalar_from_bendian(blst_scalar *out, const byte a[32]);
void blst_bendian_from_scalar(byte out[32], const blst_scalar *a);
void blst_scalar_from_lendian(blst_scalar *out, const byte a[32]);
void blst_lendian_from_scalar(byte out[32], const blst_scalar *a);
bool blst_scalar_fr_check(const blst_scalar *a);
bool blst_sk_check(const blst_scalar *a);
bool blst_sk_add_n_check(blst_scalar *out, const blst_scalar *a,
                                           const blst_scalar *b);
bool blst_sk_sub_n_check(blst_scalar *out, const blst_scalar *a,
                                           const blst_scalar *b);
bool blst_sk_mul_n_check(blst_scalar *out, const blst_scalar *a,
                                           const blst_scalar *b);
void blst_sk_inverse(blst_scalar *out, const blst_scalar *a);
bool blst_scalar_from_le_bytes(blst_scalar *out, const byte *in, size_t len);
bool blst_scalar_from_be_bytes(blst_scalar *out, const byte *in, size_t len);

#ifndef SWIG
/*
 * BLS12-381-specific Fr operations.
 */
void blst_fr_add(blst_fr *ret, const blst_fr *a, const blst_fr *b);
void blst_fr_sub(blst_fr *ret, const blst_fr *a, const blst_fr *b);
void blst_fr_mul_by_3(blst_fr *ret, const blst_fr *a);
void blst_fr_lshift(blst_fr *ret, const blst_fr *a, size_t count);
void blst_fr_rshift(blst_fr *ret, const blst_fr *a, size_t count);
void blst_fr_mul(blst_fr *ret, const blst_fr *a, const blst_fr *b);
void blst_fr_sqr(blst_fr *ret, const blst_fr *a);
void blst_fr_cneg(blst_fr *ret, const blst_fr *a, bool flag);
void blst_fr_eucl_inverse(blst_fr *ret, const blst_fr *a);
void blst_fr_inverse(blst_fr *ret, const blst_fr *a);

void blst_fr_from_uint64(blst_fr *ret, const uint64_t a[4]);
void blst_uint64_from_fr(uint64_t ret[4], const blst_fr *a);
void blst_fr_from_scalar(blst_fr *ret, const blst_scalar *a);
void blst_scalar_from_fr(blst_scalar *ret, const blst_fr *a);

/*
 * BLS12-381-specific Fp operations.
 */
void blst_fp_add(blst_fp *ret, const blst_fp *a, const blst_fp *b);
void blst_fp_sub(blst_fp *ret, const blst_fp *a, const blst_fp *b);
void blst_fp_mul_by_3(blst_fp *ret, const blst_fp *a);
void blst_fp_mul_by_8(blst_fp *ret, const blst_fp *a);
void blst_fp_lshift(blst_fp *ret, const blst_fp *a, size_t count);
void blst_fp_mul(blst_fp *ret, const blst_fp *a, const blst_fp *b);
void blst_fp_sqr(blst_fp *ret, const blst_fp *a);
void blst_fp_cneg(blst_fp *ret, const blst_fp *a, bool flag);
void blst_fp_eucl_inverse(blst_fp *ret, const blst_fp *a);
void blst_fp_inverse(blst_fp *ret, const blst_fp *a);
bool blst_fp_sqrt(blst_fp *ret, const blst_fp *a);

void blst_fp_from_uint32(blst_fp *ret, const uint32_t a[12]);
void blst_uint32_from_fp(uint32_t ret[12], const blst_fp *a);
void blst_fp_from_uint64(blst_fp *ret, const uint64_t a[6]);
void blst_uint64_from_fp(uint64_t ret[6], const blst_fp *a);
void blst_fp_from_bendian(blst_fp *ret, const byte a[48]);
void blst_bendian_from_fp(byte ret[48], const blst_fp *a);
void blst_fp_from_lendian(blst_fp *ret, const byte a[48]);
void blst_lendian_from_fp(byte ret[48], const blst_fp *a);

/*
 * BLS12-381-specific Fp2 operations.
 */
void blst_fp2_add(blst_fp2 *ret, const blst_fp2 *a, const blst_fp2 *b);
void blst_fp2_sub(blst_fp2 *ret, const blst_fp2 *a, const blst_fp2 *b);
void blst_fp2_mul_by_3(blst_fp2 *ret, const blst_fp2 *a);
void blst_fp2_mul_by_8(blst_fp2 *ret, const blst_fp2 *a);
void blst_fp2_lshift(blst_fp2 *ret, const blst_fp2 *a, size_t count);
void blst_fp2_mul(blst_fp2 *ret, const blst_fp2 *a, const blst_fp2 *b);
void blst_fp2_sqr(blst_fp2 *ret, const blst_fp2 *a);
void blst_fp2_cneg(blst_fp2 *ret, const blst_fp2 *a, bool flag);
void blst_fp2_eucl_inverse(blst_fp2 *ret, const blst_fp2 *a);
void blst_fp2_inverse(blst_fp2 *ret, const blst_fp2 *a);
bool blst_fp2_sqrt(blst_fp2 *ret, const blst_fp2 *a);

/*
 * BLS12-381-specific Fp12 operations.
 */
void blst_fp12_sqr(blst_fp12 *ret, const blst_fp12 *a);
void blst_fp12_cyclotomic_sqr(blst_fp12 *ret, const blst_fp12 *a);
void blst_fp12_mul(blst_fp12 *ret, const blst_fp12 *a, const blst_fp12 *b);
void blst_fp12_mul_by_xy00z0(blst_fp12 *ret, const blst_fp12 *a,
                                             const blst_fp6 *xy00z0);
void blst_fp12_conjugate(blst_fp12 *a);
void blst_fp12_inverse(blst_fp12 *ret, const blst_fp12 *a);
/* caveat lector! |n| has to be non-zero and not more than 3! */
void blst_fp12_frobenius_map(blst_fp12 *ret, const blst_fp12 *a, size_t n);
bool blst_fp12_is_equal(const blst_fp12 *a, const blst_fp12 *b);
bool blst_fp12_is_one(const blst_fp12 *a);
bool blst_fp12_in_group(const blst_fp12 *a);
const blst_fp12 *blst_fp12_one(void);
#endif  // SWIG

/*
 * BLS12-381-specific point operations.
 */
typedef struct { blst_fp x, y, z; } blst_p1;
typedef struct { blst_fp x, y; } blst_p1_affine;

void blst_p1_add(blst_p1 *out, const blst_p1 *a, const blst_p1 *b);
void blst_p1_add_or_double(blst_p1 *out, const blst_p1 *a, const blst_p1 *b);
void blst_p1_add_affine(blst_p1 *out, const blst_p1 *a,
                                      const blst_p1_affine *b);
void blst_p1_add_or_double_affine(blst_p1 *out, const blst_p1 *a,
                                                const blst_p1_affine *b);
void blst_p1_double(blst_p1 *out, const blst_p1 *a);
void blst_p1_mult(blst_p1 *out, const blst_p1 *p, const byte *scalar,
                                                  size_t nbits);
void blst_p1_cneg(blst_p1 *p, bool cbit);
void blst_p1_to_affine(blst_p1_affine *out, const blst_p1 *in);
void blst_p1_from_affine(blst_p1 *out, const blst_p1_affine *in);
bool blst_p1_on_curve(const blst_p1 *p);
bool blst_p1_in_g1(const blst_p1 *p);
bool blst_p1_is_equal(const blst_p1 *a, const blst_p1 *b);
bool blst_p1_is_inf(const blst_p1 *a);
const blst_p1 *blst_p1_generator(void);

bool blst_p1_affine_on_curve(const blst_p1_affine *p);
bool blst_p1_affine_in_g1(const blst_p1_affine *p);
bool blst_p1_affine_is_equal(const blst_p1_affine *a, const blst_p1_affine *b);
bool blst_p1_affine_is_inf(const blst_p1_affine *a);
const blst_p1_affine *blst_p1_affine_generator(void);

typedef struct { blst_fp2 x, y, z; } blst_p2;
typedef struct { blst_fp2 x, y; } blst_p2_affine;

void blst_p2_add(blst_p2 *out, const blst_p2 *a, const blst_p2 *b);
void blst_p2_add_or_double(blst_p2 *out, const blst_p2 *a, const blst_p2 *b);
void blst_p2_add_affine(blst_p2 *out, const blst_p2 *a,
                                      const blst_p2_affine *b);
void blst_p2_add_or_double_affine(blst_p2 *out, const blst_p2 *a,
                                                const blst_p2_affine *b);
void blst_p2_double(blst_p2 *out, const blst_p2 *a);
void blst_p2_mult(blst_p2 *out, const blst_p2 *p, const byte *scalar,
                                                  size_t nbits);
void blst_p2_cneg(blst_p2 *p, bool cbit);
void blst_p2_to_affine(blst_p2_affine *out, const blst_p2 *in);
void blst_p2_from_affine(blst_p2 *out, const blst_p2_affine *in);
bool blst_p2_on_curve(const blst_p2 *p);
bool blst_p2_in_g2(const blst_p2 *p);
bool blst_p2_is_equal(const blst_p2 *a, const blst_p2 *b);
bool blst_p2_is_inf(const blst_p2 *a);
const blst_p2 *blst_p2_generator(void);

bool blst_p2_affine_on_curve(const blst_p2_affine *p);
bool blst_p2_affine_in_g2(const blst_p2_affine *p);
bool blst_p2_affine_is_equal(const blst_p2_affine *a, const blst_p2_affine *b);
bool blst_p2_affine_is_inf(const blst_p2_affine *a);
const blst_p2_affine *blst_p2_affine_generator(void);

/*
 * Multi-scalar multiplications and other multi-point operations.
 */

void blst_p1s_to_affine(blst_p1_affine dst[], const blst_p1 *const points[],
                        size_t npoints);
void blst_p1s_add(blst_p1 *ret, const blst_p1_affine *const points[],
                                size_t npoints);

size_t blst_p1s_mult_wbits_precompute_sizeof(size_t wbits, size_t npoints);
void blst_p1s_mult_wbits_precompute(blst_p1_affine table[], size_t wbits,
                                    const blst_p1_affine *const points[],
                                    size_t npoints);
size_t blst_p1s_mult_wbits_scratch_sizeof(size_t npoints);
void blst_p1s_mult_wbits(blst_p1 *ret, const blst_p1_affine table[],
                         size_t wbits, size_t npoints,
                         const byte *const scalars[], size_t nbits,
                         limb_t *scratch);

size_t blst_p1s_mult_pippenger_scratch_sizeof(size_t npoints);
void blst_p1s_mult_pippenger(blst_p1 *ret, const blst_p1_affine *const points[],
                             size_t npoints, const byte *const scalars[],
                             size_t nbits, limb_t *scratch);
void blst_p1s_tile_pippenger(blst_p1 *ret, const blst_p1_affine *const points[],
                             size_t npoints, const byte *const scalars[],
                             size_t nbits, limb_t *scratch,
                             size_t bit0, size_t window);

void blst_p2s_to_affine(blst_p2_affine dst[], const blst_p2 *const points[],
                        size_t npoints);
void blst_p2s_add(blst_p2 *ret, const blst_p2_affine *const points[],
                                size_t npoints);

size_t blst_p2s_mult_wbits_precompute_sizeof(size_t wbits, size_t npoints);
void blst_p2s_mult_wbits_precompute(blst_p2_affine table[], size_t wbits,
                                    const blst_p2_affine *const points[],
                                    size_t npoints);
size_t blst_p2s_mult_wbits_scratch_sizeof(size_t npoints);
void blst_p2s_mult_wbits(blst_p2 *ret, const blst_p2_affine table[],
                         size_t wbits, size_t npoints,
                         const byte *const scalars[], size_t nbits,
                         limb_t *scratch);

size_t blst_p2s_mult_pippenger_scratch_sizeof(size_t npoints);
void blst_p2s_mult_pippenger(blst_p2 *ret, const blst_p2_affine *const points[],
                             size_t npoints, const byte *const scalars[],
                             size_t nbits, limb_t *scratch);
void blst_p2s_tile_pippenger(blst_p2 *ret, const blst_p2_affine *const points[],
                             size_t npoints, const byte *const scalars[],
                             size_t nbits, limb_t *scratch,
                             size_t bit0, size_t window);

/*
 * Hash-to-curve operations.
 */
#ifndef SWIG
void blst_map_to_g1(blst_p1 *out, const blst_fp *u, const blst_fp *v DEFNULL);
void blst_map_to_g2(blst_p2 *out, const blst_fp2 *u, const blst_fp2 *v DEFNULL);
#endif

void blst_encode_to_g1(blst_p1 *out,
                       const byte *msg, size_t msg_len,
                       const byte *DST DEFNULL, size_t DST_len DEFNULL,
                       const byte *aug DEFNULL, size_t aug_len DEFNULL);
void blst_hash_to_g1(blst_p1 *out,
                     const byte *msg, size_t msg_len,
                     const byte *DST DEFNULL, size_t DST_len DEFNULL,
                     const byte *aug DEFNULL, size_t aug_len DEFNULL);

void blst_encode_to_g2(blst_p2 *out,
                       const byte *msg, size_t msg_len,
                       const byte *DST DEFNULL, size_t DST_len DEFNULL,
                       const byte *aug DEFNULL, size_t aug_len DEFNULL);
void blst_hash_to_g2(blst_p2 *out,
                     const byte *msg, size_t msg_len,
                     const byte *DST DEFNULL, size_t DST_len DEFNULL,
                     const byte *aug DEFNULL, size_t aug_len DEFNULL);

/*
 * Zcash-compatible serialization/deserialization.
 */
void blst_p1_serialize(byte out[96], const blst_p1 *in);
void blst_p1_compress(byte out[48], const blst_p1 *in);
void blst_p1_affine_serialize(byte out[96], const blst_p1_affine *in);
void blst_p1_affine_compress(byte out[48], const blst_p1_affine *in);
BLST_ERROR blst_p1_uncompress(blst_p1_affine *out, const byte in[48]);
BLST_ERROR blst_p1_deserialize(blst_p1_affine *out, const byte in[96]);

void blst_p2_serialize(byte out[192], const blst_p2 *in);
void blst_p2_compress(byte out[96], const blst_p2 *in);
void blst_p2_affine_serialize(byte out[192], const blst_p2_affine *in);
void blst_p2_affine_compress(byte out[96], const blst_p2_affine *in);
BLST_ERROR blst_p2_uncompress(blst_p2_affine *out, const byte in[96]);
BLST_ERROR blst_p2_deserialize(blst_p2_affine *out, const byte in[192]);

/*
 * Specification defines two variants, 'minimal-signature-size' and
 * 'minimal-pubkey-size'. To unify appearance we choose to distinguish
 * them by suffix referring to the public key type, more specifically
 * _pk_in_g1 corresponds to 'minimal-pubkey-size' and _pk_in_g2 - to
 * 'minimal-signature-size'. It might appear a bit counterintuitive
 * in sign call, but no matter how you twist it, something is bound to
 * turn a little odd.
 */
/*
 * Secret-key operations.
 */
void blst_keygen(blst_scalar *out_SK, const byte *IKM, size_t IKM_len,
                 const byte *info DEFNULL, size_t info_len DEFNULL);
void blst_sk_to_pk_in_g1(blst_p1 *out_pk, const blst_scalar *SK);
void blst_sign_pk_in_g1(blst_p2 *out_sig, const blst_p2 *hash,
                                          const blst_scalar *SK);
void blst_sk_to_pk_in_g2(blst_p2 *out_pk, const blst_scalar *SK);
void blst_sign_pk_in_g2(blst_p1 *out_sig, const blst_p1 *hash,
                                          const blst_scalar *SK);

/*
 * Pairing interface.
 */
#ifndef SWIG
void blst_miller_loop(blst_fp12 *ret, const blst_p2_affine *Q,
                                      const blst_p1_affine *P);
void blst_miller_loop_n(blst_fp12 *ret, const blst_p2_affine *const Qs[],
                                        const blst_p1_affine *const Ps[],
                                        size_t n);
void blst_final_exp(blst_fp12 *ret, const blst_fp12 *f);
void blst_precompute_lines(blst_fp6 Qlines[68], const blst_p2_affine *Q);
void blst_miller_loop_lines(blst_fp12 *ret, const blst_fp6 Qlines[68],
                                            const blst_p1_affine *P);
bool blst_fp12_finalverify(const blst_fp12 *gt1, const blst_fp12 *gt2);
#endif

#ifdef __BLST_CGO__
typedef limb_t blst_pairing;
#elif defined(__BLST_RUST_BINDGEN__)
typedef struct {} blst_pairing;
#else
typedef struct blst_opaque blst_pairing;
#endif

size_t blst_pairing_sizeof(void);
void blst_pairing_init(blst_pairing *new_ctx, bool hash_or_encode,
                       const byte *DST DEFNULL, size_t DST_len DEFNULL);
const byte *blst_pairing_get_dst(const blst_pairing *ctx);
void blst_pairing_commit(blst_pairing *ctx);
BLST_ERROR blst_pairing_aggregate_pk_in_g2(blst_pairing *ctx,
                                           const blst_p2_affine *PK,
                                           const blst_p1_affine *signature,
                                           const byte *msg, size_t msg_len,
                                           const byte *aug DEFNULL,
                                           size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_chk_n_aggr_pk_in_g2(blst_pairing *ctx,
                                            const blst_p2_affine *PK,
                                            bool pk_grpchk,
                                            const blst_p1_affine *signature,
                                            bool sig_grpchk,
                                            const byte *msg, size_t msg_len,
                                            const byte *aug DEFNULL,
                                            size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_mul_n_aggregate_pk_in_g2(blst_pairing *ctx,
                                                 const blst_p2_affine *PK,
                                                 const blst_p1_affine *sig,
                                                 const byte *scalar,
                                                 size_t nbits,
                                                 const byte *msg,
                                                 size_t msg_len,
                                                 const byte *aug DEFNULL,
                                                 size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_chk_n_mul_n_aggr_pk_in_g2(blst_pairing *ctx,
                                                  const blst_p2_affine *PK,
                                                  bool pk_grpchk,
                                                  const blst_p1_affine *sig,
                                                  bool sig_grpchk,
                                                  const byte *scalar,
                                                  size_t nbits,
                                                  const byte *msg,
                                                  size_t msg_len,
                                                  const byte *aug DEFNULL,
                                                  size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_aggregate_pk_in_g1(blst_pairing *ctx,
                                           const blst_p1_affine *PK,
                                           const blst_p2_affine *signature,
                                           const byte *msg, size_t msg_len,
                                           const byte *aug DEFNULL,
                                           size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_chk_n_aggr_pk_in_g1(blst_pairing *ctx,
                                            const blst_p1_affine *PK,
                                            bool pk_grpchk,
                                            const blst_p2_affine *signature,
                                            bool sig_grpchk,
                                            const byte *msg, size_t msg_len,
                                            const byte *aug DEFNULL,
                                            size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_mul_n_aggregate_pk_in_g1(blst_pairing *ctx,
                                                 const blst_p1_affine *PK,
                                                 const blst_p2_affine *sig,
                                                 const byte *scalar,
                                                 size_t nbits,
                                                 const byte *msg,
                                                 size_t msg_len,
                                                 const byte *aug DEFNULL,
                                                 size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_chk_n_mul_n_aggr_pk_in_g1(blst_pairing *ctx,
                                                  const blst_p1_affine *PK,
                                                  bool pk_grpchk,
                                                  const blst_p2_affine *sig,
                                                  bool sig_grpchk,
                                                  const byte *scalar,
                                                  size_t nbits,
                                                  const byte *msg,
                                                  size_t msg_len,
                                                  const byte *aug DEFNULL,
                                                  size_t aug_len DEFNULL);
BLST_ERROR blst_pairing_merge(blst_pairing *ctx, const blst_pairing *ctx1);
bool blst_pairing_finalverify(const blst_pairing *ctx,
                              const blst_fp12 *gtsig DEFNULL);


/*
 * Customarily applications aggregate signatures separately.
 * In which case application would have to pass NULLs for |signature|
 * to blst_pairing_aggregate calls and pass aggregated signature
 * collected with these calls to blst_pairing_finalverify. Inputs are
 * Zcash-compatible "straight-from-wire" byte vectors, compressed or
 * not.
 */
BLST_ERROR blst_aggregate_in_g1(blst_p1 *out, const blst_p1 *in,
                                              const byte *zwire);
BLST_ERROR blst_aggregate_in_g2(blst_p2 *out, const blst_p2 *in,
                                              const byte *zwire);

void blst_aggregated_in_g1(blst_fp12 *out, const blst_p1_affine *signature);
void blst_aggregated_in_g2(blst_fp12 *out, const blst_p2_affine *signature);

/*
 * "One-shot" CoreVerify entry points.
 */
BLST_ERROR blst_core_verify_pk_in_g1(const blst_p1_affine *pk,
                                     const blst_p2_affine *signature,
                                     bool hash_or_encode,
                                     const byte *msg, size_t msg_len,
                                     const byte *DST DEFNULL,
                                     size_t DST_len DEFNULL,
                                     const byte *aug DEFNULL,
                                     size_t aug_len DEFNULL);
BLST_ERROR blst_core_verify_pk_in_g2(const blst_p2_affine *pk,
                                     const blst_p1_affine *signature,
                                     bool hash_or_encode,
                                     const byte *msg, size_t msg_len,
                                     const byte *DST DEFNULL,
                                     size_t DST_len DEFNULL,
                                     const byte *aug DEFNULL,
                                     size_t aug_len DEFNULL);

extern const blst_p1_affine BLS12_381_G1;
extern const blst_p1_affine BLS12_381_NEG_G1;
extern const blst_p2_affine BLS12_381_G2;
extern const blst_p2_affine BLS12_381_NEG_G2;

#include "blst_aux.h"

#ifdef __cplusplus
}
#elif defined(__blst_h_bool__)
# undef __blst_h_bool__
# undef bool
#endif
#endif
