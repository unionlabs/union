#[cfg(test)]
#[cfg(feature = "nightly")]
macro_rules! test_from_endian_slice {
    ($int: ty, $endian: ident) => {
        paste::paste! {
            quickcheck::quickcheck! {
                fn [<quickcheck_ $int _from_ $endian _slice>](int: $int, pad_length: u8) -> quickcheck::TestResult {
                    type Big = [<$int:upper>];
                    type Primitive = crate::test::types::$int;

                    use crate::test::TestConvert;
                    use crate::int::endian;

                    // pad_length is greater than the size of the integer in bytes
                    if pad_length >= Primitive::BITS as u8 / 8 {
                        return quickcheck::TestResult::discard();
                    }
                    let pad_length = pad_length as usize;

                    #[allow(unused_comparisons)]
                    let mut pad_bits = if int < 0 {
                        u8::MAX // 1111...
                    } else {
                        u8::MIN // 0000...
                    };

                    let mut bytes = int.[<to_ $endian _bytes>](); // random input bytes
                    // first, test that the original bytes as slice is converted back to the same integer
                    let mut passed = TestConvert::into(Big::[<from_ $endian _slice>](&bytes[..])) == Some(int);

                    let bytes_vec = endian::[<$endian _bytes_vec>](&bytes[..], pad_bits, pad_length); // random vector padded with a random amount of bytes
                    // test that the padded bytes are still converted back to the same integer
                    passed &= TestConvert::into(Big::[<from_ $endian _slice>](&bytes_vec[..])) == Some(int);

                    // most significant byte position, range of bytes indices to change to padding bits, range of bytes indices that will result in the same integer without the padding bits
                    let (msb, pad_range, slice_range) = endian::[<$endian _pad>](pad_length, Primitive::BITS);

                    pad_bits = {
                        #[allow(unused_comparisons)]
                        if Primitive::MIN < 0 && (bytes[msb] as i8).is_negative() {
                            u8::MAX
                        } else {
                            u8::MIN
                        }
                    };

                    for item in &mut bytes[pad_range] {
                        *item = pad_bits;
                    }
                    let correct = Some(Big::[<from_ $endian _bytes>](bytes));
                    // test that a shortened slice of bytes is converted to the same integer as the shortened slice that is padded to be the same number of bytes as the size of the integer
                    passed &= Big::[<from_ $endian _slice>](&bytes[slice_range]) == correct;

                    let bytes_vec = endian::[<$endian _bytes_vec>](&bytes[..], pad_bits, pad_length);
                    passed &= Big::[<from_ $endian _slice>](&bytes_vec[..]) == correct;

                    quickcheck::TestResult::from_bool(passed)
                }
            }
        }
    };
}

#[cfg(test)]
#[cfg(feature = "nightly")]
pub(crate) use test_from_endian_slice;

#[cfg(test)]
#[cfg(feature = "nightly")]
use alloc::vec::Vec;

#[cfg(test)]
#[cfg(feature = "nightly")]
use core::ops::{Range, RangeFrom};

#[cfg(feature = "nightly")]
#[cfg(test)]
/// Pad a slice of bytes with leading pad bits so that the resulting vector of bytes represents the same integer as the original slice
pub fn be_bytes_vec(bytes: &[u8], pad_bits: u8, pad_length: usize) -> Vec<u8> {
    let mut bytes_vec = vec![pad_bits; pad_length];
    bytes_vec.append(&mut bytes.to_vec());
    bytes_vec
}

#[cfg(feature = "nightly")]
#[cfg(test)]
pub fn be_pad(pad_length: usize, _bits: u32) -> (usize, Range<usize>, RangeFrom<usize>) {
    (pad_length, 0..pad_length, pad_length..)
}

#[cfg(feature = "nightly")]
#[cfg(test)]
/// Pad a slice of bytes with trailing pad bits so that the resulting vector of bytes represents the same integer as the original slice
pub fn le_bytes_vec(bytes: &[u8], pad_bits: u8, pad_length: usize) -> Vec<u8> {
    let mut bytes_vec = bytes.to_vec();
    bytes_vec.append(&mut vec![pad_bits; pad_length]);
    bytes_vec
}

#[cfg(feature = "nightly")]
#[cfg(test)]
pub fn le_pad(pad_length: usize, bits: u32) -> (usize, Range<usize>, Range<usize>) {
    let bytes = bits as usize / 8;
    (
        bytes - 1 - pad_length,
        (bytes - pad_length)..bytes,
        0..(bytes - pad_length),
    )
}

#[cfg(test)]
macro_rules! tests {
    ($Digit: ident; $int: ty) => {
        #[cfg(feature = "nightly")]
        use crate::test::U8ArrayWrapper;

        test_bignum! {
            function: <$int>::from_be(a: $int)
        }
        test_bignum! {
            function: <$int>::from_le(a: $int)
        }
        test_bignum! {
            function: <$int>::to_be(a: $int)
        }
        test_bignum! {
            function: <$int>::to_le(a: $int)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::to_be_bytes(a: $int)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::to_le_bytes(a: $int)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::to_ne_bytes(a: $int)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::from_be_bytes(a: U8ArrayWrapper<{<$int>::BITS as usize / 8}>)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::from_le_bytes(a: U8ArrayWrapper<{<$int>::BITS as usize / 8}>)
        }

        #[cfg(feature = "nightly")]
        test_bignum! {
            function: <$int>::from_ne_bytes(a: U8ArrayWrapper<{<$int>::BITS as usize / 8}>)
        }

        #[cfg(feature = "nightly")]
        crate::int::endian::test_from_endian_slice!($int, be);

        #[cfg(feature = "nightly")]
        crate::int::endian::test_from_endian_slice!($int, le);
    };
}

#[cfg(test)]
pub(crate) use tests;
