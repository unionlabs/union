// Copyright 2020 Brian Smith.
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

#![cfg(any(not(target_arch = "wasm32"), feature = "wasm32_c"))]
use ring::{constant_time, error, rand};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

// This logic is loosly based on BoringSSL's `TEST(ConstantTimeTest, MemCmp)`.
#[test]
#[cfg_attr(all(target_arch = "wasm32", feature = "wasm32_c"), wasm_bindgen_test)]
fn test_verify_slices_are_equal() {
    let initial: [u8; 256] = rand::generate(&rand::SystemRandom::new()).unwrap().expose();

    {
        let copy = initial;
        for len in 0..copy.len() {
            // Not equal because the lengths do not match.
            assert_eq!(
                constant_time::verify_slices_are_equal(&initial, &copy[..len]),
                Err(error::Unspecified)
            );
            // Equal lengths and equal contents.
            assert_eq!(
                constant_time::verify_slices_are_equal(&initial[..len], &copy[..len]),
                Ok(())
            );
        }
        // Equal lengths and equal contents.
        assert_eq!(
            constant_time::verify_slices_are_equal(&initial, &copy),
            Ok(())
        );
    }

    for i in 0..initial.len() {
        for bit in 0..8 {
            let mut copy = initial;
            copy[i] ^= 1u8 << bit;

            for len in 0..=initial.len() {
                // We flipped at least one bit in `copy`.
                assert_ne!(&initial[..], &copy[..]);

                let a = &initial[..len];
                let b = &copy[..len];

                let expected_result = if i < len {
                    // The flipped bit is within `b` so `a` and `b` are not equal.
                    Err(error::Unspecified)
                } else {
                    // The flipped bit is outside of `b` so `a` and `b` are equal.
                    Ok(())
                };
                assert_eq!(a == b, expected_result.is_ok()); // Sanity check.
                assert_eq!(
                    constant_time::verify_slices_are_equal(&a, &b),
                    expected_result
                );
                assert_eq!(
                    constant_time::verify_slices_are_equal(&b, &a),
                    expected_result
                );
            }
        }
    }
}
