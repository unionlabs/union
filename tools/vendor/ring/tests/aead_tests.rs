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

#![cfg(any(not(target_arch = "wasm32"), feature = "wasm32_c"))]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

use core::ops::RangeFrom;
use ring::{aead, error, test, test_file};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn aead_aes_gcm_128() {
    test_aead(
        &aead::AES_128_GCM,
        seal_with_key,
        open_with_key,
        test_file!("aead_aes_128_gcm_tests.txt"),
    );
    test_aead(
        &aead::AES_128_GCM,
        seal_with_less_safe_key,
        open_with_less_safe_key,
        test_file!("aead_aes_128_gcm_tests.txt"),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn aead_aes_gcm_256() {
    test_aead(
        &aead::AES_256_GCM,
        seal_with_key,
        open_with_key,
        test_file!("aead_aes_256_gcm_tests.txt"),
    );
    test_aead(
        &aead::AES_256_GCM,
        seal_with_less_safe_key,
        open_with_less_safe_key,
        test_file!("aead_aes_256_gcm_tests.txt"),
    );
}

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "x86_64",
    target_arch = "x86"
))]
#[test]
fn aead_chacha20_poly1305() {
    test_aead(
        &aead::CHACHA20_POLY1305,
        seal_with_key,
        open_with_key,
        test_file!("aead_chacha20_poly1305_tests.txt"),
    );
    test_aead(
        &aead::CHACHA20_POLY1305,
        seal_with_less_safe_key,
        open_with_less_safe_key,
        test_file!("aead_chacha20_poly1305_tests.txt"),
    );
}

fn test_aead<Seal, Open>(
    aead_alg: &'static aead::Algorithm,
    seal: Seal,
    open: Open,
    test_file: test::File,
) where
    Seal: Fn(
        &'static aead::Algorithm,
        &[u8],
        aead::Nonce,
        aead::Aad<&[u8]>,
        &mut Vec<u8>,
    ) -> Result<(), error::Unspecified>,
    Open: for<'a> Fn(
        &'static aead::Algorithm,
        &[u8],
        aead::Nonce,
        aead::Aad<&[u8]>,
        &'a mut [u8],
        RangeFrom<usize>,
    ) -> Result<&'a mut [u8], error::Unspecified>,
{
    test_aead_key_sizes(aead_alg);

    test::run(test_file, |section, test_case| {
        assert_eq!(section, "");
        let key_bytes = test_case.consume_bytes("KEY");
        let nonce_bytes = test_case.consume_bytes("NONCE");
        let plaintext = test_case.consume_bytes("IN");
        let aad = test_case.consume_bytes("AD");
        let mut ct = test_case.consume_bytes("CT");
        let tag = test_case.consume_bytes("TAG");
        let error = test_case.consume_optional_string("FAILS");

        match &error {
            Some(err) if err == "WRONG_NONCE_LENGTH" => {
                assert!(aead::Nonce::try_assume_unique_for_key(&nonce_bytes).is_err());
                return Ok(());
            }
            _ => (),
        };

        let mut s_in_out = plaintext.clone();
        let nonce = aead::Nonce::try_assume_unique_for_key(&nonce_bytes).unwrap();
        let s_result = seal(
            aead_alg,
            &key_bytes[..],
            nonce,
            aead::Aad::from(&aad[..]),
            &mut s_in_out,
        );

        ct.extend(tag);

        if s_result.is_ok() {
            assert_eq!(&ct, &s_in_out);
        }

        // In release builds, test all prefix lengths from 0 to 4096 bytes.
        // Debug builds are too slow for this, so for those builds, only
        // test a smaller subset.

        // TLS record headers are 5 bytes long.
        // TLS explicit nonces for AES-GCM are 8 bytes long.
        static MINIMAL_IN_PREFIX_LENS: [usize; 36] = [
            // No input prefix to overwrite; i.e. the opening is exactly
            // "in place."
            0,
            1,
            2,
            // Proposed TLS 1.3 header (no explicit nonce).
            5,
            8,
            // Probably the most common use of a non-zero `in_prefix_len`
            // would be to write a decrypted TLS record over the top of the
            // TLS header and nonce.
            5 /* record header */ + 8, /* explicit nonce */
            // The stitched AES-GCM x86-64 code works on 6-block (96 byte)
            // units. Some of the ChaCha20 code is even weirder.
            15,  // The maximum partial AES block.
            16,  // One AES block.
            17,  // One byte more than a full AES block.
            31,  // 2 AES blocks or 1 ChaCha20 block, minus 1.
            32,  // Two AES blocks, one ChaCha20 block.
            33,  // 2 AES blocks or 1 ChaCha20 block, plus 1.
            47,  // Three AES blocks - 1.
            48,  // Three AES blocks.
            49,  // Three AES blocks + 1.
            63,  // Four AES blocks or two ChaCha20 blocks, minus 1.
            64,  // Four AES blocks or two ChaCha20 blocks.
            65,  // Four AES blocks or two ChaCha20 blocks, plus 1.
            79,  // Five AES blocks, minus 1.
            80,  // Five AES blocks.
            81,  // Five AES blocks, plus 1.
            95,  // Six AES blocks or three ChaCha20 blocks, minus 1.
            96,  // Six AES blocks or three ChaCha20 blocks.
            97,  // Six AES blocks or three ChaCha20 blocks, plus 1.
            111, // Seven AES blocks, minus 1.
            112, // Seven AES blocks.
            113, // Seven AES blocks, plus 1.
            127, // Eight AES blocks or four ChaCha20 blocks, minus 1.
            128, // Eight AES blocks or four ChaCha20 blocks.
            129, // Eight AES blocks or four ChaCha20 blocks, plus 1.
            143, // Nine AES blocks, minus 1.
            144, // Nine AES blocks.
            145, // Nine AES blocks, plus 1.
            255, // 16 AES blocks or 8 ChaCha20 blocks, minus 1.
            256, // 16 AES blocks or 8 ChaCha20 blocks.
            257, // 16 AES blocks or 8 ChaCha20 blocks, plus 1.
        ];

        let mut more_comprehensive_in_prefix_lengths = [0; 4096];
        let in_prefix_lengths = if cfg!(debug_assertions) {
            &MINIMAL_IN_PREFIX_LENS[..]
        } else {
            #[allow(clippy::needless_range_loop)]
            for b in 0..more_comprehensive_in_prefix_lengths.len() {
                more_comprehensive_in_prefix_lengths[b] = b;
            }
            &more_comprehensive_in_prefix_lengths[..]
        };
        let mut o_in_out = vec![123u8; 4096];

        for &in_prefix_len in in_prefix_lengths.iter() {
            o_in_out.truncate(0);
            o_in_out.resize(in_prefix_len, 123);
            o_in_out.extend_from_slice(&ct[..]);

            let nonce = aead::Nonce::try_assume_unique_for_key(&nonce_bytes).unwrap();
            let o_result = open(
                aead_alg,
                &key_bytes,
                nonce,
                aead::Aad::from(&aad[..]),
                &mut o_in_out,
                in_prefix_len..,
            );
            match error {
                None => {
                    assert!(s_result.is_ok());
                    assert_eq!(&plaintext[..], o_result.unwrap());
                }
                Some(ref error) if error == "WRONG_NONCE_LENGTH" => {
                    assert_eq!(Err(error::Unspecified), s_result);
                    assert_eq!(Err(error::Unspecified), o_result);
                }
                Some(error) => {
                    unreachable!("Unexpected error test case: {}", error);
                }
            };
        }

        Ok(())
    });
}

fn seal_with_key(
    algorithm: &'static aead::Algorithm,
    key: &[u8],
    nonce: aead::Nonce,
    aad: aead::Aad<&[u8]>,
    in_out: &mut Vec<u8>,
) -> Result<(), error::Unspecified> {
    let mut s_key: aead::SealingKey<OneNonceSequence> = make_key(algorithm, key, nonce);
    s_key.seal_in_place_append_tag(aad, in_out)
}

fn open_with_key<'a>(
    algorithm: &'static aead::Algorithm,
    key: &[u8],
    nonce: aead::Nonce,
    aad: aead::Aad<&[u8]>,
    in_out: &'a mut [u8],
    ciphertext_and_tag: RangeFrom<usize>,
) -> Result<&'a mut [u8], error::Unspecified> {
    let mut o_key: aead::OpeningKey<OneNonceSequence> = make_key(algorithm, key, nonce);
    o_key.open_within(aad, in_out, ciphertext_and_tag)
}

fn seal_with_less_safe_key(
    algorithm: &'static aead::Algorithm,
    key: &[u8],
    nonce: aead::Nonce,
    aad: aead::Aad<&[u8]>,
    in_out: &mut Vec<u8>,
) -> Result<(), error::Unspecified> {
    let key = make_less_safe_key(algorithm, key);
    key.seal_in_place_append_tag(nonce, aad, in_out)
}

fn open_with_less_safe_key<'a>(
    algorithm: &'static aead::Algorithm,
    key: &[u8],
    nonce: aead::Nonce,
    aad: aead::Aad<&[u8]>,
    in_out: &'a mut [u8],
    ciphertext_and_tag: RangeFrom<usize>,
) -> Result<&'a mut [u8], error::Unspecified> {
    let key = make_less_safe_key(algorithm, key);
    key.open_within(nonce, aad, in_out, ciphertext_and_tag)
}

#[allow(clippy::range_plus_one)]
fn test_aead_key_sizes(aead_alg: &'static aead::Algorithm) {
    let key_len = aead_alg.key_len();
    let key_data = vec![0u8; key_len * 2];

    // Key is the right size.
    assert!(aead::UnboundKey::new(aead_alg, &key_data[..key_len]).is_ok());

    // Key is one byte too small.
    assert!(aead::UnboundKey::new(aead_alg, &key_data[..(key_len - 1)]).is_err());

    // Key is one byte too large.
    assert!(aead::UnboundKey::new(aead_alg, &key_data[..(key_len + 1)]).is_err());

    // Key is half the required size.
    assert!(aead::UnboundKey::new(aead_alg, &key_data[..(key_len / 2)]).is_err());

    // Key is twice the required size.
    assert!(aead::UnboundKey::new(aead_alg, &key_data[..(key_len * 2)]).is_err());

    // Key is empty.
    assert!(aead::UnboundKey::new(aead_alg, &[]).is_err());

    // Key is one byte.
    assert!(aead::UnboundKey::new(aead_alg, &[0]).is_err());
}

// Test that we reject non-standard nonce sizes.
#[allow(clippy::range_plus_one)]
#[test]
fn test_aead_nonce_sizes() -> Result<(), error::Unspecified> {
    let nonce_len = aead::NONCE_LEN;
    let nonce = vec![0u8; nonce_len * 2];

    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..nonce_len]).is_ok());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..(nonce_len - 1)]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..(nonce_len + 1)]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..(nonce_len / 2)]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..(nonce_len * 2)]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&[]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..1]).is_err());
    assert!(aead::Nonce::try_assume_unique_for_key(&nonce[..16]).is_err()); // 128 bits.

    Ok(())
}

#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "x86_64",
    target_arch = "x86"
))]
#[allow(clippy::range_plus_one)]
#[test]
fn aead_chacha20_poly1305_openssh() {
    // TODO: test_aead_key_sizes(...);

    test::run(
        test_file!("aead_chacha20_poly1305_openssh_tests.txt"),
        |section, test_case| {
            assert_eq!(section, "");

            // XXX: `polyfill::convert` isn't available here.
            let key_bytes = {
                let as_vec = test_case.consume_bytes("KEY");
                let mut as_array = [0u8; aead::chacha20_poly1305_openssh::KEY_LEN];
                as_array.copy_from_slice(&as_vec);
                as_array
            };

            let sequence_number = test_case.consume_usize("SEQUENCE_NUMBER");
            assert_eq!(sequence_number as u32 as usize, sequence_number);
            let sequence_num = sequence_number as u32;
            let plaintext = test_case.consume_bytes("IN");
            let ct = test_case.consume_bytes("CT");
            let expected_tag = test_case.consume_bytes("TAG");

            // TODO: Add some tests for when things fail.
            //let error = test_case.consume_optional_string("FAILS");

            let mut tag = [0u8; aead::chacha20_poly1305_openssh::TAG_LEN];
            let mut s_in_out = plaintext.clone();
            let s_key = aead::chacha20_poly1305_openssh::SealingKey::new(&key_bytes);
            s_key.seal_in_place(sequence_num, &mut s_in_out[..], &mut tag);
            assert_eq!(&ct, &s_in_out);
            assert_eq!(&expected_tag, &tag);
            let o_key = aead::chacha20_poly1305_openssh::OpeningKey::new(&key_bytes);

            {
                let o_result = o_key.open_in_place(sequence_num, &mut s_in_out[..], &tag);
                assert_eq!(o_result, Ok(&plaintext[4..]));
            }
            assert_eq!(&s_in_out[..4], &ct[..4]);
            assert_eq!(&s_in_out[4..], &plaintext[4..]);

            Ok(())
        },
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_tag_traits() {
    test::compile_time_assert_send::<aead::Tag>();
    test::compile_time_assert_sync::<aead::Tag>();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_aead_key_debug() {
    let key_bytes = [0; 32];
    let nonce = [0; aead::NONCE_LEN];

    let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes).unwrap();
    assert_eq!(
        "UnboundKey { algorithm: AES_256_GCM }",
        format!("{:?}", key)
    );

    let sealing_key: aead::SealingKey<OneNonceSequence> = make_key(
        &aead::AES_256_GCM,
        &key_bytes,
        aead::Nonce::try_assume_unique_for_key(&nonce).unwrap(),
    );
    assert_eq!(
        "SealingKey { algorithm: AES_256_GCM }",
        format!("{:?}", sealing_key)
    );

    let opening_key: aead::OpeningKey<OneNonceSequence> = make_key(
        &aead::AES_256_GCM,
        &key_bytes,
        aead::Nonce::try_assume_unique_for_key(&nonce).unwrap(),
    );
    assert_eq!(
        "OpeningKey { algorithm: AES_256_GCM }",
        format!("{:?}", opening_key)
    );

    let key: aead::LessSafeKey = make_less_safe_key(&aead::AES_256_GCM, &key_bytes);
    assert_eq!(
        "LessSafeKey { algorithm: AES_256_GCM }",
        format!("{:?}", key)
    );
}

fn make_key<K: aead::BoundKey<OneNonceSequence>>(
    algorithm: &'static aead::Algorithm,
    key: &[u8],
    nonce: aead::Nonce,
) -> K {
    let key = aead::UnboundKey::new(algorithm, key).unwrap();
    let nonce_sequence = OneNonceSequence::new(nonce);
    K::new(key, nonce_sequence)
}

fn make_less_safe_key(algorithm: &'static aead::Algorithm, key: &[u8]) -> aead::LessSafeKey {
    let key = aead::UnboundKey::new(algorithm, key).unwrap();
    aead::LessSafeKey::new(key)
}

struct OneNonceSequence(Option<aead::Nonce>);

impl OneNonceSequence {
    /// Constructs the sequence allowing `advance()` to be called
    /// `allowed_invocations` times.
    fn new(nonce: aead::Nonce) -> Self {
        Self(Some(nonce))
    }
}

impl aead::NonceSequence for OneNonceSequence {
    fn advance(&mut self) -> Result<aead::Nonce, error::Unspecified> {
        self.0.take().ok_or(error::Unspecified)
    }
}
