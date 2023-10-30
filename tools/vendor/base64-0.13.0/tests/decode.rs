extern crate base64;

use base64::*;

mod helpers;

use self::helpers::*;

#[test]
fn decode_rfc4648_0() {
    compare_decode("", "");
}

#[test]
fn decode_rfc4648_1() {
    compare_decode("f", "Zg==");
}

#[test]
fn decode_rfc4648_1_just_a_bit_of_padding() {
    // allows less padding than required
    compare_decode("f", "Zg=");
}

#[test]
fn decode_rfc4648_1_no_padding() {
    compare_decode("f", "Zg");
}

#[test]
fn decode_rfc4648_2() {
    compare_decode("fo", "Zm8=");
}

#[test]
fn decode_rfc4648_2_no_padding() {
    compare_decode("fo", "Zm8");
}

#[test]
fn decode_rfc4648_3() {
    compare_decode("foo", "Zm9v");
}

#[test]
fn decode_rfc4648_4() {
    compare_decode("foob", "Zm9vYg==");
}

#[test]
fn decode_rfc4648_4_no_padding() {
    compare_decode("foob", "Zm9vYg");
}

#[test]
fn decode_rfc4648_5() {
    compare_decode("fooba", "Zm9vYmE=");
}

#[test]
fn decode_rfc4648_5_no_padding() {
    compare_decode("fooba", "Zm9vYmE");
}

#[test]
fn decode_rfc4648_6() {
    compare_decode("foobar", "Zm9vYmFy");
}

#[test]
fn decode_reject_null() {
    assert_eq!(
        DecodeError::InvalidByte(3, 0x0),
        decode_config("YWx\0pY2U==", config_std_pad()).unwrap_err()
    );
}

#[test]
fn decode_single_pad_byte_after_2_chars_in_trailing_quad_ok() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("Zg=");

        let input_len = num_quads * 3 + 1;

        // Since there are 3 bytes in the trailing quad, want to be sure this allows for the fact
        // that it could be bad padding rather than assuming that it will decode to 2 bytes and
        // therefore allow 1 extra round of fast decode logic (stage 1 / 2).

        let mut decoded = Vec::new();
        decoded.resize(input_len, 0);

        assert_eq!(
            input_len,
            decode_config_slice(&s, STANDARD, &mut decoded).unwrap()
        );
    }
}

//this is a MAY in the rfc: https://tools.ietf.org/html/rfc4648#section-3.3
#[test]
fn decode_1_pad_byte_in_fast_loop_then_extra_padding_chunk_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2U=====");

        // since the first 8 bytes are handled in stage 1 or 2, the padding is detected as a
        // generic invalid byte, not specifcally a padding issue.
        // Could argue that the *next* padding byte (in the next quad) is technically the first
        // erroneous one, but reporting that accurately is more complex and probably nobody cares
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 7, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_2_pad_bytes_in_leftovers_then_extra_padding_chunk_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2UABB====");

        // 6 bytes (4 padding) after last 8-byte chunk, so it's decoded by stage 4.
        // First padding byte is invalid.
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 10, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_valid_bytes_after_padding_in_leftovers_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("YWxpY2UABB=B");

        // 4 bytes after last 8-byte chunk, so it's decoded by stage 4.
        // First (and only) padding byte is invalid.
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 10, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_absurd_pad_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("==Y=Wx===pY=2U=====");

        // Plenty of remaining bytes, so handled by stage 1 or 2.
        // first padding byte
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_extra_padding_after_1_pad_bytes_in_trailing_quad_returns_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("EEE===");

        // handled by stage 1, 2, or 4 depending on length
        // first padding byte -- which would be legal if it was the only padding
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 3, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_extra_padding_after_2_pad_bytes_in_trailing_quad_2_returns_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("EE====");

        // handled by stage 1, 2, or 4 depending on length
        // first padding byte -- which would be legal if it was by itself
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 2, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_start_quad_with_padding_returns_error() {
    for num_quads in 0..25 {
        // add enough padding to ensure that we'll hit all 4 stages at the different lengths
        for pad_bytes in 1..32 {
            let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
            let padding: String = std::iter::repeat("=").take(pad_bytes).collect();
            s.push_str(&padding);

            if pad_bytes % 4 == 1 {
                // detected in early length check
                assert_eq!(DecodeError::InvalidLength, decode(&s).unwrap_err());
            } else {
                // padding lengths 2 - 8 are handled by stage 4
                // padding length >= 8 will hit at least one chunk at stages 1, 2, 3 at different
                // prefix lengths
                assert_eq!(
                    DecodeError::InvalidByte(num_quads * 4, b'='),
                    decode(&s).unwrap_err()
                );
            }
        }
    }
}

#[test]
fn decode_padding_followed_by_non_padding_returns_error() {
    for num_quads in 0..25 {
        for pad_bytes in 0..31 {
            let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
            let padding: String = std::iter::repeat("=").take(pad_bytes).collect();
            s.push_str(&padding);
            s.push_str("E");

            if pad_bytes % 4 == 0 {
                assert_eq!(DecodeError::InvalidLength, decode(&s).unwrap_err());
            } else {
                // pad len 1 - 8 will be handled by stage 4
                // pad len 9 (suffix len 10) will have 8 bytes of padding handled by stage 3
                // first padding byte
                assert_eq!(
                    DecodeError::InvalidByte(num_quads * 4, b'='),
                    decode(&s).unwrap_err()
                );
            }
        }
    }
}

#[test]
fn decode_one_char_in_quad_with_padding_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push_str("E=");

        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 1, b'='),
            decode(&s).unwrap_err()
        );

        // more padding doesn't change the error
        s.push_str("=");
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 1, b'='),
            decode(&s).unwrap_err()
        );

        s.push_str("=");
        assert_eq!(
            DecodeError::InvalidByte(num_quads * 4 + 1, b'='),
            decode(&s).unwrap_err()
        );
    }
}

#[test]
fn decode_one_char_in_quad_without_padding_error() {
    for num_quads in 0..25 {
        let mut s: String = std::iter::repeat("ABCD").take(num_quads).collect();
        s.push('E');

        assert_eq!(DecodeError::InvalidLength, decode(&s).unwrap_err());
    }
}

#[test]
fn decode_reject_invalid_bytes_with_correct_error() {
    for length in 1..100 {
        for index in 0_usize..length {
            for invalid_byte in " \t\n\r\x0C\x0B\x00%*.".bytes() {
                let prefix: String = std::iter::repeat("A").take(index).collect();
                let suffix: String = std::iter::repeat("B").take(length - index - 1).collect();

                let input = prefix + &String::from_utf8(vec![invalid_byte]).unwrap() + &suffix;
                assert_eq!(
                    length,
                    input.len(),
                    "length {} error position {}",
                    length,
                    index
                );

                if length % 4 == 1 && !suffix.is_empty() {
                    assert_eq!(DecodeError::InvalidLength, decode(&input).unwrap_err());
                } else {
                    assert_eq!(
                        DecodeError::InvalidByte(index, invalid_byte),
                        decode(&input).unwrap_err()
                    );
                }
            }
        }
    }
}

#[test]
fn decode_imap() {
    assert_eq!(
        decode_config(b"+,,+", crate::IMAP_MUTF7),
        decode_config(b"+//+", crate::STANDARD_NO_PAD)
    );
}

#[test]
fn decode_invalid_trailing_bytes() {
    // The case of trailing newlines is common enough to warrant a test for a good error
    // message.
    assert_eq!(
        Err(DecodeError::InvalidByte(8, b'\n')),
        decode(b"Zm9vCg==\n")
    );
    // extra padding, however, is still InvalidLength
    assert_eq!(Err(DecodeError::InvalidLength), decode(b"Zm9vCg==="));
}

fn config_std_pad() -> Config {
    Config::new(CharacterSet::Standard, true)
}
