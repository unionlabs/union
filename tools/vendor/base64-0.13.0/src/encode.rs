use crate::{Config, PAD_BYTE};
#[cfg(any(feature = "alloc", feature = "std", test))]
use crate::{chunked_encoder, STANDARD};
#[cfg(any(feature = "alloc", feature = "std", test))]
use alloc::{string::String, vec};
use core::convert::TryInto;

///Encode arbitrary octets as base64.
///Returns a String.
///Convenience for `encode_config(input, base64::STANDARD);`.
///
///# Example
///
///```rust
///extern crate base64;
///
///fn main() {
///    let b64 = base64::encode(b"hello world");
///    println!("{}", b64);
///}
///```
#[cfg(any(feature = "alloc", feature = "std", test))]
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    encode_config(input, STANDARD)
}

///Encode arbitrary octets as base64.
///Returns a String.
///
///# Example
///
///```rust
///extern crate base64;
///
///fn main() {
///    let b64 = base64::encode_config(b"hello world~", base64::STANDARD);
///    println!("{}", b64);
///
///    let b64_url = base64::encode_config(b"hello internet~", base64::URL_SAFE);
///    println!("{}", b64_url);
///}
///```
#[cfg(any(feature = "alloc", feature = "std", test))]
pub fn encode_config<T: AsRef<[u8]>>(input: T, config: Config) -> String {
    let mut buf = match encoded_size(input.as_ref().len(), config) {
        Some(n) => vec![0; n],
        None => panic!("integer overflow when calculating buffer size"),
    };

    encode_with_padding(input.as_ref(), config, buf.len(), &mut buf[..]);

    String::from_utf8(buf).expect("Invalid UTF8")
}

///Encode arbitrary octets as base64.
///Writes into the supplied output buffer, which will grow the buffer if needed.
///
///# Example
///
///```rust
///extern crate base64;
///
///fn main() {
///    let mut buf = String::new();
///    base64::encode_config_buf(b"hello world~", base64::STANDARD, &mut buf);
///    println!("{}", buf);
///
///    buf.clear();
///    base64::encode_config_buf(b"hello internet~", base64::URL_SAFE, &mut buf);
///    println!("{}", buf);
///}
///```
#[cfg(any(feature = "alloc", feature = "std", test))]
pub fn encode_config_buf<T: AsRef<[u8]>>(input: T, config: Config, buf: &mut String) {
    let input_bytes = input.as_ref();

    {
        let mut sink = chunked_encoder::StringSink::new(buf);
        let encoder = chunked_encoder::ChunkedEncoder::new(config);

        encoder
            .encode(input_bytes, &mut sink)
            .expect("Writing to a String shouldn't fail")
    }
}

/// Encode arbitrary octets as base64.
/// Writes into the supplied output buffer.
///
/// This is useful if you wish to avoid allocation entirely (e.g. encoding into a stack-resident
/// or statically-allocated buffer).
///
/// # Panics
///
/// If `output` is too small to hold the encoded version of `input`, a panic will result.
///
/// # Example
///
/// ```rust
/// extern crate base64;
///
/// fn main() {
///     let s = b"hello internet!";
///     let mut buf = Vec::new();
///     // make sure we'll have a slice big enough for base64 + padding
///     buf.resize(s.len() * 4 / 3 + 4, 0);
///
///     let bytes_written = base64::encode_config_slice(s,
///                             base64::STANDARD, &mut buf);
///
///     // shorten our vec down to just what was written
///     buf.resize(bytes_written, 0);
///
///     assert_eq!(s, base64::decode(&buf).unwrap().as_slice());
/// }
/// ```
pub fn encode_config_slice<T: AsRef<[u8]>>(input: T, config: Config, output: &mut [u8]) -> usize {
    let input_bytes = input.as_ref();

    let encoded_size = encoded_size(input_bytes.len(), config)
        .expect("usize overflow when calculating buffer size");

    let mut b64_output = &mut output[0..encoded_size];

    encode_with_padding(&input_bytes, config, encoded_size, &mut b64_output);

    encoded_size
}

/// B64-encode and pad (if configured).
///
/// This helper exists to avoid recalculating encoded_size, which is relatively expensive on short
/// inputs.
///
/// `encoded_size` is the encoded size calculated for `input`.
///
/// `output` must be of size `encoded_size`.
///
/// All bytes in `output` will be written to since it is exactly the size of the output.
fn encode_with_padding(input: &[u8], config: Config, encoded_size: usize, output: &mut [u8]) {
    debug_assert_eq!(encoded_size, output.len());

    let b64_bytes_written = encode_to_slice(input, output, config.char_set.encode_table());

    let padding_bytes = if config.pad {
        add_padding(input.len(), &mut output[b64_bytes_written..])
    } else {
        0
    };

    let encoded_bytes = b64_bytes_written
        .checked_add(padding_bytes)
        .expect("usize overflow when calculating b64 length");

    debug_assert_eq!(encoded_size, encoded_bytes);
}

#[inline]
fn read_u64(s: &[u8]) -> u64 {
    u64::from_be_bytes(s[..8].try_into().unwrap())
}

/// Encode input bytes to utf8 base64 bytes. Does not pad.
/// `output` must be long enough to hold the encoded `input` without padding.
/// Returns the number of bytes written.
#[inline]
pub fn encode_to_slice(input: &[u8], output: &mut [u8], encode_table: &[u8; 64]) -> usize {
    let mut input_index: usize = 0;

    const BLOCKS_PER_FAST_LOOP: usize = 4;
    const LOW_SIX_BITS: u64 = 0x3F;

    // we read 8 bytes at a time (u64) but only actually consume 6 of those bytes. Thus, we need
    // 2 trailing bytes to be available to read..
    let last_fast_index = input.len().saturating_sub(BLOCKS_PER_FAST_LOOP * 6 + 2);
    let mut output_index = 0;

    if last_fast_index > 0 {
        while input_index <= last_fast_index {
            // Major performance wins from letting the optimizer do the bounds check once, mostly
            // on the output side
            let input_chunk = &input[input_index..(input_index + (BLOCKS_PER_FAST_LOOP * 6 + 2))];
            let output_chunk = &mut output[output_index..(output_index + BLOCKS_PER_FAST_LOOP * 8)];

            // Hand-unrolling for 32 vs 16 or 8 bytes produces yields performance about equivalent
            // to unsafe pointer code on a Xeon E5-1650v3. 64 byte unrolling was slightly better for
            // large inputs but significantly worse for 50-byte input, unsurprisingly. I suspect
            // that it's a not uncommon use case to encode smallish chunks of data (e.g. a 64-byte
            // SHA-512 digest), so it would be nice if that fit in the unrolled loop at least once.
            // Plus, single-digit percentage performance differences might well be quite different
            // on different hardware.

            let input_u64 = read_u64(&input_chunk[0..]);

            output_chunk[0] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
            output_chunk[1] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
            output_chunk[2] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
            output_chunk[3] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
            output_chunk[4] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
            output_chunk[5] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
            output_chunk[6] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
            output_chunk[7] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

            let input_u64 = read_u64(&input_chunk[6..]);

            output_chunk[8] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
            output_chunk[9] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
            output_chunk[10] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
            output_chunk[11] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
            output_chunk[12] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
            output_chunk[13] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
            output_chunk[14] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
            output_chunk[15] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

            let input_u64 = read_u64(&input_chunk[12..]);

            output_chunk[16] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
            output_chunk[17] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
            output_chunk[18] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
            output_chunk[19] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
            output_chunk[20] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
            output_chunk[21] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
            output_chunk[22] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
            output_chunk[23] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

            let input_u64 = read_u64(&input_chunk[18..]);

            output_chunk[24] = encode_table[((input_u64 >> 58) & LOW_SIX_BITS) as usize];
            output_chunk[25] = encode_table[((input_u64 >> 52) & LOW_SIX_BITS) as usize];
            output_chunk[26] = encode_table[((input_u64 >> 46) & LOW_SIX_BITS) as usize];
            output_chunk[27] = encode_table[((input_u64 >> 40) & LOW_SIX_BITS) as usize];
            output_chunk[28] = encode_table[((input_u64 >> 34) & LOW_SIX_BITS) as usize];
            output_chunk[29] = encode_table[((input_u64 >> 28) & LOW_SIX_BITS) as usize];
            output_chunk[30] = encode_table[((input_u64 >> 22) & LOW_SIX_BITS) as usize];
            output_chunk[31] = encode_table[((input_u64 >> 16) & LOW_SIX_BITS) as usize];

            output_index += BLOCKS_PER_FAST_LOOP * 8;
            input_index += BLOCKS_PER_FAST_LOOP * 6;
        }
    }

    // Encode what's left after the fast loop.

    const LOW_SIX_BITS_U8: u8 = 0x3F;

    let rem = input.len() % 3;
    let start_of_rem = input.len() - rem;

    // start at the first index not handled by fast loop, which may be 0.

    while input_index < start_of_rem {
        let input_chunk = &input[input_index..(input_index + 3)];
        let output_chunk = &mut output[output_index..(output_index + 4)];

        output_chunk[0] = encode_table[(input_chunk[0] >> 2) as usize];
        output_chunk[1] =
            encode_table[((input_chunk[0] << 4 | input_chunk[1] >> 4) & LOW_SIX_BITS_U8) as usize];
        output_chunk[2] =
            encode_table[((input_chunk[1] << 2 | input_chunk[2] >> 6) & LOW_SIX_BITS_U8) as usize];
        output_chunk[3] = encode_table[(input_chunk[2] & LOW_SIX_BITS_U8) as usize];

        input_index += 3;
        output_index += 4;
    }

    if rem == 2 {
        output[output_index] = encode_table[(input[start_of_rem] >> 2) as usize];
        output[output_index + 1] = encode_table[((input[start_of_rem] << 4
            | input[start_of_rem + 1] >> 4)
            & LOW_SIX_BITS_U8) as usize];
        output[output_index + 2] =
            encode_table[((input[start_of_rem + 1] << 2) & LOW_SIX_BITS_U8) as usize];
        output_index += 3;
    } else if rem == 1 {
        output[output_index] = encode_table[(input[start_of_rem] >> 2) as usize];
        output[output_index + 1] =
            encode_table[((input[start_of_rem] << 4) & LOW_SIX_BITS_U8) as usize];
        output_index += 2;
    }

    output_index
}

/// calculate the base64 encoded string size, including padding if appropriate
pub fn encoded_size(bytes_len: usize, config: Config) -> Option<usize> {
    let rem = bytes_len % 3;

    let complete_input_chunks = bytes_len / 3;
    let complete_chunk_output = complete_input_chunks.checked_mul(4);

    if rem > 0 {
        if config.pad {
            complete_chunk_output.and_then(|c| c.checked_add(4))
        } else {
            let encoded_rem = match rem {
                1 => 2,
                2 => 3,
                _ => unreachable!("Impossible remainder"),
            };
            complete_chunk_output.and_then(|c| c.checked_add(encoded_rem))
        }
    } else {
        complete_chunk_output
    }
}

/// Write padding characters.
/// `output` is the slice where padding should be written, of length at least 2.
///
/// Returns the number of padding bytes written.
pub fn add_padding(input_len: usize, output: &mut [u8]) -> usize {
    let rem = input_len % 3;
    let mut bytes_written = 0;
    for _ in 0..((3 - rem) % 3) {
        output[bytes_written] = PAD_BYTE;
        bytes_written += 1;
    }

    bytes_written
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        decode::decode_config_buf,
        tests::{assert_encode_sanity, random_config},
        Config, STANDARD, URL_SAFE_NO_PAD,
    };

    use rand::{
        distributions::{Distribution, Uniform},
        FromEntropy, Rng,
    };
    use std;
    use std::str;

    #[test]
    fn encoded_size_correct_standard() {
        assert_encoded_length(0, 0, STANDARD);

        assert_encoded_length(1, 4, STANDARD);
        assert_encoded_length(2, 4, STANDARD);
        assert_encoded_length(3, 4, STANDARD);

        assert_encoded_length(4, 8, STANDARD);
        assert_encoded_length(5, 8, STANDARD);
        assert_encoded_length(6, 8, STANDARD);

        assert_encoded_length(7, 12, STANDARD);
        assert_encoded_length(8, 12, STANDARD);
        assert_encoded_length(9, 12, STANDARD);

        assert_encoded_length(54, 72, STANDARD);

        assert_encoded_length(55, 76, STANDARD);
        assert_encoded_length(56, 76, STANDARD);
        assert_encoded_length(57, 76, STANDARD);

        assert_encoded_length(58, 80, STANDARD);
    }

    #[test]
    fn encoded_size_correct_no_pad() {
        assert_encoded_length(0, 0, URL_SAFE_NO_PAD);

        assert_encoded_length(1, 2, URL_SAFE_NO_PAD);
        assert_encoded_length(2, 3, URL_SAFE_NO_PAD);
        assert_encoded_length(3, 4, URL_SAFE_NO_PAD);

        assert_encoded_length(4, 6, URL_SAFE_NO_PAD);
        assert_encoded_length(5, 7, URL_SAFE_NO_PAD);
        assert_encoded_length(6, 8, URL_SAFE_NO_PAD);

        assert_encoded_length(7, 10, URL_SAFE_NO_PAD);
        assert_encoded_length(8, 11, URL_SAFE_NO_PAD);
        assert_encoded_length(9, 12, URL_SAFE_NO_PAD);

        assert_encoded_length(54, 72, URL_SAFE_NO_PAD);

        assert_encoded_length(55, 74, URL_SAFE_NO_PAD);
        assert_encoded_length(56, 75, URL_SAFE_NO_PAD);
        assert_encoded_length(57, 76, URL_SAFE_NO_PAD);

        assert_encoded_length(58, 78, URL_SAFE_NO_PAD);
    }

    #[test]
    fn encoded_size_overflow() {
        assert_eq!(None, encoded_size(std::usize::MAX, STANDARD));
    }

    #[test]
    fn encode_config_buf_into_nonempty_buffer_doesnt_clobber_prefix() {
        let mut orig_data = Vec::new();
        let mut prefix = String::new();
        let mut encoded_data_no_prefix = String::new();
        let mut encoded_data_with_prefix = String::new();
        let mut decoded = Vec::new();

        let prefix_len_range = Uniform::new(0, 1000);
        let input_len_range = Uniform::new(0, 1000);

        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..10_000 {
            orig_data.clear();
            prefix.clear();
            encoded_data_no_prefix.clear();
            encoded_data_with_prefix.clear();
            decoded.clear();

            let input_len = input_len_range.sample(&mut rng);

            for _ in 0..input_len {
                orig_data.push(rng.gen());
            }

            let prefix_len = prefix_len_range.sample(&mut rng);
            for _ in 0..prefix_len {
                // getting convenient random single-byte printable chars that aren't base64 is
                // annoying
                prefix.push('#');
            }
            encoded_data_with_prefix.push_str(&prefix);

            let config = random_config(&mut rng);
            encode_config_buf(&orig_data, config, &mut encoded_data_no_prefix);
            encode_config_buf(&orig_data, config, &mut encoded_data_with_prefix);

            assert_eq!(
                encoded_data_no_prefix.len() + prefix_len,
                encoded_data_with_prefix.len()
            );
            assert_encode_sanity(&encoded_data_no_prefix, config, input_len);
            assert_encode_sanity(&encoded_data_with_prefix[prefix_len..], config, input_len);

            // append plain encode onto prefix
            prefix.push_str(&mut encoded_data_no_prefix);

            assert_eq!(prefix, encoded_data_with_prefix);

            decode_config_buf(&encoded_data_no_prefix, config, &mut decoded).unwrap();
            assert_eq!(orig_data, decoded);
        }
    }

    #[test]
    fn encode_config_slice_into_nonempty_buffer_doesnt_clobber_suffix() {
        let mut orig_data = Vec::new();
        let mut encoded_data = Vec::new();
        let mut encoded_data_original_state = Vec::new();
        let mut decoded = Vec::new();

        let input_len_range = Uniform::new(0, 1000);

        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..10_000 {
            orig_data.clear();
            encoded_data.clear();
            encoded_data_original_state.clear();
            decoded.clear();

            let input_len = input_len_range.sample(&mut rng);

            for _ in 0..input_len {
                orig_data.push(rng.gen());
            }

            // plenty of existing garbage in the encoded buffer
            for _ in 0..10 * input_len {
                encoded_data.push(rng.gen());
            }

            encoded_data_original_state.extend_from_slice(&encoded_data);

            let config = random_config(&mut rng);

            let encoded_size = encoded_size(input_len, config).unwrap();

            assert_eq!(
                encoded_size,
                encode_config_slice(&orig_data, config, &mut encoded_data)
            );

            assert_encode_sanity(
                std::str::from_utf8(&encoded_data[0..encoded_size]).unwrap(),
                config,
                input_len,
            );

            assert_eq!(
                &encoded_data[encoded_size..],
                &encoded_data_original_state[encoded_size..]
            );

            decode_config_buf(&encoded_data[0..encoded_size], config, &mut decoded).unwrap();
            assert_eq!(orig_data, decoded);
        }
    }

    #[test]
    fn encode_config_slice_fits_into_precisely_sized_slice() {
        let mut orig_data = Vec::new();
        let mut encoded_data = Vec::new();
        let mut decoded = Vec::new();

        let input_len_range = Uniform::new(0, 1000);

        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..10_000 {
            orig_data.clear();
            encoded_data.clear();
            decoded.clear();

            let input_len = input_len_range.sample(&mut rng);

            for _ in 0..input_len {
                orig_data.push(rng.gen());
            }

            let config = random_config(&mut rng);

            let encoded_size = encoded_size(input_len, config).unwrap();

            encoded_data.resize(encoded_size, 0);

            assert_eq!(
                encoded_size,
                encode_config_slice(&orig_data, config, &mut encoded_data)
            );

            assert_encode_sanity(
                std::str::from_utf8(&encoded_data[0..encoded_size]).unwrap(),
                config,
                input_len,
            );

            decode_config_buf(&encoded_data[0..encoded_size], config, &mut decoded).unwrap();
            assert_eq!(orig_data, decoded);
        }
    }

    #[test]
    fn encode_to_slice_random_valid_utf8() {
        let mut input = Vec::new();
        let mut output = Vec::new();

        let input_len_range = Uniform::new(0, 1000);

        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..10_000 {
            input.clear();
            output.clear();

            let input_len = input_len_range.sample(&mut rng);

            for _ in 0..input_len {
                input.push(rng.gen());
            }

            let config = random_config(&mut rng);

            // fill up the output buffer with garbage
            let encoded_size = encoded_size(input_len, config).unwrap();
            for _ in 0..encoded_size {
                output.push(rng.gen());
            }

            let orig_output_buf = output.to_vec();

            let bytes_written =
                encode_to_slice(&input, &mut output, config.char_set.encode_table());

            // make sure the part beyond bytes_written is the same garbage it was before
            assert_eq!(orig_output_buf[bytes_written..], output[bytes_written..]);

            // make sure the encoded bytes are UTF-8
            let _ = str::from_utf8(&output[0..bytes_written]).unwrap();
        }
    }

    #[test]
    fn encode_with_padding_random_valid_utf8() {
        let mut input = Vec::new();
        let mut output = Vec::new();

        let input_len_range = Uniform::new(0, 1000);

        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..10_000 {
            input.clear();
            output.clear();

            let input_len = input_len_range.sample(&mut rng);

            for _ in 0..input_len {
                input.push(rng.gen());
            }

            let config = random_config(&mut rng);

            // fill up the output buffer with garbage
            let encoded_size = encoded_size(input_len, config).unwrap();
            for _ in 0..encoded_size + 1000 {
                output.push(rng.gen());
            }

            let orig_output_buf = output.to_vec();

            encode_with_padding(&input, config, encoded_size, &mut output[0..encoded_size]);

            // make sure the part beyond b64 is the same garbage it was before
            assert_eq!(orig_output_buf[encoded_size..], output[encoded_size..]);

            // make sure the encoded bytes are UTF-8
            let _ = str::from_utf8(&output[0..encoded_size]).unwrap();
        }
    }

    #[test]
    fn add_padding_random_valid_utf8() {
        let mut output = Vec::new();

        let mut rng = rand::rngs::SmallRng::from_entropy();

        // cover our bases for length % 3
        for input_len in 0..10 {
            output.clear();

            // fill output with random
            for _ in 0..10 {
                output.push(rng.gen());
            }

            let orig_output_buf = output.to_vec();

            let bytes_written = add_padding(input_len, &mut output);

            // make sure the part beyond bytes_written is the same garbage it was before
            assert_eq!(orig_output_buf[bytes_written..], output[bytes_written..]);

            // make sure the encoded bytes are UTF-8
            let _ = str::from_utf8(&output[0..bytes_written]).unwrap();
        }
    }

    fn assert_encoded_length(input_len: usize, encoded_len: usize, config: Config) {
        assert_eq!(encoded_len, encoded_size(input_len, config).unwrap());

        let mut bytes: Vec<u8> = Vec::new();
        let mut rng = rand::rngs::SmallRng::from_entropy();

        for _ in 0..input_len {
            bytes.push(rng.gen());
        }

        let encoded = encode_config(&bytes, config);
        assert_encode_sanity(&encoded, config, input_len);

        assert_eq!(encoded_len, encoded.len());
    }

    #[test]
    fn encode_imap() {
        assert_eq!(
            encode_config(b"\xFB\xFF", crate::IMAP_MUTF7),
            encode_config(b"\xFB\xFF", crate::STANDARD_NO_PAD).replace("/", ",")
        );
    }
}
