#![feature(test)]
extern crate test;

use std::io::Write;
use test::Bencher;
use uuid::Uuid;

#[bench]
fn hyphenated(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 36];
        write!(&mut buffer as &mut [_], "{:x}", uuid.hyphenated()).unwrap();
        buffer
    });
}

#[bench]
fn simple(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 32];
        write!(&mut buffer as &mut [_], "{:x}", uuid.simple()).unwrap();
        buffer
    })
}

#[bench]
fn urn(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 36 + 9];
        write!(&mut buffer as &mut [_], "{:x}", uuid.urn()).unwrap();
        buffer
    })
}

#[bench]
fn encode_hyphen(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 36];
        uuid.hyphenated().encode_lower(&mut buffer);
        buffer
    });
}

#[bench]
fn encode_simple(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 32];
        uuid.simple().encode_lower(&mut buffer);
        buffer
    })
}

#[bench]
fn encode_urn(b: &mut Bencher) {
    let uuid = Uuid::parse_str("F9168C5E-CEB2-4faa-B6BF-329BF39FA1E4").unwrap();
    b.iter(|| {
        let mut buffer = [0_u8; 36 + 9];
        uuid.urn().encode_lower(&mut buffer);
        buffer
    })
}
