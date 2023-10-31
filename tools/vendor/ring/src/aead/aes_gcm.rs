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
    aes::{self, Counter},
    gcm, shift, Aad, Block, Direction, Nonce, Tag, BLOCK_LEN,
};
use crate::{aead, cpu, endian::*, error, polyfill};

/// AES-128 in GCM mode with 128-bit tags and 96 bit nonces.
pub static AES_128_GCM: aead::Algorithm = aead::Algorithm {
    key_len: 16,
    init: init_128,
    seal: aes_gcm_seal,
    open: aes_gcm_open,
    id: aead::AlgorithmID::AES_128_GCM,
    max_input_len: AES_GCM_MAX_INPUT_LEN,
};

/// AES-256 in GCM mode with 128-bit tags and 96 bit nonces.
pub static AES_256_GCM: aead::Algorithm = aead::Algorithm {
    key_len: 32,
    init: init_256,
    seal: aes_gcm_seal,
    open: aes_gcm_open,
    id: aead::AlgorithmID::AES_256_GCM,
    max_input_len: AES_GCM_MAX_INPUT_LEN,
};

pub struct Key {
    gcm_key: gcm::Key, // First because it has a large alignment requirement.
    aes_key: aes::Key,
}

fn init_128(key: &[u8], cpu_features: cpu::Features) -> Result<aead::KeyInner, error::Unspecified> {
    init(key, aes::Variant::AES_128, cpu_features)
}

fn init_256(key: &[u8], cpu_features: cpu::Features) -> Result<aead::KeyInner, error::Unspecified> {
    init(key, aes::Variant::AES_256, cpu_features)
}

fn init(
    key: &[u8],
    variant: aes::Variant,
    cpu_features: cpu::Features,
) -> Result<aead::KeyInner, error::Unspecified> {
    let aes_key = aes::Key::new(key, variant, cpu_features)?;
    let gcm_key = gcm::Key::new(aes_key.encrypt_block(Block::zero()), cpu_features);
    Ok(aead::KeyInner::AesGcm(Key { aes_key, gcm_key }))
}

const CHUNK_BLOCKS: usize = 3 * 1024 / 16;

fn aes_gcm_seal(
    key: &aead::KeyInner,
    nonce: Nonce,
    aad: Aad<&[u8]>,
    in_out: &mut [u8],
    cpu_features: cpu::Features,
) -> Tag {
    aead(key, nonce, aad, in_out, Direction::Sealing, cpu_features)
}

fn aes_gcm_open(
    key: &aead::KeyInner,
    nonce: Nonce,
    aad: Aad<&[u8]>,
    in_prefix_len: usize,
    in_out: &mut [u8],
    cpu_features: cpu::Features,
) -> Tag {
    aead(
        key,
        nonce,
        aad,
        in_out,
        Direction::Opening { in_prefix_len },
        cpu_features,
    )
}

#[inline(always)] // Avoid branching on `direction`.
fn aead(
    key: &aead::KeyInner,
    nonce: Nonce,
    aad: Aad<&[u8]>,
    in_out: &mut [u8],
    direction: Direction,
    cpu_features: cpu::Features,
) -> Tag {
    let Key { aes_key, gcm_key } = match key {
        aead::KeyInner::AesGcm(key) => key,
        _ => unreachable!(),
    };

    let mut ctr = Counter::one(nonce);
    let tag_iv = ctr.increment();

    let aad_len = aad.0.len();
    let mut gcm_ctx = gcm::Context::new(gcm_key, aad, cpu_features);

    let in_prefix_len = match direction {
        Direction::Opening { in_prefix_len } => in_prefix_len,
        Direction::Sealing => 0,
    };

    let total_in_out_len = in_out.len() - in_prefix_len;

    let in_out = integrated_aes_gcm(
        aes_key,
        &mut gcm_ctx,
        in_out,
        &mut ctr,
        direction,
        cpu_features,
    );
    let in_out_len = in_out.len() - in_prefix_len;

    // Process any (remaining) whole blocks.
    let whole_len = in_out_len - (in_out_len % BLOCK_LEN);
    {
        let mut chunk_len = CHUNK_BLOCKS * BLOCK_LEN;
        let mut output = 0;
        let mut input = in_prefix_len;
        loop {
            if whole_len - output < chunk_len {
                chunk_len = whole_len - output;
            }
            if chunk_len == 0 {
                break;
            }

            if let Direction::Opening { .. } = direction {
                gcm_ctx.update_blocks(&in_out[input..][..chunk_len]);
            }

            aes_key.ctr32_encrypt_blocks(
                &mut in_out[output..][..(chunk_len + in_prefix_len)],
                direction,
                &mut ctr,
            );

            if let Direction::Sealing = direction {
                gcm_ctx.update_blocks(&in_out[output..][..chunk_len]);
            }

            output += chunk_len;
            input += chunk_len;
        }
    }

    // Process any remaining partial block.
    let remainder = &mut in_out[whole_len..];
    shift::shift_partial((in_prefix_len, remainder), |remainder| {
        let mut input = Block::zero();
        input.overwrite_part_at(0, remainder);
        if let Direction::Opening { .. } = direction {
            gcm_ctx.update_block(input);
        }
        let mut output = aes_key.encrypt_iv_xor_block(ctr.into(), input);
        if let Direction::Sealing = direction {
            output.zero_from(remainder.len());
            gcm_ctx.update_block(output);
        }
        output
    });

    // Authenticate the final block containing the input lengths.
    let aad_bits = polyfill::u64_from_usize(aad_len) << 3;
    let ciphertext_bits = polyfill::u64_from_usize(total_in_out_len) << 3;
    gcm_ctx.update_block(Block::from_u64_be(
        BigEndian::from(aad_bits),
        BigEndian::from(ciphertext_bits),
    ));

    // Finalize the tag and return it.
    gcm_ctx.pre_finish(|pre_tag| {
        let bytes = tag_iv.into_bytes_less_safe();
        let mut tag = aes_key.encrypt_block(Block::from(&bytes));
        tag.bitxor_assign(pre_tag.into());
        Tag(*tag.as_ref())
    })
}

// Returns the data that wasn't processed.
#[cfg(target_arch = "x86_64")]
#[inline] // Optimize out the match on `direction`.
fn integrated_aes_gcm<'a>(
    aes_key: &aes::Key,
    gcm_ctx: &mut gcm::Context,
    in_out: &'a mut [u8],
    ctr: &mut Counter,
    direction: Direction,
    cpu_features: cpu::Features,
) -> &'a mut [u8] {
    use crate::c;

    if !aes_key.is_aes_hw() || !gcm_ctx.is_avx2(cpu_features) {
        return in_out;
    }

    let processed = match direction {
        Direction::Opening { in_prefix_len } => {
            extern "C" {
                fn GFp_aesni_gcm_decrypt(
                    input: *const u8,
                    output: *mut u8,
                    len: c::size_t,
                    key: &aes::AES_KEY,
                    ivec: &mut Counter,
                    gcm: &mut gcm::ContextInner,
                ) -> c::size_t;
            }
            unsafe {
                GFp_aesni_gcm_decrypt(
                    in_out[in_prefix_len..].as_ptr(),
                    in_out.as_mut_ptr(),
                    in_out.len() - in_prefix_len,
                    aes_key.inner_less_safe(),
                    ctr,
                    gcm_ctx.inner(),
                )
            }
        }
        Direction::Sealing => {
            extern "C" {
                fn GFp_aesni_gcm_encrypt(
                    input: *const u8,
                    output: *mut u8,
                    len: c::size_t,
                    key: &aes::AES_KEY,
                    ivec: &mut Counter,
                    gcm: &mut gcm::ContextInner,
                ) -> c::size_t;
            }
            unsafe {
                GFp_aesni_gcm_encrypt(
                    in_out.as_ptr(),
                    in_out.as_mut_ptr(),
                    in_out.len(),
                    aes_key.inner_less_safe(),
                    ctr,
                    gcm_ctx.inner(),
                )
            }
        }
    };

    &mut in_out[processed..]
}

#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn integrated_aes_gcm<'a>(
    _: &aes::Key,
    _: &mut gcm::Context,
    in_out: &'a mut [u8],
    _: &mut Counter,
    _: Direction,
    _: cpu::Features,
) -> &'a mut [u8] {
    in_out // This doesn't process any of the input so it all remains.
}

const AES_GCM_MAX_INPUT_LEN: u64 = super::max_input_len(BLOCK_LEN, 2);

#[cfg(test)]
mod tests {
    #[test]
    fn max_input_len_test() {
        // [NIST SP800-38D] Section 5.2.1.1. Note that [RFC 5116 Section 5.1] and
        // [RFC 5116 Section 5.2] have an off-by-one error in `P_MAX`.
        //
        // [NIST SP800-38D]:
        //    http://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf
        // [RFC 5116 Section 5.1]: https://tools.ietf.org/html/rfc5116#section-5.1
        // [RFC 5116 Section 5.2]: https://tools.ietf.org/html/rfc5116#section-5.2
        const NIST_SP800_38D_MAX_BITS: u64 = (1u64 << 39) - 256;
        assert_eq!(NIST_SP800_38D_MAX_BITS, 549_755_813_632u64);
        assert_eq!(
            super::AES_128_GCM.max_input_len * 8,
            NIST_SP800_38D_MAX_BITS
        );
        assert_eq!(
            super::AES_256_GCM.max_input_len * 8,
            NIST_SP800_38D_MAX_BITS
        );
    }
}
