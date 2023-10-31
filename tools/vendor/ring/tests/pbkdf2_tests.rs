// Copyright 2015-2017 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use core::num::NonZeroU32;
use ring::{digest, error, pbkdf2, test, test_file};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

/// Test vectors from BoringSSL, Go, and other sources.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
pub fn pbkdf2_tests() {
    test::run(test_file!("pbkdf2_tests.txt"), |section, test_case| {
        assert_eq!(section, "");
        let algorithm = {
            let digest_alg = test_case.consume_digest_alg("Hash").unwrap();
            if digest_alg == &digest::SHA1_FOR_LEGACY_USE_ONLY {
                pbkdf2::PBKDF2_HMAC_SHA1
            } else if digest_alg == &digest::SHA256 {
                pbkdf2::PBKDF2_HMAC_SHA256
            } else if digest_alg == &digest::SHA384 {
                pbkdf2::PBKDF2_HMAC_SHA384
            } else if digest_alg == &digest::SHA512 {
                pbkdf2::PBKDF2_HMAC_SHA512
            } else {
                unreachable!()
            }
        };
        let iterations = test_case.consume_usize("c");
        let iterations = NonZeroU32::new(iterations as u32).unwrap();
        let secret = test_case.consume_bytes("P");
        let salt = test_case.consume_bytes("S");
        let dk = test_case.consume_bytes("DK");
        let verify_expected_result = test_case.consume_string("Verify");
        let verify_expected_result = match verify_expected_result.as_str() {
            "OK" => Ok(()),
            "Err" => Err(error::Unspecified),
            _ => panic!("Unsupported value of \"Verify\""),
        };

        {
            let mut out = vec![0u8; dk.len()];
            pbkdf2::derive(algorithm, iterations, &salt, &secret, &mut out);
            assert_eq!(dk == out, verify_expected_result.is_ok() || dk.is_empty());
        }

        #[cfg(any(not(target_arch = "wasm32"), feature = "wasm32_c"))]
        assert_eq!(
            pbkdf2::verify(algorithm, iterations, &salt, &secret, &dk),
            verify_expected_result
        );

        Ok(())
    });
}
