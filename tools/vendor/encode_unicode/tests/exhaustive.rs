/* Copyright 2018 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Tests that try all possible values for at least one parameter / byte / unit
//! of the tested function.

use std::char;
extern crate encode_unicode;
use encode_unicode::*;

#[test]
fn from_ascii() {
    for cp in 0u32..256 {
        assert_eq!(Utf8Char::from_ascii(cp as u8).is_ok(), cp & 0x80 == 0);
        if let Ok(u8c) = Utf8Char::from_ascii(cp as u8) {
            assert_eq!(u8c, Utf8Char::from(cp as u8 as char));
        }
    }
}

#[test]
fn from_bmp() {
    for cp in 0u32..0x1_00_00 {
        assert_eq!(
            Utf16Char::from_bmp(cp as u16).ok(),
            char::from_u32(cp).map(|u32c| Utf16Char::from(u32c) )
        );
    }
}
