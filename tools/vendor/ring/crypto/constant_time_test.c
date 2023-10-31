/*
 * Utilities for constant-time cryptography.
 *
 * Author: Emilia Kasper (emilia@openssl.org)
 * Based on previous work by Bodo Moeller, Emilia Kasper, Adam Langley
 * (Google).
 * ====================================================================
 * Copyright (c) 2014 The OpenSSL Project.  All rights reserved.
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

#include "internal.h"

int bssl_constant_time_test_main(void);

static int test_binary_op_w(crypto_word (*op)(crypto_word a, crypto_word b),
                            crypto_word a, crypto_word b, int is_true) {
  crypto_word c = op(a, b);
  if (is_true && c != CONSTTIME_TRUE_W) {
    return 1;
  } else if (!is_true && c != CONSTTIME_FALSE_W) {
    return 1;
  }
  return 0;
}

static int test_is_zero_w(crypto_word a) {
  crypto_word c = constant_time_is_zero_w(a);
  if (a == 0 && c != CONSTTIME_TRUE_W) {
    return 1;
  } else if (a != 0 && c != CONSTTIME_FALSE_W) {
    return 1;
  }

  c = constant_time_is_nonzero_w(a);
  if (a == 0 && c != CONSTTIME_FALSE_W) {
    return 1;
  } else if (a != 0 && c != CONSTTIME_TRUE_W) {
    return 1;
  }

  return 0;
}

static int test_select_w(crypto_word a, crypto_word b) {
  crypto_word selected = constant_time_select_w(CONSTTIME_TRUE_W, a, b);
  if (selected != a) {
    return 1;
  }
  selected = constant_time_select_w(CONSTTIME_FALSE_W, a, b);
  if (selected != b) {
    return 1;
  }
  return 0;
}

static crypto_word test_values_s[] = {
  0,
  1,
  1024,
  12345,
  32000,
#if defined(OPENSSL_64_BIT)
  0xffffffff / 2 - 1,
  0xffffffff / 2,
  0xffffffff / 2 + 1,
  0xffffffff - 1,
  0xffffffff,
#endif
  SIZE_MAX / 2 - 1,
  SIZE_MAX / 2,
  SIZE_MAX / 2 + 1,
  SIZE_MAX - 1,
  SIZE_MAX
};

int bssl_constant_time_test_main(void) {
  int num_failed = 0;

  for (size_t i = 0;
       i < sizeof(test_values_s) / sizeof(test_values_s[0]); ++i) {
    crypto_word a = test_values_s[i];
    num_failed += test_is_zero_w(a);
    for (size_t j = 0;
         j < sizeof(test_values_s) / sizeof(test_values_s[0]); ++j) {
      crypto_word b = test_values_s[j];
      num_failed += test_binary_op_w(&constant_time_eq_w, a, b, a == b);
      num_failed += test_binary_op_w(&constant_time_eq_w, b, a, b == a);
      num_failed += test_select_w(a, b);
    }
  }

  return num_failed == 0;
}
