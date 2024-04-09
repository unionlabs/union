//! Provides a "legacy" version of SSZ encoding for `Option<T> where T: Encode + Decode`.
//!
//! The SSZ specification changed in 2021 to use a 1-byte union selector, instead of a 4-byte one
//! which was used in the Lighthouse database.
//!
//! Users can use the `four_byte_option_impl` macro to define a module that can be used with the
//! `#[ssz(with = "module")]`.
//!
//! ## Example
//!
//! ```rust
//! use ssz_derive::{Encode, Decode};
//! use ssz::four_byte_option_impl;
//!
//! four_byte_option_impl!(impl_for_u64, u64);
//!
//! #[derive(Encode, Decode)]
//! struct Foo {
//!     #[ssz(with = "impl_for_u64")]
//!     a: Option<u64>,
//! }
//! ```

use crate::*;

#[macro_export]
macro_rules! four_byte_option_impl {
    ($mod_name: ident, $type: ty) => {
        #[allow(dead_code)]
        mod $mod_name {
            use super::*;

            pub mod encode {
                #[allow(unused_imports)]
                use $crate::{Decode, Encode};

                use super::*;

                pub fn is_ssz_fixed_len() -> bool {
                    false
                }

                pub fn ssz_fixed_len() -> usize {
                    $crate::BYTES_PER_LENGTH_OFFSET
                }

                pub fn ssz_bytes_len(opt: &Option<$type>) -> usize {
                    if let Some(some) = opt {
                        let len = if <$type as Encode>::is_ssz_fixed_len() {
                            <$type as Encode>::ssz_fixed_len()
                        } else {
                            <$type as Encode>::ssz_bytes_len(some)
                        };
                        len + $crate::BYTES_PER_LENGTH_OFFSET
                    } else {
                        $crate::BYTES_PER_LENGTH_OFFSET
                    }
                }

                pub fn ssz_append(opt: &Option<$type>, buf: &mut Vec<u8>) {
                    match opt {
                        None => buf
                            .extend_from_slice(&$crate::legacy::encode_four_byte_union_selector(0)),
                        Some(t) => {
                            buf.extend_from_slice(
                                &$crate::legacy::encode_four_byte_union_selector(1),
                            );
                            t.ssz_append(buf);
                        }
                    }
                }

                pub fn as_ssz_bytes(opt: &Option<$type>) -> Vec<u8> {
                    let mut buf = vec![];

                    ssz_append(opt, &mut buf);

                    buf
                }
            }

            pub mod decode {
                #[allow(unused_imports)]
                use $crate::{Decode, Encode};

                use super::*;

                pub fn is_ssz_fixed_len() -> bool {
                    false
                }

                pub fn ssz_fixed_len() -> usize {
                    $crate::BYTES_PER_LENGTH_OFFSET
                }

                pub fn from_ssz_bytes(bytes: &[u8]) -> Result<Option<$type>, $crate::DecodeError> {
                    if bytes.len() < $crate::BYTES_PER_LENGTH_OFFSET {
                        return Err($crate::DecodeError::InvalidByteLength {
                            len: bytes.len(),
                            expected: $crate::BYTES_PER_LENGTH_OFFSET,
                        });
                    }

                    let (index_bytes, value_bytes) =
                        bytes.split_at($crate::BYTES_PER_LENGTH_OFFSET);

                    let index = $crate::legacy::read_four_byte_union_selector(index_bytes)?;
                    if index == 0 {
                        Ok(None)
                    } else if index == 1 {
                        Ok(Some(<$type as Decode>::from_ssz_bytes(value_bytes)?))
                    } else {
                        Err($crate::DecodeError::BytesInvalid(format!(
                            "{} is not a valid union index for Option<T>",
                            index
                        )))
                    }
                }
            }
        }
    };
}

pub fn encode_four_byte_union_selector(selector: usize) -> [u8; BYTES_PER_LENGTH_OFFSET] {
    encode_length(selector)
}

pub fn read_four_byte_union_selector(bytes: &[u8]) -> Result<usize, DecodeError> {
    read_offset(bytes)
}
