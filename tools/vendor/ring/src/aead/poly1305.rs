// Copyright 2015-2016 Brian Smith.
// Portions Copyright (c) 2014, 2015, Google Inc.
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

// TODO: enforce maximum input length.

use super::{block::BLOCK_LEN, Tag, TAG_LEN};
use crate::{c, cpu};

/// A Poly1305 key.
pub(super) struct Key {
    key_and_nonce: [u8; KEY_LEN],
    cpu_features: cpu::Features,
}

const KEY_LEN: usize = 2 * BLOCK_LEN;

impl Key {
    #[inline]
    pub(super) fn new(key_and_nonce: [u8; KEY_LEN], cpu_features: cpu::Features) -> Self {
        Self {
            key_and_nonce,
            cpu_features,
        }
    }
}

pub struct Context {
    state: poly1305_state,
    #[allow(dead_code)]
    cpu_features: cpu::Features,
}

// Keep in sync with `poly1305_state` in GFp/poly1305.h.
//
// The C code, in particular the way the `poly1305_aligned_state` functions
// are used, is only correct when the state buffer is 64-byte aligned.
#[repr(C, align(64))]
struct poly1305_state([u8; OPAQUE_LEN]);
const OPAQUE_LEN: usize = 512;

// Abstracts the dispatching logic that chooses the NEON implementation if and
// only if it would work.
macro_rules! dispatch {
    ( $features:expr =>
      ( $f:ident | $neon_f:ident )
      ( $( $p:ident : $t:ty ),+ )
      ( $( $a:expr ),+ ) ) => {
        match () {
            // Apple's 32-bit ARM ABI is incompatible with the assembly code.
            #[cfg(all(target_arch = "arm", not(target_vendor = "apple")))]
            () if cpu::arm::NEON.available($features) => {
                extern "C" {
                    fn $neon_f( $( $p : $t ),+ );
                }
                unsafe { $neon_f( $( $a ),+ ) }
            }
            () => {
                extern "C" {
                    fn $f( $( $p : $t ),+ );
                }
                unsafe { $f( $( $a ),+ ) }
            }
        }
    }
}

impl Context {
    #[inline]
    pub(super) fn from_key(
        Key {
            key_and_nonce,
            cpu_features,
        }: Key,
    ) -> Self {
        let mut ctx = Self {
            state: poly1305_state([0u8; OPAQUE_LEN]),
            cpu_features,
        };

        dispatch!(
            cpu_features =>
            (GFp_poly1305_init | GFp_poly1305_init_neon)
            (statep: &mut poly1305_state, key: &[u8; KEY_LEN])
            (&mut ctx.state, &key_and_nonce));

        ctx
    }

    #[inline(always)]
    pub fn update(&mut self, input: &[u8]) {
        dispatch!(
            self.cpu_features =>
            (GFp_poly1305_update | GFp_poly1305_update_neon)
            (statep: &mut poly1305_state, input: *const u8, in_len: c::size_t)
            (&mut self.state, input.as_ptr(), input.len()));
    }

    pub(super) fn finish(mut self) -> Tag {
        let mut tag = Tag([0u8; TAG_LEN]);
        dispatch!(
            self.cpu_features =>
            (GFp_poly1305_finish | GFp_poly1305_finish_neon)
            (statep: &mut poly1305_state, mac: &mut [u8; TAG_LEN])
            (&mut self.state, &mut tag.0));
        tag
    }
}

/// Implements the original, non-IETF padding semantics.
///
/// This is used by chacha20_poly1305_openssh and the standalone
/// poly1305 test vectors.
pub(super) fn sign(key: Key, input: &[u8]) -> Tag {
    let mut ctx = Context::from_key(key);
    ctx.update(input);
    ctx.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use core::convert::TryInto;

    // Adapted from BoringSSL's crypto/poly1305/poly1305_test.cc.
    #[test]
    pub fn test_poly1305() {
        let cpu_features = cpu::features();
        test::run(test_file!("poly1305_test.txt"), |section, test_case| {
            assert_eq!(section, "");
            let key = test_case.consume_bytes("Key");
            let key: &[u8; BLOCK_LEN * 2] = key.as_slice().try_into().unwrap();
            let input = test_case.consume_bytes("Input");
            let expected_mac = test_case.consume_bytes("MAC");
            let key = Key::new(*key, cpu_features);
            let Tag(actual_mac) = sign(key, &input);
            assert_eq!(expected_mac, actual_mac.as_ref());

            Ok(())
        })
    }
}
