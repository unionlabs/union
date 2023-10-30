// Copyright 2016 Brian Smith.
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

//! The [chacha20-poly1305@openssh.com] AEAD-ish construct.
//!
//! This should only be used by SSH implementations. It has a similar, but
//! different API from `ring::aead` because the construct cannot use the same
//! API as `ring::aead` due to the way the construct handles the encrypted
//! packet length.
//!
//! The concatenation of a and b is denoted `a||b`. `K_1` and `K_2` are defined
//! in the [chacha20-poly1305@openssh.com] specification. `packet_length`,
//! `padding_length`, `payload`, and `random padding` are defined in
//! [RFC 4253]. The term `plaintext` is used as a shorthand for
//! `padding_length||payload||random padding`.
//!
//! [chacha20-poly1305@openssh.com]:
//!    http://cvsweb.openbsd.org/cgi-bin/cvsweb/src/usr.bin/ssh/PROTOCOL.chacha20poly1305?annotate=HEAD
//! [RFC 4253]: https://tools.ietf.org/html/rfc4253

use super::{
    chacha::{self, *},
    chacha20_poly1305::derive_poly1305_key,
    cpu, poly1305, Nonce, Tag,
};
use crate::{constant_time, endian::*, error};
use core::convert::TryInto;

/// A key for sealing packets.
pub struct SealingKey {
    key: Key,
}

impl SealingKey {
    /// Constructs a new `SealingKey`.
    pub fn new(key_material: &[u8; KEY_LEN]) -> SealingKey {
        SealingKey {
            key: Key::new(key_material, cpu::features()),
        }
    }

    /// Seals (encrypts and signs) a packet.
    ///
    /// On input, `plaintext_in_ciphertext_out` must contain the unencrypted
    /// `packet_length||plaintext` where `plaintext` is the
    /// `padding_length||payload||random padding`. It will be overwritten by
    /// `encrypted_packet_length||ciphertext`, where `encrypted_packet_length`
    /// is encrypted with `K_1` and `ciphertext` is encrypted by `K_2`.
    pub fn seal_in_place(
        &self,
        sequence_number: u32,
        plaintext_in_ciphertext_out: &mut [u8],
        tag_out: &mut [u8; TAG_LEN],
    ) {
        let mut counter = make_counter(sequence_number);
        let poly_key =
            derive_poly1305_key(&self.key.k_2, counter.increment(), self.key.cpu_features);

        {
            let (len_in_out, data_and_padding_in_out) =
                plaintext_in_ciphertext_out.split_at_mut(PACKET_LENGTH_LEN);

            self.key
                .k_1
                .encrypt_in_place(make_counter(sequence_number), len_in_out);
            self.key
                .k_2
                .encrypt_in_place(counter, data_and_padding_in_out);
        }

        let Tag(tag) = poly1305::sign(poly_key, plaintext_in_ciphertext_out);
        tag_out.copy_from_slice(tag.as_ref());
    }
}

/// A key for opening packets.
pub struct OpeningKey {
    key: Key,
}

impl OpeningKey {
    /// Constructs a new `OpeningKey`.
    pub fn new(key_material: &[u8; KEY_LEN]) -> OpeningKey {
        OpeningKey {
            key: Key::new(key_material, cpu::features()),
        }
    }

    /// Returns the decrypted, but unauthenticated, packet length.
    ///
    /// Importantly, the result won't be authenticated until `open_in_place` is
    /// called.
    pub fn decrypt_packet_length(
        &self,
        sequence_number: u32,
        encrypted_packet_length: [u8; PACKET_LENGTH_LEN],
    ) -> [u8; PACKET_LENGTH_LEN] {
        let mut packet_length = encrypted_packet_length;
        let counter = make_counter(sequence_number);
        self.key.k_1.encrypt_in_place(counter, &mut packet_length);
        packet_length
    }

    /// Opens (authenticates and decrypts) a packet.
    ///
    /// `ciphertext_in_plaintext_out` must be of the form
    /// `encrypted_packet_length||ciphertext` where `ciphertext` is the
    /// encrypted `plaintext`. When the function succeeds the ciphertext is
    /// replaced by the plaintext and the result is `Ok(plaintext)`, where
    /// `plaintext` is `&ciphertext_in_plaintext_out[PACKET_LENGTH_LEN..]`;
    /// otherwise the contents of `ciphertext_in_plaintext_out` are unspecified
    /// and must not be used.
    pub fn open_in_place<'a>(
        &self,
        sequence_number: u32,
        ciphertext_in_plaintext_out: &'a mut [u8],
        tag: &[u8; TAG_LEN],
    ) -> Result<&'a [u8], error::Unspecified> {
        let mut counter = make_counter(sequence_number);

        // We must verify the tag before decrypting so that
        // `ciphertext_in_plaintext_out` is unmodified if verification fails.
        // This is beyond what we guarantee.
        let poly_key =
            derive_poly1305_key(&self.key.k_2, counter.increment(), self.key.cpu_features);
        verify(poly_key, ciphertext_in_plaintext_out, tag)?;

        let plaintext_in_ciphertext_out = &mut ciphertext_in_plaintext_out[PACKET_LENGTH_LEN..];
        self.key
            .k_2
            .encrypt_in_place(counter, plaintext_in_ciphertext_out);

        Ok(plaintext_in_ciphertext_out)
    }
}

struct Key {
    k_1: chacha::Key,
    k_2: chacha::Key,
    cpu_features: cpu::Features,
}

impl Key {
    fn new(key_material: &[u8; KEY_LEN], cpu_features: cpu::Features) -> Key {
        // The first half becomes K_2 and the second half becomes K_1.
        let (k_2, k_1) = key_material.split_at(chacha::KEY_LEN);
        let k_1: [u8; chacha::KEY_LEN] = k_1.try_into().unwrap();
        let k_2: [u8; chacha::KEY_LEN] = k_2.try_into().unwrap();
        Key {
            k_1: chacha::Key::from(k_1),
            k_2: chacha::Key::from(k_2),
            cpu_features,
        }
    }
}

fn make_counter(sequence_number: u32) -> Counter {
    let nonce = [
        BigEndian::ZERO,
        BigEndian::ZERO,
        BigEndian::from(sequence_number),
    ];
    Counter::zero(Nonce::assume_unique_for_key(*(nonce.as_byte_array())))
}

/// The length of key.
pub const KEY_LEN: usize = chacha::KEY_LEN * 2;

/// The length in bytes of the `packet_length` field in a SSH packet.
pub const PACKET_LENGTH_LEN: usize = 4; // 32 bits

/// The length in bytes of an authentication tag.
pub const TAG_LEN: usize = super::BLOCK_LEN;

fn verify(key: poly1305::Key, msg: &[u8], tag: &[u8; TAG_LEN]) -> Result<(), error::Unspecified> {
    let Tag(calculated_tag) = poly1305::sign(key, msg);
    constant_time::verify_slices_are_equal(calculated_tag.as_ref(), tag)
}
