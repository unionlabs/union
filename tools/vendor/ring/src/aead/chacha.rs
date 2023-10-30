// Copyright 2016 Brian Smith.
// Portions Copyright (c) 2016, Google Inc.
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

use super::{counter, iv::Iv, quic::Sample, BLOCK_LEN};
use crate::{c, endian::*};

#[repr(transparent)]
pub struct Key([LittleEndian<u32>; KEY_LEN / 4]);

impl From<[u8; KEY_LEN]> for Key {
    #[inline]
    fn from(value: [u8; KEY_LEN]) -> Self {
        Self(FromByteArray::from_byte_array(&value))
    }
}

impl Key {
    #[inline] // Optimize away match on `counter`.
    pub fn encrypt_in_place(&self, counter: Counter, in_out: &mut [u8]) {
        unsafe {
            self.encrypt(
                CounterOrIv::Counter(counter),
                in_out.as_ptr(),
                in_out.len(),
                in_out.as_mut_ptr(),
            );
        }
    }

    #[inline] // Optimize away match on `iv` and length check.
    pub fn encrypt_iv_xor_blocks_in_place(&self, iv: Iv, in_out: &mut [u8; 2 * BLOCK_LEN]) {
        unsafe {
            self.encrypt(
                CounterOrIv::Iv(iv),
                in_out.as_ptr(),
                in_out.len(),
                in_out.as_mut_ptr(),
            );
        }
    }

    #[inline]
    pub fn new_mask(&self, sample: Sample) -> [u8; 5] {
        let mut out: [u8; 5] = [0; 5];
        let iv = Iv::assume_unique_for_key(sample);

        unsafe {
            self.encrypt(
                CounterOrIv::Iv(iv),
                out.as_ptr(),
                out.len(),
                out.as_mut_ptr(),
            );
        }

        out
    }

    pub fn encrypt_overlapping(&self, counter: Counter, in_out: &mut [u8], in_prefix_len: usize) {
        // XXX: The x86 and at least one branch of the ARM assembly language
        // code doesn't allow overlapping input and output unless they are
        // exactly overlapping. TODO: Figure out which branch of the ARM code
        // has this limitation and come up with a better solution.
        //
        // https://rt.openssl.org/Ticket/Display.html?id=4362
        let len = in_out.len() - in_prefix_len;
        if cfg!(any(target_arch = "arm", target_arch = "x86")) && in_prefix_len != 0 {
            in_out.copy_within(in_prefix_len.., 0);
            self.encrypt_in_place(counter, &mut in_out[..len]);
        } else {
            unsafe {
                self.encrypt(
                    CounterOrIv::Counter(counter),
                    in_out[in_prefix_len..].as_ptr(),
                    len,
                    in_out.as_mut_ptr(),
                );
            }
        }
    }

    #[inline] // Optimize away match on `counter.`
    unsafe fn encrypt(
        &self,
        counter: CounterOrIv,
        input: *const u8,
        in_out_len: usize,
        output: *mut u8,
    ) {
        let iv = match counter {
            CounterOrIv::Counter(counter) => counter.into(),
            CounterOrIv::Iv(iv) => {
                assert!(in_out_len <= 32);
                iv
            }
        };

        /// XXX: Although this takes an `Iv`, this actually uses it like a
        /// `Counter`.
        extern "C" {
            fn GFp_ChaCha20_ctr32(
                out: *mut u8,
                in_: *const u8,
                in_len: c::size_t,
                key: &Key,
                first_iv: &Iv,
            );
        }

        GFp_ChaCha20_ctr32(output, input, in_out_len, self, &iv);
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub(super) fn words_less_safe(&self) -> &[LittleEndian<u32>; KEY_LEN / 4] {
        &self.0
    }
}

pub type Counter = counter::Counter<LittleEndian<u32>>;

enum CounterOrIv {
    Counter(Counter),
    Iv(Iv),
}

const KEY_BLOCKS: usize = 2;
pub const KEY_LEN: usize = KEY_BLOCKS * BLOCK_LEN;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use alloc::vec;
    use core::convert::TryInto;

    // This verifies the encryption functionality provided by ChaCha20_ctr32
    // is successful when either computed on disjoint input/output buffers,
    // or on overlapping input/output buffers. On some branches of the 32-bit
    // x86 and ARM code the in-place operation fails in some situations where
    // the input/output buffers are not exactly overlapping. Such failures are
    // dependent not only on the degree of overlapping but also the length of
    // the data. `open()` works around that by moving the input data to the
    // output location so that the buffers exactly overlap, for those targets.
    // This test exists largely as a canary for detecting if/when that type of
    // problem spreads to other platforms.
    #[test]
    pub fn chacha20_tests() {
        test::run(test_file!("chacha_tests.txt"), |section, test_case| {
            assert_eq!(section, "");

            let key = test_case.consume_bytes("Key");
            let key: &[u8; KEY_LEN] = key.as_slice().try_into()?;
            let key = Key::from(*key);

            let ctr = test_case.consume_usize("Ctr");
            let nonce = test_case.consume_bytes("Nonce");
            let input = test_case.consume_bytes("Input");
            let output = test_case.consume_bytes("Output");

            // Pre-allocate buffer for use in test_cases.
            let mut in_out_buf = vec![0u8; input.len() + 276];

            // Run the test case over all prefixes of the input because the
            // behavior of ChaCha20 implementation changes dependent on the
            // length of the input.
            for len in 0..(input.len() + 1) {
                chacha20_test_case_inner(
                    &key,
                    &nonce,
                    ctr as u32,
                    &input[..len],
                    &output[..len],
                    len,
                    &mut in_out_buf,
                );
            }

            Ok(())
        });
    }

    fn chacha20_test_case_inner(
        key: &Key,
        nonce: &[u8],
        ctr: u32,
        input: &[u8],
        expected: &[u8],
        len: usize,
        in_out_buf: &mut [u8],
    ) {
        // Straightforward encryption into disjoint buffers is computed
        // correctly.
        unsafe {
            key.encrypt(
                CounterOrIv::Counter(Counter::from_test_vector(nonce, ctr)),
                input[..len].as_ptr(),
                len,
                in_out_buf.as_mut_ptr(),
            );
        }
        assert_eq!(&in_out_buf[..len], expected);

        // Do not test offset buffers for x86 and ARM architectures (see above
        // for rationale).
        let max_offset = if cfg!(any(target_arch = "x86", target_arch = "arm")) {
            0
        } else {
            259
        };

        // Check that in-place encryption works successfully when the pointers
        // to the input/output buffers are (partially) overlapping.
        for alignment in 0..16 {
            for offset in 0..(max_offset + 1) {
                in_out_buf[alignment + offset..][..len].copy_from_slice(input);
                let ctr = Counter::from_test_vector(nonce, ctr);
                key.encrypt_overlapping(ctr, &mut in_out_buf[alignment..], offset);
                assert_eq!(&in_out_buf[alignment..][..len], expected);
            }
        }
    }
}
