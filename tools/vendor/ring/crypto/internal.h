/* Copyright (C) 1995-1998 Eric Young (eay@cryptsoft.com)
 * All rights reserved.
 *
 * This package is an SSL implementation written
 * by Eric Young (eay@cryptsoft.com).
 * The implementation was written so as to conform with Netscapes SSL.
 *
 * This library is free for commercial and non-commercial use as long as
 * the following conditions are aheared to.  The following conditions
 * apply to all code found in this distribution, be it the RC4, RSA,
 * lhash, DES, etc., code; not just the SSL code.  The SSL documentation
 * included with this distribution is covered by the same copyright terms
 * except that the holder is Tim Hudson (tjh@cryptsoft.com).
 *
 * Copyright remains Eric Young's, and as such any Copyright notices in
 * the code are not to be removed.
 * If this package is used in a product, Eric Young should be given attribution
 * as the author of the parts of the library used.
 * This can be in the form of a textual message at program startup or
 * in documentation (online or textual) provided with the package.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. All advertising materials mentioning features or use of this software
 *    must display the following acknowledgement:
 *    "This product includes cryptographic software written by
 *     Eric Young (eay@cryptsoft.com)"
 *    The word 'cryptographic' can be left out if the rouines from the library
 *    being used are not cryptographic related :-).
 * 4. If you include any Windows specific code (or a derivative thereof) from
 *    the apps directory (application code) you must include an acknowledgement:
 *    "This product includes software written by Tim Hudson (tjh@cryptsoft.com)"
 *
 * THIS SOFTWARE IS PROVIDED BY ERIC YOUNG ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 * The licence and distribution terms for any publically available version or
 * derivative of this code cannot be changed.  i.e. this code cannot simply be
 * copied and put under another distribution licence
 * [including the GNU Public Licence.]
 */
/* ====================================================================
 * Copyright (c) 1998-2001 The OpenSSL Project.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in
 *    the documentation and/or other materials provided with the
 *    distribution.
 *
 * 3. All advertising materials mentioning features or use of this
 *    software must display the following acknowledgment:
 *    "This product includes software developed by the OpenSSL Project
 *    for use in the OpenSSL Toolkit. (http://www.openssl.org/)"
 *
 * 4. The names "OpenSSL Toolkit" and "OpenSSL Project" must not be used to
 *    endorse or promote products derived from this software without
 *    prior written permission. For written permission, please contact
 *    openssl-core@openssl.org.
 *
 * 5. Products derived from this software may not be called "OpenSSL"
 *    nor may "OpenSSL" appear in their names without prior written
 *    permission of the OpenSSL Project.
 *
 * 6. Redistributions of any form whatsoever must retain the following
 *    acknowledgment:
 *    "This product includes software developed by the OpenSSL Project
 *    for use in the OpenSSL Toolkit (http://www.openssl.org/)"
 *
 * THIS SOFTWARE IS PROVIDED BY THE OpenSSL PROJECT ``AS IS'' AND ANY
 * EXPRESSED OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE OpenSSL PROJECT OR
 * ITS CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
 * NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
 * OF THE POSSIBILITY OF SUCH DAMAGE.
 * ====================================================================
 *
 * This product includes cryptographic software written by Eric Young
 * (eay@cryptsoft.com).  This product includes software written by Tim
 * Hudson (tjh@cryptsoft.com). */

#ifndef OPENSSL_HEADER_CRYPTO_INTERNAL_H
#define OPENSSL_HEADER_CRYPTO_INTERNAL_H

#include <GFp/base.h> // Must be first.

#include "GFp/check.h"

#if defined(__GNUC__) && \
    (__GNUC__ * 10000 + __GNUC_MINOR__ * 100 + __GNUC_PATCHLEVEL__) < 40800
// |alignas| and |alignof| were added in C11. GCC added support in version 4.8.
// Testing for __STDC_VERSION__/__cplusplus doesn't work because 4.7 already
// reports support for C11.
#define alignas(x) __attribute__ ((aligned (x)))
#elif defined(_MSC_VER) && !defined(__clang__)
#define alignas(x) __declspec(align(x))
#else
#include <stdalign.h>
#endif

#if (!defined(_MSC_VER) || defined(__clang__)) && defined(OPENSSL_64_BIT)
#define BORINGSSL_HAS_UINT128
typedef __int128_t int128_t;
typedef __uint128_t uint128_t;
#endif


// Constant-time utility functions.
//
// The following methods return a bitmask of all ones (0xff...f) for true and 0
// for false. This is useful for choosing a value based on the result of a
// conditional in constant time. For example,
//
// if (a < b) {
//   c = a;
// } else {
//   c = b;
// }
//
// can be written as
//
// crypto_word lt = constant_time_lt_w(a, b);
// c = constant_time_select_w(lt, a, b);

// crypto_word is the type that most constant-time functions use. Ideally we
// would like it to be |size_t|, but NaCl builds in 64-bit mode with 32-bit
// pointers, which means that |size_t| can be 32 bits when |crypto_word| is 64
// bits.
#if defined(OPENSSL_64_BIT)
typedef uint64_t crypto_word;
#define CRYPTO_WORD_BITS (64u)
#elif defined(OPENSSL_32_BIT)
typedef uint32_t crypto_word;
#define CRYPTO_WORD_BITS (32u)
#else
#error "Must define either OPENSSL_32_BIT or OPENSSL_64_BIT"
#endif

#define CONSTTIME_TRUE_W ~((crypto_word)0)
#define CONSTTIME_FALSE_W ((crypto_word)0)

// value_barrier_w returns |a|, but prevents GCC and Clang from reasoning about
// the returned value. This is used to mitigate compilers undoing constant-time
// code, until we can express our requirements directly in the language.
//
// Note the compiler is aware that |value_barrier_w| has no side effects and
// always has the same output for a given input. This allows it to eliminate
// dead code, move computations across loops, and vectorize.
static inline crypto_word value_barrier_w(crypto_word a) {
#if !defined(OPENSSL_NO_ASM) && (defined(__GNUC__) || defined(__clang__))
  __asm__("" : "+r"(a) : /* no inputs */);
#endif
  return a;
}

// value_barrier_u32 behaves like |value_barrier_w| but takes a |uint32_t|.
static inline uint32_t value_barrier_u32(uint32_t a) {
#if !defined(OPENSSL_NO_ASM) && (defined(__GNUC__) || defined(__clang__))
  __asm__("" : "+r"(a) : /* no inputs */);
#endif
  return a;
}

// value_barrier_u64 behaves like |value_barrier_w| but takes a |uint64_t|.
static inline uint64_t value_barrier_u64(uint64_t a) {
#if !defined(OPENSSL_NO_ASM) && (defined(__GNUC__) || defined(__clang__))
  __asm__("" : "+r"(a) : /* no inputs */);
#endif
  return a;
}

// constant_time_msb_w returns the given value with the MSB copied to all the
// other bits.
static inline crypto_word constant_time_msb_w(crypto_word a) {
  return 0u - (a >> (sizeof(a) * 8 - 1));
}

// constant_time_is_zero_w returns 0xff..f if a == 0 and 0 otherwise.
static inline crypto_word constant_time_is_zero_w(crypto_word a) {
  // Here is an SMT-LIB verification of this formula:
  //
  // (define-fun is_zero ((a (_ BitVec 32))) (_ BitVec 32)
  //   (bvand (bvnot a) (bvsub a #x00000001))
  // )
  //
  // (declare-fun a () (_ BitVec 32))
  //
  // (assert (not (= (= #x00000001 (bvlshr (is_zero a) #x0000001f)) (= a #x00000000))))
  // (check-sat)
  // (get-model)
  return constant_time_msb_w(~a & (a - 1));
}

static inline crypto_word constant_time_is_nonzero_w(crypto_word a) {
  return ~constant_time_is_zero_w(a);
}

// constant_time_eq_w returns 0xff..f if a == b and 0 otherwise.
static inline crypto_word constant_time_eq_w(crypto_word a,
                                               crypto_word b) {
  return constant_time_is_zero_w(a ^ b);
}

// constant_time_select_w returns (mask & a) | (~mask & b). When |mask| is all
// 1s or all 0s (as returned by the methods above), the select methods return
// either |a| (if |mask| is nonzero) or |b| (if |mask| is zero).
static inline crypto_word constant_time_select_w(crypto_word mask,
                                                   crypto_word a,
                                                   crypto_word b) {
  // Clang recognizes this pattern as a select. While it usually transforms it
  // to a cmov, it sometimes further transforms it into a branch, which we do
  // not want.
  //
  // Adding barriers to both |mask| and |~mask| breaks the relationship between
  // the two, which makes the compiler stick with bitmasks.
  return (value_barrier_w(mask) & a) | (value_barrier_w(~mask) & b);
}

// Endianness conversions.

#if defined(__GNUC__) && __GNUC__ >= 2
static inline uint32_t CRYPTO_bswap4(uint32_t x) {
  return __builtin_bswap32(x);
}
#elif defined(_MSC_VER)
#pragma warning(push, 3)
#include <stdlib.h>
#pragma warning(pop)
#pragma intrinsic(_byteswap_uint64, _byteswap_ulong)
static inline uint32_t CRYPTO_bswap4(uint32_t x) {
  return _byteswap_ulong(x);
}
#endif

#if !defined(GFp_NOSTDLIBINC)
#include <string.h>
#endif

static inline void *GFp_memcpy(void *dst, const void *src, size_t n) {
#if !defined(GFp_NOSTDLIBINC)
  if (n == 0) {
    return dst;
  }
  return memcpy(dst, src, n);
#else
  unsigned char *d = dst;
  const unsigned char *s = src;
  for (size_t i = 0; i < n; ++i) {
    d[i] = s[i];
  }
  return dst;
#endif
}

static inline void *GFp_memset(void *dst, int c, size_t n) {
#if !defined(GFp_NOSTDLIBINC)
  if (n == 0) {
    return dst;
  }
  return memset(dst, c, n);
#else
  unsigned char *d = dst;
  for (size_t i = 0; i < n; ++i) {
    d[i] = (unsigned char)c;
  }
  return dst;
#endif
}

#endif  // OPENSSL_HEADER_CRYPTO_INTERNAL_H
