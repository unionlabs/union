/* Copyright 2016 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Test that methods gives the correct error.
//! Some also test a bit more because it's easy.

extern crate core;
use core::char;
extern crate encode_unicode;
use encode_unicode::*;
use encode_unicode::error::*;
use encode_unicode::error::InvalidUtf8Array as a;
use encode_unicode::error::InvalidUtf8Slice as s;
use encode_unicode::error::InvalidCodepoint::*;
use encode_unicode::error::InvalidUtf8::*;
use encode_unicode::error::InvalidUtf8FirstByte::*;


#[test]
fn from_u32() {
    for c in 0xd800..0xe000 {
        assert_eq!(char::from_u32_detailed(c),  Err(Utf16Reserved));
    }
    let mut c = 0x11_00_00;
    loop {
        assert_eq!(char::from_u32_detailed(c),  Err(TooHigh));
        // Don't test every value. (Range.step_by() is unstable)
        match c.checked_add(0x10_11_11) {
            Some(next) => c = next,
            None => break,
        }
    }
}

#[test]
fn utf8_extra_bytes() {
    for c in 0..256 {
        assert_eq!( (c as u8).extra_utf8_bytes(), match c {
            0b_1000_0000...0b_1011_1111 => Err(ContinuationByte),
            0b_1111_1000...0b_1111_1111 => Err(TooLongSeqence),
            0b_0000_0000...0b_0111_1111 => Ok(0),
            0b_1100_0000...0b_1101_1111 => Ok(1),
            0b_1110_0000...0b_1110_1111 => Ok(2),
            0b_1111_0000...0b_1111_0111 => Ok(3),
                         _              => unreachable!(),
        });
    }
}

#[test]
fn utf16_extra_unit() {
    for c in 0..0x1_00_00 {
        assert_eq!( (c as u16).utf16_needs_extra_unit(), match c {
            0b_0000_0000_0000_0000...0b_1101_0111_1111_1111 => Ok(false),
            0b_1101_1000_0000_0000...0b_1101_1011_1111_1111 => Ok(true),
            0b_1101_1100_0000_0000...0b_1101_1111_1111_1111 => Err(InvalidUtf16FirstUnit),
            0b_1110_0000_0000_0000...0b_1111_1111_1111_1111 => Ok(false),
                                   _                        => unreachable!(),
        });
    }
}


#[test]
fn from_utf16_tuple() {
    use encode_unicode::error::InvalidUtf16Tuple::*;
    for u in 0xdc00..0xe000 {
        let close = if u%3==0 {u-100} else {u+100};
        let doesnt_matter = if u%2==0 {Some(close)} else {None};
        assert_eq!(char::from_utf16_tuple((u,doesnt_matter)), Err(FirstIsTrailingSurrogate));
    }
    for u in (0..0xd800).chain(0xe000..0x10000) {
        assert_eq!(
            char::from_utf16_tuple((u as u16, Some((0x100+u) as u16))),
            Err(SuperfluousSecond)
        );
    }
    for u in 0xd800..0xdc00 {
        assert_eq!(char::from_utf16_tuple((u,None)), Err(MissingSecond));
        assert_eq!(char::from_utf16_tuple((u,Some(u - 0x2ff))), Err(InvalidSecond));
    }
}

#[test]
fn from_utf16_slice_start() {
    use encode_unicode::error::InvalidUtf16Slice::*;
    assert_eq!(char::from_utf16_slice_start(&[]), Err(EmptySlice));
    let mut buf = [0; 6];
    for u in 0xd800..0xdc00 {
        buf[0] = u;
        assert_eq!(char::from_utf16_slice_start(&buf[..1]), Err(MissingSecond));
        buf[1] = u;
        let pass = 2 + (u as usize % (buf.len()-2));
        assert_eq!(char::from_utf16_slice_start(&buf[..pass]), Err(SecondNotLowSurrogate));
    }
    for u in 0xdc00..0xe000 {
        buf[0] = u;
        let close = if u%3==0 {u-100} else {u+100};
        let pass = 1 + (u as usize % (buf.len()-1));
        buf[pass] = close;
        assert_eq!(char::from_utf16_slice_start(&buf[..pass]), Err(FirstLowSurrogate));
    }
}

#[test]
fn utf8_overlong() {
    let overlongs = [
        [0xf0,0x8f], [0xf0,0x87], [0xf0,0x80], // 4-byte
        [0xe0,0x9f], [0xe0,0x8f], [0xe0,0x80], // 3-byte
        [0xc1,0xbf], [0xc1,0x92], [0xc1,0x80], // 2-byte
        [0xc0,0xbf], [0xc0,0x9f], [0xc0,0x80], // 2-byte
    ];
    for o in overlongs.iter() {
        for &last in &[0x80, 0xbf] {
            let arr = [o[0], o[1], last, last];
            assert_eq!(char::from_utf8_slice_start(&arr), Err(InvalidUtf8Slice::Utf8(OverLong)));
            assert_eq!(char::from_utf8_array(arr), Err(InvalidUtf8Array::Utf8(OverLong)));
            assert_eq!(Utf8Char::from_slice_start(&arr), Err(InvalidUtf8Slice::Utf8(OverLong)));
            assert_eq!(Utf8Char::from_array(arr), Err(InvalidUtf8Array::Utf8(OverLong)));
        }
    }
}

#[test]
fn from_str_start() {
    assert_eq!(Utf8Char::from_str_start(""), Err(EmptyStrError));
    assert_eq!(Utf16Char::from_str_start(""), Err(EmptyStrError));
}

#[test] fn utf8_codepoint_is_too_high() {
    assert_eq!(Utf8Char::from_array([0xf4, 0x90, 0x80, 0x80]), Err(a::Codepoint(TooHigh)));
    assert_eq!(char::from_utf8_array([0xf4, 0x90, 0x80, 0x80]), Err(a::Codepoint(TooHigh)));
    assert_eq!(Utf8Char::from_slice_start(&[0xf4, 0x90, 0x80, 0x80]), Err(s::Codepoint(TooHigh)));
    assert_eq!(char::from_utf8_slice_start(&[0xf4, 0x90, 0x80, 0x80]), Err(s::Codepoint(TooHigh)));

    assert_eq!(Utf8Char::from_array([0xf5, 0x88, 0x99, 0xaa]), Err(a::Codepoint(TooHigh)));
    assert_eq!(char::from_utf8_array([0xf5, 0xaa, 0xbb, 0x88]), Err(a::Codepoint(TooHigh)));
    assert_eq!(Utf8Char::from_slice_start(&[0xf5, 0x99, 0xaa, 0xbb]), Err(s::Codepoint(TooHigh)));
    assert_eq!(char::from_utf8_slice_start(&[0xf5, 0xbb, 0x88, 0x99]), Err(s::Codepoint(TooHigh)));
}

#[test] fn utf8_codepoint_is_utf16_reserved() {
    assert_eq!(Utf8Char::from_array([0xed, 0xa0, 0x80, 0xff]), Err(a::Codepoint(Utf16Reserved)));
    assert_eq!(char::from_utf8_array([0xed, 0xa0, 0x8f, 0x00]), Err(a::Codepoint(Utf16Reserved)));
    assert_eq!(Utf8Char::from_slice_start(&[0xed, 0xa0, 0xbe, 0xa5]), Err(s::Codepoint(Utf16Reserved)));
    assert_eq!(char::from_utf8_slice_start(&[0xed, 0xa0, 0xbf]), Err(s::Codepoint(Utf16Reserved)));
    assert_eq!(Utf8Char::from_array([0xed, 0xbf, 0x80, 0xff]), Err(a::Codepoint(Utf16Reserved)));
    assert_eq!(char::from_utf8_array([0xed, 0xbf, 0x8f, 0x00]), Err(a::Codepoint(Utf16Reserved)));
    assert_eq!(Utf8Char::from_slice_start(&[0xed, 0xbf, 0xbe, 0xa5]), Err(s::Codepoint(Utf16Reserved)));
    assert_eq!(char::from_utf8_slice_start(&[0xed, 0xbf, 0xbf]), Err(s::Codepoint(Utf16Reserved)));
}

#[test] fn utf8_first_is_continuation_byte() {
    for first in 0x80..0xc0 {
        let arr = [first, first<<2, first<<4, first<<6];
        assert_eq!(Utf8Char::from_array(arr), Err(a::Utf8(FirstByte(ContinuationByte))));
        assert_eq!(char::from_utf8_array(arr), Err(a::Utf8(FirstByte(ContinuationByte))));
        let len = (1 + first%3) as usize;
        assert_eq!(Utf8Char::from_slice_start(&arr[..len]), Err(s::Utf8(FirstByte(ContinuationByte))));
        assert_eq!(char::from_utf8_slice_start(&arr[..len]), Err(s::Utf8(FirstByte(ContinuationByte))));
    }
}

#[test] fn utf8_too_long() {
    for first in 0xf8..0x100 {
        let arr = [first as u8, 0x88, 0x80, 0x80];
        assert_eq!(Utf8Char::from_array(arr), Err(a::Utf8(FirstByte(TooLongSeqence))));
        assert_eq!(char::from_utf8_array(arr), Err(a::Utf8(FirstByte(TooLongSeqence))));
        let arr = [first as u8, 0x88, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let slice = &arr[..if first&1 == 0 {1} else {8}];
        assert_eq!(Utf8Char::from_slice_start(slice), Err(s::Utf8(FirstByte(TooLongSeqence))));
        assert_eq!(char::from_utf8_slice_start(slice), Err(s::Utf8(FirstByte(TooLongSeqence))));
    }
}

#[test] fn utf8_not_continuation_byte() {
    for first in 0xc2..0xf4 {
        let mut arr = [first, 0x90, 0xa0, 0xb0];
        let extra = first.extra_utf8_bytes().unwrap();
        for corrupt in (1..extra).rev() {
            let expected = NotAContinuationByte(corrupt);
            for &bad in &[0x00, 0x3f,  0x40, 0x7f,  0xc0, 0xff] {
                arr[corrupt] = bad;
                assert_eq!(Utf8Char::from_array(arr), Err(a::Utf8(expected)), "{:?}", arr);
                assert_eq!(char::from_utf8_array(arr), Err(a::Utf8(expected)));
                let slice = if first&1 == 0 {&arr[..1+extra]} else {&arr};
                assert_eq!(Utf8Char::from_slice_start(slice), Err(s::Utf8(expected)), "{:?}", slice);
                assert_eq!(char::from_utf8_slice_start(slice), Err(s::Utf8(expected)));
            }
        }
    }
}
