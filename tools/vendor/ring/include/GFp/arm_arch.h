/* ====================================================================
 * Copyright (c) 1998-2011 The OpenSSL Project.  All rights reserved.
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

#ifndef OPENSSL_HEADER_ARM_ARCH_H
#define OPENSSL_HEADER_ARM_ARCH_H

#if !defined(__ARM_ARCH__)
# if defined(__CC_ARM)
#  define __ARM_ARCH__ __TARGET_ARCH_ARM
#  if defined(__BIG_ENDIAN)
#   define __ARMEB__
#  else
#   define __ARMEL__
#  endif
# elif defined(__GNUC__)
#  if defined(__aarch64__)
#    define __ARM_ARCH__ 8
#    if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#      define __ARMEB__
#    else
#      define __ARMEL__
#    endif
  // Why doesn't gcc define __ARM_ARCH__? Instead it defines
  // bunch of below macros. See all_architectires[] table in
  // gcc/config/arm/arm.c. On a side note it defines
  // __ARMEL__/__ARMEB__ for little-/big-endian.
#  elif	defined(__ARM_ARCH)
#    define __ARM_ARCH__ __ARM_ARCH
#  elif	defined(__ARM_ARCH_8A__)
#    define __ARM_ARCH__ 8
#  elif	defined(__ARM_ARCH_7__)	|| defined(__ARM_ARCH_7A__)	|| \
	defined(__ARM_ARCH_7R__)|| defined(__ARM_ARCH_7M__)	|| \
	defined(__ARM_ARCH_7EM__)
#   define __ARM_ARCH__ 7
#  elif	defined(__ARM_ARCH_6__)	|| defined(__ARM_ARCH_6J__)	|| \
	defined(__ARM_ARCH_6K__)|| defined(__ARM_ARCH_6M__)	|| \
	defined(__ARM_ARCH_6Z__)|| defined(__ARM_ARCH_6ZK__)	|| \
	defined(__ARM_ARCH_6T2__)
#   define __ARM_ARCH__ 6
#  elif	defined(__ARM_ARCH_5__)	|| defined(__ARM_ARCH_5T__)	|| \
	defined(__ARM_ARCH_5E__)|| defined(__ARM_ARCH_5TE__)	|| \
	defined(__ARM_ARCH_5TEJ__)
#   define __ARM_ARCH__ 5
#  elif	defined(__ARM_ARCH_4__)	|| defined(__ARM_ARCH_4T__)
#   define __ARM_ARCH__ 4
#  else
#   error "unsupported ARM architecture"
#  endif
# endif
#endif

// Even when building for 32-bit ARM, support for aarch64 crypto instructions
// will be included.
#if !defined(__ARM_MAX_ARCH__)
#define __ARM_MAX_ARCH__ 8
#endif

// ARMV7_NEON is true when a NEON unit is present in the current CPU.
#define ARMV7_NEON (1 << 0)

// ARMV8_SHA256 indicates support for hardware SHA-256 instructions.
#define ARMV8_SHA256 (1 << 4)

#if defined(__ASSEMBLER__)

#if defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wundef"
#endif

// Support macros for
//   - Armv8.3-A Pointer Authentication and
//   - Armv8.5-A Branch Target Identification
// features which require emitting a .note.gnu.property section with the
// appropriate architecture-dependent feature bits set.
// Read more: "ELF for the Arm® 64-bit Architecture"

#if defined(__ARM_FEATURE_BTI_DEFAULT) && __ARM_FEATURE_BTI_DEFAULT == 1
#define GNU_PROPERTY_AARCH64_BTI (1 << 0)   // Has Branch Target Identification
#define AARCH64_VALID_CALL_TARGET hint #34  // BTI 'c'
#else
#define GNU_PROPERTY_AARCH64_BTI 0  // No Branch Target Identification
#define AARCH64_VALID_CALL_TARGET
#endif

#if defined(__ARM_FEATURE_PAC_DEFAULT) && \
    (__ARM_FEATURE_PAC_DEFAULT & 1) == 1  // Signed with A-key
#define GNU_PROPERTY_AARCH64_POINTER_AUTH \
  (1 << 1)                                       // Has Pointer Authentication
#define AARCH64_SIGN_LINK_REGISTER hint #25      // PACIASP
#define AARCH64_VALIDATE_LINK_REGISTER hint #29  // AUTIASP
#elif defined(__ARM_FEATURE_PAC_DEFAULT) && \
    (__ARM_FEATURE_PAC_DEFAULT & 2) == 2  // Signed with B-key
#define GNU_PROPERTY_AARCH64_POINTER_AUTH \
  (1 << 1)                                       // Has Pointer Authentication
#define AARCH64_SIGN_LINK_REGISTER hint #27      // PACIBSP
#define AARCH64_VALIDATE_LINK_REGISTER hint #31  // AUTIBSP
#else
#define GNU_PROPERTY_AARCH64_POINTER_AUTH 0  // No Pointer Authentication
#if GNU_PROPERTY_AARCH64_BTI != 0
#define AARCH64_SIGN_LINK_REGISTER AARCH64_VALID_CALL_TARGET
#else
#define AARCH64_SIGN_LINK_REGISTER
#endif
#define AARCH64_VALIDATE_LINK_REGISTER
#endif

#if GNU_PROPERTY_AARCH64_POINTER_AUTH != 0 || GNU_PROPERTY_AARCH64_BTI != 0
.pushsection .note.gnu.property, "a";
.balign 8;
.long 4;
.long 0x10;
.long 0x5;
.asciz "GNU";
.long 0xc0000000; /* GNU_PROPERTY_AARCH64_FEATURE_1_AND */
.long 4;
.long (GNU_PROPERTY_AARCH64_POINTER_AUTH | GNU_PROPERTY_AARCH64_BTI);
.long 0;
.popsection;
#endif

#if defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

#endif  /* defined __ASSEMBLER__ */

#endif  // OPENSSL_HEADER_ARM_ARCH_H
