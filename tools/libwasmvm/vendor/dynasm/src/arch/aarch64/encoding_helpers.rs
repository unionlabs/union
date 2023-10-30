use crate::common::{bitmask, bitmask64};

pub fn encode_floating_point_immediate(value: f32) -> Option<u8> {
    // floating point ARM immediates are encoded as
    // abcdefgh => aBbbbbbc defgh000 00000000 00000000
    // where B = !b
    // which means we can just slice out "a" and "bcdefgh" and assume the rest was correct

    let bits = value.to_bits();

    let check = (bits >> 25) & 0x3F;
    if (check == 0b10_0000 || check == 0b01_1111) && (bits & 0x7_FFFF) == 0 {
        Some((((bits >> 24) & 0x80) | ((bits >> 19) & 0x7F)) as u8)
    } else {
        None
    }
}

pub fn encode_logical_immediate_32bit(value: u32) -> Option<u16> {
    let transitions = value ^ value.rotate_right(1);
    let element_size = (64u32).checked_div(transitions.count_ones())?;

    // confirm that the elements are identical
    if value != value.rotate_left(element_size) {
        return None;
    }

    let element = value & bitmask(element_size as u8);
    let ones = element.count_ones();
    let imms = (!((element_size << 1) - 1) & 0x3F) | (ones - 1);

    let immr = if (element & 1) != 0 {
        ones - (!element).trailing_zeros()
    } else {
        element_size - element.trailing_zeros()
    };

    Some(((immr as u16) << 6) | (imms as u16))
}

pub fn encode_logical_immediate_64bit(value: u64) -> Option<u16> {
    let transitions = value ^ value.rotate_right(1);
    let element_size = (128u32).checked_div(transitions.count_ones())?;

    // confirm that the elements are identical
    if value != value.rotate_left(element_size) {
        return None;
    }

    let element = value & bitmask64(element_size as u8);
    let ones = element.count_ones();
    let imms = (!((element_size << 1) - 1) & 0x7F) | (ones - 1);

    let immr = if (element & 1) != 0 {
        ones - (!element).trailing_zeros()
    } else {
        element_size - element.trailing_zeros()
    };

    let n = imms & 0x40 == 0;
    let imms = imms & 0x3F;

    Some(((n as u16) << 12) | ((immr as u16) << 6) | (imms as u16))
}

pub fn encode_stretched_immediate(value: u64) -> Option<u32> {
    // ensure the number is formatted correctly
    let mut test = value & 0x0101_0101_0101_0101;
    test |= test << 1;
    test |= test << 2;
    test |= test << 4;
    if test != value {
        return None;
    }

    // do bitwise magic
    let mut masked = value & 0x8040_2010_0804_0201;
    masked |= masked >> 32;
    masked |= masked >> 16;
    masked |= masked >> 8;
    let masked = masked as u32;
    Some(masked & 0xFF)
}

pub fn encode_wide_immediate_64bit(value: u64) -> Option<u32> {
    let offset = value.trailing_zeros() & 0b11_0000;
    let masked = 0xFFFF & (value >> offset);
    if (masked << offset) == value {
        Some((masked as u32) | (offset << 12))
    } else {
        None
    }
}

pub fn encode_wide_immediate_32bit(value: u32) -> Option<u32> {
    let offset = value.trailing_zeros() & 0b1_0000;
    let masked = 0xFFFF & (value >> offset);
    if (masked << offset) == value {
        Some((masked as u32) | (offset << 12))
    } else {
        None
    }
}
