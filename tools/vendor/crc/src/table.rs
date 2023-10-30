use crate::util::*;

pub(crate) const fn crc8_table(width: u8, poly: u8, reflect: bool) -> [u8; 256] {
    let poly = if reflect {
        let poly = poly.reverse_bits();
        poly >> (8u8 - width)
    } else {
        poly << (8u8 - width)
    };

    let mut table = [0u8; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = crc8(poly, reflect, i as u8);
        i += 1;
    }
    table
}

pub(crate) const fn crc16_table(width: u8, poly: u16, reflect: bool) -> [u16; 256] {
    let poly = if reflect {
        let poly = poly.reverse_bits();
        poly >> (16u8 - width)
    } else {
        poly << (16u8 - width)
    };

    let mut table = [0u16; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = crc16(poly, reflect, i as u16);
        i += 1;
    }
    table
}

pub(crate) const fn crc32_table(width: u8, poly: u32, reflect: bool) -> [u32; 256] {
    let poly = if reflect {
        let poly = poly.reverse_bits();
        poly >> (32u8 - width)
    } else {
        poly << (32u8 - width)
    };

    let mut table = [0u32; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = crc32(poly, reflect, i as u32);
        i += 1;
    }
    table
}

pub(crate) const fn crc64_table(width: u8, poly: u64, reflect: bool) -> [u64; 256] {
    let poly = if reflect {
        let poly = poly.reverse_bits();
        poly >> (64u8 - width)
    } else {
        poly << (64u8 - width)
    };

    let mut table = [0u64; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = crc64(poly, reflect, i as u64);
        i += 1;
    }
    table
}

pub(crate) const fn crc128_table(width: u8, poly: u128, reflect: bool) -> [u128; 256] {
    let poly = if reflect {
        let poly = poly.reverse_bits();
        poly >> (128u8 - width)
    } else {
        poly << (128u8 - width)
    };

    let mut table = [0u128; 256];
    let mut i = 0;
    while i < table.len() {
        table[i] = crc128(poly, reflect, i as u128);
        i += 1;
    }
    table
}
