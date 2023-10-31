/* Copyright 2016 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use errors::{FromStrError, EmptyStrError, NonAsciiError, InvalidUtf8Slice, InvalidUtf8Array};
use utf8_iterators::Utf8Iterator;
use traits::{CharExt, U8UtfExt};
use utf16_char::Utf16Char;
extern crate core;
use self::core::{hash, fmt, str, ptr};
use self::core::cmp::Ordering;
use self::core::borrow::Borrow;
use self::core::ops::Deref;
use self::core::mem::transmute;
#[cfg(feature="std")]
use self::core::iter::FromIterator;
#[cfg(feature="std")]
#[allow(deprecated)]
use std::ascii::AsciiExt;
#[cfg(feature="ascii")]
extern crate ascii;
#[cfg(feature="ascii")]
use self::ascii::{AsciiChar,ToAsciiChar,ToAsciiCharError};


// I don't think there is any good default value for char, but char does.
#[derive(Default)]
// char doesn't do anything more advanced than u32 for Eq/Ord, so we shouldn't either.
// The default impl of Ord for arrays works out because longer codepoints
//     start with more ones, so if they're equal, the length is the same,
// breaks down for values above 0x1f_ff_ff but those can only be created by unsafe code.
#[derive(PartialEq,Eq, PartialOrd,Ord)]

#[derive(Clone,Copy)]


/// An unicode codepoint stored as UTF-8.
///
/// It can be borrowed as a `str`, and has the same size as `char`.
pub struct Utf8Char {
    bytes: [u8; 4],
}


  /////////////////////
 //conversion traits//
/////////////////////
impl str::FromStr for Utf8Char {
    type Err = FromStrError;
    /// Create an `Utf8Char` from a string slice.
    /// The string must contain exactly one codepoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::error::FromStrError::*;
    /// use encode_unicode::Utf8Char;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(Utf8Char::from_str("a"), Ok(Utf8Char::from('a')));
    /// assert_eq!(Utf8Char::from_str("🂠"), Ok(Utf8Char::from('🂠')));
    /// assert_eq!(Utf8Char::from_str(""), Err(Empty));
    /// assert_eq!(Utf8Char::from_str("ab"), Err(MultipleCodepoints));
    /// assert_eq!(Utf8Char::from_str("é"), Err(MultipleCodepoints));// 'e'+u301 combining mark
    /// ```
    fn from_str(s: &str) -> Result<Self, FromStrError> {
        if s.is_empty() {
            Err(FromStrError::Empty)
        } else if s.len() != 1+s.as_bytes()[0].extra_utf8_bytes_unchecked() {
            Err(FromStrError::MultipleCodepoints)
        } else {
            let mut bytes = [0; 4];
            bytes[..s.len()].copy_from_slice(s.as_bytes());
            Ok(Utf8Char{bytes: bytes})
        }
    }
}
impl From<Utf16Char> for Utf8Char {
    fn from(utf16: Utf16Char) -> Utf8Char {
        match utf16.to_tuple() {
            (a @ 0...0x00_7f, _) => {
                Utf8Char{ bytes: [a as u8, 0, 0, 0] }
            },
            (u @ 0...0x07_ff, _) => {
                let b = 0x80 |  (u & 0x00_3f) as u8;
                let a = 0xc0 | ((u & 0x07_c0) >> 6) as u8;
                Utf8Char{ bytes: [a, b, 0, 0] }
            },
            (u, None) => {
                let c = 0x80 |  (u & 0x00_3f) as u8;
                let b = 0x80 | ((u & 0x0f_c0) >> 6) as u8;
                let a = 0xe0 | ((u & 0xf0_00) >> 12) as u8;
                Utf8Char{ bytes: [a, b, c, 0] }
            },
            (f, Some(s)) => {
                let f = f + (0x01_00_00u32 >> 10) as u16;
                let d = 0x80 |  (s & 0x00_3f) as u8;
                let c = 0x80 | ((s & 0x03_c0) >> 6) as u8
                             | ((f & 0x00_03) << 4) as u8;
                let b = 0x80 | ((f & 0x00_fc) >> 2) as u8;
                let a = 0xf0 | ((f & 0x07_00) >> 8) as u8;
                Utf8Char{ bytes: [a, b, c, d] }
            }
        }
    }
}
impl From<char> for Utf8Char {
    fn from(c: char) -> Self {
        Utf8Char{ bytes: c.to_utf8_array().0 }
    }
}
impl From<Utf8Char> for char {
    fn from(uc: Utf8Char) -> char {
        unsafe{ char::from_utf8_exact_slice_unchecked(&uc.bytes[..uc.len()]) }
    }
}
impl IntoIterator for Utf8Char {
    type Item=u8;
    type IntoIter=Utf8Iterator;
    /// Iterate over the byte values.
    fn into_iter(self) -> Utf8Iterator {
        Utf8Iterator::from(self)
    }
}

#[cfg(feature="std")]
impl Extend<Utf8Char> for Vec<u8> {
    fn extend<I:IntoIterator<Item=Utf8Char>>(&mut self,  iter: I) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for u8c in iter {
            // twice as fast as self.extend_from_slice(u8c.as_bytes());
            self.push(u8c.bytes[0]);
            for &extra in &u8c.bytes[1..] {
                if extra != 0 {
                    self.push(extra);
                }
            }
        }
    }
}
#[cfg(feature="std")]
impl<'a> Extend<&'a Utf8Char> for Vec<u8> {
    fn extend<I:IntoIterator<Item=&'a Utf8Char>>(&mut self,  iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}
#[cfg(feature="std")]
impl Extend<Utf8Char> for String {
    fn extend<I:IntoIterator<Item=Utf8Char>>(&mut self,  iter: I) {
        unsafe { self.as_mut_vec().extend(iter) }
    }
}
#[cfg(feature="std")]
impl<'a> Extend<&'a Utf8Char> for String {
    fn extend<I:IntoIterator<Item=&'a Utf8Char>>(&mut self,  iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf8Char> for String {
    fn from_iter<I:IntoIterator<Item=Utf8Char>>(iter: I) -> String {
        let mut string = String::new();
        string.extend(iter);
        return string;
    }
}
#[cfg(feature="std")]
impl<'a> FromIterator<&'a Utf8Char> for String {
    fn from_iter<I:IntoIterator<Item=&'a Utf8Char>>(iter: I) -> String {
        iter.into_iter().cloned().collect()
    }
}
#[cfg(feature="std")]
impl FromIterator<Utf8Char> for Vec<u8> {
    fn from_iter<I:IntoIterator<Item=Utf8Char>>(iter: I) -> Self {
        iter.into_iter().collect::<String>().into_bytes()
    }
}
#[cfg(feature="std")]
impl<'a> FromIterator<&'a Utf8Char> for Vec<u8> {
    fn from_iter<I:IntoIterator<Item=&'a Utf8Char>>(iter: I) -> Self {
        iter.into_iter().cloned().collect::<String>().into_bytes()
    }
}


  /////////////////
 //getter traits//
/////////////////
impl AsRef<[u8]> for Utf8Char {
    fn as_ref(&self) -> &[u8] {
        &self.bytes[..self.len()]
    }
}
impl AsRef<str> for Utf8Char {
    fn as_ref(&self) -> &str {
        unsafe{ str::from_utf8_unchecked( self.as_ref() ) }
    }
}
impl Borrow<[u8]> for Utf8Char {
    fn borrow(&self) -> &[u8] {
        self.as_ref()
    }
}
impl Borrow<str> for Utf8Char {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}
impl Deref for Utf8Char {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}


  ////////////////
 //ascii traits//
////////////////
#[cfg(feature="std")]
#[allow(deprecated)]
impl AsciiExt for Utf8Char {
    type Owned = Utf8Char;
    fn is_ascii(&self) -> bool {
        self.bytes[0].is_ascii()
    }
    fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        if self.is_ascii() {self.bytes[0].eq_ignore_ascii_case(&other.bytes[0])}
        else               {self == other}
    }
    fn to_ascii_uppercase(&self) -> Self::Owned {
        let mut uc = *self;
        uc.make_ascii_uppercase();
        uc
    }
    fn to_ascii_lowercase(&self) -> Self::Owned {
        let mut uc = *self;
        uc.make_ascii_lowercase();
        uc
    }
    fn make_ascii_uppercase(&mut self) {
        self.bytes[0].make_ascii_uppercase()
    }
    fn make_ascii_lowercase(&mut self) {
        self.bytes[0].make_ascii_lowercase();
    }
}

#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl From<AsciiChar> for Utf8Char {
    fn from(ac: AsciiChar) -> Self {
        Utf8Char{ bytes: [ac.as_byte(),0,0,0] }
    }
}
#[cfg(feature="ascii")]
/// Requires the feature "ascii".
impl ToAsciiChar for Utf8Char {
    fn to_ascii_char(self) -> Result<AsciiChar, ToAsciiCharError> {
        self.bytes[0].to_ascii_char()
    }
    unsafe fn to_ascii_char_unchecked(self) -> AsciiChar {
        self.bytes[0].to_ascii_char_unchecked()
    }
}


  /////////////////////////////////////////////////////////
 //Genaral traits that cannot be derived to emulate char//
/////////////////////////////////////////////////////////
impl hash::Hash for Utf8Char {
    fn hash<H : hash::Hasher>(&self,  state: &mut H) {
        self.to_char().hash(state);
    }
}
impl fmt::Debug for Utf8Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_char(), fmtr)
    }
}
impl fmt::Display for Utf8Char {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        fmtr.write_str(self.as_str())
    }
}


  ////////////////////////////////
 //Comparisons with other types//
////////////////////////////////
impl PartialEq<char> for Utf8Char {
    fn eq(&self,  u32c: &char) -> bool {
        *self == Utf8Char::from(*u32c)
    }
}
impl PartialEq<Utf8Char> for char {
    fn eq(&self,  u8c: &Utf8Char) -> bool {
        Utf8Char::from(*self) == *u8c
    }
}
impl PartialOrd<char> for Utf8Char {
    fn partial_cmp(&self,  u32c: &char) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*u32c))
    }
}
impl PartialOrd<Utf8Char> for char {
    fn partial_cmp(&self,  u8c: &Utf8Char) -> Option<Ordering> {
        Utf8Char::from(*self).partial_cmp(u8c)
    }
}

impl PartialEq<Utf16Char> for Utf8Char {
    fn eq(&self,  u16c: &Utf16Char) -> bool {
        *self == Self::from(*u16c)
    }
}
impl PartialOrd<Utf16Char> for Utf8Char {
    fn partial_cmp(&self,  u16c: &Utf16Char) -> Option<Ordering> {
        self.partial_cmp(&Self::from(*u16c))
    }
}
// The other direction is implemented in utf16_char.rs

/// Only considers the byte equal if both it and the `Utf8Char` represents ASCII characters.
///
/// There is no impl in the opposite direction, as this should only be used to
/// compare `Utf8Char`s against constants.
///
/// # Examples
///
/// ```
/// # use encode_unicode::Utf8Char;
/// assert!(Utf8Char::from('8') == b'8');
/// assert!(Utf8Char::from_array([0xf1,0x80,0x80,0x80]).unwrap() != 0xf1);
/// assert!(Utf8Char::from('\u{ff}') != 0xff);
/// assert!(Utf8Char::from('\u{80}') != 0x80);
/// ```
impl PartialEq<u8> for Utf8Char {
    fn eq(&self,  byte: &u8) -> bool {
        self.bytes[0] == *byte  &&  self.bytes[1] == 0
    }
}
#[cfg(feature = "ascii")]
/// `Utf8Char`s that are not ASCII never compare equal.
impl PartialEq<AsciiChar> for Utf8Char {
    #[inline]
    fn eq(&self,  ascii: &AsciiChar) -> bool {
        self.bytes[0] == *ascii as u8
    }
}
#[cfg(feature = "ascii")]
/// `Utf8Char`s that are not ASCII never compare equal.
impl PartialEq<Utf8Char> for AsciiChar {
    #[inline]
    fn eq(&self,  u8c: &Utf8Char) -> bool {
        u8c == self
    }
}
#[cfg(feature = "ascii")]
/// `Utf8Char`s that are not ASCII always compare greater.
impl PartialOrd<AsciiChar> for Utf8Char {
    #[inline]
    fn partial_cmp(&self,  ascii: &AsciiChar) -> Option<Ordering> {
        self.bytes[0].partial_cmp(ascii)
    }
}
#[cfg(feature = "ascii")]
/// `Utf8Char`s that are not ASCII always compare greater.
impl PartialOrd<Utf8Char> for AsciiChar {
    #[inline]
    fn partial_cmp(&self,  u8c: &Utf8Char) -> Option<Ordering> {
        self.partial_cmp(&u8c.bytes[0])
    }
}


  ///////////////////////////////////////////////////////
 //pub impls that should be together for nicer rustdoc//
///////////////////////////////////////////////////////
impl Utf8Char {
    /// Create an `Utf8Char` from the first codepoint in a `str`.
    ///
    /// Returns an error if the `str` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::Utf8Char;
    ///
    /// assert_eq!(Utf8Char::from_str_start("a"), Ok((Utf8Char::from('a'),1)));
    /// assert_eq!(Utf8Char::from_str_start("ab"), Ok((Utf8Char::from('a'),1)));
    /// assert_eq!(Utf8Char::from_str_start("🂠 "), Ok((Utf8Char::from('🂠'),4)));
    /// assert_eq!(Utf8Char::from_str_start("é"), Ok((Utf8Char::from('e'),1)));// 'e'+u301 combining mark
    /// assert!(Utf8Char::from_str_start("").is_err());
    /// ```
    pub fn from_str_start(src: &str) -> Result<(Self,usize),EmptyStrError> {
        unsafe {
            if src.is_empty() {
                Err(EmptyStrError)
            } else {
                Ok(Utf8Char::from_slice_start_unchecked(src.as_bytes()))
            }
        }
    }
    /// Create an `Utf8Char` of the first codepoint in an UTF-8 slice.  
    /// Also returns the length of the UTF-8 sequence for the codepoint.
    ///
    /// If the slice is from a `str`, use `::from_str_start()` to skip UTF-8 validation.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the slice is empty, doesn't start with a valid
    /// UTF-8 sequence or is too short for the sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::Utf8Char;
    /// use encode_unicode::error::InvalidUtf8Slice::*;
    /// use encode_unicode::error::InvalidUtf8::*;
    ///
    /// assert_eq!(Utf8Char::from_slice_start(&[b'A', b'B', b'C']), Ok((Utf8Char::from('A'),1)));
    /// assert_eq!(Utf8Char::from_slice_start(&[0xdd, 0xbb]), Ok((Utf8Char::from('\u{77b}'),2)));
    ///
    /// assert_eq!(Utf8Char::from_slice_start(&[]), Err(TooShort(1)));
    /// assert_eq!(Utf8Char::from_slice_start(&[0xf0, 0x99]), Err(TooShort(4)));
    /// assert_eq!(Utf8Char::from_slice_start(&[0xee, b'F', 0x80]), Err(Utf8(NotAContinuationByte(1))));
    /// assert_eq!(Utf8Char::from_slice_start(&[0xee, 0x99, 0x0f]), Err(Utf8(NotAContinuationByte(2))));
    /// ```
    pub fn from_slice_start(src: &[u8]) -> Result<(Self,usize),InvalidUtf8Slice> {
        char::from_utf8_slice_start(src).map(|(_,len)| {
            let mut bytes = [0; 4];
            bytes[..len].copy_from_slice(&src[..len]);
            (Utf8Char{ bytes: bytes }, len)
        })
    }
    /// A `from_slice_start()` that doesn't validate the codepoint.
    ///
    /// # Safety
    ///
    /// The slice must be non-empty and start with a valid UTF-8 codepoint.  
    /// Invalid or incomplete values might cause reads of uninitalized memory.
    pub unsafe fn from_slice_start_unchecked(src: &[u8]) -> (Self,usize) {
        let len = 1+src.get_unchecked(0).extra_utf8_bytes_unchecked();
        let mut bytes = [0; 4];
        ptr::copy_nonoverlapping(src.as_ptr(), &mut bytes[0] as *mut u8, len);
        (Utf8Char{ bytes: bytes }, len)
    }
    /// Create an `Utf8Char` from a byte array after validating it.
    ///
    /// The codepoint must start at the first byte.  
    /// Unused bytes are set to zero by this function and so can be anything.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if the array doesn't start with a valid UTF-8 sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use encode_unicode::Utf8Char;
    /// use encode_unicode::error::InvalidUtf8Array::*;
    /// use encode_unicode::error::InvalidUtf8::*;
    /// use encode_unicode::error::InvalidCodepoint::*;
    ///
    /// assert_eq!(Utf8Char::from_array([b'A', 0, 0, 0]), Ok(Utf8Char::from('A')));
    /// assert_eq!(Utf8Char::from_array([0xf4, 0x8b, 0xbb, 0xbb]), Ok(Utf8Char::from('\u{10befb}')));
    /// assert_eq!(Utf8Char::from_array([b'A', b'B', b'C', b'D']), Ok(Utf8Char::from('A')));
    /// assert_eq!(Utf8Char::from_array([0, 0, 0xcc, 0xbb]), Ok(Utf8Char::from('\0')));
    ///
    /// assert_eq!(Utf8Char::from_array([0xef, b'F', 0x80, 0x80]), Err(Utf8(NotAContinuationByte(1))));
    /// assert_eq!(Utf8Char::from_array([0xc1, 0x80, 0, 0]), Err(Utf8(OverLong)));
    /// assert_eq!(Utf8Char::from_array([0xf7, 0xaa, 0x99, 0x88]), Err(Codepoint(TooHigh)));
    /// ```
    pub fn from_array(utf8: [u8;4]) -> Result<Self,InvalidUtf8Array> {
        unsafe {
            // perform all validation
            try!(char::from_utf8_array(utf8));
            let extra = utf8[0].extra_utf8_bytes_unchecked() as u32;
            // zero unused bytes in one operation by transmuting the arrary to
            // u32, apply an endian-corrected mask and transmute back
            let mask = u32::from_le(0xff_ff_ff_ff >> 8*(3-extra));
            let unused_zeroed = mask  &  transmute::<_,u32>(utf8);
            Ok(Utf8Char{ bytes: transmute(unused_zeroed) })
        }
    }
    /// Zero-cost constructor.
    ///
    /// # Safety
    ///
    /// Must contain a valid codepoint starting at the first byte, with the
    /// unused bytes zeroed.  
    /// Bad values can easily lead to undefined behavior.
    #[inline]
    pub unsafe fn from_array_unchecked(utf8: [u8;4]) -> Self {
        Utf8Char{ bytes: utf8 }
    }
    /// Create an `Utf8Char` from a single byte.
    ///
    /// The byte must be an ASCII character.
    ///
    /// # Errors
    ///
    /// Returns `NonAsciiError` if the byte greater than 127.
    ///
    /// # Examples
    ///
    /// ```
    /// # use encode_unicode::Utf8Char;
    /// assert_eq!(Utf8Char::from_ascii(b'a').unwrap(), 'a');
    /// assert!(Utf8Char::from_ascii(128).is_err());
    /// ```
    pub fn from_ascii(ascii: u8) -> Result<Self,NonAsciiError> {
        if ascii as i8 >= 0 {
            Ok(Utf8Char{ bytes: [ascii, 0, 0, 0] })
        } else {
            Err(NonAsciiError)
        }
    }
    /// Create an `Utf8Char` from a single byte without checking that it's a
    /// valid codepoint on its own, which is only true for ASCII characters.
    ///
    /// # Safety
    ///
    /// The byte must be less than 128.
    #[inline]
    pub unsafe fn from_ascii_unchecked(ascii: u8) -> Self {
        Utf8Char{ bytes: [ascii, 0, 0, 0] }
    }

    /// The number of bytes this character needs.
    ///
    /// Is between 1 and 4 (inclusive) and identical to `.as_ref().len()` or
    /// `.as_char().len_utf8()`.
    #[inline]
    pub fn len(self) -> usize {
        // Invariants of the extra bytes enambles algorithms that
        // `u8.extra_utf8_bytes_unchecked()` cannot use.
        // Some of them turned out to require fewer x86 instructions:

        // Exploits that unused bytes are zero and calculates the number of
        // trailing zero bytes.
        // Setting a bit in the first byte prevents the function from returning
        // 0 for '\0' (which has 32 leading zeros).
        // trailing and leading is swapped below to optimize for little-endian
        // architectures.
        (4 - (u32::to_le(unsafe{transmute(self.bytes)})|1).leading_zeros()/8) as usize

        // Exploits that the extra bytes have their most significant bit set if
        // in use.
        // Takes fewer instructions than the one above if popcnt can be used,
        // (which it cannot by default,
        //  set RUSTFLAGS='-C target-cpu=native' to enable)
        //let all: u32 = unsafe{transmute(self.bytes)};
        //let msb_mask = u32::from_be(0x00808080);
        //let add_one = u32::from_be(0x80000000);
        //((all & msb_mask) | add_one).count_ones() as usize
    }
    // There is no .is_emty() because this type is never empty.

    /// Checks that the codepoint is an ASCII character.
    pub fn is_ascii(&self) -> bool {
        self.bytes[0] <= 127
    }
    /// Checks that two characters are an ASCII case-insensitive match.
    ///
    /// Is equivalent to `a.to_ascii_lowercase() == b.to_ascii_lowercase()`.
    #[cfg(feature="std")]
    pub fn eq_ignore_ascii_case(&self,  other: &Self) -> bool {
        if self.is_ascii() {self.bytes[0].eq_ignore_ascii_case(&other.bytes[0])}
        else               {self == other}
    }
    /// Converts the character to its ASCII upper case equivalent.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_uppercase(&self) -> Self {
        let mut uc = *self;
        uc.make_ascii_uppercase();
        uc
    }
    /// Converts the character to its ASCII lower case equivalent.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[cfg(feature="std")]
    pub fn to_ascii_lowercase(&self) -> Self {
        let mut uc = *self;
        uc.make_ascii_lowercase();
        uc
    }
    /// Converts the character to its ASCII upper case equivalent in-place.
    ///
    /// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z',
    /// but non-ASCII letters are unchanged.
    #[inline]
    #[cfg(feature="std")]
    pub fn make_ascii_uppercase(&mut self) {
        self.bytes[0].make_ascii_uppercase()
    }
    /// Converts the character to its ASCII lower case equivalent in-place.
    ///
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// but non-ASCII letters are unchanged.
    #[inline]
    #[cfg(feature="std")]
    pub fn make_ascii_lowercase(&mut self) {
        self.bytes[0].make_ascii_lowercase();
    }

    /// Convert from UTF-8 to UTF-32
    pub fn to_char(self) -> char {
        self.into()
    }
    /// Write the internal representation to a slice,
    /// and then returns the number of bytes written.
    ///
    /// # Panics
    ///
    /// Will panic the buffer is too small;
    /// You can get the required length from `.len()`,
    /// but a buffer of length four is always large enough.
    pub fn to_slice(self,  dst: &mut[u8]) -> usize {
        if self.len() > dst.len() {
            panic!("The provided buffer is too small.");
        }
        dst[..self.len()].copy_from_slice(&self.bytes[..self.len()]);
        self.len()
    }
    /// Expose the internal array and the number of used bytes.
    pub fn to_array(self) -> ([u8;4],usize) {
        (self.bytes, self.len())
    }
    /// Return a `str` view of the array the codepoint is stored as.
    ///
    /// Is an unambiguous version of `.as_ref()`.
    pub fn as_str(&self) -> &str {
        self.deref()
    }
}
