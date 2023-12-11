#[cfg(any(feature = "alloc", feature = "std", test))]
use alloc::string::String;
use core::cmp;
#[cfg(any(feature = "alloc", feature = "std", test))]
use core::str;

use crate::encode::add_padding;
use crate::engine::{Config, Engine};

/// The output mechanism for ChunkedEncoder's encoded bytes.
pub trait Sink {
    type Error;

    /// Handle a chunk of encoded base64 data (as UTF-8 bytes)
    fn write_encoded_bytes(&mut self, encoded: &[u8]) -> Result<(), Self::Error>;
}

const BUF_SIZE: usize = 1024;

/// A base64 encoder that emits encoded bytes in chunks without heap allocation.
pub struct ChunkedEncoder<'e, E: Engine + ?Sized> {
    engine: &'e E,
    max_input_chunk_len: usize,
}

impl<'e, E: Engine + ?Sized> ChunkedEncoder<'e, E> {
    pub fn new(engine: &'e E) -> ChunkedEncoder<'e, E> {
        ChunkedEncoder {
            engine,
            max_input_chunk_len: max_input_length(BUF_SIZE, engine.config().encode_padding()),
        }
    }

    pub fn encode<S: Sink>(&self, bytes: &[u8], sink: &mut S) -> Result<(), S::Error> {
        let mut encode_buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let mut input_index = 0;

        while input_index < bytes.len() {
            // either the full input chunk size, or it's the last iteration
            let input_chunk_len = cmp::min(self.max_input_chunk_len, bytes.len() - input_index);

            let chunk = &bytes[input_index..(input_index + input_chunk_len)];

            let mut b64_bytes_written = self.engine.internal_encode(chunk, &mut encode_buf);

            input_index += input_chunk_len;
            let more_input_left = input_index < bytes.len();

            if self.engine.config().encode_padding() && !more_input_left {
                // no more input, add padding if needed. Buffer will have room because
                // max_input_length leaves room for it.
                b64_bytes_written += add_padding(bytes.len(), &mut encode_buf[b64_bytes_written..]);
            }

            sink.write_encoded_bytes(&encode_buf[0..b64_bytes_written])?;
        }

        Ok(())
    }
}

/// Calculate the longest input that can be encoded for the given output buffer size.
///
/// If the config requires padding, two bytes of buffer space will be set aside so that the last
/// chunk of input can be encoded safely.
///
/// The input length will always be a multiple of 3 so that no encoding state has to be carried over
/// between chunks.
fn max_input_length(encoded_buf_len: usize, padded: bool) -> usize {
    let effective_buf_len = if padded {
        // make room for padding
        encoded_buf_len
            .checked_sub(2)
            .expect("Don't use a tiny buffer")
    } else {
        encoded_buf_len
    };

    // No padding, so just normal base64 expansion.
    (effective_buf_len / 4) * 3
}

// A really simple sink that just appends to a string
#[cfg(any(feature = "alloc", feature = "std", test))]
pub(crate) struct StringSink<'a> {
    string: &'a mut String,
}

#[cfg(any(feature = "alloc", feature = "std", test))]
impl<'a> StringSink<'a> {
    pub(crate) fn new(s: &mut String) -> StringSink {
        StringSink { string: s }
    }
}

#[cfg(any(feature = "alloc", feature = "std", test))]
impl<'a> Sink for StringSink<'a> {
    type Error = ();

    fn write_encoded_bytes(&mut self, s: &[u8]) -> Result<(), Self::Error> {
        self.string.push_str(str::from_utf8(s).unwrap());

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use rand::{
        distributions::{Distribution, Uniform},
        Rng, SeedableRng,
    };

    use crate::{
        alphabet::STANDARD,
        engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig, PAD},
        tests::random_engine,
    };

    use super::*;

    #[test]
    fn chunked_encode_empty() {
        assert_eq!("", chunked_encode_str(&[], PAD));
    }

    #[test]
    fn chunked_encode_intermediate_fast_loop() {
        // > 8 bytes input, will enter the pretty fast loop
        assert_eq!("Zm9vYmFyYmF6cXV4", chunked_encode_str(b"foobarbazqux", PAD));
    }

    #[test]
    fn chunked_encode_fast_loop() {
        // > 32 bytes input, will enter the uber fast loop
        assert_eq!(
            "Zm9vYmFyYmF6cXV4cXV1eGNvcmdlZ3JhdWx0Z2FycGx5eg==",
            chunked_encode_str(b"foobarbazquxquuxcorgegraultgarplyz", PAD)
        );
    }

    #[test]
    fn chunked_encode_slow_loop_only() {
        // < 8 bytes input, slow loop only
        assert_eq!("Zm9vYmFy", chunked_encode_str(b"foobar", PAD));
    }

    #[test]
    fn chunked_encode_matches_normal_encode_random_string_sink() {
        let helper = StringSinkTestHelper;
        chunked_encode_matches_normal_encode_random(&helper);
    }

    #[test]
    fn max_input_length_no_pad() {
        assert_eq!(768, max_input_length(1024, false));
    }

    #[test]
    fn max_input_length_with_pad_decrements_one_triple() {
        assert_eq!(765, max_input_length(1024, true));
    }

    #[test]
    fn max_input_length_with_pad_one_byte_short() {
        assert_eq!(765, max_input_length(1025, true));
    }

    #[test]
    fn max_input_length_with_pad_fits_exactly() {
        assert_eq!(768, max_input_length(1026, true));
    }

    #[test]
    fn max_input_length_cant_use_extra_single_encoded_byte() {
        assert_eq!(300, max_input_length(401, false));
    }

    pub fn chunked_encode_matches_normal_encode_random<S: SinkTestHelper>(sink_test_helper: &S) {
        let mut input_buf: Vec<u8> = Vec::new();
        let mut output_buf = String::new();
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let input_len_range = Uniform::new(1, 10_000);

        for _ in 0..5_000 {
            input_buf.clear();
            output_buf.clear();

            let buf_len = input_len_range.sample(&mut rng);
            for _ in 0..buf_len {
                input_buf.push(rng.gen());
            }

            let engine = random_engine(&mut rng);

            let chunk_encoded_string = sink_test_helper.encode_to_string(&engine, &input_buf);
            engine.encode_string(&input_buf, &mut output_buf);

            assert_eq!(output_buf, chunk_encoded_string, "input len={}", buf_len);
        }
    }

    fn chunked_encode_str(bytes: &[u8], config: GeneralPurposeConfig) -> String {
        let mut s = String::new();

        let mut sink = StringSink::new(&mut s);
        let engine = GeneralPurpose::new(&STANDARD, config);
        let encoder = ChunkedEncoder::new(&engine);
        encoder.encode(bytes, &mut sink).unwrap();

        s
    }

    // An abstraction around sinks so that we can have tests that easily to any sink implementation
    pub trait SinkTestHelper {
        fn encode_to_string<E: Engine>(&self, engine: &E, bytes: &[u8]) -> String;
    }

    struct StringSinkTestHelper;

    impl SinkTestHelper for StringSinkTestHelper {
        fn encode_to_string<E: Engine>(&self, engine: &E, bytes: &[u8]) -> String {
            let encoder = ChunkedEncoder::new(engine);
            let mut s = String::new();
            let mut sink = StringSink::new(&mut s);
            encoder.encode(bytes, &mut sink).unwrap();

            s
        }
    }
}
