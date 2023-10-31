/* Copyright (C) 1995-1997 Eric Young (eay@cryptsoft.com)
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
 * Copyright (c) 1998-2006 The OpenSSL Project.  All rights reserved.
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
 * Hudson (tjh@cryptsoft.com).
 *
 */
/* ====================================================================
 * Copyright 2002 Sun Microsystems, Inc. ALL RIGHTS RESERVED.
 *
 * Portions of the attached software ("Contribution") are developed by
 * SUN MICROSYSTEMS, INC., and are contributed to the OpenSSL project.
 *
 * The Contribution is licensed pursuant to the Eric Young open source
 * license provided above.
 *
 * The binary polynomial arithmetic software is originally written by
 * Sheueling Chang Shantz and Douglas Stebila of Sun Microsystems
 * Laboratories. */

#ifndef OPENSSL_HEADER_BN_INTERNAL_H
#define OPENSSL_HEADER_BN_INTERNAL_H

#include <GFp/base.h>

#if defined(OPENSSL_X86_64) && defined(_MSC_VER) && !defined(__clang__)
#pragma warning(push, 3)
#include <intrin.h>
#pragma warning(pop)
#pragma intrinsic(_umul128)
#endif

#include "../../internal.h"

typedef crypto_word BN_ULONG;

#if defined(OPENSSL_64_BIT)

#if defined(BORINGSSL_HAS_UINT128)
// MSVC doesn't support two-word integers on 64-bit.
#define BN_ULLONG uint128_t
#endif

#define BN_BITS2 64
#define BN_MONT_CTX_N0_LIMBS 1
#define BN_MONT_CTX_N0(hi, lo) TOBN(hi, lo), 0
#define TOBN(hi, lo) ((BN_ULONG)(hi) << 32 | (lo))

#elif defined(OPENSSL_32_BIT)

#define BN_ULLONG uint64_t
#define BN_BITS2 32
// On some 32-bit platforms, Montgomery multiplication is done using 64-bit
// arithmetic with SIMD instructions. On such platforms, |BN_MONT_CTX::n0|
// needs to be two words long. Only certain 32-bit platforms actually make use
// of n0[1] and shorter R value would suffice for the others. However,
// currently only the assembly files know which is which.
#define BN_MONT_CTX_N0_LIMBS 2
#define BN_MONT_CTX_N0(hi, lo) TOBN(hi, lo)
#define TOBN(hi, lo) (lo), (hi)

#else
#error "Must define either OPENSSL_32_BIT or OPENSSL_64_BIT"
#endif


// |num| must be at least 4, at least on x86.
//
// In other forks, |bn_mul_mont| returns an |int| indicating whether it
// actually did the multiplication. All our implementations always do the
// multiplication, and forcing callers to deal with the possibility of it
// failing just leads to further problems.
//
// In other forks, |bn_mod_mul|'s `num` argument has type |int| but it is
// implicitly treated as a |size_t|; when |int| is smaller than |size_t|
// then the |movq 48(%rsp),%r9| done by x86_64-xlate.pl implicitly does the
// conversion.
OPENSSL_STATIC_ASSERT(sizeof(int) == sizeof(size_t) ||
                      (sizeof(int) == 4 && sizeof(size_t) == 8),
                      "int and size_t ABI mismatch");
void GFp_bn_mul_mont(BN_ULONG *rp, const BN_ULONG *ap, const BN_ULONG *bp,
                     const BN_ULONG *np, const BN_ULONG *n0, size_t num);

static inline void bn_umult_lohi(BN_ULONG *low_out, BN_ULONG *high_out,
                                 BN_ULONG a, BN_ULONG b) {
#if defined(OPENSSL_X86_64) && defined(_MSC_VER) && !defined(__clang__)
  *low_out = _umul128(a, b, high_out);
#else
  BN_ULLONG result = (BN_ULLONG)a * b;
  *low_out = (BN_ULONG)result;
  *high_out = (BN_ULONG)(result >> BN_BITS2);
#endif
}

#endif  // OPENSSL_HEADER_BN_INTERNAL_H
