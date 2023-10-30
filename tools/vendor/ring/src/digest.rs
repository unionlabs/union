// Copyright 2015-2019 Brian Smith.
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

//! SHA-2 and the legacy SHA-1 digest algorithm.
//!
//! If all the data is available in a single contiguous slice then the `digest`
//! function should be used. Otherwise, the digest can be calculated in
//! multiple steps using `Context`.

// Note on why are we doing things the hard way: It would be easy to implement
// this using the C `EVP_MD`/`EVP_MD_CTX` interface. However, if we were to do
// things that way, we'd have a hard dependency on `malloc` and other overhead.
// The goal for this implementation is to drive the overhead as close to zero
// as possible.

use crate::{
    c, cpu, debug,
    endian::{self, BigEndian},
    polyfill,
};
use core::num::Wrapping;

mod sha1;
mod sha2;

#[derive(Clone)]
pub(crate) struct BlockContext {
    state: State,

    // Note that SHA-512 has a 128-bit input bit counter, but this
    // implementation only supports up to 2^64-1 input bits for all algorithms,
    // so a 64-bit counter is more than sufficient.
    completed_data_blocks: u64,

    /// The context's algorithm.
    pub algorithm: &'static Algorithm,

    cpu_features: cpu::Features,
}

impl BlockContext {
    pub(crate) fn new(algorithm: &'static Algorithm) -> Self {
        Self {
            state: algorithm.initial_state,
            completed_data_blocks: 0,
            algorithm,
            cpu_features: cpu::features(),
        }
    }

    #[inline]
    pub(crate) fn update(&mut self, input: &[u8]) {
        let num_blocks = input.len() / self.algorithm.block_len;
        assert_eq!(num_blocks * self.algorithm.block_len, input.len());
        if num_blocks > 0 {
            unsafe {
                (self.algorithm.block_data_order)(&mut self.state, input.as_ptr(), num_blocks);
            }
            self.completed_data_blocks = self
                .completed_data_blocks
                .checked_add(polyfill::u64_from_usize(num_blocks))
                .unwrap();
        }
    }

    pub(crate) fn finish(mut self, pending: &mut [u8], num_pending: usize) -> Digest {
        let block_len = self.algorithm.block_len;
        assert_eq!(pending.len(), block_len);
        assert!(num_pending <= pending.len());

        let mut padding_pos = num_pending;
        pending[padding_pos] = 0x80;
        padding_pos += 1;

        if padding_pos > block_len - self.algorithm.len_len {
            polyfill::slice::fill(&mut pending[padding_pos..block_len], 0);
            unsafe {
                (self.algorithm.block_data_order)(&mut self.state, pending.as_ptr(), 1);
            }
            // We don't increase |self.completed_data_blocks| because the
            // padding isn't data, and so it isn't included in the data length.
            padding_pos = 0;
        }

        polyfill::slice::fill(&mut pending[padding_pos..(block_len - 8)], 0);

        // Output the length, in bits, in big endian order.
        let completed_data_bits = self
            .completed_data_blocks
            .checked_mul(polyfill::u64_from_usize(block_len))
            .unwrap()
            .checked_add(polyfill::u64_from_usize(num_pending))
            .unwrap()
            .checked_mul(8)
            .unwrap();
        pending[(block_len - 8)..block_len].copy_from_slice(&u64::to_be_bytes(completed_data_bits));

        unsafe {
            (self.algorithm.block_data_order)(&mut self.state, pending.as_ptr(), 1);
        }

        Digest {
            algorithm: self.algorithm,
            value: (self.algorithm.format_output)(self.state),
        }
    }
}

/// A context for multi-step (Init-Update-Finish) digest calculations.
///
/// # Examples
///
/// ```
/// use ring::digest;
///
/// let one_shot = digest::digest(&digest::SHA384, b"hello, world");
///
/// let mut ctx = digest::Context::new(&digest::SHA384);
/// ctx.update(b"hello");
/// ctx.update(b", ");
/// ctx.update(b"world");
/// let multi_part = ctx.finish();
///
/// assert_eq!(&one_shot.as_ref(), &multi_part.as_ref());
/// ```
#[derive(Clone)]
pub struct Context {
    block: BlockContext,
    // TODO: More explicitly force 64-bit alignment for |pending|.
    pending: [u8; MAX_BLOCK_LEN],
    num_pending: usize,
}

impl Context {
    /// Constructs a new context.
    pub fn new(algorithm: &'static Algorithm) -> Self {
        Self {
            block: BlockContext::new(algorithm),
            pending: [0u8; MAX_BLOCK_LEN],
            num_pending: 0,
        }
    }

    pub(crate) fn clone_from(block: &BlockContext) -> Self {
        Self {
            block: block.clone(),
            pending: [0u8; MAX_BLOCK_LEN],
            num_pending: 0,
        }
    }

    /// Updates the digest with all the data in `data`. `update` may be called
    /// zero or more times until `finish` is called. It must not be called
    /// after `finish` has been called.
    pub fn update(&mut self, data: &[u8]) {
        let block_len = self.block.algorithm.block_len;
        if data.len() < block_len - self.num_pending {
            self.pending[self.num_pending..(self.num_pending + data.len())].copy_from_slice(data);
            self.num_pending += data.len();
            return;
        }

        let mut remaining = data;
        if self.num_pending > 0 {
            let to_copy = block_len - self.num_pending;
            self.pending[self.num_pending..block_len].copy_from_slice(&data[..to_copy]);
            self.block.update(&self.pending[..block_len]);
            remaining = &remaining[to_copy..];
            self.num_pending = 0;
        }

        let num_blocks = remaining.len() / block_len;
        let num_to_save_for_later = remaining.len() % block_len;
        self.block.update(&remaining[..(num_blocks * block_len)]);
        if num_to_save_for_later > 0 {
            self.pending[..num_to_save_for_later]
                .copy_from_slice(&remaining[(remaining.len() - num_to_save_for_later)..]);
            self.num_pending = num_to_save_for_later;
        }
    }

    /// Finalizes the digest calculation and returns the digest value. `finish`
    /// consumes the context so it cannot be (mis-)used after `finish` has been
    /// called.
    pub fn finish(mut self) -> Digest {
        let block_len = self.block.algorithm.block_len;
        self.block
            .finish(&mut self.pending[..block_len], self.num_pending)
    }

    /// The algorithm that this context is using.
    #[inline(always)]
    pub fn algorithm(&self) -> &'static Algorithm {
        self.block.algorithm
    }
}

/// Returns the digest of `data` using the given digest algorithm.
///
/// # Examples:
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// use ring::{digest, test};
/// let expected_hex = "09ca7e4eaa6e8ae9c7d261167129184883644d07dfba7cbfbc4c8a2e08360d5b";
/// let expected: Vec<u8> = test::from_hex(expected_hex).unwrap();
/// let actual = digest::digest(&digest::SHA256, b"hello, world");
///
/// assert_eq!(&expected, &actual.as_ref());
/// # }
/// ```
pub fn digest(algorithm: &'static Algorithm, data: &[u8]) -> Digest {
    let mut ctx = Context::new(algorithm);
    ctx.update(data);
    ctx.finish()
}

/// A calculated digest value.
///
/// Use `as_ref` to get the value as a `&[u8]`.
#[derive(Clone, Copy)]
pub struct Digest {
    value: Output,
    algorithm: &'static Algorithm,
}

impl Digest {
    /// The algorithm that was used to calculate the digest value.
    #[inline(always)]
    pub fn algorithm(&self) -> &'static Algorithm {
        self.algorithm
    }
}

impl AsRef<[u8]> for Digest {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        let as64 = unsafe { &self.value.as64 };
        &endian::as_byte_slice(as64)[..self.algorithm.output_len]
    }
}

impl core::fmt::Debug for Digest {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{:?}:", self.algorithm)?;
        debug::write_hex_bytes(fmt, self.as_ref())
    }
}

/// A digest algorithm.
pub struct Algorithm {
    /// The length of a finalized digest.
    pub output_len: usize,

    /// The size of the chaining value of the digest function, in bytes. For
    /// non-truncated algorithms (SHA-1, SHA-256, SHA-512), this is equal to
    /// `output_len`. For truncated algorithms (e.g. SHA-384, SHA-512/256),
    /// this is equal to the length before truncation. This is mostly helpful
    /// for determining the size of an HMAC key that is appropriate for the
    /// digest algorithm.
    pub chaining_len: usize,

    /// The internal block length.
    pub block_len: usize,

    /// The length of the length in the padding.
    len_len: usize,

    block_data_order: unsafe extern "C" fn(state: &mut State, data: *const u8, num: c::size_t),
    format_output: fn(input: State) -> Output,

    initial_state: State,

    id: AlgorithmID,
}

#[derive(Debug, Eq, PartialEq)]
enum AlgorithmID {
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    SHA512_256,
}

impl PartialEq for Algorithm {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Algorithm {}

derive_debug_via_id!(Algorithm);

/// SHA-1 as specified in [FIPS 180-4]. Deprecated.
///
/// [FIPS 180-4]: http://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub static SHA1_FOR_LEGACY_USE_ONLY: Algorithm = Algorithm {
    output_len: sha1::OUTPUT_LEN,
    chaining_len: sha1::CHAINING_LEN,
    block_len: sha1::BLOCK_LEN,
    len_len: 64 / 8,
    block_data_order: sha1::block_data_order,
    format_output: sha256_format_output,
    initial_state: State {
        as32: [
            Wrapping(0x67452301u32),
            Wrapping(0xefcdab89u32),
            Wrapping(0x98badcfeu32),
            Wrapping(0x10325476u32),
            Wrapping(0xc3d2e1f0u32),
            Wrapping(0),
            Wrapping(0),
            Wrapping(0),
        ],
    },
    id: AlgorithmID::SHA1,
};

/// SHA-256 as specified in [FIPS 180-4].
///
/// [FIPS 180-4]: http://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub static SHA256: Algorithm = Algorithm {
    output_len: SHA256_OUTPUT_LEN,
    chaining_len: SHA256_OUTPUT_LEN,
    block_len: 512 / 8,
    len_len: 64 / 8,
    block_data_order: sha2::GFp_sha256_block_data_order,
    format_output: sha256_format_output,
    initial_state: State {
        as32: [
            Wrapping(0x6a09e667u32),
            Wrapping(0xbb67ae85u32),
            Wrapping(0x3c6ef372u32),
            Wrapping(0xa54ff53au32),
            Wrapping(0x510e527fu32),
            Wrapping(0x9b05688cu32),
            Wrapping(0x1f83d9abu32),
            Wrapping(0x5be0cd19u32),
        ],
    },
    id: AlgorithmID::SHA256,
};

/// SHA-384 as specified in [FIPS 180-4].
///
/// [FIPS 180-4]: http://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub static SHA384: Algorithm = Algorithm {
    output_len: SHA384_OUTPUT_LEN,
    chaining_len: SHA512_OUTPUT_LEN,
    block_len: SHA512_BLOCK_LEN,
    len_len: SHA512_LEN_LEN,
    block_data_order: sha2::GFp_sha512_block_data_order,
    format_output: sha512_format_output,
    initial_state: State {
        as64: [
            Wrapping(0xcbbb9d5dc1059ed8),
            Wrapping(0x629a292a367cd507),
            Wrapping(0x9159015a3070dd17),
            Wrapping(0x152fecd8f70e5939),
            Wrapping(0x67332667ffc00b31),
            Wrapping(0x8eb44a8768581511),
            Wrapping(0xdb0c2e0d64f98fa7),
            Wrapping(0x47b5481dbefa4fa4),
        ],
    },
    id: AlgorithmID::SHA384,
};

/// SHA-512 as specified in [FIPS 180-4].
///
/// [FIPS 180-4]: http://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub static SHA512: Algorithm = Algorithm {
    output_len: SHA512_OUTPUT_LEN,
    chaining_len: SHA512_OUTPUT_LEN,
    block_len: SHA512_BLOCK_LEN,
    len_len: SHA512_LEN_LEN,
    block_data_order: sha2::GFp_sha512_block_data_order,
    format_output: sha512_format_output,
    initial_state: State {
        as64: [
            Wrapping(0x6a09e667f3bcc908),
            Wrapping(0xbb67ae8584caa73b),
            Wrapping(0x3c6ef372fe94f82b),
            Wrapping(0xa54ff53a5f1d36f1),
            Wrapping(0x510e527fade682d1),
            Wrapping(0x9b05688c2b3e6c1f),
            Wrapping(0x1f83d9abfb41bd6b),
            Wrapping(0x5be0cd19137e2179),
        ],
    },
    id: AlgorithmID::SHA512,
};

/// SHA-512/256 as specified in [FIPS 180-4].
///
/// This is *not* the same as just truncating the output of SHA-512, as
/// SHA-512/256 has its own initial state distinct from SHA-512's initial
/// state.
///
/// [FIPS 180-4]: http://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.180-4.pdf
pub static SHA512_256: Algorithm = Algorithm {
    output_len: SHA512_256_OUTPUT_LEN,
    chaining_len: SHA512_OUTPUT_LEN,
    block_len: SHA512_BLOCK_LEN,
    len_len: SHA512_LEN_LEN,
    block_data_order: sha2::GFp_sha512_block_data_order,
    format_output: sha512_format_output,
    initial_state: State {
        as64: [
            Wrapping(0x22312194fc2bf72c),
            Wrapping(0x9f555fa3c84c64c2),
            Wrapping(0x2393b86b6f53b151),
            Wrapping(0x963877195940eabd),
            Wrapping(0x96283ee2a88effe3),
            Wrapping(0xbe5e1e2553863992),
            Wrapping(0x2b0199fc2c85b8aa),
            Wrapping(0x0eb72ddc81c52ca2),
        ],
    },
    id: AlgorithmID::SHA512_256,
};

#[derive(Clone, Copy)] // XXX: Why do we need to be `Copy`?
#[repr(C)]
union State {
    as64: [Wrapping<u64>; sha2::CHAINING_WORDS],
    as32: [Wrapping<u32>; sha2::CHAINING_WORDS],
}

#[derive(Clone, Copy)]
#[repr(C)]
union Output {
    as64: [BigEndian<u64>; 512 / 8 / core::mem::size_of::<BigEndian<u64>>()],
    as32: [BigEndian<u32>; 256 / 8 / core::mem::size_of::<BigEndian<u32>>()],
}

/// The maximum block length (`Algorithm::block_len`) of all the algorithms in
/// this module.
pub const MAX_BLOCK_LEN: usize = 1024 / 8;

/// The maximum output length (`Algorithm::output_len`) of all the algorithms
/// in this module.
pub const MAX_OUTPUT_LEN: usize = 512 / 8;

/// The maximum chaining length (`Algorithm::chaining_len`) of all the
/// algorithms in this module.
pub const MAX_CHAINING_LEN: usize = MAX_OUTPUT_LEN;

fn sha256_format_output(input: State) -> Output {
    let input = unsafe { &input.as32 };
    Output {
        as32: [
            BigEndian::from(input[0]),
            BigEndian::from(input[1]),
            BigEndian::from(input[2]),
            BigEndian::from(input[3]),
            BigEndian::from(input[4]),
            BigEndian::from(input[5]),
            BigEndian::from(input[6]),
            BigEndian::from(input[7]),
        ],
    }
}

fn sha512_format_output(input: State) -> Output {
    let input = unsafe { &input.as64 };
    Output {
        as64: [
            BigEndian::from(input[0]),
            BigEndian::from(input[1]),
            BigEndian::from(input[2]),
            BigEndian::from(input[3]),
            BigEndian::from(input[4]),
            BigEndian::from(input[5]),
            BigEndian::from(input[6]),
            BigEndian::from(input[7]),
        ],
    }
}

/// The length of the output of SHA-1, in bytes.
pub const SHA1_OUTPUT_LEN: usize = sha1::OUTPUT_LEN;

/// The length of the output of SHA-256, in bytes.
pub const SHA256_OUTPUT_LEN: usize = 256 / 8;

/// The length of the output of SHA-384, in bytes.
pub const SHA384_OUTPUT_LEN: usize = 384 / 8;

/// The length of the output of SHA-512, in bytes.
pub const SHA512_OUTPUT_LEN: usize = 512 / 8;

/// The length of the output of SHA-512/256, in bytes.
pub const SHA512_256_OUTPUT_LEN: usize = 256 / 8;

/// The length of a block for SHA-512-based algorithms, in bytes.
const SHA512_BLOCK_LEN: usize = 1024 / 8;

/// The length of the length field for SHA-512-based algorithms, in bytes.
const SHA512_LEN_LEN: usize = 128 / 8;

#[cfg(test)]
mod tests {

    mod max_input {
        use super::super::super::digest;
        use crate::polyfill;
        use alloc::vec;

        macro_rules! max_input_tests {
            ( $algorithm_name:ident ) => {
                mod $algorithm_name {
                    use super::super::super::super::digest;

                    #[test]
                    fn max_input_test() {
                        super::max_input_test(&digest::$algorithm_name);
                    }

                    #[test]
                    #[should_panic]
                    fn too_long_input_test_block() {
                        super::too_long_input_test_block(&digest::$algorithm_name);
                    }

                    #[test]
                    #[should_panic]
                    fn too_long_input_test_byte() {
                        super::too_long_input_test_byte(&digest::$algorithm_name);
                    }
                }
            };
        }

        fn max_input_test(alg: &'static digest::Algorithm) {
            let mut context = nearly_full_context(alg);
            let next_input = vec![0u8; alg.block_len - 1];
            context.update(&next_input);
            let _ = context.finish(); // no panic
        }

        fn too_long_input_test_block(alg: &'static digest::Algorithm) {
            let mut context = nearly_full_context(alg);
            let next_input = vec![0u8; alg.block_len];
            context.update(&next_input);
            let _ = context.finish(); // should panic
        }

        fn too_long_input_test_byte(alg: &'static digest::Algorithm) {
            let mut context = nearly_full_context(alg);
            let next_input = vec![0u8; alg.block_len - 1];
            context.update(&next_input); // no panic
            context.update(&[0]);
            let _ = context.finish(); // should panic
        }

        fn nearly_full_context(alg: &'static digest::Algorithm) -> digest::Context {
            // All implementations currently support up to 2^64-1 bits
            // of input; according to the spec, SHA-384 and SHA-512
            // support up to 2^128-1, but that's not implemented yet.
            let max_bytes = 1u64 << (64 - 3);
            let max_blocks = max_bytes / polyfill::u64_from_usize(alg.block_len);
            digest::Context {
                block: digest::BlockContext {
                    state: alg.initial_state,
                    completed_data_blocks: max_blocks - 1,
                    algorithm: alg,
                    cpu_features: crate::cpu::features(),
                },
                pending: [0u8; digest::MAX_BLOCK_LEN],
                num_pending: 0,
            }
        }

        max_input_tests!(SHA1_FOR_LEGACY_USE_ONLY);
        max_input_tests!(SHA256);
        max_input_tests!(SHA384);
        max_input_tests!(SHA512);
    }
}
