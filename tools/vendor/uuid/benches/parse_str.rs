#![feature(test)]
extern crate test;

use test::Bencher;
use uuid::Uuid;

#[bench]
fn parse_nil(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("00000000000000000000000000000000"));
}

#[bench]
fn parse_nil_hyphenated(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("00000000-0000-0000-0000-000000000000"));
}

#[bench]
fn parse_random(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("67e5504410b1426f9247bb680e5fe0c8"));
}

#[bench]
fn parse_random_hyphenated(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8"));
}

#[bench]
fn parse_urn(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("urn:uuid:67e55044-10b1-426f-9247-bb680e5fe0c8"));
}

#[bench]
fn parse_invalid_len(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("F9168C5E-CEB2-4faa-BBF-329BF39FA1E4"))
}

#[bench]
fn parse_invalid_character(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("F9168C5E-CEB2-4faa-BGBF-329BF39FA1E4"))
}

#[bench]
fn parse_invalid_group_len(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("01020304-1112-2122-3132-41424344"));
}

#[bench]
fn parse_invalid_groups(b: &mut Bencher) {
    b.iter(|| Uuid::parse_str("F9168C5E-CEB2-4faa-B6BFF329BF39FA1E4"));
}
