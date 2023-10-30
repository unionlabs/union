use std::collections::{HashMap, HashSet};
use std::iter::Iterator;

fn main() {
    println!("pub const INVALID_VALUE: u8 = 255;");

    // A-Z
    let standard_alphabet: Vec<u8> = (0x41..0x5B)
        // a-z
        .chain(0x61..0x7B)
        // 0-9
        .chain(0x30..0x3A)
        // +
        .chain(0x2B..0x2C)
        // /
        .chain(0x2F..0x30)
        .collect();
    print_encode_table(&standard_alphabet, "STANDARD_ENCODE", 0);
    print_decode_table(&standard_alphabet, "STANDARD_DECODE", 0);

    // A-Z
    let url_alphabet: Vec<u8> = (0x41..0x5B)
        // a-z
        .chain(0x61..0x7B)
        // 0-9
        .chain(0x30..0x3A)
        // -
        .chain(0x2D..0x2E)
        // _
        .chain(0x5F..0x60)
        .collect();
    print_encode_table(&url_alphabet, "URL_SAFE_ENCODE", 0);
    print_decode_table(&url_alphabet, "URL_SAFE_DECODE", 0);

    // ./0123456789
    let crypt_alphabet: Vec<u8> = (b'.'..(b'9' + 1))
        // A-Z
        .chain(b'A'..(b'Z' + 1))
        // a-z
        .chain(b'a'..(b'z' + 1))
        .collect();
    print_encode_table(&crypt_alphabet, "CRYPT_ENCODE", 0);
    print_decode_table(&crypt_alphabet, "CRYPT_DECODE", 0);

    // ./
    let bcrypt_alphabet: Vec<u8> = (b'.'..(b'/' + 1))
        // A-Z
        .chain(b'A'..(b'Z' + 1))
        // a-z
        .chain(b'a'..(b'z' + 1))
        // 0-9
        .chain(b'0'..(b'9' + 1))
        .collect();
    print_encode_table(&bcrypt_alphabet, "BCRYPT_ENCODE", 0);
    print_decode_table(&bcrypt_alphabet, "BCRYPT_DECODE", 0);

    // A-Z
    let imap_alphabet: Vec<u8> = (0x41..0x5B)
        // a-z
        .chain(0x61..0x7B)
        // 0-9
        .chain(0x30..0x3A)
        // +
        .chain(0x2B..0x2C)
        // ,
        .chain(0x2C..0x2D)
        .collect();
    print_encode_table(&imap_alphabet, "IMAP_MUTF7_ENCODE", 0);
    print_decode_table(&imap_alphabet, "IMAP_MUTF7_DECODE", 0);

    // '!' - '-'
    let binhex_alphabet: Vec<u8> = (0x21..0x2E)
        // 0-9
        .chain(0x30..0x3A)
        // @-N
        .chain(0x40..0x4F)
        // P-V
        .chain(0x50..0x57)
        // X-[
        .chain(0x58..0x5C)
        // `-f
        .chain(0x60..0x66)
        // h-m
        .chain(0x68..0x6E)
        // p-r
        .chain(0x70..0x73)
        .collect();
    print_encode_table(&binhex_alphabet, "BINHEX_ENCODE", 0);
    print_decode_table(&binhex_alphabet, "BINHEX_DECODE", 0);
}

fn print_encode_table(alphabet: &[u8], const_name: &str, indent_depth: usize) {
    check_alphabet(alphabet);
    println!("#[rustfmt::skip]");
    println!(
        "{:width$}pub const {}: &[u8; 64] = &[",
        "",
        const_name,
        width = indent_depth
    );

    for (i, b) in alphabet.iter().enumerate() {
        println!(
            "{:width$}{}, // input {} (0x{:X}) => '{}' (0x{:X})",
            "",
            b,
            i,
            i,
            String::from_utf8(vec![*b as u8]).unwrap(),
            b,
            width = indent_depth + 4
        );
    }

    println!("{:width$}];", "", width = indent_depth);
}

fn print_decode_table(alphabet: &[u8], const_name: &str, indent_depth: usize) {
    check_alphabet(alphabet);
    // map of alphabet bytes to 6-bit morsels
    let mut input_to_morsel = HashMap::<u8, u8>::new();

    // standard base64 alphabet bytes, in order
    for (morsel, ascii_byte) in alphabet.iter().enumerate() {
        // truncation cast is fine here
        let _ = input_to_morsel.insert(*ascii_byte, morsel as u8);
    }

    println!("#[rustfmt::skip]");
    println!(
        "{:width$}pub const {}: &[u8; 256] = &[",
        "",
        const_name,
        width = indent_depth
    );
    for ascii_byte in 0..256 {
        let (value, comment) = match input_to_morsel.get(&(ascii_byte as u8)) {
            None => (
                "INVALID_VALUE".to_string(),
                format!("input {} (0x{:X})", ascii_byte, ascii_byte),
            ),
            Some(v) => (
                format!("{}", *v),
                format!(
                    "input {} (0x{:X} char '{}') => {} (0x{:X})",
                    ascii_byte,
                    ascii_byte,
                    String::from_utf8(vec![ascii_byte as u8]).unwrap(),
                    *v,
                    *v
                ),
            ),
        };

        println!(
            "{:width$}{}, // {}",
            "",
            value,
            comment,
            width = indent_depth + 4
        );
    }
    println!("{:width$}];", "", width = indent_depth);
}

fn check_alphabet(alphabet: &[u8]) {
    // ensure all characters are distinct
    assert_eq!(64, alphabet.len());
    let mut set: HashSet<u8> = HashSet::new();
    set.extend(alphabet);
    assert_eq!(64, set.len());

    // must be ASCII to be valid as single UTF-8 bytes
    for &b in alphabet {
        assert!(b <= 0x7F_u8);
        // = is assumed to be padding, so cannot be used as a symbol
        assert_ne!(b'=', b);
    }
}
