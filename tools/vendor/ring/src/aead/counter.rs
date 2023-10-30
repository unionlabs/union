// Copyright 2018 Brian Smith.
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
    iv::{Iv, IV_LEN},
    Nonce,
};
use crate::endian::*;
use core::convert::TryInto;

/// A generator of a monotonically increasing series of `Iv`s.
///
/// Intentionally not `Clone` to ensure counters aren't forked.
#[repr(C)]
pub struct Counter<U32> {
    u32s: [U32; COUNTER_LEN],
}

const COUNTER_LEN: usize = 4;

impl<U32> Counter<U32>
where
    U32: Copy,
    U32: Encoding<u32>,
    U32: From<[u8; 4]>,
    U32: Layout,
    [U32; 4]: ArrayEncoding<[u8; IV_LEN]>,
{
    pub fn zero(nonce: Nonce) -> Self {
        Self::new(nonce, 0)
    }
    pub fn one(nonce: Nonce) -> Self {
        Self::new(nonce, 1)
    }

    #[cfg(test)]
    pub fn from_test_vector(nonce: &[u8], initial_counter: u32) -> Self {
        Self::new(
            Nonce::try_assume_unique_for_key(nonce).unwrap(),
            initial_counter,
        )
    }

    fn new(nonce: Nonce, initial_counter: u32) -> Self {
        let mut r = Self {
            u32s: [U32::ZERO; COUNTER_LEN],
        };
        let nonce_index = (U32::COUNTER_INDEX + 1) % COUNTER_LEN;
        (&mut r.u32s[nonce_index..][..3])
            .iter_mut()
            .zip(nonce.as_ref().chunks_exact(4))
            .for_each(|(initial, nonce)| {
                let nonce: &[u8; 4] = nonce.try_into().unwrap();
                *initial = U32::from(*nonce);
            });
        r.u32s[U32::COUNTER_INDEX] = U32::from(initial_counter);
        r
    }

    #[inline]
    pub fn increment(&mut self) -> Iv {
        let current = Self { u32s: self.u32s };
        self.increment_by_less_safe(1);
        current.into()
    }

    #[inline]
    pub fn increment_by_less_safe(&mut self, increment_by: u32) {
        let counter = &mut self.u32s[U32::COUNTER_INDEX];
        let old_value: u32 = (*counter).into();
        *counter = U32::from(old_value + increment_by);
    }
}

pub trait Layout {
    const COUNTER_INDEX: usize;
}

impl Layout for BigEndian<u32> {
    const COUNTER_INDEX: usize = 3;
}

impl Layout for LittleEndian<u32> {
    const COUNTER_INDEX: usize = 0;
}

impl<U32> Into<Iv> for Counter<U32>
where
    [U32; 4]: ArrayEncoding<[u8; IV_LEN]>,
{
    fn into(self) -> Iv {
        Iv::assume_unique_for_key(*self.u32s.as_byte_array())
    }
}
