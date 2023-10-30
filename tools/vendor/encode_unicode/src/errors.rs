/* Copyright 2016 The encode_unicode Developers
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */


//! Boilerplatey error types

extern crate core;
use self::core::fmt::{self,Display,Formatter};
#[cfg(feature="std")]
use std::error::Error;


macro_rules! description {($err:ty, $desc:expr) => {
    #[cfg(not(feature="std"))]
    impl $err {
        #[allow(missing_docs)]
        pub fn description(&self) -> &'static str {
            ($desc)(self)
        }
    }
    #[cfg(feature="std")]
    impl Error for $err {
        fn description(&self) -> &'static str {
            ($desc)(self)
        }
    }
    impl Display for $err {
        fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
            write!(fmtr, "{}", self.description())
        }
    }
}}


macro_rules! single_cause {(#[$doc1:meta] #[$doc2:meta] $err:ident => $desc:expr) => {
    // Rust 1.15 doesn't understand $(#[$doc:meta])* $:ident
    #[$doc1]
    #[$doc2]
    #[derive(Clone,Copy, Debug, PartialEq,Eq)]
    pub struct $err;
    description!{$err, |_| $desc }
}}


single_cause!{
    /// Cannot tell whether an `u16` needs an extra unit,
    /// because it's a trailing surrogate itself.
    InvalidUtf16FirstUnit => "is a trailing surrogate"
}

single_cause!{
    /// Cannot create an `Utf8Char` or `Utf16Char` from the first codepoint of a str,
    /// because there are none.
    EmptyStrError => "is empty"
}

single_cause!{
    /// Cannot create an `Utf8Char` from a standalone `u8`
    /// that is not an ASCII character.
    NonAsciiError => "is not an ASCII character"
}

single_cause!{
    /// Cannot create an `Utf16Char` from a standalone `u16` that is not a
    /// codepoint in the basic multilingual plane, but part of a suurrogate pair.
    NonBMPError => "is not a codepoint in the basic multilingual plane"
}



macro_rules! simple {(#[$tydoc:meta] $err:ident  {
                          $($(#[$vardoc:meta])* ::$variant:ident => $string:expr),+,
                      } ) => {
    #[$tydoc]
    #[derive(Clone,Copy, Debug, PartialEq,Eq)]
    pub enum $err {
        $($(#[$vardoc])* $variant),*
    }
    description!{$err, |e: &$err| match *e {$($err::$variant=>$string),*} }
}}


simple!{/// Reasons why an `u32` is not a valid UTF codepoint.
    InvalidCodepoint {
        /// It's reserved for UTF-16 surrogate pairs."
        ::Utf16Reserved => "is reserved for UTF-16 surrogate pairs",
        /// It's higher than the highest codepoint (which is 0x10ffff).
        ::TooHigh => "is higher than the highest codepoint",
    }}
use self::InvalidCodepoint::*;
impl InvalidCodepoint {
    /// Get the range of values for which this error would be given.
    pub fn error_range(self) -> (u32,u32) {match self {
        Utf16Reserved => (0xd8_00, 0xdf_ff),
        TooHigh => (0x00_10_ff_ff, 0xff_ff_ff_ff),
    }}
}


simple!{/// Reasons why a `[u16; 2]` doesn't form a valid UTF-16 codepoint.
    InvalidUtf16Array {
        /// The first unit is a trailing/low surrogate, which is never valid.
        ::FirstIsTrailingSurrogate => "the first unit is a trailing surrogate, which is never valid",
        /// The second unit is needed, but is not a trailing surrogate.
        ::SecondIsNotTrailingSurrogate => "the second unit is needed but is not a trailing surrogate",
    }}

simple!{/// Reasons why one or two `u16`s are not valid UTF-16, in sinking precedence.
    InvalidUtf16Tuple {
        /// The first unit is a trailing/low surrogate, which is never valid.
        ///
        /// Note that the value of a low surrogate is actually higher than a high surrogate.
        ::FirstIsTrailingSurrogate => "the first unit is a trailing / low surrogate, which is never valid",
        /// You provided a second unit, but the first one stands on its own.
        ::SuperfluousSecond => "the second unit is superfluous",
        /// The first and only unit requires a second unit.
        ::MissingSecond => "the first unit requires a second unit",
        /// The first unit requires a second unit, but it's not a trailing/low surrogate.
        ///
        /// Note that the value of a low surrogate is actually higher than a high surrogate.
        ::InvalidSecond => "the required second unit is not a trailing / low surrogate",
    }}


simple!{/// Reasons why a slice of `u16`s doesn't start with valid UTF-16.
    InvalidUtf16Slice {
        /// The slice is empty.
        ::EmptySlice => "the slice is empty",
        /// The first unit is a low surrogate.
        ::FirstLowSurrogate => "the first unit is a trailing surrogate",
        /// The first and only unit requires a second unit.
        ::MissingSecond => "the first and only unit requires a second one",
        /// The first unit requires a second one, but it's not a trailing surrogate.
        ::SecondNotLowSurrogate => "the required second unit is not a trailing surrogate",
    }}

simple!{/// Types of invalid sequences encountered by `Utf16CharParser`.
    Utf16PairError {
        /// A trailing surrogate was not preceeded by a leading surrogate.
        ::UnexpectedTrailingSurrogate => "a trailing surrogate was not preceeded by a leading surrogate",
        /// A leading surrogate was followed by an unit that was not a trailing surrogate.
        ::UnmatchedLeadingSurrogate => "a leading surrogate was followed by an unit that was not a trailing surrogate",
        /// A trailing surrogate was expected when the end was reached.
        ::Incomplete => "a trailing surrogate was expected when the end was reached",
    }}


simple!{/// Reasons why `Utf8Char::from_str()` or `Utf16Char::from_str()` failed.
    FromStrError {
        /// `Utf8Char` or `Utf16Char` cannot store more than a single codepoint.
        ::MultipleCodepoints => "has more than one codepoint",
        /// `Utf8Char` or `Utf16Char` cannot be empty.
        ::Empty => "is empty",
    }}


simple!{/// Reasons why a byte is not the start of a UTF-8 codepoint.
    InvalidUtf8FirstByte {
        /// Sequences cannot be longer than 4 bytes. Is given for values >= 240.
        ::TooLongSeqence => "is greater than 247 (UTF-8 sequences cannot be longer than four bytes)",
        /// This byte belongs to a previous sequence. Is given for values between 128 and 192 (exclusive).
        ::ContinuationByte => "is a continuation of a previous sequence",
    }}
use self::InvalidUtf8FirstByte::*;



macro_rules! complex {
($err:ty
 {$($sub:ty => $to:expr,)*}
 {$($desc:pat => $string:expr),+,}
 => $use_cause:expr =>
 {$($cause:pat => $result:expr),+,} $(#[$causedoc:meta])*
) => {
    $(impl From<$sub> for $err {
          fn from(error: $sub) -> $err {
              $to(error)
          }
      })*
    #[cfg(not(feature="std"))]
    impl $err {
        #[allow(missing_docs)]
        pub fn description(&self) -> &'static str {
            match *self{ $($desc => $string,)* }
        }
        /// A hack to avoid two Display impls
        fn cause(&self) -> Option<&Display> {None}
    }
    #[cfg(feature="std")]
    impl Error for $err {
        fn description(&self) -> &'static str {
            match *self{ $($desc => $string,)* }
        }
        $(#[$causedoc])*
        fn cause(&self) -> Option<&Error> {
            match *self{ $($cause => $result,)* }
        }
    }
    impl Display for $err {
        fn fmt(&self,  fmtr: &mut Formatter) -> fmt::Result {
            match (self.cause(), $use_cause) {
                (Some(d),true) => write!(fmtr, "{}: {}", self.description(), d),
                        _      => write!(fmtr, "{}", self.description()),
            }
        }
    }
}}


/// Reasons why a byte sequence is not valid UTF-8, excluding invalid codepoint.
/// In sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8 {
    /// Something is wrong with the first byte.
    FirstByte(InvalidUtf8FirstByte),
    /// The byte at index 1...3 should be a continuation byte,
    /// but dosesn't fit the pattern 0b10xx_xxxx.
    NotAContinuationByte(usize),
    /// There are too many leading zeros: it could be a byte shorter.
    ///
    /// [Decoding this could allow someone to input otherwise prohibited
    /// characters and sequences, such as "../"](https://tools.ietf.org/html/rfc3629#section-10).
    OverLong,
}
use self::InvalidUtf8::*;
complex!{InvalidUtf8 {
        InvalidUtf8FirstByte => FirstByte,
    } {
        FirstByte(TooLongSeqence) => "the first byte is greater than 239 (UTF-8 sequences cannot be longer than four bytes)",
        FirstByte(ContinuationByte) => "the first byte is a continuation of a previous sequence",
        OverLong => "the sequence contains too many zeros and could be shorter",
        NotAContinuationByte(_) => "the sequence is too short",
    } => false => {
        FirstByte(ref cause) => Some(cause),
        _ => None,
    }/// Returns `Some` if the error is a `InvalidUtf8FirstByte`.
}


/// Reasons why a byte array is not valid UTF-8, in sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8Array {
    /// Not a valid UTF-8 sequence.
    Utf8(InvalidUtf8),
    /// Not a valid unicode codepoint.
    Codepoint(InvalidCodepoint),
}
complex!{InvalidUtf8Array {
        InvalidUtf8 => InvalidUtf8Array::Utf8,
        InvalidCodepoint => InvalidUtf8Array::Codepoint,
    } {
        InvalidUtf8Array::Utf8(_) => "the sequence is invalid UTF-8",
        InvalidUtf8Array::Codepoint(_) => "the encoded codepoint is invalid",
    } => true => {
        InvalidUtf8Array::Utf8(ref u) => Some(u),
        InvalidUtf8Array::Codepoint(ref c) => Some(c),
    }/// Always returns `Some`.
}


/// Reasons why a byte slice is not valid UTF-8, in sinking precedence.
#[derive(Clone,Copy, Debug, PartialEq,Eq)]
pub enum InvalidUtf8Slice {
    /// Something is certainly wrong with the first byte.
    Utf8(InvalidUtf8),
    /// The encoded codepoint is invalid:
    Codepoint(InvalidCodepoint),
    /// The slice is too short; n bytes was required.
    TooShort(usize),
}
complex!{InvalidUtf8Slice {
        InvalidUtf8 => InvalidUtf8Slice::Utf8,
        InvalidCodepoint => InvalidUtf8Slice::Codepoint,
    } {
        InvalidUtf8Slice::Utf8(_) => "the sequence is invalid UTF-8",
        InvalidUtf8Slice::Codepoint(_) => "the encoded codepoint is invalid",
        InvalidUtf8Slice::TooShort(1) => "the slice is empty",
        InvalidUtf8Slice::TooShort(_) => "the slice is shorter than the sequence",
    } => true => {
        InvalidUtf8Slice::Utf8(ref u) => Some(u),
        InvalidUtf8Slice::Codepoint(ref c) => Some(c),
        InvalidUtf8Slice::TooShort(_) => None,
    }
}
