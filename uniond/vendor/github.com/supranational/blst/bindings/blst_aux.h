/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLST_AUX_H__
#define __BLST_AUX_H__
/*
 * This file lists interfaces that might be promoted to blst.h or removed,
 * depending on their proven/unproven worthiness.
 */

void blst_fr_ct_bfly(blst_fr *x0, blst_fr *x1, const blst_fr *twiddle);
void blst_fr_gs_bfly(blst_fr *x0, blst_fr *x1, const blst_fr *twiddle);
void blst_fr_to(blst_fr *ret, const blst_fr *a);
void blst_fr_from(blst_fr *ret, const blst_fr *a);
#ifdef BLST_FR_PENTAROOT
void blst_fr_pentaroot(blst_fr *ret, const blst_fr *a);
void blst_fr_pentapow(blst_fr *ret, const blst_fr *a);
#endif

void blst_fp_to(blst_fp *ret, const blst_fp *a);
void blst_fp_from(blst_fp *ret, const blst_fp *a);

bool blst_fp_is_square(const blst_fp *a);
bool blst_fp2_is_square(const blst_fp2 *a);

void blst_p1_from_jacobian(blst_p1 *out, const blst_p1 *in);
void blst_p2_from_jacobian(blst_p2 *out, const blst_p2 *in);

/*
 * Below functions produce both point and deserialized outcome of
 * SkToPk and Sign. However, deserialized outputs are pre-decorated
 * with sign and infinity bits. This means that you have to bring the
 * output into compliance prior returning to application. If you want
 * compressed point value, then do [equivalent of]
 *
 *  byte temp[96];
 *  blst_sk_to_pk2_in_g1(temp, out_pk, SK);
 *  temp[0] |= 0x80;
 *  memcpy(out, temp, 48);
 *
 * Otherwise do
 *
 *  blst_sk_to_pk2_in_g1(out, out_pk, SK);
 *  out[0] &= ~0x20;
 *
 * Either |out| or |out_<point>| can be NULL.
 */
void blst_sk_to_pk2_in_g1(byte out[96], blst_p1_affine *out_pk,
                          const blst_scalar *SK);
void blst_sign_pk2_in_g1(byte out[192], blst_p2_affine *out_sig,
                         const blst_p2 *hash, const blst_scalar *SK);
void blst_sk_to_pk2_in_g2(byte out[192], blst_p2_affine *out_pk,
                          const blst_scalar *SK);
void blst_sign_pk2_in_g2(byte out[96], blst_p1_affine *out_sig,
                         const blst_p1 *hash, const blst_scalar *SK);

#ifdef __BLST_RUST_BINDGEN__
typedef struct {} blst_uniq;
#else
typedef struct blst_opaque blst_uniq;
#endif

size_t blst_uniq_sizeof(size_t n_nodes);
void blst_uniq_init(blst_uniq *tree);
bool blst_uniq_test(blst_uniq *tree, const byte *msg, size_t len);

#ifdef expand_message_xmd
void expand_message_xmd(unsigned char *bytes, size_t len_in_bytes,
                        const unsigned char *aug, size_t aug_len,
                        const unsigned char *msg, size_t msg_len,
                        const unsigned char *DST, size_t DST_len);
#else
void blst_expand_message_xmd(byte *out, size_t out_len,
                             const byte *msg, size_t msg_len,
                             const byte *DST, size_t DST_len);
#endif

void blst_p1_unchecked_mult(blst_p1 *out, const blst_p1 *p, const byte *scalar,
                                                            size_t nbits);
void blst_p2_unchecked_mult(blst_p2 *out, const blst_p2 *p, const byte *scalar,
                                                            size_t nbits);

void blst_pairing_raw_aggregate(blst_pairing *ctx, const blst_p2_affine *q,
                                                   const blst_p1_affine *p);
blst_fp12 *blst_pairing_as_fp12(blst_pairing *ctx);
void blst_bendian_from_fp12(byte out[48*12], const blst_fp12 *a);

void blst_keygen_v3(blst_scalar *out_SK, const byte *IKM, size_t IKM_len,
                    const byte *info DEFNULL, size_t info_len DEFNULL);
void blst_keygen_v4_5(blst_scalar *out_SK, const byte *IKM, size_t IKM_len,
                      const byte *salt, size_t salt_len,
                      const byte *info DEFNULL, size_t info_len DEFNULL);
void blst_keygen_v5(blst_scalar *out_SK, const byte *IKM, size_t IKM_len,
                    const byte *salt, size_t salt_len,
                    const byte *info DEFNULL, size_t info_len DEFNULL);
void blst_derive_master_eip2333(blst_scalar *out_SK,
                                const byte *IKM, size_t IKM_len);
void blst_derive_child_eip2333(blst_scalar *out_SK, const blst_scalar *SK,
                               uint32_t child_index);

void blst_scalar_from_hexascii(blst_scalar *out, const byte *hex);
void blst_fr_from_hexascii(blst_fr *ret, const byte *hex);
void blst_fp_from_hexascii(blst_fp *ret, const byte *hex);

size_t blst_p1_sizeof(void);
size_t blst_p1_affine_sizeof(void);
size_t blst_p2_sizeof(void);
size_t blst_p2_affine_sizeof(void);
size_t blst_fp12_sizeof(void);

/*
 * Single-shot SHA-256 hash function.
 */
void blst_sha256(byte out[32], const byte *msg, size_t msg_len);
#endif
