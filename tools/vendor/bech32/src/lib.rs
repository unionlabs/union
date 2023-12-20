// Copyright (c) 2017 Clark Moody
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

//! Encoding and decoding of the Bech32 format
//!
//! Bech32 is an encoding scheme that is easy to use for humans and efficient to encode in QR codes.
//!
//! A Bech32 string consists of a human-readable part (HRP), a separator (the character `'1'`), and
//! a data part. A checksum at the end of the string provides error detection to prevent mistakes
//! when the string is written off or read out loud.
//!
//! The original description in [BIP-0173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)
//! has more details.
//!
#![cfg_attr(
    feature = "std",
    doc = "
# Examples
```
use bech32::{self, FromBase32, ToBase32, Variant};
let encoded = bech32::encode(\"bech32\", vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();
assert_eq!(encoded, \"bech321qqqsyrhqy2a\".to_string());
let (hrp, data, variant) = bech32::decode(&encoded).unwrap();
assert_eq!(hrp, \"bech32\");
assert_eq!(Vec::<u8>::from_base32(&data).unwrap(), vec![0x00, 0x01, 0x02]);
assert_eq!(variant, Variant::Bech32);
```
"
)]
//!

// Allow trait objects without dyn on nightly and make 1.22 ignore the unknown lint
#![allow(unknown_lints)]
#![allow(bare_trait_objects)]
#![deny(missing_docs)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![cfg_attr(feature = "strict", deny(warnings))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

#[cfg(all(not(feature = "std"), not(test)))]
extern crate alloc;

#[cfg(any(test, feature = "std"))]
extern crate core;

#[cfg(all(not(feature = "std"), not(test)))]
use alloc::{string::String, vec::Vec};

#[cfg(all(not(feature = "std"), not(test)))]
use alloc::borrow::Cow;
#[cfg(any(feature = "std", test))]
use std::borrow::Cow;

use core::{fmt, mem};

/// Integer in the range `0..32`
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub struct u5(u8);

impl u5 {
    /// Convert a `u8` to `u5` if in range, return `Error` otherwise
    pub fn try_from_u8(value: u8) -> Result<u5, Error> {
        if value > 31 {
            Err(Error::InvalidData(value))
        } else {
            Ok(u5(value))
        }
    }

    /// Returns a copy of the underlying `u8` value
    pub fn to_u8(self) -> u8 {
        self.0
    }

    /// Get char representing this 5 bit value as defined in BIP173
    pub fn to_char(self) -> char {
        CHARSET[self.to_u8() as usize]
    }
}

impl From<u5> for u8 {
    fn from(v: u5) -> u8 {
        v.0
    }
}

impl AsRef<u8> for u5 {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

/// Interface to write `u5`s into a sink
pub trait WriteBase32 {
    /// Write error
    type Err: fmt::Debug;

    /// Write a `u5` slice
    fn write(&mut self, data: &[u5]) -> Result<(), Self::Err> {
        for b in data {
            self.write_u5(*b)?;
        }
        Ok(())
    }

    /// Write a single `u5`
    fn write_u5(&mut self, data: u5) -> Result<(), Self::Err>;
}

const CHECKSUM_LENGTH: usize = 6;

/// Allocationless Bech32 writer that accumulates the checksum data internally and writes them out
/// in the end.
pub struct Bech32Writer<'a> {
    formatter: &'a mut fmt::Write,
    chk: u32,
    variant: Variant,
}

impl<'a> Bech32Writer<'a> {
    /// Creates a new writer that can write a bech32 string without allocating itself.
    ///
    /// This is a rather low-level API and doesn't check the HRP or data length for standard
    /// compliance.
    pub fn new(
        hrp: &str,
        variant: Variant,
        fmt: &'a mut fmt::Write,
    ) -> Result<Bech32Writer<'a>, fmt::Error> {
        let mut writer = Bech32Writer {
            formatter: fmt,
            chk: 1,
            variant,
        };

        writer.formatter.write_str(hrp)?;
        writer.formatter.write_char(SEP)?;

        // expand HRP
        for b in hrp.bytes() {
            writer.polymod_step(u5(b >> 5));
        }
        writer.polymod_step(u5(0));
        for b in hrp.bytes() {
            writer.polymod_step(u5(b & 0x1f));
        }

        Ok(writer)
    }

    fn polymod_step(&mut self, v: u5) {
        let b = (self.chk >> 25) as u8;
        self.chk = (self.chk & 0x01ff_ffff) << 5 ^ (u32::from(*v.as_ref()));

        for (i, item) in GEN.iter().enumerate() {
            if (b >> i) & 1 == 1 {
                self.chk ^= item;
            }
        }
    }

    /// Write out the checksum at the end. If this method isn't called this will happen on drop.
    pub fn finalize(mut self) -> fmt::Result {
        self.write_checksum()?;
        mem::forget(self);
        Ok(())
    }

    fn write_checksum(&mut self) -> fmt::Result {
        // Pad with 6 zeros
        for _ in 0..CHECKSUM_LENGTH {
            self.polymod_step(u5(0))
        }

        let plm: u32 = self.chk ^ self.variant.constant();

        for p in 0..CHECKSUM_LENGTH {
            self.formatter
                .write_char(u5(((plm >> (5 * (5 - p))) & 0x1f) as u8).to_char())?;
        }

        Ok(())
    }
}

impl<'a> WriteBase32 for Bech32Writer<'a> {
    type Err = fmt::Error;

    /// Writes a single 5 bit value of the data part
    fn write_u5(&mut self, data: u5) -> fmt::Result {
        self.polymod_step(data);
        self.formatter.write_char(data.to_char())
    }
}

impl<'a> Drop for Bech32Writer<'a> {
    fn drop(&mut self) {
        self.write_checksum()
            .expect("Unhandled error writing the checksum on drop.")
    }
}

/// Parse/convert base32 slice to `Self`. It is the reciprocal of
/// `ToBase32`.
pub trait FromBase32: Sized {
    /// The associated error which can be returned from parsing (e.g. because of bad padding).
    type Err;

    /// Convert a base32 slice to `Self`.
    fn from_base32(b32: &[u5]) -> Result<Self, Self::Err>;
}

impl WriteBase32 for Vec<u5> {
    type Err = ();

    fn write(&mut self, data: &[u5]) -> Result<(), Self::Err> {
        self.extend_from_slice(data);
        Ok(())
    }

    fn write_u5(&mut self, data: u5) -> Result<(), Self::Err> {
        self.push(data);
        Ok(())
    }
}

impl FromBase32 for Vec<u8> {
    type Err = Error;

    /// Convert base32 to base256, removes null-padding if present, returns
    /// `Err(Error::InvalidPadding)` if padding bits are unequal `0`
    fn from_base32(b32: &[u5]) -> Result<Self, Self::Err> {
        convert_bits(b32, 5, 8, false)
    }
}

/// A trait for converting a value to a type `T` that represents a `u5` slice.
pub trait ToBase32 {
    /// Convert `Self` to base32 vector
    fn to_base32(&self) -> Vec<u5> {
        let mut vec = Vec::new();
        self.write_base32(&mut vec).unwrap();
        vec
    }

    /// Encode as base32 and write it to the supplied writer
    /// Implementations shouldn't allocate.
    fn write_base32<W: WriteBase32>(&self, writer: &mut W) -> Result<(), <W as WriteBase32>::Err>;
}

/// Interface to calculate the length of the base32 representation before actually serializing
pub trait Base32Len: ToBase32 {
    /// Calculate the base32 serialized length
    fn base32_len(&self) -> usize;
}

impl<T: AsRef<[u8]>> ToBase32 for T {
    fn write_base32<W: WriteBase32>(&self, writer: &mut W) -> Result<(), <W as WriteBase32>::Err> {
        // Amount of bits left over from last round, stored in buffer.
        let mut buffer_bits = 0u32;
        // Holds all unwritten bits left over from last round. The bits are stored beginning from
        // the most significant bit. E.g. if buffer_bits=3, then the byte with bits a, b and c will
        // look as follows: [a, b, c, 0, 0, 0, 0, 0]
        let mut buffer: u8 = 0;

        for &b in self.as_ref() {
            // Write first u5 if we have to write two u5s this round. That only happens if the
            // buffer holds too many bits, so we don't have to combine buffer bits with new bits
            // from this rounds byte.
            if buffer_bits >= 5 {
                writer.write_u5(u5((buffer & 0b1111_1000) >> 3))?;
                buffer <<= 5;
                buffer_bits -= 5;
            }

            // Combine all bits from buffer with enough bits from this rounds byte so that they fill
            // a u5. Save reamining bits from byte to buffer.
            let from_buffer = buffer >> 3;
            let from_byte = b >> (3 + buffer_bits); // buffer_bits <= 4

            writer.write_u5(u5(from_buffer | from_byte))?;
            buffer = b << (5 - buffer_bits);
            buffer_bits += 3;
        }

        // There can be at most two u5s left in the buffer after processing all bytes, write them.
        if buffer_bits >= 5 {
            writer.write_u5(u5((buffer & 0b1111_1000) >> 3))?;
            buffer <<= 5;
            buffer_bits -= 5;
        }

        if buffer_bits != 0 {
            writer.write_u5(u5(buffer >> 3))?;
        }

        Ok(())
    }
}

impl<T: AsRef<[u8]>> Base32Len for T {
    fn base32_len(&self) -> usize {
        let bits = self.as_ref().len() * 8;
        if bits % 5 == 0 {
            bits / 5
        } else {
            bits / 5 + 1
        }
    }
}

/// A trait to convert between u8 arrays and u5 arrays without changing the content of the elements,
/// but checking that they are in range.
pub trait CheckBase32<T: AsRef<[u5]>> {
    /// Error type if conversion fails
    type Err;

    /// Check if all values are in range and return array-like struct of `u5` values
    fn check_base32(self) -> Result<T, Self::Err>;
}

impl<T: AsRef<[u8]>> CheckBase32<Vec<u5>> for T {
    type Err = Error;

    fn check_base32(self) -> Result<Vec<u5>, Self::Err> {
        self.as_ref()
            .iter()
            .map(|x| u5::try_from_u8(*x))
            .collect::<Result<Vec<u5>, Error>>()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Case {
    Upper,
    Lower,
    None,
}

/// Check if the HRP is valid. Returns the case of the HRP, if any.
///
/// # Errors
/// * **MixedCase**: If the HRP contains both uppercase and lowercase characters.
/// * **InvalidChar**: If the HRP contains any non-ASCII characters (outside 33..=126).
/// * **InvalidLength**: If the HRP is outside 1..83 characters long.
fn check_hrp(hrp: &str) -> Result<Case, Error> {
    if hrp.is_empty() || hrp.len() > 83 {
        return Err(Error::InvalidLength);
    }

    let mut has_lower: bool = false;
    let mut has_upper: bool = false;
    for b in hrp.bytes() {
        // Valid subset of ASCII
        if !(33..=126).contains(&b) {
            return Err(Error::InvalidChar(b as char));
        }

        if (b'a'..=b'z').contains(&b) {
            has_lower = true;
        } else if (b'A'..=b'Z').contains(&b) {
            has_upper = true;
        };

        if has_lower && has_upper {
            return Err(Error::MixedCase);
        }
    }

    Ok(match (has_upper, has_lower) {
        (true, false) => Case::Upper,
        (false, true) => Case::Lower,
        (false, false) => Case::None,
        (true, true) => unreachable!(),
    })
}

/// Encode a bech32 payload to an [fmt::Write].
/// This method is intended for implementing traits from [std::fmt].
///
/// # Errors
/// * If [check_hrp] returns an error for the given HRP.
/// # Deviations from standard
/// * No length limits are enforced for the data part
pub fn encode_to_fmt<T: AsRef<[u5]>>(
    fmt: &mut fmt::Write,
    hrp: &str,
    data: T,
    variant: Variant,
) -> Result<fmt::Result, Error> {
    let hrp_lower = match check_hrp(hrp)? {
        Case::Upper => Cow::Owned(hrp.to_lowercase()),
        Case::Lower | Case::None => Cow::Borrowed(hrp),
    };

    match Bech32Writer::new(&hrp_lower, variant, fmt) {
        Ok(mut writer) => {
            Ok(writer.write(data.as_ref()).and_then(|_| {
                // Finalize manually to avoid panic on drop if write fails
                writer.finalize()
            }))
        }
        Err(e) => Ok(Err(e)),
    }
}

/// Encode a bech32 payload without a checksum to an [fmt::Write].
/// This method is intended for implementing traits from [std::fmt].
///
/// # Errors
/// * If [check_hrp] returns an error for the given HRP.
/// # Deviations from standard
/// * No length limits are enforced for the data part
pub fn encode_without_checksum_to_fmt<T: AsRef<[u5]>>(
    fmt: &mut fmt::Write,
    hrp: &str,
    data: T,
) -> Result<fmt::Result, Error> {
    let hrp = match check_hrp(hrp)? {
        Case::Upper => Cow::Owned(hrp.to_lowercase()),
        Case::Lower | Case::None => Cow::Borrowed(hrp),
    };

    if let Err(e) = fmt.write_str(&hrp) {
        return Ok(Err(e));
    }
    if let Err(e) = fmt.write_char(SEP) {
        return Ok(Err(e));
    }
    for b in data.as_ref() {
        if let Err(e) = fmt.write_char(b.to_char()) {
            return Ok(Err(e));
        }
    }
    Ok(Ok(()))
}

/// Used for encode/decode operations for the two variants of Bech32
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Variant {
    /// The original Bech32 described in [BIP-0173](https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki)
    Bech32,
    /// The improved Bech32m variant described in [BIP-0350](https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki)
    Bech32m,
}

const BECH32_CONST: u32 = 1;
const BECH32M_CONST: u32 = 0x2bc8_30a3;

impl Variant {
    // Produce the variant based on the remainder of the polymod operation
    fn from_remainder(c: u32) -> Option<Self> {
        match c {
            BECH32_CONST => Some(Variant::Bech32),
            BECH32M_CONST => Some(Variant::Bech32m),
            _ => None,
        }
    }

    fn constant(self) -> u32 {
        match self {
            Variant::Bech32 => BECH32_CONST,
            Variant::Bech32m => BECH32M_CONST,
        }
    }
}

/// Encode a bech32 payload to string.
///
/// # Errors
/// * If [check_hrp] returns an error for the given HRP.
/// # Deviations from standard
/// * No length limits are enforced for the data part
pub fn encode<T: AsRef<[u5]>>(hrp: &str, data: T, variant: Variant) -> Result<String, Error> {
    let mut buf = String::new();
    encode_to_fmt(&mut buf, hrp, data, variant)?.unwrap();
    Ok(buf)
}

/// Encode a bech32 payload to string without the checksum.
///
/// # Errors
/// * If [check_hrp] returns an error for the given HRP.
/// # Deviations from standard
/// * No length limits are enforced for the data part
pub fn encode_without_checksum<T: AsRef<[u5]>>(hrp: &str, data: T) -> Result<String, Error> {
    let mut buf = String::new();
    encode_without_checksum_to_fmt(&mut buf, hrp, data)?.unwrap();
    Ok(buf)
}

/// Decode a bech32 string into the raw HRP and the data bytes.
///
/// Returns the HRP in lowercase, the data with the checksum removed, and the encoding.
pub fn decode(s: &str) -> Result<(String, Vec<u5>, Variant), Error> {
    let (hrp_lower, mut data) = split_and_decode(s)?;
    if data.len() < CHECKSUM_LENGTH {
        return Err(Error::InvalidLength);
    }

    // Ensure checksum
    match verify_checksum(hrp_lower.as_bytes(), &data) {
        Some(variant) => {
            // Remove checksum from data payload
            data.truncate(data.len() - CHECKSUM_LENGTH);

            Ok((hrp_lower, data, variant))
        }
        None => Err(Error::InvalidChecksum),
    }
}

/// Decode a bech32 string into the raw HRP and the data bytes, assuming no checksum.
///
/// Returns the HRP in lowercase and the data.
pub fn decode_without_checksum(s: &str) -> Result<(String, Vec<u5>), Error> {
    split_and_decode(s)
}

/// Decode a bech32 string into the raw HRP and the `u5` data.
fn split_and_decode(s: &str) -> Result<(String, Vec<u5>), Error> {
    // Split at separator and check for two pieces
    let (raw_hrp, raw_data) = match s.rfind(SEP) {
        None => return Err(Error::MissingSeparator),
        Some(sep) => {
            let (hrp, data) = s.split_at(sep);
            (hrp, &data[1..])
        }
    };

    let mut case = check_hrp(raw_hrp)?;
    let hrp_lower = match case {
        Case::Upper => raw_hrp.to_lowercase(),
        // already lowercase
        Case::Lower | Case::None => String::from(raw_hrp),
    };

    // Check data payload
    let data = raw_data
        .chars()
        .map(|c| {
            // Only check if c is in the ASCII range, all invalid ASCII
            // characters have the value -1 in CHARSET_REV (which covers
            // the whole ASCII range) and will be filtered out later.
            if !c.is_ascii() {
                return Err(Error::InvalidChar(c));
            }

            if c.is_lowercase() {
                match case {
                    Case::Upper => return Err(Error::MixedCase),
                    Case::None => case = Case::Lower,
                    Case::Lower => {}
                }
            } else if c.is_uppercase() {
                match case {
                    Case::Lower => return Err(Error::MixedCase),
                    Case::None => case = Case::Upper,
                    Case::Upper => {}
                }
            }

            // c should be <128 since it is in the ASCII range, CHARSET_REV.len() == 128
            let num_value = CHARSET_REV[c as usize];

            if !(0..=31).contains(&num_value) {
                return Err(Error::InvalidChar(c));
            }

            Ok(u5::try_from_u8(num_value as u8).expect("range checked above, num_value <= 31"))
        })
        .collect::<Result<Vec<u5>, Error>>()?;

    Ok((hrp_lower, data))
}

fn verify_checksum(hrp: &[u8], data: &[u5]) -> Option<Variant> {
    let mut exp = hrp_expand(hrp);
    exp.extend_from_slice(data);
    Variant::from_remainder(polymod(&exp))
}

fn hrp_expand(hrp: &[u8]) -> Vec<u5> {
    let mut v: Vec<u5> = Vec::new();
    for b in hrp {
        v.push(u5::try_from_u8(*b >> 5).expect("can't be out of range, max. 7"));
    }
    v.push(u5::try_from_u8(0).unwrap());
    for b in hrp {
        v.push(u5::try_from_u8(*b & 0x1f).expect("can't be out of range, max. 31"));
    }
    v
}

fn polymod(values: &[u5]) -> u32 {
    let mut chk: u32 = 1;
    let mut b: u8;
    for v in values {
        b = (chk >> 25) as u8;
        chk = (chk & 0x01ff_ffff) << 5 ^ (u32::from(*v.as_ref()));

        for (i, item) in GEN.iter().enumerate() {
            if (b >> i) & 1 == 1 {
                chk ^= item;
            }
        }
    }
    chk
}

/// Human-readable part and data part separator
const SEP: char = '1';

/// Encoding character set. Maps data value -> char
const CHARSET: [char; 32] = [
    'q', 'p', 'z', 'r', 'y', '9', 'x', '8', //  +0
    'g', 'f', '2', 't', 'v', 'd', 'w', '0', //  +8
    's', '3', 'j', 'n', '5', '4', 'k', 'h', // +16
    'c', 'e', '6', 'm', 'u', 'a', '7', 'l', // +24
];

/// Reverse character set. Maps ASCII byte -> CHARSET index on [0,31]
const CHARSET_REV: [i8; 128] = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
    15, -1, 10, 17, 21, 20, 26, 30, 7, 5, -1, -1, -1, -1, -1, -1, -1, 29, -1, 24, 13, 25, 9, 8, 23,
    -1, 18, 22, 31, 27, 19, -1, 1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1, -1, -1, -1, -1, -1, 29,
    -1, 24, 13, 25, 9, 8, 23, -1, 18, 22, 31, 27, 19, -1, 1, 0, 3, 16, 11, 28, 12, 14, 6, 4, 2, -1,
    -1, -1, -1, -1,
];

/// Generator coefficients
const GEN: [u32; 5] = [
    0x3b6a_57b2,
    0x2650_8e6d,
    0x1ea1_19fa,
    0x3d42_33dd,
    0x2a14_62b3,
];

/// Error types for Bech32 encoding / decoding
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Error {
    /// String does not contain the separator character
    MissingSeparator,
    /// The checksum does not match the rest of the data
    InvalidChecksum,
    /// The data or human-readable part is too long or too short
    InvalidLength,
    /// Some part of the string contains an invalid character
    InvalidChar(char),
    /// Some part of the data has an invalid value
    InvalidData(u8),
    /// The bit conversion failed due to a padding issue
    InvalidPadding,
    /// The whole string must be of one case
    MixedCase,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MissingSeparator => write!(f, "missing human-readable separator, \"{}\"", SEP),
            Error::InvalidChecksum => write!(f, "invalid checksum"),
            Error::InvalidLength => write!(f, "invalid length"),
            Error::InvalidChar(n) => write!(f, "invalid character (code={})", n),
            Error::InvalidData(n) => write!(f, "invalid data point ({})", n),
            Error::InvalidPadding => write!(f, "invalid padding"),
            Error::MixedCase => write!(f, "mixed-case strings not allowed"),
        }
    }
}

#[cfg(any(feature = "std", test))]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingSeparator => "missing human-readable separator",
            Error::InvalidChecksum => "invalid checksum",
            Error::InvalidLength => "invalid length",
            Error::InvalidChar(_) => "invalid character",
            Error::InvalidData(_) => "invalid data point",
            Error::InvalidPadding => "invalid padding",
            Error::MixedCase => "mixed-case strings not allowed",
        }
    }
}

/// Convert between bit sizes
///
/// # Errors
/// * `Error::InvalidData` if any element of `data` is out of range
/// * `Error::InvalidPadding` if `pad == false` and the padding bits are not `0`
///
/// # Panics
/// Function will panic if attempting to convert `from` or `to` a bit size that
/// is 0 or larger than 8 bits.
///
/// # Examples
///
/// ```rust
/// use bech32::convert_bits;
/// let base5 = convert_bits(&[0xff], 8, 5, true);
/// assert_eq!(base5.unwrap(), vec![0x1f, 0x1c]);
/// ```
pub fn convert_bits<T>(data: &[T], from: u32, to: u32, pad: bool) -> Result<Vec<u8>, Error>
where
    T: Into<u8> + Copy,
{
    if from > 8 || to > 8 || from == 0 || to == 0 {
        panic!("convert_bits `from` and `to` parameters 0 or greater than 8");
    }
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    let mut ret: Vec<u8> = Vec::new();
    let maxv: u32 = (1 << to) - 1;
    for value in data {
        let v: u32 = u32::from(Into::<u8>::into(*value));
        if (v >> from) != 0 {
            // Input value exceeds `from` bit size
            return Err(Error::InvalidData(v as u8));
        }
        acc = (acc << from) | v;
        bits += from;
        while bits >= to {
            bits -= to;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }
    if pad {
        if bits > 0 {
            ret.push(((acc << (to - bits)) & maxv) as u8);
        }
    } else if bits >= from || ((acc << (to - bits)) & maxv) != 0 {
        return Err(Error::InvalidPadding);
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getters() {
        let decoded = decode("BC1SW50QA3JX3S").unwrap();
        let data = [16, 14, 20, 15, 0].check_base32().unwrap();
        assert_eq!(&decoded.0, "bc");
        assert_eq!(decoded.1, data.as_slice());
    }

    #[test]
    fn valid_checksum() {
        let strings: Vec<&str> = vec!(
            // Bech32
            "A12UEL5L",
            "an83characterlonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1tt5tgs",
            "abcdef1qpzry9x8gf2tvdw0s3jn54khce6mua7lmqqqxw",
            "11qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqc8247j",
            "split1checkupstagehandshakeupstreamerranterredcaperred2y9e3w",
            // Bech32m
            "A1LQFN3A",
            "a1lqfn3a",
            "an83characterlonghumanreadablepartthatcontainsthetheexcludedcharactersbioandnumber11sg7hg6",
            "abcdef1l7aum6echk45nj3s0wdvt2fg8x9yrzpqzd3ryx",
            "11llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllludsr8",
            "split1checkupstagehandshakeupstreamerranterredcaperredlc445v",
            "?1v759aa",
        );
        for s in strings {
            match decode(s) {
                Ok((hrp, payload, variant)) => {
                    let encoded = encode(&hrp, payload, variant).unwrap();
                    assert_eq!(s.to_lowercase(), encoded.to_lowercase());
                }
                Err(e) => panic!("Did not decode: {:?} Reason: {:?}", s, e),
            }
        }
    }

    #[test]
    fn invalid_strings() {
        let pairs: Vec<(&str, Error)> = vec!(
            (" 1nwldj5",
                Error::InvalidChar(' ')),
            ("abc1\u{2192}axkwrx",
                Error::InvalidChar('\u{2192}')),
            ("an84characterslonghumanreadablepartthatcontainsthenumber1andtheexcludedcharactersbio1569pvx",
                Error::InvalidLength),
            ("pzry9x0s0muk",
                Error::MissingSeparator),
            ("1pzry9x0s0muk",
                Error::InvalidLength),
            ("x1b4n0q5v",
                Error::InvalidChar('b')),
            ("ABC1DEFGOH",
                Error::InvalidChar('O')),
            ("li1dgmt3",
                Error::InvalidLength),
            ("de1lg7wt\u{ff}",
                Error::InvalidChar('\u{ff}')),
            ("\u{20}1xj0phk",
                Error::InvalidChar('\u{20}')),
            ("\u{7F}1g6xzxy",
                Error::InvalidChar('\u{7F}')),
            ("an84characterslonghumanreadablepartthatcontainsthetheexcludedcharactersbioandnumber11d6pts4",
                Error::InvalidLength),
            ("qyrz8wqd2c9m",
                Error::MissingSeparator),
            ("1qyrz8wqd2c9m",
                Error::InvalidLength),
            ("y1b0jsk6g",
                Error::InvalidChar('b')),
            ("lt1igcx5c0",
                Error::InvalidChar('i')),
            ("in1muywd",
                Error::InvalidLength),
            ("mm1crxm3i",
                Error::InvalidChar('i')),
            ("au1s5cgom",
                Error::InvalidChar('o')),
            ("M1VUXWEZ",
                Error::InvalidChecksum),
            ("16plkw9",
                Error::InvalidLength),
            ("1p2gdwpf",
                Error::InvalidLength),
            ("bc1p2",
                Error::InvalidLength),
        );
        for p in pairs {
            let (s, expected_error) = p;
            match decode(s) {
                Ok(_) => panic!("Should be invalid: {:?}", s),
                Err(e) => assert_eq!(e, expected_error, "testing input '{}'", s),
            }
        }
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn valid_conversion() {
        // Set of [data, from_bits, to_bits, pad, result]
        let tests: Vec<(Vec<u8>, u32, u32, bool, Vec<u8>)> = vec![
            (vec![0x01], 1, 1, true, vec![0x01]),
            (vec![0x01, 0x01], 1, 1, true, vec![0x01, 0x01]),
            (vec![0x01], 8, 8, true, vec![0x01]),
            (vec![0x01], 8, 4, true, vec![0x00, 0x01]),
            (vec![0x01], 8, 2, true, vec![0x00, 0x00, 0x00, 0x01]),
            (
                vec![0x01],
                8,
                1,
                true,
                vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01],
            ),
            (vec![0xff], 8, 5, true, vec![0x1f, 0x1c]),
            (vec![0x1f, 0x1c], 5, 8, false, vec![0xff]),
        ];
        for t in tests {
            let (data, from_bits, to_bits, pad, expected_result) = t;
            let result = convert_bits(&data, from_bits, to_bits, pad);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected_result);
        }
    }

    #[test]
    fn invalid_conversion() {
        // Set of [data, from_bits, to_bits, pad, expected error]
        let tests: Vec<(Vec<u8>, u32, u32, bool, Error)> = vec![
            (vec![0xff], 8, 5, false, Error::InvalidPadding),
            (vec![0x02], 1, 1, true, Error::InvalidData(0x02)),
        ];
        for t in tests {
            let (data, from_bits, to_bits, pad, expected_error) = t;
            let result = convert_bits(&data, from_bits, to_bits, pad);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), expected_error);
        }
    }

    #[test]
    fn convert_bits_invalid_bit_size() {
        use std::panic::{catch_unwind, set_hook, take_hook};

        let invalid = &[(0, 8), (5, 0), (9, 5), (8, 10), (0, 16)];

        for &(from, to) in invalid {
            set_hook(Box::new(|_| {}));
            let result = catch_unwind(|| {
                let _ = convert_bits(&[0], from, to, true);
            });
            let _ = take_hook();
            assert!(result.is_err());
        }
    }

    #[test]
    fn check_base32() {
        assert!([0u8, 1, 2, 30, 31].check_base32().is_ok());
        assert!([0u8, 1, 2, 30, 31, 32].check_base32().is_err());
        assert!([0u8, 1, 2, 30, 31, 255].check_base32().is_err());

        assert!([1u8, 2, 3, 4].check_base32().is_ok());
        assert_eq!(
            [30u8, 31, 35, 20].check_base32(),
            Err(Error::InvalidData(35))
        );
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            encode(
                "",
                vec![1u8, 2, 3, 4].check_base32().unwrap(),
                Variant::Bech32
            ),
            Err(Error::InvalidLength)
        );
    }

    #[test]
    fn from_base32() {
        assert_eq!(
            Vec::from_base32(&[0x1f, 0x1c].check_base32().unwrap()),
            Ok(vec![0xff])
        );
        assert_eq!(
            Vec::from_base32(&[0x1f, 0x1f].check_base32().unwrap()),
            Err(Error::InvalidPadding)
        );
    }

    #[test]
    fn to_base32() {
        assert_eq!([0xffu8].to_base32(), [0x1f, 0x1c].check_base32().unwrap());
    }

    #[test]
    fn reverse_charset() {
        fn get_char_value(c: char) -> i8 {
            let charset = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
            match charset.find(c.to_ascii_lowercase()) {
                Some(x) => x as i8,
                None => -1,
            }
        }

        let expected_rev_charset = (0u8..128)
            .map(|i| get_char_value(i as char))
            .collect::<Vec<_>>();

        assert_eq!(&(CHARSET_REV[..]), expected_rev_charset.as_slice());
    }

    #[test]
    fn write_with_checksum() {
        let hrp = "lnbc";
        let data = "Hello World!".as_bytes().to_base32();

        let mut written_str = String::new();
        {
            let mut writer = Bech32Writer::new(hrp, Variant::Bech32, &mut written_str).unwrap();
            writer.write(&data).unwrap();
            writer.finalize().unwrap();
        }

        let encoded_str = encode(hrp, data, Variant::Bech32).unwrap();

        assert_eq!(encoded_str, written_str);
    }

    #[test]
    fn write_without_checksum() {
        let hrp = "lnbc";
        let data = "Hello World!".as_bytes().to_base32();

        let mut written_str = String::new();
        {
            let mut writer = Bech32Writer::new(hrp, Variant::Bech32, &mut written_str).unwrap();
            writer.write(&data).unwrap();
        }

        let encoded_str = encode_without_checksum(hrp, data).unwrap();

        assert_eq!(
            encoded_str,
            written_str[..written_str.len() - CHECKSUM_LENGTH]
        );
    }

    #[test]
    fn write_with_checksum_on_drop() {
        let hrp = "lntb";
        let data = "Hello World!".as_bytes().to_base32();

        let mut written_str = String::new();
        {
            let mut writer = Bech32Writer::new(hrp, Variant::Bech32, &mut written_str).unwrap();
            writer.write(&data).unwrap();
        }

        let encoded_str = encode(hrp, data, Variant::Bech32).unwrap();

        assert_eq!(encoded_str, written_str);
    }

    #[test]
    fn roundtrip_without_checksum() {
        let hrp = "lnbc";
        let data = "Hello World!".as_bytes().to_base32();

        let encoded = encode_without_checksum(hrp, data.clone()).expect("failed to encode");
        let (decoded_hrp, decoded_data) =
            decode_without_checksum(&encoded).expect("failed to decode");

        assert_eq!(decoded_hrp, hrp);
        assert_eq!(decoded_data, data);
    }

    #[test]
    fn test_hrp_case() {
        // Tests for issue with HRP case checking being ignored for encoding
        let encoded_str = encode("HRP", [0x00, 0x00].to_base32(), Variant::Bech32).unwrap();

        assert_eq!(encoded_str, "hrp1qqqq40atq3");
    }
}
