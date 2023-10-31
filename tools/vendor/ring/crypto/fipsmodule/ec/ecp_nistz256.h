/* Copyright (c) 2014, Intel Corporation.
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
 * SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
 * OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
 * CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE. */

#ifndef OPENSSL_HEADER_EC_ECP_NISTZ256_H
#define OPENSSL_HEADER_EC_ECP_NISTZ256_H

#include "../../limbs/limbs.h"

// Keep this in sync with p256.rs.
#if defined(OPENSSL_AARCH64) || defined(OPENSSL_X86) || defined(OPENSSL_X86_64)
#define GFp_USE_LARGE_TABLE
#endif

#define P256_LIMBS (256u / LIMB_BITS)

typedef struct {
  Limb X[P256_LIMBS];
  Limb Y[P256_LIMBS];
  Limb Z[P256_LIMBS];
} P256_POINT;

#if defined(GFp_USE_LARGE_TABLE)
typedef struct {
  Limb X[P256_LIMBS];
  Limb Y[P256_LIMBS];
} P256_POINT_AFFINE;
#endif

typedef Limb PRECOMP256_ROW[64 * 2 * P256_LIMBS]; // 64 (x, y) entries.

void GFp_nistz256_mul_mont(Limb res[P256_LIMBS], const Limb a[P256_LIMBS],
                           const Limb b[P256_LIMBS]);
void GFp_nistz256_sqr_mont(Limb res[P256_LIMBS], const Limb a[P256_LIMBS]);

/* Functions that perform constant time access to the precomputed tables */
void GFp_nistz256_select_w5(P256_POINT *out, const P256_POINT table[16],
                            crypto_word index);

#if defined(GFp_USE_LARGE_TABLE)
void GFp_nistz256_select_w7(P256_POINT_AFFINE *out, const PRECOMP256_ROW table, crypto_word index);
#endif

#endif /* OPENSSL_HEADER_EC_ECP_NISTZ256_H */
