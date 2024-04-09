/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "consts.h"
#include "sha256.h"

static const vec384 BLS12_381_RRRR = {  /* RR^2 */
    TO_LIMB_T(0xed48ac6bd94ca1e0), TO_LIMB_T(0x315f831e03a7adf8),
    TO_LIMB_T(0x9a53352a615e29dd), TO_LIMB_T(0x34c04e5e921e1761),
    TO_LIMB_T(0x2512d43565724728), TO_LIMB_T(0x0aa6346091755d4d)
};

#ifdef expand_message_xmd
void expand_message_xmd(unsigned char *bytes, size_t len_in_bytes,
                        const unsigned char *aug, size_t aug_len,
                        const unsigned char *msg, size_t msg_len,
                        const unsigned char *DST, size_t DST_len);
#else
static void sha256_init_Zpad(SHA256_CTX *ctx)
{
    ctx->h[0] = 0xda5698beU;
    ctx->h[1] = 0x17b9b469U;
    ctx->h[2] = 0x62335799U;
    ctx->h[3] = 0x779fbecaU;
    ctx->h[4] = 0x8ce5d491U;
    ctx->h[5] = 0xc0d26243U;
    ctx->h[6] = 0xbafef9eaU;
    ctx->h[7] = 0x1837a9d8U;
    ctx->N = 64;
    vec_zero(ctx->buf, sizeof(ctx->buf));
    ctx->off = 0;
}

static void vec_xor(void *restrict ret, const void *restrict a,
                                        const void *restrict b, size_t num)
{
    limb_t *rp = (limb_t *)ret;
    const limb_t *ap = (const limb_t *)a;
    const limb_t *bp = (const limb_t *)b;
    size_t i;

    num /= sizeof(limb_t);

    for (i = 0; i < num; i++)
        rp[i] = ap[i] ^ bp[i];
}

static void expand_message_xmd(unsigned char *bytes, size_t len_in_bytes,
                               const unsigned char *aug, size_t aug_len,
                               const unsigned char *msg, size_t msg_len,
                               const unsigned char *DST, size_t DST_len)
{
    union { limb_t align; unsigned char c[32]; } b_0;
    union { limb_t align; unsigned char c[33+256+31]; } b_i;
    unsigned char *p;
    size_t i, b_i_bits, b_i_blocks;
    SHA256_CTX ctx;

    /*
     * compose template for 'strxor(b_0, b_(i-1)) || I2OSP(i, 1) || DST_prime'
     */
    if (DST_len > 255) {
        sha256_init(&ctx);
        sha256_update(&ctx, "H2C-OVERSIZE-DST-", 17);
        sha256_update(&ctx, DST, DST_len);
        sha256_final(b_0.c, &ctx);
        DST = b_0.c, DST_len = 32;
    }
    b_i_blocks = ((33 + DST_len + 1 + 9) + 63) & -64;
    vec_zero(b_i.c + b_i_blocks - 64, 64);

    p = b_i.c + 33;
    for (i = 0; i < DST_len; i++)
        p[i] = DST[i];
    p[i++] = (unsigned char)DST_len;
    p[i++] = 0x80;
    p[i+6] = p[i+5] = p[i+4] = p[i+3] = p[i+2] = p[i+1] = p[i+0] = 0;
    b_i_bits = (33 + DST_len + 1) * 8;
    p = b_i.c + b_i_blocks;
    p[-2] = (unsigned char)(b_i_bits >> 8);
    p[-1] = (unsigned char)(b_i_bits);

    sha256_init_Zpad(&ctx);                         /* Z_pad | */
    sha256_update(&ctx, aug, aug_len);              /* | aug | */
    sha256_update(&ctx, msg, msg_len);              /* | msg | */
    /* | I2OSP(len_in_bytes, 2) || I2OSP(0, 1) || DST_prime    */
    b_i.c[30] = (unsigned char)(len_in_bytes >> 8);
    b_i.c[31] = (unsigned char)(len_in_bytes);
    b_i.c[32] = 0;
    sha256_update(&ctx, b_i.c + 30, 3 + DST_len + 1);
    sha256_final(b_0.c, &ctx);

    sha256_init_h(ctx.h);
    vec_copy(b_i.c, b_0.c, 32);
    ++b_i.c[32];
    sha256_block_data_order(ctx.h, b_i.c, b_i_blocks / 64);
    sha256_emit(bytes, ctx.h);

    len_in_bytes += 31; /* ell = ceil(len_in_bytes / b_in_bytes), with */
    len_in_bytes /= 32; /* caller being responsible for accordingly large
                         * buffer. hash_to_field passes one with length
                         * divisible by 64, remember? which works... */
    while (--len_in_bytes) {
        sha256_init_h(ctx.h);
        vec_xor(b_i.c, b_0.c, bytes, 32);
        bytes += 32;
        ++b_i.c[32];
        sha256_block_data_order(ctx.h, b_i.c, b_i_blocks / 64);
        sha256_emit(bytes, ctx.h);
    }
}
#endif

/*
 * |nelems| is 'count * m' from spec
 */
static void hash_to_field(vec384 elems[], size_t nelems,
                          const unsigned char *aug, size_t aug_len,
                          const unsigned char *msg, size_t msg_len,
                          const unsigned char *DST, size_t DST_len)
{
    size_t L = sizeof(vec384) + 128/8;  /* ceil((ceil(log2(p)) + k) / 8) */
    size_t len_in_bytes = L * nelems;   /* divisible by 64, hurray!      */
#if !defined(__STDC_VERSION__) || __STDC_VERSION__<199901 \
                               || defined(__STDC_NO_VLA__)
    limb_t *pseudo_random = alloca(len_in_bytes);
#else
    limb_t pseudo_random[len_in_bytes/sizeof(limb_t)];
#endif
    unsigned char *bytes;
    vec768 elem;

    aug_len = aug!=NULL ? aug_len : 0;
    DST_len = DST!=NULL ? DST_len : 0;

    expand_message_xmd((unsigned char *)pseudo_random, len_in_bytes,
                       aug, aug_len, msg, msg_len, DST, DST_len);

    vec_zero(elem, sizeof(elem));
    bytes = (unsigned char *)pseudo_random;
    while (nelems--) {
        limbs_from_be_bytes(elem, bytes, L);
        bytes += L;
        /*
         * L-bytes block % P, output is in Montgomery domain...
         */
        redc_mont_384(elems[0], elem, BLS12_381_P, p0);
        mul_mont_384(elems[0], elems[0], BLS12_381_RRRR, BLS12_381_P, p0);
        elems++;
    }
}

void blst_expand_message_xmd(unsigned char *bytes, size_t len_in_bytes,
                             const unsigned char *msg, size_t msg_len,
                             const unsigned char *DST, size_t DST_len)
{
    size_t buf_len = (len_in_bytes+31) & ((size_t)0-32);
    unsigned char *buf_ptr = bytes;

    if (buf_len > 255*32)
        return;

    if (buf_len != len_in_bytes)
        buf_ptr = alloca(buf_len);

    expand_message_xmd(buf_ptr, len_in_bytes, NULL, 0, msg, msg_len,
                                              DST, DST_len);
    if (buf_ptr != bytes) {
        unsigned char *ptr = buf_ptr;
        while (len_in_bytes--)
            *bytes++ = *ptr++;
        vec_zero(buf_ptr, buf_len);
    }
}
