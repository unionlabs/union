// Copyright 2013 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Punycode ([RFC 3492](http://tools.ietf.org/html/rfc3492)) implementation.
//!
//! Since Punycode fundamentally works on unicode code points,
//! `encode` and `decode` take and return slices and vectors of `char`.
//! `encode_str` and `decode_to_string` provide convenience wrappers
//! that convert from and to Rust’s UTF-8 based `str` and `String` types.

use std::char;
use std::u32;

// Bootstring parameters for Punycode
static BASE: u32 = 36;
static T_MIN: u32 = 1;
static T_MAX: u32 = 26;
static SKEW: u32 = 38;
static DAMP: u32 = 700;
static INITIAL_BIAS: u32 = 72;
static INITIAL_N: u32 = 0x80;
static DELIMITER: char = '-';

#[inline]
fn adapt(mut delta: u32, num_points: u32, first_time: bool) -> u32 {
    delta /= if first_time { DAMP } else { 2 };
    delta += delta / num_points;
    let mut k = 0;
    while delta > ((BASE - T_MIN) * T_MAX) / 2 {
        delta /= BASE - T_MIN;
        k += BASE;
    }
    k + (((BASE - T_MIN + 1) * delta) / (delta + SKEW))
}

/// Convert Punycode to an Unicode `String`.
///
/// This is a convenience wrapper around `decode`.
#[inline]
pub fn decode_to_string(input: &str) -> Option<String> {
    decode(input).map(|chars| chars.into_iter().collect())
}

/// Convert Punycode to Unicode.
///
/// Return None on malformed input or overflow.
/// Overflow can only happen on inputs that take more than
/// 63 encoded bytes, the DNS limit on domain name labels.
pub fn decode(input: &str) -> Option<Vec<char>> {
    Some(Decoder::default().decode(input).ok()?.collect())
}

#[derive(Default)]
pub(crate) struct Decoder {
    insertions: Vec<(usize, char)>,
}

impl Decoder {
    /// Split the input iterator and return a Vec with insertions of encoded characters
    pub(crate) fn decode<'a>(&'a mut self, input: &'a str) -> Result<Decode<'a>, ()> {
        self.insertions.clear();
        // Handle "basic" (ASCII) code points.
        // They are encoded as-is before the last delimiter, if any.
        let (base, input) = match input.rfind(DELIMITER) {
            None => ("", input),
            Some(position) => (
                &input[..position],
                if position > 0 {
                    &input[position + 1..]
                } else {
                    input
                },
            ),
        };

        if !base.is_ascii() {
            return Err(());
        }

        let base_len = base.len();
        let mut length = base_len as u32;
        let mut code_point = INITIAL_N;
        let mut bias = INITIAL_BIAS;
        let mut i = 0;
        let mut iter = input.bytes();
        loop {
            let previous_i = i;
            let mut weight = 1;
            let mut k = BASE;
            let mut byte = match iter.next() {
                None => break,
                Some(byte) => byte,
            };

            // Decode a generalized variable-length integer into delta,
            // which gets added to i.
            loop {
                let digit = match byte {
                    byte @ b'0'..=b'9' => byte - b'0' + 26,
                    byte @ b'A'..=b'Z' => byte - b'A',
                    byte @ b'a'..=b'z' => byte - b'a',
                    _ => return Err(()),
                } as u32;
                if digit > (u32::MAX - i) / weight {
                    return Err(()); // Overflow
                }
                i += digit * weight;
                let t = if k <= bias {
                    T_MIN
                } else if k >= bias + T_MAX {
                    T_MAX
                } else {
                    k - bias
                };
                if digit < t {
                    break;
                }
                if weight > u32::MAX / (BASE - t) {
                    return Err(()); // Overflow
                }
                weight *= BASE - t;
                k += BASE;
                byte = match iter.next() {
                    None => return Err(()), // End of input before the end of this delta
                    Some(byte) => byte,
                };
            }

            bias = adapt(i - previous_i, length + 1, previous_i == 0);
            if i / (length + 1) > u32::MAX - code_point {
                return Err(()); // Overflow
            }

            // i was supposed to wrap around from length+1 to 0,
            // incrementing code_point each time.
            code_point += i / (length + 1);
            i %= length + 1;
            let c = match char::from_u32(code_point) {
                Some(c) => c,
                None => return Err(()),
            };

            // Move earlier insertions farther out in the string
            for (idx, _) in &mut self.insertions {
                if *idx >= i as usize {
                    *idx += 1;
                }
            }
            self.insertions.push((i as usize, c));
            length += 1;
            i += 1;
        }

        self.insertions.sort_by_key(|(i, _)| *i);
        Ok(Decode {
            base: base.chars(),
            insertions: &self.insertions,
            inserted: 0,
            position: 0,
            len: base_len + self.insertions.len(),
        })
    }
}

pub(crate) struct Decode<'a> {
    base: std::str::Chars<'a>,
    pub(crate) insertions: &'a [(usize, char)],
    inserted: usize,
    position: usize,
    len: usize,
}

impl<'a> Iterator for Decode<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.insertions.get(self.inserted) {
                Some((pos, c)) if *pos == self.position => {
                    self.inserted += 1;
                    self.position += 1;
                    return Some(*c);
                }
                _ => {}
            }
            if let Some(c) = self.base.next() {
                self.position += 1;
                return Some(c);
            } else if self.inserted >= self.insertions.len() {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len - self.position;
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for Decode<'a> {
    fn len(&self) -> usize {
        self.len - self.position
    }
}

/// Convert an Unicode `str` to Punycode.
///
/// This is a convenience wrapper around `encode`.
#[inline]
pub fn encode_str(input: &str) -> Option<String> {
    let mut buf = String::with_capacity(input.len());
    encode_into(input.chars(), &mut buf).ok().map(|()| buf)
}

/// Convert Unicode to Punycode.
///
/// Return None on overflow, which can only happen on inputs that would take more than
/// 63 encoded bytes, the DNS limit on domain name labels.
pub fn encode(input: &[char]) -> Option<String> {
    let mut buf = String::with_capacity(input.len());
    encode_into(input.iter().copied(), &mut buf)
        .ok()
        .map(|()| buf)
}

pub(crate) fn encode_into<I>(input: I, output: &mut String) -> Result<(), ()>
where
    I: Iterator<Item = char> + Clone,
{
    // Handle "basic" (ASCII) code points. They are encoded as-is.
    let (mut input_length, mut basic_length) = (0, 0);
    for c in input.clone() {
        input_length += 1;
        if c.is_ascii() {
            output.push(c);
            basic_length += 1;
        }
    }

    if basic_length > 0 {
        output.push('-')
    }
    let mut code_point = INITIAL_N;
    let mut delta = 0;
    let mut bias = INITIAL_BIAS;
    let mut processed = basic_length;
    while processed < input_length {
        // All code points < code_point have been handled already.
        // Find the next larger one.
        let min_code_point = input
            .clone()
            .map(|c| c as u32)
            .filter(|&c| c >= code_point)
            .min()
            .unwrap();
        if min_code_point - code_point > (u32::MAX - delta) / (processed + 1) {
            return Err(()); // Overflow
        }
        // Increase delta to advance the decoder’s <code_point,i> state to <min_code_point,0>
        delta += (min_code_point - code_point) * (processed + 1);
        code_point = min_code_point;
        for c in input.clone() {
            let c = c as u32;
            if c < code_point {
                delta += 1;
                if delta == 0 {
                    return Err(()); // Overflow
                }
            }
            if c == code_point {
                // Represent delta as a generalized variable-length integer:
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = if k <= bias {
                        T_MIN
                    } else if k >= bias + T_MAX {
                        T_MAX
                    } else {
                        k - bias
                    };
                    if q < t {
                        break;
                    }
                    let value = t + ((q - t) % (BASE - t));
                    output.push(value_to_digit(value));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                output.push(value_to_digit(q));
                bias = adapt(delta, processed + 1, processed == basic_length);
                delta = 0;
                processed += 1;
            }
        }
        delta += 1;
        code_point += 1;
    }
    Ok(())
}

#[inline]
fn value_to_digit(value: u32) -> char {
    match value {
        0..=25 => (value as u8 + b'a') as char,       // a..z
        26..=35 => (value as u8 - 26 + b'0') as char, // 0..9
        _ => panic!(),
    }
}
