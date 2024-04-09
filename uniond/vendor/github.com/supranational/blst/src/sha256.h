/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLS12_381_ASM_SHA256_H__
#define __BLS12_381_ASM_SHA256_H__

#include "vect.h"

#if (defined(__x86_64__) || defined(__x86_64) || defined(_M_X64)) && \
     defined(__SHA__) /* -msha */ && !defined(__BLST_PORTABLE__)
# define sha256_block_data_order blst_sha256_block_data_order_shaext
#elif defined(__aarch64__) && \
      defined(__ARM_FEATURE_CRYPTO) && !defined(__BLST_PORTABLE__)
# define sha256_block_data_order blst_sha256_block_armv8
#else
# define sha256_block_data_order blst_sha256_block_data_order
#endif
#define sha256_hcopy blst_sha256_hcopy
#define sha256_bcopy blst_sha256_bcopy
#define sha256_emit  blst_sha256_emit

void sha256_block_data_order(unsigned int *h, const void *inp, size_t blocks);
void sha256_hcopy(unsigned int dst[8], const unsigned int src[8]);
void sha256_bcopy(void *dst, const void *src, size_t len);

/*
 * If SHA256_CTX conflicts with something, just redefine it to alternative
 * custom name prior including this header.
 */
typedef struct {
    unsigned int h[8];
    unsigned long long N;
    unsigned char buf[64];
    size_t off;
} SHA256_CTX;


static void sha256_init_h(unsigned int h[8])
{
    h[0] = 0x6a09e667U;
    h[1] = 0xbb67ae85U;
    h[2] = 0x3c6ef372U;
    h[3] = 0xa54ff53aU;
    h[4] = 0x510e527fU;
    h[5] = 0x9b05688cU;
    h[6] = 0x1f83d9abU;
    h[7] = 0x5be0cd19U;
}

static void sha256_init(SHA256_CTX *ctx)
{
    sha256_init_h(ctx->h);
    ctx->N = 0;
    vec_zero(ctx->buf, sizeof(ctx->buf));
    ctx->off = 0;
}

static void sha256_update(SHA256_CTX *ctx, const void *_inp, size_t len)
{
    size_t n;
    const unsigned char *inp = _inp;

    ctx->N += len;

    if ((len != 0) & ((n = ctx->off) != 0)) {
        size_t rem = sizeof(ctx->buf) - n;

        if (rem > len) {
            sha256_bcopy(ctx->buf + n, inp, len);
            ctx->off += len;
            return;
        } else {
            sha256_bcopy(ctx->buf + n, inp, rem);
            inp += rem;
            len -= rem;
            sha256_block_data_order(ctx->h, ctx->buf, 1);
            vec_zero(ctx->buf, sizeof(ctx->buf));
            ctx->off = 0;
        }
    }

    n = len / sizeof(ctx->buf);
    if (n > 0) {
        sha256_block_data_order(ctx->h, inp, n);
        n *= sizeof(ctx->buf);
        inp += n;
        len -= n;
    }

    if (len)
        sha256_bcopy(ctx->buf, inp, ctx->off = len);
}

#define __TOBE32(ptr, val) ((ptr)[0] = (unsigned char)((val)>>24), \
                            (ptr)[1] = (unsigned char)((val)>>16), \
                            (ptr)[2] = (unsigned char)((val)>>8),  \
                            (ptr)[3] = (unsigned char)(val))

#if 1
void sha256_emit(unsigned char md[32], const unsigned int h[8]);
#else
static void sha256_emit(unsigned char md[32], const unsigned int h[8])
{
    unsigned int h_i;

    h_i = h[0]; __TOBE32(md + 0, h_i);
    h_i = h[1]; __TOBE32(md + 4, h_i);
    h_i = h[2]; __TOBE32(md + 8, h_i);
    h_i = h[3]; __TOBE32(md + 12, h_i);
    h_i = h[4]; __TOBE32(md + 16, h_i);
    h_i = h[5]; __TOBE32(md + 20, h_i);
    h_i = h[6]; __TOBE32(md + 24, h_i);
    h_i = h[7]; __TOBE32(md + 28, h_i);
}
#endif

static void sha256_final(unsigned char md[32], SHA256_CTX *ctx)
{
    unsigned long long bits = ctx->N * 8;
    size_t n = ctx->off;
    unsigned char *tail;

    ctx->buf[n++] = 0x80;

    if (n > (sizeof(ctx->buf) - 8)) {
        sha256_block_data_order(ctx->h, ctx->buf, 1);
        vec_zero(ctx->buf, sizeof(ctx->buf));
    }

    tail = ctx->buf + sizeof(ctx->buf) - 8;
    __TOBE32(tail, (unsigned int)(bits >> 32));
    __TOBE32(tail + 4, (unsigned int)bits);
    sha256_block_data_order(ctx->h, ctx->buf, 1);
    sha256_emit(md, ctx->h);
}

#undef __TOBE32
#endif
