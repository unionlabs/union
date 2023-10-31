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

use ring::{digest, test, test_file};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

/// Test vectors from BoringSSL, Go, and other sources.
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn digest_misc() {
    test::run(test_file!("digest_tests.txt"), |section, test_case| {
        assert_eq!(section, "");
        let digest_alg = test_case.consume_digest_alg("Hash").unwrap();
        let input = test_case.consume_bytes("Input");
        let repeat = test_case.consume_usize("Repeat");
        let expected = test_case.consume_bytes("Output");

        let mut ctx = digest::Context::new(digest_alg);
        let mut data = Vec::new();
        for _ in 0..repeat {
            ctx.update(&input);
            data.extend(&input);
        }
        let actual_from_chunks = ctx.finish();
        assert_eq!(&expected, &actual_from_chunks.as_ref());

        let actual_from_one_shot = digest::digest(digest_alg, &data);
        assert_eq!(&expected, &actual_from_one_shot.as_ref());

        Ok(())
    });
}

mod digest_shavs {
    use ring::{digest, test};

    fn run_known_answer_test(digest_alg: &'static digest::Algorithm, test_file: test::File) {
        let section_name = &format!("L = {}", digest_alg.output_len);
        test::run(test_file, |section, test_case| {
            assert_eq!(section_name, section);
            let len_bits = test_case.consume_usize("Len");

            let mut msg = test_case.consume_bytes("Msg");
            // The "msg" field contains the dummy value "00" when the
            // length is zero.
            if len_bits == 0 {
                assert_eq!(msg, &[0u8]);
                msg.truncate(0);
            }

            assert_eq!(msg.len() * 8, len_bits);
            let expected = test_case.consume_bytes("MD");
            let actual = digest::digest(digest_alg, &msg);
            assert_eq!(&expected, &actual.as_ref());

            Ok(())
        });
    }

    macro_rules! shavs_tests {
        ( $file_name:ident, $algorithm_name:ident ) => {
            #[allow(non_snake_case)]
            mod $algorithm_name {
                use super::{run_known_answer_test, run_monte_carlo_test};
                use ring::{digest, test_file};

                #[cfg(target_arch = "wasm32")]
                use wasm_bindgen_test::wasm_bindgen_test;

                #[test]
                #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
                fn short_msg_known_answer_test() {
                    run_known_answer_test(
                        &digest::$algorithm_name,
                        test_file!(concat!(
                            "../third_party/NIST/SHAVS/",
                            stringify!($file_name),
                            "ShortMsg.rsp"
                        )),
                    );
                }

                #[test]
                #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
                fn long_msg_known_answer_test() {
                    run_known_answer_test(
                        &digest::$algorithm_name,
                        test_file!(concat!(
                            "../third_party/NIST/SHAVS/",
                            stringify!($file_name),
                            "LongMsg.rsp"
                        )),
                    );
                }

                #[test]
                #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
                fn monte_carlo_test() {
                    run_monte_carlo_test(
                        &digest::$algorithm_name,
                        test_file!(concat!(
                            "../third_party/NIST/SHAVS/",
                            stringify!($file_name),
                            "Monte.rsp"
                        )),
                    );
                }
            }
        };
    }

    fn run_monte_carlo_test(digest_alg: &'static digest::Algorithm, test_file: test::File) {
        let section_name = &format!("L = {}", digest_alg.output_len);

        let mut expected_count: isize = -1;
        let mut seed = Vec::with_capacity(digest_alg.output_len);

        test::run(test_file, |section, test_case| {
            assert_eq!(section_name, section);

            if expected_count == -1 {
                seed.extend(test_case.consume_bytes("Seed"));
                expected_count = 0;
                return Ok(());
            }

            assert!(expected_count >= 0);
            let actual_count = test_case.consume_usize("COUNT");
            assert_eq!(expected_count as usize, actual_count);
            expected_count += 1;

            let expected_md = test_case.consume_bytes("MD");

            let mut mds = Vec::with_capacity(4);
            mds.push(seed.clone());
            mds.push(seed.clone());
            mds.push(seed.clone());
            for _ in 0..1000 {
                let mut ctx = digest::Context::new(digest_alg);
                ctx.update(&mds[0]);
                ctx.update(&mds[1]);
                ctx.update(&mds[2]);
                let md_i = ctx.finish();
                let _ = mds.remove(0);
                mds.push(Vec::from(md_i.as_ref()));
            }
            let md_j = mds.last().unwrap();
            assert_eq!(&expected_md, md_j);
            seed = md_j.clone();

            Ok(())
        });

        assert_eq!(expected_count, 100);
    }

    shavs_tests!(SHA1, SHA1_FOR_LEGACY_USE_ONLY);
    shavs_tests!(SHA256, SHA256);
    shavs_tests!(SHA384, SHA384);
    shavs_tests!(SHA512, SHA512);
}

/// Test some ways in which `Context::update` and/or `Context::finish`
/// could go wrong by testing every combination of updating three inputs
/// that vary from zero bytes to one byte larger than the block length.
///
/// These are not run in dev (debug) builds because they are too slow.
macro_rules! test_i_u_f {
    ( $test_name:ident, $alg:expr) => {
        #[cfg(not(debug_assertions))]
        // TODO: #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        #[test]
        fn $test_name() {
            let mut input = [0; (digest::MAX_BLOCK_LEN + 1) * 3];
            let max = $alg.block_len + 1;
            for i in 0..(max * 3) {
                input[i] = (i & 0xff) as u8;
            }

            for i in 0..max {
                for j in 0..max {
                    for k in 0..max {
                        let part1 = &input[..i];
                        let part2 = &input[i..(i + j)];
                        let part3 = &input[(i + j)..(i + j + k)];

                        let mut ctx = digest::Context::new(&$alg);
                        ctx.update(part1);
                        ctx.update(part2);
                        ctx.update(part3);
                        let i_u_f = ctx.finish();

                        let one_shot = digest::digest(&$alg, &input[..(i + j + k)]);

                        assert_eq!(i_u_f.as_ref(), one_shot.as_ref());
                    }
                }
            }
        }
    };
}
test_i_u_f!(digest_test_i_u_f_sha1, digest::SHA1_FOR_LEGACY_USE_ONLY);
test_i_u_f!(digest_test_i_u_f_sha256, digest::SHA256);
test_i_u_f!(digest_test_i_u_f_sha384, digest::SHA384);
test_i_u_f!(digest_test_i_u_f_sha512, digest::SHA512);

/// See https://bugzilla.mozilla.org/show_bug.cgi?id=610162. This tests the
/// calculation of 8GB of the byte 123.
///
/// You can verify the expected values in many ways. One way is
/// `python ~/p/write_big.py`, where write_big.py is:
///
/// ```python
/// chunk = bytearray([123] * (16 * 1024))
/// with open('tempfile', 'w') as f:
/// for i in xrange(0, 8 * 1024 * 1024 * 1024, len(chunk)):
///     f.write(chunk)
/// ```
/// Then:
///
/// ```sh
/// sha1sum -b tempfile
/// sha256sum -b tempfile
/// sha384sum -b tempfile
/// sha512sum -b tempfile
/// ```
///
/// This is not run in dev (debug) builds because it is too slow.
macro_rules! test_large_digest {
    ( $test_name:ident, $alg:expr, $len:expr, $expected:expr) => {
        #[cfg(not(debug_assertions))]
        #[test]
        // TODO: #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
        fn $test_name() {
            let chunk = vec![123u8; 16 * 1024];
            let chunk_len = chunk.len() as u64;
            let mut ctx = digest::Context::new(&$alg);
            let mut hashed = 0u64;
            loop {
                ctx.update(&chunk);
                hashed += chunk_len;
                if hashed >= 8u64 * 1024 * 1024 * 1024 {
                    break;
                }
            }
            let calculated = ctx.finish();
            let expected: [u8; $len] = $expected;
            assert_eq!(&expected[..], calculated.as_ref());
        }
    };
}

// XXX: This test is too slow on Android ARM.
#[cfg(any(not(target_os = "android"), not(target_arch = "arm")))]
test_large_digest!(
    digest_test_large_digest_sha1,
    digest::SHA1_FOR_LEGACY_USE_ONLY,
    160 / 8,
    [
        0xCA, 0xC3, 0x4C, 0x31, 0x90, 0x5B, 0xDE, 0x3B, 0xE4, 0x0D, 0x46, 0x6D, 0x70, 0x76, 0xAD,
        0x65, 0x3C, 0x20, 0xE4, 0xBD
    ]
);

test_large_digest!(
    digest_test_large_digest_sha256,
    digest::SHA256,
    256 / 8,
    [
        0x8D, 0xD1, 0x6D, 0xD8, 0xB2, 0x5A, 0x29, 0xCB, 0x7F, 0xB9, 0xAE, 0x86, 0x72, 0xE9, 0xCE,
        0xD6, 0x65, 0x4C, 0xB6, 0xC3, 0x5C, 0x58, 0x21, 0xA7, 0x07, 0x97, 0xC5, 0xDD, 0xAE, 0x5C,
        0x68, 0xBD
    ]
);
test_large_digest!(
    digest_test_large_digest_sha384,
    digest::SHA384,
    384 / 8,
    [
        0x3D, 0xFE, 0xC1, 0xA9, 0xD0, 0x9F, 0x08, 0xD5, 0xBB, 0xE8, 0x7C, 0x9E, 0xE0, 0x0A, 0x87,
        0x0E, 0xB0, 0xEA, 0x8E, 0xEA, 0xDB, 0x82, 0x36, 0xAE, 0x74, 0xCF, 0x9F, 0xDC, 0x86, 0x1C,
        0xE3, 0xE9, 0xB0, 0x68, 0xCD, 0x19, 0x3E, 0x39, 0x90, 0x02, 0xE1, 0x58, 0x5D, 0x66, 0xC4,
        0x55, 0x11, 0x9B
    ]
);
test_large_digest!(
    digest_test_large_digest_sha512,
    digest::SHA512,
    512 / 8,
    [
        0xFC, 0x8A, 0x98, 0x20, 0xFC, 0x82, 0xD8, 0x55, 0xF8, 0xFF, 0x2F, 0x6E, 0xAE, 0x41, 0x60,
        0x04, 0x08, 0xE9, 0x49, 0xD7, 0xCD, 0x1A, 0xED, 0x22, 0xEB, 0x55, 0xE1, 0xFD, 0x80, 0x50,
        0x3B, 0x01, 0x2F, 0xC6, 0xF4, 0x33, 0x86, 0xFB, 0x60, 0x75, 0x2D, 0xA5, 0xA9, 0x93, 0xE7,
        0x00, 0x45, 0xA8, 0x49, 0x1A, 0x6B, 0xEC, 0x9C, 0x98, 0xC8, 0x19, 0xA6, 0xA9, 0x88, 0x3E,
        0x2F, 0x09, 0xB9, 0x9A
    ]
);

// TODO: test_large_digest!(digest_test_large_digest_sha512_256,
//                            digest::SHA512_256, 256 / 8, [ ... ]);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn test_fmt_algorithm() {
    assert_eq!("SHA1", &format!("{:?}", digest::SHA1_FOR_LEGACY_USE_ONLY));
    assert_eq!("SHA256", &format!("{:?}", digest::SHA256));
    assert_eq!("SHA384", &format!("{:?}", digest::SHA384));
    assert_eq!("SHA512", &format!("{:?}", digest::SHA512));
    assert_eq!("SHA512_256", &format!("{:?}", digest::SHA512_256));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn digest_test_fmt() {
    assert_eq!(
        "SHA1:b7e23ec29af22b0b4e41da31e868d57226121c84",
        &format!(
            "{:?}",
            digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, b"hello, world")
        )
    );
    assert_eq!(
        "SHA256:09ca7e4eaa6e8ae9c7d261167129184883644d\
         07dfba7cbfbc4c8a2e08360d5b",
        &format!("{:?}", digest::digest(&digest::SHA256, b"hello, world"))
    );
    assert_eq!(
        "SHA384:1fcdb6059ce05172a26bbe2a3ccc88ed5a8cd5\
         fc53edfd9053304d429296a6da23b1cd9e5c9ed3bb34f0\
         0418a70cdb7e",
        &format!("{:?}", digest::digest(&digest::SHA384, b"hello, world"))
    );
    assert_eq!(
        "SHA512:8710339dcb6814d0d9d2290ef422285c9322b7\
         163951f9a0ca8f883d3305286f44139aa374848e4174f5\
         aada663027e4548637b6d19894aec4fb6c46a139fbf9",
        &format!("{:?}", digest::digest(&digest::SHA512, b"hello, world"))
    );

    assert_eq!(
        "SHA512_256:11f2c88c04f0a9c3d0970894ad2472505e\
         0bc6e8c7ec46b5211cd1fa3e253e62",
        &format!("{:?}", digest::digest(&digest::SHA512_256, b"hello, world"))
    );
}
