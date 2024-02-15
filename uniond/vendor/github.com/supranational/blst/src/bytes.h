/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLS12_381_ASM_BYTES_H__
#define __BLS12_381_ASM_BYTES_H__

static inline void bytes_zero(unsigned char *a, size_t num)
{
    size_t i;

    for (i = 0; i < num; i++)
        a[i] = 0;
}

static inline void limbs_from_be_bytes(limb_t *restrict ret,
                                       const unsigned char *in, size_t n)
{
    limb_t limb = 0;

    while(n--) {
        limb <<= 8;
        limb |= *in++;
        /*
         * 'if (n % sizeof(limb_t) == 0)' is omitted because it's cheaper
         * to perform redundant stores than to pay penalty for
         * mispredicted branch. Besides, some compilers unroll the
         * loop and remove redundant stores to 'restrict'-ed storage...
         */
        ret[n / sizeof(limb_t)] = limb;
    }
}

static inline void be_bytes_from_limbs(unsigned char *out, const limb_t *in,
                                       size_t n)
{
    limb_t limb;

    while(n--) {
        limb = in[n / sizeof(limb_t)];
        *out++ = (unsigned char)(limb >> (8 * (n % sizeof(limb_t))));
    }
}

static inline void limbs_from_le_bytes(limb_t *restrict ret,
                                       const unsigned char *in, size_t n)
{
    limb_t limb = 0;

    while(n--) {
        limb <<= 8;
        limb |= in[n];
        /*
         * 'if (n % sizeof(limb_t) == 0)' is omitted because it's cheaper
         * to perform redundant stores than to pay penalty for
         * mispredicted branch. Besides, some compilers unroll the
         * loop and remove redundant stores to 'restrict'-ed storage...
         */
        ret[n / sizeof(limb_t)] = limb;
    }
}

static inline void le_bytes_from_limbs(unsigned char *out, const limb_t *in,
                                       size_t n)
{
    const union {
        long one;
        char little;
    } is_endian = { 1 };
    limb_t limb;
    size_t i, j, r;

    if ((uptr_t)out == (uptr_t)in && is_endian.little)
        return;

    r = n % sizeof(limb_t);
    n /= sizeof(limb_t);

    for(i = 0; i < n; i++) {
        for (limb = in[i], j = 0; j < sizeof(limb_t); j++, limb >>= 8)
            *out++ = (unsigned char)limb;
    }
    if (r) {
        for (limb = in[i], j = 0; j < r; j++, limb >>= 8)
            *out++ = (unsigned char)limb;
    }
}

static inline char hex_from_nibble(unsigned char nibble)
{
    int mask = (9 - (nibble &= 0xf)) >> 31;
    return (char)(nibble + ((('a'-10) & mask) | ('0' & ~mask)));
}

static unsigned char nibble_from_hex(char c)
{
    int mask, ret;

    mask = (('a'-c-1) & (c-1-'f')) >> 31;
    ret  = (10 + c - 'a') & mask;
    mask = (('A'-c-1) & (c-1-'F')) >> 31;
    ret |= (10 + c - 'A') & mask;
    mask = (('0'-c-1) & (c-1-'9')) >> 31;
    ret |= (c - '0') & mask;
    mask = ((ret-1) & ~mask) >> 31;
    ret |= 16 & mask;

    return (unsigned char)ret;
}

static void bytes_from_hexascii(unsigned char *ret, size_t sz, const char *hex)
{
    size_t len;
    unsigned char b = 0;

    if (hex[0]=='0' && (hex[1]=='x' || hex[1]=='X'))
        hex += 2;

    for (len = 0; len<2*sz && nibble_from_hex(hex[len])<16; len++) ;

    bytes_zero(ret, sz);

    while(len--) {
        b <<= 4;
        b |= nibble_from_hex(*hex++);
        if (len % 2 == 0)
            ret[len / 2] = b;
    }
}

static void limbs_from_hexascii(limb_t *ret, size_t sz, const char *hex)
{
    size_t len;
    limb_t limb = 0;

    if (hex[0]=='0' && (hex[1]=='x' || hex[1]=='X'))
        hex += 2;

    for (len = 0; len<2*sz && nibble_from_hex(hex[len])<16; len++) ;

    vec_zero(ret, sz);

    while(len--) {
        limb <<= 4;
        limb |= nibble_from_hex(*hex++);
        if (len % (2*sizeof(limb_t)) == 0)
            ret[len / (2*sizeof(limb_t))] = limb;
    }
}

#endif
