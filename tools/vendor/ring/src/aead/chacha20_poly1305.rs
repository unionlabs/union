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

use super::{
    chacha::{self, Counter},
    iv::Iv,
    poly1305, Aad, Block, Direction, Nonce, Tag, BLOCK_LEN,
};
use crate::{aead, cpu, endian::*, error, polyfill};
use core::convert::TryInto;

/// ChaCha20-Poly1305 as described in [RFC 7539].
///
/// The keys are 256 bits long and the nonces are 96 bits long.
///
/// [RFC 7539]: https://tools.ietf.org/html/rfc7539
pub static CHACHA20_POLY1305: aead::Algorithm = aead::Algorithm {
    key_len: chacha::KEY_LEN,
    init: chacha20_poly1305_init,
    seal: chacha20_poly1305_seal,
    open: chacha20_poly1305_open,
    id: aead::AlgorithmID::CHACHA20_POLY1305,
    max_input_len: super::max_input_len(64, 1),
};

/// Copies |key| into |ctx_buf|.
fn chacha20_poly1305_init(
    key: &[u8],
    _todo: cpu::Features,
) -> Result<aead::KeyInner, error::Unspecified> {
    let key: [u8; chacha::KEY_LEN] = key.try_into()?;
    Ok(aead::KeyInner::ChaCha20Poly1305(chacha::Key::from(key)))
}

fn chacha20_poly1305_seal(
    key: &aead::KeyInner,
    nonce: Nonce,
    aad: Aad<&[u8]>,
    in_out: &mut [u8],
    cpu_features: cpu::Features,
) -> Tag {
    let key = match key {
        aead::KeyInner::ChaCha20Poly1305(key) => key,
        _ => unreachable!(),
    };

    #[cfg(target_arch = "x86_64")]
    {
        if cpu::intel::SSE41.available(cpu_features) {
            // XXX: BoringSSL uses `alignas(16)` on `key` instead of on the
            // structure, but Rust can't do that yet; see
            // https://github.com/rust-lang/rust/issues/73557.
            //
            // Keep in sync with the anonymous struct of BoringSSL's
            // `chacha20_poly1305_seal_data`.
            #[repr(align(16), C)]
            #[derive(Clone, Copy)]
            struct seal_data_in {
                key: [u8; chacha::KEY_LEN],
                counter: u32,
                nonce: [u8; super::NONCE_LEN],
                extra_ciphertext: *const u8,
                extra_ciphertext_len: usize,
            }

            let mut data = InOut {
                input: seal_data_in {
                    key: *key.words_less_safe().as_byte_array(),
                    counter: 0,
                    nonce: *nonce.as_ref(),
                    extra_ciphertext: core::ptr::null(),
                    extra_ciphertext_len: 0,
                },
            };

            // Encrypts `plaintext_len` bytes from `plaintext` and writes them to `out_ciphertext`.
            extern "C" {
                fn GFp_chacha20_poly1305_seal(
                    out_ciphertext: *mut u8,
                    plaintext: *const u8,
                    plaintext_len: usize,
                    ad: *const u8,
                    ad_len: usize,
                    data: &mut InOut<seal_data_in>,
                );
            }

            let out = unsafe {
                GFp_chacha20_poly1305_seal(
                    in_out.as_mut_ptr(),
                    in_out.as_ptr(),
                    in_out.len(),
                    aad.as_ref().as_ptr(),
                    aad.as_ref().len(),
                    &mut data,
                );
                &data.out
            };

            return Tag(out.tag);
        }
    }

    aead(key, nonce, aad, in_out, Direction::Sealing, cpu_features)
}

fn chacha20_poly1305_open(
    key: &aead::KeyInner,
    nonce: Nonce,
    aad: Aad<&[u8]>,
    in_prefix_len: usize,
    in_out: &mut [u8],
    cpu_features: cpu::Features,
) -> Tag {
    let key = match key {
        aead::KeyInner::ChaCha20Poly1305(key) => key,
        _ => unreachable!(),
    };

    #[cfg(target_arch = "x86_64")]
    {
        if cpu::intel::SSE41.available(cpu_features) {
            // XXX: BoringSSL uses `alignas(16)` on `key` instead of on the
            // structure, but Rust can't do that yet; see
            // https://github.com/rust-lang/rust/issues/73557.
            //
            // Keep in sync with the anonymous struct of BoringSSL's
            // `chacha20_poly1305_open_data`.
            #[derive(Copy, Clone)]
            #[repr(align(16), C)]
            struct open_data_in {
                key: [u8; chacha::KEY_LEN],
                counter: u32,
                nonce: [u8; super::NONCE_LEN],
            }

            let mut data = InOut {
                input: open_data_in {
                    key: *key.words_less_safe().as_byte_array(),
                    counter: 0,
                    nonce: *nonce.as_ref(),
                },
            };

            // Decrypts `plaintext_len` bytes from `ciphertext` and writes them to `out_plaintext`.
            extern "C" {
                fn GFp_chacha20_poly1305_open(
                    out_plaintext: *mut u8,
                    ciphertext: *const u8,
                    plaintext_len: usize,
                    ad: *const u8,
                    ad_len: usize,
                    data: &mut InOut<open_data_in>,
                );
            }

            let out = unsafe {
                GFp_chacha20_poly1305_open(
                    in_out.as_mut_ptr(),
                    in_out.as_ptr().add(in_prefix_len),
                    in_out.len() - in_prefix_len,
                    aad.as_ref().as_ptr(),
                    aad.as_ref().len(),
                    &mut data,
                );
                &data.out
            };

            return Tag(out.tag);
        }
    }

    aead(
        key,
        nonce,
        aad,
        in_out,
        Direction::Opening { in_prefix_len },
        cpu_features,
    )
}

pub type Key = chacha::Key;

// Keep in sync with BoringSSL's `chacha20_poly1305_open_data` and
// `chacha20_poly1305_seal_data`.
#[repr(C)]
#[cfg(target_arch = "x86_64")]
union InOut<T>
where
    T: Copy,
{
    input: T,
    out: Out,
}

// It isn't obvious whether the assembly code works for tags that aren't
// 16-byte aligned. In practice it will always be 16-byte aligned because it
// is embedded in a union where the other member of the union is 16-byte
// aligned.
#[cfg(target_arch = "x86_64")]
#[derive(Clone, Copy)]
#[repr(align(16), C)]
struct Out {
    tag: [u8; super::TAG_LEN],
}

#[inline(always)] // Statically eliminate branches on `direction`.
fn aead(
    chacha20_key: &Key,
    nonce: Nonce,
    Aad(aad): Aad<&[u8]>,
    in_out: &mut [u8],
    direction: Direction,
    cpu_features: cpu::Features,
) -> Tag {
    let mut counter = Counter::zero(nonce);
    let mut ctx = {
        let key = derive_poly1305_key(chacha20_key, counter.increment(), cpu_features);
        poly1305::Context::from_key(key)
    };

    poly1305_update_padded_16(&mut ctx, aad);

    let in_out_len = match direction {
        Direction::Opening { in_prefix_len } => {
            poly1305_update_padded_16(&mut ctx, &in_out[in_prefix_len..]);
            chacha20_key.encrypt_overlapping(counter, in_out, in_prefix_len);
            in_out.len() - in_prefix_len
        }
        Direction::Sealing => {
            chacha20_key.encrypt_in_place(counter, in_out);
            poly1305_update_padded_16(&mut ctx, in_out);
            in_out.len()
        }
    };

    ctx.update(
        Block::from_u64_le(
            LittleEndian::from(polyfill::u64_from_usize(aad.len())),
            LittleEndian::from(polyfill::u64_from_usize(in_out_len)),
        )
        .as_ref(),
    );
    ctx.finish()
}

#[inline]
fn poly1305_update_padded_16(ctx: &mut poly1305::Context, input: &[u8]) {
    let remainder_len = input.len() % BLOCK_LEN;
    let whole_len = input.len() - remainder_len;
    if whole_len > 0 {
        ctx.update(&input[..whole_len]);
    }
    if remainder_len > 0 {
        let mut block = Block::zero();
        block.overwrite_part_at(0, &input[whole_len..]);
        ctx.update(block.as_ref())
    }
}

// Also used by chacha20_poly1305_openssh.
pub(super) fn derive_poly1305_key(
    chacha_key: &chacha::Key,
    iv: Iv,
    cpu_features: cpu::Features,
) -> poly1305::Key {
    let mut key_bytes = [0u8; 2 * BLOCK_LEN];
    chacha_key.encrypt_iv_xor_blocks_in_place(iv, &mut key_bytes);
    poly1305::Key::new(key_bytes, cpu_features)
}

#[cfg(test)]
mod tests {
    #[test]
    fn max_input_len_test() {
        // Errata 4858 at https://www.rfc-editor.org/errata_search.php?rfc=7539.
        assert_eq!(super::CHACHA20_POLY1305.max_input_len, 274_877_906_880u64);
    }
}
