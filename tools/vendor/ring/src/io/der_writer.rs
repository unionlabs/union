// Copyright 2018 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use super::{der::*, writer::*, *};
use alloc::boxed::Box;

pub(crate) fn write_positive_integer(output: &mut dyn Accumulator, value: &Positive) {
    let first_byte = value.first_byte();
    let value = value.big_endian_without_leading_zero_as_input();
    write_tlv(output, Tag::Integer, |output| {
        if (first_byte & 0x80) != 0 {
            output.write_byte(0); // Disambiguate negative number.
        }
        write_copy(output, value)
    })
}

pub(crate) fn write_all(tag: Tag, write_value: &dyn Fn(&mut dyn Accumulator)) -> Box<[u8]> {
    let length = {
        let mut length = LengthMeasurement::zero();
        write_tlv(&mut length, tag, write_value);
        length
    };

    let mut output = Writer::with_capacity(length);
    write_tlv(&mut output, tag, write_value);

    output.into()
}

fn write_tlv<F>(output: &mut dyn Accumulator, tag: Tag, write_value: F)
where
    F: Fn(&mut dyn Accumulator),
{
    let length: usize = {
        let mut length = LengthMeasurement::zero();
        write_value(&mut length);
        length.into()
    };

    output.write_byte(tag as u8);
    if length < 0x80 {
        output.write_byte(length as u8);
    } else if length < 0x1_00 {
        output.write_byte(0x81);
        output.write_byte(length as u8);
    } else if length < 0x1_00_00 {
        output.write_byte(0x82);
        output.write_byte((length / 0x1_00) as u8);
        output.write_byte(length as u8);
    } else {
        unreachable!();
    };

    write_value(output);
}
