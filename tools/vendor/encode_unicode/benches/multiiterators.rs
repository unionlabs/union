// uses /usr/share/dict/ for text to convert to Vec<Utf*Char> and iterate over
#![cfg(all(unix, feature="std"))]
#![feature(test)]
extern crate test;
use test::{Bencher, black_box};
#[macro_use] extern crate lazy_static;
extern crate encode_unicode;
use encode_unicode::{CharExt, Utf8Char, Utf16Char, iter_bytes, iter_units};

static ENGLISH: &str = include_str!("/usr/share/dict/american-english");
// TODO find a big chinese file; `aptitude search '?provides(wordlist)'` didn't have one
lazy_static!{
    static ref UTF8CHARS: Vec<Utf8Char> = ENGLISH.chars().map(|c| c.to_utf8() ).collect();
    static ref UTF16CHARS: Vec<Utf16Char> = ENGLISH.chars().map(|c| c.to_utf16() ).collect();
}


#[bench]
fn utf16_split_all_single_mulititerator(b: &mut Bencher) {
    b.iter(|| {
        iter_units(black_box(&*UTF16CHARS)).for_each(|u| assert!(u != 0) );
    });
}
#[bench]
fn utf16_split_all_single_flatmap(b: &mut Bencher) {
    b.iter(|| {
        black_box(&*UTF16CHARS).iter().flat_map(|&u16c| u16c ).for_each(|u| assert!(u != 0) );
    });
}
#[bench]
fn utf16_split_all_single_cloned_flatten(b: &mut Bencher) {
    b.iter(|| {
        black_box(&*UTF16CHARS).iter().cloned().flatten().for_each(|u| assert!(u != 0) );
    });
}


#[bench]
fn utf8_split_mostly_ascii_multiiterator(b: &mut Bencher) {
    b.iter(|| {
        iter_bytes(black_box(&*UTF8CHARS)).for_each(|b| assert!(b != 0) );
    });
}
#[bench]
fn utf8_split_mostly_ascii_flatmap(b: &mut Bencher) {
    b.iter(|| {
        black_box(&*UTF8CHARS).iter().flat_map(|&u8c| u8c ).for_each(|b| assert!(b != 0) );
    });
}
#[bench]
fn utf8_split_mostly_ascii_cloned_flatten(b: &mut Bencher) {
    b.iter(|| {
        black_box(&*UTF8CHARS).iter().cloned().flatten().for_each(|b| assert!(b != 0) );
    });
}


#[bench]
fn utf8_extend_mostly_ascii_multiiterator(b: &mut Bencher) {
    b.iter(|| {
        let vec: Vec<u8> = iter_bytes(black_box(&*UTF8CHARS)).collect();
        assert_eq!(black_box(vec).len(), ENGLISH.len());
    });
}
#[bench]
fn utf8_extend_mostly_ascii_custom(b: &mut Bencher) {
    b.iter(|| {
        let vec: Vec<u8> = black_box(&*UTF8CHARS).iter().collect();
        assert_eq!(black_box(vec).len(), ENGLISH.len());
    });
}
#[bench]
fn utf8_extend_mostly_ascii_custom_str(b: &mut Bencher) {
    b.iter(|| {
        let vec: String = black_box(&*UTF8CHARS).iter().cloned().collect();
        assert_eq!(black_box(vec).len(), ENGLISH.len());
    });
}

#[bench]
fn utf16_extend_all_single_multiiterator(b: &mut Bencher) {
    b.iter(|| {
        let vec: Vec<u16> = iter_units(black_box(&*UTF16CHARS)).collect();
        assert!(black_box(vec).len() < ENGLISH.len());
    });
}
#[bench]
fn utf16_extend_all_single_custom(b: &mut Bencher) {
    b.iter(|| {
        let vec: Vec<u16> = black_box(&*UTF16CHARS).iter().collect();
        assert!(black_box(vec).len() < ENGLISH.len());
    });
}
