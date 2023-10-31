// Copyright 2015-2016 Brian Smith.
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

use ring::{digest, error, hmac, test, test_file};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn hmac_tests() {
    test::run(test_file!("hmac_tests.txt"), |section, test_case| {
        assert_eq!(section, "");
        let digest_alg = test_case.consume_digest_alg("HMAC");
        let key_value = test_case.consume_bytes("Key");
        let mut input = test_case.consume_bytes("Input");
        let output = test_case.consume_bytes("Output");

        let algorithm = {
            let digest_alg = match digest_alg {
                Some(digest_alg) => digest_alg,
                None => {
                    return Ok(());
                } // Unsupported digest algorithm
            };
            if digest_alg == &digest::SHA1_FOR_LEGACY_USE_ONLY {
                hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY
            } else if digest_alg == &digest::SHA256 {
                hmac::HMAC_SHA256
            } else if digest_alg == &digest::SHA384 {
                hmac::HMAC_SHA384
            } else if digest_alg == &digest::SHA512 {
                hmac::HMAC_SHA512
            } else {
                unreachable!()
            }
        };

        hmac_test_case_inner(algorithm, &key_value[..], &input[..], &output[..], true)?;

        // Tamper with the input and check that verification fails.
        if input.is_empty() {
            input.push(0);
        } else {
            input[0] ^= 1;
        }

        hmac_test_case_inner(algorithm, &key_value[..], &input[..], &output[..], false)
    });
}

fn hmac_test_case_inner(
    algorithm: hmac::Algorithm,
    key_value: &[u8],
    input: &[u8],
    output: &[u8],
    is_ok: bool,
) -> Result<(), error::Unspecified> {
    let key = hmac::Key::new(algorithm, key_value);

    // One-shot API.
    {
        let signature = hmac::sign(&key, input);
        assert_eq!(is_ok, signature.as_ref() == output);

        #[cfg(any(not(target_arch = "wasm32"), feature = "wasm32_c"))]
        assert_eq!(is_ok, hmac::verify(&key, input, output).is_ok());
    }

    // Multi-part API, one single part.
    {
        let mut s_ctx = hmac::Context::with_key(&key);
        s_ctx.update(input);
        let signature = s_ctx.sign();
        assert_eq!(is_ok, signature.as_ref() == output);
    }

    // Multi-part API, byte by byte.
    {
        let mut ctx = hmac::Context::with_key(&key);
        for b in input {
            ctx.update(&[*b]);
        }
        let signature = ctx.sign();
        assert_eq!(is_ok, signature.as_ref() == output);
    }

    Ok(())
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn hmac_debug() {
    let key = hmac::Key::new(hmac::HMAC_SHA256, &[0; 32]);
    assert_eq!("Key { algorithm: SHA256 }", format!("{:?}", &key));

    let ctx = hmac::Context::with_key(&key);
    assert_eq!("Context { algorithm: SHA256 }", format!("{:?}", &ctx));
}
